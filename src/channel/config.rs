use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 状態チャネル設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelConfig {
    /// ネットワーク設定
    pub network_config: NetworkConfig,
    /// 紛争設定
    pub dispute_config: DisputeConfig,
    /// 更新設定
    pub update_config: UpdateConfig,
    /// ウォッチャー設定
    pub watcher_config: WatcherConfig,
    /// ルーティング設定
    pub routing_config: RoutingConfig,
    /// デフォルトタイムアウト（秒）
    pub default_timeout_seconds: u64,
    /// デフォルト有効期限（秒）
    pub default_expiry_seconds: u64,
    /// デフォルト最小支払い
    pub default_min_payment: u64,
    /// デフォルト最大支払い
    pub default_max_payment: u64,
    /// デフォルト手数料率
    pub default_fee_rate: f64,
    /// 更新間隔（ミリ秒）
    pub update_interval_ms: u64,
    /// ウォッチャー間隔（ミリ秒）
    pub watcher_interval_ms: u64,
    /// ルーティング間隔（ミリ秒）
    pub routing_interval_ms: u64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// ネットワーク設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// 最大ホップ数
    pub max_hops: u32,
    /// 最大ルート数
    pub max_routes: u32,
    /// 最大ルート検索時間（ミリ秒）
    pub max_route_search_time_ms: u64,
    /// 最大ルート検索試行回数
    pub max_route_search_attempts: u32,
    /// ルート検索タイムアウト（ミリ秒）
    pub route_search_timeout_ms: u64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// 紛争設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DisputeConfig {
    /// 紛争タイムアウト（秒）
    pub dispute_timeout_seconds: u64,
    /// 紛争解決タイムアウト（秒）
    pub dispute_resolution_timeout_seconds: u64,
    /// 紛争証拠提出期間（秒）
    pub dispute_evidence_period_seconds: u64,
    /// 紛争解決戦略
    pub dispute_resolution_strategy: DisputeResolutionStrategy,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// 更新設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// 更新タイムアウト（ミリ秒）
    pub update_timeout_ms: u64,
    /// 最大更新サイズ（バイト）
    pub max_update_size: u32,
    /// 最大更新バッチサイズ
    pub max_update_batch_size: u32,
    /// 更新再試行回数
    pub update_retry_count: u32,
    /// 更新再試行間隔（ミリ秒）
    pub update_retry_interval_ms: u64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// ウォッチャー設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WatcherConfig {
    /// ウォッチャータイムアウト（ミリ秒）
    pub watcher_timeout_ms: u64,
    /// チャネルチェック間隔（ミリ秒）
    pub channel_check_interval_ms: u64,
    /// 不正行為検出閾値
    pub fraud_detection_threshold: f64,
    /// 不正行為対応戦略
    pub fraud_response_strategy: FraudResponseStrategy,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// ルーティング設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// ルーティング戦略
    pub routing_strategy: RoutingStrategy,
    /// 最大ルート長
    pub max_route_length: u32,
    /// 最小チャネル容量
    pub min_channel_capacity: u64,
    /// 最大手数料率
    pub max_fee_rate: f64,
    /// ルートキャッシュ有効期限（秒）
    pub route_cache_expiry_seconds: u64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// チャネルパラメータ
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelParams {
    /// タイムアウト（秒）
    pub timeout_seconds: u64,
    /// 有効期限（秒）
    pub expiry_seconds: u64,
    /// 最小支払い
    pub min_payment: u64,
    /// 最大支払い
    pub max_payment: u64,
    /// 手数料率
    pub fee_rate: f64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// チャネルポリシー
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelPolicy {
    /// 最大チャネル数
    pub max_channels: u32,
    /// 最大チャネル容量
    pub max_channel_capacity: u64,
    /// 最小チャネル容量
    pub min_channel_capacity: u64,
    /// 最大支払いサイズ
    pub max_payment_size: u64,
    /// 最小支払いサイズ
    pub min_payment_size: u64,
    /// 最大手数料率
    pub max_fee_rate: f64,
    /// 最小手数料率
    pub min_fee_rate: f64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// 紛争解決戦略
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisputeResolutionStrategy {
    /// 最新状態
    LatestState,
    /// 署名済み状態
    SignedState,
    /// 仲裁
    Arbitration,
    /// カスタム
    Custom(String),
}

/// 不正行為対応戦略
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FraudResponseStrategy {
    /// 即時閉鎖
    ImmediateClose,
    /// 紛争開始
    StartDispute,
    /// 警告
    Warn,
    /// カスタム
    Custom(String),
}

/// ルーティング戦略
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingStrategy {
    /// 最短パス
    ShortestPath,
    /// 最低手数料
    LowestFee,
    /// 最高成功確率
    HighestSuccessProbability,
    /// バランス
    Balanced,
    /// カスタム
    Custom(String),
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            network_config: NetworkConfig::default(),
            dispute_config: DisputeConfig::default(),
            update_config: UpdateConfig::default(),
            watcher_config: WatcherConfig::default(),
            routing_config: RoutingConfig::default(),
            default_timeout_seconds: 600,    // 10分
            default_expiry_seconds: 2592000, // 30日
            default_min_payment: 1,
            default_max_payment: 1000000000, // 10億
            default_fee_rate: 0.001,         // 0.1%
            update_interval_ms: 1000,        // 1秒
            watcher_interval_ms: 60000,      // 1分
            routing_interval_ms: 300000,     // 5分
            metadata: HashMap::new(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_hops: 5,
            max_routes: 3,
            max_route_search_time_ms: 5000, // 5秒
            max_route_search_attempts: 3,
            route_search_timeout_ms: 10000, // 10秒
            metadata: HashMap::new(),
        }
    }
}

impl Default for DisputeConfig {
    fn default() -> Self {
        Self {
            dispute_timeout_seconds: 86400,             // 1日
            dispute_resolution_timeout_seconds: 172800, // 2日
            dispute_evidence_period_seconds: 43200,     // 12時間
            dispute_resolution_strategy: DisputeResolutionStrategy::SignedState,
            metadata: HashMap::new(),
        }
    }
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            update_timeout_ms: 5000, // 5秒
            max_update_size: 65536,  // 64KB
            max_update_batch_size: 100,
            update_retry_count: 3,
            update_retry_interval_ms: 1000, // 1秒
            metadata: HashMap::new(),
        }
    }
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            watcher_timeout_ms: 10000,        // 10秒
            channel_check_interval_ms: 60000, // 1分
            fraud_detection_threshold: 0.8,   // 80%
            fraud_response_strategy: FraudResponseStrategy::StartDispute,
            metadata: HashMap::new(),
        }
    }
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            routing_strategy: RoutingStrategy::Balanced,
            max_route_length: 5,
            min_channel_capacity: 1000,
            max_fee_rate: 0.01,              // 1%
            route_cache_expiry_seconds: 300, // 5分
            metadata: HashMap::new(),
        }
    }
}

