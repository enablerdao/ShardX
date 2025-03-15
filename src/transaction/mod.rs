mod cross_shard_manager;
mod cross_shard_optimizer;
mod benchmarks;
mod parallel_processor;
mod high_throughput_engine;

use std::fmt;
use serde::{Serialize, Deserialize};

pub use cross_shard_manager::{CrossShardManager, CrossShardTransaction, CrossShardTransactionState};
pub use cross_shard_optimizer::{CrossShardOptimizer, OptimizerConfig};
pub use benchmarks::{CrossShardBenchmarker, BenchmarkResult};
pub use parallel_processor::{ParallelProcessor, ProcessorConfig, ProcessorStats};
pub use high_throughput_engine::{HighThroughputEngine, EngineConfig, EngineStats, BenchmarkResult as EngineResult};

/// トランザクションの状態
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// 保留中
    Pending,
    /// 確認済み
    Confirmed,
    /// 失敗
    Failed,
}

impl fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionStatus::Pending => write!(f, "pending"),
            TransactionStatus::Confirmed => write!(f, "confirmed"),
            TransactionStatus::Failed => write!(f, "failed"),
        }
    }
}

/// トランザクション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// トランザクションID
    pub id: String,
    /// 送信者アドレス
    pub from: String,
    /// 受信者アドレス
    pub to: String,
    /// 金額
    pub amount: String,
    /// 手数料
    pub fee: String,
    /// データ（オプション）
    pub data: Option<String>,
    /// ノンス
    pub nonce: u64,
    /// タイムスタンプ
    pub timestamp: u64,
    /// 署名
    pub signature: String,
    /// 状態
    pub status: TransactionStatus,
    /// シャードID
    pub shard_id: String,
    /// ブロックハッシュ（オプション）
    pub block_hash: Option<String>,
    /// ブロック高（オプション）
    pub block_height: Option<u64>,
    /// 親トランザクションID（クロスシャードトランザクションの場合）
    pub parent_id: Option<String>,
}

impl Transaction {
    /// 新しいトランザクションを作成
    pub fn new(
        from: String,
        to: String,
        amount: String,
        fee: String,
        data: Option<String>,
        nonce: u64,
        shard_id: String,
        signature: String,
    ) -> Self {
        let timestamp = chrono::Utc::now().timestamp() as u64;
        let id = generate_transaction_id(&from, &to, &amount, nonce, timestamp);
        
        Self {
            id,
            from,
            to,
            amount,
            fee,
            data,
            nonce,
            timestamp,
            signature,
            status: TransactionStatus::Pending,
            shard_id,
            block_hash: None,
            block_height: None,
            parent_id: None,
        }
    }
    
    /// トランザクションが確認済みかどうかを確認
    pub fn is_confirmed(&self) -> bool {
        self.status == TransactionStatus::Confirmed
    }
    
    /// トランザクションが保留中かどうかを確認
    pub fn is_pending(&self) -> bool {
        self.status == TransactionStatus::Pending
    }
    
    /// トランザクションが失敗したかどうかを確認
    pub fn is_failed(&self) -> bool {
        self.status == TransactionStatus::Failed
    }
    
    /// トランザクションがクロスシャードトランザクションかどうかを確認
    pub fn is_cross_shard(&self) -> bool {
        self.parent_id.is_some()
    }
}

/// トランザクションIDを生成
fn generate_transaction_id(from: &str, to: &str, amount: &str, nonce: u64, timestamp: u64) -> String {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(from.as_bytes());
    hasher.update(to.as_bytes());
    hasher.update(amount.as_bytes());
    hasher.update(nonce.to_string().as_bytes());
    hasher.update(timestamp.to_string().as_bytes());
    
    let result = hasher.finalize();
    hex::encode(result)
}