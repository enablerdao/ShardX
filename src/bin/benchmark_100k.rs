use std::sync::Arc;
use tokio::sync::mpsc;
use log::{info, error};
use std::time::{Duration, Instant};

use shardx::error::Error;
use shardx::transaction::{HighThroughputEngine, EngineConfig, CrossShardManager, CrossShardOptimizer};
use shardx::shard::{ShardManager, ShardConfig};
use shardx::network::{NetworkManager, NetworkMessage, NetworkConfig};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // ロガーを初期化
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    info!("Starting 100K TPS benchmark...");

    // ネットワークマネージャーを作成
    let (network_tx, network_rx) = mpsc::channel(10000);
    let network_config = NetworkConfig {
        bind_address: "0.0.0.0:54868".to_string(),
        peers: vec![],
        max_connections: 1000,
        connection_timeout_sec: 30,
        ping_interval_sec: 60,
        max_message_size: 10 * 1024 * 1024, // 10MB
    };
    
    let network_manager = NetworkManager::new(network_config, network_rx)?;
    
    // シャードマネージャーを作成
    let shard_config = ShardConfig {
        initial_shards: 256,
        min_shards: 16,
        max_shards: 1024,
        target_shard_size: 1000,
        rebalance_threshold: 0.3,
        rebalance_interval_sec: 300,
    };
    
    let shard_manager = Arc::new(ShardManager::new_with_config(network_tx.clone(), shard_config));
    
    // クロスシャードマネージャーを作成
    let cross_shard_manager = Arc::new(CrossShardManager::new(network_tx.clone()));
    
    // 高スループットエンジンを作成
    let engine_config = EngineConfig {
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
    };
    
    let engine = HighThroughputEngine::new(
        cross_shard_manager.clone(),
        shard_manager.clone(),
        network_tx.clone(),
        Some(engine_config),
    )?;
    
    // エンジンを起動
    engine.start()?;
    
    // シャードを初期化
    info!("Initializing shards...");
    for i in 0..256 {
        shard_manager.create_shard(format!("shard-{}", i)).await?;
    }
    
    // ベンチマークパラメータ
    let transaction_count = 1000000; // 100万トランザクション
    let concurrency = 256;
    let timeout_sec = 60; // 1分
    
    // ベンチマークを実行
    info!("Running benchmark with {} transactions...", transaction_count);
    let start_time = Instant::now();
    
    let result = engine.run_benchmark(transaction_count, concurrency, timeout_sec).await?;
    
    let elapsed = start_time.elapsed();
    
    // 結果を表示
    info!("Benchmark completed in {:.2} seconds", elapsed.as_secs_f64());
    info!("Transactions: {} total, {} successful, {} failed",
        result.transaction_count, result.successful_transactions, result.failed_transactions);
    info!("Throughput: {:.2} TPS", result.transactions_per_second);
    info!("Average transaction time: {:.2} ms", result.avg_transaction_time_ms);
    info!("Min/Max transaction time: {} ms / {} ms",
        result.min_transaction_time_ms, result.max_transaction_time_ms);
    
    // 目標の100K TPSを達成したかチェック
    if result.transactions_per_second >= 100000.0 {
        info!("🎉 SUCCESS: Achieved 100K+ TPS! ({:.2} TPS)", result.transactions_per_second);
    } else {
        info!("❌ FAILED: Did not achieve 100K TPS. Reached {:.2} TPS", result.transactions_per_second);
    }
    
    // エンジンを停止
    engine.stop()?;
    
    Ok(())
}