use crate::error::Error;
use crate::sharding::{ShardManager, ShardId, NodeId};
use crate::transaction::Transaction;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::mpsc;
use uuid::Uuid;

/// クロスシャードトランザクションの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossShardTxStatus {
    /// 初期化中
    Initializing,
    /// 準備完了
    Prepared,
    /// コミット中
    Committing,
    /// コミット完了
    Committed,
    /// アボート中
    Aborting,
    /// アボート完了
    Aborted,
}

/// クロスシャードトランザクション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossShardTransaction {
    /// トランザクションID
    pub id: String,
    /// 元のトランザクション
    pub original_transaction: Transaction,
    /// コーディネーターシャードID
    pub coordinator_shard: ShardId,
    /// 参加シャードID
    pub participant_shards: Vec<ShardId>,
    /// 各シャードの準備状態
    pub prepared_shards: HashMap<ShardId, bool>,
    /// 各シャードのコミット状態
    pub committed_shards: HashMap<ShardId, bool>,
    /// 状態
    pub status: CrossShardTxStatus,
    /// 作成日時
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 完了日時
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl CrossShardTransaction {
    /// 新しいクロスシャードトランザクションを作成
    pub fn new(
        original_transaction: Transaction,
        coordinator_shard: ShardId,
        participant_shards: Vec<ShardId>,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        
        // 準備状態とコミット状態を初期化
        let mut prepared_shards = HashMap::new();
        let mut committed_shards = HashMap::new();
        
        for &shard in &participant_shards {
            prepared_shards.insert(shard, false);
            committed_shards.insert(shard, false);
        }
        
        Self {
            id,
            original_transaction,
            coordinator_shard,
            participant_shards,
            prepared_shards,
            committed_shards,
            status: CrossShardTxStatus::Initializing,
            created_at: now,
            completed_at: None,
        }
    }
    
    /// すべてのシャードが準備完了かどうかを確認
    pub fn all_prepared(&self) -> bool {
        self.prepared_shards.values().all(|&prepared| prepared)
    }
    
    /// すべてのシャードがコミット完了かどうかを確認
    pub fn all_committed(&self) -> bool {
        self.committed_shards.values().all(|&committed| committed)
    }
    
    /// シャードの準備状態を更新
    pub fn set_prepared(&mut self, shard_id: ShardId, prepared: bool) -> Result<(), Error> {
        if !self.participant_shards.contains(&shard_id) {
            return Err(Error::InvalidShardId(shard_id));
        }
        
        if let Some(state) = self.prepared_shards.get_mut(&shard_id) {
            *state = prepared;
            
            // すべてのシャードが準備完了なら状態を更新
            if self.all_prepared() {
                self.status = CrossShardTxStatus::Prepared;
            }
            
            Ok(())
        } else {
            Err(Error::InvalidShardId(shard_id))
        }
    }
    
    /// シャードのコミット状態を更新
    pub fn set_committed(&mut self, shard_id: ShardId, committed: bool) -> Result<(), Error> {
        if !self.participant_shards.contains(&shard_id) {
            return Err(Error::InvalidShardId(shard_id));
        }
        
        if let Some(state) = self.committed_shards.get_mut(&shard_id) {
            *state = committed;
            
            // すべてのシャードがコミット完了なら状態を更新
            if self.all_committed() {
                self.status = CrossShardTxStatus::Committed;
                self.completed_at = Some(chrono::Utc::now());
            }
            
            Ok(())
        } else {
            Err(Error::InvalidShardId(shard_id))
        }
    }
    
    /// トランザクションをコミットフェーズに移行
    pub fn start_commit(&mut self) -> Result<(), Error> {
        if self.status != CrossShardTxStatus::Prepared {
            return Err(Error::InvalidOperation("Transaction is not prepared".to_string()));
        }
        
        self.status = CrossShardTxStatus::Committing;
        Ok(())
    }
    
    /// トランザクションをアボートフェーズに移行
    pub fn start_abort(&mut self) -> Result<(), Error> {
        if self.status != CrossShardTxStatus::Initializing && self.status != CrossShardTxStatus::Prepared {
            return Err(Error::InvalidOperation("Transaction cannot be aborted in current state".to_string()));
        }
        
        self.status = CrossShardTxStatus::Aborting;
        Ok(())
    }
    
    /// トランザクションをアボート完了に設定
    pub fn set_aborted(&mut self) {
        self.status = CrossShardTxStatus::Aborted;
        self.completed_at = Some(chrono::Utc::now());
    }
}

