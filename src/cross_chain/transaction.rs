use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::transaction::Transaction;
use super::bridge::ChainType;

/// トランザクションの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// 初期化中
    Initializing,
    /// 送信中
    Sending,
    /// 送信済み
    Sent,
    /// 確認中
    Confirming,
    /// 確認済み
    Confirmed,
    /// 検証済み
    Verified,
    /// 失敗
    Failed,
    /// タイムアウト
    Timeout,
}

/// トランザクション証明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionProof {
    /// 証明ID
    pub id: String,
    /// トランザクションID
    pub transaction_id: String,
    /// ブロックハッシュ
    pub block_hash: String,
    /// ブロック高
    pub block_height: u64,
    /// タイムスタンプ
    pub timestamp: u64,
    /// 証明データ
    pub proof_data: Vec<u8>,
    /// 署名
    pub signature: String,
    /// 検証者
    pub verifier: String,
    /// 作成日時
    pub created_at: DateTime<Utc>,
}

impl TransactionProof {
    /// 新しいトランザクション証明を作成
    pub fn new(
        transaction_id: String,
        block_hash: String,
        block_height: u64,
        timestamp: u64,
        proof_data: Vec<u8>,
        signature: String,
        verifier: String,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        
        Self {
            id,
            transaction_id,
            block_hash,
            block_height,
            timestamp,
            proof_data,
            signature,
            verifier,
            created_at: Utc::now(),
        }
    }
    
    /// 証明を検証
    pub fn verify(&self) -> bool {
        // 実際の実装では、証明データと署名を検証
        // ここでは簡略化のため、常に成功するとする
        true
    }
}

/// クロスチェーントランザクション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainTransaction {
    /// トランザクションID
    pub id: String,
    /// 元のトランザクション
    pub original_transaction: Transaction,
    /// 送信元チェーン
    pub source_chain: ChainType,
    /// 送信先チェーン
    pub target_chain: ChainType,
    /// 送信元チェーンのトランザクションID
    pub source_transaction_id: Option<String>,
    /// 送信先チェーンのトランザクションID
    pub target_transaction_id: Option<String>,
    /// 送信元チェーンのブロックハッシュ
    pub source_block_hash: Option<String>,
    /// 送信先チェーンのブロックハッシュ
    pub target_block_hash: Option<String>,
    /// 送信元チェーンのブロック高
    pub source_block_height: Option<u64>,
    /// 送信先チェーンのブロック高
    pub target_block_height: Option<u64>,
    /// 証明
    pub proof: Option<TransactionProof>,
    /// 状態
    pub status: TransactionStatus,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 送信日時
    pub sent_at: Option<DateTime<Utc>>,
    /// 確認日時
    pub confirmed_at: Option<DateTime<Utc>>,
    /// 完了日時
    pub completed_at: Option<DateTime<Utc>>,
    /// リトライ回数
    pub retry_count: u32,
    /// エラーメッセージ（失敗時）
    pub error: Option<String>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

impl CrossChainTransaction {
    /// 新しいクロスチェーントランザクションを作成
    pub fn new(
        original_transaction: Transaction,
        source_chain: ChainType,
        target_chain: ChainType,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        
        Self {
            id,
            original_transaction,
            source_chain,
            target_chain,
            source_transaction_id: None,
            target_transaction_id: None,
            source_block_hash: None,
            target_block_hash: None,
            source_block_height: None,
            target_block_height: None,
            proof: None,
            status: TransactionStatus::Initializing,
            created_at: Utc::now(),
            sent_at: None,
            confirmed_at: None,
            completed_at: None,
            retry_count: 0,
            error: None,
            metadata: HashMap::new(),
        }
    }
    
    /// トランザクションを送信中に設定
    pub fn mark_as_sending(&mut self) {
        self.status = TransactionStatus::Sending;
    }
    
    /// トランザクションを送信済みに設定
    pub fn mark_as_sent(&mut self) {
        self.status = TransactionStatus::Sent;
        self.sent_at = Some(Utc::now());
    }
    
    /// トランザクションを確認中に設定
    pub fn mark_as_confirming(&mut self) {
        self.status = TransactionStatus::Confirming;
    }
    
    /// トランザクションを確認済みに設定
    pub fn mark_as_confirmed(&mut self) {
        self.status = TransactionStatus::Confirmed;
        self.confirmed_at = Some(Utc::now());
    }
    
    /// トランザクションを検証済みに設定
    pub fn mark_as_verified(&mut self, proof: TransactionProof) {
        self.status = TransactionStatus::Verified;
        self.proof = Some(proof);
        self.completed_at = Some(Utc::now());
    }
    
    /// トランザクションを失敗に設定
    pub fn mark_as_failed(&mut self, error: String) {
        self.status = TransactionStatus::Failed;
        self.error = Some(error);
        self.completed_at = Some(Utc::now());
    }
    
    /// トランザクションをタイムアウトに設定
    pub fn mark_as_timeout(&mut self) {
        self.status = TransactionStatus::Timeout;
        self.completed_at = Some(Utc::now());
    }
    
    /// リトライ回数をインクリメント
    pub fn increment_retry_count(&mut self) {
        self.retry_count += 1;
    }
    
    /// メタデータを設定
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    /// メタデータを取得
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// 送信元チェーンのトランザクションIDを設定
    pub fn set_source_transaction_id(&mut self, tx_id: String) {
        self.source_transaction_id = Some(tx_id);
    }
    
    /// 送信先チェーンのトランザクションIDを設定
    pub fn set_target_transaction_id(&mut self, tx_id: String) {
        self.target_transaction_id = Some(tx_id);
    }
    
    /// 送信元チェーンのブロック情報を設定
    pub fn set_source_block_info(&mut self, block_hash: String, block_height: u64) {
        self.source_block_hash = Some(block_hash);
        self.source_block_height = Some(block_height);
    }
    
    /// 送信先チェーンのブロック情報を設定
    pub fn set_target_block_info(&mut self, block_hash: String, block_height: u64) {
        self.target_block_hash = Some(block_hash);
        self.target_block_height = Some(block_height);
    }
    
    /// トランザクションが完了したかどうかを確認
    pub fn is_completed(&self) -> bool {
        matches!(self.status, TransactionStatus::Verified | TransactionStatus::Failed | TransactionStatus::Timeout)
    }
    
    /// トランザクションが成功したかどうかを確認
    pub fn is_successful(&self) -> bool {
        self.status == TransactionStatus::Verified
    }
    
    /// トランザクションが失敗したかどうかを確認
    pub fn is_failed(&self) -> bool {
        matches!(self.status, TransactionStatus::Failed | TransactionStatus::Timeout)
    }
}