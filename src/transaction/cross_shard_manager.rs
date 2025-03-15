use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::transaction::{Transaction, TransactionStatus};
use crate::shard::{ShardId, ShardManager};
// use crate::network::NetworkMessage;

// Temporary NetworkMessage definition until network module is properly implemented
#[derive(Debug, Clone)]
pub enum NetworkMessage {
    PrepareCrossShardTransaction {
        transaction: crate::transaction::Transaction,
    },
    CommitCrossShardTransaction {
        transaction: crate::transaction::Transaction,
    },
    AbortCrossShardTransaction {
        transaction_id: String,
    },
}

/// クロスシャードトランザクションマネージャー
///
/// 複数のシャードにまたがるトランザクションを管理し、
/// 原子性を保証するための2フェーズコミットプロトコルを実装します。
pub struct CrossShardManager {
    /// シャードマネージャー
    shard_manager: Arc<ShardManager>,
    /// 進行中のクロスシャードトランザクション
    pending_transactions: Arc<Mutex<HashMap<String, CrossShardTransaction>>>,
    /// 完了したクロスシャードトランザクション
    completed_transactions: Arc<Mutex<HashSet<String>>>,
    /// ネットワークメッセージ送信チャネル
    network_tx: mpsc::Sender<NetworkMessage>,
}

/// クロスシャードトランザクションの状態
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrossShardTransactionState {
    /// 準備フェーズ
    Preparing,
    /// コミットフェーズ
    Committing,
    /// 完了
    Completed,
    /// 中止
    Aborted,
}

/// クロスシャードトランザクション
#[derive(Debug, Clone)]
pub struct CrossShardTransaction {
    /// トランザクションID
    pub id: String,
    /// 親トランザクション
    pub parent: Transaction,
    /// 子トランザクション
    pub children: Vec<Transaction>,
    /// 状態
    pub state: CrossShardTransactionState,
    /// 準備完了したシャード
    pub prepared_shards: HashSet<ShardId>,
    /// コミット完了したシャード
    pub committed_shards: HashSet<ShardId>,
    /// 関連するすべてのシャード
    pub involved_shards: HashSet<ShardId>,
    /// 作成時刻
    pub created_at: u64,
    /// 最終更新時刻
    pub updated_at: u64,
}

impl CrossShardManager {
    /// 新しいクロスシャードマネージャーを作成
    pub fn new(
        shard_manager: Arc<ShardManager>,
        network_tx: mpsc::Sender<NetworkMessage>,
    ) -> Self {
        Self {
            shard_manager,
            pending_transactions: Arc::new(Mutex::new(HashMap::new())),
            completed_transactions: Arc::new(Mutex::new(HashSet::new())),
            network_tx,
        }
    }

    /// クロスシャードトランザクションを開始
    pub async fn start_transaction(&self, parent: Transaction) -> Result<String, Error> {
        let now = chrono::Utc::now().timestamp() as u64;
        
        // 子トランザクションを作成
        let children = self.create_child_transactions(&parent).await?;
        
        // 関連するシャードを特定
        let mut involved_shards = HashSet::new();
        involved_shards.insert(parent.shard_id.clone());
        
        for child in &children {
            involved_shards.insert(child.shard_id.clone());
        }
        
        // クロスシャードトランザクションを作成
        let cross_tx = CrossShardTransaction {
            id: parent.id.clone(),
            parent: parent.clone(),
            children,
            state: CrossShardTransactionState::Preparing,
            prepared_shards: HashSet::new(),
            committed_shards: HashSet::new(),
            involved_shards,
            created_at: now,
            updated_at: now,
        };
        
        // 保存
        {
            let mut pending = self.pending_transactions.lock().unwrap();
            pending.insert(cross_tx.id.clone(), cross_tx.clone());
        }
        
        // 準備フェーズを開始
        self.start_prepare_phase(&cross_tx).await?;
        
        Ok(cross_tx.id)
    }

