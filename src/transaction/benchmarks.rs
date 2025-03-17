use log::{debug, error, info, warn};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use crate::error::Error;
use crate::network::NetworkMessage;
use crate::shard::ShardManager;
use crate::transaction::{
    CrossShardManager, CrossShardOptimizer, OptimizerConfig, Transaction, TransactionStatus,
};

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

/// クロスシャードトランザクションベンチマーカー
pub struct CrossShardBenchmarker {
    /// クロスシャードマネージャー
    cross_shard_manager: Arc<CrossShardManager>,
    /// クロスシャード最適化器（オプション）
    cross_shard_optimizer: Option<Arc<CrossShardOptimizer>>,
    /// シャードマネージャー
    shard_manager: Arc<ShardManager>,
    /// ネットワークメッセージ送信チャネル
    network_tx: mpsc::Sender<NetworkMessage>,
}

impl CrossShardBenchmarker {
    /// 新しいベンチマーカーを作成
    pub fn new(
        cross_shard_manager: Arc<CrossShardManager>,
        shard_manager: Arc<ShardManager>,
        network_tx: mpsc::Sender<NetworkMessage>,
        use_optimizer: bool,
    ) -> Result<Self, Error> {
        let cross_shard_optimizer = if use_optimizer {
            let optimizer = CrossShardOptimizer::new(
                cross_shard_manager.clone(),
                shard_manager.clone(),
                network_tx.clone(),
                None,
            );

            Some(Arc::new(optimizer))
        } else {
            None
        };

        Ok(Self {
            cross_shard_manager,
            cross_shard_optimizer,
            shard_manager,
            network_tx,
        })
    }

