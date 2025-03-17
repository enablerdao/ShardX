use chrono::{DateTime, Duration, Utc};
use log::{debug, error, info, warn};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};

use crate::consensus::ConsensusEngine;
use crate::error::Error;
use crate::metrics::MetricsCollector;
use crate::network::{MessageType, NetworkMessage, PeerInfo};
use crate::shard::{ShardId, ShardInfo, ShardManager};
use crate::storage::Storage;
use crate::transaction::{Transaction, TransactionStatus, TransactionType};

/// クロスシャードトランザクション
#[derive(Debug, Clone)]
pub struct CrossShardTransaction {
    /// トランザクションID
    pub id: String,
    /// 元のトランザクション
    pub transaction: Transaction,
    /// 送信元シャードID
    pub source_shard_id: ShardId,
    /// 送信先シャードID
    pub target_shard_id: ShardId,
    /// 作成時刻
    pub created_at: DateTime<Utc>,
    /// 更新時刻
    pub updated_at: DateTime<Utc>,
    /// ステータス
    pub status: CrossShardTransactionStatus,
    /// 完了時刻
    pub completed_at: Option<DateTime<Utc>>,
    /// 確認数
    pub confirmations: u32,
    /// 必要な確認数
    pub required_confirmations: u32,
    /// 再試行回数
    pub retry_count: u32,
    /// 最大再試行回数
    pub max_retries: u32,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// クロスシャードトランザクションステータス
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrossShardTransactionStatus {
    /// 初期化
    Initialized,
    /// 送信元シャードでコミット済み
    SourceCommitted,
    /// 送信先シャードに送信済み
    Transmitted,
    /// 送信先シャードで受信済み
    TargetReceived,
    /// 送信先シャードでコミット済み
    TargetCommitted,
    /// 送信元シャードで確認済み
    SourceAcknowledged,
    /// 完了
    Completed,
    /// 失敗
    Failed,
    /// タイムアウト
    TimedOut,
    /// キャンセル
    Cancelled,
}

/// クロスシャードメッセージ
#[derive(Debug, Clone)]
pub enum CrossShardMessage {
    /// トランザクション送信
    TransactionTransmit {
        /// トランザクションID
        transaction_id: String,
        /// トランザクション
        transaction: Transaction,
        /// 送信元シャードID
        source_shard_id: ShardId,
        /// 送信先シャードID
        target_shard_id: ShardId,
    },
    /// トランザクション受信確認
    TransactionReceived {
        /// トランザクションID
        transaction_id: String,
        /// 送信元シャードID
        source_shard_id: ShardId,
        /// 送信先シャードID
        target_shard_id: ShardId,
    },
    /// トランザクションコミット
    TransactionCommit {
        /// トランザクションID
        transaction_id: String,
        /// 送信元シャードID
        source_shard_id: ShardId,
        /// 送信先シャードID
        target_shard_id: ShardId,
        /// ステータス
        status: TransactionStatus,
    },
    /// トランザクション確認
    TransactionAcknowledge {
        /// トランザクションID
        transaction_id: String,
        /// 送信元シャードID
        source_shard_id: ShardId,
        /// 送信先シャードID
        target_shard_id: ShardId,
    },
    /// トランザクション状態照会
    TransactionStatusQuery {
        /// トランザクションID
        transaction_id: String,
        /// 送信元シャードID
        source_shard_id: ShardId,
        /// 送信先シャードID
        target_shard_id: ShardId,
    },
    /// トランザクション状態応答
    TransactionStatusResponse {
        /// トランザクションID
        transaction_id: String,
        /// 送信元シャードID
        source_shard_id: ShardId,
        /// 送信先シャードID
        target_shard_id: ShardId,
        /// ステータス
        status: CrossShardTransactionStatus,
    },
}

/// クロスシャードマネージャー
pub struct CrossShardManager {
    /// 現在のシャードID
    current_shard_id: ShardId,
    /// シャードマネージャー
    shard_manager: Arc<ShardManager>,
    /// コンセンサスエンジン
    consensus_engine: Arc<dyn ConsensusEngine>,
    /// ストレージ
    storage: Arc<dyn Storage>,
    /// メトリクスコレクター
    metrics: Arc<MetricsCollector>,
    /// 保留中のトランザクション
    pending_transactions: HashMap<String, CrossShardTransaction>,
    /// 完了したトランザクション
    completed_transactions: HashMap<String, CrossShardTransaction>,
    /// 送信キュー
    transmit_queue: VecDeque<CrossShardTransaction>,
    /// 確認キュー
    acknowledgement_queue: VecDeque<CrossShardTransaction>,
    /// タイムアウト（秒）
    timeout_seconds: u64,
    /// 再試行間隔（秒）
    retry_interval_seconds: u64,
    /// 最大再試行回数
    max_retries: u32,
    /// 必要な確認数
    required_confirmations: u32,
    /// 最後のクリーンアップ時刻
    last_cleanup_time: DateTime<Utc>,
    /// クリーンアップ間隔（秒）
    cleanup_interval_seconds: u64,
}

impl CrossShardManager {
    /// 新しいクロスシャードマネージャーを作成
    pub fn new(
        current_shard_id: ShardId,
        shard_manager: Arc<ShardManager>,
        consensus_engine: Arc<dyn ConsensusEngine>,
        storage: Arc<dyn Storage>,
        metrics: Arc<MetricsCollector>,
    ) -> Self {
        Self {
            current_shard_id,
            shard_manager,
            consensus_engine,
            storage,
            metrics,
            pending_transactions: HashMap::new(),
            completed_transactions: HashMap::new(),
            transmit_queue: VecDeque::new(),
            acknowledgement_queue: VecDeque::new(),
            timeout_seconds: 300,       // 5分
            retry_interval_seconds: 30, // 30秒
            max_retries: 5,
            required_confirmations: 2,
            last_cleanup_time: Utc::now(),
            cleanup_interval_seconds: 3600, // 1時間
        }
    }

