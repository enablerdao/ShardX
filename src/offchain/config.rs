use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// オフチェーン計算設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OffchainConfig {
    /// 計算ノード設定
    pub compute_node_config: ComputeNodeConfig,
    /// タスク設定
    pub task_config: TaskConfig,
    /// 検証器設定
    pub verifier_config: VerifierConfig,
    /// 証明生成器設定
    pub prover_config: ProverConfig,
    /// 実行器設定
    pub executor_config: ExecutorConfig,
    /// スケジューラー設定
    pub scheduler_config: SchedulerConfig,
    /// インセンティブ設定
    pub incentive_config: IncentiveConfig,
    /// レジストリ設定
    pub registry_config: RegistryConfig,
    /// プロトコル設定
    pub protocol_config: ProtocolConfig,
    /// スケジューラー間隔（ミリ秒）
    pub scheduler_interval_ms: u64,
    /// 実行器間隔（ミリ秒）
    pub executor_interval_ms: u64,
    /// 検証器間隔（ミリ秒）
    pub verifier_interval_ms: u64,
    /// インセンティブ間隔（ミリ秒）
    pub incentive_interval_ms: u64,
    /// ノード監視間隔（ミリ秒）
    pub node_monitor_interval_ms: u64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// 計算ノード設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComputeNodeConfig {
    /// 最大ノード数
    pub max_nodes: u32,
    /// 最小ノード数
    pub min_nodes: u32,
    /// ハートビートタイムアウト（ミリ秒）
    pub heartbeat_timeout_ms: u64,
    /// ノード登録タイムアウト（ミリ秒）
    pub registration_timeout_ms: u64,
    /// 最大タスク割り当て数
    pub max_task_assignments: u32,
    /// 最小評価スコア
    pub min_reputation_score: f64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// タスク設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskConfig {
    /// 最大タスク数
    pub max_tasks: u32,
    /// 最大タスクサイズ（バイト）
    pub max_task_size: u64,
    /// 最大入力データサイズ（バイト）
    pub max_input_data_size: u64,
    /// 最大出力データサイズ（バイト）
    pub max_output_data_size: u64,
    /// デフォルトタスクタイムアウト（ミリ秒）
    pub default_task_timeout_ms: u64,
    /// 最大タスクタイムアウト（ミリ秒）
    pub max_task_timeout_ms: u64,
    /// 最大再試行回数
    pub max_retry_count: u32,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// 検証器設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifierConfig {
    /// 検証方法
    pub verification_methods: Vec<VerificationMethod>,
    /// 検証タイムアウト（ミリ秒）
    pub verification_timeout_ms: u64,
    /// 検証スレッド数
    pub verification_threads: u32,
    /// 検証キャッシュサイズ
    pub verification_cache_size: u32,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// 証明生成器設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProverConfig {
    /// 証明タイプ
    pub proof_types: Vec<ProofType>,
    /// 証明生成タイムアウト（ミリ秒）
    pub proof_generation_timeout_ms: u64,
    /// 証明生成スレッド数
    pub proof_generation_threads: u32,
    /// 証明キャッシュサイズ
    pub proof_cache_size: u32,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// 実行器設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutorConfig {
    /// 実行環境
    pub execution_environments: Vec<ExecutionEnvironment>,
    /// 実行タイムアウト（ミリ秒）
    pub execution_timeout_ms: u64,
    /// 実行スレッド数
    pub execution_threads: u32,
    /// 最大メモリ使用量（MB）
    pub max_memory_usage_mb: u32,
    /// 最大CPU使用率
    pub max_cpu_usage: f64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// スケジューラー設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// スケジューリング戦略
    pub scheduling_strategy: SchedulingStrategy,
    /// 最大バッチサイズ
    pub max_batch_size: u32,
    /// 最大キュー長
    pub max_queue_length: u32,
    /// 優先度重み
    pub priority_weights: HashMap<String, f64>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// インセンティブ設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IncentiveConfig {
    /// 報酬モデル
    pub reward_model: RewardModel,
    /// 基本報酬率
    pub base_reward_rate: f64,
    /// ボーナス報酬率
    pub bonus_reward_rate: f64,
    /// ペナルティ率
    pub penalty_rate: f64,
    /// 最小報酬
    pub min_reward: u64,
    /// 最大報酬
    pub max_reward: u64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// レジストリ設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// 最大レジストリサイズ
    pub max_registry_size: u32,
    /// レジストリ更新間隔（ミリ秒）
    pub registry_update_interval_ms: u64,
    /// レジストリキャッシュ有効期限（ミリ秒）
    pub registry_cache_expiry_ms: u64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// プロトコル設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProtocolConfig {
    /// プロトコルバージョン
    pub protocol_version: String,
    /// 最大メッセージサイズ（バイト）
    pub max_message_size: u64,
    /// メッセージタイムアウト（ミリ秒）
    pub message_timeout_ms: u64,
    /// 再試行間隔（ミリ秒）
    pub retry_interval_ms: u64,
    /// 最大再試行回数
    pub max_retry_count: u32,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// 検証方法
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationMethod {
    /// 再実行
    Reexecution,
    /// ゼロ知識証明
    ZeroKnowledgeProof,
    /// マルチパーティ検証
    MultiPartyVerification,
    /// 信頼できる実行環境
    TrustedExecutionEnvironment,
    /// カスタム
    Custom(String),
}

