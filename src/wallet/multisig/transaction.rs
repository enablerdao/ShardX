use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use async_trait::async_trait;

use crate::crypto::{PublicKey, Signature};
use crate::transaction::{Transaction, BaseTransaction};
use crate::shard::ShardId;
use crate::error::Error;

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
    /// シャードID（基本トランザクションから取得）
    pub shard_id: String,
    /// タイムスタンプ（作成時刻から取得）
    pub timestamp: u64,
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

#[async_trait]
impl BaseTransaction for MultisigTransaction {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp
    }
    
    fn signature(&self) -> &str {
        // マルチシグトランザクションでは複数の署名があるため、
        // 最初の署名のキーを返す（実際の実装ではより適切な方法が必要）
        if let Some((_, _)) = self.signatures.iter().next() {
            "multisig"
        } else {
            ""
        }
    }
    
    fn shard_id(&self) -> &str {
        &self.shard_id
    }
    
    fn is_confirmed(&self) -> bool {
        self.status == MultisigTransactionStatus::Executed
    }
    
    fn is_pending(&self) -> bool {
        self.status == MultisigTransactionStatus::Pending || 
        self.status == MultisigTransactionStatus::TimeLocked
    }
    
    fn is_failed(&self) -> bool {
        self.status == MultisigTransactionStatus::Rejected || 
        self.status == MultisigTransactionStatus::Expired ||
        self.status == MultisigTransactionStatus::Cancelled
    }
    
    fn is_cross_shard(&self) -> bool {
        // マルチシグトランザクションは常にクロスシャードとして扱う
        true
    }
    
    async fn validate(&self) -> Result<(), Error> {
        // 基本的な検証ロジック
        if self.id.is_empty() {
            return Err(Error::ValidationError("Empty transaction ID".to_string()));
        }
        
        if self.signatures.is_empty() {
            return Err(Error::ValidationError("No signatures".to_string()));
        }
        
        // 基本トランザクションの検証
        self.transaction.validate().await?;
        
        Ok(())
    }
    
    fn serialize(&self) -> Result<Vec<u8>, Error> {
        serde_json::to_vec(self)
            .map_err(|e| Error::SerializeError(e.to_string()))
    }
}
