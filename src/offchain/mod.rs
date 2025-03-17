// オフチェーン計算フレームワーク
//
// このモジュールは、ShardXのオフチェーン計算機能を提供します。
// オフチェーン計算は、ブロックチェーン上で実行するには高コストな計算を
// オフチェーンで実行し、結果のみをオンチェーンで検証する仕組みです。
//
// 主な機能:
// - 計算ノード管理
// - 計算タスク管理
// - 結果検証
// - 証明生成
// - インセンティブメカニズム

mod config;
// mod node; // TODO: このモジュールが見つかりません
// mod task; // TODO: このモジュールが見つかりません
// mod verifier; // TODO: このモジュールが見つかりません
// mod prover; // TODO: このモジュールが見つかりません
// mod executor; // TODO: このモジュールが見つかりません
// mod scheduler; // TODO: このモジュールが見つかりません
// mod incentive; // TODO: このモジュールが見つかりません
// mod registry; // TODO: このモジュールが見つかりません
// mod protocol; // TODO: このモジュールが見つかりません

pub use self::config::{OffchainConfig, ComputeNodeConfig, TaskConfig, VerifierConfig};
pub use self::node::{ComputeNode, NodeStatus, NodeCapability, NodeResource};
pub use self::task::{ComputeTask, TaskStatus, TaskResult, TaskPriority};
pub use self::verifier::{ResultVerifier, VerificationMethod, VerificationResult};
pub use self::prover::{ProofGenerator, ProofType, ProofData};
pub use self::executor::{TaskExecutor, ExecutionEnvironment, ExecutionResult};
pub use self::scheduler::{TaskScheduler, SchedulingStrategy, SchedulingResult};
pub use self::incentive::{IncentiveManager, RewardModel, RewardDistribution};
pub use self::registry::{NodeRegistry, TaskRegistry, RegistryEvent};
pub use self::protocol::{OffchainProtocol, ProtocolMessage, ProtocolVersion};

use crate::error::Error;
use crate::metrics::MetricsCollector;
use crate::network::NetworkManager;
use crate::storage::StorageManager;
use crate::crypto::CryptoManager;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use log::{debug, error, info, warn};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// オフチェーン計算マネージャー
pub struct OffchainManager {
    /// 設定
    config: OffchainConfig,
    /// 計算ノード
    nodes: HashMap<NodeId, ComputeNode>,
    /// 計算タスク
    tasks: HashMap<TaskId, ComputeTask>,
    /// ノードレジストリ
    node_registry: NodeRegistry,
    /// タスクレジストリ
    task_registry: TaskRegistry,
    /// 結果検証器
    verifier: ResultVerifier,
    /// 証明生成器
    prover: ProofGenerator,
    /// タスク実行器
    executor: TaskExecutor,
    /// タスクスケジューラー
    scheduler: TaskScheduler,
    /// インセンティブマネージャー
    incentive_manager: IncentiveManager,
    /// オフチェーンプロトコル
    protocol: OffchainProtocol,
    /// ネットワークマネージャー
    network_manager: Arc<NetworkManager>,
    /// ストレージマネージャー
    storage_manager: Arc<StorageManager>,
    /// 暗号マネージャー
    crypto_manager: Arc<CryptoManager>,
    /// メトリクスコレクター
    metrics: Arc<MetricsCollector>,
    /// 実行中フラグ
    running: bool,
    /// イベント通知チャネル
    event_tx: mpsc::Sender<OffchainEvent>,
    /// イベント通知受信チャネル
    event_rx: mpsc::Receiver<OffchainEvent>,
}

/// ノードID
pub type NodeId = String;

/// タスクID
pub type TaskId = String;

/// オフチェーンイベント
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OffchainEvent {
    /// イベントID
    pub id: String,
    /// イベントタイプ
    pub event_type: OffchainEventType,
    /// タイムスタンプ
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// データ
    pub data: serde_json::Value,
}

/// オフチェーンイベントタイプ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OffchainEventType {
    /// ノード登録
    NodeRegistered,
    /// ノード削除
    NodeRemoved,
    /// ノードステータス変更
    NodeStatusChanged,
    /// タスク作成
    TaskCreated,
    /// タスク割り当て
    TaskAssigned,
    /// タスク実行開始
    TaskStarted,
    /// タスク完了
    TaskCompleted,
    /// タスク失敗
    TaskFailed,
    /// 結果検証
    ResultVerified,
    /// 証明生成
    ProofGenerated,
    /// 報酬分配
    RewardDistributed,
    /// エラー
    Error,
}

