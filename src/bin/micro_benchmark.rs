use std::time::{Duration, Instant};
use log::{info, error};

fn main() {
    // ロガーを初期化
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    info!("Starting micro benchmark...");

    // ベンチマークパラメータ
    let transaction_count = 1000000; // 100万トランザクション
    
    // ベンチマークを実行
    info!("Running benchmark with {} transactions...", transaction_count);
    let start_time = Instant::now();
    
    let mut successful = 0;
    
    // シンプルな計算を実行してCPUの処理能力を測定
    for i in 0..transaction_count {
        // シンプルなトランザクション処理をシミュレート
        if simulate_transaction(i) {
            successful += 1;
        }
    }
    
    let elapsed = start_time.elapsed();
    
    // 結果を表示
    info!("Benchmark completed in {:.2} seconds", elapsed.as_secs_f64());
    info!("Transactions: {} total, {} successful, {} failed",
        transaction_count, successful, transaction_count - successful);
    
    let tps = transaction_count as f64 / elapsed.as_secs_f64();
    info!("Throughput: {:.2} TPS", tps);
    
    // 目標の100K TPSを達成したかチェック
    if tps >= 100000.0 {
        info!("🎉 SUCCESS: Achieved 100K+ TPS! ({:.2} TPS)", tps);
    } else {
        info!("❌ FAILED: Did not achieve 100K TPS. Reached {:.2} TPS", tps);
    }
    
    // マルチスレッドベンチマークを実行
    run_multithreaded_benchmark(transaction_count);
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

// マルチスレッドベンチマークを実行
fn run_multithreaded_benchmark(transaction_count: usize) {
    info!("\nRunning multi-threaded benchmark...");
    
    // 利用可能なCPUコア数を取得
    let num_cpus = num_cpus::get();
    info!("Detected {} CPU cores", num_cpus);
    
    // スレッド数のバリエーションでベンチマークを実行
    let thread_counts = vec![1, 2, 4, 8, 16, num_cpus];
    
    for &threads in thread_counts.iter().filter(|&&t| t <= num_cpus) {
        info!("Testing with {} threads...", threads);
        
        let start_time = Instant::now();
        let transactions_per_thread = transaction_count / threads;
        
        // スレッドを生成
        let handles: Vec<_> = (0..threads)
            .map(|thread_id| {
                let start_idx = thread_id * transactions_per_thread;
                let end_idx = start_idx + transactions_per_thread;
                
                std::thread::spawn(move || {
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
        
        info!("  Completed in {:.2} seconds", elapsed.as_secs_f64());
        info!("  Throughput: {:.2} TPS", tps);
        
        if tps >= 100000.0 {
            info!("  🎉 SUCCESS: Achieved 100K+ TPS with {} threads!", threads);
        }
    }
}