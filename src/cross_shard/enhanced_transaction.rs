use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::Error;
use crate::shard::ShardId;
use crate::transaction::{Transaction, TransactionStatus};

/// クロスシャードトランザクション状態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CrossShardTransactionState {
    /// 開始
    Initiated,
    /// 送信元シャードでコミット済み
    SourceCommitted,
    /// 送信先シャードで検証済み
    DestinationVerified,
    /// 送信先シャードでコミット済み
    DestinationCommitted,
    /// 完了
    Completed,
    /// 失敗
    Failed,
    /// タイムアウト
    TimedOut,
    /// キャンセル
    Cancelled,
}

/// クロスシャードトランザクション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedCrossShardTransaction {
    /// トランザクションID
    pub id: String,
    /// 元のトランザクション
    pub original_transaction: Transaction,
    /// 送信元シャードID
    pub source_shard_id: ShardId,
    /// 送信先シャードID
    pub destination_shard_id: ShardId,
    /// 状態
    pub state: CrossShardTransactionState,
    /// 作成時刻
    pub created_at: DateTime<Utc>,
    /// 更新時刻
    pub updated_at: DateTime<Utc>,
    /// 完了時刻
    pub completed_at: Option<DateTime<Utc>>,
    /// タイムアウト時刻
    pub timeout_at: DateTime<Utc>,
    /// 送信元シャードのコミットハッシュ
    pub source_commit_hash: Option<String>,
    /// 送信先シャードのコミットハッシュ
    pub destination_commit_hash: Option<String>,
    /// 検証ステップ
    pub verification_steps: Vec<VerificationStep>,
    /// エラーメッセージ
    pub error_message: Option<String>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 検証ステップ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationStep {
    /// ステップID
    pub id: String,
    /// ステップ名
    pub name: String,
    /// ステップの説明
    pub description: String,
    /// 実行時刻
    pub executed_at: DateTime<Utc>,
    /// 実行シャードID
    pub executed_by_shard: ShardId,
    /// 結果
    pub result: VerificationResult,
    /// 結果の詳細
    pub result_details: Option<String>,
}

/// 検証結果
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VerificationResult {
    /// 成功
    Success,
    /// 失敗
    Failure,
    /// スキップ
    Skipped,
}

/// クロスシャードトランザクションマネージャー
pub struct EnhancedCrossShardTransactionManager {
    /// トランザクション
    transactions: HashMap<String, EnhancedCrossShardTransaction>,
    /// タイムアウト（秒）
    timeout_seconds: u64,
}

impl EnhancedCrossShardTransactionManager {
    /// 新しいクロスシャードトランザクションマネージャーを作成
    pub fn new(timeout_seconds: u64) -> Self {
        Self {
            transactions: HashMap::new(),
            timeout_seconds,
        }
    }

    /// クロスシャードトランザクションを作成
    pub fn create_transaction(
        &mut self,
        transaction: Transaction,
        source_shard_id: ShardId,
        destination_shard_id: ShardId,
    ) -> Result<EnhancedCrossShardTransaction, Error> {
        // 同じシャード内のトランザクションはエラー
        if source_shard_id == destination_shard_id {
            return Err(Error::InvalidInput(format!(
                "送信元シャードと送信先シャードが同じです: {}",
                source_shard_id
            )));
        }

        let now = Utc::now();
        let timeout_at = now + chrono::Duration::seconds(self.timeout_seconds as i64);

        let cross_shard_tx = EnhancedCrossShardTransaction {
            id: format!("cstx-{}", transaction.id),
            original_transaction: transaction,
            source_shard_id,
            destination_shard_id,
            state: CrossShardTransactionState::Initiated,
            created_at: now,
            updated_at: now,
            completed_at: None,
            timeout_at,
            source_commit_hash: None,
            destination_commit_hash: None,
            verification_steps: Vec::new(),
            error_message: None,
            metadata: None,
        };

        self.transactions
            .insert(cross_shard_tx.id.clone(), cross_shard_tx.clone());

        Ok(cross_shard_tx)
    }

    /// クロスシャードトランザクションを取得
    pub fn get_transaction(&self, id: &str) -> Result<&EnhancedCrossShardTransaction, Error> {
        self.transactions.get(id).ok_or_else(|| {
            Error::NotFound(format!(
                "クロスシャードトランザクション {} が見つかりません",
                id
            ))
        })
    }

