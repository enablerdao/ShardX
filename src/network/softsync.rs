use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::time::interval;
use crate::error::Error;
use crate::network::protocol::{NetworkMessage, MessageType};

/// ピア情報
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// ピアのアドレス
    pub address: String,
    /// 最後に通信した時間
    pub last_seen: Instant,
    /// 接続状態
    pub connected: bool,
}

/// 同期メッセージ
#[derive(Debug, Clone)]
pub struct SyncMessage {
    /// メッセージID
    pub id: String,
    /// 送信者ID
    pub sender_id: String,
    /// タイムスタンプ
    pub timestamp: u64,
    /// データ
    pub data: Vec<u8>,
}

/// 軽量P2P同期プロトコル
pub struct SoftSync {
    /// ピア情報
    peers: HashMap<String, PeerInfo>,
    /// メッセージキュー
    message_queue: VecDeque<SyncMessage>,
    /// 最後に同期した時間
    last_sync: Instant,
    /// UDPソケット
    socket: UdpSocket,
    /// ノードID
    node_id: String,
}

impl SoftSync {
    /// 新しいSoftSyncを作成
    pub async fn new(bind_addr: &str, node_id: String) -> Result<Self, Error> {
        let socket = UdpSocket::bind(bind_addr).await
            .map_err(|e| Error::NetworkError(format!("Failed to bind UDP socket: {}", e)))?;
        
        Ok(Self {
            peers: HashMap::new(),
            message_queue: VecDeque::new(),
            last_sync: Instant::now(),
            socket,
            node_id,
        })
    }
    
    /// 同期ループを開始
    pub async fn start(&mut self) -> Result<(), Error> {
        // 0.08秒ごとに同期
        let mut sync_interval = interval(Duration::from_millis(80));
        
        loop {
            tokio::select! {
                _ = sync_interval.tick() => {
                    self.sync_with_peers().await?;
                }
                
                result = self.receive_message() => {
                    if let Ok((message, addr)) = result {
                        self.handle_message(message, addr).await?;
                    }
                }
            }
        }
    }
    
    /// ピアと同期
    async fn sync_with_peers(&mut self) -> Result<(), Error> {
        // 各ピアと同期
        let peers: Vec<(String, String)> = self.peers.iter()
            .filter(|(_, info)| info.connected)
            .map(|(id, info)| (id.clone(), info.address.clone()))
            .collect();
        
        for (peer_id, peer_addr) in peers {
            // 10バイトの軽量同期データを送信
            let sync_data = self.create_sync_data();
            self.send_to_peer(&peer_id, &peer_addr, sync_data).await?;
        }
        
        self.last_sync = Instant::now();
        Ok(())
    }
    
    /// 同期データを作成
    fn create_sync_data(&self) -> Vec<u8> {
        // 10バイトの軽量同期データを作成
        // 1-2バイト: プロトコルバージョン
        // 3-6バイト: 最新ブロックハッシュの一部
        // 7-8バイト: シャード情報
        // 9-10バイト: 負荷情報
        let mut data = vec![0; 10];
        
        // プロトコルバージョン
        data[0] = 1;
        data[1] = 0;
        
        // 最新ブロックハッシュの一部（ダミー）
        data[2] = 0xAB;
        data[3] = 0xCD;
        data[4] = 0xEF;
        data[5] = 0x12;
        
        // シャード情報（ダミー）
        data[6] = 10; // シャード数
        data[7] = 5;  // 担当シャードID
        
        // 負荷情報（ダミー）
        data[8] = 50; // CPU使用率
        data[9] = 40; // メモリ使用率
        
        data
    }
    
