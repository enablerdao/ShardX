use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::crypto::{hash, PublicKey, Signature};
use crate::error::Error;
use crate::wallet::multisig::threshold::ThresholdPolicy;
use crate::wallet::WalletId;

/// マルチシグウォレット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedMultisigWallet {
    /// ウォレットID
    pub id: WalletId,
    /// ウォレット名
    pub name: String,
    /// 閾値ポリシー
    pub policy: ThresholdPolicy,
    /// 残高
    pub balance: u64,
    /// 作成時刻
    pub created_at: DateTime<Utc>,
    /// 更新時刻
    pub updated_at: DateTime<Utc>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
    /// ウォレットの説明
    pub description: Option<String>,
    /// ウォレットのタグ
    pub tags: Vec<String>,
    /// ウォレットの状態
    pub status: WalletStatus,
    /// 日次取引制限
    pub daily_limit: Option<u64>,
    /// 取引制限の残り
    pub remaining_daily_limit: Option<u64>,
    /// 制限リセット時刻
    pub limit_reset_time: Option<DateTime<Utc>>,
}

/// ウォレットの状態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WalletStatus {
    /// アクティブ
    Active,
    /// 凍結
    Frozen,
    /// 無効
    Disabled,
    /// アーカイブ
    Archived,
}

impl EnhancedMultisigWallet {
    /// 新しいマルチシグウォレットを作成
    pub fn new(name: String, policy: ThresholdPolicy) -> Self {
        let now = Utc::now();
        let id = format!("wallet-{}", hash(&format!("{}-{}", name, now)));

        Self {
            id,
            name,
            policy,
            balance: 0,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
            description: None,
            tags: Vec::new(),
            status: WalletStatus::Active,
            daily_limit: None,
            remaining_daily_limit: None,
            limit_reset_time: None,
        }
    }

