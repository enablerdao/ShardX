use std::collections::HashMap;
use crate::error::Error;

/// シャードID型
pub type ShardId = u32;
/// ノードID型
pub type NodeId = String;

/// シャードタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShardType {
    /// 軽量シャード（低スペックノード向け）
    Lightweight,
    /// 標準シャード（中スペックノード向け）
    Standard,
    /// 高負荷シャード（高スペックノード向け）
    HighLoad,
}

/// ノードスペック情報
#[derive(Debug, Clone)]
pub struct NodeSpec {
    /// CPUコア数
    pub cpu_cores: u32,
    /// メモリ容量（GB）
    pub memory_gb: u32,
    /// 高スペックノードかどうか
    pub is_high_spec: bool,
}

/// シャード情報
#[derive(Debug, Clone)]
pub struct Shard {
    /// シャードID
    pub id: ShardId,
    /// シャードタイプ
    pub shard_type: ShardType,
    /// 割り当てられたノード
    pub nodes: Vec<NodeId>,
    /// 現在の負荷（0.0〜1.0）
    pub load: f32,
}

impl Shard {
    /// 新しいシャードを作成
    pub fn new(id: ShardId, shard_type: ShardType) -> Self {
        Self {
            id,
            shard_type,
            nodes: Vec::new(),
            load: 0.0,
        }
    }
    
    /// ノードをシャードに割り当て
    pub fn assign_node(&mut self, node_id: NodeId) {
        if !self.nodes.contains(&node_id) {
            self.nodes.push(node_id);
        }
    }
    
    /// ノードをシャードから削除
    pub fn remove_node(&mut self, node_id: &NodeId) {
        self.nodes.retain(|id| id != node_id);
    }
    
    /// 負荷を更新
    pub fn update_load(&mut self, load: f32) {
        self.load = load.max(0.0).min(1.0);
    }
}

/// シャード管理マネージャー
pub struct ShardManager {
    /// シャード数
    pub shard_count: usize,
    /// シャード情報
    pub shards: Vec<Shard>,
    /// ノードスペック情報
    pub node_specs: HashMap<NodeId, NodeSpec>,
}

impl ShardManager {
    /// 新しいShardManagerを作成
    pub fn new(initial_shards: usize) -> Self {
        let mut shards = Vec::with_capacity(initial_shards);
        
        // 初期シャードの作成（軽量、標準、高負荷に分類）
        for i in 0..initial_shards {
            let shard_type = match i % 3 {
                0 => ShardType::Lightweight, // 低スペックノード向け
                1 => ShardType::Standard,    // 中スペックノード向け
                _ => ShardType::HighLoad,    // 高スペックノード向け
            };
            
            shards.push(Shard::new(i as ShardId, shard_type));
        }
        
        Self {
            shard_count: initial_shards,
            shards,
            node_specs: HashMap::new(),
        }
    }
    
    /// ノードスペックを登録
    pub fn assign_node_spec(&mut self, node_id: NodeId, cpu_cores: u32, memory_gb: u32) {
        let is_high_spec = cpu_cores >= 8 && memory_gb >= 16;
        
        let spec = NodeSpec {
            cpu_cores,
            memory_gb,
            is_high_spec,
        };
        
        self.node_specs.insert(node_id.clone(), spec);
        
        // ノードスペックに基づいてシャードを割り当て
        self.assign_shards_to_node(node_id);
    }
    
    /// ノードにシャードを割り当て
    pub fn assign_shards_to_node(&mut self, node_id: NodeId) {
        if let Some(spec) = self.node_specs.get(&node_id) {
            // ノードスペックに基づいてシャードタイプを決定
            let target_shard_type = if spec.cpu_cores >= 16 && spec.memory_gb >= 32 {
                ShardType::HighLoad
            } else if spec.cpu_cores >= 8 && spec.memory_gb >= 16 {
                ShardType::Standard
            } else {
                ShardType::Lightweight
            };
            
            // 適切なシャードを割り当て
            for shard in self.shards.iter_mut() {
                if shard.shard_type == target_shard_type {
                    shard.assign_node(node_id.clone());
                }
            }
        }
    }
    
    /// 負荷に応じてシャード数を調整
    pub fn adjust_shards(&mut self, load: u32) -> Result<(), Error> {
        // 負荷に応じてシャード数を動的に調整
        let target_shards = if load > 50000 {
            20 // 高負荷時は20シャード
        } else if load > 20000 {
            15 // 中負荷時は15シャード
        } else {
            10 // 低負荷時は10シャード
        };
        
        if target_shards > self.shard_count {
            // シャードを追加
            self.add_shards(target_shards - self.shard_count)?;
        } else if target_shards < self.shard_count {
            // シャードをマージ
            self.merge_shards(self.shard_count - target_shards)?;
        }
        
        Ok(())
    }
    
