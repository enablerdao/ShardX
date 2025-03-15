use std::collections::{HashMap, HashSet, BinaryHeap};
use std::cmp::Ordering;
use std::sync::{Arc, Mutex, RwLock};
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::shard::{ShardId, ShardManager};
use crate::network::PeerInfo;

/// シャード間接続情報
#[derive(Debug, Clone)]
pub struct ShardConnection {
    /// 送信元シャード
    pub source: ShardId,
    /// 送信先シャード
    pub destination: ShardId,
    /// レイテンシ（ミリ秒）
    pub latency_ms: u64,
    /// 帯域幅（バイト/秒）
    pub bandwidth_bps: u64,
    /// 信頼性（0.0-1.0）
    pub reliability: f64,
    /// 負荷（0.0-1.0）
    pub load: f64,
    /// 有効かどうか
    pub enabled: bool,
}

/// ルーティングテーブル
#[derive(Debug)]
pub struct RoutingTable {
    /// シャード間接続
    connections: HashMap<(ShardId, ShardId), ShardConnection>,
    /// 最短経路キャッシュ
    shortest_paths: RwLock<HashMap<(ShardId, ShardId), Vec<ShardId>>>,
    /// 最終更新時刻
    last_updated: std::time::Instant,
}

impl RoutingTable {
    /// 新しいルーティングテーブルを作成
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            shortest_paths: RwLock::new(HashMap::new()),
            last_updated: std::time::Instant::now(),
        }
    }
    
    /// 接続を追加
    pub fn add_connection(&mut self, connection: ShardConnection) {
        let key = (connection.source.clone(), connection.destination.clone());
        self.connections.insert(key, connection);
        
        // キャッシュをクリア
        let mut shortest_paths = self.shortest_paths.write().unwrap();
        shortest_paths.clear();
        
        self.last_updated = std::time::Instant::now();
    }
    
    /// 接続を更新
    pub fn update_connection(
        &mut self,
        source: &ShardId,
        destination: &ShardId,
        latency_ms: Option<u64>,
        bandwidth_bps: Option<u64>,
        reliability: Option<f64>,
        load: Option<f64>,
        enabled: Option<bool>,
    ) -> Result<(), Error> {
        let key = (source.clone(), destination.clone());
        let connection = self.connections.get_mut(&key).ok_or_else(|| {
            Error::NotFound(format!("接続 {} -> {} が見つかりません", source, destination))
        })?;
        
        if let Some(latency) = latency_ms {
            connection.latency_ms = latency;
        }
        
        if let Some(bandwidth) = bandwidth_bps {
            connection.bandwidth_bps = bandwidth;
        }
        
        if let Some(reliability) = reliability {
            connection.reliability = reliability;
        }
        
        if let Some(load) = load {
            connection.load = load;
        }
        
        if let Some(enabled) = enabled {
            connection.enabled = enabled;
        }
        
        // キャッシュをクリア
        let mut shortest_paths = self.shortest_paths.write().unwrap();
        shortest_paths.clear();
        
        self.last_updated = std::time::Instant::now();
        
        Ok(())
    }
    
    /// 接続を削除
    pub fn remove_connection(&mut self, source: &ShardId, destination: &ShardId) -> Result<(), Error> {
        let key = (source.clone(), destination.clone());
        if self.connections.remove(&key).is_none() {
            return Err(Error::NotFound(format!("接続 {} -> {} が見つかりません", source, destination)));
        }
        
        // キャッシュをクリア
        let mut shortest_paths = self.shortest_paths.write().unwrap();
        shortest_paths.clear();
        
        self.last_updated = std::time::Instant::now();
        
        Ok(())
    }
    
    /// 接続を取得
    pub fn get_connection(&self, source: &ShardId, destination: &ShardId) -> Result<ShardConnection, Error> {
        let key = (source.clone(), destination.clone());
        let connection = self.connections.get(&key).ok_or_else(|| {
            Error::NotFound(format!("接続 {} -> {} が見つかりません", source, destination))
        })?;
        
        Ok(connection.clone())
    }
    
    /// 全接続を取得
    pub fn get_all_connections(&self) -> Vec<ShardConnection> {
        self.connections.values().cloned().collect()
    }
    
    /// シャードの全接続を取得
    pub fn get_shard_connections(&self, shard_id: &ShardId) -> Vec<ShardConnection> {
        self.connections.values()
            .filter(|conn| &conn.source == shard_id || &conn.destination == shard_id)
            .cloned()
            .collect()
    }
    
    /// 最短経路を計算
    pub fn calculate_shortest_path(
        &self,
        source: &ShardId,
        destination: &ShardId,
        optimization_criteria: OptimizationCriteria,
    ) -> Result<Vec<ShardId>, Error> {
        // キャッシュをチェック
        let cache_key = (source.clone(), destination.clone());
        {
            let shortest_paths = self.shortest_paths.read().unwrap();
            if let Some(path) = shortest_paths.get(&cache_key) {
                return Ok(path.clone());
            }
        }
        
        // 同一シャードの場合は空のパスを返す
        if source == destination {
            return Ok(Vec::new());
        }
        
        // ダイクストラ法で最短経路を計算
        let path = self.dijkstra(source, destination, optimization_criteria)?;
        
        // キャッシュに保存
        let mut shortest_paths = self.shortest_paths.write().unwrap();
        shortest_paths.insert(cache_key, path.clone());
        
        Ok(path)
    }
    
    /// ダイクストラ法による最短経路計算
    fn dijkstra(
        &self,
        source: &ShardId,
        destination: &ShardId,
        criteria: OptimizationCriteria,
    ) -> Result<Vec<ShardId>, Error> {
        // 全シャードのセットを作成
        let mut all_shards = HashSet::new();
        for (src, dst) in self.connections.keys() {
            all_shards.insert(src.clone());
            all_shards.insert(dst.clone());
        }
        
        // 送信元または送信先シャードが存在しない場合はエラー
        if !all_shards.contains(source) {
            return Err(Error::NotFound(format!("シャード {} が見つかりません", source)));
        }
        
        if !all_shards.contains(destination) {
            return Err(Error::NotFound(format!("シャード {} が見つかりません", destination)));
        }
        
        // 距離と前のノードを初期化
        let mut distances: HashMap<ShardId, f64> = all_shards.iter()
            .map(|shard| (shard.clone(), f64::INFINITY))
            .collect();
        
        let mut previous: HashMap<ShardId, Option<ShardId>> = all_shards.iter()
            .map(|shard| (shard.clone(), None))
            .collect();
        
        // 送信元の距離を0に設定
        distances.insert(source.clone(), 0.0);
        
        // 優先度付きキューを初期化
        let mut queue = BinaryHeap::new();
        queue.push(State {
            shard: source.clone(),
            cost: 0.0,
        });
        
        // ダイクストラ法のメインループ
        while let Some(State { shard, cost }) = queue.pop() {
            // 送信先に到達した場合は終了
            if &shard == destination {
                break;
            }
            
            // より良い経路が既に見つかっている場合はスキップ
            if cost > *distances.get(&shard).unwrap_or(&f64::INFINITY) {
                continue;
            }
            
            // 隣接シャードを探索
            for neighbor in self.get_neighbors(&shard) {
                let connection = self.get_connection(&shard, &neighbor).unwrap();
                
                // 無効な接続はスキップ
                if !connection.enabled {
                    continue;
                }
                
                // コストを計算
                let edge_cost = match criteria {
                    OptimizationCriteria::Latency => connection.latency_ms as f64,
                    OptimizationCriteria::Bandwidth => 1_000_000_000.0 / connection.bandwidth_bps as f64,
                    OptimizationCriteria::Reliability => 1.0 - connection.reliability,
                    OptimizationCriteria::Load => connection.load,
                    OptimizationCriteria::Combined => {
                        // 複合的な評価関数
                        let latency_factor = connection.latency_ms as f64 / 100.0;
                        let bandwidth_factor = 1_000_000_000.0 / connection.bandwidth_bps as f64;
                        let reliability_factor = 1.0 - connection.reliability;
                        let load_factor = connection.load;
                        
                        0.4 * latency_factor + 0.2 * bandwidth_factor + 0.2 * reliability_factor + 0.2 * load_factor
                    }
                };
                
                let next_cost = cost + edge_cost;
                
                // より良い経路が見つかった場合は更新
                if next_cost < *distances.get(&neighbor).unwrap_or(&f64::INFINITY) {
                    distances.insert(neighbor.clone(), next_cost);
                    previous.insert(neighbor.clone(), Some(shard.clone()));
                    queue.push(State {
                        shard: neighbor.clone(),
                        cost: next_cost,
                    });
                }
            }
        }
        
        // 経路を構築
        let mut path = Vec::new();
        let mut current = destination.clone();
        
        // 送信先に到達できない場合はエラー
        if previous.get(&current).unwrap_or(&None).is_none() {
            return Err(Error::NotFound(format!("シャード {} から {} への経路が見つかりません", source, destination)));
        }
        
        // 経路を逆順に構築
        while current != *source {
            path.push(current.clone());
            current = previous.get(&current).unwrap().as_ref().unwrap().clone();
        }
        
        // 経路を正順に変換
        path.reverse();
        
        Ok(path)
    }
    
    /// シャードの隣接シャードを取得
    fn get_neighbors(&self, shard_id: &ShardId) -> Vec<ShardId> {
        self.connections.iter()
            .filter_map(|((src, dst), conn)| {
                if src == shard_id && conn.enabled {
                    Some(dst.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// ルーティングテーブルを最適化
    pub fn optimize(&mut self) {
        // 使用されていない接続を削除
        self.connections.retain(|_, conn| {
            conn.enabled && conn.reliability > 0.0
        });
        
        // キャッシュをクリア
        let mut shortest_paths = self.shortest_paths.write().unwrap();
        shortest_paths.clear();
        
        self.last_updated = std::time::Instant::now();
    }
    
    /// ルーティングテーブルを更新
    pub fn update_from_network(&mut self, shard_manager: &ShardManager) -> Result<(), Error> {
        // 全シャードの情報を取得
        let shards = shard_manager.get_all_shards()?;
        
        // 接続情報を更新
        for source_shard in &shards {
            let peers = source_shard.get_peers()?;
            
            for peer in peers {
                if let Some(peer_shard_id) = shard_manager.get_shard_id_by_peer(&peer) {
                    // 既存の接続を取得または新規作成
                    let key = (source_shard.id.clone(), peer_shard_id.clone());
                    let connection = self.connections.entry(key).or_insert_with(|| {
                        ShardConnection {
                            source: source_shard.id.clone(),
                            destination: peer_shard_id.clone(),
                            latency_ms: 100, // デフォルト値
                            bandwidth_bps: 1_000_000, // デフォルト値
                            reliability: 0.9, // デフォルト値
                            load: 0.5, // デフォルト値
                            enabled: true,
                        }
                    });
                    
                    // 接続情報を更新
                    connection.latency_ms = peer.latency_ms.unwrap_or(connection.latency_ms);
                    connection.bandwidth_bps = peer.bandwidth_bps.unwrap_or(connection.bandwidth_bps);
                    connection.reliability = peer.reliability.unwrap_or(connection.reliability);
                    connection.load = peer.load.unwrap_or(connection.load);
                    connection.enabled = peer.connected;
                }
            }
        }
        
        // キャッシュをクリア
        let mut shortest_paths = self.shortest_paths.write().unwrap();
        shortest_paths.clear();
        
        self.last_updated = std::time::Instant::now();
        
        Ok(())
    }
}

/// 最適化基準
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationCriteria {
    /// レイテンシ
    Latency,
    /// 帯域幅
    Bandwidth,
    /// 信頼性
    Reliability,
    /// 負荷
    Load,
    /// 複合
    Combined,
}

/// ダイクストラ法の状態
#[derive(Debug, Clone)]
struct State {
    /// シャードID
    shard: ShardId,
    /// コスト
    cost: f64,
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.cost.eq(&other.cost)
    }
}

impl Eq for State {}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // コストが小さい方が優先度が高い（最小ヒープ）
        // f64は直接比較できないため、逆順にして最大ヒープを最小ヒープとして使用
        other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
    }
}

/// ルーティングマネージャー
pub struct RoutingManager {
    /// ルーティングテーブル
    routing_table: Arc<Mutex<RoutingTable>>,
    /// シャードマネージャー
    shard_manager: Arc<ShardManager>,
    /// 最適化基準
    optimization_criteria: OptimizationCriteria,
    /// 更新間隔（秒）
    update_interval_seconds: u64,
    /// 最終更新時刻
    last_updated: std::time::Instant,
}

impl RoutingManager {
    /// 新しいルーティングマネージャーを作成
    pub fn new(
        shard_manager: Arc<ShardManager>,
        optimization_criteria: Option<OptimizationCriteria>,
        update_interval_seconds: Option<u64>,
    ) -> Self {
        Self {
            routing_table: Arc::new(Mutex::new(RoutingTable::new())),
            shard_manager,
            optimization_criteria: optimization_criteria.unwrap_or(OptimizationCriteria::Combined),
            update_interval_seconds: update_interval_seconds.unwrap_or(60),
            last_updated: std::time::Instant::now() - std::time::Duration::from_secs(3600), // 初回更新を強制
        }
    }
    
    /// 最適な経路を計算
    pub fn calculate_optimal_route(
        &self,
        source: &ShardId,
        destination: &ShardId,
    ) -> Result<Vec<ShardId>, Error> {
        // 必要に応じてルーティングテーブルを更新
        self.update_if_needed()?;
        
        // ルーティングテーブルから最短経路を計算
        let routing_table = self.routing_table.lock().unwrap();
        routing_table.calculate_shortest_path(source, destination, self.optimization_criteria)
    }
    
    /// 必要に応じてルーティングテーブルを更新
    fn update_if_needed(&self) -> Result<(), Error> {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_updated).as_secs();
        
        if elapsed >= self.update_interval_seconds {
            let mut routing_table = self.routing_table.lock().unwrap();
            routing_table.update_from_network(&self.shard_manager)?;
            routing_table.optimize();
            
            // 最終更新時刻を更新
            let mut last_updated = self.last_updated;
            last_updated = now;
        }
        
        Ok(())
    }
    
    /// 最適化基準を設定
    pub fn set_optimization_criteria(&mut self, criteria: OptimizationCriteria) {
        self.optimization_criteria = criteria;
    }
    
    /// 更新間隔を設定
    pub fn set_update_interval(&mut self, interval_seconds: u64) {
        self.update_interval_seconds = interval_seconds;
    }
    
    /// ルーティングテーブルを強制的に更新
    pub fn force_update(&self) -> Result<(), Error> {
        let mut routing_table = self.routing_table.lock().unwrap();
        routing_table.update_from_network(&self.shard_manager)?;
        routing_table.optimize();
        
        // 最終更新時刻を更新
        let mut last_updated = self.last_updated;
        last_updated = std::time::Instant::now();
        
        Ok(())
    }
    
    /// 全接続を取得
    pub fn get_all_connections(&self) -> Result<Vec<ShardConnection>, Error> {
        let routing_table = self.routing_table.lock().unwrap();
        Ok(routing_table.get_all_connections())
    }
    
    /// シャードの全接続を取得
    pub fn get_shard_connections(&self, shard_id: &ShardId) -> Result<Vec<ShardConnection>, Error> {
        let routing_table = self.routing_table.lock().unwrap();
        Ok(routing_table.get_shard_connections(shard_id))
    }
}