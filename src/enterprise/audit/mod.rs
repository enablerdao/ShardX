// 監査モジュール
//
// このモジュールは、ShardXの監査機能を提供します。
// 主な機能:
// - 監査ログ管理
// - 監査イベント追跡
// - 監査レポート生成
// - 監査証跡
// - 監査アラート

mod log;
mod event;
mod report;
mod trail;
mod alert;

pub use self::log::{AuditLog, LogLevel, LogFormat, LogStorage};
pub use self::event::{AuditEvent, AuditEventType, EventSource, EventTarget};
pub use self::report::{AuditReport, ReportFormat, ReportPeriod, ReportGenerator};
pub use self::trail::{AuditTrail, TrailEntry, TrailVerifier};
pub use self::alert::{AuditAlert, AlertType, AlertSeverity, AlertNotifier};

use crate::error::Error;
use crate::enterprise::AuditConfig;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc, Duration};
use log::{debug, error, info, warn};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// 監査マネージャー
pub struct AuditManager {
    /// 監査設定
    config: AuditConfig,
    /// 監査ログ
    logs: HashMap<String, AuditLog>,
    /// 監査イベント
    events: Vec<AuditEvent>,
    /// 監査レポート
    reports: HashMap<String, AuditReport>,
    /// 監査証跡
    trails: HashMap<String, AuditTrail>,
    /// 監査アラート
    alerts: Vec<AuditAlert>,
    /// イベントタイプフィルター
    event_type_filters: HashMap<AuditEventType, bool>,
    /// ログレベルフィルター
    log_level_filters: HashMap<LogLevel, bool>,
    /// 初期化済みフラグ
    initialized: bool,
}

impl AuditManager {
    /// 新しいAuditManagerを作成
    pub fn new(config: AuditConfig) -> Self {
        let mut manager = Self {
            config,
            logs: HashMap::new(),
            events: Vec::new(),
            reports: HashMap::new(),
            trails: HashMap::new(),
            alerts: Vec::new(),
            event_type_filters: HashMap::new(),
            log_level_filters: HashMap::new(),
            initialized: true,
        };
        
        // デフォルトのイベントタイプフィルターを設定
        manager.event_type_filters.insert(AuditEventType::Authentication, true);
        manager.event_type_filters.insert(AuditEventType::Authorization, true);
        manager.event_type_filters.insert(AuditEventType::DataAccess, true);
        manager.event_type_filters.insert(AuditEventType::SystemChange, true);
        manager.event_type_filters.insert(AuditEventType::UserActivity, true);
        manager.event_type_filters.insert(AuditEventType::SecurityEvent, true);
        manager.event_type_filters.insert(AuditEventType::ComplianceEvent, true);
        manager.event_type_filters.insert(AuditEventType::ResourceEvent, true);
        manager.event_type_filters.insert(AuditEventType::NetworkEvent, true);
        manager.event_type_filters.insert(AuditEventType::ApplicationEvent, true);
        manager.event_type_filters.insert(AuditEventType::DatabaseEvent, true);
        manager.event_type_filters.insert(AuditEventType::APIEvent, true);
        manager.event_type_filters.insert(AuditEventType::AccessControl, true);
        manager.event_type_filters.insert(AuditEventType::Other, true);
        
        // デフォルトのログレベルフィルターを設定
        manager.log_level_filters.insert(LogLevel::Debug, false);
        manager.log_level_filters.insert(LogLevel::Info, true);
        manager.log_level_filters.insert(LogLevel::Warning, true);
        manager.log_level_filters.insert(LogLevel::Error, true);
        manager.log_level_filters.insert(LogLevel::Critical, true);
        
        // デフォルトの監査ログを作成
        manager.create_default_audit_logs();
        
        manager
    }
    
