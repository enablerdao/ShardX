use tokio::net::UdpSocket;
use std::net::SocketAddr;
use prost::Message;
use crate::error::Error;

/// ネットワークメッセージタイプ
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageType {
    /// トランザクション
    Transaction,
    /// シャード情報
    ShardInfo,
    /// ノード情報
    NodeInfo,
    /// ハートビート
    Heartbeat,
    /// 同期要求
    SyncRequest,
    /// 同期応答
    SyncResponse,
}

/// ネットワークメッセージ
#[derive(Clone, PartialEq, Message)]
pub struct NetworkMessage {
    /// メッセージタイプ
    #[prost(uint32, tag = "1")]
    pub message_type: u32,
    
    /// 送信者ID
    #[prost(string, tag = "2")]
    pub sender_id: String,
    
    /// 受信者ID（空の場合はブロードキャスト）
    #[prost(string, tag = "3")]
    pub recipient_id: String,
    
    /// メッセージID
    #[prost(string, tag = "4")]
    pub message_id: String,
    
    /// タイムスタンプ
    #[prost(uint64, tag = "5")]
    pub timestamp: u64,
    
    /// ペイロード
    #[prost(bytes, tag = "6")]
    pub payload: Vec<u8>,
}

impl NetworkMessage {
    /// 新しいNetworkMessageを作成
    pub fn new(
        message_type: MessageType,
        sender_id: String,
        recipient_id: String,
        payload: Vec<u8>,
    ) -> Self {
        let message_id = uuid::Uuid::new_v4().to_string();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            message_type: message_type as u32,
            sender_id,
            recipient_id,
            message_id,
            timestamp,
            payload,
        }
    }
    
    /// メッセージタイプを取得
    pub fn get_message_type(&self) -> MessageType {
        match self.message_type {
            0 => MessageType::Transaction,
            1 => MessageType::ShardInfo,
            2 => MessageType::NodeInfo,
            3 => MessageType::Heartbeat,
            4 => MessageType::SyncRequest,
            5 => MessageType::SyncResponse,
            _ => MessageType::Transaction,
        }
    }
}

/// Protocol Buffersシリアライザ
pub struct ProtobufSerializer;

impl ProtobufSerializer {
    /// 新しいProtobufSerializerを作成
    pub fn new() -> Self {
        Self
    }
    
    /// メッセージをシリアライズ
    pub fn serialize(&self, message: &NetworkMessage) -> Result<Vec<u8>, Error> {
        let mut buf = Vec::new();
        message.encode(&mut buf)
            .map_err(|e| Error::SerializeError(e.to_string()))?;
        Ok(buf)
    }
    
    /// メッセージをデシリアライズ
    pub fn deserialize(&self, data: &[u8]) -> Result<NetworkMessage, Error> {
        NetworkMessage::decode(data)
            .map_err(|e| Error::DeserializeError(e.to_string()))
    }
}

/// UDPベースのネットワークプロトコル
pub struct UdpProtocol {
    /// UDPソケット
    socket: UdpSocket,
    /// シリアライザ
    serializer: ProtobufSerializer,
}

impl UdpProtocol {
    /// 新しいUdpProtocolを作成
    pub async fn new(bind_addr: &str) -> Result<Self, Error> {
        let socket = UdpSocket::bind(bind_addr).await
            .map_err(|e| Error::NetworkError(format!("Failed to bind UDP socket: {}", e)))?;
        
        Ok(Self {
            socket,
            serializer: ProtobufSerializer::new(),
        })
    }
    
    /// メッセージを送信
    pub async fn send_message(&self, peer_addr: &str, message: NetworkMessage) -> Result<(), Error> {
        // Protocol Buffersでシリアライズ
        let data = self.serializer.serialize(&message)?;
        
        // UDPで送信
        let addr = peer_addr.parse::<SocketAddr>()
            .map_err(|e| Error::NetworkError(format!("Invalid peer address: {}", e)))?;
        
        self.socket.send_to(&data, addr).await
            .map_err(|e| Error::NetworkError(format!("Failed to send message: {}", e)))?;
        
        Ok(())
    }
    