    /// クロスシャードトランザクションを作成
    pub fn create_transaction(
        &mut self,
        transaction: Transaction,
        target_shard_id: ShardId,
    ) -> Result<String, Error> {
        // 送信元シャードが現在のシャードか確認
        if transaction.shard_id != self.current_shard_id {
            return Err(Error::InvalidInput(format!(
                "Transaction shard ID ({}) does not match current shard ID ({})",
                transaction.shard_id, self.current_shard_id
            )));
        }

        // 送信先シャードが存在するか確認
        if !self.shard_manager.shard_exists(&target_shard_id) {
            return Err(Error::InvalidInput(format!(
                "Target shard not found: {}",
                target_shard_id
            )));
        }

        // トランザクションIDを生成
        let cross_shard_tx_id = format!("cstx_{}", Utc::now().timestamp_nanos());

        // クロスシャードトランザクションを作成
        let cross_shard_tx = CrossShardTransaction {
            id: cross_shard_tx_id.clone(),
            transaction: transaction.clone(),
            source_shard_id: self.current_shard_id.clone(),
            target_shard_id: target_shard_id.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: CrossShardTransactionStatus::Initialized,
            completed_at: None,
            confirmations: 0,
            required_confirmations: self.required_confirmations,
            retry_count: 0,
            max_retries: self.max_retries,
            metadata: HashMap::new(),
        };

        // 保留中のトランザクションに追加
        self.pending_transactions
            .insert(cross_shard_tx_id.clone(), cross_shard_tx.clone());

        // 送信キューに追加
        self.transmit_queue.push_back(cross_shard_tx);

        // メトリクスを更新
        self.metrics
            .increment_counter("cross_shard_transactions_created");

        Ok(cross_shard_tx_id)
    }

