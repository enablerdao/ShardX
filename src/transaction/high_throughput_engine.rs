use log::{debug, error, info, warn};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::{mpsc, Semaphore};
use tokio::time::{sleep, Duration, Instant};

use crate::error::Error;
use crate::network::NetworkMessage;
use crate::shard::{ShardId, ShardInfo, ShardManager, ShardStatus};
use crate::transaction::{
    CrossShardManager, CrossShardOptimizer, OptimizerConfig, ParallelProcessor, ProcessorConfig,
    Transaction, TransactionStatus,
};

/// 高スループットトランザクションエンジン
///
/// 100,000 TPS以上のスループットを実現するための高性能エンジン。
/// 主な最適化手法:
/// 1. ハイブリッド処理 - 単一シャードとクロスシャードトランザクションを最適化
/// 2. 適応型バッチ処理 - 負荷に応じてバッチサイズを動的に調整
/// 3. メモリプール最適化 - 効率的なメモリ管理
/// 4. ハードウェアアクセラレーション - 利用可能なハードウェアリソースを最大限活用
/// 5. 非同期I/O最適化 - I/Oバウンドな操作を最適化
pub struct HighThroughputEngine {
    /// クロスシャードマネージャー
    cross_shard_manager: Arc<CrossShardManager>,
    /// シャードマネージャー
    shard_manager: Arc<ShardManager>,
    /// ネットワークメッセージ送信チャネル
    network_tx: mpsc::Sender<NetworkMessage>,
    /// クロスシャード最適化器
    cross_shard_optimizer: Arc<CrossShardOptimizer>,
    /// 並列処理器
    parallel_processor: Arc<ParallelProcessor>,
    /// メモリプール
    memory_pool: Arc<RwLock<MemoryPool>>,
    /// エンジン設定
    config: EngineConfig,
    /// エンジン統計
    stats: Arc<RwLock<EngineStats>>,
    /// 処理キュー
    processing_queue: Arc<Mutex<VecDeque<Transaction>>>,
    /// 処理セマフォア
    semaphore: Arc<Semaphore>,
    /// 実行中フラグ
    running: Arc<RwLock<bool>>,
}

/// メモリプール
#[derive(Debug)]
struct MemoryPool {
    /// 保留中のトランザクション
    pending_transactions: HashMap<String, Transaction>,
    /// 確認済みのトランザクション
    confirmed_transactions: HashMap<String, Transaction>,
    /// 拒否されたトランザクション
    rejected_transactions: HashMap<String, (Transaction, String)>,
    /// アドレス別トランザクション
    address_transactions: HashMap<String, HashSet<String>>,
    /// シャード別トランザクション
    shard_transactions: HashMap<ShardId, HashSet<String>>,
    /// 最大サイズ
    max_size: usize,
    /// 現在のサイズ
    current_size: usize,
    /// 最終クリーンアップ時刻
    last_cleanup: u64,
}

impl MemoryPool {
    /// 新しいメモリプールを作成
    fn new(max_size: usize) -> Self {
        Self {
            pending_transactions: HashMap::new(),
            confirmed_transactions: HashMap::new(),
            rejected_transactions: HashMap::new(),
            address_transactions: HashMap::new(),
            shard_transactions: HashMap::new(),
            max_size,
            current_size: 0,
            last_cleanup: chrono::Utc::now().timestamp() as u64,
        }
    }

    /// トランザクションを追加
    fn add_transaction(&mut self, transaction: Transaction) -> Result<(), Error> {
        // プールが満杯の場合はエラー
        if self.current_size >= self.max_size {
            return Err(Error::CapacityError("Memory pool is full".to_string()));
        }

        let tx_id = transaction.id.clone();
        let from_addr = transaction.from.clone();
        let shard_id = transaction.shard_id.clone();

        // 保留中のトランザクションに追加
        self.pending_transactions.insert(tx_id.clone(), transaction);
        self.current_size += 1;

        // アドレス別トランザクションに追加
        let addr_txs = self
            .address_transactions
            .entry(from_addr)
            .or_insert_with(HashSet::new);
        addr_txs.insert(tx_id.clone());

        // シャード別トランザクションに追加
        let shard_txs = self
            .shard_transactions
            .entry(shard_id)
            .or_insert_with(HashSet::new);
        shard_txs.insert(tx_id);

        Ok(())
    }