    /// メッセージを受信
    pub async fn receive_message(&self) -> Result<(NetworkMessage, SocketAddr), Error> {
        let mut buf = vec![0; 4096];
        let (len, addr) = self.socket.recv_from(&mut buf).await
            .map_err(|e| Error::NetworkError(format!("Failed to receive message: {}", e)))?;
        
        // 受信データをデシリアライズ
        let message = self.serializer.deserialize(&buf[..len])?;
        
        Ok((message, addr))
    }
    
    /// メッセージをブロードキャスト
    pub async fn broadcast(&self, peers: &[String], message: NetworkMessage) -> Result<(), Error> {
        // ゴシッププロトコルで効率的にブロードキャスト
        let serialized = self.serializer.serialize(&message)?;
        
        // 並列送信
        let mut tasks = Vec::with_capacity(peers.len());
        
        for peer in peers {
            let socket = self.socket.clone();
            let data = serialized.clone();
            let peer_addr = peer.clone();
            
            tasks.push(tokio::spawn(async move {
                let addr = peer_addr.parse::<SocketAddr>()
                    .map_err(|e| Error::NetworkError(format!("Invalid peer address: {}", e)))?;
                
                socket.send_to(&data, addr).await
                    .map_err(|e| Error::NetworkError(format!("Failed to send message: {}", e)))?;
                
                Ok::<(), Error>(())
            }));
        }
        
        // 全ての送信を待機
        for task in tasks {
            task.await
                .map_err(|e| Error::InternalError(format!("Task join error: {}", e)))??;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;
    
    #[test]
    fn test_network_message_creation() {
        let message = NetworkMessage::new(
            MessageType::Transaction,
            "sender1".to_string(),
            "recipient1".to_string(),
            vec![1, 2, 3, 4],
        );
        
        assert_eq!(message.message_type, MessageType::Transaction as u32);
        assert_eq!(message.sender_id, "sender1");
        assert_eq!(message.recipient_id, "recipient1");
        assert_eq!(message.payload, vec![1, 2, 3, 4]);
        assert!(!message.message_id.is_empty());
        assert!(message.timestamp > 0);
    }
    
    #[test]
    fn test_serialization() {
        let serializer = ProtobufSerializer::new();
        
        let message = NetworkMessage::new(
            MessageType::Transaction,
            "sender1".to_string(),
            "recipient1".to_string(),
            vec![1, 2, 3, 4],
        );
        
        // シリアライズ
        let serialized = serializer.serialize(&message).unwrap();
        
        // デシリアライズ
        let deserialized = serializer.deserialize(&serialized).unwrap();
        
        // 元のメッセージと一致することを確認
        assert_eq!(deserialized, message);
    }
    
    #[test]
    fn test_udp_protocol() {
        let rt = Runtime::new().unwrap();
        
        rt.block_on(async {
            // サーバーとクライアントのUdpProtocolを作成
            let server = UdpProtocol::new("127.0.0.1:0").await.unwrap();
            let server_addr = server.socket.local_addr().unwrap().to_string();
            
            let client = UdpProtocol::new("127.0.0.1:0").await.unwrap();
            
            // テスト用のメッセージを作成
            let message = NetworkMessage::new(
                MessageType::Transaction,
                "client".to_string(),
                "server".to_string(),
                vec![1, 2, 3, 4],
            );
            
            // メッセージを送信
            let send_task = tokio::spawn(async move {
                client.send_message(&server_addr, message.clone()).await.unwrap();
                message
            });
            
            // メッセージを受信
            let (received_message, _) = server.receive_message().await.unwrap();
            let sent_message = send_task.await.unwrap();
            
            // 送信したメッセージと受信したメッセージが一致することを確認
            assert_eq!(received_message, sent_message);
        });
    }
}