    /// 子トランザクションを作成
    async fn create_child_transactions(&self, parent: &Transaction) -> Result<Vec<Transaction>, Error> {
        // 実際の実装では、トランザクションの内容に基づいて子トランザクションを作成
        // ここでは簡略化のため、ダミーの子トランザクションを作成
        
        let mut children = Vec::new();
        
        // 親シャード以外のすべてのシャードに子トランザクションを作成
        let shards = self.shard_manager.get_all_shards().await?;
        
        for shard in shards {
            if shard.id != parent.shard_id {
                let child = Transaction {
                    id: format!("{}-{}", parent.id, shard.id),
                    from: parent.from.clone(),
                    to: parent.to.clone(),
                    amount: parent.amount.clone(),
                    fee: parent.fee.clone(),
                    data: parent.data.clone(),
                    nonce: parent.nonce,
                    timestamp: parent.timestamp,
                    signature: parent.signature.clone(),
                    status: TransactionStatus::Pending,
                    shard_id: shard.id.clone(),
                    block_hash: None,
                    block_height: None,
                    parent_id: Some(parent.id.clone()),
                    payload: Vec::new(), // Empty payload for child transactions
                    parent_ids: Vec::new(), // No additional parent IDs
                };
                
                children.push(child);
            }
        }
        
        Ok(children)
    }

    /// 準備フェーズを開始
    async fn start_prepare_phase(&self, cross_tx: &CrossShardTransaction) -> Result<(), Error> {
        info!("Starting prepare phase for cross-shard transaction: {}", cross_tx.id);
        
        // 親トランザクションの準備メッセージを送信
        self.send_prepare_message(&cross_tx.parent).await?;
        
        // 子トランザクションの準備メッセージを送信
        for child in &cross_tx.children {
            self.send_prepare_message(child).await?;
        }
        
        // タイムアウト監視を開始
        self.start_timeout_monitor(cross_tx.id.clone());
        
        Ok(())
    }

    /// 準備メッセージを送信
    async fn send_prepare_message(&self, tx: &Transaction) -> Result<(), Error> {
        let message = NetworkMessage::PrepareCrossShardTransaction {
            transaction: tx.clone(),
        };
        
        self.network_tx.send(message).await.map_err(|e| {
            error!("Failed to send prepare message: {}", e);
            Error::NetworkError(format!("Failed to send prepare message: {}", e))
        })?;
        
        Ok(())
    }

    /// タイムアウト監視を開始
    fn start_timeout_monitor(&self, tx_id: String) {
        let pending_transactions = self.pending_transactions.clone();
        let network_tx = self.network_tx.clone();
        
        tokio::spawn(async move {
            // 30秒のタイムアウト
            sleep(Duration::from_secs(30)).await;
            
            let mut pending = pending_transactions.lock().unwrap();
            
            if let Some(tx) = pending.get(&tx_id) {
                if tx.state == CrossShardTransactionState::Preparing {
                    warn!("Cross-shard transaction timed out in prepare phase: {}", tx_id);
                    
                    // トランザクションを中止
                    let mut tx = tx.clone();
                    tx.state = CrossShardTransactionState::Aborted;
                    pending.insert(tx_id.clone(), tx.clone());
                    
                    // 中止メッセージを送信
                    let abort_message = NetworkMessage::AbortCrossShardTransaction {
                        transaction_id: tx_id.clone(),
                    };
                    
                    if let Err(e) = network_tx.try_send(abort_message) {
                        error!("Failed to send abort message: {}", e);
                    }
                }
            }
        });
    }

