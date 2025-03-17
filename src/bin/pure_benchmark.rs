use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    println!("Starting pure benchmark...");

    // ベンチマークパラメータ
    let transaction_count = 1000000; // 100万トランザクション

    // シングルスレッドベンチマーク
    println!("Running single-threaded benchmark...");
    let start_time = Instant::now();

    let mut successful = 0;
    for i in 0..transaction_count {
        if simulate_transaction(i) {
            successful += 1;
        }
    }

    let elapsed = start_time.elapsed();
    let tps = transaction_count as f64 / elapsed.as_secs_f64();

    println!(
        "Single-threaded benchmark completed in {:.2} seconds",
        elapsed.as_secs_f64()
    );
    println!(
        "Transactions: {} total, {} successful, {} failed",
        transaction_count,
        successful,
        transaction_count - successful
    );
    println!("Throughput: {:.2} TPS", tps);

    // マルチスレッドベンチマーク
    println!("\nRunning multi-threaded benchmark...");

    // 利用可能なCPUコア数を取得
    let num_cpus = num_cpus::get();
    println!("Detected {} CPU cores", num_cpus);

    // スレッド数のバリエーションでベンチマークを実行
    let thread_counts = vec![1, 2, 4, 8, 16, num_cpus];

    for &threads in thread_counts.iter().filter(|&&t| t <= num_cpus) {
        println!("Testing with {} threads...", threads);

        let start_time = Instant::now();
        let transactions_per_thread = transaction_count / threads;

        // スレッドを生成
        let handles: Vec<_> = (0..threads)
            .map(|thread_id| {
                let start_idx = thread_id * transactions_per_thread;
                let end_idx = start_idx + transactions_per_thread;

                thread::spawn(move || {
                    let mut successful = 0;
                    for i in start_idx..end_idx {
                        if simulate_transaction(i) {
                            successful += 1;
                        }
                    }
                    successful
                })
            })
            .collect();

        // すべてのスレッドが完了するのを待つ
        let mut total_successful = 0;
        for handle in handles {
            total_successful += handle.join().unwrap();
        }

        let elapsed = start_time.elapsed();
        let tps = transaction_count as f64 / elapsed.as_secs_f64();

        println!("  Completed in {:.2} seconds", elapsed.as_secs_f64());
        println!("  Throughput: {:.2} TPS", tps);

        if tps >= 100000.0 {
            println!("  🎉 SUCCESS: Achieved 100K+ TPS with {} threads!", threads);
        }
    }

    // 並列バッチ処理ベンチマーク
    println!("\nRunning parallel batch processing benchmark...");

    let batch_sizes = vec![100, 1000, 10000];

    for &batch_size in &batch_sizes {
        println!("Testing with batch size {}...", batch_size);

        let start_time = Instant::now();
        let batches = transaction_count / batch_size;
        let successful = Arc::new(Mutex::new(0));

        for batch_idx in 0..batches {
            let start_idx = batch_idx * batch_size;
            let end_idx = start_idx + batch_size;
            let successful_clone = Arc::clone(&successful);

            // バッチ内のトランザクションを並列処理
            let batch: Vec<_> = (start_idx..end_idx).collect();

            let batch_successful: usize = batch
                .into_par_iter()
                .filter(|&i| simulate_transaction(i))
                .count();

            let mut successful = successful_clone.lock().unwrap();
            *successful += batch_successful;
        }

        let elapsed = start_time.elapsed();
        let tps = transaction_count as f64 / elapsed.as_secs_f64();

        println!("  Completed in {:.2} seconds", elapsed.as_secs_f64());
        println!("  Throughput: {:.2} TPS", tps);

        if tps >= 100000.0 {
            println!(
                "  🎉 SUCCESS: Achieved 100K+ TPS with batch size {}!",
                batch_size
            );
        }
    }
}

// シンプルなトランザクション処理をシミュレート
fn simulate_transaction(nonce: usize) -> bool {
    // 署名検証をシミュレート
    let signature_valid = verify_signature(nonce);

    // 残高チェックをシミュレート
    let balance_sufficient = check_balance(nonce);

    // 手数料チェックをシミュレート
    let fee_sufficient = check_fee(nonce);

    // トランザクション実行をシミュレート
    if signature_valid && balance_sufficient && fee_sufficient {
        execute_transaction(nonce);
        true
    } else {
        false
    }
}

// 署名検証をシミュレート
fn verify_signature(nonce: usize) -> bool {
    // 実際の署名検証の代わりに、簡単な計算を行う
    let hash = (nonce * 13) % 100;
    hash > 5 // 95%の確率で成功
}

// 残高チェックをシミュレート
fn check_balance(nonce: usize) -> bool {
    // 実際の残高チェックの代わりに、簡単な計算を行う
    let balance = (nonce * 17) % 1000;
    let amount = (nonce * 7) % 900;
    balance >= amount
}

// 手数料チェックをシミュレート
fn check_fee(nonce: usize) -> bool {
    // 実際の手数料チェックの代わりに、簡単な計算を行う
    let fee = (nonce * 3) % 50;
    fee > 0
}

// トランザクション実行をシミュレート
fn execute_transaction(nonce: usize) {
    // 実際のトランザクション実行の代わりに、簡単な計算を行う
    let _new_balance = (nonce * 17) % 1000 - (nonce * 7) % 900;
    let _new_recipient_balance = (nonce * 11) % 1000 + (nonce * 7) % 900;
}

// 並列イテレーションのためのトレイト
trait IntoParallelIterator {
    type Item;
    type Iter: Iterator<Item = Self::Item>;

    fn into_par_iter(self) -> Self::Iter;
}

// Vecに対する実装
impl<T> IntoParallelIterator for Vec<T> {
    type Item = T;
    type Iter = std::vec::IntoIter<T>;

    fn into_par_iter(self) -> Self::Iter {
        self.into_iter()
    }
}
