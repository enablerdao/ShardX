//! HyperFlux.io ユニットテスト
//!
//! このファイルには、HyperFlux.ioの主要コンポーネントのユニットテストが含まれています。

#[cfg(test)]
mod tests {
    use hyperflux::{
        ai::AIPriorityManager,
        consensus::{ConsensusEngine, ProofOfFlow},
        sharding::ShardingManager,
        transaction::Transaction,
        wallet::Wallet,
    };

    /// トランザクション作成と検証のテスト
    #[test]
    fn test_transaction_creation_and_validation() {
        // テスト用のトランザクションを作成
        let parent_ids = vec!["tx_123456".to_string(), "tx_789012".to_string()];
        let payload = "Test transaction".as_bytes().to_vec();
        let tx = Transaction::new(parent_ids, payload);

        // トランザクションIDが生成されていることを確認
        assert!(!tx.id.is_empty(), "トランザクションIDが生成されていること");

        // タイムスタンプが設定されていることを確認
        assert!(tx.timestamp > 0, "タイムスタンプが設定されていること");

        // 親IDが正しく設定されていることを確認
        assert_eq!(tx.parent_ids.len(), 2, "親IDの数が正しいこと");
        assert_eq!(tx.parent_ids[0], "tx_123456", "最初の親IDが正しいこと");
        assert_eq!(tx.parent_ids[1], "tx_789012", "2番目の親IDが正しいこと");

        // ペイロードが正しく設定されていることを確認
        assert_eq!(
            tx.payload,
            "Test transaction".as_bytes().to_vec(),
            "ペイロードが正しいこと"
        );

        // トランザクションの検証
        assert!(tx.is_valid(), "トランザクションが有効であること");
    }

    /// ウォレット作成と署名のテスト
    #[test]
    fn test_wallet_creation_and_signing() {
        // テスト用のウォレットを作成
        let wallet = Wallet::new("test_password");

        // ウォレットIDが生成されていることを確認
        assert!(!wallet.id.is_empty(), "ウォレットIDが生成されていること");

        // アドレスが生成されていることを確認
        assert!(!wallet.address.is_empty(), "アドレスが生成されていること");

        // 公開鍵が生成されていることを確認
        assert!(!wallet.public_key.is_empty(), "公開鍵が生成されていること");

        // 秘密鍵が生成されていることを確認（実際の実装では秘密鍵は直接アクセスできないかもしれません）
        assert!(!wallet.private_key.is_empty(), "秘密鍵が生成されていること");

        // メッセージの署名と検証
        let message = "Test message".as_bytes().to_vec();
        let signature = wallet.sign(&message);
        assert!(wallet.verify(&message, &signature), "署名が検証できること");

        // 改ざんされたメッセージの検証が失敗することを確認
        let tampered_message = "Tampered message".as_bytes().to_vec();
        assert!(
            !wallet.verify(&tampered_message, &signature),
            "改ざんされたメッセージの検証が失敗すること"
        );
    }

    /// シャーディングのテスト
    #[test]
    fn test_sharding() {
        // テスト用のシャーディングマネージャを作成
        let mut sharding_manager = ShardingManager::new(4); // 4シャードで初期化

        // トランザクションIDに基づいてシャードを割り当て
        let tx_id1 = "tx_123456".to_string();
        let tx_id2 = "tx_789012".to_string();
        let shard1 = sharding_manager.assign_shard(&tx_id1);
        let shard2 = sharding_manager.assign_shard(&tx_id2);

        // シャードIDが有効な範囲内であることを確認
        assert!(shard1 < 4, "シャードIDが有効な範囲内であること");
        assert!(shard2 < 4, "シャードIDが有効な範囲内であること");

        // 同じトランザクションIDに対して常に同じシャードが割り当てられることを確認
        assert_eq!(
            sharding_manager.assign_shard(&tx_id1),
            shard1,
            "同じトランザクションIDに対して同じシャードが割り当てられること"
        );

        // シャード数の動的調整
        sharding_manager.adjust_shard_count(8); // シャード数を8に増やす
        assert_eq!(
            sharding_manager.get_shard_count(),
            8,
            "シャード数が正しく調整されること"
        );

        // シャード数を増やした後も、既存のトランザクションは同じシャードにマッピングされるべき
        // （実際の実装では、再シャーディングのロジックによって異なる場合があります）
        let new_shard1 = sharding_manager.assign_shard(&tx_id1);
        assert_eq!(
            new_shard1, shard1,
            "シャード数を変更しても、既存のトランザクションは同じシャードにマッピングされること"
        );
    }

    /// AIによる優先順位付けのテスト
    #[test]
    fn test_ai_prioritization() {
        // テスト用のAI優先順位マネージャを作成
        let ai_manager = AIPriorityManager::new();

        // テスト用のトランザクションを作成
        let tx1 = Transaction::new(vec![], "Small payload".as_bytes().to_vec());
        let tx2 = Transaction::new(vec![], "Very large payload that should have lower priority due to size".as_bytes().to_vec());
        let tx3 = Transaction::new(vec!["parent1".to_string(), "parent2".to_string(), "parent3".to_string()], "Medium payload with many parents".as_bytes().to_vec());

        // 優先度を計算
        let priority1 = ai_manager.calculate_priority(&tx1);
        let priority2 = ai_manager.calculate_priority(&tx2);
        let priority3 = ai_manager.calculate_priority(&tx3);

        // 小さいペイロードのトランザクションが大きいペイロードのトランザクションよりも優先度が高いことを確認
        assert!(
            priority1 > priority2,
            "小さいペイロードのトランザクションの方が優先度が高いこと"
        );

        // 親が多いトランザクションの優先度が高いことを確認
        assert!(
            priority3 > priority2,
            "親が多いトランザクションの方が優先度が高いこと"
        );
    }

    /// コンセンサスエンジンのテスト
    #[test]
    fn test_consensus_engine() {
        // テスト用のコンセンサスエンジンを作成
        let mut consensus = ProofOfFlow::new(4); // 4バリデータで初期化

        // テスト用のトランザクションを作成
        let tx = Transaction::new(vec![], "Test transaction".as_bytes().to_vec());

        // トランザクションを処理
        let result = consensus.process_transaction(&tx);
        assert!(result.is_ok(), "トランザクションが正常に処理されること");

        // トランザクションの状態を確認
        let tx_status = consensus.get_transaction_status(&tx.id);
        assert!(tx_status.is_some(), "トランザクションの状態が存在すること");
        assert_eq!(
            tx_status.unwrap(),
            "pending",
            "トランザクションの状態がpendingであること"
        );

        // バリデータの承認をシミュレート
        for i in 0..4 {
            consensus.validate_transaction(&tx.id, &format!("validator_{}", i));
        }

        // トランザクションが確認済みになっていることを確認
        let tx_status = consensus.get_transaction_status(&tx.id);
        assert_eq!(
            tx_status.unwrap(),
            "confirmed",
            "トランザクションの状態がconfirmedであること"
        );
    }
}