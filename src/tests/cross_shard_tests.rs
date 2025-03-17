use crate::cross_shard::{
    CrossShardCoordinator, CrossShardMessage, CrossShardMessageType, CrossShardTxStatus,
};
use crate::sharding::ShardManager;
use crate::transaction::{Transaction, TransactionStatus};
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

// テスト用のトランザクションを作成
fn create_test_transaction() -> Transaction {
    Transaction {
        id: Uuid::new_v4().to_string(),
        parent_ids: vec![],
        timestamp: 12345,
        payload: vec![1, 2, 3],
        signature: vec![4, 5, 6],
        status: TransactionStatus::Pending,
        created_at: chrono::Utc::now(),
    }
}

// 複数のシャードを持つテスト環境をセットアップ
async fn setup_test_environment(shard_count: usize) -> Vec<Arc<CrossShardCoordinator>> {
    let shard_manager = Arc::new(ShardManager::new(shard_count));
    let mut coordinators = Vec::with_capacity(shard_count);
    let mut channels = Vec::with_capacity(shard_count);

    // 各シャード用のチャネルを作成
    for _ in 0..shard_count {
        let (tx, rx) = mpsc::channel(100);
        channels.push((tx, rx));
    }

    // 各シャード用のコーディネーターを作成
    for i in 0..shard_count {
        let (tx, rx) = channels[i].clone();
        let coordinator = Arc::new(CrossShardCoordinator::new(
            i as u32,
            shard_manager.clone(),
            tx,
            rx,
        ));
        coordinators.push(coordinator);
    }

    // メッセージ処理ループを開始
    for coordinator in &coordinators {
        coordinator.start_message_processing().await.unwrap();
    }

    coordinators
}

#[tokio::test]
async fn test_cross_shard_transaction_success() {
    // 3つのシャードを持つテスト環境をセットアップ
    let coordinators = setup_test_environment(3).await;

    // テスト用のトランザクションを作成
    let transaction = create_test_transaction();

    // シャード0からクロスシャードトランザクションを開始
    let tx_id = coordinators[0]
        .start_transaction(transaction)
        .await
        .unwrap();

    // トランザクションが完了するまで待機
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // トランザクションの状態を確認
    let status = coordinators[0].get_transaction_status(&tx_id).unwrap();
    assert_eq!(status, CrossShardTxStatus::Committed);

    // 他のシャードでもトランザクションが正しく処理されたことを確認
    for i in 1..coordinators.len() {
        let details = coordinators[i].get_transaction_details(&tx_id);
        assert!(details.is_ok());

        let details = details.unwrap();
        assert_eq!(details.status, CrossShardTxStatus::Committed);
        assert!(details.completed_at.is_some());
    }
}

#[tokio::test]
async fn test_cross_shard_transaction_abort() {
    // 3つのシャードを持つテスト環境をセットアップ
    let coordinators = setup_test_environment(3).await;

    // テスト用のトランザクションを作成
    let transaction = create_test_transaction();

    // シャード0からクロスシャードトランザクションを開始
    let tx_id = coordinators[0]
        .start_transaction(transaction)
        .await
        .unwrap();

    // 少し待機してから、トランザクションをアボート
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    coordinators[0].start_abort_phase(&tx_id).await.unwrap();

    // トランザクションが完了するまで待機
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // トランザクションの状態を確認
    let status = coordinators[0].get_transaction_status(&tx_id).unwrap();
    assert_eq!(status, CrossShardTxStatus::Aborted);

    // 他のシャードでもトランザクションが正しくアボートされたことを確認
    for i in 1..coordinators.len() {
        let details = coordinators[i].get_transaction_details(&tx_id);

        // トランザクションがアボートされた場合、他のシャードでは詳細が見つからない可能性がある
        if details.is_ok() {
            let details = details.unwrap();
            assert!(
                details.status == CrossShardTxStatus::Aborted
                    || details.status == CrossShardTxStatus::Aborting
            );
        }
    }
}

#[tokio::test]
async fn test_cross_shard_message_processing() {
    // 2つのシャードを持つテスト環境をセットアップ
    let coordinators = setup_test_environment(2).await;

    // テスト用のトランザクションを作成
    let transaction = create_test_transaction();

    // シャード0からシャード1へのメッセージを作成
    let message = CrossShardMessage::new(
        "test_tx_id".to_string(),
        0, // from_shard
        1, // to_shard
        CrossShardMessageType::PrepareRequest,
        Some(serde_json::to_vec(&transaction).unwrap()),
    );

    // メッセージを処理
    coordinators[1].process_message(message).await.unwrap();

    // 少し待機
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // トランザクションの詳細を確認
    let details = coordinators[1].get_transaction_details("test_tx_id");
    assert!(details.is_ok());

    let details = details.unwrap();
    assert_eq!(details.coordinator_shard, 0);
    assert!(details.participant_shards.contains(&1));
}

#[tokio::test]
async fn test_cross_shard_transaction_with_multiple_shards() {
    // 5つのシャードを持つテスト環境をセットアップ
    let coordinators = setup_test_environment(5).await;

    // 複数のシャードに影響するトランザクションを作成
    let mut transaction = create_test_transaction();
    transaction.payload = vec![10, 20, 30, 40, 50]; // 複数のシャードに影響するペイロード

    // シャード0からクロスシャードトランザクションを開始
    let tx_id = coordinators[0]
        .start_transaction(transaction)
        .await
        .unwrap();

    // トランザクションが完了するまで待機
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // トランザクションの状態を確認
    let status = coordinators[0].get_transaction_status(&tx_id).unwrap();
    assert_eq!(status, CrossShardTxStatus::Committed);

    // トランザクションの詳細を確認
    let details = coordinators[0].get_transaction_details(&tx_id).unwrap();
    assert!(details.participant_shards.len() > 1); // 複数のシャードに影響していることを確認
    assert!(details.all_prepared());
    assert!(details.all_committed());
}
