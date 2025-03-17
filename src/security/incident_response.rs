use crate::error::Error;
use crate::security::vulnerability_scanner::SeverityLevel;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};
use uuid::Uuid;

/// インシデントステータス
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentStatus {
    /// 未対応
    Open,
    /// 調査中
    Investigating,
    /// 対応中
    Responding,
    /// 解決済み
    Resolved,
    /// クローズ
    Closed,
}

/// セキュリティインシデント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIncident {
    /// インシデントID
    pub id: String,
    /// タイトル
    pub title: String,
    /// 説明
    pub description: String,
    /// 重大度
    pub severity: SeverityLevel,
    /// ステータス
    pub status: IncidentStatus,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 更新日時
    pub updated_at: DateTime<Utc>,
    /// 担当者
    pub assigned_to: Option<String>,
    /// 関連する脆弱性
    pub related_vulnerabilities: Vec<String>,
    /// 解決策
    pub resolution: Option<String>,
}

/// インシデント対応計画
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentResponsePlan {
    /// 計画ID
    pub id: String,
    /// インシデントID
    pub incident_id: String,
    /// 計画名
    pub name: String,
    /// 説明
    pub description: String,
    /// 対応ステップ
    pub steps: Vec<ResponseStep>,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 更新日時
    pub updated_at: DateTime<Utc>,
    /// 作成者
    pub created_by: String,
}

/// 対応ステップ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseStep {
    /// ステップID
    pub id: String,
    /// ステップ番号
    pub step_number: u32,
    /// タイトル
    pub title: String,
    /// 説明
    pub description: String,
    /// 完了フラグ
    pub completed: bool,
    /// 完了日時
    pub completed_at: Option<DateTime<Utc>>,
    /// 担当者
    pub assigned_to: Option<String>,
}

/// インシデント通知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentNotification {
    /// 通知ID
    pub id: String,
    /// インシデントID
    pub incident_id: String,
    /// タイトル
    pub title: String,
    /// メッセージ
    pub message: String,
    /// 送信先
    pub recipients: Vec<String>,
    /// 送信日時
    pub sent_at: DateTime<Utc>,
    /// 送信者
    pub sent_by: String,
    /// 重大度
    pub severity: SeverityLevel,
}

/// インシデント対応マネージャー
pub struct IncidentResponseManager {
    /// インシデント
    incidents: Arc<Mutex<HashMap<String, SecurityIncident>>>,
    /// 対応計画
    response_plans: Arc<Mutex<HashMap<String, IncidentResponsePlan>>>,
    /// 通知履歴
    notifications: Arc<Mutex<Vec<IncidentNotification>>>,
    /// 対応ポリシー
    response_policies: HashMap<SeverityLevel, ResponsePolicy>,
    /// 通知設定
    notification_config: NotificationConfig,
}

/// 対応ポリシー
#[derive(Debug, Clone)]
struct ResponsePolicy {
    /// 重大度
    severity: SeverityLevel,
    /// 対応時間（分）
    response_time_minutes: u32,
    /// エスカレーション時間（分）
    escalation_time_minutes: u32,
    /// 通知先
    notification_recipients: Vec<String>,
    /// 自動対応アクション
    auto_response_actions: Vec<String>,
}

/// 通知設定
#[derive(Debug, Clone)]
struct NotificationConfig {
    /// 通知有効フラグ
    enabled: bool,
    /// 通知チャネル
    channels: Vec<NotificationChannel>,
    /// デフォルト送信者
    default_sender: String,
    /// 通知テンプレート
    templates: HashMap<String, String>,
}

/// 通知チャネル
#[derive(Debug, Clone, PartialEq, Eq)]
enum NotificationChannel {
    /// Eメール
    Email,
    /// SMS
    SMS,
    /// Slack
    Slack,
    /// ログ
    Log,
}

impl IncidentResponseManager {
    /// 新しいIncidentResponseManagerを作成
    pub fn new() -> Self {
        Self {
            incidents: Arc::new(Mutex::new(HashMap::new())),
            response_plans: Arc::new(Mutex::new(HashMap::new())),
            notifications: Arc::new(Mutex::new(Vec::new())),
            response_policies: Self::default_response_policies(),
            notification_config: Self::default_notification_config(),
        }
    }
    