    /// トランザクションを送信
    pub fn transmit_transaction(&mut self, transaction_id: &str) -> Result<(), Error> {
        // トランザクションを取得
        let mut tx = self
            .pending_transactions
            .get_mut(transaction_id)
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "Cross-shard transaction not found: {}",
                    transaction_id
                ))
            })?;

        // ステータスをチェック
        if tx.status != CrossShardTransactionStatus::Initialized
            && tx.status != CrossShardTransactionStatus::SourceCommitted
        {
            return Err(Error::InvalidState(format!(
                "Transaction is not in a transmittable state: {:?}",
                tx.status
            )));
        }

        // 送信元シャードでコミット
        if tx.status == CrossShardTransactionStatus::Initialized {
            // トランザクションをコミット
            self.consensus_engine
                .submit_transaction(tx.transaction.clone())?;

            // ステータスを更新
            tx.status = CrossShardTransactionStatus::SourceCommitted;
            tx.updated_at = Utc::now();

            // メトリクスを更新
            self.metrics
                .increment_counter("cross_shard_transactions_source_committed");
        }

        // 送信先シャードにメッセージを送信
        let message = CrossShardMessage::TransactionTransmit {
            transaction_id: transaction_id.to_string(),
            transaction: tx.transaction.clone(),
            source_shard_id: tx.source_shard_id.clone(),
            target_shard_id: tx.target_shard_id.clone(),
        };

        // メッセージを送信
        self.send_message_to_shard(&tx.target_shard_id, message)?;

        // ステータスを更新
        tx.status = CrossShardTransactionStatus::Transmitted;
        tx.updated_at = Utc::now();

        // メトリクスを更新
        self.metrics
            .increment_counter("cross_shard_transactions_transmitted");

        Ok(())
    }

    /// トランザクションを受信
    pub fn receive_transaction(
        &mut self,
        transaction_id: &str,
        transaction: Transaction,
        source_shard_id: ShardId,
    ) -> Result<(), Error> {
        // 送信先シャードが現在のシャードか確認
        if transaction.shard_id != self.current_shard_id {
            return Err(Error::InvalidInput(format!(
                "Transaction shard ID ({}) does not match current shard ID ({})",
                transaction.shard_id, self.current_shard_id
            )));
        }

        // トランザクションが既に存在するか確認
        if self.pending_transactions.contains_key(transaction_id) {
            // 既に受信済みの場合は何もしない
            return Ok(());
        }

        // クロスシャードトランザクションを作成
        let cross_shard_tx = CrossShardTransaction {
            id: transaction_id.to_string(),
            transaction: transaction.clone(),
            source_shard_id: source_shard_id.clone(),
            target_shard_id: self.current_shard_id.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: CrossShardTransactionStatus::TargetReceived,
            completed_at: None,
            confirmations: 0,
            required_confirmations: self.required_confirmations,
            retry_count: 0,
            max_retries: self.max_retries,
            metadata: HashMap::new(),
        };

        // 保留中のトランザクションに追加
        self.pending_transactions
            .insert(transaction_id.to_string(), cross_shard_tx);

        // 受信確認メッセージを送信
        let message = CrossShardMessage::TransactionReceived {
            transaction_id: transaction_id.to_string(),
            source_shard_id: source_shard_id.clone(),
            target_shard_id: self.current_shard_id.clone(),
        };

        // メッセージを送信
        self.send_message_to_shard(&source_shard_id, message)?;

        // トランザクションをコミット
        self.consensus_engine
            .submit_transaction(transaction.clone())?;

        // メトリクスを更新
        self.metrics
            .increment_counter("cross_shard_transactions_received");

        Ok(())
    }

    /// トランザクション受信確認を処理
    pub fn handle_transaction_received(
        &mut self,
        transaction_id: &str,
        source_shard_id: &ShardId,
        target_shard_id: &ShardId,
    ) -> Result<(), Error> {
        // トランザクションを取得
        let mut tx = self
            .pending_transactions
            .get_mut(transaction_id)
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "Cross-shard transaction not found: {}",
                    transaction_id
                ))
            })?;

        // ステータスをチェック
        if tx.status != CrossShardTransactionStatus::Transmitted {
            // 既に受信確認済みの場合は何もしない
            return Ok(());
        }

        // 送信元と送信先をチェック
        if tx.source_shard_id != *source_shard_id || tx.target_shard_id != *target_shard_id {
            return Err(Error::InvalidInput(format!(
                "Shard IDs do not match: expected source={}, target={}, got source={}, target={}",
                tx.source_shard_id, tx.target_shard_id, source_shard_id, target_shard_id
            )));
        }

        // ステータスを更新
        tx.status = CrossShardTransactionStatus::TargetReceived;
        tx.updated_at = Utc::now();

        // メトリクスを更新
        self.metrics
            .increment_counter("cross_shard_transactions_target_received");

        Ok(())
    }

    /// トランザクションコミットを処理
    pub fn handle_transaction_commit(
        &mut self,
        transaction_id: &str,
        source_shard_id: &ShardId,
        target_shard_id: &ShardId,
        status: &TransactionStatus,
    ) -> Result<(), Error> {
        // トランザクションを取得
        let mut tx = self
            .pending_transactions
            .get_mut(transaction_id)
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "Cross-shard transaction not found: {}",
                    transaction_id
                ))
            })?;

        // 送信元と送信先をチェック
        if tx.source_shard_id != *source_shard_id || tx.target_shard_id != *target_shard_id {
            return Err(Error::InvalidInput(format!(
                "Shard IDs do not match: expected source={}, target={}, got source={}, target={}",
                tx.source_shard_id, tx.target_shard_id, source_shard_id, target_shard_id
            )));
        }

        // 現在のシャードが送信元か送信先か確認
        if self.current_shard_id == *source_shard_id {
            // 送信元シャードの場合

            // ステータスをチェック
            if tx.status != CrossShardTransactionStatus::TargetReceived {
                // 既にコミット済みの場合は何もしない
                return Ok(());
            }

            // ステータスを更新
            tx.status = CrossShardTransactionStatus::TargetCommitted;
            tx.updated_at = Utc::now();

            // 確認キューに追加
            self.acknowledgement_queue.push_back(tx.clone());

            // メトリクスを更新
            self.metrics
                .increment_counter("cross_shard_transactions_target_committed");
        } else if self.current_shard_id == *target_shard_id {
            // 送信先シャードの場合

            // ステータスをチェック
            if tx.status != CrossShardTransactionStatus::TargetReceived {
                // 既にコミット済みの場合は何もしない
                return Ok(());
            }

            // トランザクションステータスを更新
            tx.transaction.status = status.clone();

            // ステータスを更新
            tx.status = CrossShardTransactionStatus::TargetCommitted;
            tx.updated_at = Utc::now();

            // コミット確認メッセージを送信
            let message = CrossShardMessage::TransactionCommit {
                transaction_id: transaction_id.to_string(),
                source_shard_id: source_shard_id.clone(),
                target_shard_id: target_shard_id.clone(),
                status: status.clone(),
            };

            // メッセージを送信
            self.send_message_to_shard(source_shard_id, message)?;

            // メトリクスを更新
            self.metrics
                .increment_counter("cross_shard_transactions_target_committed");
        } else {
            return Err(Error::InvalidState(format!(
                "Current shard ({}) is neither source nor target",
                self.current_shard_id
            )));
        }

        Ok(())
    }

    /// トランザクション確認を処理
    pub fn handle_transaction_acknowledge(
        &mut self,
        transaction_id: &str,
        source_shard_id: &ShardId,
        target_shard_id: &ShardId,
    ) -> Result<(), Error> {
        // トランザクションを取得
        let mut tx = self
            .pending_transactions
            .get_mut(transaction_id)
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "Cross-shard transaction not found: {}",
                    transaction_id
                ))
            })?;

        // 送信元と送信先をチェック
        if tx.source_shard_id != *source_shard_id || tx.target_shard_id != *target_shard_id {
            return Err(Error::InvalidInput(format!(
                "Shard IDs do not match: expected source={}, target={}, got source={}, target={}",
                tx.source_shard_id, tx.target_shard_id, source_shard_id, target_shard_id
            )));
        }

        // 現在のシャードが送信元か送信先か確認
        if self.current_shard_id == *source_shard_id {
            // 送信元シャードの場合

            // ステータスをチェック
            if tx.status != CrossShardTransactionStatus::TargetCommitted {
                // 既に確認済みの場合は何もしない
                return Ok(());
            }

            // ステータスを更新
            tx.status = CrossShardTransactionStatus::SourceAcknowledged;
            tx.updated_at = Utc::now();

            // 確認数を増やす
            tx.confirmations += 1;

            // 必要な確認数に達したか確認
            if tx.confirmations >= tx.required_confirmations {
                // ステータスを更新
                tx.status = CrossShardTransactionStatus::Completed;
                tx.completed_at = Some(Utc::now());

                // 完了したトランザクションに移動
                let completed_tx = self.pending_transactions.remove(transaction_id).unwrap();
                self.completed_transactions
                    .insert(transaction_id.to_string(), completed_tx);

                // メトリクスを更新
                self.metrics
                    .increment_counter("cross_shard_transactions_completed");
            }

            // メトリクスを更新
            self.metrics
                .increment_counter("cross_shard_transactions_source_acknowledged");
        } else if self.current_shard_id == *target_shard_id {
            // 送信先シャードの場合

            // ステータスをチェック
            if tx.status != CrossShardTransactionStatus::TargetCommitted {
                // 既に確認済みの場合は何もしない
                return Ok(());
            }

            // 確認数を増やす
            tx.confirmations += 1;

            // 必要な確認数に達したか確認
            if tx.confirmations >= tx.required_confirmations {
                // ステータスを更新
                tx.status = CrossShardTransactionStatus::Completed;
                tx.completed_at = Some(Utc::now());

                // 完了したトランザクションに移動
                let completed_tx = self.pending_transactions.remove(transaction_id).unwrap();
                self.completed_transactions
                    .insert(transaction_id.to_string(), completed_tx);

                // メトリクスを更新
                self.metrics
                    .increment_counter("cross_shard_transactions_completed");
            }
        } else {
            return Err(Error::InvalidState(format!(
                "Current shard ({}) is neither source nor target",
                self.current_shard_id
            )));
        }

        Ok(())
    }

    /// トランザクション状態照会を処理
    pub fn handle_transaction_status_query(
        &self,
        transaction_id: &str,
        source_shard_id: &ShardId,
        target_shard_id: &ShardId,
    ) -> Result<(), Error> {
        // トランザクションを取得
        let status = if let Some(tx) = self.pending_transactions.get(transaction_id) {
            tx.status.clone()
        } else if let Some(tx) = self.completed_transactions.get(transaction_id) {
            tx.status.clone()
        } else {
            return Err(Error::NotFound(format!(
                "Cross-shard transaction not found: {}",
                transaction_id
            )));
        };

        // 状態応答メッセージを送信
        let message = CrossShardMessage::TransactionStatusResponse {
            transaction_id: transaction_id.to_string(),
            source_shard_id: source_shard_id.clone(),
            target_shard_id: target_shard_id.clone(),
            status,
        };

        // メッセージを送信（照会元に応答）
        if self.current_shard_id == *source_shard_id {
            self.send_message_to_shard(target_shard_id, message)?;
        } else {
            self.send_message_to_shard(source_shard_id, message)?;
        }

        Ok(())
    }

    /// トランザクション状態応答を処理
    pub fn handle_transaction_status_response(
        &mut self,
        transaction_id: &str,
        source_shard_id: &ShardId,
        target_shard_id: &ShardId,
        status: &CrossShardTransactionStatus,
    ) -> Result<(), Error> {
        // トランザクションを取得
        let tx = self
            .pending_transactions
            .get_mut(transaction_id)
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "Cross-shard transaction not found: {}",
                    transaction_id
                ))
            })?;

        // 送信元と送信先をチェック
        if tx.source_shard_id != *source_shard_id || tx.target_shard_id != *target_shard_id {
            return Err(Error::InvalidInput(format!(
                "Shard IDs do not match: expected source={}, target={}, got source={}, target={}",
                tx.source_shard_id, tx.target_shard_id, source_shard_id, target_shard_id
            )));
        }

        // ステータスを更新（必要に応じて）
        match status {
            CrossShardTransactionStatus::Completed => {
                // 完了状態の場合は、ローカルのステータスも完了に更新
                tx.status = CrossShardTransactionStatus::Completed;
                tx.completed_at = Some(Utc::now());

                // 完了したトランザクションに移動
                let completed_tx = self.pending_transactions.remove(transaction_id).unwrap();
                self.completed_transactions
                    .insert(transaction_id.to_string(), completed_tx);

                // メトリクスを更新
                self.metrics
                    .increment_counter("cross_shard_transactions_completed");
            }
            CrossShardTransactionStatus::Failed => {
                // 失敗状態の場合は、ローカルのステータスも失敗に更新
                tx.status = CrossShardTransactionStatus::Failed;
                tx.completed_at = Some(Utc::now());

                // 完了したトランザクションに移動
                let completed_tx = self.pending_transactions.remove(transaction_id).unwrap();
                self.completed_transactions
                    .insert(transaction_id.to_string(), completed_tx);

                // メトリクスを更新
                self.metrics
                    .increment_counter("cross_shard_transactions_failed");
            }
            _ => {
                // その他のステータスの場合は、必要に応じて更新
                if status_priority(status) > status_priority(&tx.status) {
                    tx.status = status.clone();
                    tx.updated_at = Utc::now();
                }
            }
        }

        Ok(())
    }

    /// メッセージを処理
    pub fn handle_message(&mut self, message: CrossShardMessage) -> Result<(), Error> {
        match message {
            CrossShardMessage::TransactionTransmit {
                transaction_id,
                transaction,
                source_shard_id,
                target_shard_id,
            } => {
                self.receive_transaction(&transaction_id, transaction, source_shard_id)?;
            }
            CrossShardMessage::TransactionReceived {
                transaction_id,
                source_shard_id,
                target_shard_id,
            } => {
                self.handle_transaction_received(
                    &transaction_id,
                    &source_shard_id,
                    &target_shard_id,
                )?;
            }
            CrossShardMessage::TransactionCommit {
                transaction_id,
                source_shard_id,
                target_shard_id,
                status,
            } => {
                self.handle_transaction_commit(
                    &transaction_id,
                    &source_shard_id,
                    &target_shard_id,
                    &status,
                )?;
            }
            CrossShardMessage::TransactionAcknowledge {
                transaction_id,
                source_shard_id,
                target_shard_id,
            } => {
                self.handle_transaction_acknowledge(
                    &transaction_id,
                    &source_shard_id,
                    &target_shard_id,
                )?;
            }
            CrossShardMessage::TransactionStatusQuery {
                transaction_id,
                source_shard_id,
                target_shard_id,
            } => {
                self.handle_transaction_status_query(
                    &transaction_id,
                    &source_shard_id,
                    &target_shard_id,
                )?;
            }
            CrossShardMessage::TransactionStatusResponse {
                transaction_id,
                source_shard_id,
                target_shard_id,
                status,
            } => {
                self.handle_transaction_status_response(
                    &transaction_id,
                    &source_shard_id,
                    &target_shard_id,
                    &status,
                )?;
            }
        }

        Ok(())
    }

    /// 定期的な処理を実行
    pub fn process(&mut self) -> Result<(), Error> {
        // 送信キューを処理
        self.process_transmit_queue()?;

        // 確認キューを処理
        self.process_acknowledgement_queue()?;

        // タイムアウトを処理
        self.process_timeouts()?;

        // 再試行を処理
        self.process_retries()?;

        // クリーンアップを処理
        self.process_cleanup()?;

        Ok(())
    }

    /// 送信キューを処理
    fn process_transmit_queue(&mut self) -> Result<(), Error> {
        let mut processed = 0;
        let max_process = 10; // 一度に処理する最大数

        while processed < max_process && !self.transmit_queue.is_empty() {
            if let Some(tx) = self.transmit_queue.pop_front() {
                match self.transmit_transaction(&tx.id) {
                    Ok(_) => {
                        processed += 1;
                    }
                    Err(e) => {
                        // エラーが発生した場合は、キューの最後に戻す
                        self.transmit_queue.push_back(tx);
                        error!("Failed to transmit transaction: {}", e);
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// 確認キューを処理
    fn process_acknowledgement_queue(&mut self) -> Result<(), Error> {
        let mut processed = 0;
        let max_process = 10; // 一度に処理する最大数

        while processed < max_process && !self.acknowledgement_queue.is_empty() {
            if let Some(tx) = self.acknowledgement_queue.pop_front() {
                // 確認メッセージを送信
                let message = CrossShardMessage::TransactionAcknowledge {
                    transaction_id: tx.id.clone(),
                    source_shard_id: tx.source_shard_id.clone(),
                    target_shard_id: tx.target_shard_id.clone(),
                };

                match self.send_message_to_shard(&tx.target_shard_id, message) {
                    Ok(_) => {
                        processed += 1;
                    }
                    Err(e) => {
                        // エラーが発生した場合は、キューの最後に戻す
                        self.acknowledgement_queue.push_back(tx);
                        error!("Failed to send acknowledgement: {}", e);
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// タイムアウトを処理
    fn process_timeouts(&mut self) -> Result<(), Error> {
        let now = Utc::now();
        let timeout_duration = Duration::seconds(self.timeout_seconds as i64);

        let mut timed_out_ids = Vec::new();

        // タイムアウトしたトランザクションを検索
        for (id, tx) in &mut self.pending_transactions {
            if tx.status != CrossShardTransactionStatus::Completed
                && tx.status != CrossShardTransactionStatus::Failed
                && tx.status != CrossShardTransactionStatus::TimedOut
                && tx.status != CrossShardTransactionStatus::Cancelled
            {
                let elapsed = now - tx.updated_at;

                if elapsed > timeout_duration {
                    // タイムアウト
                    tx.status = CrossShardTransactionStatus::TimedOut;
                    tx.completed_at = Some(now);

                    timed_out_ids.push(id.clone());

                    // メトリクスを更新
                    self.metrics
                        .increment_counter("cross_shard_transactions_timed_out");
                }
            }
        }

        // タイムアウトしたトランザクションを完了リストに移動
        for id in timed_out_ids {
            if let Some(tx) = self.pending_transactions.remove(&id) {
                self.completed_transactions.insert(id, tx);
            }
        }

        Ok(())
    }

    /// 再試行を処理
    fn process_retries(&mut self) -> Result<(), Error> {
        let now = Utc::now();
        let retry_duration = Duration::seconds(self.retry_interval_seconds as i64);

        let mut retry_ids = Vec::new();

        // 再試行が必要なトランザクションを検索
        for (id, tx) in &mut self.pending_transactions {
            if tx.status != CrossShardTransactionStatus::Completed
                && tx.status != CrossShardTransactionStatus::Failed
                && tx.status != CrossShardTransactionStatus::TimedOut
                && tx.status != CrossShardTransactionStatus::Cancelled
            {
                let elapsed = now - tx.updated_at;

                if elapsed > retry_duration && tx.retry_count < tx.max_retries {
                    retry_ids.push(id.clone());
                }
            }
        }

        // 再試行
        for id in retry_ids {
            if let Some(tx) = self.pending_transactions.get_mut(&id) {
                // 再試行回数を増やす
                tx.retry_count += 1;

                // 現在のステータスに応じて再試行
                match tx.status {
                    CrossShardTransactionStatus::Initialized => {
                        // 送信キューに追加
                        self.transmit_queue.push_back(tx.clone());
                    }
                    CrossShardTransactionStatus::SourceCommitted
                    | CrossShardTransactionStatus::Transmitted => {
                        // 送信キューに追加
                        self.transmit_queue.push_back(tx.clone());
                    }
                    CrossShardTransactionStatus::TargetReceived => {
                        // 送信先シャードの場合は、コミットを再試行
                        if self.current_shard_id == tx.target_shard_id {
                            self.consensus_engine
                                .submit_transaction(tx.transaction.clone())?;
                        }
                    }
                    CrossShardTransactionStatus::TargetCommitted => {
                        // 確認キューに追加
                        self.acknowledgement_queue.push_back(tx.clone());
                    }
                    _ => {
                        // その他のステータスでは何もしない
                    }
                }

                // メトリクスを更新
                self.metrics
                    .increment_counter("cross_shard_transactions_retried");
            }
        }

        Ok(())
    }

    /// クリーンアップを処理
    fn process_cleanup(&mut self) -> Result<(), Error> {
        let now = Utc::now();
        let cleanup_duration = Duration::seconds(self.cleanup_interval_seconds as i64);

        // クリーンアップ間隔をチェック
        if now - self.last_cleanup_time < cleanup_duration {
            return Ok(());
        }

        // 古いトランザクションを削除
        let retention_period = Duration::days(7); // 7日間保持

        let mut old_ids = Vec::new();

        for (id, tx) in &self.completed_transactions {
            if let Some(completed_at) = tx.completed_at {
                let elapsed = now - completed_at;

                if elapsed > retention_period {
                    old_ids.push(id.clone());
                }
            }
        }

        // 古いトランザクションを削除
        for id in old_ids {
            self.completed_transactions.remove(&id);
        }

        // 最後のクリーンアップ時刻を更新
        self.last_cleanup_time = now;

        Ok(())
    }

    /// メッセージをシャードに送信
    fn send_message_to_shard(
        &self,
        shard_id: &ShardId,
        message: CrossShardMessage,
    ) -> Result<(), Error> {
        // シャード情報を取得
        let shard_info = self
            .shard_manager
            .get_shard_info(shard_id)
            .ok_or_else(|| Error::NotFound(format!("Shard not found: {}", shard_id)))?;

        // メッセージをシリアライズ
        let message_data = serde_json::to_vec(&message).map_err(|e| {
            Error::SerializationError(format!("Failed to serialize message: {}", e))
        })?;

        // ネットワークメッセージを作成
        let network_message = NetworkMessage {
            message_type: MessageType::CrossShardMessage,
            sender: self.current_shard_id.clone(),
            receiver: shard_id.clone(),
            data: message_data,
            timestamp: Utc::now(),
        };

        // メッセージを送信
        // 実際の実装では、ネットワークレイヤーを使用してメッセージを送信
        // ここでは簡易的な実装を提供

        // メトリクスを更新
        self.metrics.increment_counter("cross_shard_messages_sent");

        Ok(())
    }

    /// トランザクションを取得
    pub fn get_transaction(&self, transaction_id: &str) -> Option<&CrossShardTransaction> {
        if let Some(tx) = self.pending_transactions.get(transaction_id) {
            Some(tx)
        } else {
            self.completed_transactions.get(transaction_id)
        }
    }

    /// 保留中のトランザクションを取得
    pub fn get_pending_transactions(&self) -> &HashMap<String, CrossShardTransaction> {
        &self.pending_transactions
    }

    /// 完了したトランザクションを取得
    pub fn get_completed_transactions(&self) -> &HashMap<String, CrossShardTransaction> {
        &self.completed_transactions
    }

    /// 送信元シャードの保留中トランザクションを取得
    pub fn get_pending_source_transactions(&self) -> Vec<&CrossShardTransaction> {
        self.pending_transactions
            .values()
            .filter(|tx| tx.source_shard_id == self.current_shard_id)
            .collect()
    }

    /// 送信先シャードの保留中トランザクションを取得
    pub fn get_pending_target_transactions(&self) -> Vec<&CrossShardTransaction> {
        self.pending_transactions
            .values()
            .filter(|tx| tx.target_shard_id == self.current_shard_id)
            .collect()
    }

    /// タイムアウトを設定
    pub fn set_timeout(&mut self, timeout_seconds: u64) {
        self.timeout_seconds = timeout_seconds;
    }

    /// 再試行間隔を設定
    pub fn set_retry_interval(&mut self, retry_interval_seconds: u64) {
        self.retry_interval_seconds = retry_interval_seconds;
    }

    /// 最大再試行回数を設定
    pub fn set_max_retries(&mut self, max_retries: u32) {
        self.max_retries = max_retries;
    }

    /// 必要な確認数を設定
    pub fn set_required_confirmations(&mut self, required_confirmations: u32) {
        self.required_confirmations = required_confirmations;
    }

    /// クリーンアップ間隔を設定
    pub fn set_cleanup_interval(&mut self, cleanup_interval_seconds: u64) {
        self.cleanup_interval_seconds = cleanup_interval_seconds;
    }
}

/// ステータスの優先度を取得
fn status_priority(status: &CrossShardTransactionStatus) -> u8 {
    match status {
        CrossShardTransactionStatus::Initialized => 1,
        CrossShardTransactionStatus::SourceCommitted => 2,
        CrossShardTransactionStatus::Transmitted => 3,
        CrossShardTransactionStatus::TargetReceived => 4,
        CrossShardTransactionStatus::TargetCommitted => 5,
        CrossShardTransactionStatus::SourceAcknowledged => 6,
        CrossShardTransactionStatus::Completed => 7,
        CrossShardTransactionStatus::Failed => 8,
        CrossShardTransactionStatus::TimedOut => 9,
        CrossShardTransactionStatus::Cancelled => 10,
    }
}
