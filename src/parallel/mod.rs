// 超並列処理アーキテクチャ
//
// このモジュールは、ShardXの超並列処理アーキテクチャを提供します。
// 超並列処理は、大規模なデータと計算を複数のノードに分散し、
// 並行して処理することで、スケーラビリティと処理速度を向上させます。
//
// 主な機能:
// - 並列実行エンジン
// - データ分割と分散
// - 依存関係管理
// - 結果集約
// - 障害耐性

pub mod config;
pub mod engine;
pub mod workstealing;

pub use self::config::ParallelConfig;
pub use self::engine::ParallelEngine;
pub use workstealing::WorkStealingScheduler;
