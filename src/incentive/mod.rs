// インセンティブモジュール
//
// このモジュールは、ShardXのインセンティブメカニズムを提供します。
// 主な機能:
// - トークン報酬
// - ステーキング
// - 流動性マイニング
// - 紹介プログラム
// - 貢献者報酬

mod rewards;
// mod staking; // TODO: このモジュールが見つかりません
// mod liquidity_mining; // TODO: このモジュールが見つかりません
// mod referral; // TODO: このモジュールが見つかりません
// mod contributor; // TODO: このモジュールが見つかりません

pub use self::rewards::{Reward, RewardType, RewardDistribution, RewardManager};
pub use self::staking::{Stake, StakingPool, StakingManager, StakingConfig};
pub use self::liquidity_mining::{LiquidityPool, LiquidityMiningManager, LiquidityMiningConfig};
pub use self::referral::{Referral, ReferralProgram, ReferralManager, ReferralConfig};
pub use self::contributor::{Contribution, ContributorReward, ContributorManager, ContributionType};

use crate::error::Error;
use crate::metrics::MetricsCollector;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Serialize, Deserialize};

/// インセンティブ設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IncentiveConfig {
    /// トークン報酬設定
    pub reward_config: RewardConfig,
    /// ステーキング設定
    pub staking_config: StakingConfig,
    /// 流動性マイニング設定
    pub liquidity_mining_config: LiquidityMiningConfig,
    /// 紹介プログラム設定
    pub referral_config: ReferralConfig,
    /// 貢献者報酬設定
    pub contributor_config: ContributorConfig,
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

/// 貢献者設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContributorConfig {
    /// 貢献タイプごとの報酬率
    pub contribution_reward_rates: HashMap<ContributionType, f64>,
    /// 最小貢献閾値
    pub min_contribution_threshold: u64,
    /// 最大貢献報酬
    pub max_contribution_reward: u64,
    /// 報酬支払い頻度（秒）
    pub payment_frequency_seconds: u64,
    /// 報酬承認者
    pub reward_approvers: Vec<String>,
}

/// インセンティブマネージャー
pub struct IncentiveManager {
    /// インセンティブ設定
    config: IncentiveConfig,
    /// 報酬マネージャー
    reward_manager: RewardManager,
    /// ステーキングマネージャー
    staking_manager: StakingManager,
    /// 流動性マイニングマネージャー
    liquidity_mining_manager: LiquidityMiningManager,
    /// 紹介マネージャー
    referral_manager: ReferralManager,
    /// 貢献者マネージャー
    contributor_manager: ContributorManager,
    /// メトリクスコレクター
    metrics: Arc<MetricsCollector>,
}

impl IncentiveManager {
    /// 新しいIncentiveManagerを作成
    pub fn new(config: IncentiveConfig, metrics: Arc<MetricsCollector>) -> Self {
        let reward_manager = RewardManager::new(config.reward_config.clone());
        let staking_manager = StakingManager::new(config.staking_config.clone());
        let liquidity_mining_manager = LiquidityMiningManager::new(config.liquidity_mining_config.clone());
        let referral_manager = ReferralManager::new(config.referral_config.clone());
        let contributor_manager = ContributorManager::new(config.contributor_config.clone());
        
        Self {
            config,
            reward_manager,
            staking_manager,
            liquidity_mining_manager,
            referral_manager,
            contributor_manager,
            metrics,
        }
    }
    
    /// ブロック報酬を計算
    pub fn calculate_block_reward(&self, block_height: u64) -> u64 {
        let halvings = block_height / self.config.reward_config.halving_period;
        let reward = self.config.reward_config.block_reward >> halvings;
        
        reward
    }
    