    /// ピアにデータを送信
    async fn send_to_peer(&self, peer_id: &str, peer_addr: &str, data: Vec<u8>) -> Result<(), Error> {
        // ハートビートメッセージを作成
        let message = NetworkMessage::new(
            MessageType::Heartbeat,
            self.node_id.clone(),
            peer_id.to_string(),
            data,
        );
        
        // シリアライズ
        let serialized = message.encode_to_vec();
        
        // UDPで送信
        let addr = peer_addr.parse()
            .map_err(|e| Error::NetworkError(format!("Invalid peer address: {}", e)))?;
        
        self.socket.send_to(&serialized, addr).await
            .map_err(|e| Error::NetworkError(format!("Failed to send message: {}", e)))?;
        
        Ok(())
    }
    
    /// メッセージを受信
    async fn receive_message(&self) -> Result<(NetworkMessage, String), Error> {
        let mut buf = vec![0; 4096];
        let (len, addr) = self.socket.recv_from(&mut buf).await
            .map_err(|e| Error::NetworkError(format!("Failed to receive message: {}", e)))?;
        
        // 受信データをデシリアライズ
        let message = NetworkMessage::decode(&buf[..len])
            .map_err(|e| Error::DeserializeError(e.to_string()))?;
        
        Ok((message, addr.to_string()))
    }
    
    /// メッセージを処理
    async fn handle_message(&mut self, message: NetworkMessage, addr: String) -> Result<(), Error> {
        // 送信者IDを取得
        let sender_id = message.sender_id.clone();
        
        // ピア情報を更新
        self.update_peer_info(&sender_id, &addr);
        
        // メッセージタイプに応じて処理
        match message.get_message_type() {
            MessageType::Heartbeat => {
                // ハートビートの処理
                self.handle_heartbeat(message).await?;
            }
            MessageType::SyncRequest => {
                // 同期要求の処理
                self.handle_sync_request(message).await?;
            }
            MessageType::SyncResponse => {
                // 同期応答の処理
                self.handle_sync_response(message).await?;
            }
            _ => {
                // その他のメッセージはキューに追加
                self.message_queue.push_back(SyncMessage {
                    id: message.message_id,
                    sender_id,
                    timestamp: message.timestamp,
                    data: message.payload,
                });
            }
        }
        
        Ok(())
    }
    
    /// ピア情報を更新
    fn update_peer_info(&mut self, peer_id: &str, addr: &str) {
        let now = Instant::now();
        
        self.peers.entry(peer_id.to_string())
            .and_modify(|info| {
                info.last_seen = now;
                info.connected = true;
            })
            .or_insert_with(|| PeerInfo {
                address: addr.to_string(),
                last_seen: now,
                connected: true,
            });
    }
    
    /// ハートビートを処理
    async fn handle_heartbeat(&mut self, message: NetworkMessage) -> Result<(), Error> {
        // ハートビートの内容を解析
        if message.payload.len() >= 10 {
            // プロトコルバージョンをチェック
            let protocol_version = (message.payload[0] as u16) << 8 | message.payload[1] as u16;
            if protocol_version != 0x0100 {
                // バージョンが一致しない場合は無視
                return Ok(());
            }
            
            // 必要に応じて同期要求を送信
            let need_sync = self.check_need_sync(&message.payload[2..6]);
            if need_sync {
                self.send_sync_request(&message.sender_id).await?;
            }
        }
        
        Ok(())
    }
    
    /// 同期が必要かチェック
    fn check_need_sync(&self, hash_part: &[u8]) -> bool {
        // 実際の実装では、ローカルの最新ハッシュと比較
        // ここではダミー実装
        hash_part[0] != 0xAB || hash_part[1] != 0xCD
    }
    
