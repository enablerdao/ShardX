use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::transaction::{Transaction, TransactionStatus, CrossShardManager, CrossShardTransaction, CrossShardTransactionState};
use crate::shard::{ShardId, ShardManager, ShardInfo, ShardStatus};
use crate::network::NetworkMessage;

/// クロスシャードトランザクション最適化器
///
/// クロスシャードトランザクションのパフォーマンスを最適化するための機能を提供します。
/// 主な最適化手法:
/// 1. バッチ処理 - 複数のトランザクションをバッチ化して処理
/// 2. ルーティング最適化 - 最適なシャード間パスを選択
/// 3. 並列実行 - 依存関係のないトランザクションを並列処理
/// 4. キャッシング - 頻繁にアクセスされるデータをキャッシュ
pub struct CrossShardOptimizer {
    /// クロスシャードマネージャー
    cross_shard_manager: Arc<CrossShardManager>,
    /// シャードマネージャー
    shard_manager: Arc<ShardManager>,
    /// ネットワークメッセージ送信チャネル
    network_tx: mpsc::Sender<NetworkMessage>,
    /// バッチ処理キュー
    batch_queue: Arc<Mutex<HashMap<ShardId, VecDeque<Transaction>>>>,
    /// シャード間ルーティングテーブル
    routing_table: Arc<Mutex<HashMap<(ShardId, ShardId), Vec<ShardId>>>>,
    /// シャードパフォーマンスメトリクス
    shard_metrics: Arc<Mutex<HashMap<ShardId, ShardMetrics>>>,
    /// 最適化設定
    config: OptimizerConfig,
}

/// シャードメトリクス
#[derive(Debug, Clone)]
struct ShardMetrics {
    /// 平均処理時間（ミリ秒）
    avg_processing_time_ms: f64,
    /// 平均レイテンシ（ミリ秒）
    avg_latency_ms: f64,
    /// 成功率（0-1）
    success_rate: f64,
    /// 現在の負荷（0-1）
    current_load: f64,
    /// 最終更新時刻
    last_updated: u64,
}

/// 最適化設定
#[derive(Debug, Clone)]
pub struct OptimizerConfig {
    /// バッチサイズ
    pub batch_size: usize,
    /// バッチ処理間隔（ミリ秒）
    pub batch_interval_ms: u64,
    /// ルーティングテーブル更新間隔（秒）
    pub routing_update_interval_sec: u64,
    /// メトリクス更新間隔（秒）
    pub metrics_update_interval_sec: u64,
    /// 並列実行の最大数
    pub max_parallel_executions: usize,
    /// キャッシュの有効期限（秒）
    pub cache_expiry_sec: u64,
    /// 再試行回数
    pub max_retries: usize,
    /// 再試行間隔（ミリ秒）
    pub retry_interval_ms: u64,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            batch_size: 50,
            batch_interval_ms: 100,
            routing_update_interval_sec: 60,
            metrics_update_interval_sec: 30,
            max_parallel_executions: 10,
            cache_expiry_sec: 300,
            max_retries: 3,
            retry_interval_ms: 1000,
        }
    }
}