/// 証明タイプ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofType {
    /// ゼロ知識証明
    ZeroKnowledge,
    /// SNARKs
    SNARKs,
    /// STARKs
    STARKs,
    /// Bulletproofs
    Bulletproofs,
    /// カスタム
    Custom(String),
}

/// 実行環境
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionEnvironment {
    /// Docker
    Docker,
    /// WebAssembly
    WebAssembly,
    /// 仮想マシン
    VirtualMachine,
    /// ネイティブ
    Native,
    /// 信頼できる実行環境
    TrustedExecutionEnvironment,
    /// カスタム
    Custom(String),
}

/// スケジューリング戦略
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulingStrategy {
    /// 先入れ先出し
    FIFO,
    /// 優先度ベース
    PriorityBased,
    /// ラウンドロビン
    RoundRobin,
    /// 最短ジョブ優先
    ShortestJobFirst,
    /// リソースベース
    ResourceBased,
    /// カスタム
    Custom(String),
}

/// 報酬モデル
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RewardModel {
    /// 固定報酬
    Fixed,
    /// 計算時間ベース
    ComputationTimeBased,
    /// リソース使用量ベース
    ResourceUsageBased,
    /// 市場ベース
    MarketBased,
    /// 評価ベース
    ReputationBased,
    /// カスタム
    Custom(String),
}

impl Default for OffchainConfig {
    fn default() -> Self {
        Self {
            compute_node_config: ComputeNodeConfig::default(),
            task_config: TaskConfig::default(),
            verifier_config: VerifierConfig::default(),
            prover_config: ProverConfig::default(),
            executor_config: ExecutorConfig::default(),
            scheduler_config: SchedulerConfig::default(),
            incentive_config: IncentiveConfig::default(),
            registry_config: RegistryConfig::default(),
            protocol_config: ProtocolConfig::default(),
            scheduler_interval_ms: 1000, // 1秒
            executor_interval_ms: 1000, // 1秒
            verifier_interval_ms: 5000, // 5秒
            incentive_interval_ms: 60000, // 1分
            node_monitor_interval_ms: 30000, // 30秒
            metadata: HashMap::new(),
        }
    }
}

impl Default for ComputeNodeConfig {
    fn default() -> Self {
        Self {
            max_nodes: 1000,
            min_nodes: 3,
            heartbeat_timeout_ms: 30000, // 30秒
            registration_timeout_ms: 60000, // 1分
            max_task_assignments: 10,
            min_reputation_score: 0.7, // 70%
            metadata: HashMap::new(),
        }
    }
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self {
            max_tasks: 10000,
            max_task_size: 10485760, // 10MB
            max_input_data_size: 104857600, // 100MB
            max_output_data_size: 104857600, // 100MB
            default_task_timeout_ms: 300000, // 5分
            max_task_timeout_ms: 3600000, // 1時間
            max_retry_count: 3,
            metadata: HashMap::new(),
        }
    }
}

impl Default for VerifierConfig {
    fn default() -> Self {
        Self {
            verification_methods: vec![
                VerificationMethod::Reexecution,
                VerificationMethod::MultiPartyVerification,
            ],
            verification_timeout_ms: 60000, // 1分
            verification_threads: 4,
            verification_cache_size: 1000,
            metadata: HashMap::new(),
        }
    }
}

impl Default for ProverConfig {
    fn default() -> Self {
        Self {
            proof_types: vec![
                ProofType::ZeroKnowledge,
                ProofType::SNARKs,
            ],
            proof_generation_timeout_ms: 300000, // 5分
            proof_generation_threads: 2,
            proof_cache_size: 100,
            metadata: HashMap::new(),
        }
    }
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            execution_environments: vec![
                ExecutionEnvironment::Docker,
                ExecutionEnvironment::WebAssembly,
            ],
            execution_timeout_ms: 300000, // 5分
            execution_threads: 8,
            max_memory_usage_mb: 4096, // 4GB
            max_cpu_usage: 0.8, // 80%
            metadata: HashMap::new(),
        }
    }
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        let mut priority_weights = HashMap::new();
        priority_weights.insert("high".to_string(), 3.0);
        priority_weights.insert("medium".to_string(), 2.0);
        priority_weights.insert("low".to_string(), 1.0);
        
        Self {
            scheduling_strategy: SchedulingStrategy::PriorityBased,
            max_batch_size: 100,
            max_queue_length: 1000,
            priority_weights,
            metadata: HashMap::new(),
        }
    }
}

