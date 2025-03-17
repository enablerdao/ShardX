// 動的シャーディングモジュール
//
// このモジュールは、ShardXの動的シャーディング機能を提供します。
// 動的シャーディングは、システムの負荷とデータ分布に基づいて、
// シャードの作成、分割、マージ、再配置を自動的に行う機能です。
//
// 主な機能:
// - 負荷ベースのシャード分割
// - データ分布に基づくシャード最適化
// - ホットスポット検出と緩和
// - シャード再配置
// - 自動シャードバランシング

mod config;
mod metrics;
mod balancer;
mod splitter;
mod merger;
mod relocator;
mod hotspot;
mod optimizer;

pub use self::config::{DynamicShardingConfig, ShardSplitPolicy, ShardMergePolicy, RebalancePolicy};
pub use self::metrics::{ShardMetrics, LoadMetric, DataDistributionMetric};
pub use self::balancer::{ShardBalancer, BalancingStrategy, BalancingResult};
pub use self::splitter::{ShardSplitter, SplitStrategy, SplitResult};
pub use self::merger::{ShardMerger, MergeStrategy, MergeResult};
pub use self::relocator::{ShardRelocator, RelocationStrategy, RelocationResult};
pub use self::hotspot::{HotspotDetector, HotspotType, HotspotSeverity};
pub use self::optimizer::{ShardOptimizer, OptimizationStrategy, OptimizationResult};

use crate::error::Error;
use crate::sharding::{Shard, ShardId, ShardManager, ShardPlacement};
use crate::metrics::MetricsCollector;
use crate::network::NodeId;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use log::{debug, error, info, warn};
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};

/// 動的シャーディングマネージャー
pub struct DynamicShardingManager {
    /// 設定
    config: DynamicShardingConfig,
    /// シャードマネージャー
    shard_manager: Arc<RwLock<ShardManager>>,
    /// シャードバランサー
    balancer: ShardBalancer,
    /// シャード分割器
    splitter: ShardSplitter,
    /// シャードマージャー
    merger: ShardMerger,
    /// シャード再配置器
    relocator: ShardRelocator,
    /// ホットスポット検出器
    hotspot_detector: HotspotDetector,
    /// シャード最適化器
    optimizer: ShardOptimizer,
    /// メトリクスコレクター
    metrics: Arc<MetricsCollector>,
    /// シャードメトリクス
    shard_metrics: HashMap<ShardId, ShardMetrics>,
    /// 実行中の操作
    pending_operations: HashMap<String, DynamicShardingOperation>,
    /// 操作履歴
    operation_history: Vec<DynamicShardingOperationResult>,
    /// 操作通知チャネル
    operation_tx: mpsc::Sender<DynamicShardingOperationResult>,
    /// 操作通知受信チャネル
    operation_rx: mpsc::Receiver<DynamicShardingOperationResult>,
    /// 実行中フラグ
    running: bool,
}

/// 動的シャーディング操作
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynamicShardingOperation {
    /// 操作ID
    pub id: String,
    /// 操作タイプ
    pub operation_type: DynamicShardingOperationType,
    /// 対象シャードID
    pub shard_ids: Vec<ShardId>,
    /// 対象ノードID
    pub node_ids: Option<Vec<NodeId>>,
    /// 開始時刻
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// 予想完了時間
    pub estimated_completion_time: chrono::DateTime<chrono::Utc>,
    /// 優先度
    pub priority: OperationPriority,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// 動的シャーディング操作タイプ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DynamicShardingOperationType {
    /// シャード分割
    Split,
    /// シャードマージ
    Merge,
    /// シャード再配置
    Relocate,
    /// シャードバランシング
    Rebalance,
    /// ホットスポット緩和
    HotspotMitigation,
    /// シャード最適化
    Optimization,
}

/// 操作優先度
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationPriority {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 緊急
    Critical,
}

/// 動的シャーディング操作結果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynamicShardingOperationResult {
    /// 操作ID
    pub id: String,
    /// 操作タイプ
    pub operation_type: DynamicShardingOperationType,
    /// 対象シャードID
    pub shard_ids: Vec<ShardId>,
    /// 結果シャードID
    pub result_shard_ids: Vec<ShardId>,
    /// 対象ノードID
    pub node_ids: Option<Vec<NodeId>>,
    /// 開始時刻
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// 完了時刻
    pub completion_time: chrono::DateTime<chrono::Utc>,
    /// 状態
    pub status: OperationStatus,
    /// エラーメッセージ
    pub error_message: Option<String>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// 操作状態
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationStatus {
    /// 成功
    Success,
    /// 失敗
    Failed,
    /// キャンセル
    Cancelled,
    /// 部分的に成功
    PartialSuccess,
}

impl DynamicShardingManager {
    /// 新しいDynamicShardingManagerを作成
    pub fn new(
        config: DynamicShardingConfig,
        shard_manager: Arc<RwLock<ShardManager>>,
        metrics: Arc<MetricsCollector>,
    ) -> Self {
        let (tx, rx) = mpsc::channel(100);
        
        let balancer = ShardBalancer::new(config.rebalance_policy.clone());
        let splitter = ShardSplitter::new(config.shard_split_policy.clone());
        let merger = ShardMerger::new(config.shard_merge_policy.clone());
        let relocator = ShardRelocator::new();
        let hotspot_detector = HotspotDetector::new(config.hotspot_detection_config.clone());
        let optimizer = ShardOptimizer::new();
        
        Self {
            config,
            shard_manager,
            balancer,
            splitter,
            merger,
            relocator,
            hotspot_detector,
            optimizer,
            metrics,
            shard_metrics: HashMap::new(),
            pending_operations: HashMap::new(),
            operation_history: Vec::new(),
            operation_tx: tx,
            operation_rx: rx,
            running: false,
        }
    }
    
    /// 動的シャーディングを開始
    pub async fn start(&mut self) -> Result<(), Error> {
        if self.running {
            return Err(Error::InvalidState("Dynamic sharding is already running".to_string()));
        }
        
        self.running = true;
        
        // 初期メトリクスを収集
        self.collect_metrics().await?;
        
        // バックグラウンドタスクを開始
        self.start_background_tasks();
        
        info!("Dynamic sharding manager started");
        
        Ok(())
    }
    