    /// ポリシーを更新
    pub fn update_policy(&mut self, policy: ThresholdPolicy) -> Result<(), Error> {
        if !policy.is_valid() {
            return Err(Error::InvalidInput("無効なポリシーです".to_string()));
        }

        self.policy = policy;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// 残高を更新
    pub fn update_balance(&mut self, balance: u64) {
        self.balance = balance;
        self.updated_at = Utc::now();
    }

    /// 残高を増加
    pub fn increase_balance(&mut self, amount: u64) {
        self.balance += amount;
        self.updated_at = Utc::now();
    }

    /// 残高を減少
    pub fn decrease_balance(&mut self, amount: u64) -> Result<(), Error> {
        if amount > self.balance {
            return Err(Error::InsufficientFunds("残高が不足しています".to_string()));
        }

        self.balance -= amount;
        self.updated_at = Utc::now();

        // 日次制限を更新
        if let Some(remaining) = self.remaining_daily_limit {
            if amount <= remaining {
                self.remaining_daily_limit = Some(remaining - amount);
            } else {
                return Err(Error::LimitExceeded(
                    "日次取引制限を超えています".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// メタデータを設定
    pub fn set_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
        self.updated_at = Utc::now();
    }

    /// メタデータを取得
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// 説明を設定
    pub fn set_description(&mut self, description: &str) {
        self.description = Some(description.to_string());
        self.updated_at = Utc::now();
    }

    /// タグを追加
    pub fn add_tag(&mut self, tag: &str) {
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
            self.updated_at = Utc::now();
        }
    }

    /// タグを削除
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
        self.updated_at = Utc::now();
    }

    /// 状態を更新
    pub fn update_status(&mut self, status: WalletStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// ウォレットを凍結
    pub fn freeze(&mut self) {
        self.status = WalletStatus::Frozen;
        self.updated_at = Utc::now();
    }

    /// ウォレットの凍結を解除
    pub fn unfreeze(&mut self) {
        self.status = WalletStatus::Active;
        self.updated_at = Utc::now();
    }

    /// ウォレットを無効化
    pub fn disable(&mut self) {
        self.status = WalletStatus::Disabled;
        self.updated_at = Utc::now();
    }

    /// ウォレットをアーカイブ
    pub fn archive(&mut self) {
        self.status = WalletStatus::Archived;
        self.updated_at = Utc::now();
    }

    /// ウォレットがアクティブかどうかを確認
    pub fn is_active(&self) -> bool {
        self.status == WalletStatus::Active
    }

    /// 日次取引制限を設定
    pub fn set_daily_limit(&mut self, limit: u64) {
        self.daily_limit = Some(limit);
        self.remaining_daily_limit = Some(limit);

        // 制限リセット時刻を設定（翌日の0時）
        let now = Utc::now();
        let tomorrow = now + chrono::Duration::days(1);
        let reset_time = Utc
            .ymd(tomorrow.year(), tomorrow.month(), tomorrow.day())
            .and_hms(0, 0, 0);
        self.limit_reset_time = Some(reset_time);

        self.updated_at = now;
    }

    /// 日次取引制限を解除
    pub fn remove_daily_limit(&mut self) {
        self.daily_limit = None;
        self.remaining_daily_limit = None;
        self.limit_reset_time = None;
        self.updated_at = Utc::now();
    }

    /// 日次制限をリセット（必要な場合）
    pub fn reset_daily_limit_if_needed(&mut self) {
        if let (Some(limit), Some(reset_time)) = (self.daily_limit, self.limit_reset_time) {
            let now = Utc::now();
            if now >= reset_time {
                self.remaining_daily_limit = Some(limit);

                // 次の制限リセット時刻を設定
                let tomorrow = now + chrono::Duration::days(1);
                let next_reset = Utc
                    .ymd(tomorrow.year(), tomorrow.month(), tomorrow.day())
                    .and_hms(0, 0, 0);
                self.limit_reset_time = Some(next_reset);

                self.updated_at = now;
            }
        }
    }

    /// 取引が制限内かどうかを確認
    pub fn is_within_limit(&self, amount: u64) -> bool {
        if let Some(remaining) = self.remaining_daily_limit {
            amount <= remaining
        } else {
            true // 制限がない場合は常に許可
        }
    }

    /// ウォレットの概要を取得
    pub fn get_summary(&self) -> String {
        let status = match self.status {
            WalletStatus::Active => "アクティブ",
            WalletStatus::Frozen => "凍結",
            WalletStatus::Disabled => "無効",
            WalletStatus::Archived => "アーカイブ",
        };

        let daily_limit = if let Some(limit) = self.daily_limit {
            format!("{}", limit)
        } else {
            "なし".to_string()
        };

        let description = if let Some(desc) = &self.description {
            desc.clone()
        } else {
            "なし".to_string()
        };

        let tags = if self.tags.is_empty() {
            "なし".to_string()
        } else {
            self.tags.join(", ")
        };

        format!(
            "ID: {}\n名前: {}\n残高: {}\n状態: {}\n説明: {}\nタグ: {}\n日次制限: {}\n作成日時: {}\n更新日時: {}",
            self.id,
            self.name,
            self.balance,
            status,
            description,
            tags,
            daily_limit,
            self.created_at.format("%Y-%m-%d %H:%M:%S"),
            self.updated_at.format("%Y-%m-%d %H:%M:%S")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_keypair;

    #[test]
    fn test_multisig_wallet() {
        // キーペアを生成
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let keypair3 = generate_keypair();

        // 2-of-3ポリシーを作成
        let policy = ThresholdPolicy::new(
            2,
            vec![
                keypair1.public.clone(),
                keypair2.public.clone(),
                keypair3.public.clone(),
            ],
        );

        // ウォレットを作成
        let mut wallet = EnhancedMultisigWallet::new("テストウォレット".to_string(), policy);

        // 初期状態を確認
        assert_eq!(wallet.balance, 0);
        assert_eq!(wallet.status, WalletStatus::Active);
        assert!(wallet.is_active());

        // 残高を更新
        wallet.update_balance(1000);
        assert_eq!(wallet.balance, 1000);

        // 残高を増加
        wallet.increase_balance(500);
        assert_eq!(wallet.balance, 1500);

        // 残高を減少
        wallet.decrease_balance(300).unwrap();
        assert_eq!(wallet.balance, 1200);

        // 残高不足のテスト
        let result = wallet.decrease_balance(2000);
        assert!(result.is_err());
        assert_eq!(wallet.balance, 1200); // 残高は変わらない

        // メタデータを設定
        wallet.set_metadata("purpose", "プロジェクト資金");
        wallet.set_metadata("owner", "財務部");

        // メタデータを取得
        assert_eq!(
            wallet.get_metadata("purpose"),
            Some(&"プロジェクト資金".to_string())
        );
        assert_eq!(wallet.get_metadata("owner"), Some(&"財務部".to_string()));

        // 説明を設定
        wallet.set_description("プロジェクトXの資金管理用ウォレット");
        assert_eq!(
            wallet.description,
            Some("プロジェクトXの資金管理用ウォレット".to_string())
        );

        // タグを追加
        wallet.add_tag("プロジェクト");
        wallet.add_tag("財務");
        assert_eq!(
            wallet.tags,
            vec!["プロジェクト".to_string(), "財務".to_string()]
        );

        // タグを削除
        wallet.remove_tag("財務");
        assert_eq!(wallet.tags, vec!["プロジェクト".to_string()]);

        // ウォレットを凍結
        wallet.freeze();
        assert_eq!(wallet.status, WalletStatus::Frozen);
        assert!(!wallet.is_active());

        // ウォレットの凍結を解除
        wallet.unfreeze();
        assert_eq!(wallet.status, WalletStatus::Active);
        assert!(wallet.is_active());
    }

    #[test]
    fn test_daily_limit() {
        // キーペアを生成
        let keypair1 = generate_keypair();

        // ポリシーを作成
        let policy = ThresholdPolicy::new(1, vec![keypair1.public.clone()]);

        // ウォレットを作成
        let mut wallet = EnhancedMultisigWallet::new("制限付きウォレット".to_string(), policy);

        // 残高を設定
        wallet.update_balance(10000);

        // 日次制限を設定
        wallet.set_daily_limit(1000);
        assert_eq!(wallet.daily_limit, Some(1000));
        assert_eq!(wallet.remaining_daily_limit, Some(1000));
        assert!(wallet.limit_reset_time.is_some());

        // 制限内の取引
        assert!(wallet.is_within_limit(500));
        wallet.decrease_balance(500).unwrap();
        assert_eq!(wallet.remaining_daily_limit, Some(500));

        // 制限を超える取引
        assert!(!wallet.is_within_limit(600));
        let result = wallet.decrease_balance(600);
        assert!(result.is_err());

        // 制限内の取引（残り全て）
        assert!(wallet.is_within_limit(500));
        wallet.decrease_balance(500).unwrap();
        assert_eq!(wallet.remaining_daily_limit, Some(0));

        // 制限を超える取引（残高はあるが制限を超える）
        assert!(!wallet.is_within_limit(1));
        let result = wallet.decrease_balance(1);
        assert!(result.is_err());

        // 日次制限をリセット
        wallet.reset_daily_limit_if_needed(); // 実際には時間経過が必要

        // 日次制限を解除
        wallet.remove_daily_limit();
        assert_eq!(wallet.daily_limit, None);
        assert_eq!(wallet.remaining_daily_limit, None);
        assert_eq!(wallet.limit_reset_time, None);

        // 制限なしの取引
        assert!(wallet.is_within_limit(1000));
        wallet.decrease_balance(1000).unwrap();
        assert_eq!(wallet.balance, 8000);
    }
}
