use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::transaction::{Transaction, TransactionStatus, TransactionType};
use crate::shard::{ShardId, ShardManager, Shard};
use crate::network::{NetworkMessage, MessageType, PeerInfo};
use crate::crypto::{hash, verify_signature};

/// クロスシャードトランザクション状態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CrossShardTransactionState {
    /// 初期化
    Initialized,
    /// 準備フェーズ
    Prepared,
    /// コミットフェーズ
    Committed,
    /// アボートフェーズ
    Aborted,
    /// 完了
    Completed,
    /// タイムアウト
    TimedOut,
}

/// クロスシャードトランザクション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossShardTransaction {
    /// トランザクションID
    pub id: String,
    /// 元のトランザクション
    pub original_transaction: Transaction,
    /// 送信元シャード
    pub source_shard: ShardId,
    /// 送信先シャード
    pub destination_shard: ShardId,
    /// 経由シャード
    pub intermediate_shards: Vec<ShardId>,
    /// 状態
    pub state: CrossShardTransactionState,
    /// 準備完了シャード
    pub prepared_shards: HashSet<ShardId>,
    /// コミット完了シャード
    pub committed_shards: HashSet<ShardId>,
    /// アボート完了シャード
    pub aborted_shards: HashSet<ShardId>,
    /// 作成時刻
    pub created_at: DateTime<Utc>,
    /// 更新時刻
    pub updated_at: DateTime<Utc>,
    /// タイムアウト時刻
    pub timeout_at: DateTime<Utc>,
    /// エラー
    pub error: Option<String>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

impl CrossShardTransaction {
    /// 新しいクロスシャードトランザクションを作成
    pub fn new(
        transaction: Transaction,
        source_shard: ShardId,
        destination_shard: ShardId,
        intermediate_shards: Vec<ShardId>,
        timeout_seconds: u64,
    ) -> Self {
        let now = Utc::now();
        let id = format!("cst-{}", hash(&format!("{:?}-{}", transaction, now)));
        
        Self {
            id,
            original_transaction: transaction,
            source_shard,
            destination_shard,
            intermediate_shards,
            state: CrossShardTransactionState::Initialized,
            prepared_shards: HashSet::new(),
            committed_shards: HashSet::new(),
            aborted_shards: HashSet::new(),
            created_at: now,
            updated_at: now,
            timeout_at: now + chrono::Duration::seconds(timeout_seconds as i64),
            error: None,
            metadata: HashMap::new(),
        }
    }
    
    /// 全てのシャードのリストを取得
    pub fn all_shards(&self) -> HashSet<ShardId> {
        let mut shards = HashSet::new();
        shards.insert(self.source_shard.clone());
        shards.insert(self.destination_shard.clone());
        for shard in &self.intermediate_shards {
            shards.insert(shard.clone());
        }
        shards
    }
    
    /// 準備フェーズが完了しているかどうかを確認
    pub fn is_prepare_complete(&self) -> bool {
        let all_shards = self.all_shards();
        self.prepared_shards == all_shards
    }
    
    /// コミットフェーズが完了しているかどうかを確認
    pub fn is_commit_complete(&self) -> bool {
        let all_shards = self.all_shards();
        self.committed_shards == all_shards
    }
    
    /// アボートフェーズが完了しているかどうかを確認
    pub fn is_abort_complete(&self) -> bool {
        let all_shards = self.all_shards();
        self.aborted_shards == all_shards
    }
    
    /// タイムアウトしているかどうかを確認
    pub fn is_timed_out(&self) -> bool {
        Utc::now() > self.timeout_at
    }
}

/// クロスシャードトランザクションマネージャー
pub struct CrossShardTransactionManager {
    /// シャードマネージャー
    shard_manager: Arc<ShardManager>,
    /// トランザクション
    transactions: Arc<RwLock<HashMap<String, CrossShardTransaction>>>,
    /// タイムアウト（秒）
    timeout_seconds: u64,
    /// 最大再試行回数
    max_retries: u32,
    /// 再試行間隔（ミリ秒）
    retry_interval_ms: u64,
    /// 最適化ルーティング
    optimize_routing: bool,
    /// バッチ処理
    batch_processing: bool,
    /// 最大バッチサイズ
    max_batch_size: usize,
}

