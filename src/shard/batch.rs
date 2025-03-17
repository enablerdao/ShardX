use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use crate::error::Error;
use crate::shard::cross_shard::{
    CrossShardTransaction, CrossShardTransactionManager, CrossShardTransactionState,
};
use crate::shard::{ShardId, ShardManager};
use crate::transaction::{Transaction, TransactionStatus, TransactionType};

/// バッチ状態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BatchState {
    /// 作成済み
    Created,
    /// 処理中
    Processing,
    /// 完了
    Completed,
    /// 失敗
    Failed,
    /// キャンセル
    Cancelled,
}

/// トランザクションバッチ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionBatch {
    /// バッチID
    pub id: String,
    /// トランザクションリスト
    pub transactions: Vec<Transaction>,
    /// クロスシャードトランザクションリスト
    pub cross_shard_transactions: Vec<CrossShardTransaction>,
    /// 送信元シャード
    pub source_shard: ShardId,
    /// 送信先シャード
    pub destination_shard: ShardId,
    /// 状態
    pub state: BatchState,
    /// 作成時刻
    pub created_at: DateTime<Utc>,
    /// 更新時刻
    pub updated_at: DateTime<Utc>,
    /// 完了時刻
    pub completed_at: Option<DateTime<Utc>>,
    /// エラー
    pub error: Option<String>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

impl TransactionBatch {
    /// 新しいトランザクションバッチを作成
    pub fn new(
        transactions: Vec<Transaction>,
        cross_shard_transactions: Vec<CrossShardTransaction>,
        source_shard: ShardId,
        destination_shard: ShardId,
    ) -> Self {
        let now = Utc::now();
        let id = format!(
            "batch-{}",
            crate::crypto::hash(&format!("{:?}-{}", transactions, now))
        );

        Self {
            id,
            transactions,
            cross_shard_transactions,
            source_shard,
            destination_shard,
            state: BatchState::Created,
            created_at: now,
            updated_at: now,
            completed_at: None,
            error: None,
            metadata: HashMap::new(),
        }
    }

    /// トランザクション数を取得
    pub fn transaction_count(&self) -> usize {
        self.transactions.len() + self.cross_shard_transactions.len()
    }

    /// 全てのトランザクションが完了しているかどうかを確認
    pub fn is_complete(&self) -> bool {
        // 通常トランザクションの確認
        let normal_complete = self.transactions.iter().all(|tx| {
            tx.status == TransactionStatus::Confirmed || tx.status == TransactionStatus::Failed
        });

        // クロスシャードトランザクションの確認
        let cross_shard_complete = self.cross_shard_transactions.iter().all(|tx| {
            tx.state == CrossShardTransactionState::Completed
                || tx.state == CrossShardTransactionState::TimedOut
        });

        normal_complete && cross_shard_complete
    }

    /// 全てのトランザクションが成功しているかどうかを確認
    pub fn is_successful(&self) -> bool {
        // 通常トランザクションの確認
        let normal_success = self
            .transactions
            .iter()
            .all(|tx| tx.status == TransactionStatus::Confirmed);

        // クロスシャードトランザクションの確認
        let cross_shard_success = self
            .cross_shard_transactions
            .iter()
            .all(|tx| tx.state == CrossShardTransactionState::Completed && tx.error.is_none());

        normal_success && cross_shard_success
    }
}

/// バッチ処理設定
#[derive(Debug, Clone)]
pub struct BatchProcessorConfig {
    /// 最大バッチサイズ
    pub max_batch_size: usize,
    /// 最大待機時間（ミリ秒）
    pub max_wait_time_ms: u64,
    /// 最小バッチサイズ
    pub min_batch_size: usize,
    /// 優先度付きバッチ処理
    pub prioritized_batching: bool,
    /// 自動バッチ処理
    pub auto_batching: bool,
    /// バッチ処理間隔（ミリ秒）
    pub batch_interval_ms: u64,
}

impl Default for BatchProcessorConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            max_wait_time_ms: 1000,
            min_batch_size: 10,
            prioritized_batching: true,
            auto_batching: true,
            batch_interval_ms: 500,
        }
    }
}

