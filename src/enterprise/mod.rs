// エンタープライズモジュール
//
// このモジュールは、ShardXのエンタープライズ機能を提供します。
// 主な機能:
// - エンタープライズグレードのセキュリティ
// - コンプライアンスフレームワーク
// - 監査ログ
// - アクセス制御
// - SLA管理

pub mod security;
pub mod compliance;
pub mod audit;
// pub mod access_control; // TODO: このモジュールが見つかりません
pub mod sla;

pub use self::security::{EnterpriseSecurityManager, SecurityPolicy, SecurityControl, SecurityAudit};
pub use self::compliance::{ComplianceManager, ComplianceFramework, ComplianceRequirement, ComplianceReport};
pub use self::audit::{AuditManager, AuditLog, AuditEvent, AuditTrail};
pub use self::access_control::{AccessControlManager, AccessPolicy, Role, Permission, AccessRequest};
pub use self::sla::{SLAManager, ServiceLevelAgreement, SLAMetric, SLAReport};

use crate::error::Error;
use crate::metrics::MetricsCollector;
use std::sync::{Arc, Mutex};
use log::{debug, error, info, warn};
use serde::{Serialize, Deserialize};

/// エンタープライズ設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnterpriseConfig {
    /// セキュリティ設定
    pub security_config: SecurityConfig,
    /// コンプライアンス設定
    pub compliance_config: ComplianceConfig,
    /// 監査設定
    pub audit_config: AuditConfig,
    /// アクセス制御設定
    pub access_control_config: AccessControlConfig,
    /// SLA設定
    pub sla_config: SLAConfig,
}

/// セキュリティ設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// 暗号化設定
    pub encryption_config: EncryptionConfig,
    /// キー管理設定
    pub key_management_config: KeyManagementConfig,
    /// 認証設定
    pub authentication_config: AuthenticationConfig,
    /// セキュリティポリシー
    pub security_policies: Vec<SecurityPolicy>,
}

/// 暗号化設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// データ暗号化アルゴリズム
    pub data_encryption_algorithm: String,
    /// 通信暗号化アルゴリズム
    pub transport_encryption_algorithm: String,
    /// キー長
    pub key_length: u32,
    /// 初期化ベクトル長
    pub iv_length: u32,
}

/// キー管理設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyManagementConfig {
    /// キーローテーション間隔（秒）
    pub key_rotation_interval_seconds: u64,
    /// キーバックアップ有効フラグ
    pub enable_key_backup: bool,
    /// HSM有効フラグ
    pub enable_hsm: bool,
    /// HSM設定
    pub hsm_config: Option<HSMConfig>,
}

/// HSM設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HSMConfig {
    /// HSMタイプ
    pub hsm_type: String,
    /// HSMエンドポイント
    pub endpoint: String,
    /// HSMトークン
    pub token: String,
    /// HSMスロットID
    pub slot_id: u32,
}

/// 認証設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    /// 多要素認証有効フラグ
    pub enable_mfa: bool,
    /// パスワードポリシー
    pub password_policy: PasswordPolicy,
    /// セッションタイムアウト（秒）
    pub session_timeout_seconds: u64,
    /// 最大ログイン試行回数
    pub max_login_attempts: u32,
    /// ロックアウト期間（秒）
    pub lockout_duration_seconds: u64,
}

/// パスワードポリシー
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PasswordPolicy {
    /// 最小長
    pub min_length: u32,
    /// 大文字必須フラグ
    pub require_uppercase: bool,
    /// 小文字必須フラグ
    pub require_lowercase: bool,
    /// 数字必須フラグ
    pub require_numbers: bool,
    /// 特殊文字必須フラグ
    pub require_special_chars: bool,
    /// パスワード有効期間（秒）
    pub password_expiry_seconds: u64,
    /// パスワード履歴サイズ
    pub password_history_size: u32,
}

/// コンプライアンス設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// コンプライアンスフレームワーク
    pub frameworks: Vec<ComplianceFramework>,
    /// 自動監査有効フラグ
    pub enable_automated_audits: bool,
    /// 監査間隔（秒）
    pub audit_interval_seconds: u64,
    /// レポート生成有効フラグ
    pub enable_report_generation: bool,
    /// レポート生成間隔（秒）
    pub report_generation_interval_seconds: u64,
}

/// 監査設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditConfig {
    /// 監査ログ有効フラグ
    pub enable_audit_logging: bool,
    /// 監査ログレベル
    pub audit_log_level: AuditLogLevel,
    /// 監査ログ保持期間（秒）
    pub audit_log_retention_seconds: u64,
    /// 監査ログエクスポート有効フラグ
    pub enable_audit_log_export: bool,
    /// 監査ログエクスポート間隔（秒）
    pub audit_log_export_interval_seconds: u64,
}