impl Default for ChannelParams {
    fn default() -> Self {
        Self {
            timeout_seconds: 600,    // 10分
            expiry_seconds: 2592000, // 30日
            min_payment: 1,
            max_payment: 1000000000, // 10億
            fee_rate: 0.001,         // 0.1%
            metadata: HashMap::new(),
        }
    }
}

impl Default for ChannelPolicy {
    fn default() -> Self {
        Self {
            max_channels: 100,
            max_channel_capacity: 1000000000000, // 1兆
            min_channel_capacity: 1000,
            max_payment_size: 1000000000, // 10億
            min_payment_size: 1,
            max_fee_rate: 0.01,   // 1%
            min_fee_rate: 0.0001, // 0.01%
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_channel_config() {
        let config = ChannelConfig::default();

        // 基本設定
        assert_eq!(config.default_timeout_seconds, 600);
        assert_eq!(config.default_expiry_seconds, 2592000);
        assert_eq!(config.default_min_payment, 1);
        assert_eq!(config.default_max_payment, 1000000000);
        assert_eq!(config.default_fee_rate, 0.001);
        assert_eq!(config.update_interval_ms, 1000);
        assert_eq!(config.watcher_interval_ms, 60000);
        assert_eq!(config.routing_interval_ms, 300000);
    }

    #[test]
    fn test_network_config() {
        let config = NetworkConfig::default();

        assert_eq!(config.max_hops, 5);
        assert_eq!(config.max_routes, 3);
        assert_eq!(config.max_route_search_time_ms, 5000);
        assert_eq!(config.max_route_search_attempts, 3);
        assert_eq!(config.route_search_timeout_ms, 10000);
    }

    #[test]
    fn test_dispute_config() {
        let config = DisputeConfig::default();

        assert_eq!(config.dispute_timeout_seconds, 86400);
        assert_eq!(config.dispute_resolution_timeout_seconds, 172800);
        assert_eq!(config.dispute_evidence_period_seconds, 43200);
        assert_eq!(
            config.dispute_resolution_strategy,
            DisputeResolutionStrategy::SignedState
        );
    }

    #[test]
    fn test_update_config() {
        let config = UpdateConfig::default();

        assert_eq!(config.update_timeout_ms, 5000);
        assert_eq!(config.max_update_size, 65536);
        assert_eq!(config.max_update_batch_size, 100);
        assert_eq!(config.update_retry_count, 3);
        assert_eq!(config.update_retry_interval_ms, 1000);
    }

    #[test]
    fn test_watcher_config() {
        let config = WatcherConfig::default();

        assert_eq!(config.watcher_timeout_ms, 10000);
        assert_eq!(config.channel_check_interval_ms, 60000);
        assert_eq!(config.fraud_detection_threshold, 0.8);
        assert_eq!(
            config.fraud_response_strategy,
            FraudResponseStrategy::StartDispute
        );
    }

    #[test]
    fn test_routing_config() {
        let config = RoutingConfig::default();

        assert_eq!(config.routing_strategy, RoutingStrategy::Balanced);
        assert_eq!(config.max_route_length, 5);
        assert_eq!(config.min_channel_capacity, 1000);
        assert_eq!(config.max_fee_rate, 0.01);
        assert_eq!(config.route_cache_expiry_seconds, 300);
    }

    #[test]
    fn test_channel_params() {
        let params = ChannelParams::default();

        assert_eq!(params.timeout_seconds, 600);
        assert_eq!(params.expiry_seconds, 2592000);
        assert_eq!(params.min_payment, 1);
        assert_eq!(params.max_payment, 1000000000);
        assert_eq!(params.fee_rate, 0.001);
    }

    #[test]
    fn test_channel_policy() {
        let policy = ChannelPolicy::default();

        assert_eq!(policy.max_channels, 100);
        assert_eq!(policy.max_channel_capacity, 1000000000000);
        assert_eq!(policy.min_channel_capacity, 1000);
        assert_eq!(policy.max_payment_size, 1000000000);
        assert_eq!(policy.min_payment_size, 1);
        assert_eq!(policy.max_fee_rate, 0.01);
        assert_eq!(policy.min_fee_rate, 0.0001);
    }
}
