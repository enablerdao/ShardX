pub mod ai;
pub mod api;
pub mod api_handlers;
pub mod consensus;
pub mod dex;
pub mod node;
pub mod sharding;
pub mod transaction;
pub mod wallet;

#[cfg(test)]
mod tests {
    use super::*;
    use transaction::{DAG, Transaction, TransactionStatus};

    #[test]
    fn test_transaction_creation() {
        let parent_ids = vec!["parent1".to_string(), "parent2".to_string()];
        let payload = b"test payload".to_vec();
        let signature = b"test signature".to_vec();
        
        let tx = Transaction::new(parent_ids.clone(), payload.clone(), signature.clone());
        
        assert_eq!(tx.parent_ids, parent_ids);
        assert_eq!(tx.payload, payload);
        assert_eq!(tx.signature, signature);
        assert_eq!(tx.status, TransactionStatus::Pending);
    }
    
    #[test]
    fn test_transaction_hash() {
        let tx1 = Transaction::new(vec!["parent1".to_string()], b"payload1".to_vec(), b"sig1".to_vec());
        let tx2 = Transaction::new(vec!["parent1".to_string()], b"payload1".to_vec(), b"sig1".to_vec());
        let tx3 = Transaction::new(vec!["parent2".to_string()], b"payload2".to_vec(), b"sig2".to_vec());
        
        // 同じ内容のトランザクションでもIDが異なるため、ハッシュは異なる
        assert_ne!(tx1.hash(), tx2.hash());
        assert_ne!(tx1.hash(), tx3.hash());
    }
    
    #[test]
    fn test_dag_operations() {
        let dag = DAG::new();
        
        // 最初のトランザクションを作成（親なし）
        let tx1 = Transaction::new(vec![], b"payload1".to_vec(), b"sig1".to_vec());
        let tx1_id = tx1.id.clone();
        
        // DAGに追加
        assert!(dag.add_transaction(tx1).is_ok());
        
        // 2つ目のトランザクションを作成（tx1を親に持つ）
        let tx2 = Transaction::new(vec![tx1_id.clone()], b"payload2".to_vec(), b"sig2".to_vec());
        let tx2_id = tx2.id.clone();
        
        // DAGに追加
        assert!(dag.add_transaction(tx2).is_ok());
        
        // トランザクションの取得
        let retrieved_tx1 = dag.get_transaction(&tx1_id);
        assert!(retrieved_tx1.is_some());
        assert_eq!(retrieved_tx1.unwrap().id, tx1_id);
        
        // トランザクションのステータス更新
        assert!(dag.update_transaction_status(&tx1_id, TransactionStatus::Confirmed).is_ok());
        
        // 更新後のステータスを確認
        let updated_tx1 = dag.get_transaction(&tx1_id);
        assert_eq!(updated_tx1.unwrap().status, TransactionStatus::Confirmed);
        
        // 確認済みトランザクション数を確認
        assert_eq!(dag.confirmed_count(), 1);
    }
}