    /// クロスシャードトランザクションを更新
    pub fn update_transaction(
        &mut self,
        transaction: EnhancedCrossShardTransaction,
    ) -> Result<(), Error> {
        if !self.transactions.contains_key(&transaction.id) {
            return Err(Error::NotFound(format!(
                "クロスシャードトランザクション {} が見つかりません",
                transaction.id
            )));
        }

        self.transactions
            .insert(transaction.id.clone(), transaction);

        Ok(())
    }

    /// 送信元シャードでコミット
    pub fn commit_at_source(
        &mut self,
        id: &str,
        commit_hash: &str,
    ) -> Result<EnhancedCrossShardTransaction, Error> {
        let mut transaction = self.get_transaction(id)?.clone();

        // 状態をチェック
        if transaction.state != CrossShardTransactionState::Initiated {
            return Err(Error::InvalidState(format!(
                "トランザクションは {} 状態であり、送信元シャードでコミットできません",
                format!("{:?}", transaction.state)
            )));
        }

        // タイムアウトをチェック
        let now = Utc::now();
        if now > transaction.timeout_at {
            transaction.state = CrossShardTransactionState::TimedOut;
            transaction.updated_at = now;
            transaction.error_message = Some("トランザクションがタイムアウトしました".to_string());

            self.update_transaction(transaction.clone())?;

            return Err(Error::Timeout(
                "トランザクションがタイムアウトしました".to_string(),
            ));
        }

        // 検証ステップを追加
        let step = VerificationStep {
            id: format!("step-source-commit-{}", transaction.id),
            name: "送信元シャードでのコミット".to_string(),
            description: "送信元シャードでトランザクションをコミットしました".to_string(),
            executed_at: now,
            executed_by_shard: transaction.source_shard_id.clone(),
            result: VerificationResult::Success,
            result_details: Some(format!("コミットハッシュ: {}", commit_hash)),
        };

        transaction.verification_steps.push(step);

        // 状態を更新
        transaction.state = CrossShardTransactionState::SourceCommitted;
        transaction.updated_at = now;
        transaction.source_commit_hash = Some(commit_hash.to_string());

        self.update_transaction(transaction.clone())?;

        Ok(transaction)
    }

    /// 送信先シャードで検証
    pub fn verify_at_destination(
        &mut self,
        id: &str,
    ) -> Result<EnhancedCrossShardTransaction, Error> {
        let mut transaction = self.get_transaction(id)?.clone();

        // 状態をチェック
        if transaction.state != CrossShardTransactionState::SourceCommitted {
            return Err(Error::InvalidState(format!(
                "トランザクションは {} 状態であり、送信先シャードで検証できません",
                format!("{:?}", transaction.state)
            )));
        }

        // タイムアウトをチェック
        let now = Utc::now();
        if now > transaction.timeout_at {
            transaction.state = CrossShardTransactionState::TimedOut;
            transaction.updated_at = now;
            transaction.error_message = Some("トランザクションがタイムアウトしました".to_string());

            self.update_transaction(transaction.clone())?;

            return Err(Error::Timeout(
                "トランザクションがタイムアウトしました".to_string(),
            ));
        }

        // 検証ステップを追加
        let step = VerificationStep {
            id: format!("step-destination-verify-{}", transaction.id),
            name: "送信先シャードでの検証".to_string(),
            description: "送信先シャードでトランザクションを検証しました".to_string(),
            executed_at: now,
            executed_by_shard: transaction.destination_shard_id.clone(),
            result: VerificationResult::Success,
            result_details: None,
        };

        transaction.verification_steps.push(step);

        // 状態を更新
        transaction.state = CrossShardTransactionState::DestinationVerified;
        transaction.updated_at = now;

        self.update_transaction(transaction.clone())?;

        Ok(transaction)
    }

