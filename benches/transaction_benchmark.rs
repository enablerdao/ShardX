use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shardx::crypto::HashManager;
use shardx::parallel::WorkStealingScheduler;
use shardx::transaction::{Transaction, TransactionStatus};

fn create_test_transaction(id: &str) -> Transaction {
    Transaction {
        id: id.to_string(),
        parent_ids: vec!["parent1".to_string()],
        timestamp: 12345,
        payload: vec![1, 2, 3, 4, 5],
        signature: vec![6, 7, 8, 9, 10],
        status: TransactionStatus::Pending,
    }
}

fn hash_transaction_benchmark(c: &mut Criterion) {
    let hash_manager = HashManager::new(4);
    let tx = create_test_transaction("tx1");
    
    c.bench_function("hash_transaction", |b| {
        b.iter(|| {
            black_box(hash_manager.hash_transaction(black_box(&tx)));
        })
    });
}

fn batch_processing_benchmark(c: &mut Criterion) {
    let scheduler = WorkStealingScheduler::new();
    
    // 1000トランザクションのバッチを作成
    let txs: Vec<Transaction> = (0..1000)
        .map(|i| create_test_transaction(&format!("tx{}", i)))
        .collect();
    
    c.bench_function("process_1000_transactions", |b| {
        b.iter(|| {
            let processor = |tx: Transaction| -> Result<(), shardx::error::Error> {
                // シンプルな処理
                black_box(tx);
                Ok(())
            };
            
            black_box(scheduler.process_batch(black_box(txs.clone()), processor));
        })
    });
}

fn transaction_serialization_benchmark(c: &mut Criterion) {
    let tx = create_test_transaction("tx1");
    
    c.bench_function("transaction_serialization", |b| {
        b.iter(|| {
            black_box(bincode::serialize(black_box(&tx)).unwrap());
        })
    });
    
    let serialized = bincode::serialize(&tx).unwrap();
    
    c.bench_function("transaction_deserialization", |b| {
        b.iter(|| {
            black_box(bincode::deserialize::<Transaction>(black_box(&serialized)).unwrap());
        })
    });
}

criterion_group!(
    benches,
    hash_transaction_benchmark,
    batch_processing_benchmark,
    transaction_serialization_benchmark
);
criterion_main!(benches);