/// クロスシャードメッセージタイプ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossShardMessageType {
    /// 準備リクエスト
    PrepareRequest,
    /// 準備レスポンス
    PrepareResponse { success: bool },
    /// コミットリクエスト
    CommitRequest,
    /// コミットレスポンス
    CommitResponse { success: bool },
    /// アボートリクエスト
    AbortRequest,
    /// アボートレスポンス
    AbortResponse { success: bool },
}

/// クロスシャードメッセージ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossShardMessage {
    /// メッセージID
    pub id: String,
    /// トランザクションID
    pub transaction_id: String,
    /// 送信元シャードID
    pub from_shard: ShardId,
    /// 送信先シャードID
    pub to_shard: ShardId,
    /// メッセージタイプ
    pub message_type: CrossShardMessageType,
    /// トランザクションデータ（オプション）
    pub transaction_data: Option<Vec<u8>>,
    /// 作成日時
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl CrossShardMessage {
    /// 新しいクロスシャードメッセージを作成
    pub fn new(
        transaction_id: String,
        from_shard: ShardId,
        to_shard: ShardId,
        message_type: CrossShardMessageType,
        transaction_data: Option<Vec<u8>>,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        
        Self {
            id,
            transaction_id,
            from_shard,
            to_shard,
            message_type,
            transaction_data,
            created_at: chrono::Utc::now(),
        }
    }
}

/// クロスシャードコーディネーター
pub struct CrossShardCoordinator {
    /// 現在のシャードID
    current_shard: ShardId,
    /// シャードマネージャーの参照
    shard_manager: Arc<ShardManager>,
    /// 進行中のトランザクション
    transactions: RwLock<HashMap<String, CrossShardTransaction>>,
    /// メッセージ送信チャネル
    message_sender: mpsc::Sender<CrossShardMessage>,
    /// メッセージ受信チャネル
    message_receiver: Mutex<Option<mpsc::Receiver<CrossShardMessage>>>,
}

impl CrossShardCoordinator {
    /// 新しいCrossShardCoordinatorを作成
    pub fn new(
        current_shard: ShardId,
        shard_manager: Arc<ShardManager>,
        message_sender: mpsc::Sender<CrossShardMessage>,
        message_receiver: mpsc::Receiver<CrossShardMessage>,
    ) -> Self {
        Self {
            current_shard,
            shard_manager,
            transactions: RwLock::new(HashMap::new()),
            message_sender,
            message_receiver: Mutex::new(Some(message_receiver)),
        }
    }
    
