use std::fmt;
use std::error::Error as StdError;
use std::io;

/// ShardXのエラー型
#[derive(Debug)]
pub enum Error {
    /// 無効な署名
    InvalidSignature,
    /// 無効なトランザクション
    InvalidTransaction(String),
    /// 無効なシャードID
    InvalidShardId(u32),
    /// 無効なノードID
    InvalidNodeId(String),
    /// タイムスタンプエラー
    TimestampError(String),
    /// ストレージエラー
    StorageError(String),
    /// ネットワークエラー
    NetworkError(String),
    /// I/Oエラー
    IoError(io::Error),
    /// シリアライズエラー
    SerializeError(String),
    /// デシリアライズエラー
    DeserializeError(String),
    /// 内部エラー
    InternalError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidSignature => write!(f, "Invalid signature"),
            Error::InvalidTransaction(msg) => write!(f, "Invalid transaction: {}", msg),
            Error::InvalidShardId(id) => write!(f, "Invalid shard ID: {}", id),
            Error::InvalidNodeId(id) => write!(f, "Invalid node ID: {}", id),
            Error::TimestampError(msg) => write!(f, "Timestamp error: {}", msg),
            Error::StorageError(msg) => write!(f, "Storage error: {}", msg),
            Error::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Error::IoError(err) => write!(f, "I/O error: {}", err),
            Error::SerializeError(msg) => write!(f, "Serialize error: {}", msg),
            Error::DeserializeError(msg) => write!(f, "Deserialize error: {}", msg),
            Error::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
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