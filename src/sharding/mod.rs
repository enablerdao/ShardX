//! # シャーディングモジュール
//!
//! このモジュールは、ShardXのシャーディング機能を提供します。
//! シャーディングは、データベースを複数の小さな部分（シャード）に分割し、
//! スケーラビリティを向上させる技術です。
//!
//! ## 主な機能
//!
//! - シャード作成と管理
//! - シャード間のデータ同期
//! - シャードの動的割り当てと再バランシング
//! - シャードメトリクスの収集と分析
//!
//! ## 使用例
//!
//! ```rust
//! use shardx::sharding::{ShardManager, ShardType, NodeSpec};
//!
//! // シャードマネージャーを初期化
//! let manager = ShardManager::new();
//!
//! // シャードを作成
//! let shard = manager.create_shard(ShardType::Data);
//!
//! // ノードを追加
//! let node_spec = NodeSpec::new("node1", "127.0.0.1:8000");
//! manager.add_node(node_spec);
//! ```

pub mod assignment;
pub mod manager;

pub use manager::{NodeId, NodeSpec, Shard, ShardId, ShardManager, ShardType};
