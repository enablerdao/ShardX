//! 並列処理器のテスト

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::Duration;

use crate::error::Error;
use crate::network::NetworkMessage;
use crate::shard::{ShardId, ShardInfo, ShardManager, ShardStatus};
use crate::transaction::parallel_processor::{ParallelProcessor, ProcessorConfig};
use crate::transaction::{
    CrossShardManager, CrossShardTransaction, CrossShardTransactionState, Transaction,
    TransactionStatus,
};

/// 依存関係グループ化のテスト
#[tokio::test]
async fn test_group_by_dependencies() {
    // テスト用の依存関係グラフを作成
    let mut dependencies = HashMap::new();

    // 依存関係のないトランザクション
    dependencies.insert("tx1".to_string(), HashSet::new());
    dependencies.insert("tx2".to_string(), HashSet::new());

    // tx1に依存するトランザクション
    let mut tx3_deps = HashSet::new();
    tx3_deps.insert("tx1".to_string());
    dependencies.insert("tx3".to_string(), tx3_deps);

    // tx2に依存するトランザクション
    let mut tx4_deps = HashSet::new();
    tx4_deps.insert("tx2".to_string());
    dependencies.insert("tx4".to_string(), tx4_deps);

    // tx3とtx4に依存するトランザクション
    let mut tx5_deps = HashSet::new();
    tx5_deps.insert("tx3".to_string());
    tx5_deps.insert("tx4".to_string());
    dependencies.insert("tx5".to_string(), tx5_deps);

    // ダミーのコンポーネントを作成
    let (network_tx, _) = mpsc::channel(100);
    let cross_shard_manager = Arc::new(CrossShardManager::new(network_tx.clone()));
    let shard_manager = Arc::new(ShardManager::new(network_tx.clone()));

    // 並列処理器を作成
    let processor = ParallelProcessor::new(cross_shard_manager, shard_manager, network_tx, None);

    // 依存関係に基づいてグループ化
    let groups = processor.group_by_dependencies(dependencies);

    // 結果を検証
    assert_eq!(groups.len(), 3);

    // 最初のグループは依存関係のないトランザクション
    assert_eq!(groups[0].len(), 2);
    assert!(groups[0].contains(&"tx1".to_string()));
    assert!(groups[0].contains(&"tx2".to_string()));

    // 2番目のグループはtx1とtx2に依存するトランザクション
    assert_eq!(groups[1].len(), 2);
    assert!(groups[1].contains(&"tx3".to_string()));
    assert!(groups[1].contains(&"tx4".to_string()));

    // 3番目のグループはtx3とtx4に依存するトランザクション
    assert_eq!(groups[2].len(), 1);
    assert!(groups[2].contains(&"tx5".to_string()));
}

/// 循環依存関係の検出と解決のテスト
#[tokio::test]
async fn test_cycle_detection() {
    // テスト用の依存関係グラフを作成（循環依存関係を含む）
    let mut dependencies = HashMap::new();

    // 依存関係のないトランザクション
    dependencies.insert("tx1".to_string(), HashSet::new());

    // 循環依存関係を持つトランザクション
    let mut tx2_deps = HashSet::new();
    tx2_deps.insert("tx3".to_string());
    dependencies.insert("tx2".to_string(), tx2_deps);

    let mut tx3_deps = HashSet::new();
    tx3_deps.insert("tx4".to_string());
    dependencies.insert("tx3".to_string(), tx3_deps);

    let mut tx4_deps = HashSet::new();
    tx4_deps.insert("tx2".to_string());
    dependencies.insert("tx4".to_string(), tx4_deps);

    // ダミーのコンポーネントを作成
    let (network_tx, _) = mpsc::channel(100);
    let cross_shard_manager = Arc::new(CrossShardManager::new(network_tx.clone()));
    let shard_manager = Arc::new(ShardManager::new(network_tx.clone()));

    // 並列処理器を作成
    let processor = ParallelProcessor::new(cross_shard_manager, shard_manager, network_tx, None);

    // 依存関係に基づいてグループ化
    let groups = processor.group_by_dependencies(dependencies);

    // 結果を検証
    assert!(groups.len() >= 2); // 少なくとも2グループ（依存関係なし + 循環依存関係解決）

    // 最初のグループは依存関係のないトランザクション
    assert_eq!(groups[0].len(), 1);
    assert!(groups[0].contains(&"tx1".to_string()));

    // 2番目以降のグループは循環依存関係の解決を含む
    let all_cycle_txs: HashSet<_> = groups[1..]
        .iter()
        .flat_map(|group| group.iter().cloned())
        .collect();

    // すべての循環依存関係トランザクションが含まれていることを確認
    assert!(all_cycle_txs.contains("tx2"));
    assert!(all_cycle_txs.contains("tx3"));
    assert!(all_cycle_txs.contains("tx4"));
}

