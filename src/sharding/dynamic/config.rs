use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 動的シャーディング設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynamicShardingConfig {
    /// シャード分割ポリシー
    pub shard_split_policy: ShardSplitPolicy,
    /// シャードマージポリシー
    pub shard_merge_policy: ShardMergePolicy,
    /// リバランスポリシー
    pub rebalance_policy: RebalancePolicy,
    /// ホットスポット検出設定
    pub hotspot_detection_config: HotspotDetectionConfig,
    /// シャード分割有効フラグ
    pub enable_shard_splitting: bool,
    /// シャードマージ有効フラグ
    pub enable_shard_merging: bool,
    /// シャード再配置有効フラグ
    pub enable_shard_relocation: bool,
    /// シャードリバランス有効フラグ
    pub enable_shard_rebalancing: bool,
    /// シャード最適化有効フラグ
    pub enable_shard_optimization: bool,
    /// ホットスポット検出有効フラグ
    pub enable_hotspot_detection: bool,
    /// メトリクス収集間隔（ミリ秒）
    pub metrics_collection_interval_ms: u64,
    /// シャード分割タイムアウト（秒）
    pub shard_split_timeout_seconds: u64,
    /// シャードマージタイムアウト（秒）
    pub shard_merge_timeout_seconds: u64,
    /// シャード再配置タイムアウト（秒）
    pub shard_relocation_timeout_seconds: u64,
    /// シャードリバランスタイムアウト（秒）
    pub shard_rebalance_timeout_seconds: u64,
    /// シャード最適化タイムアウト（秒）
    pub shard_optimization_timeout_seconds: u64,
    /// リバランス閾値
    pub rebalance_threshold: f64,
    /// 最適化閾値
    pub optimization_threshold: f64,
    /// 最小リバランス間隔（秒）
    pub min_rebalance_interval_seconds: u64,
    /// 最小最適化間隔（秒）
    pub min_optimization_interval_seconds: u64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// シャード分割ポリシー
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShardSplitPolicy {
    /// 負荷閾値
    pub load_threshold: f64,
    /// データサイズ閾値（バイト）
    pub data_size_threshold: u64,
    /// キー数閾値
    pub key_count_threshold: u64,
    /// 分割戦略
    pub split_strategy: SplitStrategyType,
    /// 最大分割数
    pub max_splits_per_interval: u32,
    /// 分割後の最小シャードサイズ（バイト）
    pub min_shard_size_after_split: u64,
    /// 分割後の最小キー数
    pub min_key_count_after_split: u64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// シャードマージポリシー
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShardMergePolicy {
    /// 負荷閾値
    pub load_threshold: f64,
    /// データサイズ閾値（バイト）
    pub data_size_threshold: u64,
    /// キー数閾値
    pub key_count_threshold: u64,
    /// マージ戦略
    pub merge_strategy: MergeStrategyType,
    /// 最大マージ数
    pub max_merges_per_interval: u32,
    /// マージ後の最大シャードサイズ（バイト）
    pub max_shard_size_after_merge: u64,
    /// マージ後の最大キー数
    pub max_key_count_after_merge: u64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// リバランスポリシー
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RebalancePolicy {
    /// 負荷不均衡閾値
    pub load_imbalance_threshold: f64,
    /// データサイズ不均衡閾値
    pub data_size_imbalance_threshold: f64,
    /// リバランス戦略
    pub rebalance_strategy: RebalanceStrategyType,
    /// 最大移動シャード数
    pub max_shard_moves_per_rebalance: u32,
    /// 最大移動データサイズ（バイト）
    pub max_data_move_size_per_rebalance: u64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// ホットスポット検出設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HotspotDetectionConfig {
    /// 負荷ホットスポット閾値
    pub load_hotspot_threshold: f64,
    /// データサイズホットスポット閾値
    pub data_size_hotspot_threshold: f64,
    /// キー分布ホットスポット閾値
    pub key_skew_hotspot_threshold: f64,
    /// アクセス分布ホットスポット閾値
    pub access_skew_hotspot_threshold: f64,
    /// 検出ウィンドウサイズ（秒）
    pub detection_window_seconds: u64,
    /// 検出感度
    pub detection_sensitivity: f64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// 分割戦略タイプ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitStrategyType {
    /// キー範囲による分割
    KeyRange,
    /// ハッシュによる分割
    Hash,
    /// アクセスパターンによる分割
    AccessPattern,
    /// データサイズによる分割
    DataSize,
    /// カスタム分割
    Custom,
}

/// マージ戦略タイプ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeStrategyType {
    /// 隣接シャードのマージ
    Adjacent,
    /// 低負荷シャードのマージ
    LowLoad,
    /// 類似アクセスパターンのマージ
    SimilarAccessPattern,
    /// カスタムマージ
    Custom,
}

/// リバランス戦略タイプ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RebalanceStrategyType {
    /// 負荷ベースのリバランス
    LoadBased,
    /// データサイズベースのリバランス
    DataSizeBased,
    /// ハイブリッドリバランス
    Hybrid,
    /// カスタムリバランス
    Custom,
}

impl Default for DynamicShardingConfig {
    fn default() -> Self {
        Self {
            shard_split_policy: ShardSplitPolicy::default(),
            shard_merge_policy: ShardMergePolicy::default(),
            rebalance_policy: RebalancePolicy::default(),
            hotspot_detection_config: HotspotDetectionConfig::default(),
            enable_shard_splitting: true,
            enable_shard_merging: true,
            enable_shard_relocation: true,
            enable_shard_rebalancing: true,
            enable_shard_optimization: true,
            enable_hotspot_detection: true,
            metrics_collection_interval_ms: 10000,    // 10秒
            shard_split_timeout_seconds: 300,         // 5分
            shard_merge_timeout_seconds: 600,         // 10分
            shard_relocation_timeout_seconds: 1800,   // 30分
            shard_rebalance_timeout_seconds: 3600,    // 1時間
            shard_optimization_timeout_seconds: 1800, // 30分
            rebalance_threshold: 0.2,                 // 20%の不均衡
            optimization_threshold: 0.3,              // 30%の最適化スコア
            min_rebalance_interval_seconds: 3600,     // 1時間
            min_optimization_interval_seconds: 86400, // 1日
            metadata: HashMap::new(),
        }
    }
}

impl Default for ShardSplitPolicy {
    fn default() -> Self {
        Self {
            load_threshold: 0.8,                // 80%の負荷
            data_size_threshold: 1_073_741_824, // 1GB
            key_count_threshold: 1_000_000,     // 100万キー
            split_strategy: SplitStrategyType::KeyRange,
            max_splits_per_interval: 5,
            min_shard_size_after_split: 104_857_600, // 100MB
            min_key_count_after_split: 100_000,      // 10万キー
            metadata: HashMap::new(),
        }
    }
}

impl Default for ShardMergePolicy {
    fn default() -> Self {
        Self {
            load_threshold: 0.2,              // 20%の負荷
            data_size_threshold: 104_857_600, // 100MB
            key_count_threshold: 100_000,     // 10万キー
            merge_strategy: MergeStrategyType::Adjacent,
            max_merges_per_interval: 3,
            max_shard_size_after_merge: 1_073_741_824, // 1GB
            max_key_count_after_merge: 1_000_000,      // 100万キー
            metadata: HashMap::new(),
        }
    }
}

impl Default for RebalancePolicy {
    fn default() -> Self {
        Self {
            load_imbalance_threshold: 0.2,      // 20%の不均衡
            data_size_imbalance_threshold: 0.3, // 30%の不均衡
            rebalance_strategy: RebalanceStrategyType::Hybrid,
            max_shard_moves_per_rebalance: 10,
            max_data_move_size_per_rebalance: 10_737_418_240, // 10GB
            metadata: HashMap::new(),
        }
    }
}

impl Default for HotspotDetectionConfig {
    fn default() -> Self {
        Self {
            load_hotspot_threshold: 0.9,        // 90%の負荷
            data_size_hotspot_threshold: 0.8,   // 80%のデータサイズ
            key_skew_hotspot_threshold: 0.7,    // 70%のキー分布の偏り
            access_skew_hotspot_threshold: 0.8, // 80%のアクセス分布の偏り
            detection_window_seconds: 300,      // 5分
            detection_sensitivity: 0.8,         // 80%の感度
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DynamicShardingConfig::default();

        // 基本設定
        assert!(config.enable_shard_splitting);
        assert!(config.enable_shard_merging);
        assert!(config.enable_shard_relocation);
        assert!(config.enable_shard_rebalancing);
        assert!(config.enable_shard_optimization);
        assert!(config.enable_hotspot_detection);

        // 間隔設定
        assert_eq!(config.metrics_collection_interval_ms, 10000);
        assert_eq!(config.shard_split_timeout_seconds, 300);
        assert_eq!(config.shard_merge_timeout_seconds, 600);
        assert_eq!(config.shard_relocation_timeout_seconds, 1800);
        assert_eq!(config.shard_rebalance_timeout_seconds, 3600);
        assert_eq!(config.shard_optimization_timeout_seconds, 1800);

        // 閾値設定
        assert_eq!(config.rebalance_threshold, 0.2);
        assert_eq!(config.optimization_threshold, 0.3);
        assert_eq!(config.min_rebalance_interval_seconds, 3600);
        assert_eq!(config.min_optimization_interval_seconds, 86400);
    }

    #[test]
    fn test_shard_split_policy() {
        let policy = ShardSplitPolicy::default();

        assert_eq!(policy.load_threshold, 0.8);
        assert_eq!(policy.data_size_threshold, 1_073_741_824);
        assert_eq!(policy.key_count_threshold, 1_000_000);
        assert_eq!(policy.split_strategy, SplitStrategyType::KeyRange);
        assert_eq!(policy.max_splits_per_interval, 5);
        assert_eq!(policy.min_shard_size_after_split, 104_857_600);
        assert_eq!(policy.min_key_count_after_split, 100_000);
    }

    #[test]
    fn test_shard_merge_policy() {
        let policy = ShardMergePolicy::default();

        assert_eq!(policy.load_threshold, 0.2);
        assert_eq!(policy.data_size_threshold, 104_857_600);
        assert_eq!(policy.key_count_threshold, 100_000);
        assert_eq!(policy.merge_strategy, MergeStrategyType::Adjacent);
        assert_eq!(policy.max_merges_per_interval, 3);
        assert_eq!(policy.max_shard_size_after_merge, 1_073_741_824);
        assert_eq!(policy.max_key_count_after_merge, 1_000_000);
    }

    #[test]
    fn test_rebalance_policy() {
        let policy = RebalancePolicy::default();

        assert_eq!(policy.load_imbalance_threshold, 0.2);
        assert_eq!(policy.data_size_imbalance_threshold, 0.3);
        assert_eq!(policy.rebalance_strategy, RebalanceStrategyType::Hybrid);
        assert_eq!(policy.max_shard_moves_per_rebalance, 10);
        assert_eq!(policy.max_data_move_size_per_rebalance, 10_737_418_240);
    }

    #[test]
    fn test_hotspot_detection_config() {
        let config = HotspotDetectionConfig::default();

        assert_eq!(config.load_hotspot_threshold, 0.9);
        assert_eq!(config.data_size_hotspot_threshold, 0.8);
        assert_eq!(config.key_skew_hotspot_threshold, 0.7);
        assert_eq!(config.access_skew_hotspot_threshold, 0.8);
        assert_eq!(config.detection_window_seconds, 300);
        assert_eq!(config.detection_sensitivity, 0.8);
    }
}
