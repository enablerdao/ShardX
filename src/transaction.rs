use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use uuid::Uuid;

/// トランザクションの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Rejected,
}

/// トランザクション構造体
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    /// トランザクションID
    pub id: String,
    /// 過去トランザクションへの参照
    pub parent_ids: Vec<String>,
    /// PoHタイムスタンプ
    pub timestamp: u64,
    /// データ（送金情報など）
    pub payload: Vec<u8>,
    /// 署名
    pub signature: Vec<u8>,
    /// 現在の状態
    pub status: TransactionStatus,
    /// 作成日時
    pub created_at: DateTime<Utc>,
}

impl Transaction {
    /// 新しいトランザクションを作成
    pub fn new(parent_ids: Vec<String>, payload: Vec<u8>, signature: Vec<u8>) -> Self {
        let id = Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().timestamp_millis() as u64;
        
        Self {
            id,
            parent_ids,
            timestamp,
            payload,
            signature,
            status: TransactionStatus::Pending,
            created_at: Utc::now(),
        }
    }
    
    /// トランザクションのハッシュを計算
    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.id);
        for parent_id in &self.parent_ids {
            hasher.update(parent_id);
        }
        hasher.update(self.timestamp.to_be_bytes());
        hasher.update(&self.payload);
        hasher.update(&self.signature);
        
        hex::encode(hasher.finalize())
    }
}

/// DAG (Directed Acyclic Graph) 構造体
#[derive(Debug)]
pub struct DAG {
    /// トランザクションのマップ
    pub transactions: dashmap::DashMap<String, Transaction>,
    /// 各トランザクションの子トランザクション
    pub children: dashmap::DashMap<String, HashSet<String>>,
}

impl DAG {
    /// 新しいDAGを作成
    pub fn new() -> Self {
        Self {
            transactions: dashmap::DashMap::new(),
            children: dashmap::DashMap::new(),
        }
    }
    
    /// トランザクションをDAGに追加
    pub fn add_transaction(&self, tx: Transaction) -> Result<(), String> {
        // 親トランザクションが存在するか確認
        for parent_id in &tx.parent_ids {
            if !self.transactions.contains_key(parent_id) {
                return Err(format!("Parent transaction {} not found", parent_id));
            }
        }
        
        // 親IDのリストを保存
        let parent_ids = tx.parent_ids.clone();
        
        // トランザクションを追加
        let tx_id = tx.id.clone();
        self.transactions.insert(tx_id.clone(), tx);
        
        // 親トランザクションの子として登録
        for parent_id in &parent_ids {
            self.children
                .entry(parent_id.clone())
                .or_insert_with(HashSet::new)
                .insert(tx_id.clone());
        }
        
        Ok(())
    }
    
    /// トランザクションの状態を更新
    pub fn update_transaction_status(&self, tx_id: &str, status: TransactionStatus) -> Result<(), String> {
        if let Some(mut tx) = self.transactions.get_mut(tx_id) {
            tx.status = status;
            Ok(())
        } else {
            Err(format!("Transaction {} not found", tx_id))
        }
    }
    
    /// トランザクションを取得
    pub fn get_transaction(&self, tx_id: &str) -> Option<Transaction> {
        self.transactions.get(tx_id).map(|tx| tx.clone())
    }
    
    /// 確認済みトランザクションの数を取得
    pub fn confirmed_count(&self) -> usize {
        self.transactions
            .iter()
            .filter(|tx| tx.status == TransactionStatus::Confirmed)
            .count()
    }
}