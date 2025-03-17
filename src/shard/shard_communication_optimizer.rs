use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use log::{debug, error, info, warn};
use tokio::sync::mpsc;
use tokio::time;

use crate::error::Error;
use crate::shard::{ShardId, ShardInfo, ShardManager};
use crate::network::{NetworkMessage, MessageType, PeerInfo};
use crate::metrics::MetricsCollector;

/// シャード間通信最適化器
/// 
/// シャード間の通信を最適化し、効率的なメッセージ交換を実現する。
/// - メッセージバッチ処理
/// - 優先順位付け
/// - 圧縮
/// - キャッシング
/// - ルーティング最適化
pub struct ShardCommunicationOptimizer {
    /// シャードマネージャー
    shard_manager: Arc<ShardManager>,
    /// メッセージキュー
    message_queue: Arc<Mutex<HashMap<ShardId, VecDeque<NetworkMessage>>>>,
    /// 送信中のメッセージ
    sending: Arc<Mutex<HashSet<String>>>,
    /// バッチサイズ
    batch_size: usize,
    /// 最大バッチ数
    max_batches: usize,
    /// 最大待機時間（ミリ秒）
    max_wait_ms: u64,
    /// 最小バッチサイズ
    min_batch_size: usize,
    /// メトリクスコレクター
    metrics: Arc<MetricsCollector>,
    /// 最後の最適化時刻
    last_optimization: Arc<Mutex<Instant>>,
    /// 最適化間隔（秒）
    optimization_interval_secs: u64,
    /// メッセージキャッシュ
    message_cache: Arc<Mutex<lru::LruCache<String, Vec<u8>>>>,
    /// シャードルーティングテーブル
    routing_table: Arc<Mutex<HashMap<ShardId, Vec<ShardId>>>>,
    /// 実行中フラグ
    running: Arc<Mutex<bool>>,
    /// 圧縮閾値（バイト）
    compression_threshold: usize,
    /// 圧縮レベル（0-9）
    compression_level: u32,
}

/// メッセージバッチ
#[derive(Debug, Clone)]
pub struct MessageBatch {
    /// バッチID
    pub id: String,
    /// 送信先シャードID
    pub target_shard: ShardId,
    /// メッセージ
    pub messages: Vec<NetworkMessage>,
    /// 作成時刻
    pub created_at: Instant,
    /// 優先度
    pub priority: u8,
    /// 圧縮フラグ
    pub compressed: bool,
    /// 圧縮前のサイズ（バイト）
    pub original_size: usize,
    /// 圧縮後のサイズ（バイト）
    pub compressed_size: Option<usize>,
}

/// メッセージ優先度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    /// 低
    Low = 0,
    /// 通常
    Normal = 1,
    /// 高
    High = 2,
    /// 最高
    Critical = 3,
}

impl ShardCommunicationOptimizer {
    /// 新しいShardCommunicationOptimizerを作成
    pub fn new(
        shard_manager: Arc<ShardManager>,
        batch_size: usize,
        max_batches: usize,
        max_wait_ms: u64,
        cache_size: usize,
        metrics: Arc<MetricsCollector>,
    ) -> Self {
        let min_batch_size = batch_size / 4;
        
        Self {
            shard_manager,
            message_queue: Arc::new(Mutex::new(HashMap::new())),
            sending: Arc::new(Mutex::new(HashSet::new())),
            batch_size,
            max_batches,
            max_wait_ms,
            min_batch_size,
            metrics,
            last_optimization: Arc::new(Mutex::new(Instant::now())),
            optimization_interval_secs: 60,
            message_cache: Arc::new(Mutex::new(lru::LruCache::new(cache_size))),
            routing_table: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
            compression_threshold: 1024, // 1KB以上のメッセージを圧縮
            compression_level: 6, // 中程度の圧縮レベル
        }
    }
    
    /// メッセージを追加
    pub fn add_message(&self, message: NetworkMessage) -> Result<(), Error> {
        // メッセージキューに追加
        let mut message_queue = self.message_queue.lock().unwrap();
        
        // 送信先シャードのキューを取得または作成
        let queue = message_queue.entry(message.receiver.clone()).or_insert_with(VecDeque::new);
        
        // メッセージをキューに追加
        queue.push_back(message.clone());
        
        // メトリクスを更新
        self.metrics.increment_counter("messages_queued");
        self.metrics.set_gauge("message_queue_size", self.get_total_queue_size() as f64);
        
        Ok(())
    }
    