impl CrossShardOptimizer {
    /// 新しいクロスシャード最適化器を作成
    pub fn new(
        cross_shard_manager: Arc<CrossShardManager>,
        shard_manager: Arc<ShardManager>,
        network_tx: mpsc::Sender<NetworkMessage>,
        config: Option<OptimizerConfig>,
    ) -> Self {
        let config = config.unwrap_or_default();
        
        Self {
            cross_shard_manager,
            shard_manager,
            network_tx,
            batch_queue: Arc::new(Mutex::new(HashMap::new())),
            routing_table: Arc::new(Mutex::new(HashMap::new())),
            shard_metrics: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }
    
    /// 最適化器を起動
    pub fn start(&self) {
        info!("Starting CrossShardOptimizer");
        
        // バッチ処理タスクを開始
        self.start_batch_processor();
        
        // ルーティングテーブル更新タスクを開始
        self.start_routing_updater();
        
        // メトリクス更新タスクを開始
        self.start_metrics_updater();
    }
    
    /// トランザクションをキューに追加
    pub fn enqueue_transaction(&self, transaction: Transaction) -> Result<(), Error> {
        let shard_id = transaction.shard_id.clone();
        
        let mut batch_queue = self.batch_queue.lock().unwrap();
        
        let queue = batch_queue.entry(shard_id).or_insert_with(VecDeque::new);
        queue.push_back(transaction);
        
        Ok(())
    }
    
    /// バッチ処理タスクを開始
    fn start_batch_processor(&self) {
        let batch_queue = self.batch_queue.clone();
        let cross_shard_manager = self.cross_shard_manager.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(config.batch_interval_ms)).await;
                
                let batches = {
                    let mut batch_queue = batch_queue.lock().unwrap();
                    let mut batches = HashMap::new();
                    
                    for (shard_id, queue) in batch_queue.iter_mut() {
                        if !queue.is_empty() {
                            let batch_size = std::cmp::min(queue.len(), config.batch_size);
                            let mut batch = Vec::with_capacity(batch_size);
                            
                            for _ in 0..batch_size {
                                if let Some(tx) = queue.pop_front() {
                                    batch.push(tx);
                                }
                            }
                            
                            batches.insert(shard_id.clone(), batch);
                        }
                    }
                    
                    batches
                };
                
                // バッチごとに並列処理
                for (shard_id, batch) in batches {
                    if batch.is_empty() {
                        continue;
                    }
                    
                    let cross_shard_manager = cross_shard_manager.clone();
                    
                    tokio::spawn(async move {
                        debug!("Processing batch of {} transactions for shard {}", batch.len(), shard_id);
                        
                        // 並列処理の制限
                        let semaphore = tokio::sync::Semaphore::new(config.max_parallel_executions);
                        
                        let mut handles = Vec::new();
                        
                        for tx in batch {
                            let permit = semaphore.acquire().await.unwrap();
                            let cross_shard_manager = cross_shard_manager.clone();
                            
                            let handle = tokio::spawn(async move {
                                let result = cross_shard_manager.start_transaction(tx.clone()).await;
                                
                                if let Err(e) = &result {
                                    error!("Failed to start cross-shard transaction: {}", e);
                                }
                                
                                drop(permit);
                                result
                            });
                            
                            handles.push(handle);
                        }
                        
                        // すべてのトランザクションが完了するのを待つ
                        for handle in handles {
                            let _ = handle.await;
                        }
                    });
                }
            }
        });
    }
    
    /// ルーティングテーブル更新タスクを開始
    fn start_routing_updater(&self) {
        let routing_table = self.routing_table.clone();
        let shard_manager = self.shard_manager.clone();
        let shard_metrics = self.shard_metrics.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(config.routing_update_interval_sec)).await;
                
                match update_routing_table(shard_manager.clone(), shard_metrics.clone()).await {
                    Ok(new_table) => {
                        let mut routing_table = routing_table.lock().unwrap();
                        *routing_table = new_table;
                        debug!("Updated routing table with {} routes", routing_table.len());
                    }
                    Err(e) => {
                        error!("Failed to update routing table: {}", e);
                    }
                }
            }
        });
    }
    
    /// メトリクス更新タスクを開始
    fn start_metrics_updater(&self) {
        let shard_metrics = self.shard_metrics.clone();
        let shard_manager = self.shard_manager.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(config.metrics_update_interval_sec)).await;
                
                match update_shard_metrics(shard_manager.clone()).await {
                    Ok(new_metrics) => {
                        let mut metrics = shard_metrics.lock().unwrap();
                        *metrics = new_metrics;
                        debug!("Updated shard metrics for {} shards", metrics.len());
                    }
                    Err(e) => {
                        error!("Failed to update shard metrics: {}", e);
                    }
                }
            }
        });
    }
    
    /// 最適なシャードパスを取得
    pub fn get_optimal_path(&self, from_shard: &ShardId, to_shard: &ShardId) -> Vec<ShardId> {
        let routing_table = self.routing_table.lock().unwrap();
        
        routing_table.get(&(from_shard.clone(), to_shard.clone()))
            .cloned()
            .unwrap_or_else(|| vec![from_shard.clone(), to_shard.clone()])
    }
    
    /// シャードのパフォーマンスメトリクスを取得
    pub fn get_shard_metrics(&self, shard_id: &ShardId) -> Option<ShardMetrics> {
        let metrics = self.shard_metrics.lock().unwrap();
        metrics.get(shard_id).cloned()
    }
    
    /// すべてのシャードのパフォーマンスメトリクスを取得
    pub fn get_all_shard_metrics(&self) -> HashMap<ShardId, ShardMetrics> {
        let metrics = self.shard_metrics.lock().unwrap();
        metrics.clone()
    }
    
    /// 最適化設定を更新
    pub fn update_config(&mut self, config: OptimizerConfig) {
        self.config = config;
    }
    
    /// 現在の最適化設定を取得
    pub fn get_config(&self) -> OptimizerConfig {
        self.config.clone()
    }
}

