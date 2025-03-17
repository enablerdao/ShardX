use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use log::{debug, error, info, warn};
use tokio::sync::mpsc;
use tokio::time;

use crate::error::Error;
use crate::transaction::{Transaction, TransactionStatus, TransactionType};
use crate::metrics::MetricsCollector;

/// トランザクション最適化器
/// 
/// トランザクション処理のスループットを向上させるための最適化を行う。
/// - バッチ処理
/// - 並列処理
/// - 優先順位付け
/// - 依存関係の解決
/// - メモリプール最適化
pub struct TransactionOptimizer {
    /// メモリプール
    mempool: Arc<Mutex<HashMap<String, Transaction>>>,
    /// 処理中のトランザクション
    processing: Arc<Mutex<HashSet<String>>>,
    /// 優先キュー
    priority_queue: Arc<Mutex<VecDeque<Transaction>>>,
    /// バッチサイズ
    batch_size: usize,
    /// 最大バッチ数
    max_batches: usize,
    /// 最大並列度
    max_parallelism: usize,
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
    /// 依存関係グラフ
    dependency_graph: Arc<Mutex<HashMap<String, HashSet<String>>>>,
    /// 送信者ノンスマップ
    sender_nonce_map: Arc<Mutex<HashMap<String, u64>>>,
    /// 実行中フラグ
    running: Arc<Mutex<bool>>,
}

/// トランザクションバッチ
#[derive(Debug, Clone)]
pub struct TransactionBatch {
    /// バッチID
    pub id: String,
    /// トランザクション
    pub transactions: Vec<Transaction>,
    /// 作成時刻
    pub created_at: Instant,
    /// 優先度
    pub priority: u8,
    /// 依存バッチID
    pub depends_on: Option<String>,
}

/// トランザクション優先度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TransactionPriority {
    /// 低
    Low = 0,
    /// 通常
    Normal = 1,
    /// 高
    High = 2,
    /// 最高
    Critical = 3,
}

