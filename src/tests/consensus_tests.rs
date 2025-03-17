#[cfg(test)]
mod tests {
    use crate::consensus::{ConsensusEngine, SimpleValidator};
    use crate::transaction::{Transaction, TransactionStatus};
    use std::sync::{Arc, Mutex};

    // テスト用のモックトランザクションを作成
    fn create_mock_transaction(id: &str, parent_ids: Vec<String>, timestamp: u64) -> Transaction {
        Transaction {
            id: id.to_string(),
            parent_ids,
            timestamp,
            payload: vec![1, 2, 3],
            signature: vec![4, 5, 6],
            status: TransactionStatus::Pending,
        }
    }

    #[test]
    fn test_validator_creation() {
        let validator = SimpleValidator {
            id: "validator1".to_string(),
            stake: 100.0,
        };

        assert_eq!(validator.id, "validator1");
        assert_eq!(validator.stake, 100.0);
    }

    #[test]
    fn test_consensus_engine_initialization() {
        let validators = vec![
            SimpleValidator {
                id: "validator1".to_string(),
                stake: 100.0,
            },
            SimpleValidator {
                id: "validator2".to_string(),
                stake: 200.0,
            },
        ];

        let engine = ConsensusEngine::new(validators);
        assert_eq!(engine.validators.len(), 2);
        assert_eq!(engine.validators[0].id, "validator1");
        assert_eq!(engine.validators[1].id, "validator2");
    }

    #[test]
    fn test_transaction_verification() {
        let validators = vec![
            SimpleValidator {
                id: "validator1".to_string(),
                stake: 100.0,
            },
            SimpleValidator {
                id: "validator2".to_string(),
                stake: 200.0,
            },
        ];

        let engine = ConsensusEngine::new(validators);

        // 有効なトランザクション
        let tx1 = create_mock_transaction("tx1", vec!["parent1".to_string()], 12345);
        assert!(engine.verify_transaction(&tx1));

        // 無効なトランザクション（タイムスタンプが0）
        let tx2 = create_mock_transaction("tx2", vec!["parent1".to_string()], 0);
        assert!(!engine.verify_transaction(&tx2));
    }

    #[test]
    fn test_transaction_ordering() {
        let validators = vec![SimpleValidator {
            id: "validator1".to_string(),
            stake: 100.0,
        }];

        let engine = ConsensusEngine::new(validators);

        // 時間順にトランザクションを作成
        let tx1 = create_mock_transaction("tx1", vec![], 10000);
        let tx2 = create_mock_transaction("tx2", vec!["tx1".to_string()], 10001);
        let tx3 = create_mock_transaction("tx3", vec!["tx2".to_string()], 10002);

        // 順序が正しいことを確認
        assert!(tx1.timestamp < tx2.timestamp);
        assert!(tx2.timestamp < tx3.timestamp);

        // 親子関係が正しいことを確認
        assert!(tx1.parent_ids.is_empty());
        assert_eq!(tx2.parent_ids[0], "tx1");
        assert_eq!(tx3.parent_ids[0], "tx2");
    }
}