    /// 報酬を分配
    pub fn distribute_rewards(
        &mut self,
        block_height: u64,
        validators: &[String],
        stakers: &[String],
        liquidity_providers: &[String],
    ) -> Result<HashMap<String, u64>, Error> {
        // ブロック報酬を計算
        let block_reward = self.calculate_block_reward(block_height);
        
        // 報酬分配比率を取得
        let validator_ratio = self.config.reward_config.distribution_ratio
            .get(&RewardType::Validator)
            .cloned()
            .unwrap_or(0.4);
        
        let staker_ratio = self.config.reward_config.distribution_ratio
            .get(&RewardType::Staker)
            .cloned()
            .unwrap_or(0.3);
        
        let liquidity_ratio = self.config.reward_config.distribution_ratio
            .get(&RewardType::LiquidityProvider)
            .cloned()
            .unwrap_or(0.2);
        
        let treasury_ratio = self.config.reward_config.distribution_ratio
            .get(&RewardType::Treasury)
            .cloned()
            .unwrap_or(0.1);
        
        // 報酬を計算
        let validator_reward = (block_reward as f64 * validator_ratio) as u64;
        let staker_reward = (block_reward as f64 * staker_ratio) as u64;
        let liquidity_reward = (block_reward as f64 * liquidity_ratio) as u64;
        let treasury_reward = (block_reward as f64 * treasury_ratio) as u64;
        
        // 報酬分配を作成
        let mut distribution = HashMap::new();
        
        // バリデーター報酬を分配
        if !validators.is_empty() {
            let reward_per_validator = validator_reward / validators.len() as u64;
            
            for validator in validators {
                distribution.insert(validator.clone(), reward_per_validator);
                
                // 報酬を記録
                self.reward_manager.record_reward(
                    validator,
                    reward_per_validator,
                    RewardType::Validator,
                    block_height,
                )?;
            }
        } else {
            // バリデーターがいない場合は財務に追加
            distribution.insert("treasury".to_string(), validator_reward + treasury_reward);
        }
        
        // ステーカー報酬を分配
        if !stakers.is_empty() {
            // ステーキング残高に基づいて報酬を分配
            let staking_balances = self.staking_manager.get_staking_balances(stakers)?;
            let total_staked: u64 = staking_balances.values().sum();
            
            if total_staked > 0 {
                for (staker, balance) in staking_balances {
                    let reward = (staker_reward as f64 * balance as f64 / total_staked as f64) as u64;
                    
                    // 既存の報酬に追加
                    let current_reward = distribution.get(&staker).cloned().unwrap_or(0);
                    distribution.insert(staker.clone(), current_reward + reward);
                    
                    // 報酬を記録
                    self.reward_manager.record_reward(
                        &staker,
                        reward,
                        RewardType::Staker,
                        block_height,
                    )?;
                }
            } else {
                // ステーキングがない場合は財務に追加
                let treasury_amount = distribution.get("treasury").cloned().unwrap_or(0);
                distribution.insert("treasury".to_string(), treasury_amount + staker_reward);
            }
        } else {
            // ステーカーがいない場合は財務に追加
            let treasury_amount = distribution.get("treasury").cloned().unwrap_or(0);
            distribution.insert("treasury".to_string(), treasury_amount + staker_reward);
        }
        
        // 流動性提供者報酬を分配
        if !liquidity_providers.is_empty() {
            // 流動性に基づいて報酬を分配
            let liquidity_balances = self.liquidity_mining_manager.get_liquidity_balances(liquidity_providers)?;
            let total_liquidity: u64 = liquidity_balances.values().sum();
            
            if total_liquidity > 0 {
                for (provider, balance) in liquidity_balances {
                    let reward = (liquidity_reward as f64 * balance as f64 / total_liquidity as f64) as u64;
                    
                    // 既存の報酬に追加
                    let current_reward = distribution.get(&provider).cloned().unwrap_or(0);
                    distribution.insert(provider.clone(), current_reward + reward);
                    
                    // 報酬を記録
                    self.reward_manager.record_reward(
                        &provider,
                        reward,
                        RewardType::LiquidityProvider,
                        block_height,
                    )?;
                }
            } else {
                // 流動性がない場合は財務に追加
                let treasury_amount = distribution.get("treasury").cloned().unwrap_or(0);
                distribution.insert("treasury".to_string(), treasury_amount + liquidity_reward);
            }
        } else {
            // 流動性提供者がいない場合は財務に追加
            let treasury_amount = distribution.get("treasury").cloned().unwrap_or(0);
            distribution.insert("treasury".to_string(), treasury_amount + liquidity_reward);
        }
        
        // 財務報酬を記録
        let treasury_amount = distribution.get("treasury").cloned().unwrap_or(0);
        if treasury_amount > 0 {
            self.reward_manager.record_reward(
                "treasury",
                treasury_amount,
                RewardType::Treasury,
                block_height,
            )?;
        }
        
        // メトリクスを更新
        self.metrics.increment_counter("incentive_rewards_distributed");
        self.metrics.set_gauge("incentive_block_reward", block_reward as f64);
        
        Ok(distribution)
    }
    
