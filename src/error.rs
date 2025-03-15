use thiserror::Error;
use std::io;

/// ShardXのエラー型
#[derive(Error, Debug)]
pub enum Error {
    /// 無効な入力
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    /// 無効な署名
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    /// 無効なトランザクション
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    
    /// 無効なシャードID
    #[error("Invalid shard ID: {0}")]
    InvalidShardId(u32),
    
    /// 無効なノードID
    #[error("Invalid node ID: {0}")]
    InvalidNodeId(String),
    
    /// 無効なトランザクションID
    #[error("Invalid transaction ID: {0}")]
    InvalidTransactionId(String),
    
    /// 無効なキー
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    
    /// タイムスタンプエラー
    #[error("Timestamp error: {0}")]
    TimestampError(String),
    
    /// ストレージエラー
    #[error("Storage error: {0}")]
    StorageError(String),
    
    /// ネットワークエラー
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// I/Oエラー
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
    
    /// シリアライズエラー
    #[error("Serialization error: {0}")]
    SerializeError(String),
    
    /// デシリアライズエラー
    #[error("Deserialization error: {0}")]
    DeserializeError(String),
    
    /// 内部エラー
    #[error("Internal error: {0}")]
    InternalError(String),
    
    /// タイムアウト
    #[error("Timeout: {0}")]
    Timeout(String),
    
    /// 重複
    #[error("Duplicate: {0}")]
    Duplicate(String),
    
    /// 重複トランザクション
    #[error("Duplicate transaction: {0}")]
    DuplicateTransaction(String),
    
    /// 未実装
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    
    /// 権限エラー
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// リソース不足
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),
    
    /// レート制限
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    /// 検証エラー
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// 無効な操作
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    /// トランザクションが見つからない
    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),
    
    /// ウォレットが見つからない
    #[error("Wallet not found: {0}")]
    WalletNotFound(String),
    
    /// アカウントが見つからない
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    
    /// 無効なトランザクション状態
    #[error("Invalid transaction status: {0}")]
    InvalidTransactionStatus(String),
    
    /// 権限のない署名者
    #[error("Unauthorized signer: {0}")]
    UnauthorizedSigner(String),
    
    /// 既に署名済み
    #[error("Already signed: {0}")]
    AlreadySigned(String),
    
    /// セキュリティ違反
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    
    /// 不明なエラー
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<prost::EncodeError> for Error {
    fn from(err: prost::EncodeError) -> Self {
        Error::SerializeError(err.to_string())
    }
}

impl From<prost::DecodeError> for Error {
    fn from(err: prost::DecodeError) -> Self {
        Error::DeserializeError(err.to_string())
    }
}

impl From<bincode::Error> for Error {
    fn from(err: bincode::Error) -> Self {
        Error::SerializeError(err.to_string())
    }
}

impl From<rocksdb::Error> for Error {
    fn from(err: rocksdb::Error) -> Self {
        Error::StorageError(err.to_string())
    }
}