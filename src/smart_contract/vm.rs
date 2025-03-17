use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::smart_contract::event::ContractEvent;

/// 実行コンテキスト
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// ガス制限
    pub gas_limit: u64,
    /// 送信者
    pub sender: String,
    /// 値
    pub value: u64,
    /// データ
    pub data: Vec<u8>,
    /// アドレス
    pub address: Option<String>,
    /// ブロック高
    pub block_height: u64,
    /// ブロック時間
    pub block_time: DateTime<Utc>,
    /// 静的呼び出しフラグ
    pub is_static: bool,
    /// 呼び出し深度
    pub depth: usize,
}

/// 実行結果
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// 成功フラグ
    pub success: bool,
    /// 戻りデータ
    pub return_data: Vec<u8>,
    /// ガス使用量
    pub gas_used: u64,
    /// メモリ使用量
    pub memory_used: u64,
    /// ストレージ使用量
    pub storage_used: u64,
    /// ストレージ読み取り数
    pub storage_reads: u64,
    /// ストレージ書き込み数
    pub storage_writes: u64,
    /// ストレージ削除数
    pub storage_deletes: u64,
    /// イベント
    pub events: Vec<ContractEvent>,
    /// ログ
    pub logs: Vec<String>,
    /// アドレス
    pub address: String,
    /// エラー
    pub error: Option<VMError>,
}

/// VM エラー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VMError {
    /// 無効なオペコード
    InvalidOpcode(u8),
    /// スタックオーバーフロー
    StackOverflow,
    /// スタックアンダーフロー
    StackUnderflow,
    /// メモリオーバーフロー
    MemoryOverflow,
    /// ストレージオーバーフロー
    StorageOverflow,
    /// ガス不足
    OutOfGas,
    /// 呼び出し深度超過
    CallDepthExceeded,
    /// 静的コンテキストでの状態変更
    StateChangeInStaticCall,
    /// 無効なジャンプ先
    InvalidJumpDestination,
    /// 無効なアドレス
    InvalidAddress(String),
    /// 無効なメソッド
    InvalidMethod(String),
    /// 無効な引数
    InvalidArguments(String),
    /// 実行タイムアウト
    ExecutionTimeout,
    /// 内部エラー
    InternalError(String),
    /// カスタムエラー
    Custom(String),
}

impl From<VMError> for Error {
    fn from(error: VMError) -> Self {
        match error {
            VMError::InvalidOpcode(opcode) => Error::InvalidInput(format!("Invalid opcode: {}", opcode)),
            VMError::StackOverflow => Error::InvalidState("Stack overflow".to_string()),
            VMError::StackUnderflow => Error::InvalidState("Stack underflow".to_string()),
            VMError::MemoryOverflow => Error::ResourceExhausted("Memory overflow".to_string()),
            VMError::StorageOverflow => Error::ResourceExhausted("Storage overflow".to_string()),
            VMError::OutOfGas => Error::ResourceExhausted("Out of gas".to_string()),
            VMError::CallDepthExceeded => Error::ResourceExhausted("Call depth exceeded".to_string()),
            VMError::StateChangeInStaticCall => Error::InvalidOperation("State change in static call".to_string()),
            VMError::InvalidJumpDestination => Error::InvalidInput("Invalid jump destination".to_string()),
            VMError::InvalidAddress(address) => Error::InvalidInput(format!("Invalid address: {}", address)),
            VMError::InvalidMethod(method) => Error::InvalidInput(format!("Invalid method: {}", method)),
            VMError::InvalidArguments(args) => Error::InvalidInput(format!("Invalid arguments: {}", args)),
            VMError::ExecutionTimeout => Error::Timeout("Execution timeout".to_string()),
            VMError::InternalError(msg) => Error::Internal(msg),
            VMError::Custom(msg) => Error::Custom(msg),
        }
    }
}

impl From<Error> for VMError {
    fn from(error: Error) -> Self {
        match error {
            Error::InvalidInput(msg) => VMError::Custom(msg),
            Error::InvalidState(msg) => VMError::Custom(msg),
            Error::ResourceExhausted(msg) => VMError::Custom(msg),
            Error::InvalidOperation(msg) => VMError::Custom(msg),
            Error::Timeout(msg) => VMError::ExecutionTimeout,
            Error::Internal(msg) => VMError::InternalError(msg),
            Error::Custom(msg) => VMError::Custom(msg),
            _ => VMError::Custom(format!("{:?}", error)),
        }
    }
}

/// 仮想マシン
pub trait VirtualMachine {
    /// コントラクトをデプロイ
    fn deploy(&self, code: Vec<u8>, context: ExecutionContext) -> Result<ExecutionResult, VMError>;
    
    /// コントラクトを呼び出し
    fn call(&self, address: String, method: String, context: ExecutionContext) -> Result<ExecutionResult, VMError>;
    
    /// コントラクトを更新
    fn update(&self, address: String, code: Vec<u8>, context: ExecutionContext) -> Result<ExecutionResult, VMError>;
    
    /// コントラクトを削除
    fn delete(&self, address: String, context: ExecutionContext) -> Result<ExecutionResult, VMError>;
}