/// バッチ処理マネージャー
pub struct BatchProcessor {
    /// 設定
    config: BatchProcessorConfig,
    /// シャードマネージャー
    shard_manager: Arc<ShardManager>,
    /// クロスシャードトランザクションマネージャー
    cross_shard_manager: Arc<CrossShardTransactionManager>,
    /// 保留中のトランザクション
    pending_transactions: Arc<Mutex<HashMap<ShardId, VecDeque<Transaction>>>>,
    /// 保留中のクロスシャードトランザクション
    pending_cross_shard_transactions:
        Arc<Mutex<HashMap<(ShardId, ShardId), VecDeque<CrossShardTransaction>>>>,
    /// バッチ
    batches: Arc<RwLock<HashMap<String, TransactionBatch>>>,
    /// 最終バッチ処理時刻
    last_batch_time: Arc<Mutex<HashMap<(ShardId, ShardId), Instant>>>,
}

impl BatchProcessor {
    /// 新しいバッチ処理マネージャーを作成
    pub fn new(
        shard_manager: Arc<ShardManager>,
        cross_shard_manager: Arc<CrossShardTransactionManager>,
        config: Option<BatchProcessorConfig>,
    ) -> Self {
        Self {
            config: config.unwrap_or_default(),
            shard_manager,
            cross_shard_manager,
            pending_transactions: Arc::new(Mutex::new(HashMap::new())),
            pending_cross_shard_transactions: Arc::new(Mutex::new(HashMap::new())),
            batches: Arc::new(RwLock::new(HashMap::new())),
            last_batch_time: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// トランザクションを追加
    pub fn add_transaction(&self, transaction: Transaction) -> Result<(), Error> {
        let shard_id = transaction.shard_id.clone();

        let mut pending_transactions = self.pending_transactions.lock().unwrap();
        let queue = pending_transactions
            .entry(shard_id)
            .or_insert_with(VecDeque::new);
        queue.push_back(transaction);

        Ok(())
    }

    /// クロスシャードトランザクションを追加
    pub fn add_cross_shard_transaction(
        &self,
        transaction: CrossShardTransaction,
    ) -> Result<(), Error> {
        let source_shard = transaction.source_shard.clone();
        let destination_shard = transaction.destination_shard.clone();

        let mut pending_cross_shard_transactions =
            self.pending_cross_shard_transactions.lock().unwrap();
        let queue = pending_cross_shard_transactions
            .entry((source_shard, destination_shard))
            .or_insert_with(VecDeque::new);
        queue.push_back(transaction);

        Ok(())
    }

    /// バッチを作成
    pub fn create_batch(
        &self,
        source_shard: &ShardId,
        destination_shard: &ShardId,
    ) -> Result<Option<TransactionBatch>, Error> {
        // 保留中のトランザクションを取得
        let mut pending_transactions = self.pending_transactions.lock().unwrap();
        let mut pending_cross_shard_transactions =
            self.pending_cross_shard_transactions.lock().unwrap();

        // 同一シャード内のトランザクション
        let normal_transactions = if source_shard == destination_shard {
            let queue = pending_transactions
                .entry(source_shard.clone())
                .or_insert_with(VecDeque::new);
            let count = std::cmp::min(queue.len(), self.config.max_batch_size);

            // 最小バッチサイズに満たない場合は待機
            if count < self.config.min_batch_size {
                // 最大待機時間を超えた場合は処理
                let last_batch_time = self.last_batch_time.lock().unwrap();
                let key = (source_shard.clone(), destination_shard.clone());
                if let Some(last_time) = last_batch_time.get(&key) {
                    let elapsed = last_time.elapsed().as_millis() as u64;
                    if elapsed < self.config.max_wait_time_ms && count > 0 {
                        return Ok(None);
                    }
                }
            }

            // トランザクションを取得
            (0..count)
                .filter_map(|_| queue.pop_front())
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        // クロスシャードトランザクション
        let key = (source_shard.clone(), destination_shard.clone());
        let cross_shard_transactions =
            if let Some(queue) = pending_cross_shard_transactions.get_mut(&key) {
                let remaining = self.config.max_batch_size - normal_transactions.len();
                let count = std::cmp::min(queue.len(), remaining);

                // トランザクションを取得
                (0..count)
                    .filter_map(|_| queue.pop_front())
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };

        // バッチを作成
        let total_count = normal_transactions.len() + cross_shard_transactions.len();
        if total_count > 0 {
            let batch = TransactionBatch::new(
                normal_transactions,
                cross_shard_transactions,
                source_shard.clone(),
                destination_shard.clone(),
            );

            // バッチを保存
            let mut batches = self.batches.write().unwrap();
            batches.insert(batch.id.clone(), batch.clone());

            // 最終バッチ処理時刻を更新
            let mut last_batch_time = self.last_batch_time.lock().unwrap();
            last_batch_time.insert(key, Instant::now());

            Ok(Some(batch))
        } else {
            Ok(None)
        }
    }

    /// バッチを処理
    pub fn process_batch(&self, batch_id: &str) -> Result<TransactionBatch, Error> {
        // バッチを取得
        let mut batches = self.batches.write().unwrap();
        let batch = batches
            .get_mut(batch_id)
            .ok_or_else(|| Error::NotFound(format!("バッチ {} が見つかりません", batch_id)))?;

        // 状態をチェック
        if batch.state != BatchState::Created {
            return Err(Error::InvalidState(format!(
                "バッチは {} 状態であり、処理できません",
                format!("{:?}", batch.state)
            )));
        }

        // 状態を更新
        batch.state = BatchState::Processing;
        batch.updated_at = Utc::now();

        // 通常トランザクションを処理
        for tx in &batch.transactions {
            // 実際の実装では、シャードにトランザクションを送信
            // ここでは簡易的な実装として、ログ出力のみ
            info!("バッチ {} のトランザクション {} を処理中", batch_id, tx.id);
        }

        // クロスシャードトランザクションを処理
        for tx in &batch.cross_shard_transactions {
            // 実際の実装では、クロスシャードトランザクションマネージャーにトランザクションを送信
            // ここでは簡易的な実装として、ログ出力のみ
            info!(
                "バッチ {} のクロスシャードトランザクション {} を処理中",
                batch_id, tx.id
            );
        }

        // 状態を更新
        batch.state = BatchState::Completed;
        batch.updated_at = Utc::now();
        batch.completed_at = Some(Utc::now());

        Ok(batch.clone())
    }

    /// バッチを取得
    pub fn get_batch(&self, batch_id: &str) -> Result<TransactionBatch, Error> {
        let batches = self.batches.read().unwrap();
        let batch = batches
            .get(batch_id)
            .ok_or_else(|| Error::NotFound(format!("バッチ {} が見つかりません", batch_id)))?;

        Ok(batch.clone())
    }

    /// 全バッチを取得
    pub fn get_all_batches(&self) -> Vec<TransactionBatch> {
        let batches = self.batches.read().unwrap();
        batches.values().cloned().collect()
    }

    /// 保留中のバッチを取得
    pub fn get_pending_batches(&self) -> Vec<TransactionBatch> {
        let batches = self.batches.read().unwrap();
        batches
            .values()
            .filter(|batch| {
                batch.state == BatchState::Created || batch.state == BatchState::Processing
            })
            .cloned()
            .collect()
    }

    /// 自動バッチ処理を実行
    pub fn run_auto_batching(&self) -> Result<Vec<TransactionBatch>, Error> {
        if !self.config.auto_batching {
            return Ok(Vec::new());
        }

        let mut processed_batches = Vec::new();

        // 全シャードの組み合わせを取得
        let shard_pairs = self.get_shard_pairs()?;

        // 各シャードペアに対してバッチを作成・処理
        for (source_shard, destination_shard) in shard_pairs {
            if let Some(batch) = self.create_batch(&source_shard, &destination_shard)? {
                let processed_batch = self.process_batch(&batch.id)?;
                processed_batches.push(processed_batch);
            }
        }

        Ok(processed_batches)
    }

    /// シャードペアを取得
    fn get_shard_pairs(&self) -> Result<Vec<(ShardId, ShardId)>, Error> {
        let mut pairs = HashSet::new();

        // 通常トランザクションのシャードペア
        {
            let pending_transactions = self.pending_transactions.lock().unwrap();
            for (shard_id, queue) in pending_transactions.iter() {
                if !queue.is_empty() {
                    pairs.insert((shard_id.clone(), shard_id.clone()));
                }
            }
        }

        // クロスシャードトランザクションのシャードペア
        {
            let pending_cross_shard_transactions =
                self.pending_cross_shard_transactions.lock().unwrap();
            for ((source_shard, destination_shard), queue) in
                pending_cross_shard_transactions.iter()
            {
                if !queue.is_empty() {
                    pairs.insert((source_shard.clone(), destination_shard.clone()));
                }
            }
        }

        Ok(pairs.into_iter().collect())
    }

    /// 設定を更新
    pub fn update_config(&mut self, config: BatchProcessorConfig) {
        self.config = config;
    }
}