    /// メッセージバッチを作成
    pub fn create_batch(&self, target_shard: &ShardId) -> Option<MessageBatch> {
        let mut message_queue = self.message_queue.lock().unwrap();
        let sending = self.sending.lock().unwrap();
        
        // 対象シャードのキューを取得
        let queue = message_queue.get_mut(target_shard)?;
        
        if queue.is_empty() {
            return None;
        }
        
        // バッチに含めるメッセージを選択
        let mut batch_messages = Vec::with_capacity(self.batch_size);
        let mut batch_priority = 0;
        let mut original_size = 0;
        
        // 送信中でないメッセージを選択
        let mut i = 0;
        while i < queue.len() && batch_messages.len() < self.batch_size {
            let msg = queue.get(i).unwrap().clone();
            
            // 送信中でないか確認
            let message_id = format!("{}:{}", msg.sender, msg.timestamp.timestamp_nanos());
            if !sending.contains(&message_id) {
                // バッチに追加
                batch_messages.push(msg.clone());
                
                // 優先度を更新
                let msg_priority = self.get_message_priority(&msg);
                batch_priority = batch_priority.max(msg_priority as u8);
                
                // サイズを計算
                original_size += msg.data.len();
                
                // キューから削除
                queue.remove(i);
            } else {
                i += 1;
            }
        }
        
        if batch_messages.is_empty() {
            return None;
        }
        
        // バッチを作成
        let batch_id = format!("batch_{}", Instant::now().elapsed().as_nanos());
        
        // 圧縮が必要かチェック
        let compressed = original_size >= self.compression_threshold;
        let compressed_size = if compressed {
            // 圧縮を実行
            let compressed_data = self.compress_batch(&batch_messages);
            Some(compressed_data.len())
        } else {
            None
        };
        
        Some(MessageBatch {
            id: batch_id,
            target_shard: target_shard.clone(),
            messages: batch_messages,
            created_at: Instant::now(),
            priority: batch_priority,
            compressed,
            original_size,
            compressed_size,
        })
    }
    
