mod cross_shard_manager;

use std::fmt;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;

pub use cross_shard_manager::{CrossShardManager, CrossShardTransaction, CrossShardTransactionState};

/// トランザクションの状態
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// 保留中
    Pending,
    /// 確認済み
    Confirmed,
    /// 失敗
    Failed,
    /// 拒否
    Rejected,
}

impl fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionStatus::Pending => write!(f, "pending"),
            TransactionStatus::Confirmed => write!(f, "confirmed"),
            TransactionStatus::Failed => write!(f, "failed"),
            TransactionStatus::Rejected => write!(f, "rejected"),
        }
    }
}

/// 基本トランザクショントレイト
/// すべてのトランザクション実装が実装すべき共通インターフェース
#[async_trait]
pub trait BaseTransaction: Send + Sync + Clone {
    /// トランザクションIDを取得
    fn id(&self) -> &str;
    
    /// タイムスタンプを取得
    fn timestamp(&self) -> u64;
    
    /// 署名を取得
    fn signature(&self) -> &str;
    
    /// シャードIDを取得
    fn shard_id(&self) -> &str;
    
    /// トランザクションが確認済みかどうかを確認
    fn is_confirmed(&self) -> bool;
    
    /// トランザクションが保留中かどうかを確認
    fn is_pending(&self) -> bool;
    
    /// トランザクションが失敗したかどうかを確認
    fn is_failed(&self) -> bool;
    
    /// トランザクションがクロスシャードトランザクションかどうかを確認
    fn is_cross_shard(&self) -> bool;
    
    /// トランザクションを検証
    async fn validate(&self) -> Result<(), crate::error::Error>;
    
    /// トランザクションをシリアライズ
    fn serialize(&self) -> Result<Vec<u8>, crate::error::Error>;
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
    /// ペイロード（バイナリデータ）
    pub payload: Vec<u8>,
    /// 親トランザクションIDs（複数の親を持つ場合）
    pub parent_ids: Vec<String>,
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
            payload: Vec::new(),
            parent_ids: Vec::new(),
        }
    }
    
    /// シリアライズ可能なデータを生成
    pub fn to_signable(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.id.as_bytes());
        data.extend_from_slice(self.from.as_bytes());
        data.extend_from_slice(self.to.as_bytes());
        data.extend_from_slice(self.amount.as_bytes());
        data.extend_from_slice(self.fee.as_bytes());
        if let Some(ref d) = self.data {
            data.extend_from_slice(d.as_bytes());
        }
        data.extend_from_slice(&self.nonce.to_be_bytes());
        data.extend_from_slice(&self.timestamp.to_be_bytes());
        data.extend_from_slice(self.shard_id.as_bytes());
        data
    }
}

#[async_trait]
impl BaseTransaction for Transaction {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp
    }
    
    fn signature(&self) -> &str {
        &self.signature
    }
    
    fn shard_id(&self) -> &str {
        &self.shard_id
    }
    
    fn is_confirmed(&self) -> bool {
        self.status == TransactionStatus::Confirmed
    }
    
    fn is_pending(&self) -> bool {
        self.status == TransactionStatus::Pending
    }
    
    fn is_failed(&self) -> bool {
        self.status == TransactionStatus::Failed
    }
    
    fn is_cross_shard(&self) -> bool {
        self.parent_id.is_some()
    }
    
    async fn validate(&self) -> Result<(), crate::error::Error> {
        // 基本的な検証ロジック
        if self.id.is_empty() {
            return Err(crate::error::Error::ValidationError("Empty transaction ID".to_string()));
        }
        
        if self.from.is_empty() {
            return Err(crate::error::Error::ValidationError("Empty sender address".to_string()));
        }
        
        if self.to.is_empty() {
            return Err(crate::error::Error::ValidationError("Empty recipient address".to_string()));
        }
        
        if self.signature.is_empty() {
            return Err(crate::error::Error::ValidationError("Empty signature".to_string()));
        }
        
        Ok(())
    }
    
    fn serialize(&self) -> Result<Vec<u8>, crate::error::Error> {
        serde_json::to_vec(self)
            .map_err(|e| crate::error::Error::SerializeError(e.to_string()))
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