/// 監査ログレベル
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditLogLevel {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 最高
    Critical,
}

/// アクセス制御設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessControlConfig {
    /// アクセス制御モデル
    pub access_control_model: AccessControlModel,
    /// ロールベースアクセス制御有効フラグ
    pub enable_rbac: bool,
    /// 属性ベースアクセス制御有効フラグ
    pub enable_abac: bool,
    /// 最小権限の原則有効フラグ
    pub enforce_least_privilege: bool,
    /// 職務分掌有効フラグ
    pub enforce_separation_of_duties: bool,
}

/// アクセス制御モデル
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessControlModel {
    /// 任意アクセス制御
    DAC,
    /// 強制アクセス制御
    MAC,
    /// ロールベースアクセス制御
    RBAC,
    /// 属性ベースアクセス制御
    ABAC,
}

/// SLA設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SLAConfig {
    /// SLA監視有効フラグ
    pub enable_sla_monitoring: bool,
    /// SLA違反アラート有効フラグ
    pub enable_sla_violation_alerts: bool,
    /// SLAレポート有効フラグ
    pub enable_sla_reporting: bool,
    /// SLAレポート間隔（秒）
    pub sla_report_interval_seconds: u64,
    /// SLAメトリクス
    pub sla_metrics: Vec<SLAMetricConfig>,
}

/// SLAメトリック設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SLAMetricConfig {
    /// メトリック名
    pub name: String,
    /// 説明
    pub description: String,
    /// 目標値
    pub target_value: f64,
    /// 単位
    pub unit: String,
    /// 重要度
    pub criticality: SLACriticality,
}

/// SLA重要度
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SLACriticality {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 最高
    Critical,
}

/// エンタープライズマネージャー
pub struct EnterpriseManager {
    /// エンタープライズ設定
    config: EnterpriseConfig,
    /// セキュリティマネージャー
    security_manager: Arc<Mutex<security::EnterpriseSecurityManager>>,
    /// コンプライアンスマネージャー
    compliance_manager: Arc<Mutex<compliance::ComplianceManager>>,
    /// 監査マネージャー
    audit_manager: Arc<Mutex<audit::AuditManager>>,
    /// アクセス制御マネージャー
    access_control_manager: Arc<Mutex<access_control::AccessControlManager>>,
    /// SLAマネージャー
    sla_manager: Arc<Mutex<sla::SLAManager>>,
    /// メトリクスコレクター
    metrics: Arc<MetricsCollector>,
}

impl EnterpriseManager {
    /// 新しいEnterpriseManagerを作成
    pub fn new(config: EnterpriseConfig, metrics: Arc<MetricsCollector>) -> Self {
        let security_manager = Arc::new(Mutex::new(
            security::EnterpriseSecurityManager::new(config.security_config.clone())
        ));
        
        let compliance_manager = Arc::new(Mutex::new(
            compliance::ComplianceManager::new(config.compliance_config.clone())
        ));
        
        let audit_manager = Arc::new(Mutex::new(
            audit::AuditManager::new(config.audit_config.clone())
        ));
        
        let access_control_manager = Arc::new(Mutex::new(
            access_control::AccessControlManager::new(config.access_control_config.clone())
        ));
        
        let sla_manager = Arc::new(Mutex::new(
            sla::SLAManager::new(config.sla_config.clone())
        ));
        
        Self {
            config,
            security_manager,
            compliance_manager,
            audit_manager,
            access_control_manager,
            sla_manager,
            metrics,
        }
    }
    
    /// セキュリティマネージャーを取得
    pub fn security_manager(&self) -> Arc<Mutex<security::EnterpriseSecurityManager>> {
        self.security_manager.clone()
    }
    
    /// コンプライアンスマネージャーを取得
    pub fn compliance_manager(&self) -> Arc<Mutex<compliance::ComplianceManager>> {
        self.compliance_manager.clone()
    }
    
    /// 監査マネージャーを取得
    pub fn audit_manager(&self) -> Arc<Mutex<audit::AuditManager>> {
        self.audit_manager.clone()
    }
    
    /// アクセス制御マネージャーを取得
    pub fn access_control_manager(&self) -> Arc<Mutex<access_control::AccessControlManager>> {
        self.access_control_manager.clone()
    }
    
    /// SLAマネージャーを取得
    pub fn sla_manager(&self) -> Arc<Mutex<sla::SLAManager>> {
        self.sla_manager.clone()
    }
    
