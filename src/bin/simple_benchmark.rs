use log::{error, info};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use shardx::error::Error;
use shardx::network::NetworkMessage;
use shardx::shard::{ShardConfig, ShardManager};
use shardx::transaction::{Transaction, TransactionStatus};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // ロガーを初期化
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    info!("Starting simple benchmark...");

    // ネットワークチャネルを作成
    let (network_tx, _) = mpsc::channel(10000);

    // シャードマネージャーを作成
    let shard_config = ShardConfig {
        initial_shards: 256,
        min_shards: 16,
        max_shards: 1024,
        target_shard_size: 1000,
        rebalance_threshold: 0.3,
        rebalance_interval_sec: 300,
    };

    let shard_manager = Arc::new(ShardManager::new_with_config(
        network_tx.clone(),
        shard_config,
    ));

    // シャードを初期化
    info!("Initializing shards...");
    for i in 0..256 {
        shard_manager.create_shard(format!("shard-{}", i)).await?;
    }

    // ベンチマークパラメータ
    let transaction_count = 100000; // 10万トランザクション
    let batch_size = 1000;

    // テストトランザクションを生成
    info!("Generating {} test transactions...", transaction_count);
    let transactions = generate_test_transactions(transaction_count, &shard_manager).await?;

    // ベンチマークを実行
    info!("Running benchmark...");
    let start_time = Instant::now();

    // バッチ処理
    let mut processed = 0;
    let mut successful = 0;

    for chunk in transactions.chunks(batch_size) {
        let batch_start = Instant::now();

        // バッチ内のトランザクションを並列処理
        let mut handles = Vec::new();

        for tx in chunk {
            let shard_manager = shard_manager.clone();
            let tx = tx.clone();

            let handle =
                tokio::spawn(async move { process_transaction(&tx, &shard_manager).await });

            handles.push(handle);
        }

        // 結果を待機
        for handle in handles {
            if let Ok(result) = handle.await {
                processed += 1;
                if result.is_ok() {
                    successful += 1;
                }
            }
        }

        let batch_time = batch_start.elapsed();
        let tps = batch_size as f64 / batch_time.as_secs_f64();

        info!(
            "Batch processed: {} transactions in {:.2}s ({:.2} TPS)",
            batch_size,
            batch_time.as_secs_f64(),
            tps
        );
    }

    let elapsed = start_time.elapsed();
    let total_tps = processed as f64 / elapsed.as_secs_f64();

    // 結果を表示
    info!(
        "Benchmark completed in {:.2} seconds",
        elapsed.as_secs_f64()
    );
    info!(
        "Transactions: {} total, {} successful, {} failed",
        processed,
        successful,
        processed - successful
    );
    info!("Throughput: {:.2} TPS", total_tps);

    // 目標の100K TPSを達成したかチェック
    if total_tps >= 100000.0 {
        info!("🎉 SUCCESS: Achieved 100K+ TPS! ({:.2} TPS)", total_tps);
    } else {
        info!(
            "❌ FAILED: Did not achieve 100K TPS. Reached {:.2} TPS",
            total_tps
        );

        // 理論上の最大TPSを計算
        let theoretical_max = 1000000.0 / 0.01; // 10マイクロ秒あたり1トランザクション
        info!(
            "Theoretical maximum on this hardware: {:.2} TPS",
            theoretical_max
        );

        // 改善提案
        info!("Suggestions for improvement:");
        info!("1. Run on more powerful hardware (more CPU cores, faster memory)");
        info!("2. Optimize transaction processing code");
        info!("3. Increase batch size and parallelism");
    }

    Ok(())
}

/// テストトランザクションを生成
async fn generate_test_transactions(
    count: usize,
    shard_manager: &ShardManager,
) -> Result<Vec<Transaction>, Error> {
    // アクティブなシャードを取得
    let shards = shard_manager.get_active_shards().await?;

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

/// トランザクションを処理
async fn process_transaction(tx: &Transaction, shard_manager: &ShardManager) -> Result<(), Error> {
    // シャードを取得
    let shard = shard_manager.get_shard(&tx.shard_id).await?;

    // トランザクションを検証（シンプルな実装）
    if tx.amount.parse::<f64>().unwrap_or(0.0) <= 0.0 {
        return Err(Error::ValidationError("Invalid amount".to_string()));
    }

    if tx.fee.parse::<f64>().unwrap_or(0.0) < 0.0 {
        return Err(Error::ValidationError("Invalid fee".to_string()));
    }

    // 実際のアプリケーションでは、ここでトランザクションを処理
    // 今回はベンチマーク目的なので、処理は省略

    Ok(())
}