    /// 初期化済みかどうかを確認
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// デフォルトの監査ログを作成
    fn create_default_audit_logs(&mut self) {
        // システム監査ログ
        self.create_audit_log(
            "system",
            "System Audit Log",
            "System-wide audit log for all system events",
            LogLevel::Info,
            LogFormat::JSON,
            LogStorage::Database,
        ).unwrap_or_else(|e| {
            error!("Failed to create system audit log: {}", e);
            "".to_string()
        });
        
        // セキュリティ監査ログ
        self.create_audit_log(
            "security",
            "Security Audit Log",
            "Security-related events audit log",
            LogLevel::Warning,
            LogFormat::JSON,
            LogStorage::Database,
        ).unwrap_or_else(|e| {
            error!("Failed to create security audit log: {}", e);
            "".to_string()
        });
        
        // ユーザー監査ログ
        self.create_audit_log(
            "user",
            "User Activity Audit Log",
            "User activity and authentication events",
            LogLevel::Info,
            LogFormat::JSON,
            LogStorage::Database,
        ).unwrap_or_else(|e| {
            error!("Failed to create user audit log: {}", e);
            "".to_string()
        });
        
        // データアクセス監査ログ
        self.create_audit_log(
            "data",
            "Data Access Audit Log",
            "Data access and modification events",
            LogLevel::Info,
            LogFormat::JSON,
            LogStorage::Database,
        ).unwrap_or_else(|e| {
            error!("Failed to create data audit log: {}", e);
            "".to_string()
        });
        
        // コンプライアンス監査ログ
        self.create_audit_log(
            "compliance",
            "Compliance Audit Log",
            "Compliance-related events and activities",
            LogLevel::Info,
            LogFormat::JSON,
            LogStorage::Database,
        ).unwrap_or_else(|e| {
            error!("Failed to create compliance audit log: {}", e);
            "".to_string()
        });
    }
    
    /// 監査ログを作成
    pub fn create_audit_log(
        &mut self,
        name: &str,
        title: &str,
        description: &str,
        level: LogLevel,
        format: LogFormat,
        storage: LogStorage,
    ) -> Result<String, Error> {
        // 監査ログIDを生成
        let log_id = format!("LOG-{}-{}", name, Uuid::new_v4());
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // 監査ログを作成
        let log = AuditLog {
            id: log_id.clone(),
            name: name.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            level,
            format,
            storage,
            created_at: now,
            updated_at: now,
            retention_period: Duration::days(self.config.audit_log_retention_seconds as i64 / 86400),
            enabled: true,
            metadata: HashMap::new(),
        };
        
        // 監査ログを保存
        self.logs.insert(log_id.clone(), log);
        
        info!("Audit log created: {} ({})", name, log_id);
        
        Ok(log_id)
    }
    
