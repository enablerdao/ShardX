use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shardx::transaction::{Transaction, TransactionStatus};
use shardx::transaction::parallel_processor::{ParallelProcessor, ProcessorConfig};
use std::sync::Arc;
use std::time::{Duration, Instant};
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

// トランザクションをランダムに生成する関数
fn generate_random_transaction() -> Transaction {
    let mut rng = thread_rng();
    
    // ランダムな文字列を生成
    let from = (0..20).map(|_| rng.sample(Alphanumeric) as char).collect::<String>();
    let to = (0..20).map(|_| rng.sample(Alphanumeric) as char).collect::<String>();
    let amount = rng.gen_range(1..1000).to_string();
    let fee = rng.gen_range(1..10).to_string();
    let nonce = rng.gen_range(1..1000);
    let shard_id = format!("shard-{}", rng.gen_range(1..10));
    let signature = (0..64).map(|_| rng.sample(Alphanumeric) as char).collect::<String>();
    
    Transaction::new(
        from,
        to,
        amount,
        fee,
        None,
        nonce,
        shard_id,
        signature,
    )
}

// 指定された数のトランザクションを生成する関数
fn generate_transactions(count: usize) -> Vec<Transaction> {
    (0..count).map(|_| generate_random_transaction()).collect()
}

// シングルスレッドでトランザクションを処理するベンチマーク
fn bench_single_thread_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("transaction_processing");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));
    
    for &size in &[100, 1000, 10000] {
        group.bench_function(format!("single_thread_{}", size), |b| {
            b.iter_custom(|iters| {
                let mut total_duration = Duration::new(0, 0);
                
                for _ in 0..iters {
                    let transactions = generate_transactions(size);
                    
                    let start = Instant::now();
                    for tx in &transactions {
                        // シングルスレッドでの処理をシミュレート
                        black_box(tx.is_pending());
                        black_box(tx.is_confirmed());
                        black_box(tx.is_cross_shard());
                        // 実際の処理はここに追加
                    }
                    total_duration += start.elapsed();
                }
                
                total_duration
            });
        });
    }
    
    group.finish();
}

// マルチスレッドでトランザクションを処理するベンチマーク
fn bench_multi_thread_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("transaction_processing");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));
    
    for &size in &[100, 1000, 10000] {
        group.bench_function(format!("multi_thread_{}", size), |b| {
            b.iter_custom(|iters| {
                let mut total_duration = Duration::new(0, 0);
                
                for _ in 0..iters {
                    let transactions = generate_transactions(size);
                    
                    let start = Instant::now();
                    
                    // マルチスレッドでの処理
                    let chunks = transactions.chunks(transactions.len() / num_cpus::get().max(1));
                    let handles: Vec<_> = chunks
                        .map(|chunk| {
                            let chunk = chunk.to_vec();
                            std::thread::spawn(move || {
                                for tx in &chunk {
                                    black_box(tx.is_pending());
                                    black_box(tx.is_confirmed());
                                    black_box(tx.is_cross_shard());
                                    // 実際の処理はここに追加
                                }
                            })
                        })
                        .collect();
                    
                    for handle in handles {
                        handle.join().unwrap();
                    }
                    
                    total_duration += start.elapsed();
                }
                
                total_duration
            });
        });
    }
    
    group.finish();
}

// 並列プロセッサを使用したトランザクション処理のベンチマーク
fn bench_parallel_processor(c: &mut Criterion) {
    let mut group = c.benchmark_group("transaction_processing");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));
    
    for &size in &[100, 1000, 10000] {
        group.bench_function(format!("parallel_processor_{}", size), |b| {
            b.iter_custom(|iters| {
                let mut total_duration = Duration::new(0, 0);
                
                for _ in 0..iters {
                    let transactions = generate_transactions(size);
                    
                    // 並列プロセッサの設定
                    let config = ProcessorConfig {
                        thread_count: num_cpus::get(),
                        batch_size: 100,
                        max_queue_size: size * 2,
                    };
                    
                    let processor = ParallelProcessor::new(config);
                    
                    let start = Instant::now();
                    
                    // 並列プロセッサでトランザクションを処理
                    let results = processor.process_batch(&transactions);
                    black_box(results);
                    
                    total_duration += start.elapsed();
                }
                
                total_duration
            });
        });
    }
    
    group.finish();
}

// クロスシャードトランザクションのベンチマーク
fn bench_cross_shard_transactions(c: &mut Criterion) {
    let mut group = c.benchmark_group("cross_shard_transactions");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));
    
    for &size in &[10, 100, 1000] {
        group.bench_function(format!("cross_shard_{}", size), |b| {
            b.iter_custom(|iters| {
                let mut total_duration = Duration::new(0, 0);
                
                for _ in 0..iters {
                    let mut transactions = generate_transactions(size);
                    
                    // 一部のトランザクションをクロスシャードトランザクションに変更
                    for i in 0..transactions.len() / 3 {
                        let parent_id = format!("parent-{}", i);
                        let tx = &mut transactions[i];
                        tx.parent_id = Some(parent_id);
                    }
                    
                    let start = Instant::now();
                    
                    // クロスシャードトランザクションの処理をシミュレート
                    let processor = ParallelProcessor::new(ProcessorConfig {
                        thread_count: num_cpus::get(),
                        batch_size: 50,
                        max_queue_size: size * 2,
                    });
                    
                    let results = processor.process_cross_shard_batch(&transactions);
                    black_box(results);
                    
                    total_duration += start.elapsed();
                }
                
                total_duration
            });
        });
    }
    
    group.finish();
}

// スループットベンチマーク
fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    group.sample_size(5);
    group.measurement_time(Duration::from_secs(30));
    
    for &size in &[10000, 100000, 1000000] {
        group.throughput(criterion::Throughput::Elements(size as u64));
        
        group.bench_function(format!("throughput_{}", size), |b| {
            b.iter_custom(|iters| {
                let mut total_duration = Duration::new(0, 0);
                
                for _ in 0..iters {
                    let transactions = generate_transactions(size);
                    
                    // 並列プロセッサの設定
                    let config = ProcessorConfig {
                        thread_count: num_cpus::get(),
                        batch_size: 1000,
                        max_queue_size: size * 2,
                    };
                    
                    let processor = ParallelProcessor::new(config);
                    
                    let start = Instant::now();
                    
                    // 並列プロセッサでトランザクションを処理
                    let results = processor.process_batch(&transactions);
                    black_box(results);
                    
                    total_duration += start.elapsed();
                }
                
                total_duration
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_single_thread_processing,
    bench_multi_thread_processing,
    bench_parallel_processor,
    bench_cross_shard_transactions,
    bench_throughput
);
criterion_main!(benches);