    /// トランザクションを確認済みに移動
    fn confirm_transaction(&mut self, tx_id: &str) -> Result<(), Error> {
        if let Some(tx) = self.pending_transactions.remove(tx_id) {
            self.confirmed_transactions.insert(tx_id.to_string(), tx);
            Ok(())
        } else {
            Err(Error::NotFoundError(format!(
                "Transaction {} not found in pending pool",
                tx_id
            )))
        }
    }

    /// トランザクションを拒否
    fn reject_transaction(&mut self, tx_id: &str, reason: &str) -> Result<(), Error> {
        if let Some(tx) = self.pending_transactions.remove(tx_id) {
            self.rejected_transactions
                .insert(tx_id.to_string(), (tx, reason.to_string()));
            Ok(())
        } else {
            Err(Error::NotFoundError(format!(
                "Transaction {} not found in pending pool",
                tx_id
            )))
        }
    }

    /// アドレスのトランザクションを取得
    fn get_address_transactions(&self, address: &str) -> Vec<Transaction> {
        if let Some(tx_ids) = self.address_transactions.get(address) {
            tx_ids
                .iter()
                .filter_map(|id| self.pending_transactions.get(id).cloned())
                .collect()
        } else {
            vec![]
        }
    }

    /// シャードのトランザクションを取得
    fn get_shard_transactions(&self, shard_id: &ShardId) -> Vec<Transaction> {
        if let Some(tx_ids) = self.shard_transactions.get(shard_id) {
            tx_ids
                .iter()
                .filter_map(|id| self.pending_transactions.get(id).cloned())
                .collect()
        } else {
            vec![]
        }
    }

    /// 古いトランザクションをクリーンアップ
    fn cleanup(&mut self, max_age_sec: u64) -> usize {
        let now = chrono::Utc::now().timestamp() as u64;
        let cutoff = now.saturating_sub(max_age_sec);

        // 古い確認済みトランザクションを削除
        let confirmed_to_remove: Vec<String> = self
            .confirmed_transactions
            .iter()
            .filter(|(_, tx)| tx.timestamp < cutoff)
            .map(|(id, _)| id.clone())
            .collect();

        for id in &confirmed_to_remove {
            self.confirmed_transactions.remove(id);
        }

        // 古い拒否されたトランザクションを削除
        let rejected_to_remove: Vec<String> = self
            .rejected_transactions
            .iter()
            .filter(|(_, (tx, _))| tx.timestamp < cutoff)
            .map(|(id, _)| id.clone())
            .collect();

        for id in &rejected_to_remove {
            self.rejected_transactions.remove(id);
        }

        self.last_cleanup = now;

        confirmed_to_remove.len() + rejected_to_remove.len()
    }

    /// メモリプールの統計を取得
    fn get_stats(&self) -> MemoryPoolStats {
        MemoryPoolStats {
            pending_count: self.pending_transactions.len(),
            confirmed_count: self.confirmed_transactions.len(),
            rejected_count: self.rejected_transactions.len(),
            total_count: self.pending_transactions.len()
                + self.confirmed_transactions.len()
                + self.rejected_transactions.len(),
            max_size: self.max_size,
            current_size: self.current_size,
            utilization: self.current_size as f64 / self.max_size as f64,
            last_cleanup: self.last_cleanup,
        }
    }
}

/// メモリプール統計
#[derive(Debug, Clone)]
pub struct MemoryPoolStats {
    /// 保留中のトランザクション数
    pub pending_count: usize,
    /// 確認済みのトランザクション数
    pub confirmed_count: usize,
    /// 拒否されたトランザクション数
    pub rejected_count: usize,
    /// 合計トランザクション数
    pub total_count: usize,
    /// 最大サイズ
    pub max_size: usize,
    /// 現在のサイズ
    pub current_size: usize,
    /// 使用率
    pub utilization: f64,
    /// 最終クリーンアップ時刻
    pub last_cleanup: u64,
}