impl Default for IncentiveConfig {
    fn default() -> Self {
        Self {
            reward_model: RewardModel::ResourceUsageBased,
            base_reward_rate: 1.0,
            bonus_reward_rate: 0.2, // 20%
            penalty_rate: 0.5, // 50%
            min_reward: 1,
            max_reward: 1000000,
            metadata: HashMap::new(),
        }
    }
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            max_registry_size: 10000,
            registry_update_interval_ms: 60000, // 1分
            registry_cache_expiry_ms: 300000, // 5分
            metadata: HashMap::new(),
        }
    }
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            protocol_version: "1.0.0".to_string(),
            max_message_size: 10485760, // 10MB
            message_timeout_ms: 30000, // 30秒
            retry_interval_ms: 5000, // 5秒
            max_retry_count: 3,
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_offchain_config() {
        let config = OffchainConfig::default();
        
        // 基本設定
        assert_eq!(config.scheduler_interval_ms, 1000);
        assert_eq!(config.executor_interval_ms, 1000);
        assert_eq!(config.verifier_interval_ms, 5000);
        assert_eq!(config.incentive_interval_ms, 60000);
        assert_eq!(config.node_monitor_interval_ms, 30000);
    }
    
    #[test]
    fn test_compute_node_config() {
        let config = ComputeNodeConfig::default();
        
        assert_eq!(config.max_nodes, 1000);
        assert_eq!(config.min_nodes, 3);
        assert_eq!(config.heartbeat_timeout_ms, 30000);
        assert_eq!(config.registration_timeout_ms, 60000);
        assert_eq!(config.max_task_assignments, 10);
        assert_eq!(config.min_reputation_score, 0.7);
    }
    
    #[test]
    fn test_task_config() {
        let config = TaskConfig::default();
        
        assert_eq!(config.max_tasks, 10000);
        assert_eq!(config.max_task_size, 10485760);
        assert_eq!(config.max_input_data_size, 104857600);
        assert_eq!(config.max_output_data_size, 104857600);
        assert_eq!(config.default_task_timeout_ms, 300000);
        assert_eq!(config.max_task_timeout_ms, 3600000);
        assert_eq!(config.max_retry_count, 3);
    }
    
    #[test]
    fn test_verifier_config() {
        let config = VerifierConfig::default();
        
        assert_eq!(config.verification_methods.len(), 2);
        assert!(config.verification_methods.contains(&VerificationMethod::Reexecution));
        assert!(config.verification_methods.contains(&VerificationMethod::MultiPartyVerification));
        assert_eq!(config.verification_timeout_ms, 60000);
        assert_eq!(config.verification_threads, 4);
        assert_eq!(config.verification_cache_size, 1000);
    }
    
    #[test]
    fn test_prover_config() {
        let config = ProverConfig::default();
        
        assert_eq!(config.proof_types.len(), 2);
        assert!(config.proof_types.contains(&ProofType::ZeroKnowledge));
        assert!(config.proof_types.contains(&ProofType::SNARKs));
        assert_eq!(config.proof_generation_timeout_ms, 300000);
        assert_eq!(config.proof_generation_threads, 2);
        assert_eq!(config.proof_cache_size, 100);
    }
    
    #[test]
    fn test_executor_config() {
        let config = ExecutorConfig::default();
        
        assert_eq!(config.execution_environments.len(), 2);
        assert!(config.execution_environments.contains(&ExecutionEnvironment::Docker));
        assert!(config.execution_environments.contains(&ExecutionEnvironment::WebAssembly));
        assert_eq!(config.execution_timeout_ms, 300000);
        assert_eq!(config.execution_threads, 8);
        assert_eq!(config.max_memory_usage_mb, 4096);
        assert_eq!(config.max_cpu_usage, 0.8);
    }
    
    #[test]
    fn test_scheduler_config() {
        let config = SchedulerConfig::default();
        
        assert_eq!(config.scheduling_strategy, SchedulingStrategy::PriorityBased);
        assert_eq!(config.max_batch_size, 100);
        assert_eq!(config.max_queue_length, 1000);
        assert_eq!(config.priority_weights.len(), 3);
        assert_eq!(config.priority_weights.get("high"), Some(&3.0));
        assert_eq!(config.priority_weights.get("medium"), Some(&2.0));
        assert_eq!(config.priority_weights.get("low"), Some(&1.0));
    }
    
    #[test]
    fn test_incentive_config() {
        let config = IncentiveConfig::default();
        
        assert_eq!(config.reward_model, RewardModel::ResourceUsageBased);
        assert_eq!(config.base_reward_rate, 1.0);
        assert_eq!(config.bonus_reward_rate, 0.2);
        assert_eq!(config.penalty_rate, 0.5);
        assert_eq!(config.min_reward, 1);
        assert_eq!(config.max_reward, 1000000);
    }
    
    #[test]
    fn test_registry_config() {
        let config = RegistryConfig::default();
        
        assert_eq!(config.max_registry_size, 10000);
        assert_eq!(config.registry_update_interval_ms, 60000);
        assert_eq!(config.registry_cache_expiry_ms, 300000);
    }
    
    #[test]
    fn test_protocol_config() {
        let config = ProtocolConfig::default();
        
        assert_eq!(config.protocol_version, "1.0.0");
        assert_eq!(config.max_message_size, 10485760);
        assert_eq!(config.message_timeout_ms, 30000);
        assert_eq!(config.retry_interval_ms, 5000);
        assert_eq!(config.max_retry_count, 3);
    }
}