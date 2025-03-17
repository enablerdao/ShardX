use clap::{App, Arg};
use log::{error, info};
use prettytable::{cell, row, Table};
use std::sync::Arc;
use tokio::sync::mpsc;

use shardx::error::Error;
use shardx::network::NetworkMessage;
use shardx::shard::ShardManager;
use shardx::transaction::{CrossShardBenchmarker, CrossShardManager, OptimizerConfig};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // コマンドライン引数を解析
    let matches = App::new("ShardX Cross-Shard Transaction Benchmark")
        .version("1.0")
        .author("ShardX Team")
        .about("Benchmarks cross-shard transaction performance")
        .arg(
            Arg::with_name("transactions")
                .short("t")
                .long("transactions")
                .value_name("COUNT")
                .help("Number of transactions to process")
                .default_value("1000"),
        )
        .arg(
            Arg::with_name("concurrency")
                .short("c")
                .long("concurrency")
                .value_name("COUNT")
                .help("Number of concurrent transactions")
                .default_value("50"),
        )
        .arg(
            Arg::with_name("timeout")
                .short("s")
                .long("timeout")
                .value_name("SECONDS")
                .help("Timeout in seconds")
                .default_value("60"),
        )
        .arg(
            Arg::with_name("compare")
                .long("compare")
                .help("Run comparison between optimized and unoptimized"),
        )
        .arg(
            Arg::with_name("batch-size")
                .long("batch-size")
                .value_name("SIZE")
                .help("Batch size for optimizer")
                .default_value("50"),
        )
        .arg(
            Arg::with_name("batch-interval")
                .long("batch-interval")
                .value_name("MS")
                .help("Batch interval in milliseconds")
                .default_value("100"),
        )
        .get_matches();

    // 引数を取得
    let transaction_count = matches
        .value_of("transactions")
        .unwrap()
        .parse::<usize>()
        .unwrap_or(1000);

    let concurrency = matches
        .value_of("concurrency")
        .unwrap()
        .parse::<usize>()
        .unwrap_or(50);

    let timeout_sec = matches
        .value_of("timeout")
        .unwrap()
        .parse::<u64>()
        .unwrap_or(60);

    let compare = matches.is_present("compare");

    let batch_size = matches
        .value_of("batch-size")
        .unwrap()
        .parse::<usize>()
        .unwrap_or(50);

    let batch_interval_ms = matches
        .value_of("batch-interval")
        .unwrap()
        .parse::<u64>()
        .unwrap_or(100);

    // 必要なコンポーネントを初期化
    let (network_tx, _network_rx) = mpsc::channel(1000);
    let shard_manager = Arc::new(ShardManager::new());
    let cross_shard_manager = Arc::new(CrossShardManager::new(
        shard_manager.clone(),
        network_tx.clone(),
    ));

    // テスト用のシャードを追加
    for i in 1..=5 {
        let shard = shardx::shard::ShardInfo {
            id: format!("shard{}", i),
            name: format!("Shard {}", i),
            validators: 10,
            height: 1000,
            tps: 1000.0,
            status: shardx::shard::ShardStatus::Active,
        };

        shard_manager.add_shard(shard).await?;
    }

    // 最適化設定
    let optimizer_config = OptimizerConfig {
        batch_size,
        batch_interval_ms,
        routing_update_interval_sec: 60,
        metrics_update_interval_sec: 30,
        max_parallel_executions: concurrency,
        cache_expiry_sec: 300,
        max_retries: 3,
        retry_interval_ms: 1000,
    };

    // ベンチマーカーを作成
    let benchmarker = CrossShardBenchmarker::new(
        cross_shard_manager.clone(),
        shard_manager.clone(),
        network_tx.clone(),
        true, // 最適化を有効化
    )?;

    if compare {
        // 最適化あり/なしの比較ベンチマークを実行
        info!("Running comparison benchmark...");

        let (unoptimized, optimized) = benchmarker
            .run_comparison_benchmark(transaction_count, concurrency, timeout_sec)
            .await?;

        // 結果を表示
        print_comparison_results(&unoptimized, &optimized);
    } else {
        // 通常のベンチマークを実行
        info!(
            "Running benchmark with {} transactions, concurrency: {}, timeout: {}s",
            transaction_count, concurrency, timeout_sec
        );

        let result = benchmarker
            .run_benchmark(transaction_count, concurrency, timeout_sec)
            .await?;

        // 結果を表示
        print_benchmark_results(&result);
    }

    Ok(())
}