    /// ステーキングを作成
    pub async fn create_stake(
        &mut self,
        staker: &str,
        amount: u64,
        duration_seconds: u64,
    ) -> Result<String, Error> {
        // ステーキングを作成
        let stake_id = self.staking_manager.create_stake(staker, amount, duration_seconds)?;
        
        // メトリクスを更新
        self.metrics.increment_counter("incentive_stakes_created");
        self.metrics.increment_gauge("incentive_total_staked", amount as f64);
        
        Ok(stake_id)
    }
    
    /// ステーキングを解除
    pub async fn unstake(
        &mut self,
        stake_id: &str,
        staker: &str,
    ) -> Result<u64, Error> {
        // ステーキングを解除
        let amount = self.staking_manager.unstake(stake_id, staker)?;
        
        // メトリクスを更新
        self.metrics.increment_counter("incentive_stakes_unstaked");
        self.metrics.decrement_gauge("incentive_total_staked", amount as f64);
        
        Ok(amount)
    }
    
    /// 流動性を追加
    pub async fn add_liquidity(
        &mut self,
        provider: &str,
        pool_id: &str,
        amount: u64,
    ) -> Result<(), Error> {
        // 流動性を追加
        self.liquidity_mining_manager.add_liquidity(provider, pool_id, amount)?;
        
        // メトリクスを更新
        self.metrics.increment_counter("incentive_liquidity_added");
        self.metrics.increment_gauge("incentive_total_liquidity", amount as f64);
        
        Ok(())
    }
    
    /// 流動性を削除
    pub async fn remove_liquidity(
        &mut self,
        provider: &str,
        pool_id: &str,
        amount: u64,
    ) -> Result<(), Error> {
        // 流動性を削除
        self.liquidity_mining_manager.remove_liquidity(provider, pool_id, amount)?;
        
        // メトリクスを更新
        self.metrics.increment_counter("incentive_liquidity_removed");
        self.metrics.decrement_gauge("incentive_total_liquidity", amount as f64);
        
        Ok(())
    }
    
    /// 紹介を作成
    pub async fn create_referral(
        &mut self,
        referrer: &str,
        referee: &str,
    ) -> Result<String, Error> {
        // 紹介を作成
        let referral_id = self.referral_manager.create_referral(referrer, referee)?;
        
        // メトリクスを更新
        self.metrics.increment_counter("incentive_referrals_created");
        
        Ok(referral_id)
    }
    
    /// 紹介報酬を処理
    pub async fn process_referral_reward(
        &mut self,
        referral_id: &str,
        amount: u64,
    ) -> Result<u64, Error> {
        // 紹介報酬を処理
        let reward = self.referral_manager.process_reward(referral_id, amount)?;
        
        // メトリクスを更新
        self.metrics.increment_counter("incentive_referral_rewards_processed");
        self.metrics.increment_gauge("incentive_total_referral_rewards", reward as f64);
        
        Ok(reward)
    }
    
    /// 貢献を記録
    pub async fn record_contribution(
        &mut self,
        contributor: &str,
        contribution_type: ContributionType,
        value: u64,
        description: &str,
    ) -> Result<String, Error> {
        // 貢献を記録
        let contribution_id = self.contributor_manager.record_contribution(
            contributor,
            contribution_type,
            value,
            description,
        )?;
        
        // メトリクスを更新
        self.metrics.increment_counter("incentive_contributions_recorded");
        
        Ok(contribution_id)
    }
    
    /// 貢献報酬を承認
    pub async fn approve_contribution_reward(
        &mut self,
        contribution_id: &str,
        approver: &str,
        reward_amount: u64,
    ) -> Result<(), Error> {
        // 貢献報酬を承認
        self.contributor_manager.approve_reward(contribution_id, approver, reward_amount)?;
        
        // メトリクスを更新
        self.metrics.increment_counter("incentive_contribution_rewards_approved");
        self.metrics.increment_gauge("incentive_total_contribution_rewards", reward_amount as f64);
        
        Ok(())
    }
    
