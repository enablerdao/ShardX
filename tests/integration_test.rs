#[cfg(test)]
mod integration_tests {
    use shardx::transaction::{Transaction, TransactionStatus};
    use shardx::crypto::{HashManager, SignatureManager};
    use shardx::sharding::{ShardManager, ShardId};
    use shardx::parallel::WorkStealingScheduler;
    use shardx::storage::MemoryStorage;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};
    use rayon::prelude::*;
    use std::thread;

    // テスト用のトランザクションを生成
    fn create_test_transaction(id: &str, payload_size: usize) -> Transaction {
        let mut payload = Vec::with_capacity(payload_size);
        for i in 0..payload_size {
            payload.push((i % 256) as u8);
        }

        Transaction {
            id: id.to_string(),
            parent_ids: vec!["parent1".to_string()],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            payload,
            signature: vec![],
            status: TransactionStatus::Pending,
        }
    }

    // 署名付きトランザクションを生成
    fn create_signed_transaction(id: &str, payload_size: usize, signature_manager: &mut SignatureManager) -> Transaction {
        let mut tx = create_test_transaction(id, payload_size);
        signature_manager.sign_transaction(&mut tx).unwrap();
        tx
    }

    // 大量のトランザクションを処理するテスト
    #[test]
    fn test_process_many_transactions() {
        // 初期化
        let hash_manager = HashManager::new(num_cpus::get());
        let mut signature_manager = SignatureManager::new();
        signature_manager.generate_keypair().unwrap();
        let shard_manager = ShardManager::new(10);
        let scheduler = WorkStealingScheduler::new();
        let storage = MemoryStorage::new();

        // 処理するトランザクション数
        let tx_count = 10000;
        
        // トランザクションを生成
        let transactions: Vec<Transaction> = (0..tx_count)
            .map(|i| create_signed_transaction(&format!("tx_{}", i), 100, &mut signature_manager))
            .collect();

        // 処理開始時間を記録
        let start_time = Instant::now();

        // トランザクションを処理
        let results = scheduler.process_batch(transactions.clone(), |tx| {
            // 1. ハッシュ計算
            let tx_hash = hash_manager.hash_transaction(&tx);
            
            // 2. 署名検証
            signature_manager.verify_transaction(&tx)?;
            
            // 3. ストレージに保存
            storage.put("transactions", &tx.id, &bincode::serialize(&tx).unwrap())?;
            
            Ok(())
        });

        // 処理時間を計算
        let elapsed = start_time.elapsed();
        let tps = tx_count as f64 / elapsed.as_secs_f64();

        // 結果を確認
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        let failure_count = results.iter().filter(|r| r.is_err()).count();

        println!("処理時間: {:?}", elapsed);
        println!("TPS: {:.2}", tps);
        println!("成功: {}, 失敗: {}", success_count, failure_count);

        // すべてのトランザクションが正常に処理されたことを確認
        assert_eq!(success_count, tx_count);
        assert_eq!(failure_count, 0);
    }

    // シャーディングのパフォーマンステスト
    #[test]
    fn test_sharding_performance() {
        // 初期化
        let hash_manager = HashManager::new(num_cpus::get());
        let mut signature_manager = SignatureManager::new();
        signature_manager.generate_keypair().unwrap();
        let shard_manager = Arc::new(Mutex::new(ShardManager::new(10)));
        let scheduler = WorkStealingScheduler::new();

        // シャードごとのストレージを作成
        let storages: Vec<MemoryStorage> = (0..10).map(|_| MemoryStorage::new()).collect();

        // 処理するトランザクション数
        let tx_count = 10000;
        
        // トランザクションを生成
        let transactions: Vec<Transaction> = (0..tx_count)
            .map(|i| create_signed_transaction(&format!("tx_{}", i), 100, &mut signature_manager))
            .collect();

        // 処理開始時間を記録
        let start_time = Instant::now();

        // シャードごとにトランザクションを分配
        let shard_assignments: Vec<(ShardId, Transaction)> = transactions
            .into_iter()
            .map(|tx| {
                let shard_id = tx.id.as_bytes()[0] as ShardId % 10;
                (shard_id, tx)
            })
            .collect();

        // シャードごとにトランザクションをグループ化
        let mut shard_txs: Vec<Vec<Transaction>> = vec![Vec::new(); 10];
        for (shard_id, tx) in shard_assignments {
            shard_txs[shard_id as usize].push(tx);
        }

        // 各シャードを並列処理
        let results: Vec<usize> = shard_txs
            .par_iter()
            .enumerate()
            .map(|(shard_id, txs)| {
                // シャードごとの処理
                let storage = &storages[shard_id];
                let processed = scheduler.process_batch(txs.clone(), |tx| {
                    // 1. ハッシュ計算
                    let tx_hash = hash_manager.hash_transaction(&tx);
                    
                    // 2. 署名検証
                    signature_manager.verify_transaction(&tx)?;
                    
                    // 3. ストレージに保存
                    storage.put("transactions", &tx.id, &bincode::serialize(&tx).unwrap())?;
                    
                    Ok(())
                });

                // 成功したトランザクション数を返す
                processed.iter().filter(|r| r.is_ok()).count()
            })
            .collect();

        // 処理時間を計算
        let elapsed = start_time.elapsed();
        let tps = tx_count as f64 / elapsed.as_secs_f64();

        // 結果を確認
        let total_success = results.iter().sum::<usize>();

        println!("シャーディング処理時間: {:?}", elapsed);
        println!("シャーディングTPS: {:.2}", tps);
        println!("シャードごとの成功数: {:?}", results);
        println!("合計成功数: {}", total_success);

        // すべてのトランザクションが正常に処理されたことを確認
        assert_eq!(total_success, tx_count);
    }

    // 負荷テスト
    #[test]
    fn test_load_performance() {
        // 初期化
        let hash_manager = HashManager::new(num_cpus::get());
        let mut signature_manager = SignatureManager::new();
        signature_manager.generate_keypair().unwrap();
        let shard_manager = ShardManager::new(10);
        let scheduler = WorkStealingScheduler::new();
        let storage = MemoryStorage::new();

        // 異なる負荷でのパフォーマンスを測定
        let loads = vec![100, 1000, 10000];
        
        for &load in &loads {
            // トランザクションを生成
            let transactions: Vec<Transaction> = (0..load)
                .map(|i| create_signed_transaction(&format!("tx_{}", i), 100, &mut signature_manager))
                .collect();

            // 処理開始時間を記録
            let start_time = Instant::now();

            // トランザクションを処理
            let results = scheduler.process_batch(transactions.clone(), |tx| {
                // 1. ハッシュ計算
                let tx_hash = hash_manager.hash_transaction(&tx);
                
                // 2. 署名検証
                signature_manager.verify_transaction(&tx)?;
                
                // 3. ストレージに保存
                storage.put("transactions", &tx.id, &bincode::serialize(&tx).unwrap())?;
                
                Ok(())
            });

            // 処理時間を計算
            let elapsed = start_time.elapsed();
            let tps = load as f64 / elapsed.as_secs_f64();

            // 結果を確認
            let success_count = results.iter().filter(|r| r.is_ok()).count();

            println!("負荷 {}: 処理時間: {:?}, TPS: {:.2}", load, elapsed, tps);
            
            // すべてのトランザクションが正常に処理されたことを確認
            assert_eq!(success_count, load);
        }
    }

    // 並列処理の効率性テスト
    #[test]
    fn test_parallel_efficiency() {
        // 初期化
        let hash_manager = HashManager::new(num_cpus::get());
        let mut signature_manager = SignatureManager::new();
        signature_manager.generate_keypair().unwrap();
        
        // 処理するトランザクション数
        let tx_count = 5000;
        
        // トランザクションを生成
        let transactions: Vec<Transaction> = (0..tx_count)
            .map(|i| create_signed_transaction(&format!("tx_{}", i), 100, &mut signature_manager))
            .collect();

        // 異なるスレッド数でのパフォーマンスを測定
        let thread_counts = vec![1, 2, 4, 8, num_cpus::get()];
        
        for &threads in &thread_counts {
            // スケジューラを作成
            let scheduler = WorkStealingScheduler::new();
            scheduler.set_cpu_limit(100); // 最大CPU使用率を設定
            
            // 処理開始時間を記録
            let start_time = Instant::now();

            // トランザクションを処理
            let results = scheduler.process_batch(transactions.clone(), |tx| {
                // 1. ハッシュ計算
                let tx_hash = hash_manager.hash_transaction(&tx);
                
                // 2. 署名検証
                signature_manager.verify_transaction(&tx)?;
                
                // 3. シミュレートされた処理時間
                thread::sleep(Duration::from_micros(10));
                
                Ok(())
            });

            // 処理時間を計算
            let elapsed = start_time.elapsed();
            let tps = tx_count as f64 / elapsed.as_secs_f64();

            println!("スレッド数 {}: 処理時間: {:?}, TPS: {:.2}", threads, elapsed, tps);
        }
    }

    // 大きなデータのハッシュ計算パフォーマンステスト
    #[test]
    fn test_hash_large_data_performance() {
        // 初期化
        let hash_manager = HashManager::new(num_cpus::get());
        
        // 異なるサイズのデータでのパフォーマンスを測定
        let sizes = vec![1024, 1024 * 1024, 10 * 1024 * 1024];
        
        for &size in &sizes {
            // テストデータを生成
            let data = vec![0u8; size];
            
            // 処理開始時間を記録
            let start_time = Instant::now();

            // 通常のハッシュ計算
            let hash1 = hash_manager.hash_data(&data);
            
            let elapsed1 = start_time.elapsed();
            
            // 処理開始時間を記録
            let start_time = Instant::now();
            
            // 並列ハッシュ計算
            let hash2 = hash_manager.hash_large_data(&data);
            
            let elapsed2 = start_time.elapsed();
            
            println!("データサイズ {}: 通常ハッシュ: {:?}, 並列ハッシュ: {:?}", 
                     size, elapsed1, elapsed2);
            
            // ハッシュ値が一致することを確認
            assert_eq!(hash1, hash2);
        }
    }
}