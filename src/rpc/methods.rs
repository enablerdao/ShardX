//! RPCメソッド
//! 
//! このモジュールはShardXのRPCメソッドを実装します。

use crate::error::Error;
use crate::transaction::Transaction;
use crate::node::Node;
use std::sync::Arc;

/// RPCメソッド
pub struct RPCMethods {
    /// ノード
    node: Arc<Node>,
}

impl RPCMethods {
    /// 新しいRPCMethodsを作成
    pub fn new(node: Arc<Node>) -> Self {
        Self { node }
    }
    
    /// トランザクションを送信
    pub async fn submit_transaction(&self, tx_data: String) -> Result<String, Error> {
        // トランザクションをパース
        let tx: Transaction = serde_json::from_str(&tx_data)
            .map_err(|e| Error::ValidationError(format!("Invalid transaction data: {}", e)))?;
        
        // トランザクションを送信
        self.node.submit_transaction(tx).await
            .map_err(|e| Error::ValidationError(e))?;
        
        Ok("Transaction submitted successfully".to_string())
    }
    
    /// ノードの状態を取得
    pub fn get_node_status(&self) -> Result<String, Error> {
        let status = format!("{:?}", self.node.get_status());
        Ok(status)
    }
    
    /// 現在のTPSを取得
    pub fn get_tps(&self) -> Result<f64, Error> {
        Ok(self.node.get_tps())
    }
    
    /// 現在のシャード数を取得
    pub fn get_shard_count(&self) -> Result<u32, Error> {
        Ok(self.node.get_shard_count())
    }
}