    /// デフォルトの対応ポリシーを作成
    fn default_response_policies() -> HashMap<SeverityLevel, ResponsePolicy> {
        let mut policies = HashMap::new();
        
        // 重大度ごとのポリシーを設定
        policies.insert(SeverityLevel::Critical, ResponsePolicy {
            severity: SeverityLevel::Critical,
            response_time_minutes: 15,
            escalation_time_minutes: 30,
            notification_recipients: vec![
                "security-team@example.com".to_string(),
                "ciso@example.com".to_string(),
            ],
            auto_response_actions: vec![
                "isolate_affected_system".to_string(),
                "backup_logs".to_string(),
                "notify_security_team".to_string(),
            ],
        });
        
        policies.insert(SeverityLevel::High, ResponsePolicy {
            severity: SeverityLevel::High,
            response_time_minutes: 60,
            escalation_time_minutes: 120,
            notification_recipients: vec![
                "security-team@example.com".to_string(),
            ],
            auto_response_actions: vec![
                "backup_logs".to_string(),
                "notify_security_team".to_string(),
            ],
        });
        
        policies.insert(SeverityLevel::Medium, ResponsePolicy {
            severity: SeverityLevel::Medium,
            response_time_minutes: 240,
            escalation_time_minutes: 480,
            notification_recipients: vec![
                "security-alerts@example.com".to_string(),
            ],
            auto_response_actions: vec![
                "log_incident".to_string(),
            ],
        });
        
        policies.insert(SeverityLevel::Low, ResponsePolicy {
            severity: SeverityLevel::Low,
            response_time_minutes: 1440, // 24時間
            escalation_time_minutes: 2880, // 48時間
            notification_recipients: vec![
                "security-alerts@example.com".to_string(),
            ],
            auto_response_actions: vec![
                "log_incident".to_string(),
            ],
        });
        
        policies.insert(SeverityLevel::Info, ResponsePolicy {
            severity: SeverityLevel::Info,
            response_time_minutes: 10080, // 1週間
            escalation_time_minutes: 20160, // 2週間
            notification_recipients: vec![],
            auto_response_actions: vec![
                "log_incident".to_string(),
            ],
        });
        
        policies
    }
    
    /// デフォルトの通知設定を作成
    fn default_notification_config() -> NotificationConfig {
        let mut templates = HashMap::new();
        
        // 通知テンプレートを設定
        templates.insert(
            "incident_created".to_string(),
            "新しいセキュリティインシデントが作成されました: {{title}} (ID: {{id}})".to_string(),
        );
        
        templates.insert(
            "incident_updated".to_string(),
            "セキュリティインシデントが更新されました: {{title}} (ID: {{id}})".to_string(),
        );
        
        templates.insert(
            "incident_resolved".to_string(),
            "セキュリティインシデントが解決されました: {{title}} (ID: {{id}})".to_string(),
        );
        
        NotificationConfig {
            enabled: true,
            channels: vec![NotificationChannel::Log],
            default_sender: "security-system@example.com".to_string(),
            templates,
        }
    }
    
    /// インシデントを作成
    pub fn create_incident(&self, incident: SecurityIncident) -> Result<(), Error> {
        // インシデントを保存
        let mut incidents = self.incidents.lock().unwrap();
        
        // 既に存在するか確認
        if incidents.contains_key(&incident.id) {
            return Err(Error::DuplicateEntry(format!("Incident already exists: {}", incident.id)));
        }
        
        incidents.insert(incident.id.clone(), incident.clone());
        
        // 対応ポリシーに基づいて自動アクションを実行
        if let Some(policy) = self.response_policies.get(&incident.severity) {
            self.execute_auto_response_actions(&incident, policy)?;
        }
        
        // 通知を送信
        self.send_incident_notification(&incident, "incident_created")?;
        
        // 対応計画を作成
        self.create_response_plan(&incident)?;
        
        info!("Security incident created: {} ({})", incident.title, incident.id);
        
        Ok(())
    }
    
