use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shardx::sharding::{ShardManager, ShardType, NodeSpec};
use std::sync::Arc;
use std::time::{Duration, Instant};
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

// ランダムなノード仕様を生成する関数
fn generate_random_node_spec() -> NodeSpec {
    let mut rng = thread_rng();
    
    // ランダムな文字列を生成
    let id = format!("node-{}", rng.gen_range(1..1000));
    let host = format!("192.168.{}.{}", rng.gen_range(1..255), rng.gen_range(1..255));
    let port = rng.gen_range(10000..65535);
    
    NodeSpec::new(&id, &format!("{}:{}", host, port))
}

// シャード作成のベンチマーク
fn bench_shard_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("shard_creation");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));
    
    for &size in &[10, 50, 100] {
        group.bench_function(format!("create_shards_{}", size), |b| {
            b.iter_custom(|iters| {
                let mut total_duration = Duration::new(0, 0);
                
                for _ in 0..iters {
                    let manager = ShardManager::new();
                    
                    let start = Instant::now();
                    
                    // 指定された数のシャードを作成
                    for i in 0..size {
                        let shard_type = if i % 2 == 0 { ShardType::Data } else { ShardType::Compute };
                        let shard = manager.create_shard(shard_type);
                        black_box(shard);
                    }
                    
                    total_duration += start.elapsed();
                }
                
                total_duration
            });
        });
    }
    
    group.finish();
}

// ノード追加のベンチマーク
fn bench_node_addition(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_addition");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));
    
    for &size in &[10, 50, 100] {
        group.bench_function(format!("add_nodes_{}", size), |b| {
            b.iter_custom(|iters| {
                let mut total_duration = Duration::new(0, 0);
                
                for _ in 0..iters {
                    let manager = ShardManager::new();
                    let nodes: Vec<_> = (0..size).map(|_| generate_random_node_spec()).collect();
                    
                    let start = Instant::now();
                    
                    // 指定された数のノードを追加
                    for node in &nodes {
                        manager.add_node(node.clone());
                    }
                    
                    total_duration += start.elapsed();
                }
                
                total_duration
            });
        });
    }
    
    group.finish();
}

// シャード割り当てのベンチマーク
fn bench_shard_assignment(c: &mut Criterion) {
    let mut group = c.benchmark_group("shard_assignment");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));
    
    for &shard_count in &[10, 50, 100] {
        for &node_count in &[5, 20, 50] {
            group.bench_function(format!("assign_shards_{}_{}", shard_count, node_count), |b| {
                b.iter_custom(|iters| {
                    let mut total_duration = Duration::new(0, 0);
                    
                    for _ in 0..iters {
                        let manager = ShardManager::new();
                        
                        // ノードを追加
                        let nodes: Vec<_> = (0..node_count).map(|_| generate_random_node_spec()).collect();
                        for node in &nodes {
                            manager.add_node(node.clone());
                        }
                        
                        // シャードを作成
                        let shards: Vec<_> = (0..shard_count)
                            .map(|i| {
                                let shard_type = if i % 2 == 0 { ShardType::Data } else { ShardType::Compute };
                                manager.create_shard(shard_type)
                            })
                            .collect();
                        
                        let start = Instant::now();
                        
                        // シャードをノードに割り当て
                        manager.assign_all_shards();
                        
                        total_duration += start.elapsed();
                    }
                    
                    total_duration
                });
            });
        }
    }
    
    group.finish();
}

// シャード再バランシングのベンチマーク
fn bench_shard_rebalancing(c: &mut Criterion) {
    let mut group = c.benchmark_group("shard_rebalancing");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));
    
    for &shard_count in &[20, 100] {
        for &node_count in &[5, 20] {
            group.bench_function(format!("rebalance_shards_{}_{}", shard_count, node_count), |b| {
                b.iter_custom(|iters| {
                    let mut total_duration = Duration::new(0, 0);
                    
                    for _ in 0..iters {
                        let manager = ShardManager::new();
                        
                        // ノードを追加
                        let nodes: Vec<_> = (0..node_count).map(|_| generate_random_node_spec()).collect();
                        for node in &nodes {
                            manager.add_node(node.clone());
                        }
                        
                        // シャードを作成
                        let shards: Vec<_> = (0..shard_count)
                            .map(|i| {
                                let shard_type = if i % 2 == 0 { ShardType::Data } else { ShardType::Compute };
                                manager.create_shard(shard_type)
                            })
                            .collect();
                        
                        // シャードをノードに割り当て
                        manager.assign_all_shards();
                        
                        // 新しいノードを追加（再バランシングのトリガー）
                        let new_nodes: Vec<_> = (0..node_count/2).map(|_| generate_random_node_spec()).collect();
                        for node in &new_nodes {
                            manager.add_node(node.clone());
                        }
                        
                        let start = Instant::now();
                        
                        // シャードを再バランス
                        manager.rebalance_shards();
                        
                        total_duration += start.elapsed();
                    }
                    
                    total_duration
                });
            });
        }
    }
    
    group.finish();
}

// シャード検索のベンチマーク
fn bench_shard_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("shard_lookup");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));
    
    for &shard_count in &[100, 1000, 10000] {
        group.bench_function(format!("lookup_shards_{}", shard_count), |b| {
            // セットアップ
            let manager = ShardManager::new();
            
            // シャードを作成
            let shards: Vec<_> = (0..shard_count)
                .map(|i| {
                    let shard_type = if i % 2 == 0 { ShardType::Data } else { ShardType::Compute };
                    manager.create_shard(shard_type)
                })
                .collect();
            
            // ノードを追加
            let node_count = shard_count / 10;
            let nodes: Vec<_> = (0..node_count).map(|_| generate_random_node_spec()).collect();
            for node in &nodes {
                manager.add_node(node.clone());
            }
            
            // シャードをノードに割り当て
            manager.assign_all_shards();
            
            // ランダムなキーを生成
            let mut rng = thread_rng();
            let keys: Vec<String> = (0..1000)
                .map(|_| (0..20).map(|_| rng.sample(Alphanumeric) as char).collect())
                .collect();
            
            b.iter_custom(|iters| {
                let mut total_duration = Duration::new(0, 0);
                
                for _ in 0..iters {
                    let start = Instant::now();
                    
                    // ランダムなキーに対応するシャードを検索
                    for key in &keys {
                        let shard = manager.get_shard_for_key(key);
                        black_box(shard);
                    }
                    
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
    bench_shard_creation,
    bench_node_addition,
    bench_shard_assignment,
    bench_shard_rebalancing,
    bench_shard_lookup
);
criterion_main!(benches);