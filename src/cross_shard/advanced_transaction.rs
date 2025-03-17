use chrono::{DateTime, Duration, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};

use crate::cross_shard::enhanced_transaction::{
    CrossShardTransactionState, CrossShardTransactionStatistics, EnhancedCrossShardTransaction,
    EnhancedCrossShardTransactionManager, VerificationResult, VerificationStep,
};
use crate::error::Error;
use crate::shard::ShardId;
use crate::transaction::{Transaction, TransactionStatus};

/// 高度なクロスシャードトランザクション状態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdvancedCrossShardTransactionState {
    /// 基本状態
    Basic(CrossShardTransactionState),
    /// 準備中
    Preparing,
    /// 送信元シャードでロック中
    SourceLocking,
    /// 送信元シャードでロック済み
    SourceLocked,
    /// 送信先シャードで準備中
    DestinationPreparing,
    /// 送信先シャードでロック中
    DestinationLocking,
    /// 送信先シャードでロック済み
    DestinationLocked,
    /// 送信元シャードでコミット中
    SourceCommitting,
    /// 送信先シャードでコミット中
    DestinationCommitting,
    /// 送信元シャードでロールバック中
    SourceRollingBack,
    /// 送信先シャードでロールバック中
    DestinationRollingBack,
    /// ロールバック完了
    RolledBack,
    /// 部分的に完了
    PartiallyCompleted,
    /// 復旧中
    Recovering,
    /// 復旧完了
    Recovered,
    /// 検証中
    Validating,
    /// 検証失敗
    ValidationFailed,
    /// 一時停止
    Paused,
    /// 再開
    Resumed,
    /// 再試行中
    Retrying,
}

/// 高度なクロスシャードトランザクション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedCrossShardTransaction {
    /// 基本トランザクション情報
    pub base_transaction: EnhancedCrossShardTransaction,
    /// 高度な状態
    pub advanced_state: AdvancedCrossShardTransactionState,
    /// 関連するシャード
    pub involved_shards: Vec<ShardId>,
    /// シャードごとの状態
    pub shard_states: HashMap<ShardId, ShardTransactionState>,
    /// 依存するトランザクション
    pub dependencies: Vec<String>,
    /// 依存されるトランザクション
    pub dependents: Vec<String>,
    /// 優先度
    pub priority: TransactionPriority,
    /// 再試行回数
    pub retry_count: u32,
    /// 最大再試行回数
    pub max_retries: u32,
    /// 最後の再試行時刻
    pub last_retry_at: Option<DateTime<Utc>>,
    /// 再試行間隔（秒）
    pub retry_interval_seconds: u64,
    /// 検証ステップ
    pub validation_steps: Vec<AdvancedVerificationStep>,
    /// ロック
    pub locks: Vec<ResourceLock>,
    /// 実行計画
    pub execution_plan: Option<ExecutionPlan>,
    /// 実行結果
    pub execution_results: Vec<ExecutionResult>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
    /// 監査ログ
    pub audit_log: Vec<AuditLogEntry>,
    /// パフォーマンス指標
    pub performance_metrics: PerformanceMetrics,
}

/// シャードトランザクション状態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShardTransactionState {
    /// 未開始
    NotStarted,
    /// 準備中
    Preparing,
    /// ロック中
    Locking,
    /// ロック済み
    Locked,
    /// コミット中
    Committing,
    /// コミット済み
    Committed,
    /// ロールバック中
    RollingBack,
    /// ロールバック済み
    RolledBack,
    /// 失敗
    Failed,
    /// タイムアウト
    TimedOut,
}

/// トランザクション優先度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionPriority {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 緊急
    Critical,
    /// カスタム
    Custom(u32),
}

/// 高度な検証ステップ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedVerificationStep {
    /// ステップID
    pub id: String,
    /// ステップ名
    pub name: String,
    /// ステップの説明
    pub description: String,
    /// 実行シャード
    pub executing_shard: ShardId,
    /// 状態
    pub status: VerificationStepStatus,
    /// 開始時刻
    pub started_at: Option<DateTime<Utc>>,
    /// 完了時刻
    pub completed_at: Option<DateTime<Utc>>,
    /// 結果
    pub result: Option<AdvancedVerificationResult>,
    /// エラーメッセージ
    pub error_message: Option<String>,
    /// 再試行回数
    pub retry_count: u32,
    /// 依存するステップ
    pub dependencies: Vec<String>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 検証ステップ状態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VerificationStepStatus {
    /// 未開始
    NotStarted,
    /// 実行中
    InProgress,
    /// 完了
    Completed,
    /// 失敗
    Failed,
    /// スキップ
    Skipped,
    /// 待機中
    Waiting,
    /// タイムアウト
    TimedOut,
}

/// 高度な検証結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedVerificationResult {
    /// 成功フラグ
    pub success: bool,
    /// 検証コード
    pub code: String,
    /// メッセージ
    pub message: String,
    /// 詳細
    pub details: Option<HashMap<String, String>>,
    /// 検証時刻
    pub verified_at: DateTime<Utc>,
    /// 検証者
    pub verified_by: String,
    /// 署名
    pub signature: Option<String>,
}

/// リソースロック
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLock {
    /// ロックID
    pub id: String,
    /// リソースタイプ
    pub resource_type: ResourceType,
    /// リソースID
    pub resource_id: String,
    /// シャードID
    pub shard_id: ShardId,
    /// ロックモード
    pub lock_mode: LockMode,
    /// 取得時刻
    pub acquired_at: DateTime<Utc>,
    /// 解放時刻
    pub released_at: Option<DateTime<Utc>>,
    /// タイムアウト時刻
    pub timeout_at: DateTime<Utc>,
    /// ロック所有者
    pub owner: String,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// リソースタイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResourceType {
    /// アカウント
    Account,
    /// トランザクション
    Transaction,
    /// ブロック
    Block,
    /// コントラクト
    Contract,
    /// ストレージ
    Storage,
    /// その他
    Other(String),
}

/// ロックモード
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LockMode {
    /// 共有（読み取り）
    Shared,
    /// 排他（書き込み）
    Exclusive,
    /// インテント共有
    IntentShared,
    /// インテント排他
    IntentExclusive,
}

/// 実行計画
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// 計画ID
    pub id: String,
    /// 計画名
    pub name: String,
    /// 計画の説明
    pub description: String,
    /// 実行ステップ
    pub steps: Vec<ExecutionStep>,
    /// 作成時刻
    pub created_at: DateTime<Utc>,
    /// 更新時刻
    pub updated_at: DateTime<Utc>,
    /// 開始時刻
    pub started_at: Option<DateTime<Utc>>,
    /// 完了時刻
    pub completed_at: Option<DateTime<Utc>>,
    /// 状態
    pub status: ExecutionPlanStatus,
    /// タイムアウト（秒）
    pub timeout_seconds: u64,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 実行ステップ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    /// ステップID
    pub id: String,
    /// ステップ名
    pub name: String,
    /// ステップの説明
    pub description: String,
    /// 実行シャード
    pub executing_shard: ShardId,
    /// アクション
    pub action: ExecutionAction,
    /// 状態
    pub status: ExecutionStepStatus,
    /// 開始時刻
    pub started_at: Option<DateTime<Utc>>,
    /// 完了時刻
    pub completed_at: Option<DateTime<Utc>>,
    /// 結果
    pub result: Option<ExecutionResult>,
    /// エラーメッセージ
    pub error_message: Option<String>,
    /// 再試行回数
    pub retry_count: u32,
    /// 最大再試行回数
    pub max_retries: u32,
    /// 依存するステップ
    pub dependencies: Vec<String>,
    /// 補償ステップ
    pub compensation_step: Option<String>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 実行アクション
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionAction {
    /// 準備
    Prepare,
    /// ロック取得
    AcquireLock,
    /// 検証
    Validate,
    /// コミット
    Commit,
    /// ロック解放
    ReleaseLock,
    /// ロールバック
    Rollback,
    /// 通知
    Notify,
    /// カスタム
    Custom(String),
}

/// 実行ステップ状態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionStepStatus {
    /// 未開始
    NotStarted,
    /// 実行中
    InProgress,
    /// 完了
    Completed,
    /// 失敗
    Failed,
    /// スキップ
    Skipped,
    /// 待機中
    Waiting,
    /// タイムアウト
    TimedOut,
    /// キャンセル
    Cancelled,
}

/// 実行計画状態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionPlanStatus {
    /// 未開始
    NotStarted,
    /// 実行中
    InProgress,
    /// 完了
    Completed,
    /// 失敗
    Failed,
    /// 部分的に完了
    PartiallyCompleted,
    /// ロールバック中
    RollingBack,
    /// ロールバック完了
    RolledBack,
    /// 一時停止
    Paused,
    /// タイムアウト
    TimedOut,
    /// キャンセル
    Cancelled,
}

