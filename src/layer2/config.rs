use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// レイヤー2設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layer2Config {
    /// ロールアップ設定
    pub rollup_config: RollupConfig,
    /// サイドチェーン設定
    pub sidechain_config: SidechainConfig,
    /// プラズマチェーン設定
    pub plasma_config: PlasmaConfig,
    /// バリデータ設定
    pub validator_config: ValidatorConfig,
    /// 同期設定
    pub sync_config: SyncConfig,
    /// チャレンジ設定
    pub challenge_config: ChallengeConfig,
    /// バッチ設定
    pub batch_config: BatchConfig,
    /// 同期間隔（ミリ秒）
    pub sync_interval_ms: u64,
    /// バッチ間隔（ミリ秒）
    pub batch_interval_ms: u64,
    /// チャレンジ間隔（ミリ秒）
    pub challenge_interval_ms: u64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// ロールアップ設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RollupConfig {
    /// ID
    pub id: String,
    /// 名前
    pub name: String,
    /// 説明
    pub description: String,
    /// ロールアップタイプ
    pub rollup_type: RollupType,
    /// データ可用性モード
    pub data_availability_mode: DataAvailabilityMode,
    /// 検証モード
    pub verification_mode: VerificationMode,
    /// バッチサイズ
    pub batch_size: u32,
    /// チャレンジ期間（秒）
    pub challenge_period_seconds: u64,
    /// 最大トランザクションサイズ（バイト）
    pub max_transaction_size: u32,
    /// 最大バッチサイズ（バイト）
    pub max_batch_size: u32,
    /// ガス制限
    pub gas_limit: u64,
    /// ガス価格
    pub gas_price: u64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// サイドチェーン設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SidechainConfig {
    /// ID
    pub id: String,
    /// 名前
    pub name: String,
    /// 説明
    pub description: String,
    /// コンセンサスアルゴリズム
    pub consensus_algorithm: ConsensusAlgorithm,
    /// ブロック時間（秒）
    pub block_time_seconds: u64,
    /// 最大ブロックサイズ（バイト）
    pub max_block_size: u32,
    /// 最大トランザクションサイズ（バイト）
    pub max_transaction_size: u32,
    /// ガス制限
    pub gas_limit: u64,
    /// ガス価格
    pub gas_price: u64,
    /// バリデータ数
    pub validator_count: u32,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// プラズマチェーン設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlasmaConfig {
    /// ID
    pub id: String,
    /// 名前
    pub name: String,
    /// 説明
    pub description: String,
    /// プラズマタイプ
    pub plasma_type: PlasmaType,
    /// ブロック時間（秒）
    pub block_time_seconds: u64,
    /// 最大ブロックサイズ（バイト）
    pub max_block_size: u32,
    /// 最大トランザクションサイズ（バイト）
    pub max_transaction_size: u32,
    /// 出金遅延（秒）
    pub withdrawal_delay_seconds: u64,
    /// チャレンジ期間（秒）
    pub challenge_period_seconds: u64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// バリデータ設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// ID
    pub id: String,
    /// 名前
    pub name: String,
    /// 説明
    pub description: String,
    /// コンセンサスアルゴリズム
    pub consensus_algorithm: ConsensusAlgorithm,
    /// バリデータ数
    pub validator_count: u32,
    /// 最小ステーク量
    pub min_stake_amount: u64,
    /// 報酬率
    pub reward_rate: f64,
    /// スラッシング条件
    pub slashing_conditions: SlashingConditions,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// 同期設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncConfig {
    /// 同期モード
    pub sync_mode: SyncMode,
    /// 最大同期ブロック数
    pub max_sync_blocks: u32,
    /// 同期タイムアウト（秒）
    pub sync_timeout_seconds: u64,
    /// 同期間隔（秒）
    pub sync_interval_seconds: u64,
    /// 同期ピア数
    pub sync_peer_count: u32,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// チャレンジ設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChallengeConfig {
    /// チャレンジタイプ
    pub challenge_types: Vec<ChallengeType>,
    /// チャレンジ期間（秒）
    pub challenge_period_seconds: u64,
    /// チャレンジ保証金
    pub challenge_bond_amount: u64,
    /// チャレンジ報酬率
    pub challenge_reward_rate: f64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// バッチ設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchConfig {
    /// バッチサイズ
    pub batch_size: u32,
    /// 最大バッチサイズ（バイト）
    pub max_batch_size: u32,
    /// バッチ間隔（秒）
    pub batch_interval_seconds: u64,
    /// バッチ提出ガス制限
    pub batch_submission_gas_limit: u64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// ロールアップタイプ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RollupType {
    /// Optimisticロールアップ
    Optimistic,
    /// ZKロールアップ
    ZK,
}

/// データ可用性モード
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataAvailabilityMode {
    /// オンチェーン
    OnChain,
    /// オフチェーン
    OffChain,
    /// ハイブリッド
    Hybrid,
}

