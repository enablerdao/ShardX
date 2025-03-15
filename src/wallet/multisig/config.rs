use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::crypto::PublicKey;
use crate::transaction::TransactionType;

/// マルチシグウォレット設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigConfig {
    /// 必要な署名数（M）
    pub required_signatures: usize,
    /// 合計キー数（N）
    pub total_keys: usize,
    /// タイムロック（秒）
    pub timelock_seconds: Option<u64>,
    /// 1日あたりの最大送金額
    pub daily_limit: Option<f64>,
    /// 1回あたりの最大送金額
    pub transaction_limit: Option<f64>,
    /// 承認階層
    pub approval_hierarchy: Option<ApprovalHierarchy>,
    /// 自動承認ルール
    pub auto_approval_rules: Vec<AutoApprovalRule>,
    /// 拒否ルール
    pub rejection_rules: Vec<RejectionRule>,
    /// 通知設定
    pub notification_settings: NotificationSettings,
}

impl Default for MultisigConfig {
    fn default() -> Self {
        Self {
            required_signatures: 2,
            total_keys: 3,
            timelock_seconds: None,
            daily_limit: None,
            transaction_limit: None,
            approval_hierarchy: None,
            auto_approval_rules: Vec::new(),
            rejection_rules: Vec::new(),
            notification_settings: NotificationSettings::default(),
        }
    }
}

/// 承認階層
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalHierarchy {
    /// 階層レベル
    pub levels: Vec<ApprovalLevel>,
}

/// 承認レベル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalLevel {
    /// レベル名
    pub name: String,
    /// レベル（数値）
    pub level: u8,
    /// 必要な署名数
    pub required_signatures: usize,
    /// 所属するキー
    pub keys: Vec<PublicKey>,
    /// 承認可能な最大金額
    pub max_amount: Option<f64>,
    /// 説明
    pub description: Option<String>,
}

/// 自動承認ルール
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoApprovalRule {
    /// ルール名
    pub name: String,
    /// 送信先アドレス
    pub destination_address: Option<String>,
    /// 最大金額
    pub max_amount: Option<f64>,
    /// トランザクションタイプ
    pub transaction_type: Option<TransactionType>,
    /// 有効期限
    pub expiry: Option<DateTime<Utc>>,
    /// 説明
    pub description: Option<String>,
}

/// 拒否ルール
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectionRule {
    /// ルール名
    pub name: String,
    /// 送信先アドレス
    pub destination_address: Option<String>,
    /// 最小金額
    pub min_amount: Option<f64>,
    /// トランザクションタイプ
    pub transaction_type: Option<TransactionType>,
    /// 説明
    pub description: Option<String>,
}

/// 通知設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    /// トランザクション作成時の通知
    pub on_transaction_created: bool,
    /// 署名追加時の通知
    pub on_signature_added: bool,
    /// トランザクション実行時の通知
    pub on_transaction_executed: bool,
    /// トランザクション拒否時の通知
    pub on_transaction_rejected: bool,
    /// タイムロック解除時の通知
    pub on_timelock_released: bool,
    /// 通知先
    pub notification_destinations: Vec<NotificationDestination>,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            on_transaction_created: true,
            on_signature_added: true,
            on_transaction_executed: true,
            on_transaction_rejected: true,
            on_timelock_released: true,
            notification_destinations: Vec::new(),
        }
    }
}

/// 通知先
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationDestination {
    /// 通知タイプ
    pub notification_type: NotificationType,
    /// 送信先
    pub destination: String,
}

/// 通知タイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NotificationType {
    /// Eメール
    Email,
    /// Webhook
    Webhook,
    /// オンチェーンメッセージ
    OnChainMessage,
}