    /// 送信先シャードでコミット
    pub fn commit_at_destination(
        &mut self,
        id: &str,
        commit_hash: &str,
    ) -> Result<EnhancedCrossShardTransaction, Error> {
        let mut transaction = self.get_transaction(id)?.clone();

        // 状態をチェック
        if transaction.state != CrossShardTransactionState::DestinationVerified {
            return Err(Error::InvalidState(format!(
                "トランザクションは {} 状態であり、送信先シャードでコミットできません",
                format!("{:?}", transaction.state)
            )));
        }

        // タイムアウトをチェック
        let now = Utc::now();
        if now > transaction.timeout_at {
            transaction.state = CrossShardTransactionState::TimedOut;
            transaction.updated_at = now;
            transaction.error_message = Some("トランザクションがタイムアウトしました".to_string());

            self.update_transaction(transaction.clone())?;

            return Err(Error::Timeout(
                "トランザクションがタイムアウトしました".to_string(),
            ));
        }

        // 検証ステップを追加
        let step = VerificationStep {
            id: format!("step-destination-commit-{}", transaction.id),
            name: "送信先シャードでのコミット".to_string(),
            description: "送信先シャードでトランザクションをコミットしました".to_string(),
            executed_at: now,
            executed_by_shard: transaction.destination_shard_id.clone(),
            result: VerificationResult::Success,
            result_details: Some(format!("コミットハッシュ: {}", commit_hash)),
        };

        transaction.verification_steps.push(step);

        // 状態を更新
        transaction.state = CrossShardTransactionState::DestinationCommitted;
        transaction.updated_at = now;
        transaction.destination_commit_hash = Some(commit_hash.to_string());

        self.update_transaction(transaction.clone())?;

        Ok(transaction)
    }

    /// トランザクションを完了
    pub fn complete_transaction(
        &mut self,
        id: &str,
    ) -> Result<EnhancedCrossShardTransaction, Error> {
        let mut transaction = self.get_transaction(id)?.clone();

        // 状態をチェック
        if transaction.state != CrossShardTransactionState::DestinationCommitted {
            return Err(Error::InvalidState(format!(
                "トランザクションは {} 状態であり、完了できません",
                format!("{:?}", transaction.state)
            )));
        }

        // タイムアウトをチェック
        let now = Utc::now();
        if now > transaction.timeout_at {
            transaction.state = CrossShardTransactionState::TimedOut;
            transaction.updated_at = now;
            transaction.error_message = Some("トランザクションがタイムアウトしました".to_string());

            self.update_transaction(transaction.clone())?;

            return Err(Error::Timeout(
                "トランザクションがタイムアウトしました".to_string(),
            ));
        }

        // 検証ステップを追加
        let step = VerificationStep {
            id: format!("step-complete-{}", transaction.id),
            name: "トランザクション完了".to_string(),
            description: "クロスシャードトランザクションが正常に完了しました".to_string(),
            executed_at: now,
            executed_by_shard: transaction.source_shard_id.clone(),
            result: VerificationResult::Success,
            result_details: None,
        };

        transaction.verification_steps.push(step);

        // 状態を更新
        transaction.state = CrossShardTransactionState::Completed;
        transaction.updated_at = now;
        transaction.completed_at = Some(now);

        self.update_transaction(transaction.clone())?;

        Ok(transaction)
    }

    /// トランザクションを失敗させる
    pub fn fail_transaction(
        &mut self,
        id: &str,
        error_message: &str,
    ) -> Result<EnhancedCrossShardTransaction, Error> {
        let mut transaction = self.get_transaction(id)?.clone();

        // 既に完了または失敗している場合はエラー
        if transaction.state == CrossShardTransactionState::Completed
            || transaction.state == CrossShardTransactionState::Failed
            || transaction.state == CrossShardTransactionState::TimedOut
            || transaction.state == CrossShardTransactionState::Cancelled
        {
            return Err(Error::InvalidState(format!(
                "トランザクションは {} 状態であり、失敗させることができません",
                format!("{:?}", transaction.state)
            )));
        }

        let now = Utc::now();

        // 検証ステップを追加
        let step = VerificationStep {
            id: format!("step-fail-{}", transaction.id),
            name: "トランザクション失敗".to_string(),
            description: "クロスシャードトランザクションが失敗しました".to_string(),
            executed_at: now,
            executed_by_shard: transaction.source_shard_id.clone(),
            result: VerificationResult::Failure,
            result_details: Some(error_message.to_string()),
        };

        transaction.verification_steps.push(step);

        // 状態を更新
        transaction.state = CrossShardTransactionState::Failed;
        transaction.updated_at = now;
        transaction.error_message = Some(error_message.to_string());

        self.update_transaction(transaction.clone())?;

        Ok(transaction)
    }