/// ルーティングテーブルを更新
async fn update_routing_table(
    shard_manager: Arc<ShardManager>,
    shard_metrics: Arc<Mutex<HashMap<ShardId, ShardMetrics>>>,
) -> Result<HashMap<(ShardId, ShardId), Vec<ShardId>>, Error> {
    // アクティブなシャードを取得
    let shards = shard_manager.get_active_shards().await?;
    
    // シャード間の接続情報を取得
    let mut connections = HashMap::new();
    
    for shard in &shards {
        let shard_connections = shard_manager.get_connections_from(&shard.id)?;
        
        for conn in shard_connections {
            connections.insert((conn.from.clone(), conn.to.clone()), conn.latency);
        }
    }
    
    // シャードメトリクスを取得
    let metrics = shard_metrics.lock().unwrap();
    
    // ルーティングテーブルを計算（Dijkstraアルゴリズム）
    let mut routing_table = HashMap::new();
    
    for from_shard in &shards {
        for to_shard in &shards {
            if from_shard.id == to_shard.id {
                continue;
            }
            
            let path = calculate_shortest_path(
                &from_shard.id,
                &to_shard.id,
                &shards,
                &connections,
                &metrics,
            );
            
            routing_table.insert((from_shard.id.clone(), to_shard.id.clone()), path);
        }
    }
    
    Ok(routing_table)
}