/// 実行結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// 成功フラグ
    pub success: bool,
    /// 結果コード
    pub code: String,
    /// メッセージ
    pub message: String,
    /// 詳細
    pub details: Option<HashMap<String, String>>,
    /// 実行時刻
    pub executed_at: DateTime<Utc>,
    /// 実行者
    pub executed_by: String,
    /// 実行時間（ミリ秒）
    pub execution_time_ms: u64,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 監査ログエントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// エントリID
    pub id: String,
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// アクション
    pub action: String,
    /// アクター
    pub actor: String,
    /// リソースタイプ
    pub resource_type: String,
    /// リソースID
    pub resource_id: String,
    /// 詳細
    pub details: String,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// パフォーマンス指標
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// 準備時間（ミリ秒）
    pub preparation_time_ms: u64,
    /// ロック取得時間（ミリ秒）
    pub lock_acquisition_time_ms: u64,
    /// 検証時間（ミリ秒）
    pub validation_time_ms: u64,
    /// コミット時間（ミリ秒）
    pub commit_time_ms: u64,
    /// 総実行時間（ミリ秒）
    pub total_execution_time_ms: u64,
    /// ネットワークレイテンシ（ミリ秒）
    pub network_latency_ms: u64,
    /// シャード間通信回数
    pub inter_shard_communication_count: u32,
    /// リソース使用量
    pub resource_usage: ResourceUsage,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// リソース使用量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU使用時間（ミリ秒）
    pub cpu_time_ms: u64,
    /// メモリ使用量（バイト）
    pub memory_bytes: u64,
    /// ディスクIO（バイト）
    pub disk_io_bytes: u64,
    /// ネットワークIO（バイト）
    pub network_io_bytes: u64,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 高度なクロスシャードトランザクション統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedCrossShardTransactionStatistics {
    /// 基本統計
    pub base_statistics: CrossShardTransactionStatistics,
    /// シャードごとの統計
    pub per_shard_statistics: HashMap<ShardId, ShardTransactionStatistics>,
    /// 状態ごとの統計
    pub per_state_statistics: HashMap<AdvancedCrossShardTransactionState, u64>,
    /// 優先度ごとの統計
    pub per_priority_statistics: HashMap<TransactionPriority, PriorityStatistics>,
    /// 時間帯別統計
    pub time_based_statistics: TimeBasedStatistics,
    /// パフォーマンス統計
    pub performance_statistics: PerformanceStatistics,
    /// エラー統計
    pub error_statistics: ErrorStatistics,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// シャードトランザクション統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardTransactionStatistics {
    /// 総トランザクション数
    pub total_transactions: u64,
    /// 成功したトランザクション数
    pub successful_transactions: u64,
    /// 失敗したトランザクション数
    pub failed_transactions: u64,
    /// 処理中のトランザクション数
    pub in_progress_transactions: u64,
    /// 平均処理時間（ミリ秒）
    pub average_processing_time_ms: u64,
    /// 最大処理時間（ミリ秒）
    pub max_processing_time_ms: u64,
    /// 最小処理時間（ミリ秒）
    pub min_processing_time_ms: u64,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 優先度統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityStatistics {
    /// 総トランザクション数
    pub total_transactions: u64,
    /// 成功したトランザクション数
    pub successful_transactions: u64,
    /// 失敗したトランザクション数
    pub failed_transactions: u64,
    /// 平均処理時間（ミリ秒）
    pub average_processing_time_ms: u64,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 時間帯別統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBasedStatistics {
    /// 時間帯別トランザクション数
    pub transactions_by_hour: HashMap<u8, u64>,
    /// 日別トランザクション数
    pub transactions_by_day: HashMap<u8, u64>,
    /// 月別トランザクション数
    pub transactions_by_month: HashMap<u8, u64>,
    /// ピーク時間
    pub peak_hour: u8,
    /// ピーク日
    pub peak_day: u8,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// パフォーマンス統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStatistics {
    /// 平均準備時間（ミリ秒）
    pub average_preparation_time_ms: u64,
    /// 平均ロック取得時間（ミリ秒）
    pub average_lock_acquisition_time_ms: u64,
    /// 平均検証時間（ミリ秒）
    pub average_validation_time_ms: u64,
    /// 平均コミット時間（ミリ秒）
    pub average_commit_time_ms: u64,
    /// 平均総実行時間（ミリ秒）
    pub average_total_execution_time_ms: u64,
    /// 平均ネットワークレイテンシ（ミリ秒）
    pub average_network_latency_ms: u64,
    /// 平均シャード間通信回数
    pub average_inter_shard_communication_count: f64,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// エラー統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStatistics {
    /// 総エラー数
    pub total_errors: u64,
    /// エラータイプごとの数
    pub errors_by_type: HashMap<String, u64>,
    /// シャードごとのエラー数
    pub errors_by_shard: HashMap<ShardId, u64>,
    /// 最も一般的なエラー
    pub most_common_error: Option<String>,
    /// 最も一般的なエラーの発生回数
    pub most_common_error_count: u64,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 高度なクロスシャードトランザクションマネージャー
pub struct AdvancedCrossShardTransactionManager {
    /// 基本マネージャー
    base_manager: EnhancedCrossShardTransactionManager,
    /// トランザクションのマップ
    transactions: HashMap<String, AdvancedCrossShardTransaction>,
    /// シャードごとのトランザクションインデックス
    shard_indices: HashMap<ShardId, HashSet<String>>,
    /// 状態ごとのトランザクションインデックス
    state_indices: HashMap<AdvancedCrossShardTransactionState, HashSet<String>>,
    /// 優先度ごとのトランザクションインデックス
    priority_indices: HashMap<TransactionPriority, HashSet<String>>,
    /// 依存関係グラフ
    dependency_graph: HashMap<String, HashSet<String>>,
    /// ロックマネージャー
    lock_manager: Arc<Mutex<LockManager>>,
    /// 実行キュー
    execution_queue: VecDeque<String>,
    /// 統計
    statistics: AdvancedCrossShardTransactionStatistics,
}

/// ロックマネージャー
pub struct LockManager {
    /// アクティブなロック
    active_locks: HashMap<String, ResourceLock>,
    /// リソースごとのロック
    resource_locks: HashMap<(ResourceType, String), HashSet<String>>,
    /// 待機中のロックリクエスト
    waiting_lock_requests: VecDeque<LockRequest>,
}

/// ロックリクエスト
#[derive(Debug, Clone)]
pub struct LockRequest {
    /// リクエストID
    pub id: String,
    /// トランザクションID
    pub transaction_id: String,
    /// リソースタイプ
    pub resource_type: ResourceType,
    /// リソースID
    pub resource_id: String,
    /// シャードID
    pub shard_id: ShardId,
    /// ロックモード
    pub lock_mode: LockMode,
    /// リクエスト時刻
    pub requested_at: DateTime<Utc>,
    /// タイムアウト時刻
    pub timeout_at: DateTime<Utc>,
    /// ロック所有者
    pub owner: String,
}

impl AdvancedCrossShardTransactionManager {
    /// 新しい高度なクロスシャードトランザクションマネージャーを作成
    pub fn new() -> Self {
        let base_statistics = CrossShardTransactionStatistics {
            total_transactions: 0,
            successful_transactions: 0,
            failed_transactions: 0,
            in_progress_transactions: 0,
            average_completion_time_seconds: 0.0,
        };

        let statistics = AdvancedCrossShardTransactionStatistics {
            base_statistics,
            per_shard_statistics: HashMap::new(),
            per_state_statistics: HashMap::new(),
            per_priority_statistics: HashMap::new(),
            time_based_statistics: TimeBasedStatistics {
                transactions_by_hour: HashMap::new(),
                transactions_by_day: HashMap::new(),
                transactions_by_month: HashMap::new(),
                peak_hour: 0,
                peak_day: 0,
                metadata: None,
            },
            performance_statistics: PerformanceStatistics {
                average_preparation_time_ms: 0,
                average_lock_acquisition_time_ms: 0,
                average_validation_time_ms: 0,
                average_commit_time_ms: 0,
                average_total_execution_time_ms: 0,
                average_network_latency_ms: 0,
                average_inter_shard_communication_count: 0.0,
                metadata: None,
            },
            error_statistics: ErrorStatistics {
                total_errors: 0,
                errors_by_type: HashMap::new(),
                errors_by_shard: HashMap::new(),
                most_common_error: None,
                most_common_error_count: 0,
                metadata: None,
            },
            metadata: None,
        };

        Self {
            base_manager: EnhancedCrossShardTransactionManager::new(),
            transactions: HashMap::new(),
            shard_indices: HashMap::new(),
            state_indices: HashMap::new(),
            priority_indices: HashMap::new(),
            dependency_graph: HashMap::new(),
            lock_manager: Arc::new(Mutex::new(LockManager {
                active_locks: HashMap::new(),
                resource_locks: HashMap::new(),
                waiting_lock_requests: VecDeque::new(),
            })),
            execution_queue: VecDeque::new(),
            statistics,
        }
    }

    /// トランザクションを作成
    pub fn create_transaction(
        &mut self,
        original_transaction: Transaction,
        source_shard_id: ShardId,
        destination_shard_id: ShardId,
        involved_shards: Vec<ShardId>,
        priority: TransactionPriority,
        dependencies: Vec<String>,
    ) -> Result<String, Error> {
        // 基本トランザクションを作成
        let base_tx_id = self.base_manager.create_transaction(
            original_transaction.clone(),
            source_shard_id.clone(),
            destination_shard_id.clone(),
        )?;

        let base_transaction = self
            .base_manager
            .get_transaction(&base_tx_id)
            .unwrap()
            .clone();

        // 高度なトランザクションを作成
        let now = Utc::now();
        let tx_id = base_tx_id.clone();

        // シャードごとの状態を初期化
        let mut shard_states = HashMap::new();
        for shard_id in &involved_shards {
            shard_states.insert(shard_id.clone(), ShardTransactionState::NotStarted);
        }

        // 実行計画を作成
        let execution_plan = self.create_execution_plan(
            &tx_id,
            &source_shard_id,
            &destination_shard_id,
            &involved_shards,
        );

        let transaction = AdvancedCrossShardTransaction {
            base_transaction,
            advanced_state: AdvancedCrossShardTransactionState::Preparing,
            involved_shards,
            shard_states,
            dependencies,
            dependents: Vec::new(),
            priority,
            retry_count: 0,
            max_retries: 3,
            last_retry_at: None,
            retry_interval_seconds: 60,
            validation_steps: Vec::new(),
            locks: Vec::new(),
            execution_plan: Some(execution_plan),
            execution_results: Vec::new(),
            metadata: None,
            audit_log: vec![AuditLogEntry {
                id: format!("audit-{}-{}", tx_id, now.timestamp()),
                timestamp: now,
                action: "CREATE".to_string(),
                actor: "system".to_string(),
                resource_type: "CrossShardTransaction".to_string(),
                resource_id: tx_id.clone(),
                details: "Transaction created".to_string(),
                metadata: None,
            }],
            performance_metrics: PerformanceMetrics {
                preparation_time_ms: 0,
                lock_acquisition_time_ms: 0,
                validation_time_ms: 0,
                commit_time_ms: 0,
                total_execution_time_ms: 0,
                network_latency_ms: 0,
                inter_shard_communication_count: 0,
                resource_usage: ResourceUsage {
                    cpu_time_ms: 0,
                    memory_bytes: 0,
                    disk_io_bytes: 0,
                    network_io_bytes: 0,
                    metadata: None,
                },
                metadata: None,
            },
        };

        // トランザクションを保存
        self.transactions.insert(tx_id.clone(), transaction);

        // インデックスを更新
        for shard_id in &involved_shards {
            let transactions = self
                .shard_indices
                .entry(shard_id.clone())
                .or_insert_with(HashSet::new);
            transactions.insert(tx_id.clone());
        }

        let state_transactions = self
            .state_indices
            .entry(AdvancedCrossShardTransactionState::Preparing)
            .or_insert_with(HashSet::new);
        state_transactions.insert(tx_id.clone());

        let priority_transactions = self
            .priority_indices
            .entry(priority)
            .or_insert_with(HashSet::new);
        priority_transactions.insert(tx_id.clone());

        // 依存関係グラフを更新
        for dep_id in &dependencies {
            let dependents = self
                .dependency_graph
                .entry(dep_id.clone())
                .or_insert_with(HashSet::new);
            dependents.insert(tx_id.clone());
        }

        // 実行キューに追加
        self.execution_queue.push_back(tx_id.clone());

        // 統計を更新
        self.update_statistics_for_new_transaction(
            &tx_id,
            &source_shard_id,
            &destination_shard_id,
            &priority,
        );

        Ok(tx_id)
    }

    /// 実行計画を作成
    fn create_execution_plan(
        &self,
        tx_id: &str,
        source_shard_id: &ShardId,
        destination_shard_id: &ShardId,
        involved_shards: &[ShardId],
    ) -> ExecutionPlan {
        let now = Utc::now();
        let plan_id = format!("plan-{}-{}", tx_id, now.timestamp());

        let mut steps = Vec::new();
        let mut step_id = 0;

        // 準備ステップ
        steps.push(ExecutionStep {
            id: format!("step-{}-{}", tx_id, step_id),
            name: "Prepare".to_string(),
            description: "Prepare transaction".to_string(),
            executing_shard: source_shard_id.clone(),
            action: ExecutionAction::Prepare,
            status: ExecutionStepStatus::NotStarted,
            started_at: None,
            completed_at: None,
            result: None,
            error_message: None,
            retry_count: 0,
            max_retries: 3,
            dependencies: Vec::new(),
            compensation_step: None,
            metadata: None,
        });
        step_id += 1;

        // 送信元シャードでロックを取得
        steps.push(ExecutionStep {
            id: format!("step-{}-{}", tx_id, step_id),
            name: "AcquireSourceLock".to_string(),
            description: "Acquire lock on source shard".to_string(),
            executing_shard: source_shard_id.clone(),
            action: ExecutionAction::AcquireLock,
            status: ExecutionStepStatus::NotStarted,
            started_at: None,
            completed_at: None,
            result: None,
            error_message: None,
            retry_count: 0,
            max_retries: 3,
            dependencies: vec![format!("step-{}-{}", tx_id, step_id - 1)],
            compensation_step: Some(format!("step-{}-{}", tx_id, step_id + 6)),
            metadata: None,
        });
        step_id += 1;

        // 送信先シャードでロックを取得
        steps.push(ExecutionStep {
            id: format!("step-{}-{}", tx_id, step_id),
            name: "AcquireDestinationLock".to_string(),
            description: "Acquire lock on destination shard".to_string(),
            executing_shard: destination_shard_id.clone(),
            action: ExecutionAction::AcquireLock,
            status: ExecutionStepStatus::NotStarted,
            started_at: None,
            completed_at: None,
            result: None,
            error_message: None,
            retry_count: 0,
            max_retries: 3,
            dependencies: vec![format!("step-{}-{}", tx_id, step_id - 1)],
            compensation_step: Some(format!("step-{}-{}", tx_id, step_id + 5)),
            metadata: None,
        });
        step_id += 1;

        // 送信元シャードで検証
        steps.push(ExecutionStep {
            id: format!("step-{}-{}", tx_id, step_id),
            name: "ValidateSource".to_string(),
            description: "Validate transaction on source shard".to_string(),
            executing_shard: source_shard_id.clone(),
            action: ExecutionAction::Validate,
            status: ExecutionStepStatus::NotStarted,
            started_at: None,
            completed_at: None,
            result: None,
            error_message: None,
            retry_count: 0,
            max_retries: 3,
            dependencies: vec![format!("step-{}-{}", tx_id, step_id - 2)],
            compensation_step: None,
            metadata: None,
        });
        step_id += 1;

        // 送信先シャードで検証
        steps.push(ExecutionStep {
            id: format!("step-{}-{}", tx_id, step_id),
            name: "ValidateDestination".to_string(),
            description: "Validate transaction on destination shard".to_string(),
            executing_shard: destination_shard_id.clone(),
            action: ExecutionAction::Validate,
            status: ExecutionStepStatus::NotStarted,
            started_at: None,
            completed_at: None,
            result: None,
            error_message: None,
            retry_count: 0,
            max_retries: 3,
            dependencies: vec![format!("step-{}-{}", tx_id, step_id - 2)],
            compensation_step: None,
            metadata: None,
        });
        step_id += 1;

        // 送信元シャードでコミット
        steps.push(ExecutionStep {
            id: format!("step-{}-{}", tx_id, step_id),
            name: "CommitSource".to_string(),
            description: "Commit transaction on source shard".to_string(),
            executing_shard: source_shard_id.clone(),
            action: ExecutionAction::Commit,
            status: ExecutionStepStatus::NotStarted,
            started_at: None,
            completed_at: None,
            result: None,
            error_message: None,
            retry_count: 0,
            max_retries: 3,
            dependencies: vec![
                format!("step-{}-{}", tx_id, step_id - 2),
                format!("step-{}-{}", tx_id, step_id - 1),
            ],
            compensation_step: Some(format!("step-{}-{}", tx_id, step_id + 2)),
            metadata: None,
        });
        step_id += 1;

        // 送信先シャードでコミット
        steps.push(ExecutionStep {
            id: format!("step-{}-{}", tx_id, step_id),
            name: "CommitDestination".to_string(),
            description: "Commit transaction on destination shard".to_string(),
            executing_shard: destination_shard_id.clone(),
            action: ExecutionAction::Commit,
            status: ExecutionStepStatus::NotStarted,
            started_at: None,
            completed_at: None,
            result: None,
            error_message: None,
            retry_count: 0,
            max_retries: 3,
            dependencies: vec![format!("step-{}-{}", tx_id, step_id - 1)],
            compensation_step: Some(format!("step-{}-{}", tx_id, step_id + 1)),
            metadata: None,
        });
        step_id += 1;

        // 送信先シャードでロールバック（補償）
        steps.push(ExecutionStep {
            id: format!("step-{}-{}", tx_id, step_id),
            name: "RollbackDestination".to_string(),
            description: "Rollback transaction on destination shard".to_string(),
            executing_shard: destination_shard_id.clone(),
            action: ExecutionAction::Rollback,
            status: ExecutionStepStatus::NotStarted,
            started_at: None,
            completed_at: None,
            result: None,
            error_message: None,
            retry_count: 0,
            max_retries: 3,
            dependencies: Vec::new(),
            compensation_step: None,
            metadata: None,
        });
        step_id += 1;

        // 送信元シャードでロールバック（補償）
        steps.push(ExecutionStep {
            id: format!("step-{}-{}", tx_id, step_id),
            name: "RollbackSource".to_string(),
            description: "Rollback transaction on source shard".to_string(),
            executing_shard: source_shard_id.clone(),
            action: ExecutionAction::Rollback,
            status: ExecutionStepStatus::NotStarted,
            started_at: None,
            completed_at: None,
            result: None,
            error_message: None,
            retry_count: 0,
            max_retries: 3,
            dependencies: Vec::new(),
            compensation_step: None,
            metadata: None,
        });
        step_id += 1;

        // 送信先シャードでロック解放
        steps.push(ExecutionStep {
            id: format!("step-{}-{}", tx_id, step_id),
            name: "ReleaseDestinationLock".to_string(),
            description: "Release lock on destination shard".to_string(),
            executing_shard: destination_shard_id.clone(),
            action: ExecutionAction::ReleaseLock,
            status: ExecutionStepStatus::NotStarted,
            started_at: None,
            completed_at: None,
            result: None,
            error_message: None,
            retry_count: 0,
            max_retries: 3,
            dependencies: vec![format!("step-{}-{}", tx_id, step_id - 3)],
            compensation_step: None,
            metadata: None,
        });
        step_id += 1;

        // 送信元シャードでロック解放
        steps.push(ExecutionStep {
            id: format!("step-{}-{}", tx_id, step_id),
            name: "ReleaseSourceLock".to_string(),
            description: "Release lock on source shard".to_string(),
            executing_shard: source_shard_id.clone(),
            action: ExecutionAction::ReleaseLock,
            status: ExecutionStepStatus::NotStarted,
            started_at: None,
            completed_at: None,
            result: None,
            error_message: None,
            retry_count: 0,
            max_retries: 3,
            dependencies: vec![format!("step-{}-{}", tx_id, step_id - 1)],
            compensation_step: None,
            metadata: None,
        });

        ExecutionPlan {
            id: plan_id,
            name: format!("Execution Plan for Transaction {}", tx_id),
            description: format!(
                "Cross-shard transaction from {} to {}",
                source_shard_id, destination_shard_id
            ),
            steps,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
            status: ExecutionPlanStatus::NotStarted,
            timeout_seconds: 300,
            metadata: None,
        }
    }

    /// トランザクションを実行
    pub fn execute_transaction(&mut self, tx_id: &str) -> Result<(), Error> {
        let transaction = self
            .transactions
            .get_mut(tx_id)
            .ok_or_else(|| Error::NotFound(format!("Transaction with ID {} not found", tx_id)))?;

        // 実行計画を取得
        let execution_plan = transaction
            .execution_plan
            .as_mut()
            .ok_or_else(|| Error::InvalidState("Execution plan not found".to_string()))?;

        // 実行計画の状態を更新
        let now = Utc::now();
        execution_plan.status = ExecutionPlanStatus::InProgress;
        execution_plan.started_at = Some(now);
        execution_plan.updated_at = now;

        // トランザクションの状態を更新
        transaction.advanced_state = AdvancedCrossShardTransactionState::SourceLocking;
        transaction.base_transaction.updated_at = now;

        // 監査ログを追加
        transaction.audit_log.push(AuditLogEntry {
            id: format!("audit-{}-{}", tx_id, now.timestamp()),
            timestamp: now,
            action: "EXECUTE".to_string(),
            actor: "system".to_string(),
            resource_type: "CrossShardTransaction".to_string(),
            resource_id: tx_id.to_string(),
            details: "Transaction execution started".to_string(),
            metadata: None,
        });

        // インデックスを更新
        if let Some(state_transactions) = self
            .state_indices
            .get_mut(&AdvancedCrossShardTransactionState::Preparing)
        {
            state_transactions.remove(tx_id);
        }

        let state_transactions = self
            .state_indices
            .entry(AdvancedCrossShardTransactionState::SourceLocking)
            .or_insert_with(HashSet::new);
        state_transactions.insert(tx_id.to_string());

        // 実行ステップを開始
        self.execute_next_step(tx_id)?;

        Ok(())
    }

    /// 次のステップを実行
    fn execute_next_step(&mut self, tx_id: &str) -> Result<(), Error> {
        let transaction = self
            .transactions
            .get_mut(tx_id)
            .ok_or_else(|| Error::NotFound(format!("Transaction with ID {} not found", tx_id)))?;

        // 実行計画を取得
        let execution_plan = transaction
            .execution_plan
            .as_mut()
            .ok_or_else(|| Error::InvalidState("Execution plan not found".to_string()))?;

        // 次の実行可能なステップを探す
        let mut next_step_index = None;

        for (i, step) in execution_plan.steps.iter().enumerate() {
            if step.status == ExecutionStepStatus::NotStarted {
                // 依存関係をチェック
                let mut dependencies_met = true;

                for dep_id in &step.dependencies {
                    let dep_step = execution_plan.steps.iter().find(|s| s.id == *dep_id);

                    if let Some(dep_step) = dep_step {
                        if dep_step.status != ExecutionStepStatus::Completed {
                            dependencies_met = false;
                            break;
                        }
                    } else {
                        dependencies_met = false;
                        break;
                    }
                }

                if dependencies_met {
                    next_step_index = Some(i);
                    break;
                }
            }
        }

        if let Some(step_index) = next_step_index {
            // ステップを実行
            let now = Utc::now();
            let step = &mut execution_plan.steps[step_index];
            step.status = ExecutionStepStatus::InProgress;
            step.started_at = Some(now);

            // 実行アクションに基づいて処理
            match step.action {
                ExecutionAction::Prepare => {
                    // 準備処理
                    let result = ExecutionResult {
                        success: true,
                        code: "SUCCESS".to_string(),
                        message: "Preparation completed successfully".to_string(),
                        details: None,
                        executed_at: now,
                        executed_by: "system".to_string(),
                        execution_time_ms: 10,
                        metadata: None,
                    };

                    step.status = ExecutionStepStatus::Completed;
                    step.completed_at = Some(now);
                    step.result = Some(result);

                    // パフォーマンス指標を更新
                    transaction.performance_metrics.preparation_time_ms = 10;

                    // 監査ログを追加
                    transaction.audit_log.push(AuditLogEntry {
                        id: format!("audit-{}-{}", tx_id, now.timestamp()),
                        timestamp: now,
                        action: "PREPARE".to_string(),
                        actor: "system".to_string(),
                        resource_type: "ExecutionStep".to_string(),
                        resource_id: step.id.clone(),
                        details: "Preparation step completed".to_string(),
                        metadata: None,
                    });
                }
                ExecutionAction::AcquireLock => {
                    // ロック取得処理
                    let shard_id = step.executing_shard.clone();
                    let resource_id = transaction.base_transaction.original_transaction.id.clone();
                    let resource_type = ResourceType::Transaction;

                    // ロックリクエストを作成
                    let lock_request = LockRequest {
                        id: format!("lock-req-{}-{}", tx_id, now.timestamp()),
                        transaction_id: tx_id.to_string(),
                        resource_type: resource_type.clone(),
                        resource_id: resource_id.clone(),
                        shard_id: shard_id.clone(),
                        lock_mode: LockMode::Exclusive,
                        requested_at: now,
                        timeout_at: now + Duration::seconds(60),
                        owner: "system".to_string(),
                    };

                    // ロックを取得
                    let lock_result = self.acquire_lock(lock_request);

                    if lock_result.is_ok() {
                        let lock = lock_result.unwrap();
                        transaction.locks.push(lock);

                        let result = ExecutionResult {
                            success: true,
                            code: "SUCCESS".to_string(),
                            message: format!("Lock acquired on shard {}", shard_id),
                            details: None,
                            executed_at: now,
                            executed_by: "system".to_string(),
                            execution_time_ms: 5,
                            metadata: None,
                        };

                        step.status = ExecutionStepStatus::Completed;
                        step.completed_at = Some(now);
                        step.result = Some(result);

                        // パフォーマンス指標を更新
                        transaction.performance_metrics.lock_acquisition_time_ms += 5;

                        // 監査ログを追加
                        transaction.audit_log.push(AuditLogEntry {
                            id: format!("audit-{}-{}", tx_id, now.timestamp()),
                            timestamp: now,
                            action: "ACQUIRE_LOCK".to_string(),
                            actor: "system".to_string(),
                            resource_type: "ExecutionStep".to_string(),
                            resource_id: step.id.clone(),
                            details: format!("Lock acquired on shard {}", shard_id),
                            metadata: None,
                        });

                        // シャードの状態を更新
                        if step.name == "AcquireSourceLock" {
                            transaction.advanced_state =
                                AdvancedCrossShardTransactionState::SourceLocked;
                            transaction
                                .shard_states
                                .insert(shard_id, ShardTransactionState::Locked);

                            // インデックスを更新
                            if let Some(state_transactions) = self
                                .state_indices
                                .get_mut(&AdvancedCrossShardTransactionState::SourceLocking)
                            {
                                state_transactions.remove(tx_id);
                            }

                            let state_transactions = self
                                .state_indices
                                .entry(AdvancedCrossShardTransactionState::SourceLocked)
                                .or_insert_with(HashSet::new);
                            state_transactions.insert(tx_id.to_string());
                        } else if step.name == "AcquireDestinationLock" {
                            transaction.advanced_state =
                                AdvancedCrossShardTransactionState::DestinationLocked;
                            transaction
                                .shard_states
                                .insert(shard_id, ShardTransactionState::Locked);

                            // インデックスを更新
                            if let Some(state_transactions) = self
                                .state_indices
                                .get_mut(&AdvancedCrossShardTransactionState::SourceLocked)
                            {
                                state_transactions.remove(tx_id);
                            }

                            let state_transactions = self
                                .state_indices
                                .entry(AdvancedCrossShardTransactionState::DestinationLocked)
                                .or_insert_with(HashSet::new);
                            state_transactions.insert(tx_id.to_string());
                        }
                    } else {
                        let error = lock_result.err().unwrap();

                        let result = ExecutionResult {
                            success: false,
                            code: "LOCK_FAILED".to_string(),
                            message: format!("Failed to acquire lock: {}", error),
                            details: None,
                            executed_at: now,
                            executed_by: "system".to_string(),
                            execution_time_ms: 5,
                            metadata: None,
                        };

                        step.status = ExecutionStepStatus::Failed;
                        step.completed_at = Some(now);
                        step.result = Some(result);
                        step.error_message = Some(error.to_string());

                        // 実行計画の状態を更新
                        execution_plan.status = ExecutionPlanStatus::Failed;

                        // トランザクションの状態を更新
                        transaction.advanced_state = AdvancedCrossShardTransactionState::Basic(
                            CrossShardTransactionState::Failed,
                        );
                        transaction.base_transaction.state = CrossShardTransactionState::Failed;

                        // 監査ログを追加
                        transaction.audit_log.push(AuditLogEntry {
                            id: format!("audit-{}-{}", tx_id, now.timestamp()),
                            timestamp: now,
                            action: "ACQUIRE_LOCK_FAILED".to_string(),
                            actor: "system".to_string(),
                            resource_type: "ExecutionStep".to_string(),
                            resource_id: step.id.clone(),
                            details: format!("Failed to acquire lock: {}", error),
                            metadata: None,
                        });

                        // エラー統計を更新
                        self.update_error_statistics("LOCK_FAILED", &shard_id);

                        // 補償処理を実行
                        self.execute_compensation(tx_id, step_index)?;

                        return Ok(());
                    }
                }
                ExecutionAction::Validate => {
                    // 検証処理
                    let shard_id = step.executing_shard.clone();

                    // 検証ステップを作成
                    let verification_step = AdvancedVerificationStep {
                        id: format!("verify-{}-{}", tx_id, now.timestamp()),
                        name: step.name.clone(),
                        description: step.description.clone(),
                        executing_shard: shard_id.clone(),
                        status: VerificationStepStatus::Completed,
                        started_at: Some(now),
                        completed_at: Some(now),
                        result: Some(AdvancedVerificationResult {
                            success: true,
                            code: "VALID".to_string(),
                            message: format!("Transaction validated on shard {}", shard_id),
                            details: None,
                            verified_at: now,
                            verified_by: "system".to_string(),
                            signature: None,
                        }),
                        error_message: None,
                        retry_count: 0,
                        dependencies: Vec::new(),
                        metadata: None,
                    };

                    transaction.validation_steps.push(verification_step);

                    let result = ExecutionResult {
                        success: true,
                        code: "SUCCESS".to_string(),
                        message: format!("Validation completed on shard {}", shard_id),
                        details: None,
                        executed_at: now,
                        executed_by: "system".to_string(),
                        execution_time_ms: 15,
                        metadata: None,
                    };

                    step.status = ExecutionStepStatus::Completed;
                    step.completed_at = Some(now);
                    step.result = Some(result);

                    // パフォーマンス指標を更新
                    transaction.performance_metrics.validation_time_ms += 15;

                    // 監査ログを追加
                    transaction.audit_log.push(AuditLogEntry {
                        id: format!("audit-{}-{}", tx_id, now.timestamp()),
                        timestamp: now,
                        action: "VALIDATE".to_string(),
                        actor: "system".to_string(),
                        resource_type: "ExecutionStep".to_string(),
                        resource_id: step.id.clone(),
                        details: format!("Validation completed on shard {}", shard_id),
                        metadata: None,
                    });

                    // トランザクションの状態を更新
                    if step.name == "ValidateDestination" {
                        transaction.advanced_state = AdvancedCrossShardTransactionState::Basic(
                            CrossShardTransactionState::DestinationVerified,
                        );
                        transaction.base_transaction.state =
                            CrossShardTransactionState::DestinationVerified;

                        // インデックスを更新
                        if let Some(state_transactions) = self
                            .state_indices
                            .get_mut(&AdvancedCrossShardTransactionState::DestinationLocked)
                        {
                            state_transactions.remove(tx_id);
                        }

                        let state_transactions = self
                            .state_indices
                            .entry(AdvancedCrossShardTransactionState::Basic(
                                CrossShardTransactionState::DestinationVerified,
                            ))
                            .or_insert_with(HashSet::new);
                        state_transactions.insert(tx_id.to_string());
                    }
                }
                ExecutionAction::Commit => {
                    // コミット処理
                    let shard_id = step.executing_shard.clone();

                    let result = ExecutionResult {
                        success: true,
                        code: "SUCCESS".to_string(),
                        message: format!("Transaction committed on shard {}", shard_id),
                        details: None,
                        executed_at: now,
                        executed_by: "system".to_string(),
                        execution_time_ms: 20,
                        metadata: None,
                    };

                    step.status = ExecutionStepStatus::Completed;
                    step.completed_at = Some(now);
                    step.result = Some(result);

                    // パフォーマンス指標を更新
                    transaction.performance_metrics.commit_time_ms += 20;

                    // 監査ログを追加
                    transaction.audit_log.push(AuditLogEntry {
                        id: format!("audit-{}-{}", tx_id, now.timestamp()),
                        timestamp: now,
                        action: "COMMIT".to_string(),
                        actor: "system".to_string(),
                        resource_type: "ExecutionStep".to_string(),
                        resource_id: step.id.clone(),
                        details: format!("Transaction committed on shard {}", shard_id),
                        metadata: None,
                    });

                    // シャードの状態を更新
                    transaction
                        .shard_states
                        .insert(shard_id.clone(), ShardTransactionState::Committed);

                    // トランザクションの状態を更新
                    if step.name == "CommitSource" {
                        transaction.advanced_state = AdvancedCrossShardTransactionState::Basic(
                            CrossShardTransactionState::SourceCommitted,
                        );
                        transaction.base_transaction.state =
                            CrossShardTransactionState::SourceCommitted;

                        // インデックスを更新
                        if let Some(state_transactions) =
                            self.state_indices
                                .get_mut(&AdvancedCrossShardTransactionState::Basic(
                                    CrossShardTransactionState::DestinationVerified,
                                ))
                        {
                            state_transactions.remove(tx_id);
                        }

                        let state_transactions = self
                            .state_indices
                            .entry(AdvancedCrossShardTransactionState::Basic(
                                CrossShardTransactionState::SourceCommitted,
                            ))
                            .or_insert_with(HashSet::new);
                        state_transactions.insert(tx_id.to_string());
                    } else if step.name == "CommitDestination" {
                        transaction.advanced_state = AdvancedCrossShardTransactionState::Basic(
                            CrossShardTransactionState::DestinationCommitted,
                        );
                        transaction.base_transaction.state =
                            CrossShardTransactionState::DestinationCommitted;

                        // インデックスを更新
                        if let Some(state_transactions) =
                            self.state_indices
                                .get_mut(&AdvancedCrossShardTransactionState::Basic(
                                    CrossShardTransactionState::SourceCommitted,
                                ))
                        {
                            state_transactions.remove(tx_id);
                        }

                        let state_transactions = self
                            .state_indices
                            .entry(AdvancedCrossShardTransactionState::Basic(
                                CrossShardTransactionState::DestinationCommitted,
                            ))
                            .or_insert_with(HashSet::new);
                        state_transactions.insert(tx_id.to_string());
                    }
                }
                ExecutionAction::ReleaseLock => {
                    // ロック解放処理
                    let shard_id = step.executing_shard.clone();

                    // ロックを探す
                    let lock_index = transaction
                        .locks
                        .iter()
                        .position(|lock| lock.shard_id == shard_id);

                    if let Some(index) = lock_index {
                        let lock = transaction.locks.remove(index);

                        // ロックを解放
                        self.release_lock(&lock.id)?;

                        let result = ExecutionResult {
                            success: true,
                            code: "SUCCESS".to_string(),
                            message: format!("Lock released on shard {}", shard_id),
                            details: None,
                            executed_at: now,
                            executed_by: "system".to_string(),
                            execution_time_ms: 5,
                            metadata: None,
                        };

                        step.status = ExecutionStepStatus::Completed;
                        step.completed_at = Some(now);
                        step.result = Some(result);

                        // 監査ログを追加
                        transaction.audit_log.push(AuditLogEntry {
                            id: format!("audit-{}-{}", tx_id, now.timestamp()),
                            timestamp: now,
                            action: "RELEASE_LOCK".to_string(),
                            actor: "system".to_string(),
                            resource_type: "ExecutionStep".to_string(),
                            resource_id: step.id.clone(),
                            details: format!("Lock released on shard {}", shard_id),
                            metadata: None,
                        });

                        // 最後のステップが完了したら、トランザクションを完了状態に更新
                        if step.name == "ReleaseSourceLock" {
                            transaction.advanced_state = AdvancedCrossShardTransactionState::Basic(
                                CrossShardTransactionState::Completed,
                            );
                            transaction.base_transaction.state =
                                CrossShardTransactionState::Completed;
                            transaction.base_transaction.completed_at = Some(now);

                            // 実行計画の状態を更新
                            execution_plan.status = ExecutionPlanStatus::Completed;
                            execution_plan.completed_at = Some(now);

                            // パフォーマンス指標を更新
                            transaction.performance_metrics.total_execution_time_ms =
                                (now - execution_plan.started_at.unwrap()).num_milliseconds()
                                    as u64;

                            // 監査ログを追加
                            transaction.audit_log.push(AuditLogEntry {
                                id: format!("audit-{}-{}", tx_id, now.timestamp()),
                                timestamp: now,
                                action: "COMPLETE".to_string(),
                                actor: "system".to_string(),
                                resource_type: "CrossShardTransaction".to_string(),
                                resource_id: tx_id.to_string(),
                                details: "Transaction completed successfully".to_string(),
                                metadata: None,
                            });

                            // インデックスを更新
                            if let Some(state_transactions) = self.state_indices.get_mut(
                                &AdvancedCrossShardTransactionState::Basic(
                                    CrossShardTransactionState::DestinationCommitted,
                                ),
                            ) {
                                state_transactions.remove(tx_id);
                            }

                            let state_transactions = self
                                .state_indices
                                .entry(AdvancedCrossShardTransactionState::Basic(
                                    CrossShardTransactionState::Completed,
                                ))
                                .or_insert_with(HashSet::new);
                            state_transactions.insert(tx_id.to_string());

                            // 統計を更新
                            self.update_statistics_for_completed_transaction(tx_id);

                            return Ok(());
                        }
                    } else {
                        // ロックが見つからない場合
                        let result = ExecutionResult {
                            success: false,
                            code: "LOCK_NOT_FOUND".to_string(),
                            message: format!("Lock not found for shard {}", shard_id),
                            details: None,
                            executed_at: now,
                            executed_by: "system".to_string(),
                            execution_time_ms: 5,
                            metadata: None,
                        };

                        step.status = ExecutionStepStatus::Failed;
                        step.completed_at = Some(now);
                        step.result = Some(result);
                        step.error_message = Some(format!("Lock not found for shard {}", shard_id));

                        // エラー統計を更新
                        self.update_error_statistics("LOCK_NOT_FOUND", &shard_id);
                    }
                }
                _ => {
                    // その他のアクション（簡易実装では省略）
                }
            }

            // 次のステップを実行
            self.execute_next_step(tx_id)?;
        } else {
            // 実行可能なステップがない場合
            let all_completed = execution_plan
                .steps
                .iter()
                .filter(|step| step.action != ExecutionAction::Rollback)
                .all(|step| step.status == ExecutionStepStatus::Completed);

            if all_completed {
                // すべてのステップが完了している場合
                execution_plan.status = ExecutionPlanStatus::Completed;
                execution_plan.completed_at = Some(Utc::now());
            } else {
                // 一部のステップが完了していない場合
                let has_failed = execution_plan
                    .steps
                    .iter()
                    .any(|step| step.status == ExecutionStepStatus::Failed);

                if has_failed {
                    // 失敗したステップがある場合
                    execution_plan.status = ExecutionPlanStatus::Failed;
                } else {
                    // 待機中のステップがある場合
                    execution_plan.status = ExecutionPlanStatus::Waiting;
                }
            }
        }

        Ok(())
    }

    /// 補償処理を実行
    fn execute_compensation(&mut self, tx_id: &str, failed_step_index: usize) -> Result<(), Error> {
        let transaction = self
            .transactions
            .get_mut(tx_id)
            .ok_or_else(|| Error::NotFound(format!("Transaction with ID {} not found", tx_id)))?;

        // 実行計画を取得
        let execution_plan = transaction
            .execution_plan
            .as_mut()
            .ok_or_else(|| Error::InvalidState("Execution plan not found".to_string()))?;

        // 失敗したステップの補償ステップを取得
        let failed_step = &execution_plan.steps[failed_step_index];

        if let Some(compensation_step_id) = &failed_step.compensation_step {
            // 補償ステップを探す
            let compensation_step_index = execution_plan
                .steps
                .iter()
                .position(|step| step.id == *compensation_step_id);

            if let Some(index) = compensation_step_index {
                // 補償ステップを実行
                let now = Utc::now();
                let step = &mut execution_plan.steps[index];
                step.status = ExecutionStepStatus::InProgress;
                step.started_at = Some(now);

                // 実行アクションに基づいて処理
                match step.action {
                    ExecutionAction::Rollback => {
                        // ロールバック処理
                        let shard_id = step.executing_shard.clone();

                        let result = ExecutionResult {
                            success: true,
                            code: "SUCCESS".to_string(),
                            message: format!("Transaction rolled back on shard {}", shard_id),
                            details: None,
                            executed_at: now,
                            executed_by: "system".to_string(),
                            execution_time_ms: 15,
                            metadata: None,
                        };

                        step.status = ExecutionStepStatus::Completed;
                        step.completed_at = Some(now);
                        step.result = Some(result);

                        // 監査ログを追加
                        transaction.audit_log.push(AuditLogEntry {
                            id: format!("audit-{}-{}", tx_id, now.timestamp()),
                            timestamp: now,
                            action: "ROLLBACK".to_string(),
                            actor: "system".to_string(),
                            resource_type: "ExecutionStep".to_string(),
                            resource_id: step.id.clone(),
                            details: format!("Transaction rolled back on shard {}", shard_id),
                            metadata: None,
                        });

                        // シャードの状態を更新
                        transaction
                            .shard_states
                            .insert(shard_id, ShardTransactionState::RolledBack);
                    }
                    _ => {
                        // その他のアクション（簡易実装では省略）
                    }
                }

                // 他の補償ステップを実行
                for (i, step) in execution_plan.steps.iter().enumerate() {
                    if step.action == ExecutionAction::Rollback && i != index {
                        self.execute_compensation(tx_id, i)?;
                    }
                }

                // 実行計画の状態を更新
                execution_plan.status = ExecutionPlanStatus::RolledBack;

                // トランザクションの状態を更新
                transaction.advanced_state = AdvancedCrossShardTransactionState::RolledBack;
                transaction.base_transaction.state = CrossShardTransactionState::Failed;

                // インデックスを更新
                let old_state = transaction.advanced_state.clone();
                if let Some(state_transactions) = self.state_indices.get_mut(&old_state) {
                    state_transactions.remove(tx_id);
                }

                let state_transactions = self
                    .state_indices
                    .entry(AdvancedCrossShardTransactionState::RolledBack)
                    .or_insert_with(HashSet::new);
                state_transactions.insert(tx_id.to_string());

                // 統計を更新
                self.update_statistics_for_failed_transaction(tx_id);
            }
        }

        Ok(())
    }

    /// ロックを取得
    fn acquire_lock(&mut self, request: LockRequest) -> Result<ResourceLock, Error> {
        let mut lock_manager = self.lock_manager.lock().unwrap();

        // リソースのロック状態を確認
        let resource_key = (request.resource_type.clone(), request.resource_id.clone());

        if let Some(lock_ids) = lock_manager.resource_locks.get(&resource_key) {
            // 既存のロックをチェック
            for lock_id in lock_ids {
                if let Some(lock) = lock_manager.active_locks.get(lock_id) {
                    // 排他ロックがある場合は待機
                    if lock.lock_mode == LockMode::Exclusive {
                        // 待機リストに追加
                        lock_manager.waiting_lock_requests.push_back(request);
                        return Err(Error::ResourceLocked(format!(
                            "Resource is locked exclusively"
                        )));
                    }
                }
            }
        }

        // ロックを作成
        let now = Utc::now();
        let lock_id = format!("lock-{}-{}", request.transaction_id, now.timestamp());

        let lock = ResourceLock {
            id: lock_id.clone(),
            resource_type: request.resource_type.clone(),
            resource_id: request.resource_id.clone(),
            shard_id: request.shard_id.clone(),
            lock_mode: request.lock_mode.clone(),
            acquired_at: now,
            released_at: None,
            timeout_at: request.timeout_at,
            owner: request.owner.clone(),
            metadata: None,
        };

        // ロックを保存
        lock_manager
            .active_locks
            .insert(lock_id.clone(), lock.clone());

        // リソースのロックリストを更新
        let locks = lock_manager
            .resource_locks
            .entry(resource_key)
            .or_insert_with(HashSet::new);
        locks.insert(lock_id);

        Ok(lock)
    }

    /// ロックを解放
    fn release_lock(&mut self, lock_id: &str) -> Result<(), Error> {
        let mut lock_manager = self.lock_manager.lock().unwrap();

        // ロックを取得
        let lock = lock_manager
            .active_locks
            .get_mut(lock_id)
            .ok_or_else(|| Error::NotFound(format!("Lock with ID {} not found", lock_id)))?;

        // ロックを解放
        let now = Utc::now();
        lock.released_at = Some(now);

        // リソースのロックリストを更新
        let resource_key = (lock.resource_type.clone(), lock.resource_id.clone());

        if let Some(locks) = lock_manager.resource_locks.get_mut(&resource_key) {
            locks.remove(lock_id);

            if locks.is_empty() {
                lock_manager.resource_locks.remove(&resource_key);
            }
        }

        // アクティブなロックから削除
        lock_manager.active_locks.remove(lock_id);

        // 待機中のリクエストを処理
        while let Some(request) = lock_manager.waiting_lock_requests.pop_front() {
            // リクエストのタイムアウトをチェック
            if now > request.timeout_at {
                continue;
            }

            // リソースのロック状態を再確認
            let resource_key = (request.resource_type.clone(), request.resource_id.clone());
            let can_acquire = if let Some(lock_ids) = lock_manager.resource_locks.get(&resource_key)
            {
                // 既存のロックをチェック
                !lock_ids.iter().any(|id| {
                    if let Some(lock) = lock_manager.active_locks.get(id) {
                        lock.lock_mode == LockMode::Exclusive
                    } else {
                        false
                    }
                })
            } else {
                true
            };

            if can_acquire {
                // ロックを作成
                let new_lock_id = format!("lock-{}-{}", request.transaction_id, now.timestamp());

                let lock = ResourceLock {
                    id: new_lock_id.clone(),
                    resource_type: request.resource_type.clone(),
                    resource_id: request.resource_id.clone(),
                    shard_id: request.shard_id.clone(),
                    lock_mode: request.lock_mode.clone(),
                    acquired_at: now,
                    released_at: None,
                    timeout_at: request.timeout_at,
                    owner: request.owner.clone(),
                    metadata: None,
                };

                // ロックを保存
                lock_manager.active_locks.insert(new_lock_id.clone(), lock);

                // リソースのロックリストを更新
                let locks = lock_manager
                    .resource_locks
                    .entry(resource_key)
                    .or_insert_with(HashSet::new);
                locks.insert(new_lock_id);

                break;
            } else {
                // 再度待機リストに追加
                lock_manager.waiting_lock_requests.push_back(request);
                break;
            }
        }

        Ok(())
    }

    /// 新しいトランザクションの統計を更新
    fn update_statistics_for_new_transaction(
        &mut self,
        tx_id: &str,
        source_shard_id: &ShardId,
        destination_shard_id: &ShardId,
        priority: &TransactionPriority,
    ) {
        // 基本統計を更新
        self.statistics.base_statistics.total_transactions += 1;
        self.statistics.base_statistics.in_progress_transactions += 1;

        // シャードごとの統計を更新
        for shard_id in [source_shard_id, destination_shard_id] {
            let shard_stats = self
                .statistics
                .per_shard_statistics
                .entry(shard_id.clone())
                .or_insert_with(|| ShardTransactionStatistics {
                    total_transactions: 0,
                    successful_transactions: 0,
                    failed_transactions: 0,
                    in_progress_transactions: 0,
                    average_processing_time_ms: 0,
                    max_processing_time_ms: 0,
                    min_processing_time_ms: 0,
                    metadata: None,
                });

            shard_stats.total_transactions += 1;
            shard_stats.in_progress_transactions += 1;
        }

        // 状態ごとの統計を更新
        let state_count = self
            .statistics
            .per_state_statistics
            .entry(AdvancedCrossShardTransactionState::Preparing)
            .or_insert(0);
        *state_count += 1;

        // 優先度ごとの統計を更新
        let priority_stats = self
            .statistics
            .per_priority_statistics
            .entry(priority.clone())
            .or_insert_with(|| PriorityStatistics {
                total_transactions: 0,
                successful_transactions: 0,
                failed_transactions: 0,
                average_processing_time_ms: 0,
                metadata: None,
            });

        priority_stats.total_transactions += 1;

        // 時間帯別統計を更新
        let now = Utc::now();
        let hour = now.hour() as u8;
        let day = now.weekday().num_days_from_monday() as u8;
        let month = now.month() as u8;

        let hour_count = self
            .statistics
            .time_based_statistics
            .transactions_by_hour
            .entry(hour)
            .or_insert(0);
        *hour_count += 1;

        let day_count = self
            .statistics
            .time_based_statistics
            .transactions_by_day
            .entry(day)
            .or_insert(0);
        *day_count += 1;

        let month_count = self
            .statistics
            .time_based_statistics
            .transactions_by_month
            .entry(month)
            .or_insert(0);
        *month_count += 1;

        // ピーク時間を更新
        if *hour_count
            > self
                .statistics
                .time_based_statistics
                .transactions_by_hour
                .get(&self.statistics.time_based_statistics.peak_hour)
                .unwrap_or(&0)
        {
            self.statistics.time_based_statistics.peak_hour = hour;
        }

        // ピーク日を更新
        if *day_count
            > self
                .statistics
                .time_based_statistics
                .transactions_by_day
                .get(&self.statistics.time_based_statistics.peak_day)
                .unwrap_or(&0)
        {
            self.statistics.time_based_statistics.peak_day = day;
        }
    }

    /// 完了したトランザクションの統計を更新
    fn update_statistics_for_completed_transaction(&mut self, tx_id: &str) {
        // トランザクションを取得
        if let Some(transaction) = self.transactions.get(tx_id) {
            // 基本統計を更新
            self.statistics.base_statistics.successful_transactions += 1;
            self.statistics.base_statistics.in_progress_transactions -= 1;

            // 完了時間を計算
            let start_time = transaction.base_transaction.created_at;
            let end_time = transaction
                .base_transaction
                .completed_at
                .unwrap_or(Utc::now());
            let completion_time_seconds = (end_time - start_time).num_seconds() as f64;

            // 平均完了時間を更新
            let total_successful = self.statistics.base_statistics.successful_transactions as f64;
            self.statistics
                .base_statistics
                .average_completion_time_seconds = (self
                .statistics
                .base_statistics
                .average_completion_time_seconds
                * (total_successful - 1.0)
                + completion_time_seconds)
                / total_successful;

            // シャードごとの統計を更新
            for shard_id in &transaction.involved_shards {
                if let Some(shard_stats) = self.statistics.per_shard_statistics.get_mut(shard_id) {
                    shard_stats.successful_transactions += 1;
                    shard_stats.in_progress_transactions -= 1;

                    // 処理時間を更新
                    let processing_time_ms =
                        transaction.performance_metrics.total_execution_time_ms;

                    // 平均処理時間を更新
                    let total_successful = shard_stats.successful_transactions as f64;
                    shard_stats.average_processing_time_ms =
                        ((shard_stats.average_processing_time_ms as f64 * (total_successful - 1.0)
                            + processing_time_ms as f64)
                            / total_successful) as u64;

                    // 最大処理時間を更新
                    shard_stats.max_processing_time_ms =
                        shard_stats.max_processing_time_ms.max(processing_time_ms);

                    // 最小処理時間を更新
                    if shard_stats.min_processing_time_ms == 0 {
                        shard_stats.min_processing_time_ms = processing_time_ms;
                    } else {
                        shard_stats.min_processing_time_ms =
                            shard_stats.min_processing_time_ms.min(processing_time_ms);
                    }
                }
            }

            // 状態ごとの統計を更新
            if let Some(state_count) = self
                .statistics
                .per_state_statistics
                .get_mut(&transaction.advanced_state)
            {
                *state_count -= 1;
            }

            let state_count = self
                .statistics
                .per_state_statistics
                .entry(AdvancedCrossShardTransactionState::Basic(
                    CrossShardTransactionState::Completed,
                ))
                .or_insert(0);
            *state_count += 1;

            // 優先度ごとの統計を更新
            if let Some(priority_stats) = self
                .statistics
                .per_priority_statistics
                .get_mut(&transaction.priority)
            {
                priority_stats.successful_transactions += 1;

                // 平均処理時間を更新
                let total_successful = priority_stats.successful_transactions as f64;
                priority_stats.average_processing_time_ms =
                    ((priority_stats.average_processing_time_ms as f64 * (total_successful - 1.0)
                        + transaction.performance_metrics.total_execution_time_ms as f64)
                        / total_successful) as u64;
            }

            // パフォーマンス統計を更新
            let total_successful = self.statistics.base_statistics.successful_transactions as f64;

            self.statistics
                .performance_statistics
                .average_preparation_time_ms = ((self
                .statistics
                .performance_statistics
                .average_preparation_time_ms
                as f64
                * (total_successful - 1.0)
                + transaction.performance_metrics.preparation_time_ms as f64)
                / total_successful) as u64;

            self.statistics
                .performance_statistics
                .average_lock_acquisition_time_ms = ((self
                .statistics
                .performance_statistics
                .average_lock_acquisition_time_ms
                as f64
                * (total_successful - 1.0)
                + transaction.performance_metrics.lock_acquisition_time_ms as f64)
                / total_successful) as u64;

            self.statistics
                .performance_statistics
                .average_validation_time_ms = ((self
                .statistics
                .performance_statistics
                .average_validation_time_ms as f64
                * (total_successful - 1.0)
                + transaction.performance_metrics.validation_time_ms as f64)
                / total_successful) as u64;

            self.statistics
                .performance_statistics
                .average_commit_time_ms = ((self
                .statistics
                .performance_statistics
                .average_commit_time_ms as f64
                * (total_successful - 1.0)
                + transaction.performance_metrics.commit_time_ms as f64)
                / total_successful) as u64;

            self.statistics
                .performance_statistics
                .average_total_execution_time_ms = ((self
                .statistics
                .performance_statistics
                .average_total_execution_time_ms
                as f64
                * (total_successful - 1.0)
                + transaction.performance_metrics.total_execution_time_ms as f64)
                / total_successful) as u64;

            self.statistics
                .performance_statistics
                .average_network_latency_ms = ((self
                .statistics
                .performance_statistics
                .average_network_latency_ms as f64
                * (total_successful - 1.0)
                + transaction.performance_metrics.network_latency_ms as f64)
                / total_successful) as u64;

            self.statistics
                .performance_statistics
                .average_inter_shard_communication_count = (self
                .statistics
                .performance_statistics
                .average_inter_shard_communication_count
                * (total_successful - 1.0)
                + transaction
                    .performance_metrics
                    .inter_shard_communication_count as f64)
                / total_successful;
        }
    }

    /// 失敗したトランザクションの統計を更新
    fn update_statistics_for_failed_transaction(&mut self, tx_id: &str) {
        // トランザクションを取得
        if let Some(transaction) = self.transactions.get(tx_id) {
            // 基本統計を更新
            self.statistics.base_statistics.failed_transactions += 1;
            self.statistics.base_statistics.in_progress_transactions -= 1;

            // シャードごとの統計を更新
            for shard_id in &transaction.involved_shards {
                if let Some(shard_stats) = self.statistics.per_shard_statistics.get_mut(shard_id) {
                    shard_stats.failed_transactions += 1;
                    shard_stats.in_progress_transactions -= 1;
                }
            }

            // 状態ごとの統計を更新
            if let Some(state_count) = self
                .statistics
                .per_state_statistics
                .get_mut(&transaction.advanced_state)
            {
                *state_count -= 1;
            }

            let state_count = self
                .statistics
                .per_state_statistics
                .entry(AdvancedCrossShardTransactionState::Basic(
                    CrossShardTransactionState::Failed,
                ))
                .or_insert(0);
            *state_count += 1;

            // 優先度ごとの統計を更新
            if let Some(priority_stats) = self
                .statistics
                .per_priority_statistics
                .get_mut(&transaction.priority)
            {
                priority_stats.failed_transactions += 1;
            }
        }
    }

    /// エラー統計を更新
    fn update_error_statistics(&mut self, error_type: &str, shard_id: &ShardId) {
        // 総エラー数を更新
        self.statistics.error_statistics.total_errors += 1;

        // エラータイプごとの数を更新
        let type_count = self
            .statistics
            .error_statistics
            .errors_by_type
            .entry(error_type.to_string())
            .or_insert(0);
        *type_count += 1;

        // シャードごとのエラー数を更新
        let shard_count = self
            .statistics
            .error_statistics
            .errors_by_shard
            .entry(shard_id.clone())
            .or_insert(0);
        *shard_count += 1;

        // 最も一般的なエラーを更新
        if *type_count > self.statistics.error_statistics.most_common_error_count {
            self.statistics.error_statistics.most_common_error = Some(error_type.to_string());
            self.statistics.error_statistics.most_common_error_count = *type_count;
        }
    }

    /// トランザクションを取得
    pub fn get_transaction(&self, tx_id: &str) -> Option<&AdvancedCrossShardTransaction> {
        self.transactions.get(tx_id)
    }

    /// シャードのトランザクションを取得
    pub fn get_shard_transactions(
        &self,
        shard_id: &ShardId,
    ) -> Vec<&AdvancedCrossShardTransaction> {
        if let Some(tx_ids) = self.shard_indices.get(shard_id) {
            tx_ids
                .iter()
                .filter_map(|id| self.transactions.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 状態のトランザクションを取得
    pub fn get_state_transactions(
        &self,
        state: &AdvancedCrossShardTransactionState,
    ) -> Vec<&AdvancedCrossShardTransaction> {
        if let Some(tx_ids) = self.state_indices.get(state) {
            tx_ids
                .iter()
                .filter_map(|id| self.transactions.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 優先度のトランザクションを取得
    pub fn get_priority_transactions(
        &self,
        priority: &TransactionPriority,
    ) -> Vec<&AdvancedCrossShardTransaction> {
        if let Some(tx_ids) = self.priority_indices.get(priority) {
            tx_ids
                .iter()
                .filter_map(|id| self.transactions.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 統計を取得
    pub fn get_statistics(&self) -> &AdvancedCrossShardTransactionStatistics {
        &self.statistics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_transaction() {
        // マネージャーを作成
        let mut manager = AdvancedCrossShardTransactionManager::new();

        // トランザクションを作成
        let transaction = Transaction {
            id: "tx1".to_string(),
            sender: "addr1".to_string(),
            receiver: "addr2".to_string(),
            amount: 100,
            fee: 10,
            timestamp: Utc::now().timestamp(),
            signature: None,
            status: TransactionStatus::Pending,
            data: None,
        };

        let source_shard_id = ShardId::new(1);
        let destination_shard_id = ShardId::new(2);
        let involved_shards = vec![source_shard_id.clone(), destination_shard_id.clone()];
        let priority = TransactionPriority::High;
        let dependencies = Vec::new();

        let result = manager.create_transaction(
            transaction,
            source_shard_id,
            destination_shard_id,
            involved_shards,
            priority,
            dependencies,
        );

        assert!(result.is_ok());
        let tx_id = result.unwrap();

        // トランザクションを取得
        let tx = manager.get_transaction(&tx_id);
        assert!(tx.is_some());

        let tx = tx.unwrap();
        assert_eq!(
            tx.advanced_state,
            AdvancedCrossShardTransactionState::Preparing
        );
        assert_eq!(tx.involved_shards.len(), 2);
        assert_eq!(tx.priority, TransactionPriority::High);

        // 実行計画を確認
        assert!(tx.execution_plan.is_some());
        let plan = tx.execution_plan.as_ref().unwrap();
        assert_eq!(plan.status, ExecutionPlanStatus::NotStarted);
        assert!(!plan.steps.is_empty());
    }
}