    /// ベンチマークを実行
    pub async fn run_benchmark(
        &self,
        transaction_count: usize,
        concurrency: usize,
        timeout_sec: u64,
    ) -> Result<BenchmarkResult, Error> {
        info!("Starting cross-shard transaction benchmark with {} transactions, concurrency: {}, timeout: {}s",
            transaction_count, concurrency, timeout_sec);

        // 最適化器を起動（存在する場合）
        if let Some(optimizer) = &self.cross_shard_optimizer {
            optimizer.start();
        }

        // テストトランザクションを生成
        let transactions = self.generate_test_transactions(transaction_count).await?;

        // ベンチマーク開始
        let start_time = Instant::now();

        // トランザクション実行結果を追跡
        let mut successful_transactions = 0;
        let mut failed_transactions = 0;
        let mut transaction_times = Vec::with_capacity(transaction_count);

        // セマフォアを使用して並行実行を制限
        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));

        // タイムアウトを設定
        let timeout = Duration::from_secs(timeout_sec);

        // トランザクションを実行
        let mut handles = Vec::with_capacity(transaction_count);

        for tx in transactions {
            let permit = semaphore.clone().acquire_owned().await.unwrap();

            let cross_shard_manager = self.cross_shard_manager.clone();
            let cross_shard_optimizer = self.cross_shard_optimizer.clone();

            let handle = tokio::spawn(async move {
                let tx_start_time = Instant::now();

                let result = if let Some(optimizer) = &cross_shard_optimizer {
                    // 最適化器を使用
                    optimizer.enqueue_transaction(tx.clone())
                } else {
                    // 直接クロスシャードマネージャーを使用
                    cross_shard_manager.start_transaction(tx).await.map(|_| ())
                };

                let tx_time = tx_start_time.elapsed();

                (result, tx_time.as_millis() as u64)
            });

            handles.push(handle);
        }

        // タイムアウト付きで結果を待機
        for handle in handles {
            match tokio::time::timeout(timeout, handle).await {
                Ok(Ok((result, tx_time))) => {
                    transaction_times.push(tx_time);

                    match result {
                        Ok(_) => successful_transactions += 1,
                        Err(_) => failed_transactions += 1,
                    }
                }
                Ok(Err(_)) => {
                    // ジョインエラー
                    failed_transactions += 1;
                }
                Err(_) => {
                    // タイムアウト
                    failed_transactions += 1;
                }
            }
        }

        // 合計実行時間
        let total_time = start_time.elapsed();
        let total_time_ms = total_time.as_millis() as u64;

        // 統計を計算
        let avg_transaction_time_ms = if !transaction_times.is_empty() {
            transaction_times.iter().sum::<u64>() as f64 / transaction_times.len() as f64
        } else {
            0.0
        };

        let min_transaction_time_ms = transaction_times.iter().min().copied().unwrap_or(0);
        let max_transaction_time_ms = transaction_times.iter().max().copied().unwrap_or(0);

        let transactions_per_second = if total_time_ms > 0 {
            (successful_transactions as f64) / (total_time_ms as f64 / 1000.0)
        } else {
            0.0
        };

        // 結果を作成
        let result = BenchmarkResult {
            transaction_count,
            successful_transactions,
            failed_transactions,
            total_time_ms,
            avg_transaction_time_ms,
            transactions_per_second,
            min_transaction_time_ms,
            max_transaction_time_ms,
            optimization_enabled: self.cross_shard_optimizer.is_some(),
        };

        info!(
            "Benchmark completed: {} TPS, avg time: {:.2}ms, success rate: {:.2}%",
            result.transactions_per_second,
            result.avg_transaction_time_ms,
            (result.successful_transactions as f64 / transaction_count as f64) * 100.0
        );

        Ok(result)
    }

    /// 最適化設定を更新
    pub fn update_optimizer_config(&self, config: OptimizerConfig) -> Result<(), Error> {
        if let Some(optimizer) = &self.cross_shard_optimizer {
            // 実際の実装では、最適化器の設定を更新
            // ここでは簡易的な実装として、何もしない
            Ok(())
        } else {
            Err(Error::ValidationError(
                "Optimizer is not enabled".to_string(),
            ))
        }
    }

    /// テストトランザクションを生成
    async fn generate_test_transactions(&self, count: usize) -> Result<Vec<Transaction>, Error> {
        // アクティブなシャードを取得
        let shards = self.shard_manager.get_active_shards().await?;

        if shards.len() < 2 {
            return Err(Error::ValidationError(
                "At least 2 active shards are required for cross-shard benchmark".to_string(),
            ));
        }

        let mut transactions = Vec::with_capacity(count);

        for i in 0..count {
            // 送信元と送信先のシャードをランダムに選択
            let from_shard_index = i % shards.len();
            let to_shard_index = (i + 1) % shards.len();

            let from_shard = &shards[from_shard_index];
            let to_shard = &shards[to_shard_index];

            // トランザクションを作成
            let tx = Transaction {
                id: format!("bench-tx-{}", i),
                from: format!("bench-addr-{}", i),
                to: format!("bench-addr-{}", i + 1),
                amount: "1.0".to_string(),
                fee: "0.001".to_string(),
                data: None,
                nonce: i as u64,
                timestamp: chrono::Utc::now().timestamp() as u64,
                signature: "benchmark-signature".to_string(),
                status: TransactionStatus::Pending,
                shard_id: from_shard.id.clone(),
                block_hash: None,
                block_height: None,
                parent_id: None,
            };

            transactions.push(tx);
        }

        Ok(transactions)
    }

    /// 最適化の有無による比較ベンチマークを実行
    pub async fn run_comparison_benchmark(
        &self,
        transaction_count: usize,
        concurrency: usize,
        timeout_sec: u64,
    ) -> Result<(BenchmarkResult, BenchmarkResult), Error> {
        // 最適化なしのベンチマーカーを作成
        let unoptimized_benchmarker = CrossShardBenchmarker::new(
            self.cross_shard_manager.clone(),
            self.shard_manager.clone(),
            self.network_tx.clone(),
            false,
        )?;

        // 最適化ありのベンチマーカーを作成
        let optimized_benchmarker = CrossShardBenchmarker::new(
            self.cross_shard_manager.clone(),
            self.shard_manager.clone(),
            self.network_tx.clone(),
            true,
        )?;

        // 最適化なしのベンチマークを実行
        info!("Running benchmark without optimization...");
        let unoptimized_result = unoptimized_benchmarker
            .run_benchmark(transaction_count, concurrency, timeout_sec)
            .await?;

        // 少し待機してシステムを安定化
        tokio::time::sleep(Duration::from_secs(5)).await;

        // 最適化ありのベンチマークを実行
        info!("Running benchmark with optimization...");
        let optimized_result = optimized_benchmarker
            .run_benchmark(transaction_count, concurrency, timeout_sec)
            .await?;

        // 結果を比較
        let improvement = (optimized_result.transactions_per_second
            - unoptimized_result.transactions_per_second)
            / unoptimized_result.transactions_per_second
            * 100.0;

        info!(
            "Benchmark comparison: Optimization improved TPS by {:.2}%",
            improvement
        );

        Ok((unoptimized_result, optimized_result))
    }
}
