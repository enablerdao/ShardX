use chrono::{DateTime, Duration, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::crypto::{hash, KeyPair, PrivateKey, PublicKey, Signature};
use crate::error::Error;
use crate::transaction::{Transaction, TransactionStatus};
use crate::wallet::multisig::config::{
    ApprovalHierarchy, ApprovalLevel, AutoApprovalRule, MultisigConfig, NotificationSettings,
    RejectionRule,
};
use crate::wallet::multisig::enhanced_transaction::{
    EnhancedMultisigTransaction, MultisigTransactionState,
};
use crate::wallet::multisig::enhanced_wallet::{EnhancedMultisigWallet, WalletStatus};
use crate::wallet::multisig::threshold::ThresholdPolicy;
use crate::wallet::multisig::transaction::{
    MultisigTransaction, MultisigTransactionStatus, TransactionAction, TransactionHistoryEntry,
    TransactionStep, TransactionStepStatus,
};
use crate::wallet::WalletId;

/// 高度なマルチシグウォレット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedMultisigWallet {
    /// 基本ウォレット情報
    pub base_wallet: EnhancedMultisigWallet,
    /// 承認階層
    pub approval_hierarchy: ApprovalHierarchy,
    /// 自動承認ルール
    pub auto_approval_rules: Vec<AutoApprovalRule>,
    /// 拒否ルール
    pub rejection_rules: Vec<RejectionRule>,
    /// 通知設定
    pub notification_settings: NotificationSettings,
    /// 承認者の公開鍵
    pub approver_keys: HashMap<String, PublicKey>,
    /// 承認者のメタデータ
    pub approver_metadata: HashMap<String, HashMap<String, String>>,
    /// 承認者の権限レベル
    pub approver_levels: HashMap<String, ApprovalLevel>,
    /// 承認者のアクティブ状態
    pub approver_active: HashMap<String, bool>,
    /// 承認者の最終アクティビティ
    pub approver_last_activity: HashMap<String, DateTime<Utc>>,
    /// 承認者の追加履歴
    pub approver_history: Vec<ApproverHistoryEntry>,
    /// ウォレットの設定
    pub wallet_settings: AdvancedWalletSettings,
    /// ウォレットの統計
    pub wallet_stats: AdvancedWalletStats,
    /// ウォレットのセキュリティ設定
    pub security_settings: SecuritySettings,
    /// ウォレットの復旧設定
    pub recovery_settings: RecoverySettings,
}

/// 承認者履歴エントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproverHistoryEntry {
    /// エントリID
    pub id: String,
    /// 承認者ID
    pub approver_id: String,
    /// アクション
    pub action: ApproverAction,
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// 実行者
    pub executed_by: String,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 承認者アクション
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApproverAction {
    /// 追加
    Added,
    /// 削除
    Removed,
    /// 権限変更
    LevelChanged(ApprovalLevel, ApprovalLevel),
    /// 無効化
    Deactivated,
    /// 有効化
    Activated,
    /// メタデータ更新
    MetadataUpdated,
}

/// 高度なウォレット設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedWalletSettings {
    /// トランザクション有効期限（秒）
    pub transaction_expiry_seconds: u64,
    /// 自動拒否の有効化
    pub enable_auto_rejection: bool,
    /// 自動拒否の期限（秒）
    pub auto_rejection_seconds: u64,
    /// 承認者の最小数
    pub min_approvers: usize,
    /// 承認者の最大数
    pub max_approvers: usize,
    /// 承認階層の最大数
    pub max_approval_levels: usize,
    /// 承認タイムアウトの有効化
    pub enable_approval_timeout: bool,
    /// 承認タイムアウト（秒）
    pub approval_timeout_seconds: u64,
    /// 承認リマインダーの有効化
    pub enable_approval_reminders: bool,
    /// 承認リマインダーの間隔（秒）
    pub approval_reminder_interval_seconds: u64,
    /// 取引履歴の保持期間（日）
    pub transaction_history_retention_days: u64,
    /// ガスリミットの自動調整
    pub auto_adjust_gas_limit: bool,
    /// 最大ガス価格
    pub max_gas_price: Option<u64>,
    /// 取引の優先度
    pub transaction_priority: TransactionPriority,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 取引の優先度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionPriority {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// カスタム
    Custom(u64),
}

/// 高度なウォレット統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedWalletStats {
    /// 総トランザクション数
    pub total_transactions: usize,
    /// 承認済みトランザクション数
    pub approved_transactions: usize,
    /// 拒否されたトランザクション数
    pub rejected_transactions: usize,
    /// 保留中のトランザクション数
    pub pending_transactions: usize,
    /// 期限切れのトランザクション数
    pub expired_transactions: usize,
    /// 総取引量
    pub total_volume: u64,
    /// 平均承認時間（秒）
    pub average_approval_time_seconds: f64,
    /// 平均承認者数
    pub average_approver_count: f64,
    /// 最も活発な承認者
    pub most_active_approver: Option<String>,
    /// 最も遅い承認者
    pub slowest_approver: Option<String>,
    /// 最後のトランザクション時刻
    pub last_transaction_time: Option<DateTime<Utc>>,
    /// 最大取引額
    pub max_transaction_amount: u64,
    /// 最小取引額
    pub min_transaction_amount: u64,
    /// 日次取引量の履歴
    pub daily_volume_history: HashMap<DateTime<Utc>, u64>,
    /// 月次取引量の履歴
    pub monthly_volume_history: HashMap<DateTime<Utc>, u64>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// セキュリティ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    /// 2要素認証の有効化
    pub enable_2fa: bool,
    /// 2要素認証の方法
    pub two_factor_method: TwoFactorMethod,
    /// IPホワイトリスト
    pub ip_whitelist: Option<Vec<String>>,
    /// 最大試行回数
    pub max_attempts: u32,
    /// ロックアウト期間（秒）
    pub lockout_period_seconds: u64,
    /// セッションタイムアウト（秒）
    pub session_timeout_seconds: u64,
    /// 承認デバイスの制限
    pub restrict_approval_devices: bool,
    /// 承認デバイスリスト
    pub approved_devices: Option<Vec<DeviceInfo>>,
    /// 地理的制限の有効化
    pub enable_geo_restrictions: bool,
    /// 許可された国コード
    pub allowed_country_codes: Option<Vec<String>>,
    /// 高度なセキュリティログの有効化
    pub enable_advanced_security_logging: bool,
    /// セキュリティ通知の有効化
    pub enable_security_notifications: bool,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 2要素認証の方法
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TwoFactorMethod {
    /// なし
    None,
    /// SMS
    SMS,
    /// Eメール
    Email,
    /// アプリ
    App,
    /// ハードウェアキー
    HardwareKey,
    /// 複数
    Multiple(Vec<String>),
}

