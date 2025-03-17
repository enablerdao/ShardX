use crate::error::Error;
use crate::parallel::config::{EngineConfig, ExecutionMode};
use crate::parallel::executor::ExecutionUnit;
use crate::parallel::scheduler::{SchedulingPlan, TaskScheduler};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use uuid::Uuid;

/// 並列エンジン
#[derive(Clone)]
pub struct ParallelEngine {
    /// 設定
    config: EngineConfig,
    /// 実行コンテキスト
    contexts: Arc<RwLock<HashMap<String, ExecutionContext>>>,
    /// 実行キュー
    execution_queue: Arc<Mutex<Vec<String>>>,
    /// 実行中フラグ
    running: Arc<RwLock<bool>>,
}

/// 実行コンテキスト
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// コンテキストID
    pub id: String,
    /// 名前
    pub name: String,
    /// 説明
    pub description: String,
    /// 実行モード
    pub execution_mode: ExecutionMode,
    /// 並列度
    pub parallelism: u32,
    /// 優先度
    pub priority: u32,
    /// ステータス
    pub status: ExecutionStatus,
    /// ユニットID
    pub units: Vec<String>,
    /// 依存関係
    pub dependencies: HashMap<String, DependencyType>,
    /// 作成時間
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 開始時間
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 完了時間
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 統計
    pub stats: Option<ExecutionStats>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// 実行ステータス
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// 作成済み
    Created,
    /// 割り当て済み
    Assigned,
    /// 実行中
    Running,
    /// 停止中
    Stopping,
    /// 完了
    Completed,
    /// 失敗
    Failed,
    /// 停止
    Stopped,
}

/// 依存関係タイプ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    /// データ依存
    Data,
    /// 制御依存
    Control,
    /// リソース依存
    Resource,
    /// カスタム
    Custom(String),
}

/// 実行統計
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionStats {
    /// 総ユニット数
    pub total_units: usize,
    /// 完了ユニット数
    pub completed_units: usize,
    /// 失敗ユニット数
    pub failed_units: usize,
    /// 総実行時間（ミリ秒）
    pub total_execution_time_ms: u64,
    /// 平均実行時間（ミリ秒）
    pub avg_execution_time_ms: f64,
    /// 最大実行時間（ミリ秒）
    pub max_execution_time_ms: u64,
    /// 総時間（ミリ秒）
    pub total_time_ms: u64,
    /// 並列度
    pub parallelism: u32,
    /// 実効並列度
    pub effective_parallelism: f64,
}