    /// 動的シャーディングを停止
    pub async fn stop(&mut self) -> Result<(), Error> {
        if !self.running {
            return Err(Error::InvalidState("Dynamic sharding is not running".to_string()));
        }
        
        self.running = false;
        
        // 実行中の操作をキャンセル
        for (op_id, op) in &self.pending_operations {
            info!("Cancelling operation: {}", op_id);
            
            // 操作結果を作成
            let result = DynamicShardingOperationResult {
                id: op_id.clone(),
                operation_type: op.operation_type.clone(),
                shard_ids: op.shard_ids.clone(),
                result_shard_ids: Vec::new(),
                node_ids: op.node_ids.clone(),
                start_time: op.start_time,
                completion_time: chrono::Utc::now(),
                status: OperationStatus::Cancelled,
                error_message: Some("Operation cancelled due to manager shutdown".to_string()),
                metadata: op.metadata.clone(),
            };
            
            // 履歴に追加
            self.operation_history.push(result.clone());
            
            // 通知を送信
            if let Err(e) = self.operation_tx.send(result).await {
                error!("Failed to send operation result: {}", e);
            }
        }
        
        // 保留中の操作をクリア
        self.pending_operations.clear();
        
        info!("Dynamic sharding manager stopped");
        
        Ok(())
    }
    
    /// バックグラウンドタスクを開始
    fn start_background_tasks(&self) {
        // メトリクス収集タスク
        let metrics_interval = self.config.metrics_collection_interval_ms;
        let metrics_tx = self.operation_tx.clone();
        let metrics_manager = Arc::new(Mutex::new(self.clone()));
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(metrics_interval));
            
