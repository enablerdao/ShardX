//! トランザクションタイプモジュール
//! 
//! このモジュールはShardXのトランザクションタイプを定義します。

use serde::{Serialize, Deserialize};

/// トランザクションタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransactionType {
    /// 通常の送金トランザクション
    Transfer,
    /// スマートコントラクトの呼び出し
    ContractCall,
    /// スマートコントラクトのデプロイ
    ContractDeploy,
    /// ステーキング
    Staking,
    /// 投票
    Voting,
    /// マルチシグトランザクション
    Multisig,
    /// クロスシャードトランザクション
    CrossShard,
    /// データストレージ
    DataStorage,
    /// オラクルデータ更新
    OracleUpdate,
    /// DEX取引
    DexTrade,
    /// その他
    Other,
}

impl TransactionType {
    /// トランザクションタイプが高優先度かどうかを判定
    pub fn is_high_priority(&self) -> bool {
        match self {
            TransactionType::OracleUpdate => true,
            TransactionType::Voting => true,
            _ => false,
        }
    }
    
    /// トランザクションタイプがクロスシャードかどうかを判定
    pub fn is_cross_shard(&self) -> bool {
        match self {
            TransactionType::CrossShard => true,
            _ => false,
        }
    }
    
    /// トランザクションタイプがスマートコントラクト関連かどうかを判定
    pub fn is_contract_related(&self) -> bool {
        match self {
            TransactionType::ContractCall => true,
            TransactionType::ContractDeploy => true,
            _ => false,
        }
    }
    
    /// トランザクションタイプがDEX関連かどうかを判定
    pub fn is_dex_related(&self) -> bool {
        match self {
            TransactionType::DexTrade => true,
            _ => false,
        }
    }
    
    /// トランザクションタイプがガバナンス関連かどうかを判定
    pub fn is_governance_related(&self) -> bool {
        match self {
            TransactionType::Voting => true,
            _ => false,
        }
    }
    
    /// トランザクションタイプの文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionType::Transfer => "transfer",
            TransactionType::ContractCall => "contract_call",
            TransactionType::ContractDeploy => "contract_deploy",
            TransactionType::Staking => "staking",
            TransactionType::Voting => "voting",
            TransactionType::Multisig => "multisig",
            TransactionType::CrossShard => "cross_shard",
            TransactionType::DataStorage => "data_storage",
            TransactionType::OracleUpdate => "oracle_update",
            TransactionType::DexTrade => "dex_trade",
            TransactionType::Other => "other",
        }
    }
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for TransactionType {
    fn default() -> Self {
        TransactionType::Transfer
    }
}

impl From<&str> for TransactionType {
    fn from(s: &str) -> Self {
        match s {
            "transfer" => TransactionType::Transfer,
            "contract_call" => TransactionType::ContractCall,
            "contract_deploy" => TransactionType::ContractDeploy,
            "staking" => TransactionType::Staking,
            "voting" => TransactionType::Voting,
            "multisig" => TransactionType::Multisig,
            "cross_shard" => TransactionType::CrossShard,
            "data_storage" => TransactionType::DataStorage,
            "oracle_update" => TransactionType::OracleUpdate,
            "dex_trade" => TransactionType::DexTrade,
            _ => TransactionType::Other,
        }
    }
}