    /// クロスシャードトランザクションを開始
    pub async fn start_transaction(&self, transaction: Transaction) -> Result<String, Error> {
        // トランザクションが影響するシャードを特定
        let affected_shards = self.identify_affected_shards(&transaction)?;
        
        // 単一シャードの場合は通常のトランザクションとして処理
        if affected_shards.len() <= 1 {
            return Err(Error::InvalidOperation("Transaction affects only one shard".to_string()));
        }
        
        // クロスシャードトランザクションを作成
        let cross_tx = CrossShardTransaction::new(
            transaction,
            self.current_shard,
            affected_shards.clone(),
        );
        
        let tx_id = cross_tx.id.clone();
        
        // トランザクションを保存
        {
            let mut transactions = self.transactions.write().unwrap();
            transactions.insert(tx_id.clone(), cross_tx);
        }
        
        // 各参加シャードに準備リクエストを送信
        for &shard_id in &affected_shards {
            if shard_id == self.current_shard {
                // 自分自身のシャードは直接準備
                self.prepare_local_transaction(&tx_id).await?;
            } else {
                // 他のシャードには準備リクエストを送信
                let message = CrossShardMessage::new(
                    tx_id.clone(),
                    self.current_shard,
                    shard_id,
                    CrossShardMessageType::PrepareRequest,
                    Some(serde_json::to_vec(&transaction).map_err(|e| Error::SerializationError(e.to_string()))?),
                );
                
                self.message_sender.send(message).await
                    .map_err(|e| Error::InternalError(format!("Failed to send message: {}", e)))?;
            }
        }
        
        info!("Started cross-shard transaction: {}", tx_id);
        Ok(tx_id)
    }
    
    /// トランザクションが影響するシャードを特定
    fn identify_affected_shards(&self, transaction: &Transaction) -> Result<Vec<ShardId>, Error> {
        // 実際の実装では、トランザクションの内容に基づいて影響するシャードを特定
        
        let mut affected_shards = HashSet::with_capacity(5); // 通常は少数のシャードに影響
        
        // 現在のシャードを追加
        affected_shards.insert(self.current_shard);
        
        // ペイロードが空の場合は現在のシャードのみ
        if transaction.payload.is_empty() {
            return Ok(vec![self.current_shard]);
        }
        
        // ペイロードを解析してシャードを決定
        // 最適化: ペイロードの先頭部分のみを解析
        let payload_prefix = if transaction.payload.len() > 16 {
            &transaction.payload[0..16]
        } else {
            &transaction.payload
        };
        
        // ペイロードのハッシュに基づいてシャードを決定
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash_slice(payload_prefix, &mut hasher);
        let hash = std::hash::Hasher::finish(&hasher);
        
        // ハッシュ値に基づいて追加のシャードを決定
        let shard_count = self.shard_manager.shard_count;
        let additional_shard = (hash % shard_count as u64) as u32;
        
        if additional_shard != self.current_shard {
            affected_shards.insert(additional_shard);
        }
        
        // 親トランザクションのシャードも影響を受ける
        // 最適化: 親トランザクションのキャッシュを使用
        for parent_id in &transaction.parent_ids {
            // 実際の実装では、親トランザクションのシャードを取得
            // キャッシュを使用して高速化
            if let Some(parent_shard) = self.get_transaction_shard(parent_id) {
                if parent_shard != self.current_shard {
                    affected_shards.insert(parent_shard);
                }
            } else {
                // キャッシュにない場合はハッシュを使用
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                std::hash::Hash::hash(parent_id, &mut hasher);
                let hash = std::hash::Hasher::finish(&hasher);
                let parent_shard = (hash % shard_count as u64) as u32;
                
                if parent_shard != self.current_shard {
                    affected_shards.insert(parent_shard);
                }
            }
        }
        
        // 最適化: 影響するシャード数が多すぎる場合は制限
        const MAX_AFFECTED_SHARDS: usize = 5;
        let mut result: Vec<ShardId> = affected_shards.into_iter().collect();
        
        if result.len() > MAX_AFFECTED_SHARDS {
            // 現在のシャードは必ず含める
            result.retain(|&shard| shard == self.current_shard);
            
            // 残りのシャードからランダムに選択
            let mut rng = rand::thread_rng();
            while result.len() < MAX_AFFECTED_SHARDS && !affected_shards.is_empty() {
                let index = rng.gen_range(0..affected_shards.len());
                let shard = affected_shards.iter().nth(index).cloned().unwrap();
                if shard != self.current_shard {
                    result.push(shard);
                    affected_shards.remove(&shard);
                }
            }
        }
        
        Ok(result)
    }
    