/// デバイス情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// デバイスID
    pub device_id: String,
    /// デバイス名
    pub device_name: String,
    /// デバイスタイプ
    pub device_type: String,
    /// 最終アクセス時刻
    pub last_access: DateTime<Utc>,
    /// IPアドレス
    pub ip_address: Option<String>,
    /// ユーザーエージェント
    pub user_agent: Option<String>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 復旧設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoverySettings {
    /// 復旧の有効化
    pub enable_recovery: bool,
    /// 復旧方法
    pub recovery_method: RecoveryMethod,
    /// 復旧の閾値
    pub recovery_threshold: usize,
    /// 復旧の遅延（秒）
    pub recovery_delay_seconds: u64,
    /// 復旧の承認者
    pub recovery_approvers: Vec<String>,
    /// 復旧のメタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 復旧方法
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecoveryMethod {
    /// 社会的復旧
    Social,
    /// シードフレーズ
    SeedPhrase,
    /// ハードウェアバックアップ
    HardwareBackup,
    /// タイムロック
    TimeLock,
    /// カスタム
    Custom(String),
}

/// 高度なマルチシグトランザクション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedMultisigTransaction {
    /// 基本トランザクション情報
    pub base_transaction: EnhancedMultisigTransaction,
    /// 承認階層の進捗
    pub hierarchy_progress: HashMap<ApprovalLevel, usize>,
    /// 承認者の署名
    pub approver_signatures: HashMap<String, Signature>,
    /// 承認者のコメント
    pub approver_comments: HashMap<String, String>,
    /// 承認者の承認時刻
    pub approver_timestamps: HashMap<String, DateTime<Utc>>,
    /// 承認者の承認デバイス
    pub approver_devices: HashMap<String, DeviceInfo>,
    /// 承認リマインダーの送信履歴
    pub reminder_history: Vec<ReminderHistoryEntry>,
    /// 自動承認ルールの適用結果
    pub auto_approval_results: Vec<AutoApprovalResult>,
    /// 拒否ルールの適用結果
    pub rejection_rule_results: Vec<RejectionRuleResult>,
    /// トランザクションの優先度
    pub priority: TransactionPriority,
    /// ガス価格
    pub gas_price: Option<u64>,
    /// ガスリミット
    pub gas_limit: Option<u64>,
    /// 有効期限
    pub expiry_time: DateTime<Utc>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// リマインダー履歴エントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderHistoryEntry {
    /// エントリID
    pub id: String,
    /// 送信時刻
    pub sent_at: DateTime<Utc>,
    /// 送信先
    pub sent_to: Vec<String>,
    /// 送信方法
    pub sent_via: String,
    /// 送信ステータス
    pub status: ReminderStatus,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// リマインダーステータス
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReminderStatus {
    /// 送信済み
    Sent,
    /// 配信済み
    Delivered,
    /// 既読
    Read,
    /// 失敗
    Failed,
}

/// 自動承認結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoApprovalResult {
    /// ルールID
    pub rule_id: String,
    /// 適用結果
    pub applied: bool,
    /// 適用時刻
    pub applied_at: DateTime<Utc>,
    /// 結果の説明
    pub description: String,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 拒否ルール結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectionRuleResult {
    /// ルールID
    pub rule_id: String,
    /// 適用結果
    pub applied: bool,
    /// 適用時刻
    pub applied_at: DateTime<Utc>,
    /// 結果の説明
    pub description: String,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 高度なマルチシグウォレットマネージャー
pub struct AdvancedMultisigManager {
    /// ウォレットのマップ
    wallets: HashMap<WalletId, AdvancedMultisigWallet>,
    /// トランザクションのマップ
    transactions: HashMap<String, AdvancedMultisigTransaction>,
    /// 承認者のマップ
    approvers: HashMap<String, ApproverInfo>,
    /// ウォレットのインデックス
    wallet_indices: HashMap<String, Vec<WalletId>>,
}

/// 承認者情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproverInfo {
    /// 承認者ID
    pub id: String,
    /// 承認者名
    pub name: String,
    /// 公開鍵
    pub public_key: PublicKey,
    /// メタデータ
    pub metadata: HashMap<String, String>,
    /// 関連するウォレット
    pub associated_wallets: Vec<WalletId>,
    /// 最終アクティビティ
    pub last_activity: DateTime<Utc>,
    /// アクティブ状態
    pub is_active: bool,
    /// 承認デバイス
    pub approved_devices: Vec<DeviceInfo>,
    /// 連絡先情報
    pub contact_info: ContactInfo,
}

/// 連絡先情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    /// Eメール
    pub email: Option<String>,
    /// 電話番号
    pub phone: Option<String>,
    /// 通知設定
    pub notification_preferences: HashMap<String, bool>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

impl AdvancedMultisigManager {
    /// 新しい高度なマルチシグウォレットマネージャーを作成
    pub fn new() -> Self {
        Self {
            wallets: HashMap::new(),
            transactions: HashMap::new(),
            approvers: HashMap::new(),
            wallet_indices: HashMap::new(),
        }
    }

    /// ウォレットを作成
    pub fn create_wallet(
        &mut self,
        name: String,
        policy: ThresholdPolicy,
        approval_hierarchy: ApprovalHierarchy,
        initial_approvers: HashMap<String, PublicKey>,
        approver_levels: HashMap<String, ApprovalLevel>,
        wallet_settings: AdvancedWalletSettings,
        security_settings: SecuritySettings,
        recovery_settings: RecoverySettings,
    ) -> Result<WalletId, Error> {
        // ウォレットIDを生成
        let wallet_id = WalletId::new();

        // 基本ウォレット情報を作成
        let now = Utc::now();
        let base_wallet = EnhancedMultisigWallet {
            id: wallet_id.clone(),
            name: name.clone(),
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
        };

        // 承認者の履歴を作成
        let mut approver_history = Vec::new();
        for (approver_id, _) in &initial_approvers {
            let entry = ApproverHistoryEntry {
                id: format!("hist-{}-{}", approver_id, now.timestamp()),
                approver_id: approver_id.clone(),
                action: ApproverAction::Added,
                timestamp: now,
                executed_by: "system".to_string(),
                metadata: None,
            };
            approver_history.push(entry);

            // 承認者情報を更新
            if let Some(approver) = self.approvers.get_mut(approver_id) {
                approver.associated_wallets.push(wallet_id.clone());
                approver.last_activity = now;
            } else {
                // 新しい承認者を作成
                let approver_info = ApproverInfo {
                    id: approver_id.clone(),
                    name: format!("Approver {}", approver_id),
                    public_key: initial_approvers[approver_id].clone(),
                    metadata: HashMap::new(),
                    associated_wallets: vec![wallet_id.clone()],
                    last_activity: now,
                    is_active: true,
                    approved_devices: Vec::new(),
                    contact_info: ContactInfo {
                        email: None,
                        phone: None,
                        notification_preferences: HashMap::new(),
                        metadata: None,
                    },
                };
                self.approvers.insert(approver_id.clone(), approver_info);
            }
        }

        // ウォレット統計を初期化
        let wallet_stats = AdvancedWalletStats {
            total_transactions: 0,
            approved_transactions: 0,
            rejected_transactions: 0,
            pending_transactions: 0,
            expired_transactions: 0,
            total_volume: 0,
            average_approval_time_seconds: 0.0,
            average_approver_count: 0.0,
            most_active_approver: None,
            slowest_approver: None,
            last_transaction_time: None,
            max_transaction_amount: 0,
            min_transaction_amount: 0,
            daily_volume_history: HashMap::new(),
            monthly_volume_history: HashMap::new(),
            metadata: None,
        };

        // 高度なウォレットを作成
        let wallet = AdvancedMultisigWallet {
            base_wallet,
            approval_hierarchy,
            auto_approval_rules: Vec::new(),
            rejection_rules: Vec::new(),
            notification_settings: NotificationSettings {
                enabled: true,
                destinations: Vec::new(),
                notification_types: Vec::new(),
            },
            approver_keys: initial_approvers,
            approver_metadata: HashMap::new(),
            approver_levels,
            approver_active: initial_approvers
                .keys()
                .map(|k| (k.clone(), true))
                .collect(),
            approver_last_activity: initial_approvers.keys().map(|k| (k.clone(), now)).collect(),
            approver_history,
            wallet_settings,
            wallet_stats,
            security_settings,
            recovery_settings,
        };

        // ウォレットを保存
        self.wallets.insert(wallet_id.clone(), wallet);

        // インデックスを更新
        let name_key = name.to_lowercase();
        let wallets = self.wallet_indices.entry(name_key).or_insert_with(Vec::new);
        wallets.push(wallet_id.clone());

        Ok(wallet_id)
    }

    /// ウォレットを取得
    pub fn get_wallet(&self, wallet_id: &WalletId) -> Option<&AdvancedMultisigWallet> {
        self.wallets.get(wallet_id)
    }

    /// ウォレットを更新
    pub fn update_wallet(&mut self, wallet: AdvancedMultisigWallet) -> Result<(), Error> {
        let wallet_id = wallet.base_wallet.id.clone();

        if !self.wallets.contains_key(&wallet_id) {
            return Err(Error::NotFound(format!(
                "Wallet with ID {} not found",
                wallet_id
            )));
        }

        // ウォレットを更新
        self.wallets.insert(wallet_id, wallet);

        Ok(())
    }

    /// トランザクションを作成
    pub fn create_transaction(
        &mut self,
        wallet_id: &WalletId,
        amount: u64,
        recipient: String,
        initiator: String,
        metadata: Option<HashMap<String, String>>,
        priority: TransactionPriority,
        gas_price: Option<u64>,
        gas_limit: Option<u64>,
    ) -> Result<String, Error> {
        let wallet = self
            .wallets
            .get_mut(wallet_id)
            .ok_or_else(|| Error::NotFound(format!("Wallet with ID {} not found", wallet_id)))?;

        // ウォレットの状態を確認
        if wallet.base_wallet.status != WalletStatus::Active {
            return Err(Error::InvalidState(format!(
                "Wallet is not active: {:?}",
                wallet.base_wallet.status
            )));
        }

        // 残高を確認
        if wallet.base_wallet.balance < amount {
            return Err(Error::InsufficientFunds(format!(
                "Insufficient funds: {} < {}",
                wallet.base_wallet.balance, amount
            )));
        }

        // 日次制限を確認
        if let Some(daily_limit) = wallet.base_wallet.daily_limit {
            if let Some(remaining) = wallet.base_wallet.remaining_daily_limit {
                if amount > remaining {
                    return Err(Error::LimitExceeded(format!(
                        "Daily limit exceeded: {} > {}",
                        amount, remaining
                    )));
                }
            }
        }

        // トランザクションIDを生成
        let now = Utc::now();
        let tx_id = format!("tx-{}-{}", wallet_id, now.timestamp());

        // 基本トランザクション情報を作成
        let base_transaction = EnhancedMultisigTransaction {
            id: tx_id.clone(),
            wallet_id: wallet_id.clone(),
            amount,
            recipient,
            initiator: initiator.clone(),
            created_at: now,
            updated_at: now,
            status: MultisigTransactionStatus::Pending,
            state: MultisigTransactionState::AwaitingApprovals,
            approvals: HashMap::new(),
            rejections: HashMap::new(),
            executed_at: None,
            execution_transaction_id: None,
            metadata: metadata.clone(),
            required_approvals: wallet.base_wallet.policy.threshold,
            expiry_time: now
                + Duration::seconds(wallet.wallet_settings.transaction_expiry_seconds as i64),
        };

        // 高度なトランザクションを作成
        let transaction = AdvancedMultisigTransaction {
            base_transaction,
            hierarchy_progress: HashMap::new(),
            approver_signatures: HashMap::new(),
            approver_comments: HashMap::new(),
            approver_timestamps: HashMap::new(),
            approver_devices: HashMap::new(),
            reminder_history: Vec::new(),
            auto_approval_results: Vec::new(),
            rejection_rule_results: Vec::new(),
            priority,
            gas_price,
            gas_limit,
            expiry_time: now
                + Duration::seconds(wallet.wallet_settings.transaction_expiry_seconds as i64),
            metadata,
        };

        // トランザクションを保存
        self.transactions.insert(tx_id.clone(), transaction);

        // ウォレット統計を更新
        wallet.wallet_stats.pending_transactions += 1;
        wallet.wallet_stats.total_transactions += 1;
        wallet.wallet_stats.last_transaction_time = Some(now);

        // 日次制限を更新
        if let Some(daily_limit) = wallet.base_wallet.daily_limit {
            if let Some(remaining) = wallet.base_wallet.remaining_daily_limit.as_mut() {
                *remaining = remaining.saturating_sub(amount);
            }
        }

        // 自動承認ルールを適用
        self.apply_auto_approval_rules(&tx_id, initiator)?;

        // 拒否ルールを適用
        self.apply_rejection_rules(&tx_id)?;

        Ok(tx_id)
    }

    /// トランザクションを承認
    pub fn approve_transaction(
        &mut self,
        tx_id: &str,
        approver_id: &str,
        signature: Signature,
        device_info: Option<DeviceInfo>,
        comment: Option<String>,
    ) -> Result<(), Error> {
        let transaction = self
            .transactions
            .get_mut(tx_id)
            .ok_or_else(|| Error::NotFound(format!("Transaction with ID {} not found", tx_id)))?;

        let wallet_id = transaction.base_transaction.wallet_id.clone();
        let wallet = self
            .wallets
            .get_mut(&wallet_id)
            .ok_or_else(|| Error::NotFound(format!("Wallet with ID {} not found", wallet_id)))?;

        // トランザクションの状態を確認
        if transaction.base_transaction.status != MultisigTransactionStatus::Pending {
            return Err(Error::InvalidState(format!(
                "Transaction is not pending: {:?}",
                transaction.base_transaction.status
            )));
        }

        // 承認者の権限を確認
        if !wallet.approver_keys.contains_key(approver_id) {
            return Err(Error::Unauthorized(format!(
                "Approver {} is not authorized for this wallet",
                approver_id
            )));
        }

        // 承認者のアクティブ状態を確認
        if let Some(active) = wallet.approver_active.get(approver_id) {
            if !active {
                return Err(Error::Unauthorized(format!(
                    "Approver {} is not active",
                    approver_id
                )));
            }
        }

        // 署名を検証
        let public_key = wallet.approver_keys.get(approver_id).unwrap();
        let message = format!("approve:{}:{}", tx_id, transaction.base_transaction.amount);
        if !public_key.verify(message.as_bytes(), &signature) {
            return Err(Error::InvalidSignature("Invalid signature".to_string()));
        }

        // 承認を記録
        let now = Utc::now();
        transaction
            .base_transaction
            .approvals
            .insert(approver_id.to_string(), now);
        transaction
            .approver_signatures
            .insert(approver_id.to_string(), signature);
        transaction.base_transaction.updated_at = now;

        if let Some(comment_text) = comment {
            transaction
                .approver_comments
                .insert(approver_id.to_string(), comment_text);
        }

        transaction
            .approver_timestamps
            .insert(approver_id.to_string(), now);

        if let Some(device) = device_info {
            transaction
                .approver_devices
                .insert(approver_id.to_string(), device);
        }

        // 承認者の最終アクティビティを更新
        wallet
            .approver_last_activity
            .insert(approver_id.to_string(), now);

        // 承認階層の進捗を更新
        if let Some(level) = wallet.approver_levels.get(approver_id) {
            let progress = transaction
                .hierarchy_progress
                .entry(level.clone())
                .or_insert(0);
            *progress += 1;
        }

        // 必要な承認数を確認
        let approval_count = transaction.base_transaction.approvals.len();
        let required_approvals = transaction.base_transaction.required_approvals;

        if approval_count >= required_approvals {
            // トランザクションを実行
            self.execute_transaction(tx_id)?;
        }

        Ok(())
    }

    /// トランザクションを拒否
    pub fn reject_transaction(
        &mut self,
        tx_id: &str,
        approver_id: &str,
        signature: Signature,
        device_info: Option<DeviceInfo>,
        comment: Option<String>,
    ) -> Result<(), Error> {
        let transaction = self
            .transactions
            .get_mut(tx_id)
            .ok_or_else(|| Error::NotFound(format!("Transaction with ID {} not found", tx_id)))?;

        let wallet_id = transaction.base_transaction.wallet_id.clone();
        let wallet = self
            .wallets
            .get_mut(&wallet_id)
            .ok_or_else(|| Error::NotFound(format!("Wallet with ID {} not found", wallet_id)))?;

        // トランザクションの状態を確認
        if transaction.base_transaction.status != MultisigTransactionStatus::Pending {
            return Err(Error::InvalidState(format!(
                "Transaction is not pending: {:?}",
                transaction.base_transaction.status
            )));
        }

        // 承認者の権限を確認
        if !wallet.approver_keys.contains_key(approver_id) {
            return Err(Error::Unauthorized(format!(
                "Approver {} is not authorized for this wallet",
                approver_id
            )));
        }

        // 承認者のアクティブ状態を確認
        if let Some(active) = wallet.approver_active.get(approver_id) {
            if !active {
                return Err(Error::Unauthorized(format!(
                    "Approver {} is not active",
                    approver_id
                )));
            }
        }

        // 署名を検証
        let public_key = wallet.approver_keys.get(approver_id).unwrap();
        let message = format!("reject:{}:{}", tx_id, transaction.base_transaction.amount);
        if !public_key.verify(message.as_bytes(), &signature) {
            return Err(Error::InvalidSignature("Invalid signature".to_string()));
        }

        // 拒否を記録
        let now = Utc::now();
        transaction
            .base_transaction
            .rejections
            .insert(approver_id.to_string(), now);
        transaction
            .approver_signatures
            .insert(approver_id.to_string(), signature);
        transaction.base_transaction.updated_at = now;

        if let Some(comment_text) = comment {
            transaction
                .approver_comments
                .insert(approver_id.to_string(), comment_text);
        }

        transaction
            .approver_timestamps
            .insert(approver_id.to_string(), now);

        if let Some(device) = device_info {
            transaction
                .approver_devices
                .insert(approver_id.to_string(), device);
        }

        // 承認者の最終アクティビティを更新
        wallet
            .approver_last_activity
            .insert(approver_id.to_string(), now);

        // トランザクションを拒否
        transaction.base_transaction.status = MultisigTransactionStatus::Rejected;
        transaction.base_transaction.state = MultisigTransactionState::Rejected;

        // ウォレット統計を更新
        wallet.wallet_stats.rejected_transactions += 1;
        wallet.wallet_stats.pending_transactions -= 1;

        Ok(())
    }

    /// トランザクションを実行
    fn execute_transaction(&mut self, tx_id: &str) -> Result<(), Error> {
        let transaction = self
            .transactions
            .get_mut(tx_id)
            .ok_or_else(|| Error::NotFound(format!("Transaction with ID {} not found", tx_id)))?;

        let wallet_id = transaction.base_transaction.wallet_id.clone();
        let wallet = self
            .wallets
            .get_mut(&wallet_id)
            .ok_or_else(|| Error::NotFound(format!("Wallet with ID {} not found", wallet_id)))?;

        // トランザクションの状態を確認
        if transaction.base_transaction.status != MultisigTransactionStatus::Pending {
            return Err(Error::InvalidState(format!(
                "Transaction is not pending: {:?}",
                transaction.base_transaction.status
            )));
        }

        // 残高を確認
        let amount = transaction.base_transaction.amount;
        if wallet.base_wallet.balance < amount {
            return Err(Error::InsufficientFunds(format!(
                "Insufficient funds: {} < {}",
                wallet.base_wallet.balance, amount
            )));
        }

        // トランザクションを実行
        let now = Utc::now();
        transaction.base_transaction.status = MultisigTransactionStatus::Executed;
        transaction.base_transaction.state = MultisigTransactionState::Executed;
        transaction.base_transaction.executed_at = Some(now);
        transaction.base_transaction.updated_at = now;

        // 実行トランザクションIDを生成
        let execution_tx_id = format!("exec-{}-{}", tx_id, now.timestamp());
        transaction.base_transaction.execution_transaction_id = Some(execution_tx_id);

        // ウォレットの残高を更新
        wallet.base_wallet.balance -= amount;
        wallet.base_wallet.updated_at = now;

        // ウォレット統計を更新
        wallet.wallet_stats.approved_transactions += 1;
        wallet.wallet_stats.pending_transactions -= 1;
        wallet.wallet_stats.total_volume += amount;

        // 平均承認時間を更新
        let approval_time = (now - transaction.base_transaction.created_at).num_seconds() as f64;
        let total_approved = wallet.wallet_stats.approved_transactions as f64;
        wallet.wallet_stats.average_approval_time_seconds =
            (wallet.wallet_stats.average_approval_time_seconds * (total_approved - 1.0)
                + approval_time)
                / total_approved;

        // 平均承認者数を更新
        let approver_count = transaction.base_transaction.approvals.len() as f64;
        wallet.wallet_stats.average_approver_count =
            (wallet.wallet_stats.average_approver_count * (total_approved - 1.0) + approver_count)
                / total_approved;

        // 最大・最小取引額を更新
        wallet.wallet_stats.max_transaction_amount =
            wallet.wallet_stats.max_transaction_amount.max(amount);
        if wallet.wallet_stats.min_transaction_amount == 0 {
            wallet.wallet_stats.min_transaction_amount = amount;
        } else {
            wallet.wallet_stats.min_transaction_amount =
                wallet.wallet_stats.min_transaction_amount.min(amount);
        }

        // 日次・月次取引量を更新
        let day_key =
            DateTime::<Utc>::from_utc(now.naive_utc().date().and_hms_opt(0, 0, 0).unwrap(), Utc);
        let month_key = DateTime::<Utc>::from_utc(
            now.naive_utc()
                .with_day(1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            Utc,
        );

        let day_volume = wallet
            .wallet_stats
            .daily_volume_history
            .entry(day_key)
            .or_insert(0);
        *day_volume += amount;

        let month_volume = wallet
            .wallet_stats
            .monthly_volume_history
            .entry(month_key)
            .or_insert(0);
        *month_volume += amount;

        Ok(())
    }

    /// 自動承認ルールを適用
    fn apply_auto_approval_rules(&mut self, tx_id: &str, initiator: String) -> Result<(), Error> {
        let transaction = self
            .transactions
            .get(tx_id)
            .ok_or_else(|| Error::NotFound(format!("Transaction with ID {} not found", tx_id)))?;

        let wallet_id = transaction.base_transaction.wallet_id.clone();
        let wallet = self
            .wallets
            .get(&wallet_id)
            .ok_or_else(|| Error::NotFound(format!("Wallet with ID {} not found", wallet_id)))?;

        // 自動承認ルールを適用
        let mut auto_approved = false;
        let now = Utc::now();

        for rule in &wallet.auto_approval_rules {
            // ルールの条件を確認
            let mut rule_applies = false;

            match rule {
                AutoApprovalRule::AmountBelow(threshold) => {
                    if transaction.base_transaction.amount < *threshold {
                        rule_applies = true;
                    }
                }
                AutoApprovalRule::RecipientWhitelisted(recipients) => {
                    if recipients.contains(&transaction.base_transaction.recipient) {
                        rule_applies = true;
                    }
                }
                AutoApprovalRule::InitiatorTrusted(initiators) => {
                    if initiators.contains(&initiator) {
                        rule_applies = true;
                    }
                }
                _ => {}
            }

            if rule_applies {
                // トランザクションを自動承認
                let mut transaction = self.transactions.get_mut(tx_id).unwrap();
                let result = AutoApprovalResult {
                    rule_id: format!("rule-{}", now.timestamp()),
                    applied: true,
                    applied_at: now,
                    description: format!("Auto-approved by rule: {:?}", rule),
                    metadata: None,
                };
                transaction.auto_approval_results.push(result);

                // システム承認を追加
                transaction
                    .base_transaction
                    .approvals
                    .insert("system".to_string(), now);
                auto_approved = true;
            }
        }

        // 自動承認された場合、必要な承認数を確認
        if auto_approved {
            let transaction = self.transactions.get(tx_id).unwrap();
            let approval_count = transaction.base_transaction.approvals.len();
            let required_approvals = transaction.base_transaction.required_approvals;

            if approval_count >= required_approvals {
                // トランザクションを実行
                self.execute_transaction(tx_id)?;
            }
        }

        Ok(())
    }

    /// 拒否ルールを適用
    fn apply_rejection_rules(&mut self, tx_id: &str) -> Result<(), Error> {
        let transaction = self
            .transactions
            .get(tx_id)
            .ok_or_else(|| Error::NotFound(format!("Transaction with ID {} not found", tx_id)))?;

        let wallet_id = transaction.base_transaction.wallet_id.clone();
        let wallet = self
            .wallets
            .get(&wallet_id)
            .ok_or_else(|| Error::NotFound(format!("Wallet with ID {} not found", wallet_id)))?;

        // 拒否ルールを適用
        let mut auto_rejected = false;
        let now = Utc::now();

        for rule in &wallet.rejection_rules {
            // ルールの条件を確認
            let mut rule_applies = false;

            match rule {
                RejectionRule::AmountAbove(threshold) => {
                    if transaction.base_transaction.amount > *threshold {
                        rule_applies = true;
                    }
                }
                RejectionRule::RecipientBlacklisted(recipients) => {
                    if recipients.contains(&transaction.base_transaction.recipient) {
                        rule_applies = true;
                    }
                }
                _ => {}
            }

            if rule_applies {
                // トランザクションを自動拒否
                let mut transaction = self.transactions.get_mut(tx_id).unwrap();
                let result = RejectionRuleResult {
                    rule_id: format!("rule-{}", now.timestamp()),
                    applied: true,
                    applied_at: now,
                    description: format!("Auto-rejected by rule: {:?}", rule),
                    metadata: None,
                };
                transaction.rejection_rule_results.push(result);

                // トランザクションを拒否
                transaction.base_transaction.status = MultisigTransactionStatus::Rejected;
                transaction.base_transaction.state = MultisigTransactionState::Rejected;
                transaction.base_transaction.updated_at = now;

                // ウォレット統計を更新
                let wallet = self.wallets.get_mut(&wallet_id).unwrap();
                wallet.wallet_stats.rejected_transactions += 1;
                wallet.wallet_stats.pending_transactions -= 1;

                auto_rejected = true;
                break;
            }
        }

        Ok(())
    }

    /// 承認者を追加
    pub fn add_approver(
        &mut self,
        wallet_id: &WalletId,
        approver_id: String,
        public_key: PublicKey,
        level: ApprovalLevel,
        metadata: Option<HashMap<String, String>>,
        executed_by: String,
    ) -> Result<(), Error> {
        let wallet = self
            .wallets
            .get_mut(wallet_id)
            .ok_or_else(|| Error::NotFound(format!("Wallet with ID {} not found", wallet_id)))?;

        // 承認者の最大数を確認
        if wallet.approver_keys.len() >= wallet.wallet_settings.max_approvers {
            return Err(Error::LimitExceeded(format!(
                "Maximum number of approvers reached: {}",
                wallet.wallet_settings.max_approvers
            )));
        }

        // 承認者を追加
        let now = Utc::now();
        wallet
            .approver_keys
            .insert(approver_id.clone(), public_key.clone());
        wallet.approver_levels.insert(approver_id.clone(), level);
        wallet.approver_active.insert(approver_id.clone(), true);
        wallet
            .approver_last_activity
            .insert(approver_id.clone(), now);

        if let Some(meta) = metadata {
            wallet.approver_metadata.insert(approver_id.clone(), meta);
        }

        // 承認者の履歴を更新
        let entry = ApproverHistoryEntry {
            id: format!("hist-{}-{}", approver_id, now.timestamp()),
            approver_id: approver_id.clone(),
            action: ApproverAction::Added,
            timestamp: now,
            executed_by,
            metadata: None,
        };
        wallet.approver_history.push(entry);

        // 承認者情報を更新
        if let Some(approver) = self.approvers.get_mut(&approver_id) {
            approver.associated_wallets.push(wallet_id.clone());
            approver.last_activity = now;
        } else {
            // 新しい承認者を作成
            let approver_info = ApproverInfo {
                id: approver_id.clone(),
                name: format!("Approver {}", approver_id),
                public_key,
                metadata: HashMap::new(),
                associated_wallets: vec![wallet_id.clone()],
                last_activity: now,
                is_active: true,
                approved_devices: Vec::new(),
                contact_info: ContactInfo {
                    email: None,
                    phone: None,
                    notification_preferences: HashMap::new(),
                    metadata: None,
                },
            };
            self.approvers.insert(approver_id, approver_info);
        }

        Ok(())
    }

    /// 承認者を削除
    pub fn remove_approver(
        &mut self,
        wallet_id: &WalletId,
        approver_id: &str,
        executed_by: String,
    ) -> Result<(), Error> {
        let wallet = self
            .wallets
            .get_mut(wallet_id)
            .ok_or_else(|| Error::NotFound(format!("Wallet with ID {} not found", wallet_id)))?;

        // 承認者の最小数を確認
        if wallet.approver_keys.len() <= wallet.wallet_settings.min_approvers {
            return Err(Error::InvalidOperation(format!(
                "Minimum number of approvers reached: {}",
                wallet.wallet_settings.min_approvers
            )));
        }

        // 承認者を削除
        if !wallet.approver_keys.contains_key(approver_id) {
            return Err(Error::NotFound(format!(
                "Approver with ID {} not found",
                approver_id
            )));
        }

        wallet.approver_keys.remove(approver_id);
        wallet.approver_levels.remove(approver_id);
        wallet.approver_active.remove(approver_id);
        wallet.approver_last_activity.remove(approver_id);
        wallet.approver_metadata.remove(approver_id);

        // 承認者の履歴を更新
        let now = Utc::now();
        let entry = ApproverHistoryEntry {
            id: format!("hist-{}-{}", approver_id, now.timestamp()),
            approver_id: approver_id.to_string(),
            action: ApproverAction::Removed,
            timestamp: now,
            executed_by,
            metadata: None,
        };
        wallet.approver_history.push(entry);

        // 承認者情報を更新
        if let Some(approver) = self.approvers.get_mut(approver_id) {
            approver.associated_wallets.retain(|id| id != wallet_id);
            approver.last_activity = now;
        }

        Ok(())
    }

    /// 承認者の権限を変更
    pub fn change_approver_level(
        &mut self,
        wallet_id: &WalletId,
        approver_id: &str,
        new_level: ApprovalLevel,
        executed_by: String,
    ) -> Result<(), Error> {
        let wallet = self
            .wallets
            .get_mut(wallet_id)
            .ok_or_else(|| Error::NotFound(format!("Wallet with ID {} not found", wallet_id)))?;

        // 承認者を確認
        if !wallet.approver_keys.contains_key(approver_id) {
            return Err(Error::NotFound(format!(
                "Approver with ID {} not found",
                approver_id
            )));
        }

        // 現在のレベルを取得
        let old_level = wallet
            .approver_levels
            .get(approver_id)
            .cloned()
            .unwrap_or(ApprovalLevel::Standard);

        // レベルを変更
        wallet
            .approver_levels
            .insert(approver_id.to_string(), new_level.clone());

        // 承認者の履歴を更新
        let now = Utc::now();
        let entry = ApproverHistoryEntry {
            id: format!("hist-{}-{}", approver_id, now.timestamp()),
            approver_id: approver_id.to_string(),
            action: ApproverAction::LevelChanged(old_level, new_level),
            timestamp: now,
            executed_by,
            metadata: None,
        };
        wallet.approver_history.push(entry);

        Ok(())
    }

    /// 承認者を無効化
    pub fn deactivate_approver(
        &mut self,
        wallet_id: &WalletId,
        approver_id: &str,
        executed_by: String,
    ) -> Result<(), Error> {
        let wallet = self
            .wallets
            .get_mut(wallet_id)
            .ok_or_else(|| Error::NotFound(format!("Wallet with ID {} not found", wallet_id)))?;

        // 承認者を確認
        if !wallet.approver_keys.contains_key(approver_id) {
            return Err(Error::NotFound(format!(
                "Approver with ID {} not found",
                approver_id
            )));
        }

        // 承認者を無効化
        wallet
            .approver_active
            .insert(approver_id.to_string(), false);

        // 承認者の履歴を更新
        let now = Utc::now();
        let entry = ApproverHistoryEntry {
            id: format!("hist-{}-{}", approver_id, now.timestamp()),
            approver_id: approver_id.to_string(),
            action: ApproverAction::Deactivated,
            timestamp: now,
            executed_by,
            metadata: None,
        };
        wallet.approver_history.push(entry);

        // 承認者情報を更新
        if let Some(approver) = self.approvers.get_mut(approver_id) {
            approver.is_active = false;
            approver.last_activity = now;
        }

        Ok(())
    }

    /// 承認者を有効化
    pub fn activate_approver(
        &mut self,
        wallet_id: &WalletId,
        approver_id: &str,
        executed_by: String,
    ) -> Result<(), Error> {
        let wallet = self
            .wallets
            .get_mut(wallet_id)
            .ok_or_else(|| Error::NotFound(format!("Wallet with ID {} not found", wallet_id)))?;

        // 承認者を確認
        if !wallet.approver_keys.contains_key(approver_id) {
            return Err(Error::NotFound(format!(
                "Approver with ID {} not found",
                approver_id
            )));
        }

        // 承認者を有効化
        wallet.approver_active.insert(approver_id.to_string(), true);

        // 承認者の履歴を更新
        let now = Utc::now();
        let entry = ApproverHistoryEntry {
            id: format!("hist-{}-{}", approver_id, now.timestamp()),
            approver_id: approver_id.to_string(),
            action: ApproverAction::Activated,
            timestamp: now,
            executed_by,
            metadata: None,
        };
        wallet.approver_history.push(entry);

        // 承認者情報を更新
        if let Some(approver) = self.approvers.get_mut(approver_id) {
            approver.is_active = true;
            approver.last_activity = now;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_wallet() {
        // マネージャーを作成
        let mut manager = AdvancedMultisigManager::new();

        // 承認者の公開鍵を作成
        let key_pair1 = KeyPair::generate();
        let key_pair2 = KeyPair::generate();
        let key_pair3 = KeyPair::generate();

        let mut approvers = HashMap::new();
        approvers.insert("approver1".to_string(), key_pair1.public_key());
        approvers.insert("approver2".to_string(), key_pair2.public_key());
        approvers.insert("approver3".to_string(), key_pair3.public_key());

        let mut approver_levels = HashMap::new();
        approver_levels.insert("approver1".to_string(), ApprovalLevel::Admin);
        approver_levels.insert("approver2".to_string(), ApprovalLevel::Standard);
        approver_levels.insert("approver3".to_string(), ApprovalLevel::Standard);

        // 閾値ポリシーを作成
        let policy = ThresholdPolicy {
            threshold: 2,
            timeout_seconds: 86400,
        };

        // 承認階層を作成
        let approval_hierarchy = ApprovalHierarchy {
            levels: vec![ApprovalLevel::Admin, ApprovalLevel::Standard],
            required_levels: vec![(ApprovalLevel::Admin, 1), (ApprovalLevel::Standard, 1)],
        };

        // ウォレット設定を作成
        let wallet_settings = AdvancedWalletSettings {
            transaction_expiry_seconds: 86400,
            enable_auto_rejection: true,
            auto_rejection_seconds: 86400,
            min_approvers: 2,
            max_approvers: 10,
            max_approval_levels: 3,
            enable_approval_timeout: true,
            approval_timeout_seconds: 3600,
            enable_approval_reminders: true,
            approval_reminder_interval_seconds: 3600,
            transaction_history_retention_days: 365,
            auto_adjust_gas_limit: true,
            max_gas_price: Some(100),
            transaction_priority: TransactionPriority::Medium,
            metadata: None,
        };

        // セキュリティ設定を作成
        let security_settings = SecuritySettings {
            enable_2fa: true,
            two_factor_method: TwoFactorMethod::App,
            ip_whitelist: None,
            max_attempts: 5,
            lockout_period_seconds: 300,
            session_timeout_seconds: 1800,
            restrict_approval_devices: false,
            approved_devices: None,
            enable_geo_restrictions: false,
            allowed_country_codes: None,
            enable_advanced_security_logging: true,
            enable_security_notifications: true,
            metadata: None,
        };

        // 復旧設定を作成
        let recovery_settings = RecoverySettings {
            enable_recovery: true,
            recovery_method: RecoveryMethod::Social,
            recovery_threshold: 2,
            recovery_delay_seconds: 86400,
            recovery_approvers: vec!["approver1".to_string(), "approver2".to_string()],
            metadata: None,
        };

        // ウォレットを作成
        let result = manager.create_wallet(
            "Test Wallet".to_string(),
            policy,
            approval_hierarchy,
            approvers,
            approver_levels,
            wallet_settings,
            security_settings,
            recovery_settings,
        );

        assert!(result.is_ok());
        let wallet_id = result.unwrap();

        // ウォレットを取得
        let wallet = manager.get_wallet(&wallet_id);
        assert!(wallet.is_some());

        let wallet = wallet.unwrap();
        assert_eq!(wallet.base_wallet.name, "Test Wallet");
        assert_eq!(wallet.base_wallet.policy.threshold, 2);
        assert_eq!(wallet.approver_keys.len(), 3);
        assert_eq!(wallet.approver_levels.len(), 3);
        assert_eq!(wallet.base_wallet.balance, 0);
    }
}