            loop {
                interval.tick().await;
                
                let manager = metrics_manager.lock().unwrap();
                if !manager.running {
                    break;
                }
                
                // メトリクスを収集
                match manager.collect_metrics().await {
                    Ok(_) => debug!("Metrics collected successfully"),
                    Err(e) => error!("Failed to collect metrics: {}", e),
                }
                
                // ホットスポットを検出
                match manager.detect_hotspots().await {
                    Ok(hotspots) => {
                        if !hotspots.is_empty() {
                            info!("Detected {} hotspots", hotspots.len());
                            
                            // ホットスポットを緩和
                            for hotspot in hotspots {
                                match manager.mitigate_hotspot(&hotspot).await {
                                    Ok(op_id) => info!("Started hotspot mitigation: {}", op_id),
                                    Err(e) => error!("Failed to mitigate hotspot: {}", e),
                                }
                            }
                        }
                    },
                    Err(e) => error!("Failed to detect hotspots: {}", e),
                }
                
                // シャードバランスをチェック
                if manager.should_rebalance().await {
                    match manager.rebalance_shards().await {
                        Ok(op_id) => info!("Started shard rebalancing: {}", op_id),
                        Err(e) => error!("Failed to rebalance shards: {}", e),
                    }
                }
                
                // シャード最適化をチェック
                if manager.should_optimize().await {
                    match manager.optimize_shards().await {
                        Ok(op_id) => info!("Started shard optimization: {}", op_id),
                        Err(e) => error!("Failed to optimize shards: {}", e),
                    }
                }
            }
        });
        
        // 操作監視タスク
        let operation_interval = 1000; // 1秒
        let operation_tx = self.operation_tx.clone();
        let operation_manager = Arc::new(Mutex::new(self.clone()));
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(operation_interval));
            
            loop {
                interval.tick().await;
                
                let manager = operation_manager.lock().unwrap();
                if !manager.running {
                    break;
                }
                
                // 実行中の操作をチェック
                let completed_ops: Vec<String> = manager.pending_operations.iter()
                    .filter(|(_, op)| {
                        let now = chrono::Utc::now();
                        op.estimated_completion_time <= now
                    })
                    .map(|(id, _)| id.clone())
                    .collect();
                
                // 完了した操作を処理
                for op_id in completed_ops {
                    if let Some(op) = manager.pending_operations.get(&op_id) {
                        // 操作の結果を取得
                        match manager.get_operation_result(&op).await {
                            Ok(result) => {
                                // 履歴に追加
                                manager.operation_history.push(result.clone());
                                
                                // 通知を送信
                                if let Err(e) = operation_tx.send(result).await {
                                    error!("Failed to send operation result: {}", e);
                                }
                            },
                            Err(e) => error!("Failed to get operation result: {}", e),
                        }
                        
                        // 保留中の操作から削除
                        manager.pending_operations.remove(&op_id);
                    }
                }
            }
        });
    }
    
    /// メトリクスを収集
    async fn collect_metrics(&self) -> Result<(), Error> {
        let shard_manager = self.shard_manager.read().unwrap();
        let shards = shard_manager.get_all_shards();
        
        for shard in shards {
            let shard_id = shard.id.clone();
            
            // シャードメトリクスを収集
            let load_metrics = self.collect_load_metrics(&shard).await?;
            let distribution_metrics = self.collect_distribution_metrics(&shard).await?;
            
            // シャードメトリクスを更新
            let metrics = ShardMetrics {
                shard_id: shard_id.clone(),
                timestamp: chrono::Utc::now(),
                load_metrics,
                distribution_metrics,
                metadata: HashMap::new(),
            };
            
            // メトリクスを保存
            self.shard_metrics.insert(shard_id.clone(), metrics);
            
            // メトリクスコレクターを更新
            for (name, value) in &load_metrics {
                self.metrics.set_gauge(&format!("shard_{}_load_{}", shard_id, name), *value);
            }
            
            for (name, value) in &distribution_metrics {
                self.metrics.set_gauge(&format!("shard_{}_distribution_{}", shard_id, name), *value);
            }
        }
        
        Ok(())
    }
    
    /// 負荷メトリクスを収集
    async fn collect_load_metrics(&self, shard: &Shard) -> Result<HashMap<String, f64>, Error> {
        let mut metrics = HashMap::new();
        
        // トランザクション処理レート
        let tx_rate = shard.get_transaction_rate();
        metrics.insert("transaction_rate".to_string(), tx_rate as f64);
        
        // クエリ処理レート
        let query_rate = shard.get_query_rate();
        metrics.insert("query_rate".to_string(), query_rate as f64);
        
        // CPU使用率
        let cpu_usage = shard.get_cpu_usage();
        metrics.insert("cpu_usage".to_string(), cpu_usage);
        
        // メモリ使用率
        let memory_usage = shard.get_memory_usage();
        metrics.insert("memory_usage".to_string(), memory_usage);
        
        // ディスク使用率
        let disk_usage = shard.get_disk_usage();
        metrics.insert("disk_usage".to_string(), disk_usage);
        
        // ネットワーク使用率
        let network_usage = shard.get_network_usage();
        metrics.insert("network_usage".to_string(), network_usage);
        
        // レイテンシ
        let latency = shard.get_average_latency();
        metrics.insert("latency".to_string(), latency);
        
        Ok(metrics)
    }
    
    /// データ分布メトリクスを収集
    async fn collect_distribution_metrics(&self, shard: &Shard) -> Result<HashMap<String, f64>, Error> {
        let mut metrics = HashMap::new();
        
        // データサイズ
        let data_size = shard.get_data_size();
        metrics.insert("data_size".to_string(), data_size as f64);
        
        // キー数
        let key_count = shard.get_key_count();
        metrics.insert("key_count".to_string(), key_count as f64);
        
        // キー分布の偏り
        let key_skew = shard.get_key_distribution_skew();
        metrics.insert("key_skew".to_string(), key_skew);
        
        // アクセス分布の偏り
        let access_skew = shard.get_access_distribution_skew();
        metrics.insert("access_skew".to_string(), access_skew);
        
        // 書き込み/読み取り比率
        let write_read_ratio = shard.get_write_read_ratio();
        metrics.insert("write_read_ratio".to_string(), write_read_ratio);
        
        Ok(metrics)
    }
    
    /// ホットスポットを検出
    async fn detect_hotspots(&self) -> Result<Vec<hotspot::Hotspot>, Error> {
        // ホットスポット検出が有効かチェック
        if !self.config.enable_hotspot_detection {
            return Ok(Vec::new());
        }
        
        // シャードメトリクスを取得
        let metrics: Vec<&ShardMetrics> = self.shard_metrics.values().collect();
        
        // ホットスポットを検出
        let hotspots = self.hotspot_detector.detect_hotspots(&metrics)?;
        
        // ホットスポットをログに記録
        for hotspot in &hotspots {
            info!(
                "Detected hotspot: shard={}, type={:?}, severity={:?}, metric={}, value={}",
                hotspot.shard_id,
                hotspot.hotspot_type,
                hotspot.severity,
                hotspot.metric_name,
                hotspot.metric_value
            );
        }
        
        Ok(hotspots)
    }
    
    /// ホットスポットを緩和
    async fn mitigate_hotspot(&self, hotspot: &hotspot::Hotspot) -> Result<String, Error> {
        // ホットスポットの種類に基づいて緩和戦略を選択
        match hotspot.hotspot_type {
            HotspotType::Load => {
                // 負荷ホットスポットはシャード分割で緩和
                self.split_shard(&hotspot.shard_id).await
            },
            HotspotType::DataSize => {
                // データサイズホットスポットはシャード分割で緩和
                self.split_shard(&hotspot.shard_id).await
            },
            HotspotType::KeySkew => {
                // キー分布の偏りはシャード最適化で緩和
                self.optimize_shard(&hotspot.shard_id).await
            },
            HotspotType::AccessSkew => {
                // アクセス分布の偏りはシャード再配置で緩和
                self.relocate_shard(&hotspot.shard_id).await
            },
        }
    }
    
    /// シャードを分割
    pub async fn split_shard(&self, shard_id: &ShardId) -> Result<String, Error> {
        // シャード分割が有効かチェック
        if !self.config.enable_shard_splitting {
            return Err(Error::InvalidState("Shard splitting is not enabled".to_string()));
        }
        
        // シャードが存在するかチェック
        let shard_manager = self.shard_manager.read().unwrap();
        if !shard_manager.shard_exists(shard_id) {
            return Err(Error::NotFound(format!("Shard not found: {}", shard_id)));
        }
        
        // シャードメトリクスを取得
        let metrics = self.shard_metrics.get(shard_id)
            .ok_or_else(|| Error::NotFound(format!("Shard metrics not found: {}", shard_id)))?;
        
        // 分割戦略を決定
        let strategy = self.splitter.determine_split_strategy(metrics)?;
        
        // 操作IDを生成
        let operation_id = format!("split-{}-{}", shard_id, uuid::Uuid::new_v4());
        
        // 操作を作成
        let operation = DynamicShardingOperation {
            id: operation_id.clone(),
            operation_type: DynamicShardingOperationType::Split,
            shard_ids: vec![shard_id.clone()],
            node_ids: None,
            start_time: chrono::Utc::now(),
            estimated_completion_time: chrono::Utc::now() + chrono::Duration::seconds(self.config.shard_split_timeout_seconds as i64),
            priority: OperationPriority::High,
            metadata: HashMap::new(),
        };
        
        // 操作を保留中に追加
        self.pending_operations.insert(operation_id.clone(), operation);
        
        // バックグラウンドでシャード分割を実行
        let splitter = self.splitter.clone();
        let shard_manager_arc = self.shard_manager.clone();
        let operation_id_clone = operation_id.clone();
        let shard_id_clone = shard_id.clone();
        let tx = self.operation_tx.clone();
        
        tokio::spawn(async move {
            let result = splitter.split_shard(&shard_id_clone, strategy, shard_manager_arc).await;
            
            let (status, error_message, result_shard_ids) = match result {
                Ok(split_result) => {
                    (OperationStatus::Success, None, split_result.new_shard_ids)
                },
                Err(e) => {
                    (OperationStatus::Failed, Some(e.to_string()), Vec::new())
                },
            };
            
            // 操作結果を作成
            let operation_result = DynamicShardingOperationResult {
                id: operation_id_clone.clone(),
                operation_type: DynamicShardingOperationType::Split,
                shard_ids: vec![shard_id_clone],
                result_shard_ids,
                node_ids: None,
                start_time: chrono::Utc::now(),
                completion_time: chrono::Utc::now(),
                status,
                error_message,
                metadata: HashMap::new(),
            };
            
            // 結果を送信
            if let Err(e) = tx.send(operation_result).await {
                error!("Failed to send operation result: {}", e);
            }
        });
        
        info!("Started shard split operation: {}", operation_id);
        
        Ok(operation_id)
    }
    
    /// シャードをマージ
    pub async fn merge_shards(&self, shard_ids: &[ShardId]) -> Result<String, Error> {
        // シャードマージが有効かチェック
        if !self.config.enable_shard_merging {
            return Err(Error::InvalidState("Shard merging is not enabled".to_string()));
        }
        
        // シャード数をチェック
        if shard_ids.len() < 2 {
            return Err(Error::InvalidArgument("At least two shards are required for merging".to_string()));
        }
        
        // シャードが存在するかチェック
        let shard_manager = self.shard_manager.read().unwrap();
        for shard_id in shard_ids {
            if !shard_manager.shard_exists(shard_id) {
                return Err(Error::NotFound(format!("Shard not found: {}", shard_id)));
            }
        }
        
        // シャードメトリクスを取得
        let mut metrics = Vec::new();
        for shard_id in shard_ids {
            let shard_metrics = self.shard_metrics.get(shard_id)
                .ok_or_else(|| Error::NotFound(format!("Shard metrics not found: {}", shard_id)))?;
            metrics.push(shard_metrics);
        }
        
        // マージ戦略を決定
        let strategy = self.merger.determine_merge_strategy(&metrics)?;
        
        // 操作IDを生成
        let operation_id = format!("merge-{}-{}", shard_ids.join("-"), uuid::Uuid::new_v4());
        
        // 操作を作成
        let operation = DynamicShardingOperation {
            id: operation_id.clone(),
            operation_type: DynamicShardingOperationType::Merge,
            shard_ids: shard_ids.to_vec(),
            node_ids: None,
            start_time: chrono::Utc::now(),
            estimated_completion_time: chrono::Utc::now() + chrono::Duration::seconds(self.config.shard_merge_timeout_seconds as i64),
            priority: OperationPriority::Medium,
            metadata: HashMap::new(),
        };
        
        // 操作を保留中に追加
        self.pending_operations.insert(operation_id.clone(), operation);
        
        // バックグラウンドでシャードマージを実行
        let merger = self.merger.clone();
        let shard_manager_arc = self.shard_manager.clone();
        let operation_id_clone = operation_id.clone();
        let shard_ids_clone = shard_ids.to_vec();
        let tx = self.operation_tx.clone();
        
        tokio::spawn(async move {
            let result = merger.merge_shards(&shard_ids_clone, strategy, shard_manager_arc).await;
            
            let (status, error_message, result_shard_ids) = match result {
                Ok(merge_result) => {
                    (OperationStatus::Success, None, vec![merge_result.new_shard_id])
                },
                Err(e) => {
                    (OperationStatus::Failed, Some(e.to_string()), Vec::new())
                },
            };
            
            // 操作結果を作成
            let operation_result = DynamicShardingOperationResult {
                id: operation_id_clone.clone(),
                operation_type: DynamicShardingOperationType::Merge,
                shard_ids: shard_ids_clone,
                result_shard_ids,
                node_ids: None,
                start_time: chrono::Utc::now(),
                completion_time: chrono::Utc::now(),
                status,
                error_message,
                metadata: HashMap::new(),
            };
            
            // 結果を送信
            if let Err(e) = tx.send(operation_result).await {
                error!("Failed to send operation result: {}", e);
            }
        });
        
        info!("Started shard merge operation: {}", operation_id);
        
        Ok(operation_id)
    }
    
    /// シャードを再配置
    pub async fn relocate_shard(&self, shard_id: &ShardId) -> Result<String, Error> {
        // シャード再配置が有効かチェック
        if !self.config.enable_shard_relocation {
            return Err(Error::InvalidState("Shard relocation is not enabled".to_string()));
        }
        
        // シャードが存在するかチェック
        let shard_manager = self.shard_manager.read().unwrap();
        if !shard_manager.shard_exists(shard_id) {
            return Err(Error::NotFound(format!("Shard not found: {}", shard_id)));
        }
        
        // シャードメトリクスを取得
        let metrics = self.shard_metrics.get(shard_id)
            .ok_or_else(|| Error::NotFound(format!("Shard metrics not found: {}", shard_id)))?;
        
        // 再配置戦略を決定
        let (strategy, target_nodes) = self.relocator.determine_relocation_strategy(metrics, &shard_manager)?;
        
        // 操作IDを生成
        let operation_id = format!("relocate-{}-{}", shard_id, uuid::Uuid::new_v4());
        
        // 操作を作成
        let operation = DynamicShardingOperation {
            id: operation_id.clone(),
            operation_type: DynamicShardingOperationType::Relocate,
            shard_ids: vec![shard_id.clone()],
            node_ids: Some(target_nodes.clone()),
            start_time: chrono::Utc::now(),
            estimated_completion_time: chrono::Utc::now() + chrono::Duration::seconds(self.config.shard_relocation_timeout_seconds as i64),
            priority: OperationPriority::Medium,
            metadata: HashMap::new(),
        };
        
        // 操作を保留中に追加
        self.pending_operations.insert(operation_id.clone(), operation);
        
        // バックグラウンドでシャード再配置を実行
        let relocator = self.relocator.clone();
        let shard_manager_arc = self.shard_manager.clone();
        let operation_id_clone = operation_id.clone();
        let shard_id_clone = shard_id.clone();
        let tx = self.operation_tx.clone();
        
        tokio::spawn(async move {
            let result = relocator.relocate_shard(&shard_id_clone, &target_nodes, strategy, shard_manager_arc).await;
            
            let (status, error_message) = match result {
                Ok(_) => (OperationStatus::Success, None),
                Err(e) => (OperationStatus::Failed, Some(e.to_string())),
            };
            
            // 操作結果を作成
            let operation_result = DynamicShardingOperationResult {
                id: operation_id_clone.clone(),
                operation_type: DynamicShardingOperationType::Relocate,
                shard_ids: vec![shard_id_clone],
                result_shard_ids: vec![shard_id_clone],
                node_ids: Some(target_nodes),
                start_time: chrono::Utc::now(),
                completion_time: chrono::Utc::now(),
                status,
                error_message,
                metadata: HashMap::new(),
            };
            
            // 結果を送信
            if let Err(e) = tx.send(operation_result).await {
                error!("Failed to send operation result: {}", e);
            }
        });
        
        info!("Started shard relocation operation: {}", operation_id);
        
        Ok(operation_id)
    }
    
    /// シャードをリバランス
    pub async fn rebalance_shards(&self) -> Result<String, Error> {
        // シャードリバランスが有効かチェック
        if !self.config.enable_shard_rebalancing {
            return Err(Error::InvalidState("Shard rebalancing is not enabled".to_string()));
        }
        
        // シャードマネージャーを取得
        let shard_manager = self.shard_manager.read().unwrap();
        
        // リバランス戦略を決定
        let strategy = self.balancer.determine_rebalancing_strategy(&shard_manager)?;
        
        // 操作IDを生成
        let operation_id = format!("rebalance-{}", uuid::Uuid::new_v4());
        
        // 操作を作成
        let operation = DynamicShardingOperation {
            id: operation_id.clone(),
            operation_type: DynamicShardingOperationType::Rebalance,
            shard_ids: Vec::new(), // リバランスは全シャードが対象
            node_ids: None,
            start_time: chrono::Utc::now(),
            estimated_completion_time: chrono::Utc::now() + chrono::Duration::seconds(self.config.shard_rebalance_timeout_seconds as i64),
            priority: OperationPriority::Low,
            metadata: HashMap::new(),
        };
        
        // 操作を保留中に追加
        self.pending_operations.insert(operation_id.clone(), operation);
        
        // バックグラウンドでシャードリバランスを実行
        let balancer = self.balancer.clone();
        let shard_manager_arc = self.shard_manager.clone();
        let operation_id_clone = operation_id.clone();
        let tx = self.operation_tx.clone();
        
        tokio::spawn(async move {
            let result = balancer.rebalance_shards(strategy, shard_manager_arc).await;
            
            let (status, error_message, affected_shards) = match result {
                Ok(rebalance_result) => {
                    (OperationStatus::Success, None, rebalance_result.affected_shards)
                },
                Err(e) => {
                    (OperationStatus::Failed, Some(e.to_string()), Vec::new())
                },
            };
            
            // 操作結果を作成
            let operation_result = DynamicShardingOperationResult {
                id: operation_id_clone.clone(),
                operation_type: DynamicShardingOperationType::Rebalance,
                shard_ids: affected_shards.clone(),
                result_shard_ids: affected_shards,
                node_ids: None,
                start_time: chrono::Utc::now(),
                completion_time: chrono::Utc::now(),
                status,
                error_message,
                metadata: HashMap::new(),
            };
            
            // 結果を送信
            if let Err(e) = tx.send(operation_result).await {
                error!("Failed to send operation result: {}", e);
            }
        });
        
        info!("Started shard rebalancing operation: {}", operation_id);
        
        Ok(operation_id)
    }
    
    /// シャードを最適化
    pub async fn optimize_shard(&self, shard_id: &ShardId) -> Result<String, Error> {
        // シャード最適化が有効かチェック
        if !self.config.enable_shard_optimization {
            return Err(Error::InvalidState("Shard optimization is not enabled".to_string()));
        }
        
        // シャードが存在するかチェック
        let shard_manager = self.shard_manager.read().unwrap();
        if !shard_manager.shard_exists(shard_id) {
            return Err(Error::NotFound(format!("Shard not found: {}", shard_id)));
        }
        
        // シャードメトリクスを取得
        let metrics = self.shard_metrics.get(shard_id)
            .ok_or_else(|| Error::NotFound(format!("Shard metrics not found: {}", shard_id)))?;
        
        // 最適化戦略を決定
        let strategy = self.optimizer.determine_optimization_strategy(metrics)?;
        
        // 操作IDを生成
        let operation_id = format!("optimize-{}-{}", shard_id, uuid::Uuid::new_v4());
        
        // 操作を作成
        let operation = DynamicShardingOperation {
            id: operation_id.clone(),
            operation_type: DynamicShardingOperationType::Optimization,
            shard_ids: vec![shard_id.clone()],
            node_ids: None,
            start_time: chrono::Utc::now(),
            estimated_completion_time: chrono::Utc::now() + chrono::Duration::seconds(self.config.shard_optimization_timeout_seconds as i64),
            priority: OperationPriority::Low,
            metadata: HashMap::new(),
        };
        
        // 操作を保留中に追加
        self.pending_operations.insert(operation_id.clone(), operation);
        
        // バックグラウンドでシャード最適化を実行
        let optimizer = self.optimizer.clone();
        let shard_manager_arc = self.shard_manager.clone();
        let operation_id_clone = operation_id.clone();
        let shard_id_clone = shard_id.clone();
        let tx = self.operation_tx.clone();
        
        tokio::spawn(async move {
            let result = optimizer.optimize_shard(&shard_id_clone, strategy, shard_manager_arc).await;
            
            let (status, error_message) = match result {
                Ok(_) => (OperationStatus::Success, None),
                Err(e) => (OperationStatus::Failed, Some(e.to_string())),
            };
            
            // 操作結果を作成
            let operation_result = DynamicShardingOperationResult {
                id: operation_id_clone.clone(),
                operation_type: DynamicShardingOperationType::Optimization,
                shard_ids: vec![shard_id_clone.clone()],
                result_shard_ids: vec![shard_id_clone],
                node_ids: None,
                start_time: chrono::Utc::now(),
                completion_time: chrono::Utc::now(),
                status,
                error_message,
                metadata: HashMap::new(),
            };
            
            // 結果を送信
            if let Err(e) = tx.send(operation_result).await {
                error!("Failed to send operation result: {}", e);
            }
        });
        
        info!("Started shard optimization operation: {}", operation_id);
        
        Ok(operation_id)
    }
    
    /// 全シャードを最適化
    pub async fn optimize_shards(&self) -> Result<String, Error> {
        // シャード最適化が有効かチェック
        if !self.config.enable_shard_optimization {
            return Err(Error::InvalidState("Shard optimization is not enabled".to_string()));
        }
        
        // シャードマネージャーを取得
        let shard_manager = self.shard_manager.read().unwrap();
        let shards = shard_manager.get_all_shards();
        let shard_ids: Vec<ShardId> = shards.iter().map(|s| s.id.clone()).collect();
        
        // 操作IDを生成
        let operation_id = format!("optimize-all-{}", uuid::Uuid::new_v4());
        
        // 操作を作成
        let operation = DynamicShardingOperation {
            id: operation_id.clone(),
            operation_type: DynamicShardingOperationType::Optimization,
            shard_ids: shard_ids.clone(),
            node_ids: None,
            start_time: chrono::Utc::now(),
            estimated_completion_time: chrono::Utc::now() + chrono::Duration::seconds(self.config.shard_optimization_timeout_seconds as i64),
            priority: OperationPriority::Low,
            metadata: HashMap::new(),
        };
        
        // 操作を保留中に追加
        self.pending_operations.insert(operation_id.clone(), operation);
        
        // バックグラウンドで全シャード最適化を実行
        let optimizer = self.optimizer.clone();
        let shard_manager_arc = self.shard_manager.clone();
        let operation_id_clone = operation_id.clone();
        let shard_ids_clone = shard_ids.clone();
        let tx = self.operation_tx.clone();
        
        tokio::spawn(async move {
            let mut success_count = 0;
            let mut failed_shards = Vec::new();
            
            for shard_id in &shard_ids_clone {
                // シャードメトリクスを取得
                let shard_manager = shard_manager_arc.read().unwrap();
                let shard = shard_manager.get_shard(shard_id);
                
                if let Some(shard) = shard {
                    // 最適化戦略を決定
                    let strategy = optimizer.determine_optimization_strategy_from_shard(&shard);
                    
                    // シャードを最適化
                    match optimizer.optimize_shard(shard_id, strategy, shard_manager_arc.clone()).await {
                        Ok(_) => {
                            success_count += 1;
                        },
                        Err(e) => {
                            error!("Failed to optimize shard {}: {}", shard_id, e);
                            failed_shards.push(shard_id.clone());
                        },
                    }
                }
            }
            
            // 操作結果を作成
            let status = if failed_shards.is_empty() {
                OperationStatus::Success
            } else if success_count > 0 {
                OperationStatus::PartialSuccess
            } else {
                OperationStatus::Failed
            };
            
            let error_message = if !failed_shards.is_empty() {
                Some(format!("Failed to optimize {} shards", failed_shards.len()))
            } else {
                None
            };
            
            let operation_result = DynamicShardingOperationResult {
                id: operation_id_clone.clone(),
                operation_type: DynamicShardingOperationType::Optimization,
                shard_ids: shard_ids_clone,
                result_shard_ids: shard_ids_clone.clone(),
                node_ids: None,
                start_time: chrono::Utc::now(),
                completion_time: chrono::Utc::now(),
                status,
                error_message,
                metadata: HashMap::new(),
            };
            
            // 結果を送信
            if let Err(e) = tx.send(operation_result).await {
                error!("Failed to send operation result: {}", e);
            }
        });
        
        info!("Started optimization of all shards: {}", operation_id);
        
        Ok(operation_id)
    }
    
    /// リバランスが必要かどうかをチェック
    async fn should_rebalance(&self) -> bool {
        // リバランスが有効かチェック
        if !self.config.enable_shard_rebalancing {
            return false;
        }
        
        // 前回のリバランスからの経過時間をチェック
        let last_rebalance = self.operation_history.iter()
            .filter(|op| op.operation_type == DynamicShardingOperationType::Rebalance && op.status == OperationStatus::Success)
            .map(|op| op.completion_time)
            .max();
        
        if let Some(last_time) = last_rebalance {
            let now = chrono::Utc::now();
            let elapsed = now.signed_duration_since(last_time);
            
            if elapsed.num_seconds() < self.config.min_rebalance_interval_seconds as i64 {
                return false;
            }
        }
        
        // シャードマネージャーを取得
        let shard_manager = self.shard_manager.read().unwrap();
        
        // ノード間の負荷不均衡をチェック
        let imbalance = self.balancer.calculate_load_imbalance(&shard_manager);
        
        imbalance > self.config.rebalance_threshold
    }
    
    /// 最適化が必要かどうかをチェック
    async fn should_optimize(&self) -> bool {
        // 最適化が有効かチェック
        if !self.config.enable_shard_optimization {
            return false;
        }
        
        // 前回の最適化からの経過時間をチェック
        let last_optimization = self.operation_history.iter()
            .filter(|op| op.operation_type == DynamicShardingOperationType::Optimization && op.status == OperationStatus::Success)
            .map(|op| op.completion_time)
            .max();
        
        if let Some(last_time) = last_optimization {
            let now = chrono::Utc::now();
            let elapsed = now.signed_duration_since(last_time);
            
            if elapsed.num_seconds() < self.config.min_optimization_interval_seconds as i64 {
                return false;
            }
        }
        
        // シャードの最適化スコアをチェック
        let mut needs_optimization = false;
        
        for metrics in self.shard_metrics.values() {
            let optimization_score = self.optimizer.calculate_optimization_score(metrics);
            
            if optimization_score > self.config.optimization_threshold {
                needs_optimization = true;
                break;
            }
        }
        
        needs_optimization
    }
    
    /// 操作結果を取得
    async fn get_operation_result(&self, operation: &DynamicShardingOperation) -> Result<DynamicShardingOperationResult, Error> {
        // 操作タイプに基づいて結果を取得
        match operation.operation_type {
            DynamicShardingOperationType::Split => {
                // シャード分割の結果を取得
                let shard_id = &operation.shard_ids[0];
                let shard_manager = self.shard_manager.read().unwrap();
                
                // 元のシャードが存在するかチェック
                if shard_manager.shard_exists(shard_id) {
                    // 分割が失敗した場合
                    Ok(DynamicShardingOperationResult {
                        id: operation.id.clone(),
                        operation_type: operation.operation_type.clone(),
                        shard_ids: operation.shard_ids.clone(),
                        result_shard_ids: Vec::new(),
                        node_ids: operation.node_ids.clone(),
                        start_time: operation.start_time,
                        completion_time: chrono::Utc::now(),
                        status: OperationStatus::Failed,
                        error_message: Some("Split operation timed out".to_string()),
                        metadata: operation.metadata.clone(),
                    })
                } else {
                    // 分割が成功した場合、子シャードを検索
                    let child_shards: Vec<ShardId> = shard_manager.get_all_shards().iter()
                        .filter(|s| s.parent_id.as_ref().map_or(false, |p| p == shard_id))
                        .map(|s| s.id.clone())
                        .collect();
                    
                    if child_shards.is_empty() {
                        // 子シャードが見つからない場合
                        Ok(DynamicShardingOperationResult {
                            id: operation.id.clone(),
                            operation_type: operation.operation_type.clone(),
                            shard_ids: operation.shard_ids.clone(),
                            result_shard_ids: Vec::new(),
                            node_ids: operation.node_ids.clone(),
                            start_time: operation.start_time,
                            completion_time: chrono::Utc::now(),
                            status: OperationStatus::Failed,
                            error_message: Some("No child shards found after split".to_string()),
                            metadata: operation.metadata.clone(),
                        })
                    } else {
                        // 分割成功
                        Ok(DynamicShardingOperationResult {
                            id: operation.id.clone(),
                            operation_type: operation.operation_type.clone(),
                            shard_ids: operation.shard_ids.clone(),
                            result_shard_ids: child_shards,
                            node_ids: operation.node_ids.clone(),
                            start_time: operation.start_time,
                            completion_time: chrono::Utc::now(),
                            status: OperationStatus::Success,
                            error_message: None,
                            metadata: operation.metadata.clone(),
                        })
                    }
                }
            },
            DynamicShardingOperationType::Merge => {
                // シャードマージの結果を取得
                let shard_ids = &operation.shard_ids;
                let shard_manager = self.shard_manager.read().unwrap();
                
                // 元のシャードが存在するかチェック
                let mut all_exist = true;
                for shard_id in shard_ids {
                    if shard_manager.shard_exists(shard_id) {
                        all_exist = false;
                        break;
                    }
                }
                
                if all_exist {
                    // マージが失敗した場合
                    Ok(DynamicShardingOperationResult {
                        id: operation.id.clone(),
                        operation_type: operation.operation_type.clone(),
                        shard_ids: operation.shard_ids.clone(),
                        result_shard_ids: Vec::new(),
                        node_ids: operation.node_ids.clone(),
                        start_time: operation.start_time,
                        completion_time: chrono::Utc::now(),
                        status: OperationStatus::Failed,
                        error_message: Some("Merge operation timed out".to_string()),
                        metadata: operation.metadata.clone(),
                    })
                } else {
                    // マージが成功した場合、新しいシャードを検索
                    let new_shards: Vec<ShardId> = shard_manager.get_all_shards().iter()
                        .filter(|s| {
                            if let Some(parent_ids) = &s.merged_from {
                                let parent_set: HashSet<_> = parent_ids.iter().collect();
                                let shard_set: HashSet<_> = shard_ids.iter().collect();
                                parent_set == shard_set
                            } else {
                                false
                            }
                        })
                        .map(|s| s.id.clone())
                        .collect();
                    
                    if new_shards.is_empty() {
                        // 新しいシャードが見つからない場合
                        Ok(DynamicShardingOperationResult {
                            id: operation.id.clone(),
                            operation_type: operation.operation_type.clone(),
                            shard_ids: operation.shard_ids.clone(),
                            result_shard_ids: Vec::new(),
                            node_ids: operation.node_ids.clone(),
                            start_time: operation.start_time,
                            completion_time: chrono::Utc::now(),
                            status: OperationStatus::Failed,
                            error_message: Some("No new shard found after merge".to_string()),
                            metadata: operation.metadata.clone(),
                        })
                    } else {
                        // マージ成功
                        Ok(DynamicShardingOperationResult {
                            id: operation.id.clone(),
                            operation_type: operation.operation_type.clone(),
                            shard_ids: operation.shard_ids.clone(),
                            result_shard_ids: new_shards,
                            node_ids: operation.node_ids.clone(),
                            start_time: operation.start_time,
                            completion_time: chrono::Utc::now(),
                            status: OperationStatus::Success,
                            error_message: None,
                            metadata: operation.metadata.clone(),
                        })
                    }
                }
            },
            _ => {
                // その他の操作はタイムアウトとして扱う
                Ok(DynamicShardingOperationResult {
                    id: operation.id.clone(),
                    operation_type: operation.operation_type.clone(),
                    shard_ids: operation.shard_ids.clone(),
                    result_shard_ids: operation.shard_ids.clone(),
                    node_ids: operation.node_ids.clone(),
                    start_time: operation.start_time,
                    completion_time: chrono::Utc::now(),
                    status: OperationStatus::Failed,
                    error_message: Some("Operation timed out".to_string()),
                    metadata: operation.metadata.clone(),
                })
            },
        }
    }
    
    /// 操作履歴を取得
    pub fn get_operation_history(&self) -> &[DynamicShardingOperationResult] {
        &self.operation_history
    }
    
    /// 保留中の操作を取得
    pub fn get_pending_operations(&self) -> &HashMap<String, DynamicShardingOperation> {
        &self.pending_operations
    }
    
    /// シャードメトリクスを取得
    pub fn get_shard_metrics(&self, shard_id: &ShardId) -> Option<&ShardMetrics> {
        self.shard_metrics.get(shard_id)
    }
    
    /// 全シャードメトリクスを取得
    pub fn get_all_shard_metrics(&self) -> &HashMap<ShardId, ShardMetrics> {
        &self.shard_metrics
    }
    
    /// 設定を取得
    pub fn get_config(&self) -> &DynamicShardingConfig {
        &self.config
    }
    
    /// 設定を更新
    pub fn update_config(&mut self, config: DynamicShardingConfig) {
        self.config = config.clone();
        self.splitter.update_config(config.shard_split_policy);
        self.merger.update_config(config.shard_merge_policy);
        self.balancer.update_config(config.rebalance_policy);
        self.hotspot_detector.update_config(config.hotspot_detection_config);
    }
}

