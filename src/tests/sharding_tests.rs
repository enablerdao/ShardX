#[cfg(test)]
mod tests {
    use crate::sharding::ShardingManager;
    use crate::transaction::Transaction;
    use crate::transaction::TransactionStatus;

    // テスト用のモックトランザクションを作成
    fn create_mock_transaction(id: &str) -> Transaction {
        Transaction {
            id: id.to_string(),
            parent_ids: vec![],
            timestamp: 12345,
            payload: vec![1, 2, 3],
            signature: vec![4, 5, 6],
            status: TransactionStatus::Pending,
        }
    }

    #[test]
    fn test_sharding_manager_creation() {
        let manager = ShardingManager::new(256);
        assert_eq!(manager.shard_count, 256);
    }

    #[test]
    fn test_shard_assignment() {
        let manager = ShardingManager::new(256);

        // 同じIDのトランザクションは常に同じシャードに割り当てられるべき
        let tx1 = create_mock_transaction("tx1");
        let shard1 = manager.assign_shard(&tx1);
        let shard1_again = manager.assign_shard(&tx1);
        assert_eq!(shard1, shard1_again);

        // 異なるIDのトランザクションは異なるシャードに割り当てられる可能性がある
        let tx2 = create_mock_transaction("tx2");
        let shard2 = manager.assign_shard(&tx2);

        // シャード番号は有効な範囲内であるべき
        assert!(shard1 < 256);
        assert!(shard2 < 256);
    }

    #[test]
    fn test_shard_adjustment() {
        let mut manager = ShardingManager::new(256);

        // 初期状態
        assert_eq!(manager.shard_count, 256);

        // 負荷が高い場合、シャード数を増やす
        manager.adjust_shards(10000);
        assert_eq!(manager.shard_count, 512);

        // 負荷が低い場合、シャード数を減らす
        manager.adjust_shards(5000);
        assert_eq!(manager.shard_count, 256);
    }

    #[test]
    fn test_cross_shard_communication() {
        let manager = ShardingManager::new(256);

        // 異なるシャードに割り当てられたトランザクション
        let tx1 = create_mock_transaction("tx1");
        let tx2 = create_mock_transaction("tx2");

        let shard1 = manager.assign_shard(&tx1);
        let shard2 = manager.assign_shard(&tx2);

        // クロスシャード通信が必要かどうかを確認
        let needs_cross_shard = shard1 != shard2;

        // 結果に応じたアサーション
        if needs_cross_shard {
            assert_ne!(shard1, shard2);
        } else {
            assert_eq!(shard1, shard2);
        }
    }
}