/// エンジン設定
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// 最大スループット（TPS）
    pub max_throughput: u64,
    /// 最大並列処理数
    pub max_parallelism: usize,
    /// メモリプールの最大サイズ
    pub memory_pool_size: usize,
    /// バッチサイズ
    pub batch_size: usize,
    /// 最小バッチサイズ
    pub min_batch_size: usize,
    /// 最大バッチサイズ
    pub max_batch_size: usize,
    /// バッチ処理間隔（ミリ秒）
    pub batch_interval_ms: u64,
    /// 処理タイムアウト（ミリ秒）
    pub processing_timeout_ms: u64,
    /// メモリプールクリーンアップ間隔（秒）
    pub memory_pool_cleanup_interval_sec: u64,
    /// 古いトランザクションの最大年齢（秒）
    pub max_transaction_age_sec: u64,
    /// 統計更新間隔（ミリ秒）
    pub stats_update_interval_ms: u64,
    /// ハードウェアアクセラレーション有効
    pub hardware_acceleration_enabled: bool,
    /// 適応型バッチ処理有効
    pub adaptive_batching_enabled: bool,
    /// 負荷閾値（高）
    pub high_load_threshold: f64,
    /// 負荷閾値（低）
    pub low_load_threshold: f64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_throughput: 100000,
            max_parallelism: 256,
            memory_pool_size: 1000000,
            batch_size: 1000,
            min_batch_size: 100,
            max_batch_size: 10000,
            batch_interval_ms: 10,
            processing_timeout_ms: 5000,
            memory_pool_cleanup_interval_sec: 300,
            max_transaction_age_sec: 3600,
            stats_update_interval_ms: 1000,
            hardware_acceleration_enabled: true,
            adaptive_batching_enabled: true,
            high_load_threshold: 0.8,
            low_load_threshold: 0.3,
        }
    }
}

/// エンジン統計
#[derive(Debug, Clone)]
pub struct EngineStats {
    /// 処理されたトランザクション数
    pub processed_transactions: u64,
    /// 成功したトランザクション数
    pub successful_transactions: u64,
    /// 失敗したトランザクション数
    pub failed_transactions: u64,
    /// 平均処理時間（ミリ秒）
    pub avg_processing_time_ms: f64,
    /// 最大処理時間（ミリ秒）
    pub max_processing_time_ms: u64,
    /// 最小処理時間（ミリ秒）
    pub min_processing_time_ms: u64,
    /// 現在のスループット（TPS）
    pub current_throughput: f64,
    /// 最大スループット（TPS）
    pub max_throughput: f64,
    /// 現在のバッチサイズ
    pub current_batch_size: usize,
    /// 現在の並列度
    pub current_parallelism: usize,
    /// 現在の負荷
    pub current_load: f64,
    /// メモリプール統計
    pub memory_pool_stats: MemoryPoolStats,
    /// 最終更新時刻
    pub last_updated: u64,
    /// 開始時刻
    pub start_time: u64,
    /// 稼働時間（秒）
    pub uptime_sec: u64,
}

impl Default for EngineStats {
    fn default() -> Self {
        let now = chrono::Utc::now().timestamp() as u64;

        Self {
            processed_transactions: 0,
            successful_transactions: 0,
            failed_transactions: 0,
            avg_processing_time_ms: 0.0,
            max_processing_time_ms: 0,
            min_processing_time_ms: u64::MAX,
            current_throughput: 0.0,
            max_throughput: 0.0,
            current_batch_size: 0,
            current_parallelism: 0,
            current_load: 0.0,
            memory_pool_stats: MemoryPoolStats {
                pending_count: 0,
                confirmed_count: 0,
                rejected_count: 0,
                total_count: 0,
                max_size: 0,
                current_size: 0,
                utilization: 0.0,
                last_cleanup: now,
            },
            last_updated: now,
            start_time: now,
            uptime_sec: 0,
        }
    }
}