    /// 準備完了通知を処理
    pub async fn handle_prepare_ack(
        &self,
        tx_id: String,
        shard_id: ShardId,
    ) -> Result<(), Error> {
        let mut pending = self.pending_transactions.lock().unwrap();
        
        if let Some(tx) = pending.get_mut(&tx_id) {
            if tx.state == CrossShardTransactionState::Preparing {
                // 準備完了したシャードを記録
                tx.prepared_shards.insert(shard_id.clone());
                tx.updated_at = chrono::Utc::now().timestamp() as u64;
                
                debug!(
                    "Received prepare ACK for transaction {} from shard {}. Prepared: {}/{}",
                    tx_id,
                    shard_id,
                    tx.prepared_shards.len(),
                    tx.involved_shards.len()
                );
                
                // すべてのシャードが準備完了したらコミットフェーズを開始
                if tx.prepared_shards == tx.involved_shards {
                    info!("All shards prepared for transaction {}. Starting commit phase.", tx_id);
                    
                    tx.state = CrossShardTransactionState::Committing;
                    
                    // コミットフェーズを開始
                    let tx_clone = tx.clone();
                    drop(pending); // ロックを解放
                    
                    self.start_commit_phase(&tx_clone).await?;
                }
            }
        } else {
            warn!("Received prepare ACK for unknown transaction: {}", tx_id);
        }
        
        Ok(())
    }

    /// コミットフェーズを開始
    async fn start_commit_phase(&self, cross_tx: &CrossShardTransaction) -> Result<(), Error> {
        info!("Starting commit phase for cross-shard transaction: {}", cross_tx.id);
        
        // 親トランザクションのコミットメッセージを送信
        self.send_commit_message(&cross_tx.parent).await?;
        
        // 子トランザクションのコミットメッセージを送信
        for child in &cross_tx.children {
            self.send_commit_message(child).await?;
        }
        
        Ok(())
    }

    /// コミットメッセージを送信
    async fn send_commit_message(&self, tx: &Transaction) -> Result<(), Error> {
        let message = NetworkMessage::CommitCrossShardTransaction {
            transaction: tx.clone(),
        };
        
        self.network_tx.send(message).await.map_err(|e| {
            error!("Failed to send commit message: {}", e);
            Error::NetworkError(format!("Failed to send commit message: {}", e))
        })?;
        
        Ok(())
    }

    /// コミット完了通知を処理
    pub async fn handle_commit_ack(
        &self,
        tx_id: String,
        shard_id: ShardId,
    ) -> Result<(), Error> {
        let mut pending = self.pending_transactions.lock().unwrap();
        
        if let Some(tx) = pending.get_mut(&tx_id) {
            if tx.state == CrossShardTransactionState::Committing {
                // コミット完了したシャードを記録
                tx.committed_shards.insert(shard_id);
                tx.updated_at = chrono::Utc::now().timestamp() as u64;
                
                debug!(
                    "Received commit ACK for transaction {} from shard {}. Committed: {}/{}",
                    tx_id,
                    shard_id,
                    tx.committed_shards.len(),
                    tx.involved_shards.len()
                );
                
                // すべてのシャードがコミット完了したらトランザクションを完了
                if tx.committed_shards == tx.involved_shards {
                    info!("All shards committed for transaction {}. Transaction completed.", tx_id);
                    
                    tx.state = CrossShardTransactionState::Completed;
                    
                    // 完了したトランザクションを記録
                    let mut completed = self.completed_transactions.lock().unwrap();
                    completed.insert(tx_id.clone());
                    
                    // 一定時間後に保留中のトランザクションから削除
                    let pending_transactions = self.pending_transactions.clone();
                    let tx_id_clone = tx_id.clone();
                    
                    tokio::spawn(async move {
                        sleep(Duration::from_secs(300)).await; // 5分後
                        
                        let mut pending = pending_transactions.lock().unwrap();
                        pending.remove(&tx_id_clone);
                        
                        debug!("Removed completed cross-shard transaction from pending: {}", tx_id_clone);
                    });
                }
            }
        } else {
            warn!("Received commit ACK for unknown transaction: {}", tx_id);
        }
        
        Ok(())
    }