    /// トランザクションのシャードをキャッシュから取得
    fn get_transaction_shard(&self, tx_id: &str) -> Option<ShardId> {
        // 実際の実装では、トランザクションのシャードをキャッシュから取得
        // ここでは簡略化のため、トランザクションIDのハッシュに基づいて決定
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(tx_id, &mut hasher);
        let hash = std::hash::Hasher::finish(&hasher);
        let shard_count = self.shard_manager.shard_count;
        
        Some((hash % shard_count as u64) as u32)
    }
    
    /// トランザクションの検証
    fn validate_transaction(&self, transaction: &Transaction) -> Result<(), Error> {
        // トランザクションIDの検証
        if transaction.id.is_empty() {
            return Err(Error::ValidationError("Empty transaction ID".to_string()));
        }
        
        // タイムスタンプの検証
        let current_time = chrono::Utc::now().timestamp();
        let max_future_time = current_time + 60; // 最大1分先の未来まで許容
        
        if transaction.timestamp > max_future_time {
            return Err(Error::ValidationError(format!(
                "Transaction timestamp too far in the future: {} > {}",
                transaction.timestamp, max_future_time
            )));
        }
        
        // 古すぎるトランザクションの拒否
        let min_time = current_time - 3600; // 1時間前まで許容
        if transaction.timestamp < min_time {
            return Err(Error::ValidationError(format!(
                "Transaction too old: {} < {}",
                transaction.timestamp, min_time
            )));
        }
        
        // ペイロードサイズの検証
        const MAX_PAYLOAD_SIZE: usize = 1024 * 1024; // 1MB
        if transaction.payload.len() > MAX_PAYLOAD_SIZE {
            return Err(Error::ValidationError(format!(
                "Payload too large: {} > {} bytes",
                transaction.payload.len(), MAX_PAYLOAD_SIZE
            )));
        }
        
        // 署名の検証
        if transaction.signature.is_empty() {
            return Err(Error::ValidationError("Empty signature".to_string()));
        }
        
        // TODO: 実際の署名検証ロジックを実装
        // let is_valid = crypto::verify_transaction_signature(transaction);
        // if !is_valid {
        //     return Err(Error::ValidationError("Invalid signature".to_string()));
        // }
        
        // 親トランザクションの検証
        for parent_id in &transaction.parent_ids {
            if parent_id.is_empty() {
                return Err(Error::ValidationError("Empty parent transaction ID".to_string()));
            }
            
            // TODO: 親トランザクションの存在確認
            // if !self.transaction_exists(parent_id) {
            //     return Err(Error::ValidationError(format!("Parent transaction not found: {}", parent_id)));
            // }
        }
        
        Ok(())
    }
    
    /// 重複トランザクションのチェック
    fn is_duplicate_transaction(&self, tx_id: &str) -> bool {
        let transactions = self.transactions.read().unwrap();
        transactions.contains_key(tx_id)
    }
    