    /// インシデントを更新
    pub fn update_incident(&self, incident_id: &str, updates: IncidentUpdates) -> Result<(), Error> {
        let mut incidents = self.incidents.lock().unwrap();
        
        // インシデントを取得
        let incident = incidents.get_mut(incident_id)
            .ok_or_else(|| Error::NotFound(format!("Incident not found: {}", incident_id)))?;
        
        // 更新を適用
        if let Some(title) = updates.title {
            incident.title = title;
        }
        
        if let Some(description) = updates.description {
            incident.description = description;
        }
        
        if let Some(severity) = updates.severity {
            incident.severity = severity;
        }
        
        if let Some(status) = updates.status {
            incident.status = status;
        }
        
        if let Some(assigned_to) = updates.assigned_to {
            incident.assigned_to = Some(assigned_to);
        }
        
        // 更新日時を更新
        incident.updated_at = Utc::now();
        
        // 通知を送信
        self.send_incident_notification(incident, "incident_updated")?;
        
        info!("Security incident updated: {} ({})", incident.title, incident.id);
        
        Ok(())
    }
    
    /// インシデントを解決
    pub fn resolve_incident(&self, incident_id: &str, resolution: &str) -> Result<(), Error> {
        let mut incidents = self.incidents.lock().unwrap();
        
        // インシデントを取得
        let incident = incidents.get_mut(incident_id)
            .ok_or_else(|| Error::NotFound(format!("Incident not found: {}", incident_id)))?;
        
        // 解決済みに更新
        incident.status = IncidentStatus::Resolved;
        incident.resolution = Some(resolution.to_string());
        incident.updated_at = Utc::now();
        
        // 通知を送信
        self.send_incident_notification(incident, "incident_resolved")?;
        
        info!("Security incident resolved: {} ({})", incident.title, incident.id);
        
        Ok(())
    }
    
