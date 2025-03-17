use crate::error::Error;
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 報酬タイプ
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RewardType {
    /// バリデーター
    Validator,
    /// ステーカー
    Staker,
    /// 流動性提供者
    LiquidityProvider,
    /// 財務
    Treasury,
    /// 紹介
    Referral,
    /// 貢献者
    Contributor,
    /// その他
    Other,
}

/// 報酬
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reward {
    /// 報酬ID
    pub id: String,
    /// 受取人
    pub recipient: String,
    /// 金額
    pub amount: u64,
    /// 報酬タイプ
    pub reward_type: RewardType,
    /// ブロック高
    pub block_height: u64,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 支払い日時
    pub paid_at: Option<DateTime<Utc>>,
    /// ロック解除日時
    pub unlock_at: Option<DateTime<Utc>>,
    /// ステータス
    pub status: RewardStatus,
    /// トランザクションID
    pub transaction_id: Option<String>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// 報酬ステータス
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RewardStatus {
    /// 保留中
    Pending,
    /// ロック中
    Locked,
    /// 支払い済み
    Paid,
    /// キャンセル
    Cancelled,
}

/// 報酬分配
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RewardDistribution {
    /// 分配ID
    pub id: String,
    /// ブロック高
    pub block_height: u64,
    /// 合計金額
    pub total_amount: u64,
    /// 分配マップ
    pub distribution: HashMap<String, u64>,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// 報酬設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RewardConfig {
    /// ブロック報酬
    pub block_reward: u64,
    /// 報酬半減期（ブロック数）
    pub halving_period: u64,
    /// 最大供給量
    pub max_supply: u64,
    /// 報酬分配比率
    pub distribution_ratio: HashMap<RewardType, f64>,
    /// 報酬ロックアップ期間（秒）
    pub lockup_period_seconds: u64,
}

/// 報酬マネージャー
pub struct RewardManager {
    /// 報酬設定
    config: RewardConfig,
    /// 報酬マップ
    rewards: HashMap<String, Reward>,
    /// 受取人ごとの報酬リスト
    recipient_rewards: HashMap<String, Vec<String>>,
    /// 報酬分配リスト
    distributions: Vec<RewardDistribution>,
    /// 合計発行量
    total_issued: u64,
}

impl RewardManager {
    /// 新しいRewardManagerを作成
    pub fn new(config: RewardConfig) -> Self {
        Self {
            config,
            rewards: HashMap::new(),
            recipient_rewards: HashMap::new(),
            distributions: Vec::new(),
            total_issued: 0,
        }
    }

    /// 報酬を記録
    pub fn record_reward(
        &mut self,
        recipient: &str,
        amount: u64,
        reward_type: RewardType,
        block_height: u64,
    ) -> Result<String, Error> {
        // 最大供給量をチェック
        if self.total_issued + amount > self.config.max_supply {
            return Err(Error::InvalidState("Maximum supply reached".to_string()));
        }

        // 報酬IDを生成
        let reward_id = Uuid::new_v4().to_string();

        // 現在時刻を取得
        let now = Utc::now();

        // ロック解除日時を計算
        let unlock_at = if self.config.lockup_period_seconds > 0 {
            Some(now + chrono::Duration::seconds(self.config.lockup_period_seconds as i64))
        } else {
            None
        };

        // 報酬ステータスを決定
        let status = if unlock_at.is_some() {
            RewardStatus::Locked
        } else {
            RewardStatus::Pending
        };

        // 報酬を作成
        let reward = Reward {
            id: reward_id.clone(),
            recipient: recipient.to_string(),
            amount,
            reward_type,
            block_height,
            created_at: now,
            paid_at: None,
            unlock_at,
            status,
            transaction_id: None,
            metadata: HashMap::new(),
        };

        // 報酬を保存
        self.rewards.insert(reward_id.clone(), reward);

        // 受取人の報酬リストに追加
        let recipient_rewards = self
            .recipient_rewards
            .entry(recipient.to_string())
            .or_insert_with(Vec::new);

        recipient_rewards.push(reward_id.clone());

        // 合計発行量を更新
        self.total_issued += amount;

        info!(
            "Reward recorded: {} tokens to {} ({})",
            amount, recipient, reward_id
        );

        Ok(reward_id)
    }

    /// 報酬分配を記録
    pub fn record_distribution(
        &mut self,
        block_height: u64,
        distribution: HashMap<String, u64>,
    ) -> Result<String, Error> {
        // 分配IDを生成
        let distribution_id = Uuid::new_v4().to_string();

        // 合計金額を計算
        let total_amount: u64 = distribution.values().sum();

        // 最大供給量をチェック
        if self.total_issued + total_amount > self.config.max_supply {
            return Err(Error::InvalidState("Maximum supply reached".to_string()));
        }

        // 現在時刻を取得
        let now = Utc::now();

        // 報酬分配を作成
        let reward_distribution = RewardDistribution {
            id: distribution_id.clone(),
            block_height,
            total_amount,
            distribution: distribution.clone(),
            created_at: now,
            metadata: HashMap::new(),
        };

        // 報酬分配を保存
        self.distributions.push(reward_distribution);

        // 合計発行量を更新
        self.total_issued += total_amount;

        info!(
            "Reward distribution recorded: {} tokens at block {}",
            total_amount, block_height
        );

        Ok(distribution_id)
    }

    /// 報酬を支払い
    pub fn pay_reward(&mut self, reward_id: &str, transaction_id: &str) -> Result<(), Error> {
        // 報酬を取得
        let reward = self
            .rewards
            .get_mut(reward_id)
            .ok_or_else(|| Error::NotFound(format!("Reward not found: {}", reward_id)))?;

        // ステータスをチェック
        if reward.status == RewardStatus::Paid {
            return Err(Error::InvalidState("Reward already paid".to_string()));
        }

        if reward.status == RewardStatus::Cancelled {
            return Err(Error::InvalidState("Reward is cancelled".to_string()));
        }

        // ロック解除日時をチェック
        if let Some(unlock_at) = reward.unlock_at {
            let now = Utc::now();
            if now < unlock_at {
                return Err(Error::InvalidState(format!(
                    "Reward is locked until {}",
                    unlock_at
                )));
            }
        }

        // 報酬を支払い
        reward.status = RewardStatus::Paid;
        reward.paid_at = Some(Utc::now());
        reward.transaction_id = Some(transaction_id.to_string());

        info!("Reward paid: {} (tx: {})", reward_id, transaction_id);

        Ok(())
    }

    /// 報酬をキャンセル
    pub fn cancel_reward(&mut self, reward_id: &str) -> Result<(), Error> {
        // 報酬を取得
        let reward = self
            .rewards
            .get_mut(reward_id)
            .ok_or_else(|| Error::NotFound(format!("Reward not found: {}", reward_id)))?;

        // ステータスをチェック
        if reward.status == RewardStatus::Paid {
            return Err(Error::InvalidState("Cannot cancel paid reward".to_string()));
        }

        // 報酬をキャンセル
        reward.status = RewardStatus::Cancelled;

        // 合計発行量を更新
        self.total_issued -= reward.amount;

        info!("Reward cancelled: {}", reward_id);

        Ok(())
    }

    /// 報酬を取得
    pub fn get_reward(&self, reward_id: &str) -> Result<Reward, Error> {
        self.rewards
            .get(reward_id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Reward not found: {}", reward_id)))
    }

    /// 報酬履歴を取得
    pub fn get_reward_history(&self, recipient: &str) -> Result<Vec<Reward>, Error> {
        let reward_ids = self
            .recipient_rewards
            .get(recipient)
            .cloned()
            .unwrap_or_default();

        let rewards: Vec<Reward> = reward_ids
            .iter()
            .filter_map(|id| self.rewards.get(id).cloned())
            .collect();

        Ok(rewards)
    }

    /// 報酬分配を取得
    pub fn get_distribution(&self, block_height: u64) -> Option<&RewardDistribution> {
        self.distributions
            .iter()
            .find(|d| d.block_height == block_height)
    }

    /// 合計発行量を取得
    pub fn get_total_issued(&self) -> u64 {
        self.total_issued
    }

    /// 残りの供給量を取得
    pub fn get_remaining_supply(&self) -> u64 {
        self.config.max_supply.saturating_sub(self.total_issued)
    }

    /// 設定を更新
    pub fn update_config(&mut self, config: RewardConfig) {
        self.config = config;
    }

    /// 設定を取得
    pub fn get_config(&self) -> &RewardConfig {
        &self.config
    }
}

impl Default for RewardConfig {
    fn default() -> Self {
        let mut distribution_ratio = HashMap::new();
        distribution_ratio.insert(RewardType::Validator, 0.4);
        distribution_ratio.insert(RewardType::Staker, 0.3);
        distribution_ratio.insert(RewardType::LiquidityProvider, 0.2);
        distribution_ratio.insert(RewardType::Treasury, 0.1);

        Self {
            block_reward: 100,
            halving_period: 2_100_000, // 約4年（1日あたり約1440ブロック）
            max_supply: 21_000_000_000,
            distribution_ratio,
            lockup_period_seconds: 86400, // 1日
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reward_recording() {
        let config = RewardConfig::default();
        let mut manager = RewardManager::new(config);

        // 報酬を記録
        let reward_id = manager
            .record_reward("recipient1", 100, RewardType::Validator, 12345)
            .unwrap();

        // 報酬を取得
        let reward = manager.get_reward(&reward_id).unwrap();

        // 報酬をチェック
        assert_eq!(reward.recipient, "recipient1");
        assert_eq!(reward.amount, 100);
        assert_eq!(reward.reward_type, RewardType::Validator);
        assert_eq!(reward.block_height, 12345);
        assert_eq!(reward.status, RewardStatus::Locked);
        assert!(reward.unlock_at.is_some());
        assert!(reward.paid_at.is_none());
        assert!(reward.transaction_id.is_none());

        // 合計発行量をチェック
        assert_eq!(manager.get_total_issued(), 100);
        assert_eq!(manager.get_remaining_supply(), 21_000_000_000 - 100);
    }

    #[test]
    fn test_reward_payment() {
        let mut config = RewardConfig::default();
        config.lockup_period_seconds = 0; // ロックアップなし

        let mut manager = RewardManager::new(config);

        // 報酬を記録
        let reward_id = manager
            .record_reward("recipient1", 100, RewardType::Validator, 12345)
            .unwrap();

        // 報酬を支払い
        manager.pay_reward(&reward_id, "tx123").unwrap();

        // 報酬を取得
        let reward = manager.get_reward(&reward_id).unwrap();

        // 報酬をチェック
        assert_eq!(reward.status, RewardStatus::Paid);
        assert!(reward.paid_at.is_some());
        assert_eq!(reward.transaction_id, Some("tx123".to_string()));
    }

    #[test]
    fn test_reward_cancellation() {
        let config = RewardConfig::default();
        let mut manager = RewardManager::new(config);

        // 報酬を記録
        let reward_id = manager
            .record_reward("recipient1", 100, RewardType::Validator, 12345)
            .unwrap();

        // 報酬をキャンセル
        manager.cancel_reward(&reward_id).unwrap();

        // 報酬を取得
        let reward = manager.get_reward(&reward_id).unwrap();

        // 報酬をチェック
        assert_eq!(reward.status, RewardStatus::Cancelled);

        // 合計発行量をチェック
        assert_eq!(manager.get_total_issued(), 0);
    }

    #[test]
    fn test_reward_history() {
        let config = RewardConfig::default();
        let mut manager = RewardManager::new(config);

        // 複数の報酬を記録
        manager
            .record_reward("recipient1", 100, RewardType::Validator, 12345)
            .unwrap();

        manager
            .record_reward("recipient1", 200, RewardType::Staker, 12346)
            .unwrap();

        manager
            .record_reward("recipient2", 300, RewardType::LiquidityProvider, 12347)
            .unwrap();

        // 報酬履歴を取得
        let history1 = manager.get_reward_history("recipient1").unwrap();
        let history2 = manager.get_reward_history("recipient2").unwrap();

        // 履歴をチェック
        assert_eq!(history1.len(), 2);
        assert_eq!(history2.len(), 1);

        assert_eq!(history1[0].amount, 100);
        assert_eq!(history1[0].reward_type, RewardType::Validator);

        assert_eq!(history1[1].amount, 200);
        assert_eq!(history1[1].reward_type, RewardType::Staker);

        assert_eq!(history2[0].amount, 300);
        assert_eq!(history2[0].reward_type, RewardType::LiquidityProvider);
    }

    #[test]
    fn test_max_supply_limit() {
        let mut config = RewardConfig::default();
        config.max_supply = 1000; // 最大供給量を小さく設定

        let mut manager = RewardManager::new(config);

        // 報酬を記録
        manager
            .record_reward("recipient1", 500, RewardType::Validator, 12345)
            .unwrap();

        manager
            .record_reward("recipient2", 400, RewardType::Staker, 12346)
            .unwrap();

        // 最大供給量を超える報酬を記録
        let result = manager.record_reward("recipient3", 200, RewardType::LiquidityProvider, 12347);

        // エラーをチェック
        assert!(result.is_err());

        // 合計発行量をチェック
        assert_eq!(manager.get_total_issued(), 900);
        assert_eq!(manager.get_remaining_supply(), 100);
    }
}