    /// ローカルトランザクションを準備
    async fn prepare_local_transaction(&self, tx_id: &str) -> Result<(), Error> {
        // トランザクションを取得
        let transaction = {
            let transactions = self.transactions.read().unwrap();
            transactions.get(tx_id).cloned().ok_or_else(|| Error::TransactionNotFound(tx_id.to_string()))?
        };
        
        // 実際の実装では、トランザクションの検証や準備処理を行う
        // ここでは簡略化のため、常に成功するとする
        
        // 準備完了を設定
        {
            let mut transactions = self.transactions.write().unwrap();
            if let Some(tx) = transactions.get_mut(tx_id) {
                tx.set_prepared(self.current_shard, true)?;
            }
        }
        
        // コーディネーターの場合、すべてのシャードが準備完了したかチェック
        if transaction.coordinator_shard == self.current_shard {
            self.check_all_prepared(tx_id).await?;
        } else {
            // 参加者の場合、コーディネーターに準備完了を通知
            let message = CrossShardMessage::new(
                tx_id.to_string(),
                self.current_shard,
                transaction.coordinator_shard,
                CrossShardMessageType::PrepareResponse { success: true },
                None,
            );
            
            self.message_sender.send(message).await
                .map_err(|e| Error::InternalError(format!("Failed to send message: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// すべてのシャードが準備完了したかチェック
    async fn check_all_prepared(&self, tx_id: &str) -> Result<(), Error> {
        let all_prepared = {
            let transactions = self.transactions.read().unwrap();
            let tx = transactions.get(tx_id).ok_or_else(|| Error::TransactionNotFound(tx_id.to_string()))?;
            tx.all_prepared()
        };
        
        if all_prepared {
            // すべてのシャードが準備完了したらコミットフェーズを開始
            self.start_commit_phase(tx_id).await?;
        }
        
        Ok(())
    }
    
    /// コミットフェーズを開始
    async fn start_commit_phase(&self, tx_id: &str) -> Result<(), Error> {
        // トランザクションの状態を更新
        {
            let mut transactions = self.transactions.write().unwrap();
            let tx = transactions.get_mut(tx_id).ok_or_else(|| Error::TransactionNotFound(tx_id.to_string()))?;
            tx.start_commit()?;
        }
        
        // トランザクションを取得
        let transaction = {
            let transactions = self.transactions.read().unwrap();
            transactions.get(tx_id).cloned().ok_or_else(|| Error::TransactionNotFound(tx_id.to_string()))?
        };
        
        // 各参加シャードにコミットリクエストを送信
        for &shard_id in &transaction.participant_shards {
            if shard_id == self.current_shard {
                // 自分自身のシャードは直接コミット
                self.commit_local_transaction(tx_id).await?;
            } else {
                // 他のシャードにはコミットリクエストを送信
                let message = CrossShardMessage::new(
                    tx_id.to_string(),
                    self.current_shard,
                    shard_id,
                    CrossShardMessageType::CommitRequest,
                    None,
                );
                
                self.message_sender.send(message).await
                    .map_err(|e| Error::InternalError(format!("Failed to send message: {}", e)))?;
            }
        }
        
        info!("Started commit phase for transaction: {}", tx_id);
        Ok(())
    }
    
    /// ローカルトランザクションをコミット
    async fn commit_local_transaction(&self, tx_id: &str) -> Result<(), Error> {
        // トランザクションを取得
        let transaction = {
            let transactions = self.transactions.read().unwrap();
            transactions.get(tx_id).cloned().ok_or_else(|| Error::TransactionNotFound(tx_id.to_string()))?
        };
        
        // 実際の実装では、トランザクションのコミット処理を行う
        // ここでは簡略化のため、常に成功するとする
        
        // コミット完了を設定
        {
            let mut transactions = self.transactions.write().unwrap();
            if let Some(tx) = transactions.get_mut(tx_id) {
                tx.set_committed(self.current_shard, true)?;
            }
        }
        
        // コーディネーターの場合、すべてのシャードがコミット完了したかチェック
        if transaction.coordinator_shard == self.current_shard {
            self.check_all_committed(tx_id).await?;
        } else {
            // 参加者の場合、コーディネーターにコミット完了を通知
            let message = CrossShardMessage::new(
                tx_id.to_string(),
                self.current_shard,
                transaction.coordinator_shard,
                CrossShardMessageType::CommitResponse { success: true },
                None,
            );
            
            self.message_sender.send(message).await
                .map_err(|e| Error::InternalError(format!("Failed to send message: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// すべてのシャードがコミット完了したかチェック
    async fn check_all_committed(&self, tx_id: &str) -> Result<(), Error> {
        let all_committed = {
            let transactions = self.transactions.read().unwrap();
            let tx = transactions.get(tx_id).ok_or_else(|| Error::TransactionNotFound(tx_id.to_string()))?;
            tx.all_committed()
        };
        
        if all_committed {
            info!("Transaction {} successfully committed on all shards", tx_id);
            
            // 実際の実装では、完了したトランザクションの後処理を行う
            // ここでは簡略化のため、特に何もしない
        }
        
        Ok(())
    }
    
    /// アボートフェーズを開始
    async fn start_abort_phase(&self, tx_id: &str) -> Result<(), Error> {
        // トランザクションの状態を更新
        {
            let mut transactions = self.transactions.write().unwrap();
            let tx = transactions.get_mut(tx_id).ok_or_else(|| Error::TransactionNotFound(tx_id.to_string()))?;
            tx.start_abort()?;
        }
        
        // トランザクションを取得
        let transaction = {
            let transactions = self.transactions.read().unwrap();
            transactions.get(tx_id).cloned().ok_or_else(|| Error::TransactionNotFound(tx_id.to_string()))?
        };
        
        // 各参加シャードにアボートリクエストを送信
        for &shard_id in &transaction.participant_shards {
            if shard_id == self.current_shard {
                // 自分自身のシャードは直接アボート
                self.abort_local_transaction(tx_id).await?;
            } else {
                // 他のシャードにはアボートリクエストを送信
                let message = CrossShardMessage::new(
                    tx_id.to_string(),
                    self.current_shard,
                    shard_id,
                    CrossShardMessageType::AbortRequest,
                    None,
                );
                
                self.message_sender.send(message).await
                    .map_err(|e| Error::InternalError(format!("Failed to send message: {}", e)))?;
            }
        }
        
        info!("Started abort phase for transaction: {}", tx_id);
        Ok(())
    }
    
    /// ローカルトランザクションをアボート
    async fn abort_local_transaction(&self, tx_id: &str) -> Result<(), Error> {
        // トランザクションを取得
        let transaction = {
            let transactions = self.transactions.read().unwrap();
            transactions.get(tx_id).cloned().ok_or_else(|| Error::TransactionNotFound(tx_id.to_string()))?
        };
        
        // 実際の実装では、トランザクションのアボート処理を行う
        // ここでは簡略化のため、特に何もしない
        
        // コーディネーターの場合、トランザクションをアボート完了に設定
        if transaction.coordinator_shard == self.current_shard {
            let mut transactions = self.transactions.write().unwrap();
            if let Some(tx) = transactions.get_mut(tx_id) {
                tx.set_aborted();
            }
            
            info!("Transaction {} aborted", tx_id);
        } else {
            // 参加者の場合、コーディネーターにアボート完了を通知
            let message = CrossShardMessage::new(
                tx_id.to_string(),
                self.current_shard,
                transaction.coordinator_shard,
                CrossShardMessageType::AbortResponse { success: true },
                None,
            );
            
            self.message_sender.send(message).await
                .map_err(|e| Error::InternalError(format!("Failed to send message: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// メッセージを処理
    pub async fn process_message(&self, message: CrossShardMessage) -> Result<(), Error> {
        // メッセージの検証
        self.validate_message(&message)?;
        
        // メッセージ処理のレート制限
        self.check_message_rate_limit(&message)?;
        
        // メッセージ処理のタイムアウト設定
        let timeout = tokio::time::timeout(
            std::time::Duration::from_secs(5), // 5秒のタイムアウト
            self.process_message_internal(message)
        ).await;
        
        match timeout {
            Ok(result) => result,
            Err(_) => {
                warn!("Message processing timed out");
                Err(Error::Timeout("Message processing timed out".to_string()))
            }
        }
    }
    
    /// メッセージの検証
    fn validate_message(&self, message: &CrossShardMessage) -> Result<(), Error> {
        // トランザクションIDの検証
        if message.transaction_id.is_empty() {
            return Err(Error::ValidationError("Empty transaction ID".to_string()));
        }
        
        // シャードIDの検証
        if message.from_shard >= self.shard_manager.shard_count {
            return Err(Error::ValidationError(format!(
                "Invalid from_shard: {} (max: {})",
                message.from_shard,
                self.shard_manager.shard_count - 1
            )));
        }
        
        if message.to_shard >= self.shard_manager.shard_count {
            return Err(Error::ValidationError(format!(
                "Invalid to_shard: {} (max: {})",
                message.to_shard,
                self.shard_manager.shard_count - 1
            )));
        }
        
        // 宛先シャードの検証
        if message.to_shard != self.current_shard {
            return Err(Error::ValidationError(format!(
                "Message sent to wrong shard: expected {}, got {}",
                self.current_shard,
                message.to_shard
            )));
        }
        
        // メッセージタイプに応じた検証
        match message.message_type {
            CrossShardMessageType::PrepareRequest => {
                // 準備リクエストにはトランザクションデータが必要
                if message.transaction_data.is_none() {
                    return Err(Error::ValidationError("Missing transaction data in prepare request".to_string()));
                }
            },
            _ => {
                // 他のメッセージタイプの検証
            }
        }
        
        Ok(())
    }
    
    /// メッセージ処理のレート制限
    fn check_message_rate_limit(&self, message: &CrossShardMessage) -> Result<(), Error> {
        // 実際の実装では、シャードごとのメッセージレートを制限
        // 簡略化のため、ここではダミー実装
        Ok(())
    }
    
    /// メッセージ内部処理
    async fn process_message_internal(&self, message: CrossShardMessage) -> Result<(), Error> {
        match message.message_type {
            CrossShardMessageType::PrepareRequest => {
                // 準備リクエストを処理
                if let Some(tx_data) = message.transaction_data {
                    // トランザクションデータをデシリアライズ
                    let transaction: Transaction = serde_json::from_slice(&tx_data)
                        .map_err(|e| Error::DeserializationError(e.to_string()))?;
                    
                    // トランザクションの検証
                    self.validate_transaction(&transaction)?;
                    
                    // 重複トランザクションのチェック
                    if self.is_duplicate_transaction(&message.transaction_id) {
                        warn!("Duplicate transaction detected: {}", message.transaction_id);
                        return Err(Error::DuplicateTransaction(message.transaction_id));
                    }
                    
                    // クロスシャードトランザクションを作成
                    let affected_shards = self.identify_affected_shards(&transaction)?;
                    let cross_tx = CrossShardTransaction::new(
                        transaction,
                        message.from_shard,
                        affected_shards,
                    );
                    
                    // トランザクションを保存
                    {
                        let mut transactions = self.transactions.write().unwrap();
                        transactions.insert(message.transaction_id.clone(), cross_tx);
                    }
                    
                    // ローカルトランザクションを準備
                    self.prepare_local_transaction(&message.transaction_id).await?;
                    
                    // 処理成功のログ
                    info!("Prepared transaction: {}", message.transaction_id);
                }
            },
            CrossShardMessageType::PrepareResponse { success } => {
                // 準備レスポンスを処理
                if success {
                    // 準備成功を記録
                    {
                        let mut transactions = self.transactions.write().unwrap();
                        if let Some(tx) = transactions.get_mut(&message.transaction_id) {
                            tx.set_prepared(message.from_shard, true)?;
                        }
                    }
                    
                    // すべてのシャードが準備完了したかチェック
                    self.check_all_prepared(&message.transaction_id).await?;
                } else {
                    // 準備失敗の場合はアボート
                    self.start_abort_phase(&message.transaction_id).await?;
                }
            },
            CrossShardMessageType::CommitRequest => {
                // コミットリクエストを処理
                self.commit_local_transaction(&message.transaction_id).await?;
            },
            CrossShardMessageType::CommitResponse { success } => {
                // コミットレスポンスを処理
                if success {
                    // コミット成功を記録
                    {
                        let mut transactions = self.transactions.write().unwrap();
                        if let Some(tx) = transactions.get_mut(&message.transaction_id) {
                            tx.set_committed(message.from_shard, true)?;
                        }
                    }
                    
                    // すべてのシャードがコミット完了したかチェック
                    self.check_all_committed(&message.transaction_id).await?;
                } else {
                    // コミット失敗の場合はログに記録
                    warn!("Commit failed on shard {} for transaction {}", message.from_shard, message.transaction_id);
                }
            },
            CrossShardMessageType::AbortRequest => {
                // アボートリクエストを処理
                self.abort_local_transaction(&message.transaction_id).await?;
            },
            CrossShardMessageType::AbortResponse { success: _ } => {
                // アボートレスポンスを処理
                // 特に何もしない
            },
        }
        
        Ok(())
    }
    
    /// メッセージ処理ループを開始
    pub async fn start_message_processing(&self) -> Result<(), Error> {
        let mut receiver = self.message_receiver.lock().unwrap().take()
            .ok_or_else(|| Error::InternalError("Message receiver already taken".to_string()))?;
        
        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                debug!("Received cross-shard message: {:?}", message);
                
                if let Err(e) = self.process_message(message).await {
                    warn!("Error processing cross-shard message: {}", e);
                }
            }
        });
        
        Ok(())
    }
    
    /// トランザクションの状態を取得
    pub fn get_transaction_status(&self, tx_id: &str) -> Result<CrossShardTxStatus, Error> {
        let transactions = self.transactions.read().unwrap();
        let tx = transactions.get(tx_id).ok_or_else(|| Error::TransactionNotFound(tx_id.to_string()))?;
        
        Ok(tx.status)
    }
    
    /// トランザクションの詳細を取得
    pub fn get_transaction_details(&self, tx_id: &str) -> Result<CrossShardTransaction, Error> {
        let transactions = self.transactions.read().unwrap();
        let tx = transactions.get(tx_id).ok_or_else(|| Error::TransactionNotFound(tx_id.to_string()))?;
        
        Ok(tx.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::TransactionStatus;
    
    fn create_test_transaction() -> Transaction {
        Transaction {
            id: Uuid::new_v4().to_string(),
            parent_ids: vec![],
            timestamp: 12345,
            payload: vec![1, 2, 3],
            signature: vec![4, 5, 6],
            status: TransactionStatus::Pending,
            created_at: chrono::Utc::now(),
        }
    }
    
    #[tokio::test]
    async fn test_cross_shard_transaction_lifecycle() {
        // テスト用のチャネルを作成
        let (tx1, rx1) = mpsc::channel(100);
        let (tx2, rx2) = mpsc::channel(100);
        
        // シャードマネージャーのモックを作成
        let shard_manager = Arc::new(ShardManager::new(10));
        
        // コーディネーターを作成
        let coordinator = Arc::new(CrossShardCoordinator::new(
            0,
            shard_manager.clone(),
            tx1,
            rx2,
        ));
        
        // 参加者を作成
        let participant = Arc::new(CrossShardCoordinator::new(
            1,
            shard_manager.clone(),
            tx2,
            rx1,
        ));
        
        // メッセージ処理ループを開始
        coordinator.start_message_processing().await.unwrap();
        participant.start_message_processing().await.unwrap();
        
        // テスト用のトランザクションを作成
        let transaction = create_test_transaction();
        
        // クロスシャードトランザクションを開始
        let tx_id = coordinator.start_transaction(transaction).await.unwrap();
        
        // 少し待機してメッセージが処理されるのを待つ
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // トランザクションの状態を確認
        let status = coordinator.get_transaction_status(&tx_id).unwrap();
        assert_eq!(status, CrossShardTxStatus::Committed);
        
        // トランザクションの詳細を確認
        let details = coordinator.get_transaction_details(&tx_id).unwrap();
        assert!(details.all_prepared());
        assert!(details.all_committed());
        assert!(details.completed_at.is_some());
    }
}