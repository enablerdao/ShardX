// エンタープライズセキュリティモジュール
//
// このモジュールは、ShardXのエンタープライズグレードのセキュリティ機能を提供します。
// 主な機能:
// - 高度な暗号化
// - キー管理
// - 多要素認証
// - セキュリティポリシー
// - 脆弱性管理

mod encryption;
mod key_management;
mod authentication;
mod policy;
mod vulnerability;

pub use self::encryption::{EncryptionManager, EncryptionAlgorithm, EncryptionMode};
pub use self::key_management::{KeyManager, KeyRotation, KeyBackup, HSMIntegration};
pub use self::authentication::{AuthenticationManager, MFAProvider, AuthenticationMethod};
pub use self::policy::{PolicyManager, SecurityPolicy, PolicyEnforcement};
pub use self::vulnerability::{VulnerabilityManager, VulnerabilityScan, VulnerabilityReport};

use crate::error::Error;
use crate::enterprise::{SecurityConfig, EncryptionConfig, KeyManagementConfig, AuthenticationConfig};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// セキュリティ制御
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecurityControl {
    /// 制御ID
    pub id: String,
    /// 名前
    pub name: String,
    /// 説明
    pub description: String,
    /// カテゴリ
    pub category: SecurityControlCategory,
    /// 重要度
    pub criticality: SecurityControlCriticality,
    /// 実装ステータス
    pub implementation_status: ImplementationStatus,
    /// 検証ステータス
    pub verification_status: VerificationStatus,
    /// 最終レビュー日時
    pub last_reviewed_at: Option<DateTime<Utc>>,
    /// レビュー担当者
    pub reviewed_by: Option<String>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// セキュリティ制御カテゴリ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityControlCategory {
    /// 管理的制御
    Administrative,
    /// 技術的制御
    Technical,
    /// 物理的制御
    Physical,
    /// 運用的制御
    Operational,
}

/// セキュリティ制御重要度
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityControlCriticality {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 最高
    Critical,
}

/// 実装ステータス
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImplementationStatus {
    /// 未実装
    NotImplemented,
    /// 部分的に実装
    PartiallyImplemented,
    /// 実装済み
    Implemented,
    /// 適用外
    NotApplicable,
}

/// 検証ステータス
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// 未検証
    NotVerified,
    /// 検証済み
    Verified,
    /// 検証失敗
    Failed,
    /// 適用外
    NotApplicable,
}

/// セキュリティ監査
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecurityAudit {
    /// 監査ID
    pub id: String,
    /// 名前
    pub name: String,
    /// 説明
    pub description: String,
    /// 監査タイプ
    pub audit_type: SecurityAuditType,
    /// 開始日時
    pub start_time: DateTime<Utc>,
    /// 終了日時
    pub end_time: Option<DateTime<Utc>>,
    /// ステータス
    pub status: SecurityAuditStatus,
    /// 監査担当者
    pub auditor: String,
    /// 監査対象
    pub target: String,
    /// 結果
    pub results: Vec<SecurityAuditResult>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// セキュリティ監査タイプ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityAuditType {
    /// 内部監査
    Internal,
    /// 外部監査
    External,
    /// 自動監査
    Automated,
    /// 手動監査
    Manual,
}

/// セキュリティ監査ステータス
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityAuditStatus {
    /// 予定
    Scheduled,
    /// 進行中
    InProgress,
    /// 完了
    Completed,
    /// キャンセル
    Cancelled,
}

/// セキュリティ監査結果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecurityAuditResult {
    /// 結果ID
    pub id: String,
    /// 制御ID
    pub control_id: String,
    /// ステータス
    pub status: SecurityAuditResultStatus,
    /// 説明
    pub description: String,
    /// 証拠
    pub evidence: Option<String>,
    /// 重大度
    pub severity: SecurityAuditResultSeverity,
    /// 修正アクション
    pub remediation_action: Option<String>,
    /// 修正期限
    pub remediation_deadline: Option<DateTime<Utc>>,
    /// 修正担当者
    pub remediation_owner: Option<String>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// セキュリティ監査結果ステータス
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityAuditResultStatus {
    /// 合格
    Pass,
    /// 不合格
    Fail,
    /// 部分的に合格
    PartialPass,
    /// 適用外
    NotApplicable,
}

/// セキュリティ監査結果重大度
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityAuditResultSeverity {
    /// 情報
    Info,
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 最高
    Critical,
}