    /// イベントを記録
    pub fn log_event(
        &mut self,
        event_type: AuditEventType,
        user_id: &str,
        resource: &str,
        action: &str,
        result: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<String, Error> {
        // 監査ログが有効かチェック
        if !self.config.enable_audit_logging {
            return Err(Error::InvalidState("Audit logging is not enabled".to_string()));
        }
        
        // イベントタイプフィルターをチェック
        if !self.event_type_filters.get(&event_type).cloned().unwrap_or(false) {
            debug!("Event type filtered: {:?}", event_type);
            return Err(Error::InvalidArgument(format!("Event type is filtered: {:?}", event_type)));
        }
        
        // イベントIDを生成
        let event_id = format!("EVENT-{}", Uuid::new_v4());
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // イベントソースを決定
        let source = if user_id.is_empty() || user_id == "system" {
            EventSource::System
        } else {
            EventSource::User(user_id.to_string())
        };
        
        // イベントターゲットを決定
        let target = EventTarget::Resource(resource.to_string());
        
        // イベントを作成
        let event = AuditEvent {
            id: event_id.clone(),
            event_type,
            timestamp: now,
            source,
            target,
            action: action.to_string(),
            result: result.to_string(),
            metadata: metadata.unwrap_or(serde_json::json!({})),
        };
        
        // イベントを保存
        self.events.push(event.clone());
        
        // イベントを監査証跡に追加
        self.add_event_to_trails(&event);
        
        // イベントをログに記録
        self.write_event_to_logs(&event)?;
        
        // イベントをアラートチェック
        self.check_event_for_alerts(&event);
        
        info!("Audit event logged: {} ({}, {})", event_id, event_type.to_string(), action);
        
        Ok(event_id)
    }
    
    /// イベントを監査証跡に追加
    fn add_event_to_trails(&mut self, event: &AuditEvent) {
        // すべての監査証跡にイベントを追加
        for (trail_id, trail) in &mut self.trails {
            // 証跡のフィルターをチェック
            if trail.should_include_event(event) {
                let entry = TrailEntry {
                    id: format!("ENTRY-{}", Uuid::new_v4()),
                    event_id: event.id.clone(),
                    timestamp: event.timestamp,
                    event_type: event.event_type.clone(),
                    source: event.source.clone(),
                    target: event.target.clone(),
                    action: event.action.clone(),
                    result: event.result.clone(),
                    metadata: event.metadata.clone(),
                };
                
                trail.entries.push(entry);
                trail.updated_at = Utc::now();
                
                debug!("Event added to trail: {} ({})", event.id, trail_id);
            }
        }
    }
    
    /// イベントをログに記録
    fn write_event_to_logs(&self, event: &AuditEvent) -> Result<(), Error> {
        // イベントタイプに基づいてログレベルを決定
        let log_level = match event.event_type {
            AuditEventType::SecurityEvent => LogLevel::Warning,
            AuditEventType::Authentication | AuditEventType::Authorization => {
                if event.result == "failed" || event.result == "denied" {
                    LogLevel::Warning
                } else {
                    LogLevel::Info
                }
            },
            AuditEventType::SystemChange => LogLevel::Info,
            _ => LogLevel::Info,
        };
        
        // ログレベルフィルターをチェック
        if !self.log_level_filters.get(&log_level).cloned().unwrap_or(false) {
            debug!("Log level filtered: {:?}", log_level);
            return Ok(());
        }
        
        // イベントタイプに基づいて適切なログを選択
        let log_name = match event.event_type {
            AuditEventType::Authentication | AuditEventType::Authorization | AuditEventType::UserActivity => "user",
            AuditEventType::SecurityEvent => "security",
            AuditEventType::DataAccess => "data",
            AuditEventType::ComplianceEvent => "compliance",
            _ => "system",
        };
        
        // ログにイベントを書き込む（実際の実装では、選択したストレージに書き込む）
        if let Some(log) = self.logs.get(log_name) {
            if log.enabled {
                // ここでは、ログに書き込むことをシミュレート
                debug!("Event written to log: {} ({})", event.id, log.name);
            }
        }
        
        Ok(())
    }
    
    /// イベントをアラートチェック
    fn check_event_for_alerts(&mut self, event: &AuditEvent) {
        // アラートルールをチェック
        let alert_rules = self.get_alert_rules_for_event_type(event.event_type.clone());
        
        for rule in alert_rules {
            // ルールの条件をチェック
            if self.check_alert_rule_condition(&rule, event) {
                // アラートを生成
                let alert_id = format!("ALERT-{}", Uuid::new_v4());
                
                let alert = AuditAlert {
                    id: alert_id.clone(),
                    alert_type: rule.alert_type.clone(),
                    severity: rule.severity.clone(),
                    timestamp: Utc::now(),
                    event_id: event.id.clone(),
                    message: format!("{}: {}", rule.name, rule.description),
                    acknowledged: false,
                    acknowledged_by: None,
                    acknowledged_at: None,
                    metadata: serde_json::json!({
                        "rule_id": rule.id,
                        "event_type": event.event_type.to_string(),
                        "source": format!("{:?}", event.source),
                        "target": format!("{:?}", event.target),
                        "action": event.action,
                        "result": event.result,
                    }),
                };
                
                // アラートを保存
                self.alerts.push(alert.clone());
                
                // アラート通知を送信（実際の実装では、通知を送信）
                info!("Alert generated: {} ({})", alert_id, rule.name);
            }
        }
    }
    
    /// イベントタイプに対するアラートルールを取得
    fn get_alert_rules_for_event_type(&self, event_type: AuditEventType) -> Vec<AlertRule> {
        // 実際の実装では、データベースやファイルからルールを取得
        // ここでは、ハードコードされたルールを返す
        
        let mut rules = Vec::new();
        
        match event_type {
            AuditEventType::Authentication => {
                rules.push(AlertRule {
                    id: "RULE-AUTH-001".to_string(),
                    name: "Failed Authentication".to_string(),
                    description: "Multiple failed authentication attempts".to_string(),
                    event_type: AuditEventType::Authentication,
                    condition: "result == 'failed'".to_string(),
                    alert_type: AlertType::Security,
                    severity: AlertSeverity::High,
                    enabled: true,
                });
            },
            AuditEventType::Authorization => {
                rules.push(AlertRule {
                    id: "RULE-AUTHZ-001".to_string(),
                    name: "Access Denied".to_string(),
                    description: "Access to sensitive resource denied".to_string(),
                    event_type: AuditEventType::Authorization,
                    condition: "result == 'denied'".to_string(),
                    alert_type: AlertType::Security,
                    severity: AlertSeverity::Medium,
                    enabled: true,
                });
            },
            AuditEventType::SecurityEvent => {
                rules.push(AlertRule {
                    id: "RULE-SEC-001".to_string(),
                    name: "Security Violation".to_string(),
                    description: "Security policy violation detected".to_string(),
                    event_type: AuditEventType::SecurityEvent,
                    condition: "true".to_string(),
                    alert_type: AlertType::Security,
                    severity: AlertSeverity::High,
                    enabled: true,
                });
            },
            AuditEventType::SystemChange => {
                rules.push(AlertRule {
                    id: "RULE-SYS-001".to_string(),
                    name: "Critical System Change".to_string(),
                    description: "Critical system configuration changed".to_string(),
                    event_type: AuditEventType::SystemChange,
                    condition: "action.contains('config') || action.contains('setting')".to_string(),
                    alert_type: AlertType::System,
                    severity: AlertSeverity::Medium,
                    enabled: true,
                });
            },
            _ => {}
        }
        
        rules
    }
    
    /// アラートルールの条件をチェック
    fn check_alert_rule_condition(&self, rule: &AlertRule, event: &AuditEvent) -> bool {
        // 実際の実装では、条件式を解析して評価
        // ここでは、簡単な条件チェックを行う
        
        match rule.condition.as_str() {
            "true" => true,
            "result == 'failed'" => event.result == "failed",
            "result == 'denied'" => event.result == "denied",
            condition if condition.contains("action.contains") => {
                let start = condition.find('\'').unwrap_or(0) + 1;
                let end = condition.rfind('\'').unwrap_or(condition.len());
                if start < end {
                    let search_term = &condition[start..end];
                    event.action.contains(search_term)
                } else {
                    false
                }
            },
            _ => false,
        }
    }
    
    /// 監査証跡を作成
    pub fn create_audit_trail(
        &mut self,
        name: &str,
        description: &str,
        event_types: Vec<AuditEventType>,
    ) -> Result<String, Error> {
        // 監査証跡IDを生成
        let trail_id = format!("TRAIL-{}", Uuid::new_v4());
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // 監査証跡を作成
        let trail = AuditTrail {
            id: trail_id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            created_at: now,
            updated_at: now,
            event_types,
            entries: Vec::new(),
            metadata: HashMap::new(),
        };
        
        // 監査証跡を保存
        self.trails.insert(trail_id.clone(), trail);
        
        info!("Audit trail created: {} ({})", name, trail_id);
        
        Ok(trail_id)
    }
    
    /// 監査証跡を取得
    pub fn get_audit_trail(&self, trail_id: &str) -> Result<&AuditTrail, Error> {
        self.trails.get(trail_id)
            .ok_or_else(|| Error::NotFound(format!("Audit trail not found: {}", trail_id)))
    }
    
    /// 監査証跡リストを取得
    pub fn get_audit_trails(&self) -> Vec<&AuditTrail> {
        self.trails.values().collect()
    }
    
    /// 監査証跡エントリーを取得
    pub fn get_trail_entries(
        &self,
        trail_id: &str,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Result<Vec<&TrailEntry>, Error> {
        let trail = self.get_audit_trail(trail_id)?;
        
        let mut entries: Vec<&TrailEntry> = trail.entries.iter()
            .filter(|e| {
                start_time.map_or(true, |t| e.timestamp >= t) &&
                end_time.map_or(true, |t| e.timestamp <= t)
            })
            .collect();
        
        // 時間順にソート
        entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        // 上限を適用
        if let Some(limit) = limit {
            if entries.len() > limit {
                entries.truncate(limit);
            }
        }
        
        Ok(entries)
    }
    
    /// 監査レポートを生成
    pub fn generate_report(
        &mut self,
        name: &str,
        description: &str,
        report_period: ReportPeriod,
        format: ReportFormat,
        event_types: Option<Vec<AuditEventType>>,
    ) -> Result<String, Error> {
        // レポートIDを生成
        let report_id = format!("REPORT-{}", Uuid::new_v4());
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // 期間の開始時刻と終了時刻を計算
        let (start_time, end_time) = match report_period {
            ReportPeriod::Daily => (
                now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc() - Duration::days(1),
                now.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc() - Duration::days(1),
            ),
            ReportPeriod::Weekly => (
                now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc() - Duration::weeks(1),
                now,
            ),
            ReportPeriod::Monthly => (
                now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc() - Duration::days(30),
                now,
            ),
            ReportPeriod::Custom(start, end) => (start, end),
        };
        
        // イベントを収集
        let events: Vec<&AuditEvent> = self.events.iter()
            .filter(|e| {
                e.timestamp >= start_time &&
                e.timestamp <= end_time &&
                event_types.as_ref().map_or(true, |types| types.contains(&e.event_type))
            })
            .collect();
        
        // イベントタイプごとの集計
        let mut event_type_counts = HashMap::new();
        for event in &events {
            *event_type_counts.entry(event.event_type.clone()).or_insert(0) += 1;
        }
        
        // 結果ごとの集計
        let mut result_counts = HashMap::new();
        for event in &events {
            *result_counts.entry(event.result.clone()).or_insert(0) += 1;
        }
        
        // レポートコンテンツを生成
        let content = match format {
            ReportFormat::JSON => {
                serde_json::json!({
                    "report_id": report_id,
                    "name": name,
                    "description": description,
                    "period": {
                        "start": start_time,
                        "end": end_time,
                    },
                    "summary": {
                        "total_events": events.len(),
                        "event_types": event_type_counts,
                        "results": result_counts,
                    },
                    "events": events.iter().map(|e| {
                        serde_json::json!({
                            "id": e.id,
                            "event_type": e.event_type.to_string(),
                            "timestamp": e.timestamp,
                            "source": format!("{:?}", e.source),
                            "target": format!("{:?}", e.target),
                            "action": e.action,
                            "result": e.result,
                        })
                    }).collect::<Vec<_>>(),
                })
            },
            ReportFormat::CSV => {
                let mut csv = "id,event_type,timestamp,source,target,action,result\n".to_string();
                for event in &events {
                    csv.push_str(&format!(
                        "{},{},{},{:?},{:?},{},{}\n",
                        event.id,
                        event.event_type.to_string(),
                        event.timestamp,
                        event.source,
                        event.target,
                        event.action,
                        event.result,
                    ));
                }
                serde_json::json!(csv)
            },
            ReportFormat::HTML => {
                let html = format!(
                    r#"<!DOCTYPE html>
<html>
<head>
    <title>Audit Report: {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        h1 {{ color: #333; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
        tr:nth-child(even) {{ background-color: #f9f9f9; }}
    </style>
</head>
<body>
    <h1>Audit Report: {}</h1>
    <p>{}</p>
    <h2>Summary</h2>
    <p>Period: {} to {}</p>
    <p>Total Events: {}</p>
    <h3>Event Types</h3>
    <table>
        <tr><th>Event Type</th><th>Count</th></tr>
        {}
    </table>
    <h3>Results</h3>
    <table>
        <tr><th>Result</th><th>Count</th></tr>
        {}
    </table>
    <h2>Events</h2>
    <table>
        <tr><th>ID</th><th>Event Type</th><th>Timestamp</th><th>Source</th><th>Target</th><th>Action</th><th>Result</th></tr>
        {}
    </table>
</body>
</html>"#,
                    name,
                    name,
                    description,
                    start_time,
                    end_time,
                    events.len(),
                    event_type_counts.iter().map(|(k, v)| format!("<tr><td>{}</td><td>{}</td></tr>", k.to_string(), v)).collect::<Vec<_>>().join(""),
                    result_counts.iter().map(|(k, v)| format!("<tr><td>{}</td><td>{}</td></tr>", k, v)).collect::<Vec<_>>().join(""),
                    events.iter().map(|e| format!(
                        "<tr><td>{}</td><td>{}</td><td>{}</td><td>{:?}</td><td>{:?}</td><td>{}</td><td>{}</td></tr>",
                        e.id,
                        e.event_type.to_string(),
                        e.timestamp,
                        e.source,
                        e.target,
                        e.action,
                        e.result,
                    )).collect::<Vec<_>>().join(""),
                );
                serde_json::json!(html)
            },
            ReportFormat::PDF => {
                // 実際の実装では、PDFを生成
                // ここでは、ダミーのPDFコンテンツを返す
                serde_json::json!("PDF content would be generated here")
            },
        };
        
        // レポートを作成
        let report = AuditReport {
            id: report_id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            created_at: now,
            period: report_period,
            format,
            content,
            metadata: HashMap::new(),
        };
        
        // レポートを保存
        self.reports.insert(report_id.clone(), report);
        
        info!("Audit report generated: {} ({})", name, report_id);
        
        Ok(report_id)
    }
    
    /// 監査レポートを取得
    pub fn get_report(&self, report_id: &str) -> Result<&AuditReport, Error> {
        self.reports.get(report_id)
            .ok_or_else(|| Error::NotFound(format!("Audit report not found: {}", report_id)))
    }
    
    /// 監査レポートリストを取得
    pub fn get_reports(&self) -> Vec<&AuditReport> {
        self.reports.values().collect()
    }
    
    /// 監査アラートを取得
    pub fn get_alerts(
        &self,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        severity: Option<AlertSeverity>,
        acknowledged: Option<bool>,
    ) -> Vec<&AuditAlert> {
        self.alerts.iter()
            .filter(|a| {
                start_time.map_or(true, |t| a.timestamp >= t) &&
                end_time.map_or(true, |t| a.timestamp <= t) &&
                severity.map_or(true, |s| a.severity == s) &&
                acknowledged.map_or(true, |ack| a.acknowledged == ack)
            })
            .collect()
    }
    
    /// 監査アラートを確認
    pub fn acknowledge_alert(&mut self, alert_id: &str, user_id: &str) -> Result<(), Error> {
        // アラートを検索
        let alert = self.alerts.iter_mut()
            .find(|a| a.id == alert_id)
            .ok_or_else(|| Error::NotFound(format!("Audit alert not found: {}", alert_id)))?;
        
        // アラートを確認
        alert.acknowledged = true;
        alert.acknowledged_by = Some(user_id.to_string());
        alert.acknowledged_at = Some(Utc::now());
        
        info!("Audit alert acknowledged: {} by {}", alert_id, user_id);
        
        Ok(())
    }
    
    /// イベントタイプフィルターを設定
    pub fn set_event_type_filter(&mut self, event_type: AuditEventType, enabled: bool) {
        self.event_type_filters.insert(event_type, enabled);
    }
    
    /// ログレベルフィルターを設定
    pub fn set_log_level_filter(&mut self, level: LogLevel, enabled: bool) {
        self.log_level_filters.insert(level, enabled);
    }
    
    /// 監査ログを有効化/無効化
    pub fn set_audit_log_enabled(&mut self, log_id: &str, enabled: bool) -> Result<(), Error> {
        let log = self.logs.get_mut(log_id)
            .ok_or_else(|| Error::NotFound(format!("Audit log not found: {}", log_id)))?;
        
        log.enabled = enabled;
        log.updated_at = Utc::now();
        
        info!("Audit log {} {}", log_id, if enabled { "enabled" } else { "disabled" });
        
        Ok(())
    }
    
    /// 設定を更新
    pub fn update_config(&mut self, config: AuditConfig) {
        self.config = config;
    }
    
    /// 設定を取得
    pub fn get_config(&self) -> &AuditConfig {
        &self.config
    }
}

/// アラートルール
struct AlertRule {
    /// ルールID
    id: String,
    /// 名前
    name: String,
    /// 説明
    description: String,
    /// イベントタイプ
    event_type: AuditEventType,
    /// 条件
    condition: String,
    /// アラートタイプ
    alert_type: AlertType,
    /// 重大度
    severity: AlertSeverity,
    /// 有効フラグ
    enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audit_manager_initialization() {
        let config = AuditConfig::default();
        let manager = AuditManager::new(config);
        
        assert!(manager.is_initialized());
        
        // デフォルトの監査ログが初期化されていることを確認
        assert!(manager.logs.contains_key("LOG-system"));
        assert!(manager.logs.contains_key("LOG-security"));
        assert!(manager.logs.contains_key("LOG-user"));
        assert!(manager.logs.contains_key("LOG-data"));
        assert!(manager.logs.contains_key("LOG-compliance"));
        
        // イベントタイプフィルターが設定されていることを確認
        assert!(manager.event_type_filters.get(&AuditEventType::Authentication).cloned().unwrap_or(false));
        assert!(manager.event_type_filters.get(&AuditEventType::Authorization).cloned().unwrap_or(false));
        assert!(manager.event_type_filters.get(&AuditEventType::DataAccess).cloned().unwrap_or(false));
        
        // ログレベルフィルターが設定されていることを確認
        assert!(!manager.log_level_filters.get(&LogLevel::Debug).cloned().unwrap_or(false));
        assert!(manager.log_level_filters.get(&LogLevel::Info).cloned().unwrap_or(false));
        assert!(manager.log_level_filters.get(&LogLevel::Warning).cloned().unwrap_or(false));
        assert!(manager.log_level_filters.get(&LogLevel::Error).cloned().unwrap_or(false));
        assert!(manager.log_level_filters.get(&LogLevel::Critical).cloned().unwrap_or(false));
    }
    
    #[test]
    fn test_audit_log_creation() {
        let config = AuditConfig::default();
        let mut manager = AuditManager::new(config);
        
        // 監査ログを作成
        let log_id = manager.create_audit_log(
            "test",
            "Test Audit Log",
            "Test audit log for unit testing",
            LogLevel::Info,
            LogFormat::JSON,
            LogStorage::Database,
        ).unwrap();
        
        // 監査ログが作成されたことを確認
        assert!(manager.logs.contains_key(&log_id));
        
        let log = manager.logs.get(&log_id).unwrap();
        assert_eq!(log.name, "test");
        assert_eq!(log.title, "Test Audit Log");
        assert_eq!(log.level, LogLevel::Info);
        assert_eq!(log.format, LogFormat::JSON);
        assert_eq!(log.storage, LogStorage::Database);
        assert!(log.enabled);
    }
    
    #[test]
    fn test_event_logging() {
        let config = AuditConfig::default();
        let mut manager = AuditManager::new(config);
        
        // イベントを記録
        let event_id = manager.log_event(
            AuditEventType::Authentication,
            "user123",
            "login-service",
            "login",
            "success",
            Some(serde_json::json!({
                "ip_address": "192.168.1.1",
                "user_agent": "Mozilla/5.0",
            })),
        ).unwrap();
        
        // イベントが記録されたことを確認
        assert_eq!(manager.events.len(), 1);
        
        let event = &manager.events[0];
        assert_eq!(event.id, event_id);
        assert_eq!(event.event_type, AuditEventType::Authentication);
        assert_eq!(event.action, "login");
        assert_eq!(event.result, "success");
        
        match &event.source {
            EventSource::User(user_id) => assert_eq!(user_id, "user123"),
            _ => panic!("Expected User source"),
        }
        
        match &event.target {
            EventTarget::Resource(resource) => assert_eq!(resource, "login-service"),
            _ => panic!("Expected Resource target"),
        }
        
        // メタデータをチェック
        let metadata = &event.metadata;
        assert_eq!(metadata["ip_address"], "192.168.1.1");
        assert_eq!(metadata["user_agent"], "Mozilla/5.0");
    }
    
    #[test]
    fn test_audit_trail() {
        let config = AuditConfig::default();
        let mut manager = AuditManager::new(config);
        
        // 監査証跡を作成
        let trail_id = manager.create_audit_trail(
            "Authentication Trail",
            "Trail of all authentication events",
            vec![AuditEventType::Authentication],
        ).unwrap();
        
        // 監査証跡が作成されたことを確認
        let trail = manager.get_audit_trail(&trail_id).unwrap();
        assert_eq!(trail.name, "Authentication Trail");
        assert_eq!(trail.event_types, vec![AuditEventType::Authentication]);
        assert!(trail.entries.is_empty());
        
        // 認証イベントを記録
        manager.log_event(
            AuditEventType::Authentication,
            "user123",
            "login-service",
            "login",
            "success",
            None,
        ).unwrap();
        
        // 別のタイプのイベントを記録
        manager.log_event(
            AuditEventType::DataAccess,
            "user123",
            "database",
            "read",
            "success",
            None,
        ).unwrap();
        
        // 監査証跡にエントリーが追加されたことを確認
        let trail = manager.get_audit_trail(&trail_id).unwrap();
        assert_eq!(trail.entries.len(), 1); // 認証イベントのみが追加されるべき
        
        let entry = &trail.entries[0];
        assert_eq!(entry.event_type, AuditEventType::Authentication);
        assert_eq!(entry.action, "login");
        assert_eq!(entry.result, "success");
    }
    
    #[test]
    fn test_report_generation() {
        let config = AuditConfig::default();
        let mut manager = AuditManager::new(config);
        
        // いくつかのイベントを記録
        for i in 0..10 {
            let event_type = if i % 2 == 0 {
                AuditEventType::Authentication
            } else {
                AuditEventType::DataAccess
            };
            
            let result = if i % 3 == 0 { "failed" } else { "success" };
            
            manager.log_event(
                event_type,
                &format!("user{}", i % 3 + 1),
                "test-service",
                &format!("action{}", i),
                result,
                None,
            ).unwrap();
        }
        
        // レポートを生成
        let report_id = manager.generate_report(
            "Test Report",
            "Test audit report",
            ReportPeriod::Daily,
            ReportFormat::JSON,
            Some(vec![AuditEventType::Authentication]),
        ).unwrap();
        
        // レポートが生成されたことを確認
        let report = manager.get_report(&report_id).unwrap();
        assert_eq!(report.name, "Test Report");
        assert_eq!(report.format, ReportFormat::JSON);
        
        // レポートコンテンツをチェック
        let content = &report.content;
        let summary = &content["summary"];
        
        // 認証イベントのみがレポートに含まれることを確認
        assert_eq!(summary["total_events"], 5); // 10イベント中5つが認証イベント
        
        // イベントタイプの集計をチェック
        let event_types = &summary["event_types"];
        assert_eq!(event_types["Authentication"], 5);
        
        // 結果の集計をチェック
        let results = &summary["results"];
        assert!(results.get("success").is_some());
        assert!(results.get("failed").is_some());
    }
    
    #[test]
    fn test_alert_generation() {
        let config = AuditConfig::default();
        let mut manager = AuditManager::new(config);
        
        // 失敗した認証イベントを記録（アラートをトリガー）
        manager.log_event(
            AuditEventType::Authentication,
            "user123",
            "login-service",
            "login",
            "failed",
            None,
        ).unwrap();
        
        // アラートが生成されたことを確認
        let alerts = manager.get_alerts(None, None, None, Some(false));
        assert!(!alerts.is_empty());
        
        let alert = alerts[0];
        assert_eq!(alert.alert_type, AlertType::Security);
        assert_eq!(alert.severity, AlertSeverity::High);
        assert!(!alert.acknowledged);
        
        // アラートを確認
        manager.acknowledge_alert(&alert.id, "admin").unwrap();
        
        // アラートが確認されたことを確認
        let acknowledged_alerts = manager.get_alerts(None, None, None, Some(true));
        assert!(!acknowledged_alerts.is_empty());
        
        let acknowledged_alert = acknowledged_alerts[0];
        assert!(acknowledged_alert.acknowledged);
        assert_eq!(acknowledged_alert.acknowledged_by, Some("admin".to_string()));
        assert!(acknowledged_alert.acknowledged_at.is_some());
    }
}