/// 検証モード
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationMode {
    /// 楽観的
    Optimistic,
    /// ゼロ知識証明
    ZeroKnowledge,
    /// 直接検証
    DirectVerification,
}

/// コンセンサスアルゴリズム
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusAlgorithm {
    /// プルーフオブステーク
    ProofOfStake,
    /// プルーフオブオーソリティ
    ProofOfAuthority,
    /// デリゲイテッドプルーフオブステーク
    DelegatedProofOfStake,
    /// プラクティカルビザンチン障害耐性
    PBFT,
    /// テンダーミント
    Tendermint,
    /// カスタム
    Custom(String),
}

/// プラズマタイプ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlasmaType {
    /// MVP
    MVP,
    /// キャッシュ
    Cash,
    /// MoreVP
    MoreVP,
    /// カスタム
    Custom(String),
}

/// 同期モード
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncMode {
    /// フル
    Full,
    /// 高速
    Fast,
    /// ライト
    Light,
    /// スナップショット
    Snapshot,
}

/// チャレンジタイプ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChallengeType {
    /// 不正トランザクション
    InvalidTransaction,
    /// 不正状態遷移
    InvalidStateTransition,
    /// 不正ブロック
    InvalidBlock,
    /// 不正証明
    InvalidProof,
    /// データ可用性
    DataAvailability,
}

/// スラッシング条件
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SlashingConditions {
    /// ダブルサイン
    pub double_signing_penalty_percentage: u32,
    /// オフライン
    pub offline_penalty_percentage: u32,
    /// 不正行為
    pub malicious_behavior_penalty_percentage: u32,
}

impl Default for Layer2Config {
    fn default() -> Self {
        Self {
            rollup_config: RollupConfig::default(),
            sidechain_config: SidechainConfig::default(),
            plasma_config: PlasmaConfig::default(),
            validator_config: ValidatorConfig::default(),
            sync_config: SyncConfig::default(),
            challenge_config: ChallengeConfig::default(),
            batch_config: BatchConfig::default(),
            sync_interval_ms: 10000, // 10秒
            batch_interval_ms: 60000, // 1分
            challenge_interval_ms: 300000, // 5分
            metadata: HashMap::new(),
        }
    }
}

impl Default for RollupConfig {
    fn default() -> Self {
        Self {
            id: "default_rollup".to_string(),
            name: "Default Rollup".to_string(),
            description: "Default rollup configuration".to_string(),
            rollup_type: RollupType::Optimistic,
            data_availability_mode: DataAvailabilityMode::OnChain,
            verification_mode: VerificationMode::Optimistic,
            batch_size: 100,
            challenge_period_seconds: 604800, // 1週間
            max_transaction_size: 65536, // 64KB
            max_batch_size: 1048576, // 1MB
            gas_limit: 10000000,
            gas_price: 1000000000, // 1 Gwei
            metadata: HashMap::new(),
        }
    }
}

impl Default for SidechainConfig {
    fn default() -> Self {
        Self {
            id: "default_sidechain".to_string(),
            name: "Default Sidechain".to_string(),
            description: "Default sidechain configuration".to_string(),
            consensus_algorithm: ConsensusAlgorithm::ProofOfAuthority,
            block_time_seconds: 5,
            max_block_size: 1048576, // 1MB
            max_transaction_size: 65536, // 64KB
            gas_limit: 10000000,
            gas_price: 1000000000, // 1 Gwei
            validator_count: 5,
            metadata: HashMap::new(),
        }
    }
}

impl Default for PlasmaConfig {
    fn default() -> Self {
        Self {
            id: "default_plasma".to_string(),
            name: "Default Plasma".to_string(),
            description: "Default plasma configuration".to_string(),
            plasma_type: PlasmaType::MVP,
            block_time_seconds: 15,
            max_block_size: 1048576, // 1MB
            max_transaction_size: 65536, // 64KB
            withdrawal_delay_seconds: 604800, // 1週間
            challenge_period_seconds: 604800, // 1週間
            metadata: HashMap::new(),
        }
    }
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            id: "default_validator".to_string(),
            name: "Default Validator".to_string(),
            description: "Default validator configuration".to_string(),
            consensus_algorithm: ConsensusAlgorithm::ProofOfStake,
            validator_count: 5,
            min_stake_amount: 1000000000000000000, // 1 ETH
            reward_rate: 0.05, // 5%
            slashing_conditions: SlashingConditions {
                double_signing_penalty_percentage: 100, // 100%
                offline_penalty_percentage: 10, // 10%
                malicious_behavior_penalty_percentage: 100, // 100%
            },
            metadata: HashMap::new(),
        }
    }
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            sync_mode: SyncMode::Fast,
            max_sync_blocks: 1000,
            sync_timeout_seconds: 300, // 5分
            sync_interval_seconds: 60, // 1分
            sync_peer_count: 3,
            metadata: HashMap::new(),
        }
    }
}