/// 最短パスを計算（Dijkstraアルゴリズム）
fn calculate_shortest_path(
    from_shard: &ShardId,
    to_shard: &ShardId,
    shards: &[ShardInfo],
    connections: &HashMap<(ShardId, ShardId), u64>,
    metrics: &HashMap<ShardId, ShardMetrics>,
) -> Vec<ShardId> {
    // シャードIDのセットを作成
    let shard_ids: HashSet<ShardId> = shards.iter().map(|s| s.id.clone()).collect();
    
    // 距離テーブルを初期化
    let mut distances: HashMap<ShardId, u64> = shard_ids.iter()
        .map(|id| (id.clone(), if id == from_shard { 0 } else { u64::MAX }))
        .collect();
    
    // 前のノードを追跡
    let mut previous: HashMap<ShardId, Option<ShardId>> = shard_ids.iter()
        .map(|id| (id.clone(), None))
        .collect();
    
    // 未訪問のノード
    let mut unvisited = shard_ids.clone();
    
    // Dijkstraアルゴリズム
    while !unvisited.is_empty() {
        // 最小距離のノードを見つける
        let current = unvisited.iter()
            .min_by_key(|id| distances.get(*id).unwrap_or(&u64::MAX))
            .cloned();
        
        if let Some(current) = current {
            // 目的地に到達した場合は終了
            if current == *to_shard {
                break;
            }
            
            // 現在のノードを訪問済みにする
            unvisited.remove(&current);
            
            // 現在の距離を取得
            let current_distance = *distances.get(&current).unwrap_or(&u64::MAX);
            
            // 無限大の距離の場合はスキップ
            if current_distance == u64::MAX {
                continue;
            }
            
            // 隣接ノードを処理
            for neighbor in &shard_ids {
                if !unvisited.contains(neighbor) {
                    continue;
                }
                
                // 接続情報を取得
                let edge_weight = connections.get(&(current.clone(), neighbor.clone()))
                    .copied()
                    .unwrap_or(u64::MAX);
                
                // 無効な接続はスキップ
                if edge_weight == u64::MAX {
                    continue;
                }
                
                // シャードのパフォーマンスメトリクスを考慮した重み
                let performance_weight = calculate_performance_weight(neighbor, metrics);
                
                // 合計重み
                let total_weight = edge_weight + performance_weight;
                
                // 新しい距離を計算
                let new_distance = current_distance.saturating_add(total_weight);
                
                // より短いパスが見つかった場合は更新
                if new_distance < *distances.get(neighbor).unwrap_or(&u64::MAX) {
                    distances.insert(neighbor.clone(), new_distance);
                    previous.insert(neighbor.clone(), Some(current.clone()));
                }
            }
        } else {
            break;
        }
    }
    
    // パスを再構築
    let mut path = Vec::new();
    let mut current = to_shard.clone();
    
    path.push(current.clone());
    
    while let Some(Some(prev)) = previous.get(&current) {
        path.push(prev.clone());
        current = prev.clone();
        
        if current == *from_shard {
            break;
        }
    }
    
    // パスを反転
    path.reverse();
    
    // パスが有効かチェック
    if path.first().map(|id| id == from_shard).unwrap_or(false) && 
       path.last().map(|id| id == to_shard).unwrap_or(false) {
        path
    } else {
        // 有効なパスが見つからない場合は直接パスを返す
        vec![from_shard.clone(), to_shard.clone()]
    }
}

/// シャードのパフォーマンスに基づく重みを計算
fn calculate_performance_weight(
    shard_id: &ShardId,
    metrics: &HashMap<ShardId, ShardMetrics>,
) -> u64 {
    if let Some(metric) = metrics.get(shard_id) {
        // レイテンシと負荷に基づく重み
        let latency_weight = (metric.avg_latency_ms as f64 * 0.5) as u64;
        let load_weight = (metric.current_load * 100.0) as u64;
        
        // 成功率に基づく重み（成功率が低いほど重みが大きい）
        let success_weight = ((1.0 - metric.success_rate) * 200.0) as u64;
        
        latency_weight + load_weight + success_weight
    } else {
        // メトリクスがない場合はデフォルトの重み
        50
    }
}

/// シャードメトリクスを更新
async fn update_shard_metrics(
    shard_manager: Arc<ShardManager>,
) -> Result<HashMap<ShardId, ShardMetrics>, Error> {
    // アクティブなシャードを取得
    let shards = shard_manager.get_active_shards().await?;
    
    let mut metrics = HashMap::new();
    let now = chrono::Utc::now().timestamp() as u64;
    
    for shard in shards {
        // 実際の実装では、シャードから実際のメトリクスを取得
        // ここでは簡易的な実装として、ダミーデータを生成
        
        let avg_processing_time_ms = 50.0 + (rand::random::<f64>() * 50.0);
        let avg_latency_ms = 20.0 + (rand::random::<f64>() * 30.0);
        let success_rate = 0.95 + (rand::random::<f64>() * 0.05);
        let current_load = rand::random::<f64>() * 0.8;
        
        metrics.insert(shard.id, ShardMetrics {
            avg_processing_time_ms,
            avg_latency_ms,
            success_rate,
            current_load,
            last_updated: now,
        });
    }
    
    Ok(metrics)
}