    /// 監査イベントを記録
    pub async fn log_audit_event(
        &self,
        event_type: audit::AuditEventType,
        user_id: &str,
        resource: &str,
        action: &str,
        result: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<String, Error> {
        let mut audit_manager = self.audit_manager.lock().unwrap();
        let event_id = audit_manager.log_event(event_type, user_id, resource, action, result, metadata)?;
        
        // メトリクスを更新
        self.metrics.increment_counter("enterprise_audit_events_logged");
        
        Ok(event_id)
    }
    
    /// アクセス要求を検証
    pub async fn verify_access(
        &self,
        user_id: &str,
        resource: &str,
        action: &str,
    ) -> Result<bool, Error> {
        let access_control_manager = self.access_control_manager.lock().unwrap();
        let has_access = access_control_manager.verify_access(user_id, resource, action)?;
        
        // アクセス検証を監査ログに記録
        let result = if has_access { "allowed" } else { "denied" };
        let metadata = Some(serde_json::json!({
            "resource": resource,
            "action": action,
        }));
        
        let mut audit_manager = self.audit_manager.lock().unwrap();
        audit_manager.log_event(
            audit::AuditEventType::AccessControl,
            user_id,
            resource,
            action,
            result,
            metadata,
        )?;
        
        // メトリクスを更新
        if has_access {
            self.metrics.increment_counter("enterprise_access_allowed");
        } else {
            self.metrics.increment_counter("enterprise_access_denied");
        }
        
        Ok(has_access)
    }
    
    /// セキュリティポリシーを適用
    pub async fn apply_security_policy(
        &self,
        policy_id: &str,
        target: &str,
    ) -> Result<(), Error> {
        let mut security_manager = self.security_manager.lock().unwrap();
        security_manager.apply_policy(policy_id, target)?;
        
        // メトリクスを更新
        self.metrics.increment_counter("enterprise_security_policies_applied");
        
        Ok(())
    }
    
    /// コンプライアンス監査を実行
    pub async fn run_compliance_audit(
        &self,
        framework_id: &str,
    ) -> Result<compliance::ComplianceReport, Error> {
        let mut compliance_manager = self.compliance_manager.lock().unwrap();
        let report = compliance_manager.run_audit(framework_id)?;
        
        // メトリクスを更新
        self.metrics.increment_counter("enterprise_compliance_audits_run");
        
        Ok(report)
    }
    
    /// SLAメトリックを更新
    pub async fn update_sla_metric(
        &self,
        metric_name: &str,
        value: f64,
    ) -> Result<(), Error> {
        let mut sla_manager = self.sla_manager.lock().unwrap();
        sla_manager.update_metric(metric_name, value)?;
        
        // メトリクスを更新
        self.metrics.increment_counter("enterprise_sla_metrics_updated");
        
        Ok(())
    }
    
    /// SLAレポートを生成
    pub async fn generate_sla_report(
        &self,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<sla::SLAReport, Error> {
        let mut sla_manager = self.sla_manager.lock().unwrap();
        let report = sla_manager.generate_report(start_time, end_time)?;
        
        // メトリクスを更新
        self.metrics.increment_counter("enterprise_sla_reports_generated");
        
        Ok(report)
    }
    
    /// 設定を取得
    pub fn get_config(&self) -> &EnterpriseConfig {
        &self.config
    }
    
    /// 設定を更新
    pub fn update_config(&mut self, config: EnterpriseConfig) -> Result<(), Error> {
        // 各マネージャーの設定を更新
        {
            let mut security_manager = self.security_manager.lock().unwrap();
            security_manager.update_config(config.security_config.clone());
        }
        
        {
            let mut compliance_manager = self.compliance_manager.lock().unwrap();
            compliance_manager.update_config(config.compliance_config.clone());
        }
        
        {
            let mut audit_manager = self.audit_manager.lock().unwrap();
            audit_manager.update_config(config.audit_config.clone());
        }
        
        {
            let mut access_control_manager = self.access_control_manager.lock().unwrap();
            access_control_manager.update_config(config.access_control_config.clone());
        }
        
        {
            let mut sla_manager = self.sla_manager.lock().unwrap();
            sla_manager.update_config(config.sla_config.clone());
        }
        
        // 設定を更新
        self.config = config;
        
        Ok(())
    }
}

impl Default for EnterpriseConfig {
    fn default() -> Self {
        Self {
            security_config: SecurityConfig::default(),
            compliance_config: ComplianceConfig::default(),
            audit_config: AuditConfig::default(),
            access_control_config: AccessControlConfig::default(),
            sla_config: SLAConfig::default(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encryption_config: EncryptionConfig {
                data_encryption_algorithm: "AES-256-GCM".to_string(),
                transport_encryption_algorithm: "TLS-1.3".to_string(),
                key_length: 256,
                iv_length: 12,
            },
            key_management_config: KeyManagementConfig {
                key_rotation_interval_seconds: 7776000, // 90日
                enable_key_backup: true,
                enable_hsm: false,
                hsm_config: None,
            },
            authentication_config: AuthenticationConfig {
                enable_mfa: true,
                password_policy: PasswordPolicy {
                    min_length: 12,
                    require_uppercase: true,
                    require_lowercase: true,
                    require_numbers: true,
                    require_special_chars: true,
                    password_expiry_seconds: 7776000, // 90日
                    password_history_size: 10,
                },
                session_timeout_seconds: 3600, // 1時間
                max_login_attempts: 5,
                lockout_duration_seconds: 1800, // 30分
            },
            security_policies: Vec::new(),
        }
    }
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            frameworks: Vec::new(),
            enable_automated_audits: true,
            audit_interval_seconds: 2592000, // 30日
            enable_report_generation: true,
            report_generation_interval_seconds: 2592000, // 30日
        }
    }
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enable_audit_logging: true,
            audit_log_level: AuditLogLevel::Medium,
            audit_log_retention_seconds: 31536000, // 1年
            enable_audit_log_export: true,
            audit_log_export_interval_seconds: 86400, // 1日
        }
    }
}