impl Default for ChallengeConfig {
    fn default() -> Self {
        Self {
            challenge_types: vec![
                ChallengeType::InvalidTransaction,
                ChallengeType::InvalidStateTransition,
                ChallengeType::InvalidBlock,
                ChallengeType::InvalidProof,
                ChallengeType::DataAvailability,
            ],
            challenge_period_seconds: 604800, // 1週間
            challenge_bond_amount: 1000000000000000000, // 1 ETH
            challenge_reward_rate: 0.1, // 10%
            metadata: HashMap::new(),
        }
    }
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            max_batch_size: 1048576, // 1MB
            batch_interval_seconds: 600, // 10分
            batch_submission_gas_limit: 1000000,
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_layer2_config() {
        let config = Layer2Config::default();
        
        // 基本設定
        assert_eq!(config.sync_interval_ms, 10000);
        assert_eq!(config.batch_interval_ms, 60000);
        assert_eq!(config.challenge_interval_ms, 300000);
    }
    
    #[test]
    fn test_rollup_config() {
        let config = RollupConfig::default();
        
        assert_eq!(config.id, "default_rollup");
        assert_eq!(config.name, "Default Rollup");
        assert_eq!(config.rollup_type, RollupType::Optimistic);
        assert_eq!(config.data_availability_mode, DataAvailabilityMode::OnChain);
        assert_eq!(config.verification_mode, VerificationMode::Optimistic);
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.challenge_period_seconds, 604800);
        assert_eq!(config.max_transaction_size, 65536);
        assert_eq!(config.max_batch_size, 1048576);
        assert_eq!(config.gas_limit, 10000000);
        assert_eq!(config.gas_price, 1000000000);
    }
    
    #[test]
    fn test_sidechain_config() {
        let config = SidechainConfig::default();
        
        assert_eq!(config.id, "default_sidechain");
        assert_eq!(config.name, "Default Sidechain");
        assert_eq!(config.consensus_algorithm, ConsensusAlgorithm::ProofOfAuthority);
        assert_eq!(config.block_time_seconds, 5);
        assert_eq!(config.max_block_size, 1048576);
        assert_eq!(config.max_transaction_size, 65536);
        assert_eq!(config.gas_limit, 10000000);
        assert_eq!(config.gas_price, 1000000000);
        assert_eq!(config.validator_count, 5);
    }
    
    #[test]
    fn test_plasma_config() {
        let config = PlasmaConfig::default();
        
        assert_eq!(config.id, "default_plasma");
        assert_eq!(config.name, "Default Plasma");
        assert_eq!(config.plasma_type, PlasmaType::MVP);
        assert_eq!(config.block_time_seconds, 15);
        assert_eq!(config.max_block_size, 1048576);
        assert_eq!(config.max_transaction_size, 65536);
        assert_eq!(config.withdrawal_delay_seconds, 604800);
        assert_eq!(config.challenge_period_seconds, 604800);
    }
    
    #[test]
    fn test_validator_config() {
        let config = ValidatorConfig::default();
        
        assert_eq!(config.id, "default_validator");
        assert_eq!(config.name, "Default Validator");
        assert_eq!(config.consensus_algorithm, ConsensusAlgorithm::ProofOfStake);
        assert_eq!(config.validator_count, 5);
        assert_eq!(config.min_stake_amount, 1000000000000000000);
        assert_eq!(config.reward_rate, 0.05);
        assert_eq!(config.slashing_conditions.double_signing_penalty_percentage, 100);
        assert_eq!(config.slashing_conditions.offline_penalty_percentage, 10);
        assert_eq!(config.slashing_conditions.malicious_behavior_penalty_percentage, 100);
    }
    
    #[test]
    fn test_sync_config() {
        let config = SyncConfig::default();
        
        assert_eq!(config.sync_mode, SyncMode::Fast);
        assert_eq!(config.max_sync_blocks, 1000);
        assert_eq!(config.sync_timeout_seconds, 300);
        assert_eq!(config.sync_interval_seconds, 60);
        assert_eq!(config.sync_peer_count, 3);
    }
    
    #[test]
    fn test_challenge_config() {
        let config = ChallengeConfig::default();
        
        assert_eq!(config.challenge_types.len(), 5);
        assert!(config.challenge_types.contains(&ChallengeType::InvalidTransaction));
        assert!(config.challenge_types.contains(&ChallengeType::InvalidStateTransition));
        assert!(config.challenge_types.contains(&ChallengeType::InvalidBlock));
        assert!(config.challenge_types.contains(&ChallengeType::InvalidProof));
        assert!(config.challenge_types.contains(&ChallengeType::DataAvailability));
        assert_eq!(config.challenge_period_seconds, 604800);
        assert_eq!(config.challenge_bond_amount, 1000000000000000000);
        assert_eq!(config.challenge_reward_rate, 0.1);
    }
    
    #[test]
    fn test_batch_config() {
        let config = BatchConfig::default();
        
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.max_batch_size, 1048576);
        assert_eq!(config.batch_interval_seconds, 600);
        assert_eq!(config.batch_submission_gas_limit, 1000000);
    }
}