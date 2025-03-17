//! # トランザクション処理モジュール
//!
//! このモジュールは、ShardXのトランザクション処理機能を提供します。
//! トランザクション処理は、ブロックチェーンの中核機能であり、
//! 安全で効率的なトランザクションの検証、実行、保存を担当します。
//!
//! ## 主な機能
//!
//! - トランザクションの検証と処理
//! - クロスシャードトランザクション管理
//! - 並列トランザクション処理
//! - 高スループットエンジン
//! - パフォーマンスベンチマーク
//!
//! ## 使用例
//!
//! ```rust
//! use shardx::transaction::{Transaction, TransactionStatus};
//!
//! // トランザクションを作成
//! let tx = Transaction::new(
//!     "sender_address".to_string(),
//!     "receiver_address".to_string(),
//!     "100".to_string(),
//!     "0.01".to_string(),
//!     None,
//!     1,
//!     "shard-1".to_string(),
//!     "signature".to_string()
//! );
//!
//! // トランザクションの状態を確認
//! assert!(tx.is_pending());
//! assert!(!tx.is_confirmed());
//! ```

mod benchmarks;
mod cross_shard_manager;
mod cross_shard_optimizer;
mod high_throughput_engine;
mod parallel_processor;

use serde::{Deserialize, Serialize};
use std::fmt;

pub use benchmarks::{BenchmarkResult, CrossShardBenchmarker};
pub use cross_shard_manager::{
    CrossShardManager, CrossShardTransaction, CrossShardTransactionState,
};
pub use cross_shard_optimizer::{CrossShardOptimizer, OptimizerConfig};
pub use high_throughput_engine::{
    BenchmarkResult as EngineResult, EngineConfig, EngineStats, HighThroughputEngine,
};
pub use parallel_processor::{ParallelProcessor, ProcessorConfig, ProcessorStats};

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
///
/// ブロックチェーン上で処理される取引を表します。
/// トランザクションには、送信者、受信者、金額、手数料などの情報が含まれます。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// トランザクションID
    ///
    /// トランザクションを一意に識別するハッシュ値です。
    pub id: String,

    /// 送信者アドレス
    ///
    /// トランザクションの送信者のブロックチェーンアドレスです。
    pub from: String,

    /// 受信者アドレス
    ///
    /// トランザクションの受信者のブロックチェーンアドレスです。
    pub to: String,

    /// 金額
    ///
    /// 送信される通貨の量を文字列で表したものです。
    pub amount: String,

    /// 手数料
    ///
    /// トランザクション処理のために支払われる手数料です。
    pub fee: String,

    /// データ（オプション）
    ///
    /// トランザクションに添付される追加データです。
    /// スマートコントラクトの呼び出しなどに使用されます。
    pub data: Option<String>,

    /// ノンス
    ///
    /// トランザクションの順序を保証し、リプレイ攻撃を防ぐための値です。
    pub nonce: u64,

    /// タイムスタンプ
    ///
    /// トランザクションが作成された時刻のUNIXタイムスタンプです。
    pub timestamp: u64,

    /// 署名
    ///
    /// トランザクションの送信者による電子署名です。
    /// この署名により、トランザクションの真正性と完全性が保証されます。
    pub signature: String,

    /// 状態
    ///
    /// トランザクションの現在の処理状態です。
    pub status: TransactionStatus,

    /// シャードID
    ///
    /// このトランザクションが処理されるシャードの識別子です。
    pub shard_id: String,

    /// ブロックハッシュ（オプション）
    ///
    /// トランザクションが含まれるブロックのハッシュ値です。
    /// トランザクションがまだブロックに含まれていない場合はNoneです。
    pub block_hash: Option<String>,

    /// ブロック高（オプション）
    ///
    /// トランザクションが含まれるブロックの高さです。
    /// トランザクションがまだブロックに含まれていない場合はNoneです。
    pub block_height: Option<u64>,

    /// 親トランザクションID（クロスシャードトランザクションの場合）
    ///
    /// クロスシャードトランザクションの場合、親トランザクションのIDです。
    /// 通常のトランザクションの場合はNoneです。
    pub parent_id: Option<String>,
}

impl Transaction {
    /// 新しいトランザクションを作成します。
    ///
    /// # 引数
    ///
    /// * `from` - 送信者のアドレス
    /// * `to` - 受信者のアドレス
    /// * `amount` - 送信する金額
    /// * `fee` - トランザクション手数料
    /// * `data` - 追加データ（オプション）
    /// * `nonce` - トランザクションのノンス
    /// * `shard_id` - 処理するシャードのID
    /// * `signature` - トランザクションの署名
    ///
    /// # 戻り値
    ///
    /// 新しく作成されたトランザクションのインスタンス
    ///
    /// # 例
    ///
    /// ```
    /// use shardx::transaction::Transaction;
    ///
    /// let tx = Transaction::new(
    ///     "sender_address".to_string(),
    ///     "receiver_address".to_string(),
    ///     "100".to_string(),
    ///     "0.01".to_string(),
    ///     None,
    ///     1,
    ///     "shard-1".to_string(),
    ///     "signature".to_string()
    /// );
    /// ```
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

    /// トランザクションが確認済みかどうかを確認します。
    ///
    /// # 戻り値
    ///
    /// トランザクションが確認済みの場合は`true`、そうでない場合は`false`
    pub fn is_confirmed(&self) -> bool {
        self.status == TransactionStatus::Confirmed
    }

    /// トランザクションが保留中かどうかを確認します。
    ///
    /// # 戻り値
    ///
    /// トランザクションが保留中の場合は`true`、そうでない場合は`false`
    pub fn is_pending(&self) -> bool {
        self.status == TransactionStatus::Pending
    }

    /// トランザクションが失敗したかどうかを確認します。
    ///
    /// # 戻り値
    ///
    /// トランザクションが失敗した場合は`true`、そうでない場合は`false`
    pub fn is_failed(&self) -> bool {
        self.status == TransactionStatus::Failed
    }

    /// トランザクションがクロスシャードトランザクションかどうかを確認します。
    ///
    /// クロスシャードトランザクションは、複数のシャードにまたがって処理される
    /// トランザクションです。これらは親トランザクションIDを持ちます。
    ///
    /// # 戻り値
    ///
    /// トランザクションがクロスシャードトランザクションの場合は`true`、
    /// そうでない場合は`false`
    pub fn is_cross_shard(&self) -> bool {
        self.parent_id.is_some()
    }
}

/// トランザクションIDを生成します。
///
/// トランザクションの各フィールドを組み合わせてハッシュ化し、
/// 一意のトランザクションIDを生成します。
///
/// # 引数
///
/// * `from` - 送信者のアドレス
/// * `to` - 受信者のアドレス
/// * `amount` - 送信する金額
/// * `nonce` - トランザクションのノンス
/// * `timestamp` - トランザクションのタイムスタンプ
///
/// # 戻り値
///
/// 16進数文字列としてエンコードされたトランザクションID
fn generate_transaction_id(
    from: &str,
    to: &str,
    amount: &str,
    nonce: u64,
    timestamp: u64,
) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(from.as_bytes());
    hasher.update(to.as_bytes());
    hasher.update(amount.as_bytes());
    hasher.update(nonce.to_string().as_bytes());
    hasher.update(timestamp.to_string().as_bytes());

    let result = hasher.finalize();
    hex::encode(result)
}