/// ベンチマーク結果を表示
fn print_benchmark_results(result: &shardx::transaction::BenchmarkResult) {
    let mut table = Table::new();

    table.add_row(row!["Metric", "Value"]);
    table.add_row(row!["Transaction Count", result.transaction_count]);
    table.add_row(row![
        "Successful Transactions",
        result.successful_transactions
    ]);
    table.add_row(row!["Failed Transactions", result.failed_transactions]);
    table.add_row(row![
        "Success Rate",
        format!(
            "{:.2}%",
            (result.successful_transactions as f64 / result.transaction_count as f64) * 100.0
        )
    ]);
    table.add_row(row!["Total Time", format!("{} ms", result.total_time_ms)]);
    table.add_row(row![
        "Avg Transaction Time",
        format!("{:.2} ms", result.avg_transaction_time_ms)
    ]);
    table.add_row(row![
        "Min Transaction Time",
        format!("{} ms", result.min_transaction_time_ms)
    ]);
    table.add_row(row![
        "Max Transaction Time",
        format!("{} ms", result.max_transaction_time_ms)
    ]);
    table.add_row(row![
        "Transactions Per Second",
        format!("{:.2}", result.transactions_per_second)
    ]);
    table.add_row(row!["Optimization Enabled", result.optimization_enabled]);

    table.printstd();
}

/// 比較ベンチマーク結果を表示
fn print_comparison_results(
    unoptimized: &shardx::transaction::BenchmarkResult,
    optimized: &shardx::transaction::BenchmarkResult,
) {
    let mut table = Table::new();

    // TPS改善率を計算
    let tps_improvement = (optimized.transactions_per_second - unoptimized.transactions_per_second)
        / unoptimized.transactions_per_second
        * 100.0;

    // レイテンシ改善率を計算
    let latency_improvement = (unoptimized.avg_transaction_time_ms
        - optimized.avg_transaction_time_ms)
        / unoptimized.avg_transaction_time_ms
        * 100.0;

    table.add_row(row![
        "Metric",
        "Without Optimization",
        "With Optimization",
        "Improvement"
    ]);

    table.add_row(row![
        "Transaction Count",
        unoptimized.transaction_count,
        optimized.transaction_count,
        "-"
    ]);

    table.add_row(row![
        "Successful Transactions",
        unoptimized.successful_transactions,
        optimized.successful_transactions,
        format!(
            "{:.2}%",
            (optimized.successful_transactions as f64 - unoptimized.successful_transactions as f64)
                / unoptimized.successful_transactions as f64
                * 100.0
        )
    ]);

    table.add_row(row![
        "Success Rate",
        format!(
            "{:.2}%",
            (unoptimized.successful_transactions as f64 / unoptimized.transaction_count as f64)
                * 100.0
        ),
        format!(
            "{:.2}%",
            (optimized.successful_transactions as f64 / optimized.transaction_count as f64) * 100.0
        ),
        format!(
            "{:.2}%",
            (optimized.successful_transactions as f64 / optimized.transaction_count as f64) * 100.0
                - (unoptimized.successful_transactions as f64
                    / unoptimized.transaction_count as f64)
                    * 100.0
        )
    ]);

    table.add_row(row![
        "Total Time",
        format!("{} ms", unoptimized.total_time_ms),
        format!("{} ms", optimized.total_time_ms),
        format!(
            "{:.2}%",
            (unoptimized.total_time_ms as f64 - optimized.total_time_ms as f64)
                / unoptimized.total_time_ms as f64
                * 100.0
        )
    ]);

    table.add_row(row![
        "Avg Transaction Time",
        format!("{:.2} ms", unoptimized.avg_transaction_time_ms),
        format!("{:.2} ms", optimized.avg_transaction_time_ms),
        format!("{:.2}%", latency_improvement)
    ]);

    table.add_row(row![
        "Transactions Per Second",
        format!("{:.2}", unoptimized.transactions_per_second),
        format!("{:.2}", optimized.transactions_per_second),
        format!("{:.2}%", tps_improvement)
    ]);

    table.printstd();

    println!("\nSummary:");
    println!("- TPS improved by {:.2}%", tps_improvement);
    println!("- Latency reduced by {:.2}%", latency_improvement);

    if tps_improvement > 0.0 && latency_improvement > 0.0 {
        println!("✅ Optimization is effective");
    } else {
        println!("❌ Optimization is not effective");
    }
}
