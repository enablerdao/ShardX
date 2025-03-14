use crate::transaction::{DAG, Transaction, TransactionStatus};
use async_trait::async_trait;
use log::{debug, info};
use std::sync::Arc;

/// バリデータの役割を定義するトレイト
#[async_trait]
pub trait Validator: Send + Sync {
    /// トランザクションの署名を検証
    async fn verify_signature(&self, signature: &[u8], payload: &[u8]) -> bool;
    
    /// トランザクションを検証
    async fn validate_transaction(&self, tx: &Transaction, dag: &DAG) -> bool;
}

/// Proof of Flow コンセンサスエンジン
pub struct ProofOfFlow {
    /// DAGの参照
    dag: Arc<DAG>,
    /// バリデータのリスト
    validators: Vec<Arc<dyn Validator>>,
}

impl ProofOfFlow {
    /// 新しいProof of Flowエンジンを作成
    pub fn new(dag: Arc<DAG>, validators: Vec<Arc<dyn Validator>>) -> Self {
        Self { dag, validators }
    }
    
    /// トランザクションを検証してDAGに追加
    pub async fn process_transaction(&self, tx: Transaction) -> Result<(), String> {
        // タイムスタンプの順序を確認
        for parent_id in &tx.parent_ids {
            if let Some(parent) = self.dag.get_transaction(parent_id) {
                if parent.timestamp >= tx.timestamp {
                    return Err("Invalid timestamp order".to_string());
                }
            } else {
                return Err(format!("Parent transaction {} not found", parent_id));
            }
        }
        
        // バリデータによる検証
        let mut valid_votes = 0;
        for validator in &self.validators {
            if validator.validate_transaction(&tx, &self.dag).await {
                valid_votes += 1;
            }
        }
        
        // 過半数のバリデータが承認した場合
        let required_votes = (self.validators.len() / 2) + 1;
        if valid_votes >= required_votes {
            // トランザクションをDAGに追加
            self.dag.add_transaction(tx.clone())?;
            
            // トランザクションを確認済みに更新
            self.dag.update_transaction_status(&tx.id, TransactionStatus::Confirmed)?;
            
            info!("Transaction {} confirmed with {}/{} votes", tx.id, valid_votes, self.validators.len());
            Ok(())
        } else {
            debug!("Transaction {} rejected with {}/{} votes", tx.id, valid_votes, self.validators.len());
            Err(format!("Not enough validator votes: {}/{}", valid_votes, required_votes))
        }
    }
    
    /// 現在のTPS（1秒あたりのトランザクション数）を計算
    pub fn calculate_tps(&self, window_seconds: u64) -> f64 {
        // 実際の実装では、過去window_seconds間に確認されたトランザクション数をカウント
        // 簡略化のため、現在は固定値を返す
        let confirmed_count = self.dag.confirmed_count() as f64;
        confirmed_count / window_seconds as f64
    }
}

/// シンプルなバリデータの実装
pub struct SimpleValidator {
    /// バリデータのID
    pub id: String,
}

#[async_trait]
impl Validator for SimpleValidator {
    async fn verify_signature(&self, _signature: &[u8], _payload: &[u8]) -> bool {
        // 簡略化のため、常にtrueを返す
        // 実際の実装では、暗号署名の検証を行う
        true
    }
    
    async fn validate_transaction(&self, tx: &Transaction, dag: &DAG) -> bool {
        // 親トランザクションが存在し、タイムスタンプの順序が正しいことを確認
        for parent_id in &tx.parent_ids {
            if let Some(parent) = dag.get_transaction(parent_id) {
                if parent.timestamp >= tx.timestamp {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        // 署名を検証
        self.verify_signature(&tx.signature, &tx.payload).await
    }
}