    /// シャードを追加
    fn add_shards(&mut self, count: usize) -> Result<(), Error> {
        for i in 0..count {
            let new_id = (self.shard_count + i) as ShardId;
            
            // シャードタイプをバランスよく割り当て
            let shard_type = match (self.shard_count + i) % 3 {
                0 => ShardType::Lightweight,
                1 => ShardType::Standard,
                _ => ShardType::HighLoad,
            };
            
            let new_shard = Shard::new(new_id, shard_type);
            self.shards.push(new_shard);
        }
        
        self.shard_count += count;
        
        // 新しいシャードにノードを割り当て
        self.rebalance_nodes()?;
        
        Ok(())
    }
    
    /// シャードをマージ
    fn merge_shards(&mut self, count: usize) -> Result<(), Error> {
        if count >= self.shard_count {
            return Err(Error::InvalidShardId(0));
        }
        
        // 負荷の低いシャードから順にマージ
        self.shards.sort_by(|a, b| a.load.partial_cmp(&b.load).unwrap());
        
        // マージするシャードを削除
        self.shards.drain(0..count);
        self.shard_count -= count;
        
        // シャードIDを振り直し
        for (i, shard) in self.shards.iter_mut().enumerate() {
            shard.id = i as ShardId;
        }
        
        // ノードを再割り当て
        self.rebalance_nodes()?;
        
        Ok(())
    }
    
    /// ノードを再割り当て
    fn rebalance_nodes(&mut self) -> Result<(), Error> {
        // 現在のノードスペック情報をコピー
        let node_specs = self.node_specs.clone();
        
        // 全シャードのノードをクリア
        for shard in &mut self.shards {
            shard.nodes.clear();
        }
        
        // ノードを再割り当て
        for (node_id, _) in node_specs {
            self.assign_shards_to_node(node_id);
        }
        
        Ok(())
    }
    
    /// シャード間の負荷バランスを最適化
    pub fn optimize_distribution(&mut self) -> Result<(), Error> {
        // 負荷の高いシャードから低いシャードへトランザクションを移動
        let mut shard_loads: Vec<(ShardId, f32)> = self.shards
            .iter()
            .map(|shard| (shard.id, shard.load))
            .collect();
        
        // 負荷でソート
        shard_loads.sort_by(|(_, load_a), (_, load_b)| load_b.partial_cmp(load_a).unwrap());
        
        // 負荷の高いシャードと低いシャードをペアにして再バランス
        let high_load_shards: Vec<ShardId> = shard_loads
            .iter()
            .filter(|(_, load)| *load > 0.8) // 80%以上の負荷
            .map(|(id, _)| *id)
            .collect();
        
        let low_load_shards: Vec<ShardId> = shard_loads
            .iter()
            .rev() // 逆順にして負荷の低いシャードから取得
            .filter(|(_, load)| *load < 0.3) // 30%以下の負荷
            .map(|(id, _)| *id)
            .collect();
        
        // 再バランス
        let pairs = std::cmp::min(high_load_shards.len(), low_load_shards.len());
        for i in 0..pairs {
            self.rebalance_shards(high_load_shards[i], low_load_shards[i])?;
        }
        
        Ok(())
    }
    
    /// 2つのシャード間でトランザクションを再バランス
    pub fn rebalance_shards(&mut self, from_shard: ShardId, to_shard: ShardId) -> Result<(), Error> {
        // シャードの存在確認
        if !self.shards.iter().any(|s| s.id == from_shard) {
            return Err(Error::InvalidShardId(from_shard));
        }
        
        if !self.shards.iter().any(|s| s.id == to_shard) {
            return Err(Error::InvalidShardId(to_shard));
        }
        
        // 実際の再バランスロジックを実装
        // ここではシミュレーションのみ
        
        // 負荷を更新
        if let Some(from) = self.shards.iter_mut().find(|s| s.id == from_shard) {
            from.load -= 0.2;
            from.load = from.load.max(0.0);
        }
        
        if let Some(to) = self.shards.iter_mut().find(|s| s.id == to_shard) {
            to.load += 0.2;
            to.load = to.load.min(1.0);
        }
        
        Ok(())
    }
    
    /// 軽量シャードのIDを取得
    pub fn get_lightweight_shard_id(&self) -> ShardId {
        // 負荷の低い軽量シャードを探す
        self.shards
            .iter()
            .filter(|s| s.shard_type == ShardType::Lightweight)
            .min_by(|a, b| a.load.partial_cmp(&b.load).unwrap())
            .map(|s| s.id)
            .unwrap_or(0)
    }
    
    /// 高負荷シャードのIDを取得
    pub fn get_high_load_shard_id(&self) -> ShardId {
        // 負荷の低い高負荷シャードを探す
        self.shards
            .iter()
            .filter(|s| s.shard_type == ShardType::HighLoad)
            .min_by(|a, b| a.load.partial_cmp(&b.load).unwrap())
            .map(|s| s.id)
            .unwrap_or(0)
    }
    
    /// 各シャードの負荷を取得
    pub fn get_shard_loads(&self) -> HashMap<ShardId, f32> {
        self.shards
            .iter()
            .map(|shard| (shard.id, shard.load))
            .collect()
    }
}