/// エンタープライズセキュリティマネージャー
pub struct EnterpriseSecurityManager {
    /// セキュリティ設定
    config: SecurityConfig,
    /// 暗号化マネージャー
    encryption_manager: encryption::EncryptionManager,
    /// キーマネージャー
    key_manager: key_management::KeyManager,
    /// 認証マネージャー
    authentication_manager: authentication::AuthenticationManager,
    /// ポリシーマネージャー
    policy_manager: policy::PolicyManager,
    /// 脆弱性マネージャー
    vulnerability_manager: vulnerability::VulnerabilityManager,
    /// セキュリティ制御
    security_controls: HashMap<String, SecurityControl>,
    /// セキュリティ監査
    security_audits: HashMap<String, SecurityAudit>,
    /// 初期化済みフラグ
    initialized: bool,
}

impl EnterpriseSecurityManager {
    /// 新しいEnterpriseSecurityManagerを作成
    pub fn new(config: SecurityConfig) -> Self {
        let encryption_manager = encryption::EncryptionManager::new(config.encryption_config.clone());
        let key_manager = key_management::KeyManager::new(config.key_management_config.clone());
        let authentication_manager = authentication::AuthenticationManager::new(config.authentication_config.clone());
        let policy_manager = policy::PolicyManager::new(config.security_policies.clone());
        let vulnerability_manager = vulnerability::VulnerabilityManager::new();
        
        let mut manager = Self {
            config,
            encryption_manager,
            key_manager,
            authentication_manager,
            policy_manager,
            vulnerability_manager,
            security_controls: HashMap::new(),
            security_audits: HashMap::new(),
            initialized: true,
        };
        
        // デフォルトのセキュリティ制御を初期化
        manager.initialize_default_controls();
        
        manager
    }
    
    /// 初期化済みかどうかを確認
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// デフォルトのセキュリティ制御を初期化
    fn initialize_default_controls(&mut self) {
        // 管理的制御
        self.add_security_control(
            "SEC-ADM-001",
            "Security Policy and Procedures",
            "Establish and maintain security policies and procedures",
            SecurityControlCategory::Administrative,
            SecurityControlCriticality::High,
        );
        
        self.add_security_control(
            "SEC-ADM-002",
            "Risk Assessment",
            "Conduct regular risk assessments",
            SecurityControlCategory::Administrative,
            SecurityControlCriticality::High,
        );
        
        self.add_security_control(
            "SEC-ADM-003",
            "Security Awareness Training",
            "Provide security awareness training to all personnel",
            SecurityControlCategory::Administrative,
            SecurityControlCriticality::Medium,
        );
        
        // 技術的制御
        self.add_security_control(
            "SEC-TEC-001",
            "Access Control",
            "Implement access control mechanisms",
            SecurityControlCategory::Technical,
            SecurityControlCriticality::Critical,
        );
        
        self.add_security_control(
            "SEC-TEC-002",
            "Encryption",
            "Implement data encryption at rest and in transit",
            SecurityControlCategory::Technical,
            SecurityControlCriticality::Critical,
        );
        
        self.add_security_control(
            "SEC-TEC-003",
            "Multi-Factor Authentication",
            "Implement multi-factor authentication for all privileged accounts",
            SecurityControlCategory::Technical,
            SecurityControlCriticality::High,
        );
        
        self.add_security_control(
            "SEC-TEC-004",
            "Audit Logging",
            "Implement comprehensive audit logging and monitoring",
            SecurityControlCategory::Technical,
            SecurityControlCriticality::High,
        );
        
        // 物理的制御
        self.add_security_control(
            "SEC-PHY-001",
            "Physical Access Control",
            "Implement physical access controls to data centers and sensitive areas",
            SecurityControlCategory::Physical,
            SecurityControlCriticality::High,
        );
        
        self.add_security_control(
            "SEC-PHY-002",
            "Environmental Controls",
            "Implement environmental controls to protect against environmental threats",
            SecurityControlCategory::Physical,
            SecurityControlCriticality::Medium,
        );
        
        // 運用的制御
        self.add_security_control(
            "SEC-OPS-001",
            "Incident Response",
            "Establish and maintain an incident response capability",
            SecurityControlCategory::Operational,
            SecurityControlCriticality::High,
        );
        
        self.add_security_control(
            "SEC-OPS-002",
            "Change Management",
            "Implement change management procedures",
            SecurityControlCategory::Operational,
            SecurityControlCriticality::Medium,
        );
        
        self.add_security_control(
            "SEC-OPS-003",
            "Backup and Recovery",
            "Implement backup and recovery procedures",
            SecurityControlCategory::Operational,
            SecurityControlCriticality::High,
        );
    }
    