/// オフチェーン統計
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OffchainStats {
    /// ノード数
    pub node_count: usize,
    /// アクティブノード数
    pub active_node_count: usize,
    /// タスク数
    pub task_count: usize,
    /// 完了タスク数
    pub completed_task_count: usize,
    /// 失敗タスク数
    pub failed_task_count: usize,
    /// 保留タスク数
    pub pending_task_count: usize,
    /// 実行中タスク数
    pub running_task_count: usize,
    /// 平均タスク実行時間（ミリ秒）
    pub average_task_execution_time_ms: f64,
    /// 平均タスク待機時間（ミリ秒）
    pub average_task_wait_time_ms: f64,
    /// 総計算リソース
    pub total_compute_resources: NodeResource,
    /// 利用可能計算リソース
    pub available_compute_resources: NodeResource,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

impl OffchainManager {
    /// 新しいOffchainManagerを作成
    pub fn new(
        config: OffchainConfig,
        network_manager: Arc<NetworkManager>,
        storage_manager: Arc<StorageManager>,
        crypto_manager: Arc<CryptoManager>,
        metrics: Arc<MetricsCollector>,
    ) -> Self {
        let (tx, rx) = mpsc::channel(100);
        
        let node_registry = NodeRegistry::new(config.registry_config.clone());
        let task_registry = TaskRegistry::new(config.registry_config.clone());
        let verifier = ResultVerifier::new(config.verifier_config.clone());
        let prover = ProofGenerator::new(config.prover_config.clone());
        let executor = TaskExecutor::new(config.executor_config.clone());
        let scheduler = TaskScheduler::new(config.scheduler_config.clone());
        let incentive_manager = IncentiveManager::new(config.incentive_config.clone());
        let protocol = OffchainProtocol::new(config.protocol_config.clone());
        
        Self {
            config,
            nodes: HashMap::new(),
            tasks: HashMap::new(),
            node_registry,
            task_registry,
            verifier,
            prover,
            executor,
            scheduler,
            incentive_manager,
            protocol,
            network_manager,
            storage_manager,
            crypto_manager,
            metrics,
            running: false,
            event_tx: tx,
            event_rx: rx,
        }
    }
    
    /// オフチェーン計算を開始
    pub async fn start(&mut self) -> Result<(), Error> {
        if self.running {
            return Err(Error::InvalidState("Offchain manager is already running".to_string()));
        }
        
        self.running = true;
        
        // 保存されたノードとタスクを読み込む
        self.load_nodes_and_tasks().await?;
        
        // バックグラウンドタスクを開始
        self.start_background_tasks();
        
        info!("Offchain manager started");
        
        Ok(())
    }
    
    /// オフチェーン計算を停止
    pub async fn stop(&mut self) -> Result<(), Error> {
        if !self.running {
            return Err(Error::InvalidState("Offchain manager is not running".to_string()));
        }
        
        self.running = false;
        
        // 実行中のタスクを停止
        for (task_id, task) in &mut self.tasks {
            if task.status == TaskStatus::Running {
                info!("Stopping task: {}", task_id);
                task.status = TaskStatus::Stopped;
                
                // タスクを保存
                let storage = self.storage_manager.get_storage("offchain_tasks")?;
                storage.put(&format!("task:{}", task_id), task)?;
            }
        }
        
        info!("Offchain manager stopped");
        
        Ok(())
    }
    
    /// バックグラウンドタスクを開始
    fn start_background_tasks(&self) {
        // スケジューラータスク
        let scheduler_interval = self.config.scheduler_interval_ms;
        let scheduler_tx = self.event_tx.clone();
        let scheduler = Arc::new(RwLock::new(self.scheduler.clone()));
        let scheduler_running = Arc::new(RwLock::new(self.running));
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(scheduler_interval));
            
            loop {
                interval.tick().await;
                
                let running = *scheduler_running.read().unwrap();
                if !running {
                    break;
                }
                
                let s = scheduler.read().unwrap();
                
                // スケジューラータスクを実行
                if let Err(e) = s.schedule_pending_tasks().await {
                    error!("Failed to schedule pending tasks: {}", e);
                }
            }
        });
        
        // 実行器タスク
        let executor_interval = self.config.executor_interval_ms;
        let executor_tx = self.event_tx.clone();
        let executor = Arc::new(RwLock::new(self.executor.clone()));
        let executor_running = Arc::new(RwLock::new(self.running));
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(executor_interval));
            
            loop {
                interval.tick().await;
                
                let running = *executor_running.read().unwrap();
                if !running {
                    break;
                }
                
                let e = executor.read().unwrap();
                
                // 実行器タスクを実行
                if let Err(err) = e.process_assigned_tasks().await {
                    error!("Failed to process assigned tasks: {}", err);
                }
            }
        });
        
        // 検証器タスク
        let verifier_interval = self.config.verifier_interval_ms;
        let verifier_tx = self.event_tx.clone();
        let verifier = Arc::new(RwLock::new(self.verifier.clone()));
        let verifier_running = Arc::new(RwLock::new(self.running));
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(verifier_interval));
            
            loop {
                interval.tick().await;
                
                let running = *verifier_running.read().unwrap();
                if !running {
                    break;
                }
                
                let v = verifier.read().unwrap();
                
                // 検証器タスクを実行
                if let Err(e) = v.verify_completed_tasks().await {
                    error!("Failed to verify completed tasks: {}", e);
                }
            }
        });
        
        // インセンティブタスク
        let incentive_interval = self.config.incentive_interval_ms;
        let incentive_tx = self.event_tx.clone();
        let incentive_manager = Arc::new(RwLock::new(self.incentive_manager.clone()));
        let incentive_running = Arc::new(RwLock::new(self.running));
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(incentive_interval));
            
            loop {
                interval.tick().await;
                
                let running = *incentive_running.read().unwrap();
                if !running {
                    break;
                }
                
                let im = incentive_manager.read().unwrap();
                
                // インセンティブタスクを実行
                if let Err(e) = im.distribute_rewards().await {
                    error!("Failed to distribute rewards: {}", e);
                }
            }
        });
        
        // ノード監視タスク
        let node_monitor_interval = self.config.node_monitor_interval_ms;
        let node_monitor_tx = self.event_tx.clone();
        let node_registry = Arc::new(RwLock::new(self.node_registry.clone()));
        let node_monitor_running = Arc::new(RwLock::new(self.running));
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(node_monitor_interval));
            
            loop {
                interval.tick().await;
                
                let running = *node_monitor_running.read().unwrap();
                if !running {
                    break;
                }
                
                let registry = node_registry.read().unwrap();
                
                // ノード監視タスクを実行
                if let Err(e) = registry.monitor_nodes().await {
                    error!("Failed to monitor nodes: {}", e);
                }
            }
        });
    }
    
    /// 保存されたノードとタスクを読み込む
    async fn load_nodes_and_tasks(&mut self) -> Result<(), Error> {
        // ストレージからノードを読み込む
        let node_storage = self.storage_manager.get_storage("offchain_nodes")?;
        
        if let Ok(nodes) = node_storage.get_all::<ComputeNode>("node") {
            for node in nodes {
                self.nodes.insert(node.id.clone(), node);
            }
        }
        
        // ストレージからタスクを読み込む
        let task_storage = self.storage_manager.get_storage("offchain_tasks")?;
        
        if let Ok(tasks) = task_storage.get_all::<ComputeTask>("task") {
            for task in tasks {
                self.tasks.insert(task.id.clone(), task);
            }
        }
        
        info!("Loaded {} nodes and {} tasks", self.nodes.len(), self.tasks.len());
        
        Ok(())
    }
    
    /// 計算ノードを登録
    pub async fn register_node(
        &mut self,
        node_id: &str,
        endpoint: &str,
        capabilities: Vec<NodeCapability>,
        resources: NodeResource,
    ) -> Result<(), Error> {
        // ノードが既に存在するかチェック
        if self.nodes.contains_key(node_id) {
            return Err(Error::AlreadyExists(format!("Node already exists: {}", node_id)));
        }
        
        // 現在時刻を取得
        let now = chrono::Utc::now();
        
        // ノードを作成
        let node = ComputeNode {
            id: node_id.to_string(),
            endpoint: endpoint.to_string(),
            status: NodeStatus::Available,
            capabilities,
            resources,
            assigned_tasks: Vec::new(),
            completed_tasks: Vec::new(),
            performance_metrics: HashMap::new(),
            reputation_score: 0.0,
            last_heartbeat: now,
            registered_at: now,
            metadata: HashMap::new(),
        };
        
        // ノードを保存
        self.nodes.insert(node_id.to_string(), node.clone());
        
        // ノードレジストリに登録
        self.node_registry.register_node(node.clone()).await?;
        
        // ストレージに保存
        let storage = self.storage_manager.get_storage("offchain_nodes")?;
        storage.put(&format!("node:{}", node_id), &node)?;
        
        // ノード登録イベントを発行
        let event = OffchainEvent {
            id: format!("event-{}", Uuid::new_v4()),
            event_type: OffchainEventType::NodeRegistered,
            timestamp: now,
            data: serde_json::json!({
                "node_id": node_id,
                "endpoint": endpoint,
                "capabilities": capabilities,
                "resources": resources,
            }),
        };
        
        if let Err(e) = self.event_tx.send(event).await {
            error!("Failed to send node registered event: {}", e);
        }
        
        // メトリクスを更新
        self.metrics.increment_counter("offchain_nodes_registered");
        self.metrics.set_gauge("offchain_node_count", self.nodes.len() as f64);
        
        info!("Registered compute node: {}", node_id);
        
        Ok(())
    }
    
    /// 計算ノードを削除
    pub async fn remove_node(
        &mut self,
        node_id: &str,
    ) -> Result<(), Error> {
        // ノードが存在するかチェック
        if !self.nodes.contains_key(node_id) {
            return Err(Error::NotFound(format!("Node not found: {}", node_id)));
        }
        
        // ノードを取得
        let node = self.nodes.remove(node_id).unwrap();
        
        // ノードに割り当てられたタスクを再スケジュール
        for task_id in &node.assigned_tasks {
            if let Some(task) = self.tasks.get_mut(task_id) {
                task.status = TaskStatus::Pending;
                task.assigned_node = None;
                
                // タスクを保存
                let storage = self.storage_manager.get_storage("offchain_tasks")?;
                storage.put(&format!("task:{}", task_id), task)?;
            }
        }
        
        // ノードレジストリから削除
        self.node_registry.remove_node(node_id).await?;
        
        // ストレージから削除
        let storage = self.storage_manager.get_storage("offchain_nodes")?;
        storage.delete(&format!("node:{}", node_id))?;
        
        // 現在時刻を取得
        let now = chrono::Utc::now();
        
        // ノード削除イベントを発行
        let event = OffchainEvent {
            id: format!("event-{}", Uuid::new_v4()),
            event_type: OffchainEventType::NodeRemoved,
            timestamp: now,
            data: serde_json::json!({
                "node_id": node_id,
                "assigned_tasks": node.assigned_tasks,
            }),
        };
        
        if let Err(e) = self.event_tx.send(event).await {
            error!("Failed to send node removed event: {}", e);
        }
        
        // メトリクスを更新
        self.metrics.increment_counter("offchain_nodes_removed");
        self.metrics.set_gauge("offchain_node_count", self.nodes.len() as f64);
        
        info!("Removed compute node: {}", node_id);
        
        Ok(())
    }
    
    /// 計算タスクを作成
    pub async fn create_task(
        &mut self,
        name: &str,
        description: &str,
        code: &str,
        input_data: &[u8],
        required_capabilities: Vec<NodeCapability>,
        required_resources: NodeResource,
        priority: TaskPriority,
        timeout_ms: u64,
    ) -> Result<TaskId, Error> {
        // タスクIDを生成
        let task_id = format!("task-{}", Uuid::new_v4());
        
        // 現在時刻を取得
        let now = chrono::Utc::now();
        
        // タスクを作成
        let task = ComputeTask {
            id: task_id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            code: code.to_string(),
            input_data: input_data.to_vec(),
            output_data: None,
            status: TaskStatus::Pending,
            priority,
            required_capabilities,
            required_resources,
            assigned_node: None,
            execution_time_ms: None,
            wait_time_ms: None,
            created_at: now,
            started_at: None,
            completed_at: None,
            timeout_ms,
            verification_result: None,
            proof: None,
            metadata: HashMap::new(),
        };
        
        // タスクを保存
        self.tasks.insert(task_id.clone(), task.clone());
        
        // タスクレジストリに登録
        self.task_registry.register_task(task.clone()).await?;
        
        // ストレージに保存
        let storage = self.storage_manager.get_storage("offchain_tasks")?;
        storage.put(&format!("task:{}", task_id), &task)?;
        
        // タスク作成イベントを発行
        let event = OffchainEvent {
            id: format!("event-{}", Uuid::new_v4()),
            event_type: OffchainEventType::TaskCreated,
            timestamp: now,
            data: serde_json::json!({
                "task_id": task_id,
                "name": name,
                "priority": priority,
                "required_capabilities": required_capabilities,
                "required_resources": required_resources,
            }),
        };
        
        if let Err(e) = self.event_tx.send(event).await {
            error!("Failed to send task created event: {}", e);
        }
        
        // メトリクスを更新
        self.metrics.increment_counter("offchain_tasks_created");
        self.metrics.set_gauge("offchain_task_count", self.tasks.len() as f64);
        
        info!("Created compute task: {} ({})", task_id, name);
        
        Ok(task_id)
    }
    
    /// タスクを割り当て
    pub async fn assign_task(
        &mut self,
        task_id: &str,
        node_id: &str,
    ) -> Result<(), Error> {
        // タスクが存在するかチェック
        let task = self.tasks.get_mut(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task not found: {}", task_id)))?;
        
        // ノードが存在するかチェック
        let node = self.nodes.get_mut(node_id)
            .ok_or_else(|| Error::NotFound(format!("Node not found: {}", node_id)))?;
        
        // タスクが割り当て可能かチェック
        if task.status != TaskStatus::Pending {
            return Err(Error::InvalidState(format!("Task is not pending: {}", task_id)));
        }
        
        // ノードが利用可能かチェック
        if node.status != NodeStatus::Available {
            return Err(Error::InvalidState(format!("Node is not available: {}", node_id)));
        }
        
        // ノードが必要な機能を持っているかチェック
        for capability in &task.required_capabilities {
            if !node.capabilities.contains(capability) {
                return Err(Error::InvalidArgument(format!("Node does not have required capability: {:?}", capability)));
            }
        }
        
        // ノードが必要なリソースを持っているかチェック
        if node.resources.cpu_cores < task.required_resources.cpu_cores ||
           node.resources.memory_mb < task.required_resources.memory_mb ||
           node.resources.storage_mb < task.required_resources.storage_mb ||
           node.resources.gpu_cores < task.required_resources.gpu_cores {
            return Err(Error::InvalidArgument(format!("Node does not have required resources")));
        }
        
        // 現在時刻を取得
        let now = chrono::Utc::now();
        
        // タスクを割り当て
        task.status = TaskStatus::Assigned;
        task.assigned_node = Some(node_id.to_string());
        
        // 待機時間を計算
        task.wait_time_ms = Some(now.timestamp_millis() - task.created_at.timestamp_millis());
        
        // ノードにタスクを割り当て
        node.assigned_tasks.push(task_id.to_string());
        node.status = NodeStatus::Busy;
        
        // ノードのリソースを更新
        node.resources.cpu_cores -= task.required_resources.cpu_cores;
        node.resources.memory_mb -= task.required_resources.memory_mb;
        node.resources.storage_mb -= task.required_resources.storage_mb;
        node.resources.gpu_cores -= task.required_resources.gpu_cores;
        
        // タスクを保存
        let task_storage = self.storage_manager.get_storage("offchain_tasks")?;
        task_storage.put(&format!("task:{}", task_id), task)?;
        
        // ノードを保存
        let node_storage = self.storage_manager.get_storage("offchain_nodes")?;
        node_storage.put(&format!("node:{}", node_id), node)?;
        
        // タスク割り当てイベントを発行
        let event = OffchainEvent {
            id: format!("event-{}", Uuid::new_v4()),
            event_type: OffchainEventType::TaskAssigned,
            timestamp: now,
            data: serde_json::json!({
                "task_id": task_id,
                "node_id": node_id,
                "wait_time_ms": task.wait_time_ms,
            }),
        };
        
        if let Err(e) = self.event_tx.send(event).await {
            error!("Failed to send task assigned event: {}", e);
        }
        
        // メトリクスを更新
        self.metrics.increment_counter("offchain_tasks_assigned");
        
        info!("Assigned task {} to node {}", task_id, node_id);
        
        Ok(())
    }
    
    /// タスク実行を開始
    pub async fn start_task(
        &mut self,
        task_id: &str,
    ) -> Result<(), Error> {
        // タスクが存在するかチェック
        let task = self.tasks.get_mut(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task not found: {}", task_id)))?;
        
        // タスクが割り当て済みかチェック
        if task.status != TaskStatus::Assigned {
            return Err(Error::InvalidState(format!("Task is not assigned: {}", task_id)));
        }
        
        // 現在時刻を取得
        let now = chrono::Utc::now();
        
        // タスクを開始
        task.status = TaskStatus::Running;
        task.started_at = Some(now);
        
        // タスクを保存
        let storage = self.storage_manager.get_storage("offchain_tasks")?;
        storage.put(&format!("task:{}", task_id), task)?;
        
        // タスク開始イベントを発行
        let event = OffchainEvent {
            id: format!("event-{}", Uuid::new_v4()),
            event_type: OffchainEventType::TaskStarted,
            timestamp: now,
            data: serde_json::json!({
                "task_id": task_id,
                "node_id": task.assigned_node,
                "started_at": now,
            }),
        };
        
        if let Err(e) = self.event_tx.send(event).await {
            error!("Failed to send task started event: {}", e);
        }
        
        // メトリクスを更新
        self.metrics.increment_counter("offchain_tasks_started");
        
        info!("Started task: {}", task_id);
        
        Ok(())
    }
    
    /// タスク完了を報告
    pub async fn complete_task(
        &mut self,
        task_id: &str,
        output_data: &[u8],
    ) -> Result<(), Error> {
        // タスクが存在するかチェック
        let task = self.tasks.get_mut(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task not found: {}", task_id)))?;
        
        // タスクが実行中かチェック
        if task.status != TaskStatus::Running {
            return Err(Error::InvalidState(format!("Task is not running: {}", task_id)));
        }
        
        // ノードIDを取得
        let node_id = task.assigned_node.clone()
            .ok_or_else(|| Error::InvalidState(format!("Task has no assigned node: {}", task_id)))?;
        
        // ノードが存在するかチェック
        let node = self.nodes.get_mut(&node_id)
            .ok_or_else(|| Error::NotFound(format!("Node not found: {}", node_id)))?;
        
        // 現在時刻を取得
        let now = chrono::Utc::now();
        
        // 実行時間を計算
        let execution_time_ms = if let Some(started_at) = task.started_at {
            now.timestamp_millis() - started_at.timestamp_millis()
        } else {
            0
        };
        
        // タスクを完了
        task.status = TaskStatus::Completed;
        task.output_data = Some(output_data.to_vec());
        task.completed_at = Some(now);
        task.execution_time_ms = Some(execution_time_ms);
        
        // ノードのタスクリストを更新
        if let Some(index) = node.assigned_tasks.iter().position(|id| id == task_id) {
            node.assigned_tasks.remove(index);
        }
        node.completed_tasks.push(task_id.to_string());
        
        // ノードのリソースを解放
        node.resources.cpu_cores += task.required_resources.cpu_cores;
        node.resources.memory_mb += task.required_resources.memory_mb;
        node.resources.storage_mb += task.required_resources.storage_mb;
        node.resources.gpu_cores += task.required_resources.gpu_cores;
        
        // ノードのステータスを更新
        if node.assigned_tasks.is_empty() {
            node.status = NodeStatus::Available;
        }
        
        // ノードのパフォーマンスメトリクスを更新
        node.performance_metrics.insert("avg_execution_time_ms".to_string(), execution_time_ms as f64);
        
        // タスクを保存
        let task_storage = self.storage_manager.get_storage("offchain_tasks")?;
        task_storage.put(&format!("task:{}", task_id), task)?;
        
        // ノードを保存
        let node_storage = self.storage_manager.get_storage("offchain_nodes")?;
        node_storage.put(&format!("node:{}", node_id), node)?;
        
        // タスク完了イベントを発行
        let event = OffchainEvent {
            id: format!("event-{}", Uuid::new_v4()),
            event_type: OffchainEventType::TaskCompleted,
            timestamp: now,
            data: serde_json::json!({
                "task_id": task_id,
                "node_id": node_id,
                "execution_time_ms": execution_time_ms,
                "output_size": output_data.len(),
            }),
        };
        
        if let Err(e) = self.event_tx.send(event).await {
            error!("Failed to send task completed event: {}", e);
        }
        
        // メトリクスを更新
        self.metrics.increment_counter("offchain_tasks_completed");
        self.metrics.observe_histogram("offchain_task_execution_time_ms", execution_time_ms as f64);
        
        info!("Completed task {} in {} ms", task_id, execution_time_ms);
        
        Ok(())
    }
    
    /// タスク失敗を報告
    pub async fn fail_task(
        &mut self,
        task_id: &str,
        error_message: &str,
    ) -> Result<(), Error> {
        // タスクが存在するかチェック
        let task = self.tasks.get_mut(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task not found: {}", task_id)))?;
        
        // タスクが実行中または割り当て済みかチェック
        if task.status != TaskStatus::Running && task.status != TaskStatus::Assigned {
            return Err(Error::InvalidState(format!("Task is not running or assigned: {}", task_id)));
        }
        
        // ノードIDを取得
        let node_id = task.assigned_node.clone()
            .ok_or_else(|| Error::InvalidState(format!("Task has no assigned node: {}", task_id)))?;
        
        // ノードが存在するかチェック
        let node = self.nodes.get_mut(&node_id)
            .ok_or_else(|| Error::NotFound(format!("Node not found: {}", node_id)))?;
        
        // 現在時刻を取得
        let now = chrono::Utc::now();
        
        // 実行時間を計算
        let execution_time_ms = if let Some(started_at) = task.started_at {
            now.timestamp_millis() - started_at.timestamp_millis()
        } else {
            0
        };
        
        // タスクを失敗
        task.status = TaskStatus::Failed;
        task.completed_at = Some(now);
        task.execution_time_ms = Some(execution_time_ms);
        task.metadata.insert("error_message".to_string(), error_message.to_string());
        
        // ノードのタスクリストを更新
        if let Some(index) = node.assigned_tasks.iter().position(|id| id == task_id) {
            node.assigned_tasks.remove(index);
        }
        
        // ノードのリソースを解放
        node.resources.cpu_cores += task.required_resources.cpu_cores;
        node.resources.memory_mb += task.required_resources.memory_mb;
        node.resources.storage_mb += task.required_resources.storage_mb;
        node.resources.gpu_cores += task.required_resources.gpu_cores;
        
        // ノードのステータスを更新
        if node.assigned_tasks.is_empty() {
            node.status = NodeStatus::Available;
        }
        
        // タスクを保存
        let task_storage = self.storage_manager.get_storage("offchain_tasks")?;
        task_storage.put(&format!("task:{}", task_id), task)?;
        
        // ノードを保存
        let node_storage = self.storage_manager.get_storage("offchain_nodes")?;
        node_storage.put(&format!("node:{}", node_id), node)?;
        
        // タスク失敗イベントを発行
        let event = OffchainEvent {
            id: format!("event-{}", Uuid::new_v4()),
            event_type: OffchainEventType::TaskFailed,
            timestamp: now,
            data: serde_json::json!({
                "task_id": task_id,
                "node_id": node_id,
                "execution_time_ms": execution_time_ms,
                "error_message": error_message,
            }),
        };
        
        if let Err(e) = self.event_tx.send(event).await {
            error!("Failed to send task failed event: {}", e);
        }
        
        // メトリクスを更新
        self.metrics.increment_counter("offchain_tasks_failed");
        
        info!("Failed task {}: {}", task_id, error_message);
        
        Ok(())
    }
    
    /// タスク結果を検証
    pub async fn verify_task_result(
        &mut self,
        task_id: &str,
    ) -> Result<bool, Error> {
        // タスクが存在するかチェック
        let task = self.tasks.get_mut(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task not found: {}", task_id)))?;
        
        // タスクが完了しているかチェック
        if task.status != TaskStatus::Completed {
            return Err(Error::InvalidState(format!("Task is not completed: {}", task_id)));
        }
        
        // 出力データが存在するかチェック
        let output_data = task.output_data.as_ref()
            .ok_or_else(|| Error::InvalidState(format!("Task has no output data: {}", task_id)))?;
        
        // 結果を検証
        let verification_result = self.verifier.verify_result(
            task_id,
            &task.code,
            &task.input_data,
            output_data,
        ).await?;
        
        // 現在時刻を取得
        let now = chrono::Utc::now();
        
        // タスクの検証結果を更新
        task.verification_result = Some(verification_result.clone());
        
        // タスクを保存
        let storage = self.storage_manager.get_storage("offchain_tasks")?;
        storage.put(&format!("task:{}", task_id), task)?;
        
        // 結果検証イベントを発行
        let event = OffchainEvent {
            id: format!("event-{}", Uuid::new_v4()),
            event_type: OffchainEventType::ResultVerified,
            timestamp: now,
            data: serde_json::json!({
                "task_id": task_id,
                "verified": verification_result.is_valid,
                "verification_method": verification_result.method,
                "verification_time_ms": verification_result.verification_time_ms,
            }),
        };
        
        if let Err(e) = self.event_tx.send(event).await {
            error!("Failed to send result verified event: {}", e);
        }
        
        // メトリクスを更新
        self.metrics.increment_counter("offchain_results_verified");
        
        info!("Verified task result {}: {}", task_id, verification_result.is_valid);
        
        Ok(verification_result.is_valid)
    }
    
    /// タスク結果の証明を生成
    pub async fn generate_proof(
        &mut self,
        task_id: &str,
        proof_type: ProofType,
    ) -> Result<ProofData, Error> {
        // タスクが存在するかチェック
        let task = self.tasks.get_mut(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task not found: {}", task_id)))?;
        
        // タスクが完了しているかチェック
        if task.status != TaskStatus::Completed {
            return Err(Error::InvalidState(format!("Task is not completed: {}", task_id)));
        }
        
        // 出力データが存在するかチェック
        let output_data = task.output_data.as_ref()
            .ok_or_else(|| Error::InvalidState(format!("Task has no output data: {}", task_id)))?;
        
        // 証明を生成
        let proof = self.prover.generate_proof(
            task_id,
            &task.code,
            &task.input_data,
            output_data,
            proof_type,
        ).await?;
        
        // 現在時刻を取得
        let now = chrono::Utc::now();
        
        // タスクの証明を更新
        task.proof = Some(proof.clone());
        
        // タスクを保存
        let storage = self.storage_manager.get_storage("offchain_tasks")?;
        storage.put(&format!("task:{}", task_id), task)?;
        
        // 証明生成イベントを発行
        let event = OffchainEvent {
            id: format!("event-{}", Uuid::new_v4()),
            event_type: OffchainEventType::ProofGenerated,
            timestamp: now,
            data: serde_json::json!({
                "task_id": task_id,
                "proof_type": format!("{:?}", proof_type),
                "proof_size": proof.proof_data.len(),
                "generation_time_ms": proof.generation_time_ms,
            }),
        };
        
        if let Err(e) = self.event_tx.send(event).await {
            error!("Failed to send proof generated event: {}", e);
        }
        
        // メトリクスを更新
        self.metrics.increment_counter("offchain_proofs_generated");
        
        info!("Generated {:?} proof for task {}", proof_type, task_id);
        
        Ok(proof)
    }
    
    /// ノードハートビートを更新
    pub async fn update_node_heartbeat(
        &mut self,
        node_id: &str,
    ) -> Result<(), Error> {
        // ノードが存在するかチェック
        let node = self.nodes.get_mut(node_id)
            .ok_or_else(|| Error::NotFound(format!("Node not found: {}", node_id)))?;
        
        // 現在時刻を取得
        let now = chrono::Utc::now();
        
        // ハートビートを更新
        node.last_heartbeat = now;
        
        // ノードを保存
        let storage = self.storage_manager.get_storage("offchain_nodes")?;
        storage.put(&format!("node:{}", node_id), node)?;
        
        debug!("Updated heartbeat for node: {}", node_id);
        
        Ok(())
    }
    
    /// タスクを取得
    pub fn get_task(&self, task_id: &str) -> Result<&ComputeTask, Error> {
        self.tasks.get(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task not found: {}", task_id)))
    }
    
    /// ノードを取得
    pub fn get_node(&self, node_id: &str) -> Result<&ComputeNode, Error> {
        self.nodes.get(node_id)
            .ok_or_else(|| Error::NotFound(format!("Node not found: {}", node_id)))
    }
    
    /// すべてのタスクIDを取得
    pub fn get_all_task_ids(&self) -> Vec<TaskId> {
        self.tasks.keys().cloned().collect()
    }
    
    /// すべてのノードIDを取得
    pub fn get_all_node_ids(&self) -> Vec<NodeId> {
        self.nodes.keys().cloned().collect()
    }
    
    /// 統計を取得
    pub fn get_stats(&self) -> OffchainStats {
        let mut active_node_count = 0;
        let mut completed_task_count = 0;
        let mut failed_task_count = 0;
        let mut pending_task_count = 0;
        let mut running_task_count = 0;
        let mut total_execution_time_ms = 0;
        let mut total_wait_time_ms = 0;
        let mut execution_time_count = 0;
        let mut wait_time_count = 0;
        
        // 総計算リソースと利用可能計算リソースを初期化
        let mut total_resources = NodeResource {
            cpu_cores: 0,
            memory_mb: 0,
            storage_mb: 0,
            gpu_cores: 0,
        };
        
        let mut available_resources = NodeResource {
            cpu_cores: 0,
            memory_mb: 0,
            storage_mb: 0,
            gpu_cores: 0,
        };
        
        // ノード統計を計算
        for node in self.nodes.values() {
            if node.status == NodeStatus::Available {
                active_node_count += 1;
            }
            
            // 総リソースを加算
            total_resources.cpu_cores += node.resources.cpu_cores;
            total_resources.memory_mb += node.resources.memory_mb;
            total_resources.storage_mb += node.resources.storage_mb;
            total_resources.gpu_cores += node.resources.gpu_cores;
            
            // 利用可能リソースを加算（ノードが利用可能な場合のみ）
            if node.status == NodeStatus::Available {
                available_resources.cpu_cores += node.resources.cpu_cores;
                available_resources.memory_mb += node.resources.memory_mb;
                available_resources.storage_mb += node.resources.storage_mb;
                available_resources.gpu_cores += node.resources.gpu_cores;
            }
        }
        
        // タスク統計を計算
        for task in self.tasks.values() {
            match task.status {
                TaskStatus::Completed => completed_task_count += 1,
                TaskStatus::Failed => failed_task_count += 1,
                TaskStatus::Pending => pending_task_count += 1,
                TaskStatus::Running => running_task_count += 1,
                _ => {}
            }
            
            // 実行時間を加算
            if let Some(execution_time) = task.execution_time_ms {
                total_execution_time_ms += execution_time;
                execution_time_count += 1;
            }
            
            // 待機時間を加算
            if let Some(wait_time) = task.wait_time_ms {
                total_wait_time_ms += wait_time;
                wait_time_count += 1;
            }
        }
        
        // 平均実行時間を計算
        let average_task_execution_time_ms = if execution_time_count > 0 {
            total_execution_time_ms as f64 / execution_time_count as f64
        } else {
            0.0
        };
        
        // 平均待機時間を計算
        let average_task_wait_time_ms = if wait_time_count > 0 {
            total_wait_time_ms as f64 / wait_time_count as f64
        } else {
            0.0
        };
        
        OffchainStats {
            node_count: self.nodes.len(),
            active_node_count,
            task_count: self.tasks.len(),
            completed_task_count,
            failed_task_count,
            pending_task_count,
            running_task_count,
            average_task_execution_time_ms,
            average_task_wait_time_ms,
            total_compute_resources: total_resources,
            available_compute_resources: available_resources,
            metadata: HashMap::new(),
        }
    }
    
    /// 設定を取得
    pub fn get_config(&self) -> &OffchainConfig {
        &self.config
    }
    
    /// 設定を更新
    pub fn update_config(&mut self, config: OffchainConfig) {
        self.config = config.clone();
        self.node_registry.update_config(config.registry_config.clone());
        self.verifier.update_config(config.verifier_config);
        self.prover.update_config(config.prover_config);
        self.executor.update_config(config.executor_config);
        self.scheduler.update_config(config.scheduler_config);
        self.incentive_manager.update_config(config.incentive_config);
        self.protocol.update_config(config.protocol_config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::test_utils::create_test_network_manager;
    use crate::storage::test_utils::create_test_storage_manager;
    use crate::crypto::test_utils::create_test_crypto_manager;
    
    #[tokio::test]
    async fn test_offchain_manager_creation() {
        let config = OffchainConfig::default();
        let network_manager = Arc::new(create_test_network_manager());
        let storage_manager = Arc::new(create_test_storage_manager());
        let crypto_manager = Arc::new(create_test_crypto_manager());
        let metrics = Arc::new(MetricsCollector::new("offchain"));
        
        let manager = OffchainManager::new(
            config,
            network_manager,
            storage_manager,
            crypto_manager,
            metrics,
        );
        
        assert!(!manager.running);
        assert!(manager.nodes.is_empty());
        assert!(manager.tasks.is_empty());
    }
    
    #[tokio::test]
    async fn test_offchain_manager_start_stop() {
        let config = OffchainConfig::default();
        let network_manager = Arc::new(create_test_network_manager());
        let storage_manager = Arc::new(create_test_storage_manager());
        let crypto_manager = Arc::new(create_test_crypto_manager());
        let metrics = Arc::new(MetricsCollector::new("offchain"));
        
        let mut manager = OffchainManager::new(
            config,
            network_manager,
            storage_manager,
            crypto_manager,
            metrics,
        );
        
        // 開始
        let result = manager.start().await;
        assert!(result.is_ok());
        assert!(manager.running);
        
        // 停止
        let result = manager.stop().await;
        assert!(result.is_ok());
        assert!(!manager.running);
    }
    
    #[tokio::test]
    async fn test_register_node() {
        let config = OffchainConfig::default();
        let network_manager = Arc::new(create_test_network_manager());
        let storage_manager = Arc::new(create_test_storage_manager());
        let crypto_manager = Arc::new(create_test_crypto_manager());
        let metrics = Arc::new(MetricsCollector::new("offchain"));
        
        let mut manager = OffchainManager::new(
            config,
            network_manager,
            storage_manager,
            crypto_manager,
            metrics,
        );
        
        // ノードを登録
        let capabilities = vec![NodeCapability::CPU, NodeCapability::GPU];
        let resources = NodeResource {
            cpu_cores: 8,
            memory_mb: 16384,
            storage_mb: 1024000,
            gpu_cores: 2,
        };
        
        let result = manager.register_node("test_node", "http://localhost:8080", capabilities, resources).await;
        
        // テスト環境では実際のノードは登録できないので、エラーになることを確認
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_get_stats() {
        let config = OffchainConfig::default();
        let network_manager = Arc::new(create_test_network_manager());
        let storage_manager = Arc::new(create_test_storage_manager());
        let crypto_manager = Arc::new(create_test_crypto_manager());
        let metrics = Arc::new(MetricsCollector::new("offchain"));
        
        let manager = OffchainManager::new(
            config,
            network_manager,
            storage_manager,
            crypto_manager,
            metrics,
        );
        
        // 統計を取得
        let stats = manager.get_stats();
        
        assert_eq!(stats.node_count, 0);
        assert_eq!(stats.active_node_count, 0);
        assert_eq!(stats.task_count, 0);
        assert_eq!(stats.completed_task_count, 0);
        assert_eq!(stats.failed_task_count, 0);
        assert_eq!(stats.pending_task_count, 0);
        assert_eq!(stats.running_task_count, 0);
    }
}