    /// トランザクションをキャンセル
    pub fn cancel_transaction(
        &mut self,
        id: &str,
        reason: &str,
    ) -> Result<EnhancedCrossShardTransaction, Error> {
        let mut transaction = self.get_transaction(id)?.clone();

        // 既に完了または失敗している場合はエラー
        if transaction.state == CrossShardTransactionState::Completed
            || transaction.state == CrossShardTransactionState::Failed
            || transaction.state == CrossShardTransactionState::TimedOut
            || transaction.state == CrossShardTransactionState::Cancelled
        {
            return Err(Error::InvalidState(format!(
                "トランザクションは {} 状態であり、キャンセルできません",
                format!("{:?}", transaction.state)
            )));
        }

        let now = Utc::now();

        // 検証ステップを追加
        let step = VerificationStep {
            id: format!("step-cancel-{}", transaction.id),
            name: "トランザクションキャンセル".to_string(),
            description: "クロスシャードトランザクションがキャンセルされました".to_string(),
            executed_at: now,
            executed_by_shard: transaction.source_shard_id.clone(),
            result: VerificationResult::Skipped,
            result_details: Some(reason.to_string()),
        };

        transaction.verification_steps.push(step);

        // 状態を更新
        transaction.state = CrossShardTransactionState::Cancelled;
        transaction.updated_at = now;
        transaction.error_message = Some(format!("キャンセル理由: {}", reason));

        self.update_transaction(transaction.clone())?;

        Ok(transaction)
    }

    /// タイムアウトしたトランザクションをチェック
    pub fn check_timeouts(&mut self) -> Vec<EnhancedCrossShardTransaction> {
        let now = Utc::now();
        let mut timed_out_txs = Vec::new();

        for (id, tx) in self.transactions.iter() {
            if tx.state != CrossShardTransactionState::Completed
                && tx.state != CrossShardTransactionState::Failed
                && tx.state != CrossShardTransactionState::TimedOut
                && tx.state != CrossShardTransactionState::Cancelled
                && now > tx.timeout_at
            {
                // タイムアウトしたトランザクションを処理
                if let Ok(updated_tx) =
                    self.fail_transaction(id, "トランザクションがタイムアウトしました")
                {
                    timed_out_txs.push(updated_tx);
                }
            }
        }

        timed_out_txs
    }

    /// 送信元シャードのトランザクションを取得
    pub fn get_transactions_by_source_shard(
        &self,
        shard_id: &ShardId,
    ) -> Vec<&EnhancedCrossShardTransaction> {
        self.transactions
            .values()
            .filter(|tx| tx.source_shard_id == *shard_id)
            .collect()
    }

    /// 送信先シャードのトランザクションを取得
    pub fn get_transactions_by_destination_shard(
        &self,
        shard_id: &ShardId,
    ) -> Vec<&EnhancedCrossShardTransaction> {
        self.transactions
            .values()
            .filter(|tx| tx.destination_shard_id == *shard_id)
            .collect()
    }

    /// 状態別のトランザクションを取得
    pub fn get_transactions_by_state(
        &self,
        state: CrossShardTransactionState,
    ) -> Vec<&EnhancedCrossShardTransaction> {
        self.transactions
            .values()
            .filter(|tx| tx.state == state)
            .collect()
    }

    /// 保留中のトランザクションを取得
    pub fn get_pending_transactions(&self) -> Vec<&EnhancedCrossShardTransaction> {
        self.transactions
            .values()
            .filter(|tx| {
                tx.state != CrossShardTransactionState::Completed
                    && tx.state != CrossShardTransactionState::Failed
                    && tx.state != CrossShardTransactionState::TimedOut
                    && tx.state != CrossShardTransactionState::Cancelled
            })
            .collect()
    }

    /// トランザクションの進行状況を取得
    pub fn get_transaction_progress(&self, id: &str) -> Result<f64, Error> {
        let transaction = self.get_transaction(id)?;

        // 状態に基づいて進行状況を計算
        let progress = match transaction.state {
            CrossShardTransactionState::Initiated => 0.0,
            CrossShardTransactionState::SourceCommitted => 0.25,
            CrossShardTransactionState::DestinationVerified => 0.5,
            CrossShardTransactionState::DestinationCommitted => 0.75,
            CrossShardTransactionState::Completed => 1.0,
            CrossShardTransactionState::Failed => 1.0,
            CrossShardTransactionState::TimedOut => 1.0,
            CrossShardTransactionState::Cancelled => 1.0,
        };

        Ok(progress)
    }

