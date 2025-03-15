//! シャーディングモジュール
//! 
//! このモジュールはShardXのシャーディング機能を実装します。
//! シャードの作成、管理、およびシャード間の通信を処理します。

pub mod manager;
pub mod assignment;

pub use manager::{ShardManager, Shard, ShardType, NodeSpec, ShardId, NodeId};

use crate::error::Error;
use std::collections::HashMap;

/// シャードマネージャー
pub struct ShardingManager {
    /// シャードマップ
    shards: HashMap<String, ShardInfo>,
    /// シャード設定
    config: ShardingConfig,
}

/// シャード情報
#[derive(Debug, Clone)]
pub struct ShardInfo {
    /// シャードID
    pub id: String,
    /// シャード名
    pub name: String,
    /// シャードのノード数
    pub node_count: usize,
    /// シャードの状態
    pub status: ShardStatus,
}

/// シャードの状態
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShardStatus {
    /// アクティブ
    Active,
    /// 非アクティブ
    Inactive,
    /// 初期化中
    Initializing,
    /// 終了中
    Terminating,
}

/// シャーディング設定
#[derive(Debug, Clone)]
pub struct ShardingConfig {
    /// シャード数
    pub shard_count: usize,
    /// 最小ノード数/シャード
    pub min_nodes_per_shard: usize,
    /// 最大ノード数/シャード
    pub max_nodes_per_shard: usize,
    /// 自動シャーディングを有効にする
    pub auto_sharding: bool,
    /// シャードの再バランス間隔（ブロック数）
    pub rebalance_interval: u64,
}

impl Default for ShardingConfig {
    fn default() -> Self {
        Self {
            shard_count: 16,
            min_nodes_per_shard: 3,
            max_nodes_per_shard: 10,
            auto_sharding: true,
            rebalance_interval: 1000,
        }
    }
}