impl TransactionOptimizer {
    /// 新しいTransactionOptimizerを作成
    pub fn new(
        batch_size: usize,
        max_batches: usize,
        max_parallelism: usize,
        max_wait_ms: u64,
        metrics: Arc<MetricsCollector>,
    ) -> Self {
        let min_batch_size = batch_size / 4;
        
        Self {
            mempool: Arc::new(Mutex::new(HashMap::new())),
            processing: Arc::new(Mutex::new(HashSet::new())),
            priority_queue: Arc::new(Mutex::new(VecDeque::new())),
            batch_size,
            max_batches,
            max_parallelism,
            max_wait_ms,
            min_batch_size,
            metrics,
            last_optimization: Arc::new(Mutex::new(Instant::now())),
            optimization_interval_secs: 60,
            dependency_graph: Arc::new(Mutex::new(HashMap::new())),
            sender_nonce_map: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    /// トランザクションを追加
    pub fn add_transaction(&self, transaction: Transaction) -> Result<(), Error> {
        // メモリプールに追加
        let mut mempool = self.mempool.lock().unwrap();
        
        // 既に存在するか確認
        if mempool.contains_key(&transaction.id) {
            return Err(Error::DuplicateTransaction(format!(
                "Transaction already exists: {}", transaction.id
            )));
        }
        
        // 送信者のノンスをチェック
        let mut sender_nonce_map = self.sender_nonce_map.lock().unwrap();
        let current_nonce = sender_nonce_map.get(&transaction.sender).cloned().unwrap_or(0);
        
        if transaction.nonce < current_nonce {
            return Err(Error::InvalidNonce(format!(
                "Transaction nonce ({}) is less than current nonce ({})",
                transaction.nonce, current_nonce
            )));
        }
        
        // 依存関係を更新
        if transaction.nonce > current_nonce {
            // ノンスのギャップがある場合は依存関係を追加
            let mut dependency_graph = self.dependency_graph.lock().unwrap();
            
            for n in current_nonce..transaction.nonce {
                let dependency_key = format!("{}:{}", transaction.sender, n);
                
                let dependencies = dependency_graph.entry(transaction.id.clone())
                    .or_insert_with(HashSet::new);
                
                dependencies.insert(dependency_key);
            }
        }
        
        // ノンスマップを更新
        sender_nonce_map.insert(transaction.sender.clone(), transaction.nonce + 1);
        
        // メモリプールに追加
        mempool.insert(transaction.id.clone(), transaction.clone());
        
        // 優先キューに追加
        let mut priority_queue = self.priority_queue.lock().unwrap();
        priority_queue.push_back(transaction);
        
        // メトリクスを更新
        self.metrics.increment_counter("transactions_added_to_mempool");
        self.metrics.set_gauge("mempool_size", mempool.len() as f64);
        
        Ok(())
    }
    
    /// トランザクションバッチを作成
    pub fn create_batch(&self) -> Option<TransactionBatch> {
        let mut priority_queue = self.priority_queue.lock().unwrap();
        let processing = self.processing.lock().unwrap();
        
        if priority_queue.is_empty() {
            return None;
        }
        
        // バッチに含めるトランザクションを選択
        let mut batch_transactions = Vec::with_capacity(self.batch_size);
        let mut batch_priority = 0;
        
        // 処理中でないトランザクションを選択
        let mut i = 0;
        while i < priority_queue.len() && batch_transactions.len() < self.batch_size {
            let tx = priority_queue.get(i).unwrap().clone();
            
            // 処理中でないか確認
            if !processing.contains(&tx.id) {
                // 依存関係をチェック
                let dependency_graph = self.dependency_graph.lock().unwrap();
                let has_unresolved_dependencies = if let Some(dependencies) = dependency_graph.get(&tx.id) {
                    dependencies.iter().any(|dep_id| {
                        let mempool = self.mempool.lock().unwrap();
                        mempool.contains_key(dep_id) && !processing.contains(dep_id)
                    })
                } else {
                    false
                };
                
                if !has_unresolved_dependencies {
                    // バッチに追加
                    batch_transactions.push(tx.clone());
                    
                    // 優先度を更新
                    let tx_priority = self.get_transaction_priority(&tx);
                    batch_priority = batch_priority.max(tx_priority as u8);
                    
                    // キューから削除
                    priority_queue.remove(i);
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
        
        if batch_transactions.is_empty() {
            return None;
        }
        
        // バッチを作成
        let batch_id = format!("batch_{}", Instant::now().elapsed().as_nanos());
        
        Some(TransactionBatch {
            id: batch_id,
            transactions: batch_transactions,
            created_at: Instant::now(),
            priority: batch_priority,
            depends_on: None,
        })
    }
    
    /// トランザクションの優先度を取得
    fn get_transaction_priority(&self, transaction: &Transaction) -> TransactionPriority {
        match transaction.transaction_type {
            TransactionType::System => TransactionPriority::Critical,
            TransactionType::SmartContract => {
                if transaction.fee > 1000 {
                    TransactionPriority::High
                } else {
                    TransactionPriority::Normal
                }
            },
            TransactionType::Transfer => {
                if transaction.fee > 500 {
                    TransactionPriority::High
                } else if transaction.fee > 100 {
                    TransactionPriority::Normal
                } else {
                    TransactionPriority::Low
                }
            },
            _ => TransactionPriority::Normal,
        }
    }
    
    /// トランザクション処理を開始
    pub async fn start_processing<F>(&self, processor: F) -> Result<(), Error>
    where
        F: Fn(TransactionBatch) -> Result<Vec<(String, TransactionStatus)>, Error> + Send + Sync + 'static,
    {
        // 既に実行中かチェック
        let mut running = self.running.lock().unwrap();
        if *running {
            return Err(Error::InvalidState("Transaction optimizer is already running".to_string()));
        }
        
        *running = true;
        drop(running);
        
        // チャネルを作成
        let (batch_tx, mut batch_rx) = mpsc::channel(self.max_batches);
        
        // 最適化タスクを開始
        let mempool = self.mempool.clone();
        let processing = self.processing.clone();
        let priority_queue = self.priority_queue.clone();
        let metrics = self.metrics.clone();
        let last_optimization = self.last_optimization.clone();
        let optimization_interval_secs = self.optimization_interval_secs;
        let dependency_graph = self.dependency_graph.clone();
        let sender_nonce_map = self.sender_nonce_map.clone();
        let running = self.running.clone();
        let batch_size = self.batch_size;
        let min_batch_size = self.min_batch_size;
        let max_wait_ms = self.max_wait_ms;
        
        // バッチ生成タスク
        tokio::spawn(async move {
            let mut last_batch_time = Instant::now();
            
            while *running.lock().unwrap() {
                // バッチを作成
                let batch_option = {
                    let self_ref = &self;
                    self_ref.create_batch()
                };
                
                if let Some(batch) = batch_option {
                    // 処理中に追加
                    {
                        let mut processing = processing.lock().unwrap();
                        for tx in &batch.transactions {
                            processing.insert(tx.id.clone());
                        }
                    }
                    
                    // バッチを送信
                    if let Err(e) = batch_tx.send(batch).await {
                        error!("Failed to send batch: {}", e);
                        
                        // 処理中から削除
                        let mut processing = processing.lock().unwrap();
                        for tx in &e.0.transactions {
                            processing.remove(&tx.id);
                        }
                    }
                    
                    // メトリクスを更新
                    metrics.increment_counter("transaction_batches_created");
                    
                    // 最後のバッチ時間を更新
                    last_batch_time = Instant::now();
                } else {
                    // 最小バッチサイズに達していない場合は待機
                    let queue_size = priority_queue.lock().unwrap().len();
                    
                    if queue_size >= min_batch_size || last_batch_time.elapsed().as_millis() as u64 >= max_wait_ms {
                        // 最適化を実行
                        let mut last_opt = last_optimization.lock().unwrap();
                        if last_opt.elapsed().as_secs() >= optimization_interval_secs {
                            drop(last_opt);
                            
                            // メモリプールを最適化
                            Self::optimize_mempool(
                                mempool.clone(),
                                processing.clone(),
                                dependency_graph.clone(),
                                sender_nonce_map.clone(),
                                metrics.clone(),
                            );
                            
                            // 最後の最適化時刻を更新
                            *last_optimization.lock().unwrap() = Instant::now();
                        }
                    }
                    
                    // 少し待機
                    time::sleep(Duration::from_millis(10)).await;
                }
            }
        });
        
        // バッチ処理タスク
        let mempool = self.mempool.clone();
        let processing = self.processing.clone();
        let metrics = self.metrics.clone();
        let max_parallelism = self.max_parallelism;
        let running = self.running.clone();
        
        tokio::spawn(async move {
            // 並列処理用のセマフォ
            let semaphore = Arc::new(tokio::sync::Semaphore::new(max_parallelism));
            
            while *running.lock().unwrap() {
                // バッチを受信
                if let Some(batch) = batch_rx.recv().await {
                    // メトリクスを更新
                    metrics.increment_counter("transaction_batches_received");
                    metrics.observe_histogram("transaction_batch_size", batch.transactions.len() as f64);
                    
                    // セマフォを取得
                    let permit = semaphore.clone().acquire_owned().await.unwrap();
                    
                    // 処理関数のクローン
                    let processor = processor.clone();
                    let mempool = mempool.clone();
                    let processing = processing.clone();
                    let metrics = metrics.clone();
                    
                    // バッチを処理
                    tokio::spawn(async move {
                        let start_time = Instant::now();
                        let batch_size = batch.transactions.len();
                        
                        // バッチを処理
                        let result = processor(batch.clone());
                        
                        match result {
                            Ok(statuses) => {
                                // 処理結果を反映
                                let mut mempool = mempool.lock().unwrap();
                                let mut processing = processing.lock().unwrap();
                                
                                for (tx_id, status) in statuses {
                                    // メモリプールから削除
                                    if let Some(mut tx) = mempool.remove(&tx_id) {
                                        // ステータスを更新
                                        tx.status = status;
                                        
                                        // 処理中から削除
                                        processing.remove(&tx_id);
                                        
                                        // メトリクスを更新
                                        metrics.increment_counter("transactions_processed");
                                    }
                                }
                                
                                // メトリクスを更新
                                metrics.observe_histogram("transaction_batch_processing_time", start_time.elapsed().as_secs_f64());
                                metrics.increment_counter("transaction_batches_processed");
                                
                                // 成功率を計算
                                let success_rate = statuses.len() as f64 / batch_size as f64;
                                metrics.observe_histogram("transaction_batch_success_rate", success_rate);
                            },
                            Err(e) => {
                                // エラーをログに記録
                                error!("Failed to process batch: {}", e);
                                
                                // 処理中から削除
                                let mut processing = processing.lock().unwrap();
                                for tx in &batch.transactions {
                                    processing.remove(&tx.id);
                                }
                                
                                // メトリクスを更新
                                metrics.increment_counter("transaction_batches_failed");
                            },
                        }
                        
                        // セマフォを解放（permitがドロップされる）
                        drop(permit);
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
    
    /// メモリプールを最適化
    fn optimize_mempool(
        mempool: Arc<Mutex<HashMap<String, Transaction>>>,
        processing: Arc<Mutex<HashSet<String>>>,
        dependency_graph: Arc<Mutex<HashMap<String, HashSet<String>>>>,
        sender_nonce_map: Arc<Mutex<HashMap<String, u64>>>,
        metrics: Arc<MetricsCollector>,
    ) {
        // 古いトランザクションを削除
        let mut mempool = mempool.lock().unwrap();
        let processing = processing.lock().unwrap();
        let mut dependency_graph = dependency_graph.lock().unwrap();
        
        let now = Instant::now();
        let mut expired_count = 0;
        let mut orphaned_count = 0;
        
        // 期限切れのトランザクションを削除
        let expired_txs: Vec<String> = mempool.iter()
            .filter(|(id, tx)| {
                // 処理中でないトランザクションのみ
                if processing.contains(*id) {
                    return false;
                }
                
                // 1時間以上経過したトランザクションを期限切れとする
                let tx_age = now.duration_since(tx.timestamp.into()).as_secs();
                tx_age > 3600
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        for tx_id in &expired_txs {
            mempool.remove(tx_id);
            expired_count += 1;
        }
        
        // 依存関係を更新
        for tx_id in &expired_txs {
            dependency_graph.remove(tx_id);
            
            // このトランザクションに依存するトランザクションの依存関係を更新
            for (_, dependencies) in dependency_graph.iter_mut() {
                dependencies.remove(tx_id);
            }
        }
        
        // 孤立したトランザクションを検出
        let mut sender_nonce_map = sender_nonce_map.lock().unwrap();
        let orphaned_txs: Vec<String> = mempool.iter()
            .filter(|(id, tx)| {
                // 処理中でないトランザクションのみ
                if processing.contains(*id) {
                    return false;
                }
                
                // 送信者のノンスをチェック
                if let Some(current_nonce) = sender_nonce_map.get(&tx.sender) {
                    // ノンスが大きすぎる場合は孤立している
                    tx.nonce > *current_nonce + 10
                } else {
                    false
                }
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        for tx_id in &orphaned_txs {
            mempool.remove(tx_id);
            orphaned_count += 1;
        }
        
        // 依存関係を更新
        for tx_id in &orphaned_txs {
            dependency_graph.remove(tx_id);
            
            // このトランザクションに依存するトランザクションの依存関係を更新
            for (_, dependencies) in dependency_graph.iter_mut() {
                dependencies.remove(tx_id);
            }
        }
        
        // メトリクスを更新
        metrics.increment_counter_by("transactions_expired", expired_count);
        metrics.increment_counter_by("transactions_orphaned", orphaned_count);
        metrics.set_gauge("mempool_size", mempool.len() as f64);
    }
    
    /// メモリプールのサイズを取得
    pub fn get_mempool_size(&self) -> usize {
        self.mempool.lock().unwrap().len()
    }
    
    /// 処理中のトランザクション数を取得
    pub fn get_processing_count(&self) -> usize {
        self.processing.lock().unwrap().len()
    }
    
    /// 優先キューのサイズを取得
    pub fn get_queue_size(&self) -> usize {
        self.priority_queue.lock().unwrap().len()
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
    
    /// 最大並列度を設定
    pub fn set_max_parallelism(&mut self, max_parallelism: usize) {
        self.max_parallelism = max_parallelism;
    }
    
    /// 最大待機時間を設定
    pub fn set_max_wait_ms(&mut self, max_wait_ms: u64) {
        self.max_wait_ms = max_wait_ms;
    }
    
    /// 最適化間隔を設定
    pub fn set_optimization_interval_secs(&mut self, optimization_interval_secs: u64) {
        self.optimization_interval_secs = optimization_interval_secs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::Transaction;
    use chrono::Utc;
    
    #[test]
    fn test_add_transaction() {
        let metrics = Arc::new(MetricsCollector::new("test"));
        let optimizer = TransactionOptimizer::new(100, 10, 4, 1000, metrics);
        
        // トランザクションを作成
        let tx = Transaction {
            id: "tx1".to_string(),
            transaction_type: TransactionType::Transfer,
            sender: "sender1".to_string(),
            recipient: "recipient1".to_string(),
            amount: 100,
            fee: 10,
            nonce: 0,
            data: vec![],
            timestamp: Utc::now(),
            signature: None,
            status: TransactionStatus::Pending,
            block_id: None,
            shard_id: "shard1".to_string(),
        };
        
        // トランザクションを追加
        let result = optimizer.add_transaction(tx.clone());
        assert!(result.is_ok());
        
        // メモリプールのサイズを確認
        assert_eq!(optimizer.get_mempool_size(), 1);
        
        // 同じトランザクションを再度追加
        let result = optimizer.add_transaction(tx);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_create_batch() {
        let metrics = Arc::new(MetricsCollector::new("test"));
        let optimizer = TransactionOptimizer::new(10, 10, 4, 1000, metrics);
        
        // トランザクションを追加
        for i in 0..20 {
            let tx = Transaction {
                id: format!("tx{}", i),
                transaction_type: TransactionType::Transfer,
                sender: format!("sender{}", i % 5),
                recipient: format!("recipient{}", i % 3),
                amount: 100,
                fee: 10,
                nonce: (i / 5) as u64,
                data: vec![],
                timestamp: Utc::now(),
                signature: None,
                status: TransactionStatus::Pending,
                block_id: None,
                shard_id: "shard1".to_string(),
            };
            
            optimizer.add_transaction(tx).unwrap();
        }
        
        // バッチを作成
        let batch = optimizer.create_batch();
        assert!(batch.is_some());
        
        let batch = batch.unwrap();
        assert_eq!(batch.transactions.len(), 10);
        
        // 処理中のトランザクション数を確認
        assert_eq!(optimizer.get_processing_count(), 10);
        
        // 優先キューのサイズを確認
        assert_eq!(optimizer.get_queue_size(), 10);
    }
    
    #[tokio::test]
    async fn test_transaction_processing() {
        let metrics = Arc::new(MetricsCollector::new("test"));
        let optimizer = TransactionOptimizer::new(10, 10, 4, 100, metrics);
        
        // トランザクションを追加
        for i in 0..20 {
            let tx = Transaction {
                id: format!("tx{}", i),
                transaction_type: TransactionType::Transfer,
                sender: format!("sender{}", i % 5),
                recipient: format!("recipient{}", i % 3),
                amount: 100,
                fee: 10,
                nonce: (i / 5) as u64,
                data: vec![],
                timestamp: Utc::now(),
                signature: None,
                status: TransactionStatus::Pending,
                block_id: None,
                shard_id: "shard1".to_string(),
            };
            
            optimizer.add_transaction(tx).unwrap();
        }
        
        // 処理関数
        let processor = |batch: TransactionBatch| -> Result<Vec<(String, TransactionStatus)>, Error> {
            let mut results = Vec::new();
            
            for tx in batch.transactions {
                results.push((tx.id, TransactionStatus::Confirmed));
            }
            
            Ok(results)
        };
        
        // 処理を開始
        optimizer.start_processing(processor).await.unwrap();
        
        // 少し待機
        time::sleep(Duration::from_millis(500)).await;
        
        // 処理を停止
        optimizer.stop();
        
        // メモリプールが空になっていることを確認
        assert_eq!(optimizer.get_mempool_size(), 0);
        
        // 処理中のトランザクションがないことを確認
        assert_eq!(optimizer.get_processing_count(), 0);
    }
}