    /// 同期要求を送信
    async fn send_sync_request(&self, peer_id: &str) -> Result<(), Error> {
        if let Some(peer_info) = self.peers.get(peer_id) {
            // 同期要求メッセージを作成
            let message = NetworkMessage::new(
                MessageType::SyncRequest,
                self.node_id.clone(),
                peer_id.to_string(),
                vec![],
            );
            
            // シリアライズ
            let serialized = message.encode_to_vec();
            
            // UDPで送信
            let addr = peer_info.address.parse()
                .map_err(|e| Error::NetworkError(format!("Invalid peer address: {}", e)))?;
            
            self.socket.send_to(&serialized, addr).await
                .map_err(|e| Error::NetworkError(format!("Failed to send message: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// 同期要求を処理
    async fn handle_sync_request(&self, message: NetworkMessage) -> Result<(), Error> {
        // 同期応答を送信
        if let Some(peer_info) = self.peers.get(&message.sender_id) {
            // 同期データを作成
            let sync_data = self.create_full_sync_data();
            
            // 同期応答メッセージを作成
            let response = NetworkMessage::new(
                MessageType::SyncResponse,
                self.node_id.clone(),
                message.sender_id.clone(),
                sync_data,
            );
            
            // シリアライズ
            let serialized = response.encode_to_vec();
            
            // UDPで送信
            let addr = peer_info.address.parse()
                .map_err(|e| Error::NetworkError(format!("Invalid peer address: {}", e)))?;
            
            self.socket.send_to(&serialized, addr).await
                .map_err(|e| Error::NetworkError(format!("Failed to send message: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// 完全な同期データを作成
    fn create_full_sync_data(&self) -> Vec<u8> {
        // 実際の実装では、現在の状態の完全なスナップショットを作成
        // ここではダミー実装
        vec![0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56, 0x78, 0x90]
    }
    
    /// 同期応答を処理
    async fn handle_sync_response(&self, message: NetworkMessage) -> Result<(), Error> {
        // 同期データを適用
        // 実際の実装では、受信したデータを使用して状態を更新
        // ここではダミー実装
        
        Ok(())
    }
    
    /// ピアを追加
    pub fn add_peer(&mut self, peer_id: String, peer_addr: String) {
        self.peers.insert(peer_id, PeerInfo {
            address: peer_addr,
            last_seen: Instant::now(),
            connected: true,
        });
    }
    
    /// ピアを削除
    pub fn remove_peer(&mut self, peer_id: &str) {
        self.peers.remove(peer_id);
    }
    
    /// ピア一覧を取得
    pub fn get_peers(&self) -> Vec<(String, PeerInfo)> {
        self.peers.iter()
            .map(|(id, info)| (id.clone(), info.clone()))
            .collect()
    }
    
    /// 切断されたピアをクリーンアップ
    pub fn cleanup_disconnected_peers(&mut self) {
        let now = Instant::now();
        let timeout = Duration::from_secs(60); // 60秒間応答がないピアを切断
        
        self.peers.retain(|_, info| {
            if now.duration_since(info.last_seen) > timeout {
                info.connected = false;
                false // ピアを削除
            } else {
                true // ピアを保持
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;
    
    #[test]
    fn test_create_sync_data() {
        let rt = Runtime::new().unwrap();
        
        rt.block_on(async {
            let soft_sync = SoftSync::new("127.0.0.1:0", "node1".to_string()).await.unwrap();
            
            // 同期データを作成
            let data = soft_sync.create_sync_data();
            
            // データサイズを確認
            assert_eq!(data.len(), 10);
            
            // プロトコルバージョンを確認
            assert_eq!(data[0], 1);
            assert_eq!(data[1], 0);
        });
    }
    
    #[test]
    fn test_peer_management() {
        let rt = Runtime::new().unwrap();
        
        rt.block_on(async {
            let mut soft_sync = SoftSync::new("127.0.0.1:0", "node1".to_string()).await.unwrap();
            
            // ピアを追加
            soft_sync.add_peer("peer1".to_string(), "127.0.0.1:8000".to_string());
            soft_sync.add_peer("peer2".to_string(), "127.0.0.1:8001".to_string());
            
            // ピア数を確認
            assert_eq!(soft_sync.get_peers().len(), 2);
            
            // ピアを削除
            soft_sync.remove_peer("peer1");
            
            // ピア数を確認
            assert_eq!(soft_sync.get_peers().len(), 1);
            
            // 残りのピアを確認
            let peers = soft_sync.get_peers();
            assert_eq!(peers[0].0, "peer2");
            assert_eq!(peers[0].1.address, "127.0.0.1:8001");
        });
    }
}