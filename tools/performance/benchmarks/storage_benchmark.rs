use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shardx::storage::{Storage, StorageConfig, StorageEngine};
use std::sync::Arc;
use std::time::{Duration, Instant};
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use tempfile::tempdir;

// ランダムなキーを生成する関数
fn generate_random_key(length: usize) -> String {
    let mut rng = thread_rng();
    (0..length).map(|_| rng.sample(Alphanumeric) as char).collect()
}

// ランダムな値を生成する関数
fn generate_random_value(length: usize) -> Vec<u8> {
    let mut rng = thread_rng();
    (0..length).map(|_| rng.gen::<u8>()).collect()
}

// 書き込みベンチマーク
fn bench_storage_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_write");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));
    
    for &size in &[100, 1000, 10000] {
        for &value_size in &[100, 1000, 10000] {
            group.bench_function(format!("write_{}_{}", size, value_size), |b| {
                b.iter_custom(|iters| {
                    let mut total_duration = Duration::new(0, 0);
                    
                    for _ in 0..iters {
                        // 一時ディレクトリを作成
                        let temp_dir = tempdir().unwrap();
                        let path = temp_dir.path().to_str().unwrap().to_string();
                        
                        // ストレージを初期化
                        let config = StorageConfig {
                            path,
                            engine: StorageEngine::RocksDB,
                            cache_size_mb: 128,
                            max_open_files: 1000,
                            create_if_missing: true,
                        };
                        
                        let storage = Storage::new(config).unwrap();
                        
                        // キーと値のペアを生成
                        let pairs: Vec<_> = (0..size)
                            .map(|_| {
                                let key = generate_random_key(20);
                                let value = generate_random_value(value_size);
                                (key, value)
                            })
                            .collect();
                        
                        let start = Instant::now();
                        
                        // 書き込み
                        for (key, value) in &pairs {
                            storage.put(key.as_bytes(), value).unwrap();
                        }
                        
                        total_duration += start.elapsed();
                    }
                    
                    total_duration
                });
            });
        }
    }
    
    group.finish();
}

// 読み込みベンチマーク
fn bench_storage_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_read");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));
    
    for &size in &[100, 1000, 10000] {
        for &value_size in &[100, 1000, 10000] {
            group.bench_function(format!("read_{}_{}", size, value_size), |b| {
                // セットアップ
                let temp_dir = tempdir().unwrap();
                let path = temp_dir.path().to_str().unwrap().to_string();
                
                // ストレージを初期化
                let config = StorageConfig {
                    path,
                    engine: StorageEngine::RocksDB,
                    cache_size_mb: 128,
                    max_open_files: 1000,
                    create_if_missing: true,
                };
                
                let storage = Storage::new(config).unwrap();
                
                // キーと値のペアを生成して書き込み
                let pairs: Vec<_> = (0..size)
                    .map(|_| {
                        let key = generate_random_key(20);
                        let value = generate_random_value(value_size);
                        storage.put(key.as_bytes(), &value).unwrap();
                        key
                    })
                    .collect();
                
                b.iter_custom(|iters| {
                    let mut total_duration = Duration::new(0, 0);
                    
                    for _ in 0..iters {
                        let start = Instant::now();
                        
                        // 読み込み
                        for key in &pairs {
                            let value = storage.get(key.as_bytes()).unwrap();
                            black_box(value);
                        }
                        
                        total_duration += start.elapsed();
                    }
                    
                    total_duration
                });
            });
        }
    }
    
    group.finish();
}

