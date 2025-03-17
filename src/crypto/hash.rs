use crate::error::Error;
use crate::transaction::Transaction;
use blake3::Hasher;
use rayon::prelude::*;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;

/// Blake3ハッシュを使用した軽量ハッシュ計算マネージャー
pub struct HashManager {
    thread_pool: ThreadPool,
    thread_count: usize,
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
            thread_count,
        }
    }

    /// トランザクションのハッシュを計算
    ///
    /// # Arguments
    /// * `tx` - ハッシュを計算するトランザクション
    pub fn hash_transaction(&self, tx: &Transaction) -> String {
        // トランザクションをシリアライズ
        let serialized = bincode::serialize(tx).unwrap_or_default();

        // BLAKE3でハッシュ化
        let mut hasher = Hasher::new();
        hasher.update(&serialized);
        let hash = hasher.finalize();

        // 16進数文字列に変換
        hex::encode(hash.as_bytes())
    }

    /// 複数のトランザクションを並列ハッシュ化
    pub fn hash_transactions(&self, txs: &[Transaction]) -> Vec<String> {
        // Rayonを使用して並列処理
        txs.par_iter().map(|tx| self.hash_transaction(tx)).collect()
    }

    /// マークルツリーのルートハッシュを計算
    pub fn compute_merkle_root(&self, hashes: &[String]) -> Result<String, Error> {
        if hashes.is_empty() {
            return Err(Error::InvalidInput("Empty hash list".to_string()));
        }

        if hashes.len() == 1 {
            return Ok(hashes[0].clone());
        }

        // マークルツリーを構築
        let mut current_level = hashes.to_vec();

        while current_level.len() > 1 {
            let mut next_level = Vec::with_capacity((current_level.len() + 1) / 2);

            // 現在のレベルのハッシュをペアにして次のレベルのハッシュを計算
            for chunk in current_level.chunks(2) {
                let left = &chunk[0];
                let right = if chunk.len() > 1 { &chunk[1] } else { left };

                let combined = format!("{}{}", left, right);
                let mut hasher = Hasher::new();
                hasher.update(combined.as_bytes());
                let hash = hasher.finalize();

                next_level.push(hex::encode(hash.as_bytes()));
            }

            current_level = next_level;
        }

        Ok(current_level[0].clone())
    }

    /// データをハッシュ化
    pub fn hash_data(&self, data: &[u8]) -> String {
        let mut hasher = Hasher::new();
        hasher.update(data);
        let hash = hasher.finalize();

        hex::encode(hash.as_bytes())
    }

    /// 大きなデータを並列ハッシュ化
    pub fn hash_large_data(&self, data: &[u8]) -> String {
        if data.len() < 1024 * 1024 || self.thread_count <= 1 {
            // 小さなデータまたは単一スレッドの場合は通常のハッシュ
            return self.hash_data(data);
        }

        // データをチャンクに分割
        let chunk_size = data.len() / self.thread_count;
        let chunks: Vec<&[u8]> = data.chunks(chunk_size).collect();

        // 各チャンクを並列ハッシュ化
        let chunk_hashes: Vec<String> = chunks
            .par_iter()
            .map(|chunk| self.hash_data(chunk))
            .collect();

        // チャンクハッシュを結合して最終ハッシュを計算
        let combined = chunk_hashes.join("");
        let mut hasher = Hasher::new();
        hasher.update(combined.as_bytes());
        let hash = hasher.finalize();

        hex::encode(hash.as_bytes())
    }

    /// ハッシュを検証
    pub fn verify_hash(&self, data: &[u8], expected_hash: &str) -> bool {
        let actual_hash = self.hash_data(data);
        actual_hash == expected_hash
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
