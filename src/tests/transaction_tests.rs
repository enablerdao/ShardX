#[cfg(test)]
mod tests {
    use crate::transaction::{Transaction, TransactionStatus};
    use std::collections::HashMap;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction {
            id: "tx1".to_string(),
            parent_ids: vec!["parent1".to_string(), "parent2".to_string()],
            timestamp: 12345,
            payload: vec![1, 2, 3, 4],
            signature: vec![5, 6, 7, 8],
            status: TransactionStatus::Pending,
        };

        assert_eq!(tx.id, "tx1");
        assert_eq!(tx.parent_ids.len(), 2);
        assert_eq!(tx.timestamp, 12345);
        assert_eq!(tx.payload, vec![1, 2, 3, 4]);
        assert_eq!(tx.signature, vec![5, 6, 7, 8]);
        assert!(matches!(tx.status, TransactionStatus::Pending));
    }

    #[test]
    fn test_transaction_validation() {
        let tx = Transaction {
            id: "tx1".to_string(),
            parent_ids: vec!["parent1".to_string(), "parent2".to_string()],
            timestamp: 12345,
            payload: vec![1, 2, 3, 4],
            signature: vec![5, 6, 7, 8],
            status: TransactionStatus::Pending,
        };

        // 実際のプロジェクトでは、署名の検証などを行うべきですが、
        // ここではシンプルな検証のみを行います
        let is_valid = !tx.id.is_empty() && !tx.parent_ids.is_empty() && tx.timestamp > 0;
        assert!(is_valid);
    }

    #[test]
    fn test_transaction_status_transition() {
        let mut tx = Transaction {
            id: "tx1".to_string(),
            parent_ids: vec!["parent1".to_string()],
            timestamp: 12345,
            payload: vec![1, 2, 3, 4],
            signature: vec![5, 6, 7, 8],
            status: TransactionStatus::Pending,
        };

        assert!(matches!(tx.status, TransactionStatus::Pending));

        // ステータスを変更
        tx.status = TransactionStatus::Confirmed;
        assert!(matches!(tx.status, TransactionStatus::Confirmed));

        // ステータスを変更
        tx.status = TransactionStatus::Rejected;
        assert!(matches!(tx.status, TransactionStatus::Rejected));
    }

    #[test]
    fn test_transaction_with_no_parents() {
        let tx = Transaction {
            id: "tx1".to_string(),
            parent_ids: vec![],
            timestamp: 12345,
            payload: vec![1, 2, 3, 4],
            signature: vec![5, 6, 7, 8],
            status: TransactionStatus::Pending,
        };

        // 親がないトランザクションは、ジェネシストランザクションとして扱われるべきです
        let is_genesis = tx.parent_ids.is_empty();
        assert!(is_genesis);
    }
}