impl CrossShardTransactionManager {
    /// 新しいクロスシャードトランザクションマネージャーを作成
    pub fn new(
        shard_manager: Arc<ShardManager>,
        timeout_seconds: Option<u64>,
        max_retries: Option<u32>,
        retry_interval_ms: Option<u64>,
        optimize_routing: Option<bool>,
        batch_processing: Option<bool>,
        max_batch_size: Option<usize>,
    ) -> Self {
        Self {
            shard_manager,
            transactions: Arc::new(RwLock::new(HashMap::new())),
            timeout_seconds: timeout_seconds.unwrap_or(30),
            max_retries: max_retries.unwrap_or(3),
            retry_interval_ms: retry_interval_ms.unwrap_or(1000),
            optimize_routing: optimize_routing.unwrap_or(true),
            batch_processing: batch_processing.unwrap_or(true),
            max_batch_size: max_batch_size.unwrap_or(100),
        }
    }
    
    /// クロスシャードトランザクションを作成
    pub fn create_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<CrossShardTransaction, Error> {
        // 送信元シャードと送信先シャードを取得
        let source_shard = transaction.shard_id.clone();
        let destination_shard = self.shard_manager.get_shard_for_address(&transaction.to_address)?;
        
        // 同一シャード内のトランザクションの場合はエラー
        if source_shard == destination_shard {
            return Err(Error::InvalidOperation("送信元と送信先が同じシャードです。クロスシャードトランザクションは不要です。".to_string()));
        }
        
        // 経由シャードを計算
        let intermediate_shards = if self.optimize_routing {
            self.calculate_optimal_route(&source_shard, &destination_shard)?
        } else {
            Vec::new()
        };
        
        // クロスシャードトランザクションを作成
        let cross_shard_tx = CrossShardTransaction::new(
            transaction,
            source_shard,
            destination_shard,
            intermediate_shards,
            self.timeout_seconds,
        );
        
        // トランザクションを保存
        let mut transactions = self.transactions.write().unwrap();
        transactions.insert(cross_shard_tx.id.clone(), cross_shard_tx.clone());
        
        Ok(cross_shard_tx)
    }
    
    /// クロスシャードトランザクションを実行
    pub fn execute_transaction(&self, transaction_id: &str) -> Result<CrossShardTransaction, Error> {
        // トランザクションを取得
        let mut transactions = self.transactions.write().unwrap();
        let tx = transactions.get_mut(transaction_id).ok_or_else(|| {
            Error::NotFound(format!("クロスシャードトランザクション {} が見つかりません", transaction_id))
        })?;
        
        // 状態をチェック
        if tx.state != CrossShardTransactionState::Initialized {
            return Err(Error::InvalidState(format!(
                "トランザクションは {} 状態であり、実行できません",
                format!("{:?}", tx.state)
            )));
        }
        
        // 準備フェーズを開始
        self.start_prepare_phase(tx)?;
        
        // トランザクションを返す
        Ok(tx.clone())
    }
    
    /// 準備フェーズを開始
    fn start_prepare_phase(&self, tx: &mut CrossShardTransaction) -> Result<(), Error> {
        // 状態を更新
        tx.state = CrossShardTransactionState::Prepared;
        tx.updated_at = Utc::now();
        
        // 全シャードに準備メッセージを送信
        let all_shards = tx.all_shards();
        for shard_id in &all_shards {
            self.send_prepare_message(tx, shard_id)?;
        }
        
        Ok(())
    }
    
    /// 準備メッセージを送信
    fn send_prepare_message(&self, tx: &CrossShardTransaction, shard_id: &ShardId) -> Result<(), Error> {
        // シャードを取得
        let shard = self.shard_manager.get_shard(shard_id)?;
        
        // 準備メッセージを作成
        let message = NetworkMessage {
            message_type: MessageType::CrossShardPrepare,
            sender: "cross_shard_manager".to_string(),
            receiver: shard_id.clone(),
            data: serde_json::to_string(tx)?,
            timestamp: Utc::now(),
        };
        
        // メッセージを送信
        shard.handle_network_message(message)?;
        
        Ok(())
    }
    