    /// インシデントを取得
    pub fn get_incident(&self, incident_id: &str) -> Result<SecurityIncident, Error> {
        let incidents = self.incidents.lock().unwrap();
        
        // インシデントを取得
        incidents.get(incident_id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Incident not found: {}", incident_id)))
    }
    
    /// すべてのインシデントを取得
    pub fn get_all_incidents(&self) -> Vec<SecurityIncident> {
        let incidents = self.incidents.lock().unwrap();
        incidents.values().cloned().collect()
    }
    
    /// 対応計画を作成
    fn create_response_plan(&self, incident: &SecurityIncident) -> Result<(), Error> {
        // 対応計画を作成
        let plan_id = format!("PLAN-{}", Uuid::new_v4());
        
        let steps = self.generate_response_steps(incident);
        
        let plan = IncidentResponsePlan {
            id: plan_id.clone(),
            incident_id: incident.id.clone(),
            name: format!("Response Plan for {}", incident.title),
            description: format!("Incident response plan for {}", incident.title),
            steps,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: "system".to_string(),
        };
        
        // 対応計画を保存
        let mut response_plans = self.response_plans.lock().unwrap();
        response_plans.insert(plan_id, plan);
        
        Ok(())
    }
    
    /// 対応ステップを生成
    fn generate_response_steps(&self, incident: &SecurityIncident) -> Vec<ResponseStep> {
        // 重大度に応じたステップを生成
        let mut steps = Vec::new();
        
        // 共通ステップ
        steps.push(ResponseStep {
            id: format!("STEP-{}", Uuid::new_v4()),
            step_number: 1,
            title: "インシデントの初期評価".to_string(),
            description: "インシデントの範囲と影響を評価します。".to_string(),
            completed: false,
            completed_at: None,
            assigned_to: None,
        });
        
        steps.push(ResponseStep {
            id: format!("STEP-{}", Uuid::new_v4()),
            step_number: 2,
            title: "証拠の収集".to_string(),
            description: "ログファイルやその他の証拠を収集します。".to_string(),
            completed: false,
            completed_at: None,
            assigned_to: None,
        });
        
        // 重大度に応じた追加ステップ
        match incident.severity {
            SeverityLevel::Critical | SeverityLevel::High => {
                steps.push(ResponseStep {
                    id: format!("STEP-{}", Uuid::new_v4()),
                    step_number: 3,
                    title: "影響を受けるシステムの隔離".to_string(),
                    description: "影響を受けるシステムをネットワークから隔離します。".to_string(),
                    completed: false,
                    completed_at: None,
                    assigned_to: None,
                });
                
                steps.push(ResponseStep {
                    id: format!("STEP-{}", Uuid::new_v4()),
                    step_number: 4,
                    title: "フォレンジック分析".to_string(),
                    description: "詳細なフォレンジック分析を実施します。".to_string(),
                    completed: false,
                    completed_at: None,
                    assigned_to: None,
                });
            },
            _ => {}
        }
        
        // 最終ステップ
        steps.push(ResponseStep {
            id: format!("STEP-{}", Uuid::new_v4()),
            step_number: steps.len() as u32 + 1,
            title: "修復と復旧".to_string(),
            description: "影響を受けるシステムを修復し、通常運用に復旧します。".to_string(),
            completed: false,
            completed_at: None,
            assigned_to: None,
        });
        
        steps.push(ResponseStep {
            id: format!("STEP-{}", Uuid::new_v4()),
            step_number: steps.len() as u32 + 1,
            title: "事後分析と報告".to_string(),
            description: "インシデントの事後分析を行い、報告書を作成します。".to_string(),
            completed: false,
            completed_at: None,
            assigned_to: None,
        });
        
        steps
    }
    
    /// 対応計画を取得
    pub fn get_response_plan(&self, incident_id: &str) -> Result<IncidentResponsePlan, Error> {
        let response_plans = self.response_plans.lock().unwrap();
        
        // インシデントIDに関連する対応計画を検索
        for plan in response_plans.values() {
            if plan.incident_id == incident_id {
                return Ok(plan.clone());
            }
        }
        
        Err(Error::NotFound(format!("Response plan not found for incident: {}", incident_id)))
    }
    
    /// 対応ステップを更新
    pub fn update_response_step(
        &self,
        plan_id: &str,
        step_id: &str,
        completed: bool,
        assigned_to: Option<String>,
    ) -> Result<(), Error> {
        let mut response_plans = self.response_plans.lock().unwrap();
        
        // 対応計画を取得
        let plan = response_plans.get_mut(plan_id)
            .ok_or_else(|| Error::NotFound(format!("Response plan not found: {}", plan_id)))?;
        
        // ステップを検索
        let step = plan.steps.iter_mut()
            .find(|s| s.id == step_id)
            .ok_or_else(|| Error::NotFound(format!("Response step not found: {}", step_id)))?;
        
        // ステップを更新
        step.completed = completed;
        if completed {
            step.completed_at = Some(Utc::now());
        } else {
            step.completed_at = None;
        }
        
        if let Some(assignee) = assigned_to {
            step.assigned_to = Some(assignee);
        }
        
        // 計画の更新日時を更新
        plan.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// 自動対応アクションを実行
    fn execute_auto_response_actions(
        &self,
        incident: &SecurityIncident,
        policy: &ResponsePolicy,
    ) -> Result<(), Error> {
        for action in &policy.auto_response_actions {
            match action.as_str() {
                "isolate_affected_system" => {
                    // 影響を受けるシステムを隔離
                    info!("Auto-action: Isolating affected system for incident {}", incident.id);
                    // 実際の実装では、システム隔離のロジックを実装
                },
                "backup_logs" => {
                    // ログをバックアップ
                    info!("Auto-action: Backing up logs for incident {}", incident.id);
                    // 実際の実装では、ログバックアップのロジックを実装
                },
                "notify_security_team" => {
                    // セキュリティチームに通知
                    info!("Auto-action: Notifying security team for incident {}", incident.id);
                    // 実際の実装では、通知のロジックを実装
                },
                "log_incident" => {
                    // インシデントをログに記録
                    info!("Auto-action: Logging incident {}", incident.id);
                    // 実際の実装では、ログ記録のロジックを実装
                },
                _ => {
                    warn!("Unknown auto-response action: {}", action);
                }
            }
        }
        
        Ok(())
    }
    
    /// インシデント通知を送信
    fn send_incident_notification(
        &self,
        incident: &SecurityIncident,
        template_key: &str,
    ) -> Result<(), Error> {
        // 通知が無効の場合は何もしない
        if !self.notification_config.enabled {
            return Ok(());
        }
        
        // 通知テンプレートを取得
        let template = self.notification_config.templates.get(template_key)
            .cloned()
            .unwrap_or_else(|| format!("Security incident update: {} (ID: {})", incident.title, incident.id));
        
        // テンプレート変数を置換
        let message = template
            .replace("{{id}}", &incident.id)
            .replace("{{title}}", &incident.title);
        
        // 受信者を決定
        let recipients = if let Some(policy) = self.response_policies.get(&incident.severity) {
            policy.notification_recipients.clone()
        } else {
            Vec::new()
        };
        
        // 通知を作成
        let notification = IncidentNotification {
            id: format!("NOTIF-{}", Uuid::new_v4()),
            incident_id: incident.id.clone(),
            title: format!("Security Incident: {}", incident.title),
            message,
            recipients: recipients.clone(),
            sent_at: Utc::now(),
            sent_by: self.notification_config.default_sender.clone(),
            severity: incident.severity.clone(),
        };
        
        // 通知を保存
        let mut notifications = self.notifications.lock().unwrap();
        notifications.push(notification.clone());
        
        // 通知チャネルごとに送信
        for channel in &self.notification_config.channels {
            match channel {
                NotificationChannel::Email => {
                    // Eメールで通知
                    info!("Sending email notification for incident {} to {} recipients", incident.id, recipients.len());
                    // 実際の実装では、Eメール送信のロジックを実装
                },
                NotificationChannel::SMS => {
                    // SMSで通知
                    info!("Sending SMS notification for incident {} to {} recipients", incident.id, recipients.len());
                    // 実際の実装では、SMS送信のロジックを実装
                },
                NotificationChannel::Slack => {
                    // Slackで通知
                    info!("Sending Slack notification for incident {} to {} recipients", incident.id, recipients.len());
                    // 実際の実装では、Slack通知のロジックを実装
                },
                NotificationChannel::Log => {
                    // ログに記録
                    info!("Incident notification: {} - {}", notification.title, notification.message);
                },
            }
        }
        
        Ok(())
    }
    
    /// 通知設定を更新
    pub fn update_notification_config(
        &mut self,
        enabled: Option<bool>,
        channels: Option<Vec<String>>,
        default_sender: Option<String>,
    ) -> Result<(), Error> {
        if let Some(enabled_value) = enabled {
            self.notification_config.enabled = enabled_value;
        }
        
        if let Some(channels_value) = channels {
            let mut new_channels = Vec::new();
            
            for channel in channels_value {
                match channel.to_lowercase().as_str() {
                    "email" => new_channels.push(NotificationChannel::Email),
                    "sms" => new_channels.push(NotificationChannel::SMS),
                    "slack" => new_channels.push(NotificationChannel::Slack),
                    "log" => new_channels.push(NotificationChannel::Log),
                    _ => return Err(Error::InvalidArgument(format!("Unknown notification channel: {}", channel))),
                }
            }
            
            self.notification_config.channels = new_channels;
        }
        
        if let Some(sender) = default_sender {
            self.notification_config.default_sender = sender;
        }
        
        Ok(())
    }
    
    /// 対応ポリシーを更新
    pub fn update_response_policy(
        &mut self,
        severity: SeverityLevel,
        response_time_minutes: Option<u32>,
        escalation_time_minutes: Option<u32>,
        notification_recipients: Option<Vec<String>>,
        auto_response_actions: Option<Vec<String>>,
    ) -> Result<(), Error> {
        // 既存のポリシーを取得または新規作成
        let policy = self.response_policies.entry(severity.clone()).or_insert_with(|| ResponsePolicy {
            severity: severity.clone(),
            response_time_minutes: 60,
            escalation_time_minutes: 120,
            notification_recipients: Vec::new(),
            auto_response_actions: Vec::new(),
        });
        
        // ポリシーを更新
        if let Some(response_time) = response_time_minutes {
            policy.response_time_minutes = response_time;
        }
        
        if let Some(escalation_time) = escalation_time_minutes {
            policy.escalation_time_minutes = escalation_time;
        }
        
        if let Some(recipients) = notification_recipients {
            policy.notification_recipients = recipients;
        }
        
        if let Some(actions) = auto_response_actions {
            policy.auto_response_actions = actions;
        }
        
        Ok(())
    }
    
    /// インシデント統計を取得
    pub fn get_incident_statistics(&self) -> IncidentStatistics {
        let incidents = self.incidents.lock().unwrap();
        
        let total = incidents.len();
        let mut open = 0;
        let mut investigating = 0;
        let mut responding = 0;
        let mut resolved = 0;
        let mut closed = 0;
        
        let mut critical = 0;
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;
        let mut info = 0;
        
        for incident in incidents.values() {
            // ステータス別カウント
            match incident.status {
                IncidentStatus::Open => open += 1,
                IncidentStatus::Investigating => investigating += 1,
                IncidentStatus::Responding => responding += 1,
                IncidentStatus::Resolved => resolved += 1,
                IncidentStatus::Closed => closed += 1,
            }
            
            // 重大度別カウント
            match incident.severity {
                SeverityLevel::Critical => critical += 1,
                SeverityLevel::High => high += 1,
                SeverityLevel::Medium => medium += 1,
                SeverityLevel::Low => low += 1,
                SeverityLevel::Info => info += 1,
            }
        }
        
        IncidentStatistics {
            total,
            by_status: IncidentStatusStatistics {
                open,
                investigating,
                responding,
                resolved,
                closed,
            },
            by_severity: IncidentSeverityStatistics {
                critical,
                high,
                medium,
                low,
                info,
            },
        }
    }
}

/// インシデント更新情報
#[derive(Debug, Clone)]
pub struct IncidentUpdates {
    /// タイトル
    pub title: Option<String>,
    /// 説明
    pub description: Option<String>,
    /// 重大度
    pub severity: Option<SeverityLevel>,
    /// ステータス
    pub status: Option<IncidentStatus>,
    /// 担当者
    pub assigned_to: Option<String>,
}

/// インシデント統計
#[derive(Debug, Clone)]
pub struct IncidentStatistics {
    /// 合計
    pub total: usize,
    /// ステータス別統計
    pub by_status: IncidentStatusStatistics,
    /// 重大度別統計
    pub by_severity: IncidentSeverityStatistics,
}

/// ステータス別インシデント統計
#[derive(Debug, Clone)]
pub struct IncidentStatusStatistics {
    /// 未対応
    pub open: usize,
    /// 調査中
    pub investigating: usize,
    /// 対応中
    pub responding: usize,
    /// 解決済み
    pub resolved: usize,
    /// クローズ
    pub closed: usize,
}

/// 重大度別インシデント統計
#[derive(Debug, Clone)]
pub struct IncidentSeverityStatistics {
    /// 重大
    pub critical: usize,
    /// 高
    pub high: usize,
    /// 中
    pub medium: usize,
    /// 低
    pub low: usize,
    /// 情報
    pub info: usize,
}

impl Default for IncidentResponseManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_incident_creation_and_resolution() {
        let manager = IncidentResponseManager::new();
        
        // インシデントを作成
        let incident = SecurityIncident {
            id: "INC-TEST-001".to_string(),
            title: "Test Security Incident".to_string(),
            description: "This is a test security incident".to_string(),
            severity: SeverityLevel::High,
            status: IncidentStatus::Open,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            assigned_to: None,
            related_vulnerabilities: Vec::new(),
            resolution: None,
        };
        
        let result = manager.create_incident(incident.clone());
        assert!(result.is_ok());
        
        // インシデントを取得
        let retrieved = manager.get_incident("INC-TEST-001").unwrap();
        assert_eq!(retrieved.id, "INC-TEST-001");
        assert_eq!(retrieved.title, "Test Security Incident");
        assert_eq!(retrieved.status, IncidentStatus::Open);
        
        // 対応計画を取得
        let plan = manager.get_response_plan("INC-TEST-001").unwrap();
        assert_eq!(plan.incident_id, "INC-TEST-001");
        assert!(!plan.steps.is_empty());
        
        // インシデントを解決
        let result = manager.resolve_incident("INC-TEST-001", "Issue has been resolved");
        assert!(result.is_ok());
        
        // 解決済みのインシデントを取得
        let resolved = manager.get_incident("INC-TEST-001").unwrap();
        assert_eq!(resolved.status, IncidentStatus::Resolved);
        assert_eq!(resolved.resolution, Some("Issue has been resolved".to_string()));
    }
    
    #[test]
    fn test_incident_updates() {
        let manager = IncidentResponseManager::new();
        
        // インシデントを作成
        let incident = SecurityIncident {
            id: "INC-TEST-002".to_string(),
            title: "Another Test Incident".to_string(),
            description: "This is another test incident".to_string(),
            severity: SeverityLevel::Medium,
            status: IncidentStatus::Open,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            assigned_to: None,
            related_vulnerabilities: Vec::new(),
            resolution: None,
        };
        
        manager.create_incident(incident).unwrap();
        
        // インシデントを更新
        let updates = IncidentUpdates {
            title: Some("Updated Test Incident".to_string()),
            description: Some("This is an updated test incident".to_string()),
            severity: Some(SeverityLevel::High),
            status: Some(IncidentStatus::Investigating),
            assigned_to: Some("security-team@example.com".to_string()),
        };
        
        let result = manager.update_incident("INC-TEST-002", updates);
        assert!(result.is_ok());
        
        // 更新されたインシデントを取得
        let updated = manager.get_incident("INC-TEST-002").unwrap();
        assert_eq!(updated.title, "Updated Test Incident");
        assert_eq!(updated.description, "This is an updated test incident");
        assert_eq!(updated.severity, SeverityLevel::High);
        assert_eq!(updated.status, IncidentStatus::Investigating);
        assert_eq!(updated.assigned_to, Some("security-team@example.com".to_string()));
    }
    
    #[test]
    fn test_incident_statistics() {
        let manager = IncidentResponseManager::new();
        
        // 複数のインシデントを作成
        let incidents = vec![
            SecurityIncident {
                id: "INC-STAT-001".to_string(),
                title: "Critical Incident".to_string(),
                description: "Critical severity incident".to_string(),
                severity: SeverityLevel::Critical,
                status: IncidentStatus::Open,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                assigned_to: None,
                related_vulnerabilities: Vec::new(),
                resolution: None,
            },
            SecurityIncident {
                id: "INC-STAT-002".to_string(),
                title: "High Incident".to_string(),
                description: "High severity incident".to_string(),
                severity: SeverityLevel::High,
                status: IncidentStatus::Investigating,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                assigned_to: None,
                related_vulnerabilities: Vec::new(),
                resolution: None,
            },
            SecurityIncident {
                id: "INC-STAT-003".to_string(),
                title: "Medium Incident".to_string(),
                description: "Medium severity incident".to_string(),
                severity: SeverityLevel::Medium,
                status: IncidentStatus::Resolved,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                assigned_to: None,
                related_vulnerabilities: Vec::new(),
                resolution: Some("Resolved".to_string()),
            },
        ];
        
        for incident in incidents {
            manager.create_incident(incident).unwrap();
        }
        
        // 統計を取得
        let stats = manager.get_incident_statistics();
        
        // 統計を検証
        assert_eq!(stats.total, 3);
        assert_eq!(stats.by_status.open, 1);
        assert_eq!(stats.by_status.investigating, 1);
        assert_eq!(stats.by_status.resolved, 1);
        assert_eq!(stats.by_severity.critical, 1);
        assert_eq!(stats.by_severity.high, 1);
        assert_eq!(stats.by_severity.medium, 1);
    }
}