impl ParallelEngine {
    /// 新しいParallelEngineを作成
    pub fn new(config: EngineConfig) -> Self {
        Self {
            config,
            contexts: Arc::new(RwLock::new(HashMap::new())),
            execution_queue: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// エンジンを開始
    pub async fn start(&self) -> Result<(), Error> {
        let mut running = self.running.write().unwrap();
        if *running {
            return Err(Error::InvalidState(
                "Parallel engine is already running".to_string(),
            ));
        }

        *running = true;

        // バックグラウンドタスクを開始
        self.start_background_tasks();

        info!("Parallel engine started");

        Ok(())
    }

    /// エンジンを停止
    pub async fn stop(&self) -> Result<(), Error> {
        let mut running = self.running.write().unwrap();
        if !*running {
            return Err(Error::InvalidState(
                "Parallel engine is not running".to_string(),
            ));
        }

        *running = false;

        info!("Parallel engine stopped");

        Ok(())
    }

    /// バックグラウンドタスクを開始
    fn start_background_tasks(&self) {
        let execution_queue = self.execution_queue.clone();
        let contexts = self.contexts.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(100));

            loop {
                interval.tick().await;

                let is_running = *running.read().unwrap();
                if !is_running {
                    break;
                }

                // 実行キューからコンテキストを取得
                let context_id = {
                    let mut queue = execution_queue.lock().unwrap();
                    if queue.is_empty() {
                        continue;
                    }
                    queue.remove(0)
                };

                // コンテキストを実行
                let mut contexts_map = contexts.write().unwrap();
                if let Some(context) = contexts_map.get_mut(&context_id) {
                    if context.status == ExecutionStatus::Assigned {
                        context.status = ExecutionStatus::Running;
                        context.started_at = Some(chrono::Utc::now());

                        debug!("Started execution of context: {}", context_id);
                    }
                }
            }
        });
    }

    /// 実行計画を送信
    pub async fn submit_execution_plan(&self, plan: SchedulingPlan) -> Result<(), Error> {
        // コンテキストIDを取得
        let context_id = plan.context_id.clone();

        // コンテキストを取得
        let mut contexts = self.contexts.write().unwrap();
        let context = contexts.get_mut(&context_id).ok_or_else(|| {
            Error::NotFound(format!("Execution context not found: {}", context_id))
        })?;

        // コンテキストのステータスを更新
        context.status = ExecutionStatus::Assigned;

        // 実行キューに追加
        let mut queue = self.execution_queue.lock().unwrap();
        queue.push(context_id.clone());

        info!("Submitted execution plan for context: {}", context_id);

        Ok(())
    }

    /// コンテキストを登録
    pub fn register_context(&self, context: ExecutionContext) -> Result<(), Error> {
        let mut contexts = self.contexts.write().unwrap();

        // コンテキストが既に存在するかチェック
        if contexts.contains_key(&context.id) {
            return Err(Error::AlreadyExists(format!(
                "Execution context already exists: {}",
                context.id
            )));
        }

        // コンテキストを登録
        contexts.insert(context.id.clone(), context);

        Ok(())
    }

    /// コンテキストを取得
    pub fn get_context(&self, context_id: &str) -> Result<ExecutionContext, Error> {
        let contexts = self.contexts.read().unwrap();

        // コンテキストを取得
        let context = contexts.get(context_id).ok_or_else(|| {
            Error::NotFound(format!("Execution context not found: {}", context_id))
        })?;

        Ok(context.clone())
    }

    /// コンテキストを更新
    pub fn update_context(&self, context: ExecutionContext) -> Result<(), Error> {
        let mut contexts = self.contexts.write().unwrap();

        // コンテキストが存在するかチェック
        if !contexts.contains_key(&context.id) {
            return Err(Error::NotFound(format!(
                "Execution context not found: {}",
                context.id
            )));
        }

        // コンテキストを更新
        contexts.insert(context.id.clone(), context);

        Ok(())
    }

    /// コンテキストを削除
    pub fn remove_context(&self, context_id: &str) -> Result<(), Error> {
        let mut contexts = self.contexts.write().unwrap();

        // コンテキストが存在するかチェック
        if !contexts.contains_key(context_id) {
            return Err(Error::NotFound(format!(
                "Execution context not found: {}",
                context_id
            )));
        }

        // コンテキストを削除
        contexts.remove(context_id);

        Ok(())
    }

    /// すべてのコンテキストIDを取得
    pub fn get_all_context_ids(&self) -> Vec<String> {
        let contexts = self.contexts.read().unwrap();
        contexts.keys().cloned().collect()
    }

    /// 実行中のコンテキスト数を取得
    pub fn get_running_context_count(&self) -> usize {
        let contexts = self.contexts.read().unwrap();
        contexts
            .values()
            .filter(|c| c.status == ExecutionStatus::Running)
            .count()
    }

    /// 設定を取得
    pub fn get_config(&self) -> EngineConfig {
        self.config.clone()
    }

    /// 設定を更新
    pub fn update_config(&mut self, config: EngineConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_engine_creation() {
        let config = EngineConfig::default();
        let engine = ParallelEngine::new(config.clone());

        assert_eq!(engine.get_config().max_parallelism, config.max_parallelism);
        assert_eq!(
            engine.get_config().default_parallelism,
            config.default_parallelism
        );
        assert_eq!(engine.get_config().execution_mode, config.execution_mode);
    }

    #[tokio::test]
    async fn test_register_and_get_context() {
        let config = EngineConfig::default();
        let engine = ParallelEngine::new(config);

        let context_id = format!("ctx-{}", Uuid::new_v4());
        let context = ExecutionContext {
            id: context_id.clone(),
            name: "Test Context".to_string(),
            description: "Test execution context".to_string(),
            execution_mode: ExecutionMode::Parallel,
            parallelism: 4,
            priority: 1,
            status: ExecutionStatus::Created,
            units: Vec::new(),
            dependencies: HashMap::new(),
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            stats: None,
            metadata: HashMap::new(),
        };

        // コンテキストを登録
        let result = engine.register_context(context.clone());
        assert!(result.is_ok());

        // コンテキストを取得
        let retrieved_context = engine.get_context(&context_id);
        assert!(retrieved_context.is_ok());

        let retrieved = retrieved_context.unwrap();
        assert_eq!(retrieved.id, context_id);
        assert_eq!(retrieved.name, "Test Context");
        assert_eq!(retrieved.execution_mode, ExecutionMode::Parallel);
        assert_eq!(retrieved.parallelism, 4);
    }

    #[tokio::test]
    async fn test_update_context() {
        let config = EngineConfig::default();
        let engine = ParallelEngine::new(config);

        let context_id = format!("ctx-{}", Uuid::new_v4());
        let mut context = ExecutionContext {
            id: context_id.clone(),
            name: "Test Context".to_string(),
            description: "Test execution context".to_string(),
            execution_mode: ExecutionMode::Parallel,
            parallelism: 4,
            priority: 1,
            status: ExecutionStatus::Created,
            units: Vec::new(),
            dependencies: HashMap::new(),
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            stats: None,
            metadata: HashMap::new(),
        };

        // コンテキストを登録
        let result = engine.register_context(context.clone());
        assert!(result.is_ok());

        // コンテキストを更新
        context.name = "Updated Context".to_string();
        context.parallelism = 8;

        let update_result = engine.update_context(context.clone());
        assert!(update_result.is_ok());

        // 更新されたコンテキストを取得
        let retrieved_context = engine.get_context(&context_id);
        assert!(retrieved_context.is_ok());

        let retrieved = retrieved_context.unwrap();
        assert_eq!(retrieved.name, "Updated Context");
        assert_eq!(retrieved.parallelism, 8);
    }

    #[tokio::test]
    async fn test_remove_context() {
        let config = EngineConfig::default();
        let engine = ParallelEngine::new(config);

        let context_id = format!("ctx-{}", Uuid::new_v4());
        let context = ExecutionContext {
            id: context_id.clone(),
            name: "Test Context".to_string(),
            description: "Test execution context".to_string(),
            execution_mode: ExecutionMode::Parallel,
            parallelism: 4,
            priority: 1,
            status: ExecutionStatus::Created,
            units: Vec::new(),
            dependencies: HashMap::new(),
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            stats: None,
            metadata: HashMap::new(),
        };

        // コンテキストを登録
        let result = engine.register_context(context.clone());
        assert!(result.is_ok());

        // コンテキストを削除
        let remove_result = engine.remove_context(&context_id);
        assert!(remove_result.is_ok());

        // 削除されたコンテキストを取得
        let retrieved_context = engine.get_context(&context_id);
        assert!(retrieved_context.is_err());
    }
}