    /// トランザクションの所要時間を取得（ミリ秒）
    pub fn get_transaction_duration(&self, id: &str) -> Result<Option<i64>, Error> {
        let transaction = self.get_transaction(id)?;

        // 完了または失敗した場合のみ所要時間を計算
        if transaction.state == CrossShardTransactionState::Completed
            || transaction.state == CrossShardTransactionState::Failed
            || transaction.state == CrossShardTransactionState::TimedOut
            || transaction.state == CrossShardTransactionState::Cancelled
        {
            let end_time = transaction.completed_at.unwrap_or(transaction.updated_at);
            let duration = (end_time - transaction.created_at).num_milliseconds();
            return Ok(Some(duration));
        }

        Ok(None)
    }

    /// トランザクションの統計情報を取得
    pub fn get_statistics(&self) -> CrossShardTransactionStatistics {
        let total = self.transactions.len();

        let completed = self
            .transactions
            .values()
            .filter(|tx| tx.state == CrossShardTransactionState::Completed)
            .count();

        let failed = self
            .transactions
            .values()
            .filter(|tx| tx.state == CrossShardTransactionState::Failed)
            .count();

        let timed_out = self
            .transactions
            .values()
            .filter(|tx| tx.state == CrossShardTransactionState::TimedOut)
            .count();

        let cancelled = self
            .transactions
            .values()
            .filter(|tx| tx.state == CrossShardTransactionState::Cancelled)
            .count();

        let pending = total - completed - failed - timed_out - cancelled;

        // 完了したトランザクションの平均所要時間を計算
        let completed_txs: Vec<&EnhancedCrossShardTransaction> = self
            .transactions
            .values()
            .filter(|tx| tx.state == CrossShardTransactionState::Completed)
            .collect();

        let avg_completion_time = if !completed_txs.is_empty() {
            let total_time: i64 = completed_txs
                .iter()
                .filter_map(|tx| {
                    tx.completed_at
                        .map(|t| (t - tx.created_at).num_milliseconds())
                })
                .sum();

            Some(total_time / completed_txs.len() as i64)
        } else {
            None
        };

        // シャードごとのトランザクション数を集計
        let mut transactions_by_source_shard: HashMap<ShardId, usize> = HashMap::new();
        let mut transactions_by_destination_shard: HashMap<ShardId, usize> = HashMap::new();

        for tx in self.transactions.values() {
            let source_count = transactions_by_source_shard
                .entry(tx.source_shard_id.clone())
                .or_insert(0);
            *source_count += 1;

            let dest_count = transactions_by_destination_shard
                .entry(tx.destination_shard_id.clone())
                .or_insert(0);
            *dest_count += 1;
        }

        CrossShardTransactionStatistics {
            total,
            completed,
            failed,
            timed_out,
            cancelled,
            pending,
            avg_completion_time,
            transactions_by_source_shard,
            transactions_by_destination_shard,
        }
    }
}