/// 大量のトランザクションの依存関係解析のテスト
#[tokio::test]
async fn test_large_batch_dependencies() {
    // 大量のトランザクションを生成
    let mut transactions = Vec::new();
    for i in 0..1000 {
        let tx = Transaction {
            id: format!("tx{}", i),
            from: format!("addr{}", i % 100), // 100アドレスに分散
            to: format!("addr{}", (i + 1) % 100),
            amount: i as f64,
            nonce: i as u64 % 10, // 各アドレスごとに0-9のnonce
            parent_id: if i > 0 && i % 10 == 0 {
                Some(format!("tx{}", i - 1))
            } else {
                None
            },
            status: TransactionStatus::Pending,
            timestamp: chrono::Utc::now().timestamp() as u64,
            data: None,
            signature: None,
        };
        transactions.push(tx);
    }

    // ダミーのコンポーネントを作成
    let (network_tx, _) = mpsc::channel(100);
    let cross_shard_manager = Arc::new(CrossShardManager::new(network_tx.clone()));
    let shard_manager = Arc::new(ShardManager::new(network_tx.clone()));

    // 並列処理器を作成
    let processor = ParallelProcessor::new(cross_shard_manager, shard_manager, network_tx, None);

    // 依存関係を解析
    let start_time = std::time::Instant::now();
    let groups = processor
        .analyze_batch_dependencies(&transactions)
        .await
        .unwrap();
    let elapsed = start_time.elapsed();

    // 結果を検証
    println!(
        "Analyzed {} transactions in {:?}, found {} groups",
        transactions.len(),
        elapsed,
        groups.len()
    );

    // すべてのトランザクションが含まれていることを確認
    let total_txs: usize = groups.iter().map(|g| g.len()).sum();
    assert_eq!(total_txs, transactions.len());

    // 処理時間が合理的であることを確認（1000トランザクションで1秒以内）
    assert!(elapsed < Duration::from_secs(1));
}

/// 設定更新のテスト
#[tokio::test]
async fn test_config_update() {
    // ダミーのコンポーネントを作成
    let (network_tx, _) = mpsc::channel(100);
    let cross_shard_manager = Arc::new(CrossShardManager::new(network_tx.clone()));
    let shard_manager = Arc::new(ShardManager::new(network_tx.clone()));

    // 初期設定
    let initial_config = ProcessorConfig {
        max_parallelism: 32,
        ..ProcessorConfig::default()
    };

    // 並列処理器を作成
    let mut processor = ParallelProcessor::new(
        cross_shard_manager,
        shard_manager,
        network_tx,
        Some(initial_config.clone()),
    );

    // 初期設定を確認
    let config = processor.get_config();
    assert_eq!(config.max_parallelism, 32);

    // 設定を更新
    let new_config = ProcessorConfig {
        max_parallelism: 64,
        batch_size: 200,
        ..ProcessorConfig::default()
    };
    processor.update_config(new_config.clone());

    // 更新後の設定を確認
    let updated_config = processor.get_config();
    assert_eq!(updated_config.max_parallelism, 64);
    assert_eq!(updated_config.batch_size, 200);
}

/// 統計更新のテスト
#[tokio::test]
async fn test_stats_update() {
    // ダミーのコンポーネントを作成
    let (network_tx, _) = mpsc::channel(100);
    let cross_shard_manager = Arc::new(CrossShardManager::new(network_tx.clone()));
    let shard_manager = Arc::new(ShardManager::new(network_tx.clone()));

    // 並列処理器を作成
    let processor = ParallelProcessor::new(cross_shard_manager, shard_manager, network_tx, None);

    // 初期統計を確認
    let initial_stats = processor.get_stats();
    assert_eq!(initial_stats.processed_transactions, 0);
    assert_eq!(initial_stats.successful_transactions, 0);
    assert_eq!(initial_stats.failed_transactions, 0);

    // 統計を更新
    processor.update_stats("test_tx".to_string(), true, 100);

    // 更新後の統計を確認
    let updated_stats = processor.get_stats();
    assert_eq!(updated_stats.processed_transactions, 1);
    assert_eq!(updated_stats.successful_transactions, 1);
    assert_eq!(updated_stats.failed_transactions, 0);
    assert!(updated_stats.avg_processing_time_ms > 0.0);
    assert_eq!(updated_stats.max_processing_time_ms, 100);
    assert_eq!(updated_stats.min_processing_time_ms, 100);

    // 失敗したトランザクションの統計を更新
    processor.update_stats("test_tx2".to_string(), false, 50);

    // 再度更新後の統計を確認
    let final_stats = processor.get_stats();
    assert_eq!(final_stats.processed_transactions, 2);
    assert_eq!(final_stats.successful_transactions, 1);
    assert_eq!(final_stats.failed_transactions, 1);
    assert!(final_stats.min_processing_time_ms == 50);
}