    /// バッチを圧縮
    fn compress_batch(&self, messages: &[NetworkMessage]) -> Vec<u8> {
        // メッセージをシリアライズ
        let serialized = bincode::serialize(messages).unwrap_or_default();
        
        // 圧縮を実行
        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::new(self.compression_level));
        std::io::Write::write_all(&mut encoder, &serialized).unwrap_or_default();
        encoder.finish().unwrap_or_default()
    }
    
    /// バッチを解凍
    fn decompress_batch(&self, compressed_data: &[u8]) -> Result<Vec<NetworkMessage>, Error> {
        // 解凍を実行
        let mut decoder = flate2::read::ZlibDecoder::new(compressed_data);
        let mut decompressed = Vec::new();
        std::io::Read::read_to_end(&mut decoder, &mut decompressed)
            .map_err(|e| Error::DecompressionError(format!("Failed to decompress batch: {}", e)))?;
        
        // デシリアライズ
        bincode::deserialize(&decompressed)
            .map_err(|e| Error::DeserializationError(format!("Failed to deserialize batch: {}", e)))
    }
    
    /// メッセージの優先度を取得
    fn get_message_priority(&self, message: &NetworkMessage) -> MessagePriority {
        match message.message_type {
            MessageType::Consensus => MessagePriority::Critical,
            MessageType::Transaction => MessagePriority::High,
            MessageType::Block => MessagePriority::High,
            MessageType::CrossShardMessage => MessagePriority::High,
            MessageType::ShardInfo => MessagePriority::Normal,
            MessageType::PeerInfo => MessagePriority::Normal,
            MessageType::Heartbeat => MessagePriority::Low,
            MessageType::Discovery => MessagePriority::Low,
            _ => MessagePriority::Normal,
        }
    }
    
    /// メッセージ処理を開始
    pub async fn start_processing<F>(&self, sender: F) -> Result<(), Error>
    where
        F: Fn(MessageBatch) -> Result<(), Error> + Send + Sync + 'static,
    {
        // 既に実行中かチェック
        let mut running = self.running.lock().unwrap();
        if *running {
            return Err(Error::InvalidState("Shard communication optimizer is already running".to_string()));
        }
        
        *running = true;
        drop(running);
        
        // チャネルを作成
        let (batch_tx, mut batch_rx) = mpsc::channel(self.max_batches);
        
        // 最適化タスクを開始
        let message_queue = self.message_queue.clone();
        let sending = self.sending.clone();
        let metrics = self.metrics.clone();
        let last_optimization = self.last_optimization.clone();
        let optimization_interval_secs = self.optimization_interval_secs;
        let routing_table = self.routing_table.clone();
        let running = self.running.clone();
        let min_batch_size = self.min_batch_size;
        let max_wait_ms = self.max_wait_ms;
        let shard_manager = self.shard_manager.clone();
        
        // バッチ生成タスク
        tokio::spawn(async move {
            let mut last_batch_times: HashMap<ShardId, Instant> = HashMap::new();
            
            while *running.lock().unwrap() {
                // アクティブなシャードを取得
                let shards = shard_manager.get_active_shards();
                
                for shard in &shards {
                    // 最後のバッチ時間を取得または初期化
                    let last_batch_time = last_batch_times.entry(shard.id.clone()).or_insert_with(Instant::now);
                    
                    // バッチを作成
                    let batch_option = {
                        let self_ref = &self;
                        self_ref.create_batch(&shard.id)
                    };
                    
                    if let Some(batch) = batch_option {
                        // 送信中に追加
                        {
                            let mut sending = sending.lock().unwrap();
                            for msg in &batch.messages {
                                let message_id = format!("{}:{}", msg.sender, msg.timestamp.timestamp_nanos());
                                sending.insert(message_id);
                            }
                        }
                        
                        // バッチを送信
                        if let Err(e) = batch_tx.send(batch.clone()).await {
                            error!("Failed to send batch: {}", e);
                            
                            // 送信中から削除
                            let mut sending = sending.lock().unwrap();
                            for msg in &e.0.messages {
                                let message_id = format!("{}:{}", msg.sender, msg.timestamp.timestamp_nanos());
                                sending.remove(&message_id);
                            }
                        }
                        
                        // メトリクスを更新
                        metrics.increment_counter("message_batches_created");
                        
                        // 最後のバッチ時間を更新
                        *last_batch_time = Instant::now();
                    } else {
                        // 最小バッチサイズに達していない場合は待機
                        let queue_size = {
                            let message_queue = message_queue.lock().unwrap();
                            message_queue.get(&shard.id).map_or(0, |q| q.len())
                        };
                        
                        if queue_size >= min_batch_size || last_batch_time.elapsed().as_millis() as u64 >= max_wait_ms {
                            // 最適化を実行
                            let mut last_opt = last_optimization.lock().unwrap();
                            if last_opt.elapsed().as_secs() >= optimization_interval_secs {
                                drop(last_opt);
                                
                                // ルーティングテーブルを最適化
                                Self::optimize_routing_table(
                                    routing_table.clone(),
                                    shard_manager.clone(),
                                    metrics.clone(),
                                );
                                
                                // 最後の最適化時刻を更新
                                *last_optimization.lock().unwrap() = Instant::now();
                            }
                        }
                    }
                }
                
                // 少し待機
                time::sleep(Duration::from_millis(10)).await;
            }
        });
        
        // バッチ処理タスク
        let sending = self.sending.clone();
        let metrics = self.metrics.clone();
        let running = self.running.clone();
        
        tokio::spawn(async move {
            while *running.lock().unwrap() {
                // バッチを受信
                if let Some(batch) = batch_rx.recv().await {
                    // メトリクスを更新
                    metrics.increment_counter("message_batches_received");
                    metrics.observe_histogram("message_batch_size", batch.messages.len() as f64);
                    
                    // 処理関数のクローン
                    let sender_fn = sender.clone();
                    let sending = sending.clone();
                    let metrics = metrics.clone();
                    
                    // バッチを処理
                    tokio::spawn(async move {
                        let start_time = Instant::now();
                        
                        // バッチを送信
                        let result = sender_fn(batch.clone());
                        
                        match result {
                            Ok(_) => {
                                // 送信中から削除
                                let mut sending = sending.lock().unwrap();
                                for msg in &batch.messages {
                                    let message_id = format!("{}:{}", msg.sender, msg.timestamp.timestamp_nanos());
                                    sending.remove(&message_id);
                                }
                                
                                // メトリクスを更新
                                metrics.observe_histogram("message_batch_processing_time", start_time.elapsed().as_secs_f64());
                                metrics.increment_counter("message_batches_sent");
                                
                                // 圧縮率を計算
                                if let Some(compressed_size) = batch.compressed_size {
                                    let compression_ratio = batch.original_size as f64 / compressed_size as f64;
                                    metrics.observe_histogram("message_batch_compression_ratio", compression_ratio);
                                }
                            },
                            Err(e) => {
                                // エラーをログに記録
                                error!("Failed to send batch: {}", e);
                                
                                // 送信中から削除
                                let mut sending = sending.lock().unwrap();
                                for msg in &batch.messages {
                                    let message_id = format!("{}:{}", msg.sender, msg.timestamp.timestamp_nanos());
                                    sending.remove(&message_id);
                                }
                                
                                // メトリクスを更新
                                metrics.increment_counter("message_batches_failed");
                            },
                        }
                    });
                } else {
                    // チャネルが閉じられた場合は終了
                    break;
                }
            }
        });
        
        Ok(())
    }
    
    /// 処理を停止
    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }
    
    /// ルーティングテーブルを最適化
    fn optimize_routing_table(
        routing_table: Arc<Mutex<HashMap<ShardId, Vec<ShardId>>>>,
        shard_manager: Arc<ShardManager>,
        metrics: Arc<MetricsCollector>,
    ) {
        // アクティブなシャードを取得
        let shards = shard_manager.get_active_shards();
        
        // ルーティングテーブルを更新
        let mut routing_table = routing_table.lock().unwrap();
        
        // 古いエントリを削除
        let active_shard_ids: HashSet<ShardId> = shards.iter().map(|s| s.id.clone()).collect();
        routing_table.retain(|shard_id, _| active_shard_ids.contains(shard_id));
        
        // 各シャードのルーティングパスを最適化
        for shard in &shards {
            // 最適なルーティングパスを計算
            let mut paths = Vec::new();
            
            for target in &shards {
                if shard.id == target.id {
                    continue;
                }
                
                // 直接接続可能なシャードを優先
                if shard_manager.are_shards_connected(&shard.id, &target.id) {
                    paths.push(target.id.clone());
                } else {
                    // 中継シャードを探す
                    let mut best_relay = None;
                    let mut min_hops = usize::MAX;
                    
                    for relay in &shards {
                        if relay.id == shard.id || relay.id == target.id {
                            continue;
                        }
                        
                        if shard_manager.are_shards_connected(&shard.id, &relay.id) && 
                           shard_manager.are_shards_connected(&relay.id, &target.id) {
                            // 2ホップで到達可能
                            if min_hops > 2 {
                                min_hops = 2;
                                best_relay = Some(relay.id.clone());
                            }
                        }
                    }
                    
                    if let Some(relay) = best_relay {
                        paths.push(relay);
                    } else {
                        // 直接接続できないシャードは最後に追加
                        paths.push(target.id.clone());
                    }
                }
            }
            
            // ルーティングテーブルを更新
            routing_table.insert(shard.id.clone(), paths);
        }
        
        // メトリクスを更新
        metrics.set_gauge("routing_table_size", routing_table.len() as f64);
    }
    
    /// 次のホップを取得
    pub fn get_next_hop(&self, source: &ShardId, target: &ShardId) -> Option<ShardId> {
        if source == target {
            return None;
        }
        
        let routing_table = self.routing_table.lock().unwrap();
        
        if let Some(paths) = routing_table.get(source) {
            // ターゲットへの直接パスを探す
            for path in paths {
                if path == target {
                    return Some(target.clone());
                }
            }
            
            // 中継シャードを探す
            for path in paths {
                if let Some(next_paths) = routing_table.get(path) {
                    if next_paths.contains(target) {
                        return Some(path.clone());
                    }
                }
            }
        }
        
        // デフォルトでは直接ターゲットに送信
        Some(target.clone())
    }
    
    /// メッセージをキャッシュ
    pub fn cache_message(&self, key: &str, data: &[u8]) {
        let mut cache = self.message_cache.lock().unwrap();
        cache.put(key.to_string(), data.to_vec());
    }
    
    /// キャッシュからメッセージを取得
    pub fn get_cached_message(&self, key: &str) -> Option<Vec<u8>> {
        let mut cache = self.message_cache.lock().unwrap();
        cache.get(key).cloned()
    }
    
    /// キューのサイズを取得
    pub fn get_queue_size(&self, shard_id: &ShardId) -> usize {
        let message_queue = self.message_queue.lock().unwrap();
        message_queue.get(shard_id).map_or(0, |q| q.len())
    }
    
    /// 全キューの合計サイズを取得
    pub fn get_total_queue_size(&self) -> usize {
        let message_queue = self.message_queue.lock().unwrap();
        message_queue.values().map(|q| q.len()).sum()
    }
    
    /// 送信中のメッセージ数を取得
    pub fn get_sending_count(&self) -> usize {
        self.sending.lock().unwrap().len()
    }
    
    /// バッチサイズを設定
    pub fn set_batch_size(&mut self, batch_size: usize) {
        self.batch_size = batch_size;
        self.min_batch_size = batch_size / 4;
    }
    
    /// 最大バッチ数を設定
    pub fn set_max_batches(&mut self, max_batches: usize) {
        self.max_batches = max_batches;
    }
    
    /// 最大待機時間を設定
    pub fn set_max_wait_ms(&mut self, max_wait_ms: u64) {
        self.max_wait_ms = max_wait_ms;
    }
    
    /// 最適化間隔を設定
    pub fn set_optimization_interval_secs(&mut self, optimization_interval_secs: u64) {
        self.optimization_interval_secs = optimization_interval_secs;
    }
    
    /// 圧縮閾値を設定
    pub fn set_compression_threshold(&mut self, threshold: usize) {
        self.compression_threshold = threshold;
    }
    
    /// 圧縮レベルを設定
    pub fn set_compression_level(&mut self, level: u32) {
        self.compression_level = level.min(9);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shard::ShardInfo;
    use crate::network::{NetworkMessage, MessageType};
    use chrono::Utc;
    use std::sync::Arc;
    use mockall::predicate::*;
    use mockall::mock;
    
    // ShardManagerのモック
    mock! {
        ShardManager {
            fn get_active_shards(&self) -> Vec<ShardInfo>;
            fn are_shards_connected(&self, shard1: &ShardId, shard2: &ShardId) -> bool;
            fn get_shard_info(&self, shard_id: &ShardId) -> Option<ShardInfo>;
            fn shard_exists(&self, shard_id: &ShardId) -> bool;
        }
    }
    
    #[test]
    fn test_add_message() {
        // ShardManagerのモックを作成
        let mut mock_shard_manager = MockShardManager::new();
        mock_shard_manager.expect_get_active_shards()
            .returning(|| vec![
                ShardInfo {
                    id: "shard1".to_string(),
                    peers: vec![],
                    status: "active".to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
                ShardInfo {
                    id: "shard2".to_string(),
                    peers: vec![],
                    status: "active".to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
            ]);
        
        let metrics = Arc::new(MetricsCollector::new("test"));
        let optimizer = ShardCommunicationOptimizer::new(
            Arc::new(mock_shard_manager),
            100,
            10,
            1000,
            1000,
            metrics,
        );
        
        // メッセージを作成
        let message = NetworkMessage {
            message_type: MessageType::Transaction,
            sender: "shard1".to_string(),
            receiver: "shard2".to_string(),
            data: vec![1, 2, 3, 4],
            timestamp: Utc::now(),
        };
        
        // メッセージを追加
        let result = optimizer.add_message(message);
        assert!(result.is_ok());
        
        // キューサイズを確認
        assert_eq!(optimizer.get_queue_size(&"shard2".to_string()), 1);
        assert_eq!(optimizer.get_total_queue_size(), 1);
    }
    
    #[test]
    fn test_create_batch() {
        // ShardManagerのモックを作成
        let mut mock_shard_manager = MockShardManager::new();
        mock_shard_manager.expect_get_active_shards()
            .returning(|| vec![
                ShardInfo {
                    id: "shard1".to_string(),
                    peers: vec![],
                    status: "active".to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
                ShardInfo {
                    id: "shard2".to_string(),
                    peers: vec![],
                    status: "active".to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
            ]);
        
        let metrics = Arc::new(MetricsCollector::new("test"));
        let optimizer = ShardCommunicationOptimizer::new(
            Arc::new(mock_shard_manager),
            10,
            10,
            1000,
            1000,
            metrics,
        );
        
        // メッセージを追加
        for i in 0..20 {
            let message = NetworkMessage {
                message_type: MessageType::Transaction,
                sender: "shard1".to_string(),
                receiver: "shard2".to_string(),
                data: vec![i as u8; 100],
                timestamp: Utc::now(),
            };
            
            optimizer.add_message(message).unwrap();
        }
        
        // バッチを作成
        let batch = optimizer.create_batch(&"shard2".to_string());
        assert!(batch.is_some());
        
        let batch = batch.unwrap();
        assert_eq!(batch.messages.len(), 10);
        assert_eq!(batch.target_shard, "shard2");
        
        // 送信中のメッセージ数を確認
        assert_eq!(optimizer.get_sending_count(), 10);
        
        // キューサイズを確認
        assert_eq!(optimizer.get_queue_size(&"shard2".to_string()), 10);
    }
    
    #[test]
    fn test_compression() {
        // ShardManagerのモックを作成
        let mut mock_shard_manager = MockShardManager::new();
        mock_shard_manager.expect_get_active_shards()
            .returning(|| vec![
                ShardInfo {
                    id: "shard1".to_string(),
                    peers: vec![],
                    status: "active".to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
                ShardInfo {
                    id: "shard2".to_string(),
                    peers: vec![],
                    status: "active".to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
            ]);
        
        let metrics = Arc::new(MetricsCollector::new("test"));
        let mut optimizer = ShardCommunicationOptimizer::new(
            Arc::new(mock_shard_manager),
            10,
            10,
            1000,
            1000,
            metrics,
        );
        
        // 圧縮閾値を設定
        optimizer.set_compression_threshold(100);
        
        // メッセージを追加（圧縮対象）
        for i in 0..10 {
            let message = NetworkMessage {
                message_type: MessageType::Transaction,
                sender: "shard1".to_string(),
                receiver: "shard2".to_string(),
                data: vec![i as u8; 200],
                timestamp: Utc::now(),
            };
            
            optimizer.add_message(message).unwrap();
        }
        
        // バッチを作成
        let batch = optimizer.create_batch(&"shard2".to_string());
        assert!(batch.is_some());
        
        let batch = batch.unwrap();
        assert_eq!(batch.messages.len(), 10);
        assert!(batch.compressed);
        assert!(batch.compressed_size.is_some());
        assert!(batch.compressed_size.unwrap() < batch.original_size);
        
        // 圧縮と解凍のテスト
        let compressed = optimizer.compress_batch(&batch.messages);
        let decompressed = optimizer.decompress_batch(&compressed);
        assert!(decompressed.is_ok());
        assert_eq!(decompressed.unwrap().len(), batch.messages.len());
    }
    
    #[tokio::test]
    async fn test_message_processing() {
        // ShardManagerのモックを作成
        let mut mock_shard_manager = MockShardManager::new();
        mock_shard_manager.expect_get_active_shards()
            .returning(|| vec![
                ShardInfo {
                    id: "shard1".to_string(),
                    peers: vec![],
                    status: "active".to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
                ShardInfo {
                    id: "shard2".to_string(),
                    peers: vec![],
                    status: "active".to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
            ]);
        mock_shard_manager.expect_are_shards_connected()
            .returning(|_, _| true);
        
        let metrics = Arc::new(MetricsCollector::new("test"));
        let optimizer = ShardCommunicationOptimizer::new(
            Arc::new(mock_shard_manager),
            10,
            10,
            100,
            1000,
            metrics,
        );
        
        // メッセージを追加
        for i in 0..20 {
            let message = NetworkMessage {
                message_type: MessageType::Transaction,
                sender: "shard1".to_string(),
                receiver: "shard2".to_string(),
                data: vec![i as u8; 100],
                timestamp: Utc::now(),
            };
            
            optimizer.add_message(message).unwrap();
        }
        
        // 送信関数
        let sender = |batch: MessageBatch| -> Result<(), Error> {
            // 実際の実装では、ネットワークを通じてメッセージを送信
            // ここではテスト用に成功を返す
            Ok(())
        };
        
        // 処理を開始
        optimizer.start_processing(sender).await.unwrap();
        
        // 少し待機
        time::sleep(Duration::from_millis(500)).await;
        
        // 処理を停止
        optimizer.stop();
        
        // キューが空になっていることを確認
        assert_eq!(optimizer.get_total_queue_size(), 0);
        
        // 送信中のメッセージがないことを確認
        assert_eq!(optimizer.get_sending_count(), 0);
    }
}