/// クロスシャードトランザクション統計情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossShardTransactionStatistics {
    /// 総トランザクション数
    pub total: usize,
    /// 完了したトランザクション数
    pub completed: usize,
    /// 失敗したトランザクション数
    pub failed: usize,
    /// タイムアウトしたトランザクション数
    pub timed_out: usize,
    /// キャンセルされたトランザクション数
    pub cancelled: usize,
    /// 保留中のトランザクション数
    pub pending: usize,
    /// 平均完了時間（ミリ秒）
    pub avg_completion_time: Option<i64>,
    /// 送信元シャード別トランザクション数
    pub transactions_by_source_shard: HashMap<ShardId, usize>,
    /// 送信先シャード別トランザクション数
    pub transactions_by_destination_shard: HashMap<ShardId, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_transaction() -> Transaction {
        Transaction {
            id: "tx1".to_string(),
            sender: "sender1".to_string(),
            receiver: "receiver1".to_string(),
            amount: 100,
            fee: 10,
            timestamp: Utc::now().timestamp(),
            signature: None,
            status: TransactionStatus::Pending,
            data: None,
        }
    }

    #[test]
    fn test_cross_shard_transaction_flow() {
        let mut manager = EnhancedCrossShardTransactionManager::new(3600);

        // トランザクションを作成
        let transaction = create_test_transaction();
        let cross_tx = manager
            .create_transaction(transaction, "shard1".to_string(), "shard2".to_string())
            .unwrap();

        assert_eq!(cross_tx.state, CrossShardTransactionState::Initiated);

        // 送信元シャードでコミット
        let cross_tx = manager
            .commit_at_source(&cross_tx.id, "source_hash")
            .unwrap();
        assert_eq!(cross_tx.state, CrossShardTransactionState::SourceCommitted);
        assert_eq!(cross_tx.source_commit_hash, Some("source_hash".to_string()));

        // 送信先シャードで検証
        let cross_tx = manager.verify_at_destination(&cross_tx.id).unwrap();
        assert_eq!(
            cross_tx.state,
            CrossShardTransactionState::DestinationVerified
        );

        // 送信先シャードでコミット
        let cross_tx = manager
            .commit_at_destination(&cross_tx.id, "dest_hash")
            .unwrap();
        assert_eq!(
            cross_tx.state,
            CrossShardTransactionState::DestinationCommitted
        );
        assert_eq!(
            cross_tx.destination_commit_hash,
            Some("dest_hash".to_string())
        );

        // トランザクションを完了
        let cross_tx = manager.complete_transaction(&cross_tx.id).unwrap();
        assert_eq!(cross_tx.state, CrossShardTransactionState::Completed);
        assert!(cross_tx.completed_at.is_some());

        // 統計情報を取得
        let stats = manager.get_statistics();
        assert_eq!(stats.total, 1);
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.failed, 0);
    }

    #[test]
    fn test_cross_shard_transaction_failure() {
        let mut manager = EnhancedCrossShardTransactionManager::new(3600);

        // トランザクションを作成
        let transaction = create_test_transaction();
        let cross_tx = manager
            .create_transaction(transaction, "shard1".to_string(), "shard2".to_string())
            .unwrap();

        // トランザクションを失敗させる
        let cross_tx = manager
            .fail_transaction(&cross_tx.id, "テストエラー")
            .unwrap();
        assert_eq!(cross_tx.state, CrossShardTransactionState::Failed);
        assert_eq!(cross_tx.error_message, Some("テストエラー".to_string()));

        // 統計情報を取得
        let stats = manager.get_statistics();
        assert_eq!(stats.total, 1);
        assert_eq!(stats.completed, 0);
        assert_eq!(stats.failed, 1);
    }

    #[test]
    fn test_cross_shard_transaction_cancellation() {
        let mut manager = EnhancedCrossShardTransactionManager::new(3600);

        // トランザクションを作成
        let transaction = create_test_transaction();
        let cross_tx = manager
            .create_transaction(transaction, "shard1".to_string(), "shard2".to_string())
            .unwrap();

        // トランザクションをキャンセル
        let cross_tx = manager
            .cancel_transaction(&cross_tx.id, "ユーザーによるキャンセル")
            .unwrap();
        assert_eq!(cross_tx.state, CrossShardTransactionState::Cancelled);
        assert!(cross_tx
            .error_message
            .unwrap()
            .contains("ユーザーによるキャンセル"));

        // 統計情報を取得
        let stats = manager.get_statistics();
        assert_eq!(stats.total, 1);
        assert_eq!(stats.completed, 0);
        assert_eq!(stats.cancelled, 1);
    }

    #[test]
    fn test_cross_shard_transaction_progress() {
        let mut manager = EnhancedCrossShardTransactionManager::new(3600);

        // トランザクションを作成
        let transaction = create_test_transaction();
        let cross_tx = manager
            .create_transaction(transaction, "shard1".to_string(), "shard2".to_string())
            .unwrap();

        // 進行状況をチェック
        let progress = manager.get_transaction_progress(&cross_tx.id).unwrap();
        assert_eq!(progress, 0.0);

        // 送信元シャードでコミット
        let cross_tx = manager
            .commit_at_source(&cross_tx.id, "source_hash")
            .unwrap();
        let progress = manager.get_transaction_progress(&cross_tx.id).unwrap();
        assert_eq!(progress, 0.25);

        // 送信先シャードで検証
        let cross_tx = manager.verify_at_destination(&cross_tx.id).unwrap();
        let progress = manager.get_transaction_progress(&cross_tx.id).unwrap();
        assert_eq!(progress, 0.5);

        // 送信先シャードでコミット
        let cross_tx = manager
            .commit_at_destination(&cross_tx.id, "dest_hash")
            .unwrap();
        let progress = manager.get_transaction_progress(&cross_tx.id).unwrap();
        assert_eq!(progress, 0.75);

        // トランザクションを完了
        let cross_tx = manager.complete_transaction(&cross_tx.id).unwrap();
        let progress = manager.get_transaction_progress(&cross_tx.id).unwrap();
        assert_eq!(progress, 1.0);
    }
}