    /// 報酬履歴を取得
    pub fn get_reward_history(&self, address: &str) -> Result<Vec<Reward>, Error> {
        self.reward_manager.get_reward_history(address)
    }
    
    /// ステーキング情報を取得
    pub fn get_stake(&self, stake_id: &str) -> Result<Stake, Error> {
        self.staking_manager.get_stake(stake_id)
    }
    
    /// アドレスのステーキングリストを取得
    pub fn get_stakes_by_address(&self, address: &str) -> Result<Vec<Stake>, Error> {
        self.staking_manager.get_stakes_by_address(address)
    }
    
    /// 流動性プールを取得
    pub fn get_liquidity_pool(&self, pool_id: &str) -> Result<LiquidityPool, Error> {
        self.liquidity_mining_manager.get_pool(pool_id)
    }
    
    /// アドレスの流動性を取得
    pub fn get_liquidity_by_address(&self, address: &str) -> Result<HashMap<String, u64>, Error> {
        self.liquidity_mining_manager.get_liquidity_by_address(address)
    }
    
    /// 紹介を取得
    pub fn get_referral(&self, referral_id: &str) -> Result<Referral, Error> {
        self.referral_manager.get_referral(referral_id)
    }
    
    /// 紹介者の紹介リストを取得
    pub fn get_referrals_by_referrer(&self, referrer: &str) -> Result<Vec<Referral>, Error> {
        self.referral_manager.get_referrals_by_referrer(referrer)
    }
    
    /// 貢献を取得
    pub fn get_contribution(&self, contribution_id: &str) -> Result<Contribution, Error> {
        self.contributor_manager.get_contribution(contribution_id)
    }
    
    /// 貢献者の貢献リストを取得
    pub fn get_contributions_by_contributor(&self, contributor: &str) -> Result<Vec<Contribution>, Error> {
        self.contributor_manager.get_contributions_by_contributor(contributor)
    }
    
    /// 設定を取得
    pub fn get_config(&self) -> &IncentiveConfig {
        &self.config
    }
    
    /// 設定を更新
    pub fn update_config(&mut self, config: IncentiveConfig) {
        self.config = config.clone();
        self.reward_manager.update_config(config.reward_config);
        self.staking_manager.update_config(config.staking_config);
        self.liquidity_mining_manager.update_config(config.liquidity_mining_config);
        self.referral_manager.update_config(config.referral_config);
        self.contributor_manager.update_config(config.contributor_config);
    }
}