impl HighThroughputEngine {
    /// 新しい高スループットエンジンを作成
    pub fn new(
        cross_shard_manager: Arc<CrossShardManager>,
        shard_manager: Arc<ShardManager>,
        network_tx: mpsc::Sender<NetworkMessage>,
        config: Option<EngineConfig>,
    ) -> Result<Self, Error> {
        let config = config.unwrap_or_default();

        // クロスシャード最適化器を作成
        let optimizer_config = OptimizerConfig {
            batch_size: config.batch_size,
            batch_interval_ms: config.batch_interval_ms,
            routing_update_interval_sec: 60,
            metrics_update_interval_sec: 30,
            max_parallel_executions: config.max_parallelism / 4,
            cache_expiry_sec: 300,
            max_retries: 3,
            retry_interval_ms: 1000,
        };

        let cross_shard_optimizer = Arc::new(CrossShardOptimizer::new(
            cross_shard_manager.clone(),
            shard_manager.clone(),
            network_tx.clone(),
            Some(optimizer_config),
        ));

        // 並列処理器を作成
        let processor_config = ProcessorConfig {
            max_parallelism: config.max_parallelism,
            max_queue_size: config.memory_pool_size,
            batch_size: config.batch_size,
            processing_timeout_ms: config.processing_timeout_ms,
            dependency_resolution_timeout_ms: config.processing_timeout_ms / 2,
            max_retries: 3,
            retry_interval_ms: 1000,
            dynamic_scaling_enabled: true,
            min_parallelism: config.max_parallelism / 8,
            high_load_threshold: config.high_load_threshold,
            low_load_threshold: config.low_load_threshold,
        };

        let parallel_processor = Arc::new(ParallelProcessor::new(
            cross_shard_manager.clone(),
            shard_manager.clone(),
            network_tx.clone(),
            Some(processor_config),
        ));

        // メモリプールを作成
        let memory_pool = Arc::new(RwLock::new(MemoryPool::new(config.memory_pool_size)));

        // セマフォアを作成
        let semaphore = Arc::new(Semaphore::new(config.max_parallelism));

        // エンジン統計を初期化
        let stats = Arc::new(RwLock::new(EngineStats::default()));

        Ok(Self {
            cross_shard_manager,
            shard_manager,
            network_tx,
            cross_shard_optimizer,
            parallel_processor,
            memory_pool,
            config,
            stats,
            processing_queue: Arc::new(Mutex::new(VecDeque::new())),
            semaphore,
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// エンジンを起動
    pub fn start(&self) -> Result<(), Error> {
        {
            let mut running = self.running.write().unwrap();
            if *running {
                return Err(Error::ValidationError(
                    "Engine is already running".to_string(),
                ));
            }
            *running = true;
        }

        info!(
            "Starting HighThroughputEngine with max throughput: {} TPS",
            self.config.max_throughput
        );

        // クロスシャード最適化器を起動
        self.cross_shard_optimizer.start();

        // 並列処理器を起動
        self.parallel_processor.start();

        // バッチ処理タスクを開始
        self.start_batch_processor();

        // メモリプールクリーンアップタスクを開始
        self.start_memory_pool_cleaner();

        // 統計更新タスクを開始
        self.start_stats_updater();

        // 適応型バッチ処理タスクを開始
        if self.config.adaptive_batching_enabled {
            self.start_adaptive_batcher();
        }

        Ok(())
    }

    /// エンジンを停止
    pub fn stop(&self) -> Result<(), Error> {
        let mut running = self.running.write().unwrap();
        if !*running {
            return Err(Error::ValidationError("Engine is not running".to_string()));
        }
        *running = false;

        info!("Stopping HighThroughputEngine");

        Ok(())
    }

    /// トランザクションを処理
    pub async fn process_transaction(&self, transaction: Transaction) -> Result<(), Error> {
        // メモリプールに追加
        {
            let mut pool = self.memory_pool.write().unwrap();
            pool.add_transaction(transaction.clone())?;
        }

        // 処理キューに追加
        {
            let mut queue = self.processing_queue.lock().unwrap();
            queue.push_back(transaction);
        }

        Ok(())
    }

    /// 複数のトランザクションを一括処理
    pub async fn process_transactions(&self, transactions: Vec<Transaction>) -> Result<(), Error> {
        if transactions.is_empty() {
            return Ok(());
        }

        info!("Processing batch of {} transactions", transactions.len());

        // メモリプールに追加
        {
            let mut pool = self.memory_pool.write().unwrap();

            for tx in &transactions {
                if let Err(e) = pool.add_transaction(tx.clone()) {
                    warn!("Failed to add transaction to memory pool: {}", e);
                }
            }
        }

        // 処理キューに追加
        {
            let mut queue = self.processing_queue.lock().unwrap();

            for tx in transactions {
                queue.push_back(tx);
            }
        }

        Ok(())
    }

    /// バッチ処理タスクを開始
    fn start_batch_processor(&self) {
        let processing_queue = self.processing_queue.clone();
        let parallel_processor = self.parallel_processor.clone();
        let memory_pool = self.memory_pool.clone();
        let stats = self.stats.clone();
        let config = self.config.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            while *running.read().unwrap() {
                // バッチ間隔を待機
                sleep(Duration::from_millis(config.batch_interval_ms)).await;

                // 現在のバッチサイズを取得
                let batch_size = {
                    let stats = stats.read().unwrap();
                    stats.current_batch_size
                };

                // キューからトランザクションを取得
                let batch = {
                    let mut queue = processing_queue.lock().unwrap();
                    let batch_size = std::cmp::min(queue.len(), batch_size);

                    let mut batch = Vec::with_capacity(batch_size);
                    for _ in 0..batch_size {
                        if let Some(tx) = queue.pop_front() {
                            batch.push(tx);
                        } else {
                            break;
                        }
                    }

                    batch
                };

                if batch.is_empty() {
                    continue;
                }

                // 処理開始時間
                let start_time = Instant::now();

                // バッチを処理
                let results = parallel_processor.process_transactions(batch.clone()).await;

                // 処理時間
                let processing_time = start_time.elapsed().as_millis() as u64;

                // 結果を処理
                if let Ok(results) = results {
                    let mut successful = 0;
                    let mut failed = 0;

                    let mut pool = memory_pool.write().unwrap();

                    for (i, result) in results.iter().enumerate() {
                        if i >= batch.len() {
                            break;
                        }

                        let tx_id = &batch[i].id;

                        match result {
                            Ok(_) => {
                                successful += 1;
                                let _ = pool.confirm_transaction(tx_id);
                            }
                            Err(e) => {
                                failed += 1;
                                let _ = pool.reject_transaction(tx_id, &format!("{:?}", e));
                            }
                        }
                    }

                    // 統計を更新
                    {
                        let mut stats = stats.write().unwrap();
                        stats.processed_transactions += batch.len() as u64;
                        stats.successful_transactions += successful as u64;
                        stats.failed_transactions += failed as u64;

                        // 平均処理時間を更新
                        let total_time = stats.avg_processing_time_ms
                            * (stats.processed_transactions - batch.len() as u64) as f64
                            + processing_time as f64;
                        stats.avg_processing_time_ms =
                            total_time / stats.processed_transactions as f64;

                        // 最大・最小処理時間を更新
                        stats.max_processing_time_ms =
                            stats.max_processing_time_ms.max(processing_time);
                        if processing_time > 0 {
                            stats.min_processing_time_ms =
                                stats.min_processing_time_ms.min(processing_time);
                        }
                    }
                } else if let Err(e) = results {
                    error!("Failed to process batch: {}", e);
                }
            }
        });
    }

    /// メモリプールクリーンアップタスクを開始
    fn start_memory_pool_cleaner(&self) {
        let memory_pool = self.memory_pool.clone();
        let config = self.config.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            while *running.read().unwrap() {
                // クリーンアップ間隔を待機
                sleep(Duration::from_secs(config.memory_pool_cleanup_interval_sec)).await;

                // メモリプールをクリーンアップ
                let removed = {
                    let mut pool = memory_pool.write().unwrap();
                    pool.cleanup(config.max_transaction_age_sec)
                };

                if removed > 0 {
                    debug!("Cleaned up {} old transactions from memory pool", removed);
                }
            }
        });
    }

    /// 統計更新タスクを開始
    fn start_stats_updater(&self) {
        let stats = self.stats.clone();
        let memory_pool = self.memory_pool.clone();
        let parallel_processor = self.parallel_processor.clone();
        let config = self.config.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut last_processed = 0;
            let mut last_time = chrono::Utc::now().timestamp() as u64;

            while *running.read().unwrap() {
                // 更新間隔を待機
                sleep(Duration::from_millis(config.stats_update_interval_ms)).await;

                let now = chrono::Utc::now().timestamp() as u64;
                let elapsed = now - last_time;

                if elapsed > 0 {
                    // メモリプール統計を取得
                    let pool_stats = {
                        let pool = memory_pool.read().unwrap();
                        pool.get_stats()
                    };

                    // 並列処理器統計を取得
                    let processor_stats = parallel_processor.get_stats();

                    // エンジン統計を更新
                    {
                        let mut stats_guard = stats.write().unwrap();

                        // スループットを計算
                        let new_processed = stats_guard.processed_transactions;
                        let processed_diff = new_processed - last_processed;

                        stats_guard.current_throughput = processed_diff as f64 / elapsed as f64;
                        stats_guard.max_throughput = stats_guard
                            .max_throughput
                            .max(stats_guard.current_throughput);

                        // 並列度と負荷を更新
                        stats_guard.current_parallelism = processor_stats.current_parallelism;
                        stats_guard.current_load = processor_stats.current_load;

                        // メモリプール統計を更新
                        stats_guard.memory_pool_stats = pool_stats;

                        // 稼働時間を更新
                        stats_guard.uptime_sec = now - stats_guard.start_time;

                        // 最終更新時刻を更新
                        stats_guard.last_updated = now;

                        // 値を更新
                        last_processed = new_processed;
                        last_time = now;
                    }
                }
            }
        });
    }

    /// 適応型バッチ処理タスクを開始
    fn start_adaptive_batcher(&self) {
        let stats = self.stats.clone();
        let config = self.config.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            while *running.read().unwrap() {
                // 更新間隔を待機
                sleep(Duration::from_secs(5)).await;

                // 現在の負荷とスループットを取得
                let (current_load, current_throughput, current_batch_size) = {
                    let stats = stats.read().unwrap();
                    (
                        stats.current_load,
                        stats.current_throughput,
                        stats.current_batch_size,
                    )
                };

                // バッチサイズを調整
                let new_batch_size = if current_load > config.high_load_threshold {
                    // 負荷が高い場合、バッチサイズを減少
                    let new_size = (current_batch_size as f64 * 0.8) as usize;
                    new_size.max(config.min_batch_size)
                } else if current_load < config.low_load_threshold
                    && current_throughput < config.max_throughput as f64 * 0.8
                {
                    // 負荷が低く、スループットが目標より低い場合、バッチサイズを増加
                    let new_size = (current_batch_size as f64 * 1.2) as usize;
                    new_size.min(config.max_batch_size)
                } else {
                    // 現状維持
                    current_batch_size
                };

                if new_batch_size != current_batch_size {
                    // バッチサイズを更新
                    let mut stats = stats.write().unwrap();
                    stats.current_batch_size = new_batch_size;

                    debug!(
                        "Adjusted batch size from {} to {} (load: {:.2}, throughput: {:.2} TPS)",
                        current_batch_size, new_batch_size, current_load, current_throughput
                    );
                }
            }
        });
    }

    /// 現在のエンジン統計を取得
    pub fn get_stats(&self) -> EngineStats {
        let stats = self.stats.read().unwrap();
        stats.clone()
    }

    /// エンジン設定を更新
    pub fn update_config(&mut self, config: EngineConfig) -> Result<(), Error> {
        self.config = config;
        Ok(())
    }

    /// 現在のエンジン設定を取得
    pub fn get_config(&self) -> EngineConfig {
        self.config.clone()
    }

    /// ベンチマークを実行
    pub async fn run_benchmark(
        &self,
        transaction_count: usize,
        concurrency: usize,
        timeout_sec: u64,
    ) -> Result<BenchmarkResult, Error> {
        info!(
            "Starting benchmark with {} transactions, concurrency: {}, timeout: {}s",
            transaction_count, concurrency, timeout_sec
        );

        // テストトランザクションを生成
        let transactions = self.generate_test_transactions(transaction_count).await?;

        // ベンチマーク開始
        let start_time = Instant::now();

        // トランザクションを処理
        self.process_transactions(transactions).await?;

        // タイムアウトを設定
        let timeout = Duration::from_secs(timeout_sec);

        // 処理完了を待機
        let mut completed = false;
        let mut last_processed = 0;
        let mut unchanged_count = 0;

        while !completed && start_time.elapsed() < timeout {
            sleep(Duration::from_millis(100)).await;

            let stats = self.get_stats();
            let processed = stats.processed_transactions;

            if processed >= transaction_count as u64 {
                completed = true;
                break;
            }

            // 処理が停滞していないかチェック
            if processed == last_processed {
                unchanged_count += 1;

                // 10回連続で変化がない場合はタイムアウト
                if unchanged_count >= 10 {
                    warn!("Benchmark processing stalled at {} transactions", processed);
                    break;
                }
            } else {
                unchanged_count = 0;
                last_processed = processed;
            }
        }

        // 合計実行時間
        let total_time = start_time.elapsed();
        let total_time_ms = total_time.as_millis() as u64;

        // 統計を取得
        let stats = self.get_stats();

        // 結果を作成
        let result = BenchmarkResult {
            transaction_count,
            successful_transactions: stats.successful_transactions as usize,
            failed_transactions: stats.failed_transactions as usize,
            total_time_ms,
            avg_transaction_time_ms: stats.avg_processing_time_ms,
            transactions_per_second: if total_time_ms > 0 {
                (stats.successful_transactions as f64) / (total_time_ms as f64 / 1000.0)
            } else {
                0.0
            },
            min_transaction_time_ms: stats.min_transaction_time_ms,
            max_transaction_time_ms: stats.max_processing_time_ms,
            optimization_enabled: true,
        };

        info!(
            "Benchmark completed: {} TPS, avg time: {:.2}ms, success rate: {:.2}%",
            result.transactions_per_second,
            result.avg_transaction_time_ms,
            (result.successful_transactions as f64 / transaction_count as f64) * 100.0
        );

        Ok(result)
    }

    /// テストトランザクションを生成
    async fn generate_test_transactions(&self, count: usize) -> Result<Vec<Transaction>, Error> {
        // アクティブなシャードを取得
        let shards = self.shard_manager.get_active_shards().await?;

        if shards.is_empty() {
            return Err(Error::ValidationError("No active shards found".to_string()));
        }

        let mut transactions = Vec::with_capacity(count);

        for i in 0..count {
            // シャードをランダムに選択
            let shard_index = i % shards.len();
            let shard = &shards[shard_index];

            // トランザクションを作成
            let tx = Transaction {
                id: format!("bench-tx-{}", i),
                from: format!("bench-addr-{}", i % 1000),
                to: format!("bench-addr-{}", (i + 1) % 1000),
                amount: "1.0".to_string(),
                fee: "0.001".to_string(),
                data: None,
                nonce: i as u64,
                timestamp: chrono::Utc::now().timestamp() as u64,
                signature: "benchmark-signature".to_string(),
                status: TransactionStatus::Pending,
                shard_id: shard.id.clone(),
                block_hash: None,
                block_height: None,
                parent_id: None,
            };

            transactions.push(tx);
        }

        Ok(transactions)
    }
}