    /// 準備完了通知を処理
    pub fn handle_prepare_ack(
        &self,
        transaction_id: &str,
        shard_id: &ShardId,
        success: bool,
        error: Option<String>,
    ) -> Result<(), Error> {
        // トランザクションを取得
        let mut transactions = self.transactions.write().unwrap();
        let tx = transactions.get_mut(transaction_id).ok_or_else(|| {
            Error::NotFound(format!("クロスシャードトランザクション {} が見つかりません", transaction_id))
        })?;
        
        // 状態をチェック
        if tx.state != CrossShardTransactionState::Prepared {
            return Err(Error::InvalidState(format!(
                "トランザクションは {} 状態であり、準備完了通知を処理できません",
                format!("{:?}", tx.state)
            )));
        }
        
        if success {
            // 準備完了シャードに追加
            tx.prepared_shards.insert(shard_id.clone());
            
            // 全シャードの準備が完了したかチェック
            if tx.is_prepare_complete() {
                // コミットフェーズを開始
                self.start_commit_phase(tx)?;
            }
        } else {
            // エラーを設定
            tx.error = error;
            
            // アボートフェーズを開始
            self.start_abort_phase(tx)?;
        }
        
        // 更新時刻を更新
        tx.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// コミットフェーズを開始
    fn start_commit_phase(&self, tx: &mut CrossShardTransaction) -> Result<(), Error> {
        // 状態を更新
        tx.state = CrossShardTransactionState::Committed;
        tx.updated_at = Utc::now();
        
        // 全シャードにコミットメッセージを送信
        let all_shards = tx.all_shards();
        for shard_id in &all_shards {
            self.send_commit_message(tx, shard_id)?;
        }
        
        Ok(())
    }
    
    /// コミットメッセージを送信
    fn send_commit_message(&self, tx: &CrossShardTransaction, shard_id: &ShardId) -> Result<(), Error> {
        // シャードを取得
        let shard = self.shard_manager.get_shard(shard_id)?;
        
        // コミットメッセージを作成
        let message = NetworkMessage {
            message_type: MessageType::CrossShardCommit,
            sender: "cross_shard_manager".to_string(),
            receiver: shard_id.clone(),
            data: serde_json::to_string(tx)?,
            timestamp: Utc::now(),
        };
        
        // メッセージを送信
        shard.handle_network_message(message)?;
        
        Ok(())
    }
    
    /// コミット完了通知を処理
    pub fn handle_commit_ack(
        &self,
        transaction_id: &str,
        shard_id: &ShardId,
        success: bool,
        error: Option<String>,
    ) -> Result<(), Error> {
        // トランザクションを取得
        let mut transactions = self.transactions.write().unwrap();
        let tx = transactions.get_mut(transaction_id).ok_or_else(|| {
            Error::NotFound(format!("クロスシャードトランザクション {} が見つかりません", transaction_id))
        })?;
        
        // 状態をチェック
        if tx.state != CrossShardTransactionState::Committed {
            return Err(Error::InvalidState(format!(
                "トランザクションは {} 状態であり、コミット完了通知を処理できません",
                format!("{:?}", tx.state)
            )));
        }
        
        if success {
            // コミット完了シャードに追加
            tx.committed_shards.insert(shard_id.clone());
            
            // 全シャードのコミットが完了したかチェック
            if tx.is_commit_complete() {
                // 完了状態に更新
                tx.state = CrossShardTransactionState::Completed;
            }
        } else {
            // エラーを設定
            tx.error = error;
            
            // アボートフェーズを開始
            self.start_abort_phase(tx)?;
        }
        
        // 更新時刻を更新
        tx.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// アボートフェーズを開始
    fn start_abort_phase(&self, tx: &mut CrossShardTransaction) -> Result<(), Error> {
        // 状態を更新
        tx.state = CrossShardTransactionState::Aborted;
        tx.updated_at = Utc::now();
        
        // 準備完了したシャードにアボートメッセージを送信
        for shard_id in &tx.prepared_shards {
            self.send_abort_message(tx, shard_id)?;
        }
        
        Ok(())
    }
    
    /// アボートメッセージを送信
    fn send_abort_message(&self, tx: &CrossShardTransaction, shard_id: &ShardId) -> Result<(), Error> {
        // シャードを取得
        let shard = self.shard_manager.get_shard(shard_id)?;
        
        // アボートメッセージを作成
        let message = NetworkMessage {
            message_type: MessageType::CrossShardAbort,
            sender: "cross_shard_manager".to_string(),
            receiver: shard_id.clone(),
            data: serde_json::to_string(tx)?,
            timestamp: Utc::now(),
        };
        
        // メッセージを送信
        shard.handle_network_message(message)?;
        
        Ok(())
    }
    
    /// アボート完了通知を処理
    pub fn handle_abort_ack(
        &self,
        transaction_id: &str,
        shard_id: &ShardId,
    ) -> Result<(), Error> {
        // トランザクションを取得
        let mut transactions = self.transactions.write().unwrap();
        let tx = transactions.get_mut(transaction_id).ok_or_else(|| {
            Error::NotFound(format!("クロスシャードトランザクション {} が見つかりません", transaction_id))
        })?;
        
        // 状態をチェック
        if tx.state != CrossShardTransactionState::Aborted {
            return Err(Error::InvalidState(format!(
                "トランザクションは {} 状態であり、アボート完了通知を処理できません",
                format!("{:?}", tx.state)
            )));
        }
        
        // アボート完了シャードに追加
        tx.aborted_shards.insert(shard_id.clone());
        
        // 全シャードのアボートが完了したかチェック
        if tx.is_abort_complete() {
            // 完了状態に更新（アボート完了も完了とみなす）
            tx.state = CrossShardTransactionState::Completed;
        }
        
        // 更新時刻を更新
        tx.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// タイムアウトしたトランザクションをチェック
    pub fn check_timeouts(&self) -> Result<Vec<CrossShardTransaction>, Error> {
        let mut timed_out_txs = Vec::new();
        let mut transactions = self.transactions.write().unwrap();
        
        for (_, tx) in transactions.iter_mut() {
            // 完了または既にタイムアウト状態のトランザクションはスキップ
            if tx.state == CrossShardTransactionState::Completed || 
               tx.state == CrossShardTransactionState::TimedOut {
                continue;
            }
            
            // タイムアウトをチェック
            if tx.is_timed_out() {
                // タイムアウト状態に更新
                tx.state = CrossShardTransactionState::TimedOut;
                tx.updated_at = Utc::now();
                tx.error = Some("トランザクションがタイムアウトしました".to_string());
                
                // アボートフェーズを開始
                self.start_abort_phase(tx)?;
                
                timed_out_txs.push(tx.clone());
            }
        }
        
        Ok(timed_out_txs)
    }
    
    /// トランザクションを取得
    pub fn get_transaction(&self, transaction_id: &str) -> Result<CrossShardTransaction, Error> {
        let transactions = self.transactions.read().unwrap();
        let tx = transactions.get(transaction_id).ok_or_else(|| {
            Error::NotFound(format!("クロスシャードトランザクション {} が見つかりません", transaction_id))
        })?;
        
        Ok(tx.clone())
    }
    
    /// 全トランザクションを取得
    pub fn get_all_transactions(&self) -> Vec<CrossShardTransaction> {
        let transactions = self.transactions.read().unwrap();
        transactions.values().cloned().collect()
    }
    
    /// 保留中のトランザクションを取得
    pub fn get_pending_transactions(&self) -> Vec<CrossShardTransaction> {
        let transactions = self.transactions.read().unwrap();
        transactions.values()
            .filter(|tx| {
                tx.state != CrossShardTransactionState::Completed && 
                tx.state != CrossShardTransactionState::TimedOut
            })
            .cloned()
            .collect()
    }
    
    /// 最適なルートを計算
    fn calculate_optimal_route(
        &self,
        source_shard: &ShardId,
        destination_shard: &ShardId,
    ) -> Result<Vec<ShardId>, Error> {
        // 実際の実装では、シャード間の接続性やレイテンシを考慮して最適なルートを計算
        // ここでは簡易的な実装として、直接接続を返す
        
        Ok(Vec::new())
    }
    
    /// バッチ処理を実行
    pub fn process_batch(&self) -> Result<Vec<CrossShardTransaction>, Error> {
        if !self.batch_processing {
            return Ok(Vec::new());
        }
        
        // 保留中のトランザクションを取得
        let pending_txs = self.get_pending_transactions();
        
        // バッチサイズを制限
        let batch_size = std::cmp::min(pending_txs.len(), self.max_batch_size);
        let batch = &pending_txs[0..batch_size];
        
        // バッチ処理を実行
        let mut processed_txs = Vec::new();
        for tx in batch {
            if tx.state == CrossShardTransactionState::Initialized {
                match self.execute_transaction(&tx.id) {
                    Ok(processed_tx) => processed_txs.push(processed_tx),
                    Err(e) => {
                        error!("バッチ処理中にエラーが発生しました: {}", e);
                        // エラーが発生しても処理を続行
                    }
                }
            }
        }
        
        Ok(processed_txs)
    }
}

/// クロスシャードトランザクションハンドラー
pub trait CrossShardTransactionHandler {
    /// 準備フェーズを処理
    fn handle_prepare(&self, transaction: &CrossShardTransaction) -> Result<bool, Error>;
    
    /// コミットフェーズを処理
    fn handle_commit(&self, transaction: &CrossShardTransaction) -> Result<bool, Error>;
    
    /// アボートフェーズを処理
    fn handle_abort(&self, transaction: &CrossShardTransaction) -> Result<(), Error>;
}

impl CrossShardTransactionHandler for Shard {
    /// 準備フェーズを処理
    fn handle_prepare(&self, transaction: &CrossShardTransaction) -> Result<bool, Error> {
        // トランザクションの検証
        self.validate_cross_shard_transaction(transaction)?;
        
        // リソースの予約
        self.reserve_resources(transaction)?;
        
        // 成功を返す
        Ok(true)
    }
    
    /// コミットフェーズを処理
    fn handle_commit(&self, transaction: &CrossShardTransaction) -> Result<bool, Error> {
        // トランザクションの実行
        self.execute_cross_shard_transaction(transaction)?;
        
        // 成功を返す
        Ok(true)
    }
    
    /// アボートフェーズを処理
    fn handle_abort(&self, transaction: &CrossShardTransaction) -> Result<(), Error> {
        // リソースの解放
        self.release_resources(transaction)?;
        
        Ok(())
    }
    
    /// クロスシャードトランザクションを検証
    fn validate_cross_shard_transaction(&self, transaction: &CrossShardTransaction) -> Result<(), Error> {
        // 実際の実装では、トランザクションの署名や残高などを検証
        // ここでは簡易的な実装として、常に成功を返す
        
        Ok(())
    }
    
    /// リソースを予約
    fn reserve_resources(&self, transaction: &CrossShardTransaction) -> Result<(), Error> {
        // 実際の実装では、トランザクションに必要なリソース（残高など）を予約
        // ここでは簡易的な実装として、常に成功を返す
        
        Ok(())
    }
    
    /// リソースを解放
    fn release_resources(&self, transaction: &CrossShardTransaction) -> Result<(), Error> {
        // 実際の実装では、予約したリソースを解放
        // ここでは簡易的な実装として、常に成功を返す
        
        Ok(())
    }
    
    /// クロスシャードトランザクションを実行
    fn execute_cross_shard_transaction(&self, transaction: &CrossShardTransaction) -> Result<(), Error> {
        // 実際の実装では、トランザクションを実行し、状態を更新
        // ここでは簡易的な実装として、常に成功を返す
        
        Ok(())
    }
}