use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::crypto::{PublicKey, Signature};
use crate::shard::ShardId;
use crate::transaction::Transaction;

/// マルチシグトランザクション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigTransaction {
    /// トランザクションID
    pub id: String,
    /// 作成者
    pub creator: PublicKey,
    /// 作成時刻
    pub created_at: DateTime<Utc>,
    /// 実行時刻
    pub executed_at: Option<DateTime<Utc>>,
    /// タイムロック解除時刻
    pub timelock_release_at: Option<DateTime<Utc>>,
    /// 署名
    pub signatures: HashMap<PublicKey, Signature>,
    /// 拒否
    pub rejections: HashMap<PublicKey, String>,
    /// ステータス
    pub status: MultisigTransactionStatus,
    /// 基本トランザクション
    pub transaction: Transaction,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// マルチシグトランザクションステータス
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MultisigTransactionStatus {
    /// 保留中
    Pending,
    /// タイムロック中
    TimeLocked,
    /// 実行済み
    Executed,
    /// 拒否
    Rejected,
    /// 期限切れ
    Expired,
    /// キャンセル
    Cancelled,
}

/// トランザクション処理ステップ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStep {
    /// ステップ名
    pub name: String,
    /// 開始時刻
    pub start_time: DateTime<Utc>,
    /// 終了時刻
    pub end_time: Option<DateTime<Utc>>,
    /// 処理シャード
    pub shard_id: Option<ShardId>,
    /// ステータス
    pub status: TransactionStepStatus,
    /// 詳細
    pub details: Option<String>,
}

/// トランザクションステップステータス
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionStepStatus {
    /// 待機中
    Waiting,
    /// 処理中
    InProgress,
    /// 完了
    Completed,
    /// 失敗
    Failed,
    /// スキップ
    Skipped,
}

/// トランザクション履歴エントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionHistoryEntry {
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// アクション
    pub action: TransactionAction,
    /// 実行者
    pub actor: Option<PublicKey>,
    /// 詳細
    pub details: Option<String>,
}

/// トランザクションアクション
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionAction {
    /// 作成
    Created,
    /// 署名追加
    SignatureAdded,
    /// 拒否
    Rejected,
    /// タイムロック設定
    TimeLocked,
    /// タイムロック解除
    TimeLockReleased,
    /// 実行
    Executed,
    /// 期限切れ
    Expired,
    /// キャンセル
    Cancelled,
    /// 更新
    Updated,
}