/// ベンチマーク結果
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// トランザクション数
    pub transaction_count: usize,
    /// 成功したトランザクション数
    pub successful_transactions: usize,
    /// 失敗したトランザクション数
    pub failed_transactions: usize,
    /// 合計実行時間（ミリ秒）
    pub total_time_ms: u64,
    /// 平均トランザクション時間（ミリ秒）
    pub avg_transaction_time_ms: f64,
    /// 1秒あたりのトランザクション数
    pub transactions_per_second: f64,
    /// 最小トランザクション時間（ミリ秒）
    pub min_transaction_time_ms: u64,
    /// 最大トランザクション時間（ミリ秒）
    pub max_transaction_time_ms: u64,
    /// 最適化が有効かどうか
    pub optimization_enabled: bool,
}

// 単体テスト
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_memory_pool() {
        // メモリプールを作成
        let mut pool = MemoryPool::new(1000);

        // テストトランザクションを作成
        let tx = Transaction {
            id: "test-tx-1".to_string(),
            from: "test-addr-1".to_string(),
            to: "test-addr-2".to_string(),
            amount: "1.0".to_string(),
            fee: "0.001".to_string(),
            data: None,
            nonce: 1,
            timestamp: chrono::Utc::now().timestamp() as u64,
            signature: "test-signature".to_string(),
            status: TransactionStatus::Pending,
            shard_id: "shard-1".to_string(),
            block_hash: None,
            block_height: None,
            parent_id: None,
        };

        // トランザクションを追加
        pool.add_transaction(tx.clone()).unwrap();

        // トランザクションが追加されたことを確認
        assert_eq!(pool.pending_transactions.len(), 1);
        assert_eq!(pool.current_size, 1);

        // アドレス別トランザクションを確認
        let addr_txs = pool.get_address_transactions(&tx.from);
        assert_eq!(addr_txs.len(), 1);
        assert_eq!(addr_txs[0].id, tx.id);

        // シャード別トランザクションを確認
        let shard_txs = pool.get_shard_transactions(&tx.shard_id);
        assert_eq!(shard_txs.len(), 1);
        assert_eq!(shard_txs[0].id, tx.id);

        // トランザクションを確認済みに移動
        pool.confirm_transaction(&tx.id).unwrap();

        // 確認済みトランザクションを確認
        assert_eq!(pool.pending_transactions.len(), 0);
        assert_eq!(pool.confirmed_transactions.len(), 1);

        // 統計を確認
        let stats = pool.get_stats();
        assert_eq!(stats.pending_count, 0);
        assert_eq!(stats.confirmed_count, 1);
        assert_eq!(stats.rejected_count, 0);
        assert_eq!(stats.total_count, 1);
    }
}