    /// セキュリティ制御を追加
    fn add_security_control(
        &mut self,
        id: &str,
        name: &str,
        description: &str,
        category: SecurityControlCategory,
        criticality: SecurityControlCriticality,
    ) {
        let control = SecurityControl {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            category,
            criticality,
            implementation_status: ImplementationStatus::NotImplemented,
            verification_status: VerificationStatus::NotVerified,
            last_reviewed_at: None,
            reviewed_by: None,
            metadata: HashMap::new(),
        };
        
        self.security_controls.insert(id.to_string(), control);
    }
    
    /// データを暗号化
    pub fn encrypt_data(&self, data: &[u8], context: Option<&str>) -> Result<Vec<u8>, Error> {
        self.encryption_manager.encrypt(data, context)
    }
    
    /// データを復号
    pub fn decrypt_data(&self, encrypted_data: &[u8], context: Option<&str>) -> Result<Vec<u8>, Error> {
        self.encryption_manager.decrypt(encrypted_data, context)
    }
    
    /// キーをローテーション
    pub fn rotate_keys(&mut self) -> Result<(), Error> {
        self.key_manager.rotate_keys()
    }
    
    /// キーをバックアップ
    pub fn backup_keys(&self, backup_path: &str) -> Result<(), Error> {
        self.key_manager.backup_keys(backup_path)
    }
    
    /// キーを復元
    pub fn restore_keys(&mut self, backup_path: &str) -> Result<(), Error> {
        self.key_manager.restore_keys(backup_path)
    }
    
    /// 認証を検証
    pub fn verify_authentication(
        &self,
        user_id: &str,
        password: &str,
        mfa_code: Option<&str>,
    ) -> Result<bool, Error> {
        self.authentication_manager.verify_authentication(user_id, password, mfa_code)
    }
    
    /// MFAを有効化
    pub fn enable_mfa(&mut self, user_id: &str, mfa_type: authentication::MFAType) -> Result<String, Error> {
        self.authentication_manager.enable_mfa(user_id, mfa_type)
    }
    
    /// MFAを無効化
    pub fn disable_mfa(&mut self, user_id: &str) -> Result<(), Error> {
        self.authentication_manager.disable_mfa(user_id)
    }
    
    /// セキュリティポリシーを適用
    pub fn apply_policy(&mut self, policy_id: &str, target: &str) -> Result<(), Error> {
        self.policy_manager.apply_policy(policy_id, target)
    }
    
    /// セキュリティポリシーを検証
    pub fn verify_policy_compliance(&self, policy_id: &str, target: &str) -> Result<bool, Error> {
        self.policy_manager.verify_compliance(policy_id, target)
    }
    
    /// 脆弱性スキャンを実行
    pub fn run_vulnerability_scan(
        &mut self,
        target: &str,
        scan_type: vulnerability::ScanType,
    ) -> Result<String, Error> {
        self.vulnerability_manager.run_scan(target, scan_type)
    }
    
    /// 脆弱性レポートを取得
    pub fn get_vulnerability_report(&self, scan_id: &str) -> Result<vulnerability::VulnerabilityReport, Error> {
        self.vulnerability_manager.get_report(scan_id)
    }
    