    /// 中止通知を処理
    pub async fn handle_abort(
        &self,
        tx_id: String,
        shard_id: ShardId,
    ) -> Result<(), Error> {
        let mut pending = self.pending_transactions.lock().unwrap();
        
        if let Some(tx) = pending.get_mut(&tx_id) {
            if tx.state == CrossShardTransactionState::Preparing {
                warn!(
                    "Received abort for transaction {} from shard {}. Aborting transaction.",
                    tx_id,
                    shard_id.clone()
                );
                
                tx.state = CrossShardTransactionState::Aborted;
                tx.updated_at = chrono::Utc::now().timestamp() as u64;
                
                // 中止メッセージを送信
                let abort_message = NetworkMessage::AbortCrossShardTransaction {
                    transaction_id: tx_id.clone(),
                };
                
                let network_tx = self.network_tx.clone();
                drop(pending); // ロックを解放
                
                if let Err(e) = network_tx.send(abort_message).await {
                    error!("Failed to send abort message: {}", e);
                }
            }
        } else {
            warn!("Received abort for unknown transaction: {}", tx_id);
        }
        
        Ok(())
    }

    /// トランザクションの状態を取得
    pub fn get_transaction_state(&self, tx_id: &str) -> Option<CrossShardTransactionState> {
        let pending = self.pending_transactions.lock().unwrap();
        
        if let Some(tx) = pending.get(tx_id) {
            return Some(tx.state.clone());
        }
        
        let completed = self.completed_transactions.lock().unwrap();
        
        if completed.contains(tx_id) {
            return Some(CrossShardTransactionState::Completed);
        }
        
        None
    }

    /// クロスシャードトランザクションを取得
    pub fn get_transaction(&self, tx_id: &str) -> Option<CrossShardTransaction> {
        let pending = self.pending_transactions.lock().unwrap();
        
        if let Some(tx) = pending.get(tx_id) {
            return Some(tx.clone());
        }
        
        None
    }

    /// 保留中のクロスシャードトランザクションをすべて取得
    pub fn get_pending_transactions(&self) -> Vec<CrossShardTransaction> {
        let pending = self.pending_transactions.lock().unwrap();
        
        pending.values().cloned().collect()
    }

    /// 古いトランザクションをクリーンアップ
    pub async fn cleanup_old_transactions(&self) {
        let now = chrono::Utc::now().timestamp() as u64;
        let mut pending = self.pending_transactions.lock().unwrap();
        
        // 1時間以上前のアボートされたトランザクションを削除
        let old_tx_ids: Vec<String> = pending
            .iter()
            .filter(|(_, tx)| {
                tx.state == CrossShardTransactionState::Aborted && now - tx.updated_at > 3600
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in old_tx_ids {
            pending.remove(&id);
            debug!("Removed old aborted cross-shard transaction: {}", id);
        }
        
        // 1日以上前の完了したトランザクションを削除
        let mut completed = self.completed_transactions.lock().unwrap();
        let completed_tx_ids: Vec<String> = completed.iter().cloned().collect();
        
        for id in completed_tx_ids {
            if let Some(tx) = pending.get(&id) {
                if now - tx.updated_at > 86400 {
                    pending.remove(&id);
                    completed.remove(&id);
                    debug!("Removed old completed cross-shard transaction: {}", id);
                }
            }
        }
    }

    /// クリーンアップタスクを開始
    pub fn start_cleanup_task(&self) {
        let self_clone = self.clone();
        
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(3600)).await; // 1時間ごとにクリーンアップ
                self_clone.cleanup_old_transactions().await;
            }
        });
    }
}

impl Clone for CrossShardManager {
    fn clone(&self) -> Self {
        Self {
            shard_manager: self.shard_manager.clone(),
            pending_transactions: self.pending_transactions.clone(),
            completed_transactions: self.completed_transactions.clone(),
            network_tx: self.network_tx.clone(),
        }
    }
}