impl Default for AccessControlConfig {
    fn default() -> Self {
        Self {
            access_control_model: AccessControlModel::RBAC,
            enable_rbac: true,
            enable_abac: false,
            enforce_least_privilege: true,
            enforce_separation_of_duties: true,
        }
    }
}

impl Default for SLAConfig {
    fn default() -> Self {
        Self {
            enable_sla_monitoring: true,
            enable_sla_violation_alerts: true,
            enable_sla_reporting: true,
            sla_report_interval_seconds: 604800, // 1週間
            sla_metrics: vec![
                SLAMetricConfig {
                    name: "availability".to_string(),
                    description: "System availability percentage".to_string(),
                    target_value: 99.99,
                    unit: "percent".to_string(),
                    criticality: SLACriticality::Critical,
                },
                SLAMetricConfig {
                    name: "response_time".to_string(),
                    description: "Average response time".to_string(),
                    target_value: 100.0,
                    unit: "milliseconds".to_string(),
                    criticality: SLACriticality::High,
                },
                SLAMetricConfig {
                    name: "throughput".to_string(),
                    description: "System throughput".to_string(),
                    target_value: 1000.0,
                    unit: "transactions_per_second".to_string(),
                    criticality: SLACriticality::Medium,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_enterprise_config() {
        let config = EnterpriseConfig::default();
        
        // セキュリティ設定をチェック
        assert_eq!(config.security_config.encryption_config.data_encryption_algorithm, "AES-256-GCM");
        assert_eq!(config.security_config.encryption_config.key_length, 256);
        assert_eq!(config.security_config.key_management_config.key_rotation_interval_seconds, 7776000);
        assert!(config.security_config.authentication_config.enable_mfa);
        assert_eq!(config.security_config.authentication_config.password_policy.min_length, 12);
        
        // 監査設定をチェック
        assert!(config.audit_config.enable_audit_logging);
        assert_eq!(config.audit_config.audit_log_level, AuditLogLevel::Medium);
        assert_eq!(config.audit_config.audit_log_retention_seconds, 31536000);
        
        // アクセス制御設定をチェック
        assert_eq!(config.access_control_config.access_control_model, AccessControlModel::RBAC);
        assert!(config.access_control_config.enable_rbac);
        assert!(config.access_control_config.enforce_least_privilege);
        
        // SLA設定をチェック
        assert!(config.sla_config.enable_sla_monitoring);
        assert_eq!(config.sla_config.sla_metrics.len(), 3);
        assert_eq!(config.sla_config.sla_metrics[0].name, "availability");
        assert_eq!(config.sla_config.sla_metrics[0].target_value, 99.99);
    }
    
    #[tokio::test]
    async fn test_enterprise_manager() {
        let config = EnterpriseConfig::default();
        let metrics = Arc::new(MetricsCollector::new("enterprise"));
        
        let manager = EnterpriseManager::new(config, metrics);
        
        // 各マネージャーが正しく初期化されていることを確認
        assert!(manager.security_manager.lock().unwrap().is_initialized());
        assert!(manager.compliance_manager.lock().unwrap().is_initialized());
        assert!(manager.audit_manager.lock().unwrap().is_initialized());
        assert!(manager.access_control_manager.lock().unwrap().is_initialized());
        assert!(manager.sla_manager.lock().unwrap().is_initialized());
    }
}