    /// セキュリティ制御を取得
    pub fn get_security_control(&self, control_id: &str) -> Result<SecurityControl, Error> {
        self.security_controls.get(control_id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Security control not found: {}", control_id)))
    }
    
    /// セキュリティ制御リストを取得
    pub fn get_security_controls(&self, category: Option<SecurityControlCategory>) -> Vec<SecurityControl> {
        self.security_controls.values()
            .filter(|c| category.as_ref().map_or(true, |cat| c.category == *cat))
            .cloned()
            .collect()
    }
    
    /// セキュリティ制御ステータスを更新
    pub fn update_security_control_status(
        &mut self,
        control_id: &str,
        implementation_status: ImplementationStatus,
        verification_status: VerificationStatus,
        reviewer: &str,
    ) -> Result<(), Error> {
        let control = self.security_controls.get_mut(control_id)
            .ok_or_else(|| Error::NotFound(format!("Security control not found: {}", control_id)))?;
        
        control.implementation_status = implementation_status;
        control.verification_status = verification_status;
        control.last_reviewed_at = Some(Utc::now());
        control.reviewed_by = Some(reviewer.to_string());
        
        Ok(())
    }
    
    /// セキュリティ監査を作成
    pub fn create_security_audit(
        &mut self,
        name: &str,
        description: &str,
        audit_type: SecurityAuditType,
        auditor: &str,
        target: &str,
    ) -> Result<String, Error> {
        // 監査IDを生成
        let audit_id = format!("AUDIT-{}", Uuid::new_v4());
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // 監査を作成
        let audit = SecurityAudit {
            id: audit_id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            audit_type,
            start_time: now,
            end_time: None,
            status: SecurityAuditStatus::InProgress,
            auditor: auditor.to_string(),
            target: target.to_string(),
            results: Vec::new(),
            metadata: HashMap::new(),
        };
        
        // 監査を保存
        self.security_audits.insert(audit_id.clone(), audit);
        
        info!("Security audit created: {} ({})", name, audit_id);
        
        Ok(audit_id)
    }
    
    /// セキュリティ監査結果を追加
    pub fn add_security_audit_result(
        &mut self,
        audit_id: &str,
        control_id: &str,
        status: SecurityAuditResultStatus,
        description: &str,
        evidence: Option<&str>,
        severity: SecurityAuditResultSeverity,
        remediation_action: Option<&str>,
        remediation_deadline: Option<DateTime<Utc>>,
        remediation_owner: Option<&str>,
    ) -> Result<String, Error> {
        // 監査を取得
        let audit = self.security_audits.get_mut(audit_id)
            .ok_or_else(|| Error::NotFound(format!("Security audit not found: {}", audit_id)))?;
        
        // 監査ステータスをチェック
        if audit.status != SecurityAuditStatus::InProgress {
            return Err(Error::InvalidState(format!("Security audit is not in progress: {}", audit_id)));
        }
        
        // 制御が存在するかチェック
        if !self.security_controls.contains_key(control_id) {
            return Err(Error::NotFound(format!("Security control not found: {}", control_id)));
        }
        
        // 結果IDを生成
        let result_id = format!("RESULT-{}", Uuid::new_v4());
        
        // 結果を作成
        let result = SecurityAuditResult {
            id: result_id.clone(),
            control_id: control_id.to_string(),
            status,
            description: description.to_string(),
            evidence: evidence.map(|e| e.to_string()),
            severity,
            remediation_action: remediation_action.map(|a| a.to_string()),
            remediation_deadline,
            remediation_owner: remediation_owner.map(|o| o.to_string()),
            metadata: HashMap::new(),
        };
        
        // 結果を追加
        audit.results.push(result);
        
        info!("Security audit result added: {} for control {}", result_id, control_id);
        
        Ok(result_id)
    }
    
    /// セキュリティ監査を完了
    pub fn complete_security_audit(&mut self, audit_id: &str) -> Result<(), Error> {
        // 監査を取得
        let audit = self.security_audits.get_mut(audit_id)
            .ok_or_else(|| Error::NotFound(format!("Security audit not found: {}", audit_id)))?;
        
        // 監査ステータスをチェック
        if audit.status != SecurityAuditStatus::InProgress {
            return Err(Error::InvalidState(format!("Security audit is not in progress: {}", audit_id)));
        }
        
        // 監査を完了
        audit.status = SecurityAuditStatus::Completed;
        audit.end_time = Some(Utc::now());
        
        info!("Security audit completed: {}", audit_id);
        
        Ok(())
    }
    
    /// セキュリティ監査を取得
    pub fn get_security_audit(&self, audit_id: &str) -> Result<SecurityAudit, Error> {
        self.security_audits.get(audit_id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Security audit not found: {}", audit_id)))
    }
    
    /// セキュリティ監査リストを取得
    pub fn get_security_audits(&self, status: Option<SecurityAuditStatus>) -> Vec<SecurityAudit> {
        self.security_audits.values()
            .filter(|a| status.as_ref().map_or(true, |s| a.status == *s))
            .cloned()
            .collect()
    }
    
    /// 設定を更新
    pub fn update_config(&mut self, config: SecurityConfig) {
        self.config = config.clone();
        self.encryption_manager.update_config(config.encryption_config);
        self.key_manager.update_config(config.key_management_config);
        self.authentication_manager.update_config(config.authentication_config);
        self.policy_manager.update_policies(config.security_policies);
    }
    
    /// 設定を取得
    pub fn get_config(&self) -> &SecurityConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_security_manager_initialization() {
        let config = SecurityConfig::default();
        let manager = EnterpriseSecurityManager::new(config);
        
        assert!(manager.is_initialized());
        
        // デフォルトのセキュリティ制御が初期化されていることを確認
        let controls = manager.get_security_controls(None);
        assert!(!controls.is_empty());
        
        // 管理的制御をチェック
        let admin_controls = manager.get_security_controls(Some(SecurityControlCategory::Administrative));
        assert!(!admin_controls.is_empty());
        
        // 技術的制御をチェック
        let tech_controls = manager.get_security_controls(Some(SecurityControlCategory::Technical));
        assert!(!tech_controls.is_empty());
        
        // 物理的制御をチェック
        let phys_controls = manager.get_security_controls(Some(SecurityControlCategory::Physical));
        assert!(!phys_controls.is_empty());
        
        // 運用的制御をチェック
        let ops_controls = manager.get_security_controls(Some(SecurityControlCategory::Operational));
        assert!(!ops_controls.is_empty());
    }
    
    #[test]
    fn test_security_control_update() {
        let config = SecurityConfig::default();
        let mut manager = EnterpriseSecurityManager::new(config);
        
        // セキュリティ制御を取得
        let control_id = "SEC-TEC-001"; // Access Control
        let control = manager.get_security_control(control_id).unwrap();
        
        // 初期ステータスを確認
        assert_eq!(control.implementation_status, ImplementationStatus::NotImplemented);
        assert_eq!(control.verification_status, VerificationStatus::NotVerified);
        assert!(control.last_reviewed_at.is_none());
        assert!(control.reviewed_by.is_none());
        
        // ステータスを更新
        manager.update_security_control_status(
            control_id,
            ImplementationStatus::Implemented,
            VerificationStatus::Verified,
            "security-admin",
        ).unwrap();
        
        // 更新後のステータスを確認
        let updated_control = manager.get_security_control(control_id).unwrap();
        assert_eq!(updated_control.implementation_status, ImplementationStatus::Implemented);
        assert_eq!(updated_control.verification_status, VerificationStatus::Verified);
        assert!(updated_control.last_reviewed_at.is_some());
        assert_eq!(updated_control.reviewed_by, Some("security-admin".to_string()));
    }
    
    #[test]
    fn test_security_audit() {
        let config = SecurityConfig::default();
        let mut manager = EnterpriseSecurityManager::new(config);
        
        // 監査を作成
        let audit_id = manager.create_security_audit(
            "Annual Security Audit",
            "Comprehensive security audit of all controls",
            SecurityAuditType::Internal,
            "security-auditor",
            "all-systems",
        ).unwrap();
        
        // 監査を取得
        let audit = manager.get_security_audit(&audit_id).unwrap();
        
        // 監査をチェック
        assert_eq!(audit.name, "Annual Security Audit");
        assert_eq!(audit.audit_type, SecurityAuditType::Internal);
        assert_eq!(audit.status, SecurityAuditStatus::InProgress);
        assert_eq!(audit.auditor, "security-auditor");
        assert_eq!(audit.target, "all-systems");
        assert!(audit.results.is_empty());
        
        // 監査結果を追加
        let result_id = manager.add_security_audit_result(
            &audit_id,
            "SEC-TEC-001", // Access Control
            SecurityAuditResultStatus::Pass,
            "Access control mechanisms are properly implemented",
            Some("Access control logs and configuration review"),
            SecurityAuditResultSeverity::High,
            None,
            None,
            None,
        ).unwrap();
        
        // 監査を完了
        manager.complete_security_audit(&audit_id).unwrap();
        
        // 完了した監査を取得
        let completed_audit = manager.get_security_audit(&audit_id).unwrap();
        
        // 監査をチェック
        assert_eq!(completed_audit.status, SecurityAuditStatus::Completed);
        assert!(completed_audit.end_time.is_some());
        assert_eq!(completed_audit.results.len(), 1);
        assert_eq!(completed_audit.results[0].control_id, "SEC-TEC-001");
        assert_eq!(completed_audit.results[0].status, SecurityAuditResultStatus::Pass);
    }
}