// バッチ書き込みベンチマーク
fn bench_storage_batch_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_batch_write");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));
    
    for &size in &[100, 1000, 10000] {
        for &value_size in &[100, 1000, 10000] {
            group.bench_function(format!("batch_write_{}_{}", size, value_size), |b| {
                b.iter_custom(|iters| {
                    let mut total_duration = Duration::new(0, 0);
                    
                    for _ in 0..iters {
                        // 一時ディレクトリを作成
                        let temp_dir = tempdir().unwrap();
                        let path = temp_dir.path().to_str().unwrap().to_string();
                        
                        // ストレージを初期化
                        let config = StorageConfig {
                            path,
                            engine: StorageEngine::RocksDB,
                            cache_size_mb: 128,
                            max_open_files: 1000,
                            create_if_missing: true,
                        };
                        
                        let storage = Storage::new(config).unwrap();
                        
                        // キーと値のペアを生成
                        let pairs: Vec<_> = (0..size)
                            .map(|_| {
                                let key = generate_random_key(20);
                                let value = generate_random_value(value_size);
                                (key, value)
                            })
                            .collect();
                        
                        let start = Instant::now();
                        
                        // バッチ書き込み
                        let mut batch = storage.create_batch();
                        for (key, value) in &pairs {
                            batch.put(key.as_bytes(), value);
                        }
                        storage.write_batch(batch).unwrap();
                        
                        total_duration += start.elapsed();
                    }
                    
                    total_duration
                });
            });
        }
    }
    
    group.finish();
}

// 範囲スキャンベンチマーク
fn bench_storage_range_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_range_scan");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));
    
    for &size in &[100, 1000, 10000] {
        group.bench_function(format!("range_scan_{}", size), |b| {
            // セットアップ
            let temp_dir = tempdir().unwrap();
            let path = temp_dir.path().to_str().unwrap().to_string();
            
            // ストレージを初期化
            let config = StorageConfig {
                path,
                engine: StorageEngine::RocksDB,
                cache_size_mb: 128,
                max_open_files: 1000,
                create_if_missing: true,
            };
            
            let storage = Storage::new(config).unwrap();
            
            // キーと値のペアを生成して書き込み
            let mut keys = Vec::new();
            for i in 0..size {
                let key = format!("key_{:010}", i);
                let value = format!("value_{}", i).into_bytes();
                storage.put(key.as_bytes(), &value).unwrap();
                keys.push(key);
            }
            
            // 範囲スキャンのためのキー
            let start_key = "key_0000000000".to_string();
            let end_key = format!("key_{:010}", size);
            
            b.iter_custom(|iters| {
                let mut total_duration = Duration::new(0, 0);
                
                for _ in 0..iters {
                    let start = Instant::now();
                    
                    // 範囲スキャン
                    let iter = storage.range_scan(start_key.as_bytes(), end_key.as_bytes()).unwrap();
                    let results: Vec<_> = iter.collect();
                    black_box(results);
                    
                    total_duration += start.elapsed();
                }
                
                total_duration
            });
        });
    }
    
    group.finish();
}

// 削除ベンチマーク
fn bench_storage_delete(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_delete");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));
    
    for &size in &[100, 1000, 10000] {
        group.bench_function(format!("delete_{}", size), |b| {
            b.iter_custom(|iters| {
                let mut total_duration = Duration::new(0, 0);
                
                for _ in 0..iters {
                    // 一時ディレクトリを作成
                    let temp_dir = tempdir().unwrap();
                    let path = temp_dir.path().to_str().unwrap().to_string();
                    
                    // ストレージを初期化
                    let config = StorageConfig {
                        path,
                        engine: StorageEngine::RocksDB,
                        cache_size_mb: 128,
                        max_open_files: 1000,
                        create_if_missing: true,
                    };
                    
                    let storage = Storage::new(config).unwrap();
                    
                    // キーと値のペアを生成して書き込み
                    let keys: Vec<_> = (0..size)
                        .map(|_| {
                            let key = generate_random_key(20);
                            let value = generate_random_value(100);
                            storage.put(key.as_bytes(), &value).unwrap();
                            key
                        })
                        .collect();
                    
                    let start = Instant::now();
                    
                    // 削除
                    for key in &keys {
                        storage.delete(key.as_bytes()).unwrap();
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
    bench_storage_write,
    bench_storage_read,
    bench_storage_batch_write,
    bench_storage_range_scan,
    bench_storage_delete
);
criterion_main!(benches);