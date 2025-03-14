use blake3::Hash as Blake3Hash;
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;
use std::sync::mpsc::channel;
use crate::transaction::Transaction;
use crate::error::Error;

/// Blake3ハッシュを使用した軽量ハッシュ計算マネージャー
pub struct HashManager {
    thread_pool: ThreadPool,
}

impl HashManager {
    /// 新しいHashManagerを作成
    /// 
    /// # Arguments
    /// * `max_threads` - 使用する最大スレッド数
    pub fn new(max_threads: usize) -> Self {
        // スレッドプールを作成（CPUコア数の半分を使用）
        let thread_count = std::cmp::min(max_threads, num_cpus::get() / 2);
        
        Self {
            thread_pool: ThreadPool::new(thread_count),
        }
    }
    
    /// トランザクションのハッシュを計算
    /// 
    /// # Arguments
    /// * `tx` - ハッシュを計算するトランザクション
    pub fn hash_transaction(&self, tx: &Transaction) -> Blake3Hash {
        // Blake3ハッシュを使用（SHA-256の半分の負荷）
        // 0.05ms以下の処理時間を目標
        blake3::hash(&tx.serialize())
    }
    
    /// トランザクションのバッチ検証
    /// 
    /// # Arguments
    /// * `txs` - 検証するトランザクションのベクター
    pub fn verify_batch(&self, txs: Vec<Transaction>) -> Vec<Result<(), Error>> {
        let (sender, receiver) = channel();
        let txs_len = txs.len();
        
        for tx in txs {
            let sender = sender.clone();
            
            self.thread_pool.execute(move || {
                // 署名検証を別スレッドで実行
                let result = verify_signature(&tx);
                sender.send((tx.id.clone(), result)).unwrap();
            });
        }
        
        // 結果を収集
        let mut results = Vec::with_capacity(txs_len);
        for _ in 0..txs_len {
            if let Ok((id, result)) = receiver.recv() {
                results.push((id, result));
            }
        }
        
        // IDでソート
        results.sort_by(|(id_a, _), (id_b, _)| id_a.cmp(id_b));
        
        // 結果のみを返す
        results.into_iter().map(|(_, result)| result).collect()
    }
    
    /// 単一トランザクションの署名を検証
    /// 
    /// # Arguments
    /// * `tx` - 検証するトランザクション
    pub fn verify_signature(&self, tx: &Transaction) -> Result<(), Error> {
        verify_signature(tx)
    }
}

/// トランザクションの署名を検証
/// 
/// # Arguments
/// * `tx` - 検証するトランザクション
fn verify_signature(tx: &Transaction) -> Result<(), Error> {
    // 実際の署名検証ロジックを実装
    // 現在はダミー実装
    if tx.signature.is_empty() {
        return Err(Error::InvalidSignature);
    }
    
    // 署名が有効と仮定
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{Transaction, TransactionStatus};
    
    #[test]
    fn test_hash_transaction() {
        let tx = Transaction {
            id: "tx1".to_string(),
            parent_ids: vec!["parent1".to_string()],
            timestamp: 12345,
            payload: vec![1, 2, 3],
            signature: vec![4, 5, 6],
            status: TransactionStatus::Pending,
        };
        
        let hash_manager = HashManager::new(2);
        let hash = hash_manager.hash_transaction(&tx);
        
        // ハッシュが空でないことを確認
        assert!(!hash.as_bytes().is_empty());
    }
    
    #[test]
    fn test_verify_signature() {
        let tx = Transaction {
            id: "tx1".to_string(),
            parent_ids: vec!["parent1".to_string()],
            timestamp: 12345,
            payload: vec![1, 2, 3],
            signature: vec![4, 5, 6], // 有効な署名と仮定
            status: TransactionStatus::Pending,
        };
        
        let hash_manager = HashManager::new(2);
        let result = hash_manager.verify_signature(&tx);
        
        // 署名が有効であることを確認
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_verify_batch() {
        let txs = vec![
            Transaction {
                id: "tx1".to_string(),
                parent_ids: vec!["parent1".to_string()],
                timestamp: 12345,
                payload: vec![1, 2, 3],
                signature: vec![4, 5, 6], // 有効な署名と仮定
                status: TransactionStatus::Pending,
            },
            Transaction {
                id: "tx2".to_string(),
                parent_ids: vec!["parent2".to_string()],
                timestamp: 12346,
                payload: vec![7, 8, 9],
                signature: vec![10, 11, 12], // 有効な署名と仮定
                status: TransactionStatus::Pending,
            },
        ];
        
        let hash_manager = HashManager::new(2);
        let results = hash_manager.verify_batch(txs);
        
        // すべての署名が有効であることを確認
        assert_eq!(results.len(), 2);
        for result in results {
            assert!(result.is_ok());
        }
    }
}