impl Clone for DynamicShardingManager {
    fn clone(&self) -> Self {
        let (tx, rx) = mpsc::channel(100);
        
        Self {
            config: self.config.clone(),
            shard_manager: self.shard_manager.clone(),
            balancer: self.balancer.clone(),
            splitter: self.splitter.clone(),
            merger: self.merger.clone(),
            relocator: self.relocator.clone(),
            hotspot_detector: self.hotspot_detector.clone(),
            optimizer: self.optimizer.clone(),
            metrics: self.metrics.clone(),
            shard_metrics: self.shard_metrics.clone(),
            pending_operations: self.pending_operations.clone(),
            operation_history: self.operation_history.clone(),
            operation_tx: tx,
            operation_rx: rx,
            running: self.running,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sharding::test_utils::create_test_shard_manager;
    
    #[tokio::test]
    async fn test_dynamic_sharding_manager_creation() {
        let config = DynamicShardingConfig::default();
        let shard_manager = Arc::new(RwLock::new(create_test_shard_manager()));
        let metrics = Arc::new(MetricsCollector::new("dynamic_sharding"));
        
        let manager = DynamicShardingManager::new(config, shard_manager, metrics);
        
        assert!(!manager.running);
        assert!(manager.shard_metrics.is_empty());
        assert!(manager.pending_operations.is_empty());
        assert!(manager.operation_history.is_empty());
    }
    
    #[tokio::test]
    async fn test_dynamic_sharding_manager_start_stop() {
        let config = DynamicShardingConfig::default();
        let shard_manager = Arc::new(RwLock::new(create_test_shard_manager()));
        let metrics = Arc::new(MetricsCollector::new("dynamic_sharding"));
        
        let mut manager = DynamicShardingManager::new(config, shard_manager, metrics);
        
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
    async fn test_collect_metrics() {
        let config = DynamicShardingConfig::default();
        let shard_manager = Arc::new(RwLock::new(create_test_shard_manager()));
        let metrics = Arc::new(MetricsCollector::new("dynamic_sharding"));
        
        let manager = DynamicShardingManager::new(config, shard_manager, metrics);
        
        // メトリクスを収集
        let result = manager.collect_metrics().await;
        assert!(result.is_ok());
        
        // シャードメトリクスが収集されたことを確認
        assert!(!manager.shard_metrics.is_empty());
        
        // 各シャードのメトリクスをチェック
        for (shard_id, metrics) in &manager.shard_metrics {
            assert_eq!(&metrics.shard_id, shard_id);
            assert!(!metrics.load_metrics.is_empty());
            assert!(!metrics.distribution_metrics.is_empty());
        }
    }
    
    #[tokio::test]
    async fn test_split_shard() {
        let mut config = DynamicShardingConfig::default();
        config.enable_shard_splitting = true;
        
        let shard_manager = Arc::new(RwLock::new(create_test_shard_manager()));
        let metrics = Arc::new(MetricsCollector::new("dynamic_sharding"));
        
        let mut manager = DynamicShardingManager::new(config, shard_manager.clone(), metrics);
        
        // メトリクスを収集
        let _ = manager.collect_metrics().await;
        
        // シャードを取得
        let shard_id = {
            let sm = shard_manager.read().unwrap();
            let shards = sm.get_all_shards();
            assert!(!shards.is_empty());
            shards[0].id.clone()
        };
        
        // シャードを分割
        let result = manager.split_shard(&shard_id).await;
        assert!(result.is_ok());
        
        // 操作IDを取得
        let operation_id = result.unwrap();
        
        // 保留中の操作に追加されたことを確認
        assert!(manager.pending_operations.contains_key(&operation_id));
        
        // 操作タイプを確認
        let operation = &manager.pending_operations[&operation_id];
        assert_eq!(operation.operation_type, DynamicShardingOperationType::Split);
        assert_eq!(operation.shard_ids, vec![shard_id]);
    }
    
    #[tokio::test]
    async fn test_merge_shards() {
        let mut config = DynamicShardingConfig::default();
        config.enable_shard_merging = true;
        
        let shard_manager = Arc::new(RwLock::new(create_test_shard_manager()));
        let metrics = Arc::new(MetricsCollector::new("dynamic_sharding"));
        
        let mut manager = DynamicShardingManager::new(config, shard_manager.clone(), metrics);
        
        // メトリクスを収集
        let _ = manager.collect_metrics().await;
        
        // シャードを取得
        let shard_ids = {
            let sm = shard_manager.read().unwrap();
            let shards = sm.get_all_shards();
            assert!(shards.len() >= 2);
            vec![shards[0].id.clone(), shards[1].id.clone()]
        };
        
        // シャードをマージ
        let result = manager.merge_shards(&shard_ids).await;
        assert!(result.is_ok());
        
        // 操作IDを取得
        let operation_id = result.unwrap();
        
        // 保留中の操作に追加されたことを確認
        assert!(manager.pending_operations.contains_key(&operation_id));
        
        // 操作タイプを確認
        let operation = &manager.pending_operations[&operation_id];
        assert_eq!(operation.operation_type, DynamicShardingOperationType::Merge);
        assert_eq!(operation.shard_ids, shard_ids);
    }
    
    #[tokio::test]
    async fn test_relocate_shard() {
        let mut config = DynamicShardingConfig::default();
        config.enable_shard_relocation = true;
        
        let shard_manager = Arc::new(RwLock::new(create_test_shard_manager()));
        let metrics = Arc::new(MetricsCollector::new("dynamic_sharding"));
        
        let mut manager = DynamicShardingManager::new(config, shard_manager.clone(), metrics);
        
        // メトリクスを収集
        let _ = manager.collect_metrics().await;
        
        // シャードを取得
        let shard_id = {
            let sm = shard_manager.read().unwrap();
            let shards = sm.get_all_shards();
            assert!(!shards.is_empty());
            shards[0].id.clone()
        };
        
        // シャードを再配置
        let result = manager.relocate_shard(&shard_id).await;
        assert!(result.is_ok());
        
        // 操作IDを取得
        let operation_id = result.unwrap();
        
        // 保留中の操作に追加されたことを確認
        assert!(manager.pending_operations.contains_key(&operation_id));
        
        // 操作タイプを確認
        let operation = &manager.pending_operations[&operation_id];
        assert_eq!(operation.operation_type, DynamicShardingOperationType::Relocate);
        assert_eq!(operation.shard_ids, vec![shard_id]);
    }
}