impl Default for IncentiveConfig {
    fn default() -> Self {
        Self {
            reward_config: RewardConfig::default(),
            staking_config: StakingConfig::default(),
            liquidity_mining_config: LiquidityMiningConfig::default(),
            referral_config: ReferralConfig::default(),
            contributor_config: ContributorConfig::default(),
        }
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

impl Default for ContributorConfig {
    fn default() -> Self {
        let mut contribution_reward_rates = HashMap::new();
        contribution_reward_rates.insert(ContributionType::Development, 1.0);
        contribution_reward_rates.insert(ContributionType::Marketing, 0.8);
        contribution_reward_rates.insert(ContributionType::Community, 0.7);
        contribution_reward_rates.insert(ContributionType::Content, 0.6);
        contribution_reward_rates.insert(ContributionType::Translation, 0.5);
        contribution_reward_rates.insert(ContributionType::BugReport, 0.4);
        contribution_reward_rates.insert(ContributionType::Other, 0.3);
        
        Self {
            contribution_reward_rates,
            min_contribution_threshold: 10,
            max_contribution_reward: 10000,
            payment_frequency_seconds: 604800, // 1週間
            reward_approvers: vec!["admin".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_incentive_config() {
        let config = IncentiveConfig::default();
        
        // 報酬設定をチェック
        assert_eq!(config.reward_config.block_reward, 100);
        assert_eq!(config.reward_config.halving_period, 2_100_000);
        assert_eq!(config.reward_config.max_supply, 21_000_000_000);
        assert_eq!(config.reward_config.lockup_period_seconds, 86400);
        
        // 分配比率をチェック
        assert_eq!(
            *config.reward_config.distribution_ratio.get(&RewardType::Validator).unwrap(),
            0.4
        );
        assert_eq!(
            *config.reward_config.distribution_ratio.get(&RewardType::Staker).unwrap(),
            0.3
        );
        assert_eq!(
            *config.reward_config.distribution_ratio.get(&RewardType::LiquidityProvider).unwrap(),
            0.2
        );
        assert_eq!(
            *config.reward_config.distribution_ratio.get(&RewardType::Treasury).unwrap(),
            0.1
        );
        
        // 貢献者設定をチェック
        assert_eq!(config.contributor_config.min_contribution_threshold, 10);
        assert_eq!(config.contributor_config.max_contribution_reward, 10000);
        assert_eq!(config.contributor_config.payment_frequency_seconds, 604800);
        assert_eq!(config.contributor_config.reward_approvers, vec!["admin".to_string()]);
    }
    
    #[test]
    fn test_block_reward_calculation() {
        let config = IncentiveConfig::default();
        let metrics = Arc::new(MetricsCollector::new("incentive"));
        let manager = IncentiveManager::new(config, metrics);
        
        // 初期ブロック報酬をチェック
        assert_eq!(manager.calculate_block_reward(0), 100);
        
        // 半減期前のブロック報酬をチェック
        assert_eq!(manager.calculate_block_reward(2_100_000 - 1), 100);
        
        // 半減期後のブロック報酬をチェック
        assert_eq!(manager.calculate_block_reward(2_100_000), 50);
        
        // 2回目の半減期後のブロック報酬をチェック
        assert_eq!(manager.calculate_block_reward(2 * 2_100_000), 25);
        
        // 3回目の半減期後のブロック報酬をチェック
        assert_eq!(manager.calculate_block_reward(3 * 2_100_000), 12);
    }
    
    #[tokio::test]
    async fn test_reward_distribution() {
        let config = IncentiveConfig::default();
        let metrics = Arc::new(MetricsCollector::new("incentive"));
        let mut manager = IncentiveManager::new(config, metrics);
        
        // バリデーター、ステーカー、流動性提供者を設定
        let validators = vec!["validator1".to_string(), "validator2".to_string()];
        let stakers = vec!["staker1".to_string(), "staker2".to_string()];
        let liquidity_providers = vec!["provider1".to_string(), "provider2".to_string()];
        
        // ステーキング残高を設定
        manager.staking_manager.test_set_staking_balance("staker1", 1000).unwrap();
        manager.staking_manager.test_set_staking_balance("staker2", 2000).unwrap();
        
        // 流動性残高を設定
        manager.liquidity_mining_manager.test_set_liquidity_balance("provider1", "pool1", 3000).unwrap();
        manager.liquidity_mining_manager.test_set_liquidity_balance("provider2", "pool1", 1000).unwrap();
        
        // 報酬を分配
        let distribution = manager.distribute_rewards(
            0, // ブロック高
            &validators,
            &stakers,
            &liquidity_providers,
        ).unwrap();
        
        // 報酬分配をチェック
        assert_eq!(distribution.len(), 6); // バリデーター2 + ステーカー2 + 流動性提供者2
        
        // バリデーター報酬をチェック（40%を均等に分配）
        assert_eq!(distribution.get("validator1").unwrap(), &20); // 100 * 0.4 / 2
        assert_eq!(distribution.get("validator2").unwrap(), &20); // 100 * 0.4 / 2
        
        // ステーカー報酬をチェック（30%を残高に応じて分配）
        assert_eq!(distribution.get("staker1").unwrap(), &10); // 100 * 0.3 * (1000 / 3000)
        assert_eq!(distribution.get("staker2").unwrap(), &20); // 100 * 0.3 * (2000 / 3000)
        
        // 流動性提供者報酬をチェック（20%を残高に応じて分配）
        assert_eq!(distribution.get("provider1").unwrap(), &15); // 100 * 0.2 * (3000 / 4000)
        assert_eq!(distribution.get("provider2").unwrap(), &5);  // 100 * 0.2 * (1000 / 4000)
        
        // 財務報酬をチェック（10%）
        assert_eq!(distribution.get("treasury").unwrap(), &10); // 100 * 0.1
    }
}