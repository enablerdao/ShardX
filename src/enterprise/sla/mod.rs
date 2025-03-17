// SLAモジュール
//
// このモジュールは、ShardXのSLA（Service Level Agreement）管理機能を提供します。
// 主な機能:
// - SLA定義と管理
// - SLAメトリクス監視
// - SLAレポート生成
// - SLA違反アラート
// - サポートチケット管理

mod agreement;
mod metric;
mod report;
mod alert;
mod support;

pub use self::agreement::{ServiceLevelAgreement, SLALevel, SLATier, SLAStatus};
pub use self::metric::{SLAMetric, MetricType, MetricValue, MetricThreshold};
pub use self::report::{SLAReport, ReportPeriod, ReportFormat, ComplianceStatus};
pub use self::alert::{SLAAlert, AlertType, AlertSeverity, AlertStatus};
pub use self::support::{SupportTicket, TicketPriority, TicketStatus, TicketCategory};

use crate::error::Error;
use crate::enterprise::SLAConfig;
use crate::metrics::MetricsCollector;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc, Duration};
use log::{debug, error, info, warn};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// SLAマネージャー
pub struct SLAManager {
    /// SLA設定
    config: SLAConfig,
    /// SLA
    agreements: HashMap<String, ServiceLevelAgreement>,
    /// メトリクス
    metrics: HashMap<String, SLAMetric>,
    /// メトリクス履歴
    metric_history: HashMap<String, Vec<MetricValue>>,
    /// レポート
    reports: HashMap<String, SLAReport>,
    /// アラート
    alerts: Vec<SLAAlert>,
    /// サポートチケット
    tickets: HashMap<String, SupportTicket>,
    /// メトリクスコレクター
    metrics_collector: Arc<MetricsCollector>,
    /// 初期化済みフラグ
    initialized: bool,
}

impl SLAManager {
    /// 新しいSLAManagerを作成
    pub fn new(config: SLAConfig) -> Self {
        let mut manager = Self {
            config,
            agreements: HashMap::new(),
            metrics: HashMap::new(),
            metric_history: HashMap::new(),
            reports: HashMap::new(),
            alerts: Vec::new(),
            tickets: HashMap::new(),
            metrics_collector: Arc::new(MetricsCollector::new("sla")),
            initialized: true,
        };
        
        // デフォルトのSLAメトリクスを初期化
        manager.initialize_default_metrics();
        
        // デフォルトのSLAを初期化
        manager.initialize_default_agreements();
        
        manager
    }
    
    /// 初期化済みかどうかを確認
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// デフォルトのSLAメトリクスを初期化
    fn initialize_default_metrics(&mut self) {
        // 可用性メトリクス
        self.create_metric(
            "availability",
            "System Availability",
            "Percentage of time the system is operational",
            MetricType::Percentage,
            99.9,
            "%",
        ).unwrap_or_else(|e| {
            error!("Failed to create availability metric: {}", e);
            "".to_string()
        });
        
        // レスポンスタイムメトリクス
        self.create_metric(
            "response_time",
            "Response Time",
            "Average response time for API requests",
            MetricType::Duration,
            100.0,
            "ms",
        ).unwrap_or_else(|e| {
            error!("Failed to create response time metric: {}", e);
            "".to_string()
        });
        
        // スループットメトリクス
        self.create_metric(
            "throughput",
            "Throughput",
            "Number of transactions processed per second",
            MetricType::Count,
            1000.0,
            "tps",
        ).unwrap_or_else(|e| {
            error!("Failed to create throughput metric: {}", e);
            "".to_string()
        });
        
        // エラー率メトリクス
        self.create_metric(
            "error_rate",
            "Error Rate",
            "Percentage of failed requests",
            MetricType::Percentage,
            0.1,
            "%",
        ).unwrap_or_else(|e| {
            error!("Failed to create error rate metric: {}", e);
            "".to_string()
        });
        
        // データ整合性メトリクス
        self.create_metric(
            "data_integrity",
            "Data Integrity",
            "Percentage of data integrity checks passed",
            MetricType::Percentage,
            99.999,
            "%",
        ).unwrap_or_else(|e| {
            error!("Failed to create data integrity metric: {}", e);
            "".to_string()
        });
    }
    
    /// デフォルトのSLAを初期化
    fn initialize_default_agreements(&mut self) {
        // スタンダードSLA
        self.create_agreement(
            "standard",
            "Standard SLA",
            "Standard service level agreement for all customers",
            SLATier::Standard,
            vec![
                ("availability", 99.9),
                ("response_time", 100.0),
                ("throughput", 1000.0),
                ("error_rate", 0.1),
                ("data_integrity", 99.999),
            ],
            8, // 営業時間内サポート
            24, // 24時間以内の初回応答
        ).unwrap_or_else(|e| {
            error!("Failed to create standard SLA: {}", e);
            "".to_string()
        });
        
        // プレミアムSLA
        self.create_agreement(
            "premium",
            "Premium SLA",
            "Premium service level agreement for enterprise customers",
            SLATier::Premium,
            vec![
                ("availability", 99.99),
                ("response_time", 50.0),
                ("throughput", 2000.0),
                ("error_rate", 0.05),
                ("data_integrity", 99.9999),
            ],
            24, // 24時間サポート
            4, // 4時間以内の初回応答
        ).unwrap_or_else(|e| {
            error!("Failed to create premium SLA: {}", e);
            "".to_string()
        });
        
        // エンタープライズSLA
        self.create_agreement(
            "enterprise",
            "Enterprise SLA",
            "Enterprise service level agreement for mission-critical deployments",
            SLATier::Enterprise,
            vec![
                ("availability", 99.999),
                ("response_time", 20.0),
                ("throughput", 5000.0),
                ("error_rate", 0.01),
                ("data_integrity", 99.99999),
            ],
            24, // 24時間サポート
            1, // 1時間以内の初回応答
        ).unwrap_or_else(|e| {
            error!("Failed to create enterprise SLA: {}", e);
            "".to_string()
        });
    }
    
    /// メトリクスを作成
    pub fn create_metric(
        &mut self,
        name: &str,
        title: &str,
        description: &str,
        metric_type: MetricType,
        target_value: f64,
        unit: &str,
    ) -> Result<String, Error> {
        // メトリクスIDを生成
        let metric_id = format!("METRIC-{}", name);
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // しきい値を設定
        let threshold = match metric_type {
            MetricType::Percentage => {
                MetricThreshold {
                    warning: target_value - 0.5,
                    critical: target_value - 1.0,
                    comparison: if name == "error_rate" { "lt" } else { "gt" }.to_string(),
                }
            },
            MetricType::Duration => {
                MetricThreshold {
                    warning: target_value * 1.5,
                    critical: target_value * 2.0,
                    comparison: "lt".to_string(),
                }
            },
            MetricType::Count => {
                MetricThreshold {
                    warning: target_value * 0.8,
                    critical: target_value * 0.6,
                    comparison: "gt".to_string(),
                }
            },
            MetricType::Size => {
                MetricThreshold {
                    warning: target_value * 0.8,
                    critical: target_value * 0.6,
                    comparison: "gt".to_string(),
                }
            },
            MetricType::Custom => {
                MetricThreshold {
                    warning: target_value * 0.9,
                    critical: target_value * 0.8,
                    comparison: "gt".to_string(),
                }
            },
        };
        
        // メトリクスを作成
        let metric = SLAMetric {
            id: metric_id.clone(),
            name: name.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            metric_type,
            target_value,
            current_value: target_value,
            unit: unit.to_string(),
            threshold,
            last_updated: now,
            enabled: true,
            metadata: HashMap::new(),
        };
        
        // メトリクスを保存
        self.metrics.insert(metric_id.clone(), metric);
        
        // メトリクス履歴を初期化
        self.metric_history.insert(metric_id.clone(), Vec::new());
        
        info!("SLA metric created: {} ({})", name, metric_id);
        
        Ok(metric_id)
    }
    
    /// SLAを作成
    pub fn create_agreement(
        &mut self,
        name: &str,
        title: &str,
        description: &str,
        tier: SLATier,
        metric_targets: Vec<(&str, f64)>,
        support_hours: u32,
        response_time_hours: u32,
    ) -> Result<String, Error> {
        // SLA IDを生成
        let agreement_id = format!("SLA-{}", name);
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // メトリクスターゲットを設定
        let mut metrics = HashMap::new();
        for (metric_name, target_value) in metric_targets {
            let metric_id = format!("METRIC-{}", metric_name);
            if !self.metrics.contains_key(&metric_id) {
                return Err(Error::NotFound(format!("Metric not found: {}", metric_name)));
            }
            metrics.insert(metric_id, target_value);
        }
        
        // SLAを作成
        let agreement = ServiceLevelAgreement {
            id: agreement_id.clone(),
            name: name.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            tier,
            metrics,
            support_hours,
            response_time_hours,
            created_at: now,
            updated_at: now,
            status: SLAStatus::Active,
            metadata: HashMap::new(),
        };
        
        // SLAを保存
        self.agreements.insert(agreement_id.clone(), agreement);
        
        info!("SLA created: {} ({})", name, agreement_id);
        
        Ok(agreement_id)
    }
    
    /// メトリクスを更新
    pub fn update_metric(
        &mut self,
        metric_name: &str,
        value: f64,
    ) -> Result<(), Error> {
        // SLA監視が有効かチェック
        if !self.config.enable_sla_monitoring {
            return Err(Error::InvalidState("SLA monitoring is not enabled".to_string()));
        }
        
        // メトリクスIDを生成
        let metric_id = format!("METRIC-{}", metric_name);
        
        // メトリクスを取得
        let metric = self.metrics.get_mut(&metric_id)
            .ok_or_else(|| Error::NotFound(format!("Metric not found: {}", metric_name)))?;
        
        // メトリクスが有効かチェック
        if !metric.enabled {
            return Err(Error::InvalidState(format!("Metric is not enabled: {}", metric_name)));
        }
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // 前回の値を保存
        let previous_value = metric.current_value;
        
        // メトリクス値を更新
        metric.current_value = value;
        metric.last_updated = now;
        
        // メトリクス履歴に追加
        if let Some(history) = self.metric_history.get_mut(&metric_id) {
            history.push(MetricValue {
                timestamp: now,
                value,
            });
            
            // 履歴サイズを制限
            if history.len() > 1000 {
                history.remove(0);
            }
        }
        
        // メトリクスコレクターを更新
        self.metrics_collector.set_gauge(&format!("sla_{}", metric_name), value);
        
        // しきい値をチェック
        self.check_metric_threshold(metric, previous_value)?;
        
        debug!("Metric updated: {} = {} {}", metric_name, value, metric.unit);
        
        Ok(())
    }
    
    /// メトリクスのしきい値をチェック
    fn check_metric_threshold(
        &mut self,
        metric: &SLAMetric,
        previous_value: f64,
    ) -> Result<(), Error> {
        // SLA違反アラートが有効かチェック
        if !self.config.enable_sla_violation_alerts {
            return Ok(());
        }
        
        let current_value = metric.current_value;
        let threshold = &metric.threshold;
        
        // しきい値違反をチェック
        let (is_critical, is_warning) = match threshold.comparison.as_str() {
            "gt" => {
                // 大きい方が良い（可用性など）
                let is_critical = current_value < threshold.critical;
                let is_warning = !is_critical && current_value < threshold.warning;
                (is_critical, is_warning)
            },
            "lt" => {
                // 小さい方が良い（レスポンスタイムなど）
                let is_critical = current_value > threshold.critical;
                let is_warning = !is_critical && current_value > threshold.warning;
                (is_critical, is_warning)
            },
            _ => (false, false),
        };
        
        // 前回の値と比較して状態が変化した場合のみアラート
        let previous_critical = match threshold.comparison.as_str() {
            "gt" => previous_value < threshold.critical,
            "lt" => previous_value > threshold.critical,
            _ => false,
        };
        
        let previous_warning = match threshold.comparison.as_str() {
            "gt" => !previous_critical && previous_value < threshold.warning,
            "lt" => !previous_critical && previous_value > threshold.warning,
            _ => false,
        };
        
        // クリティカルアラート
        if is_critical && !previous_critical {
            self.create_sla_alert(
                &metric.id,
                AlertType::MetricViolation,
                AlertSeverity::Critical,
                &format!(
                    "Critical threshold violated for {}: {} {} (threshold: {} {})",
                    metric.title,
                    current_value,
                    metric.unit,
                    threshold.critical,
                    metric.unit,
                ),
            )?;
        }
        // 警告アラート
        else if is_warning && !previous_warning {
            self.create_sla_alert(
                &metric.id,
                AlertType::MetricViolation,
                AlertSeverity::Warning,
                &format!(
                    "Warning threshold violated for {}: {} {} (threshold: {} {})",
                    metric.title,
                    current_value,
                    metric.unit,
                    threshold.warning,
                    metric.unit,
                ),
            )?;
        }
        // 回復アラート
        else if !is_critical && !is_warning && (previous_critical || previous_warning) {
            self.create_sla_alert(
                &metric.id,
                AlertType::MetricRecovery,
                AlertSeverity::Info,
                &format!(
                    "Metric recovered for {}: {} {}",
                    metric.title,
                    current_value,
                    metric.unit,
                ),
            )?;
        }
        
        Ok(())
    }
    
    /// SLAアラートを作成
    fn create_sla_alert(
        &mut self,
        metric_id: &str,
        alert_type: AlertType,
        severity: AlertSeverity,
        message: &str,
    ) -> Result<String, Error> {
        // アラートIDを生成
        let alert_id = format!("ALERT-{}", Uuid::new_v4());
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // アラートを作成
        let alert = SLAAlert {
            id: alert_id.clone(),
            metric_id: metric_id.to_string(),
            alert_type,
            severity,
            message: message.to_string(),
            timestamp: now,
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
            resolved: false,
            resolved_at: None,
            status: AlertStatus::Active,
            metadata: HashMap::new(),
        };
        
        // アラートを保存
        self.alerts.push(alert);
        
        // メトリクスを更新
        self.metrics_collector.increment_counter("sla_alerts_generated");
        
        info!("SLA alert created: {} ({})", message, alert_id);
        
        Ok(alert_id)
    }
    
    /// SLAレポートを生成
    pub fn generate_report(
        &mut self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<SLAReport, Error> {
        // SLAレポートが有効かチェック
        if !self.config.enable_sla_reporting {
            return Err(Error::InvalidState("SLA reporting is not enabled".to_string()));
        }
        
        // レポートIDを生成
        let report_id = format!("REPORT-{}", Uuid::new_v4());
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // 期間をチェック
        if end_time <= start_time {
            return Err(Error::InvalidArgument("End time must be after start time".to_string()));
        }
        
        // メトリクスデータを収集
        let mut metrics_data = HashMap::new();
        for (metric_id, metric) in &self.metrics {
            if let Some(history) = self.metric_history.get(metric_id) {
                // 期間内のデータを抽出
                let period_data: Vec<&MetricValue> = history.iter()
                    .filter(|v| v.timestamp >= start_time && v.timestamp <= end_time)
                    .collect();
                
                if !period_data.is_empty() {
                    // 平均値を計算
                    let avg_value = period_data.iter().map(|v| v.value).sum::<f64>() / period_data.len() as f64;
                    
                    // 最小値と最大値を計算
                    let min_value = period_data.iter().map(|v| v.value).fold(f64::INFINITY, f64::min);
                    let max_value = period_data.iter().map(|v| v.value).fold(f64::NEG_INFINITY, f64::max);
                    
                    // データポイント数
                    let data_points = period_data.len();
                    
                    // SLA準拠状態を計算
                    let compliance_status = if metric.metric_type == MetricType::Duration {
                        // レスポンスタイムなどは小さい方が良い
                        if avg_value <= metric.target_value {
                            ComplianceStatus::Compliant
                        } else if avg_value <= metric.threshold.warning {
                            ComplianceStatus::Warning
                        } else {
                            ComplianceStatus::Violation
                        }
                    } else {
                        // 可用性などは大きい方が良い
                        if avg_value >= metric.target_value {
                            ComplianceStatus::Compliant
                        } else if avg_value >= metric.threshold.warning {
                            ComplianceStatus::Warning
                        } else {
                            ComplianceStatus::Violation
                        }
                    };
                    
                    // メトリクスデータを追加
                    metrics_data.insert(metric_id.clone(), serde_json::json!({
                        "name": metric.name,
                        "title": metric.title,
                        "type": format!("{:?}", metric.metric_type),
                        "unit": metric.unit,
                        "target_value": metric.target_value,
                        "avg_value": avg_value,
                        "min_value": min_value,
                        "max_value": max_value,
                        "data_points": data_points,
                        "compliance_status": format!("{:?}", compliance_status),
                    }));
                }
            }
        }
        
        // アラートデータを収集
        let alerts_data: Vec<serde_json::Value> = self.alerts.iter()
            .filter(|a| a.timestamp >= start_time && a.timestamp <= end_time)
            .map(|a| serde_json::json!({
                "id": a.id,
                "metric_id": a.metric_id,
                "type": format!("{:?}", a.alert_type),
                "severity": format!("{:?}", a.severity),
                "message": a.message,
                "timestamp": a.timestamp,
                "acknowledged": a.acknowledged,
                "resolved": a.resolved,
                "status": format!("{:?}", a.status),
            }))
            .collect();
        
        // SLA準拠状態を計算
        let overall_status = if metrics_data.values().any(|v| v["compliance_status"] == "Violation") {
            ComplianceStatus::Violation
        } else if metrics_data.values().any(|v| v["compliance_status"] == "Warning") {
            ComplianceStatus::Warning
        } else {
            ComplianceStatus::Compliant
        };
        
        // レポートを作成
        let report = SLAReport {
            id: report_id.clone(),
            title: format!("SLA Report: {} to {}", start_time.format("%Y-%m-%d"), end_time.format("%Y-%m-%d")),
            description: "Service Level Agreement compliance report".to_string(),
            start_time,
            end_time,
            generated_at: now,
            overall_status,
            metrics: metrics_data,
            alerts: alerts_data,
            format: ReportFormat::JSON,
            metadata: HashMap::new(),
        };
        
        // レポートを保存
        self.reports.insert(report_id.clone(), report.clone());
        
        // メトリクスを更新
        self.metrics_collector.increment_counter("sla_reports_generated");
        
        info!("SLA report generated: {}", report_id);
        
        Ok(report)
    }
    
    /// サポートチケットを作成
    pub fn create_support_ticket(
        &mut self,
        title: &str,
        description: &str,
        category: TicketCategory,
        priority: TicketPriority,
        customer_id: &str,
        customer_email: &str,
    ) -> Result<String, Error> {
        // チケットIDを生成
        let ticket_id = format!("TICKET-{}", Uuid::new_v4());
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // チケットを作成
        let ticket = SupportTicket {
            id: ticket_id.clone(),
            title: title.to_string(),
            description: description.to_string(),
            category,
            priority,
            status: TicketStatus::Open,
            customer_id: customer_id.to_string(),
            customer_email: customer_email.to_string(),
            assigned_to: None,
            created_at: now,
            updated_at: now,
            resolved_at: None,
            resolution: None,
            comments: Vec::new(),
            attachments: Vec::new(),
            metadata: HashMap::new(),
        };
        
        // チケットを保存
        self.tickets.insert(ticket_id.clone(), ticket);
        
        // メトリクスを更新
        self.metrics_collector.increment_counter("support_tickets_created");
        
        info!("Support ticket created: {} ({})", title, ticket_id);
        
        Ok(ticket_id)
    }
    
    /// サポートチケットにコメントを追加
    pub fn add_ticket_comment(
        &mut self,
        ticket_id: &str,
        author: &str,
        content: &str,
        is_internal: bool,
    ) -> Result<String, Error> {
        // チケットを取得
        let ticket = self.tickets.get_mut(ticket_id)
            .ok_or_else(|| Error::NotFound(format!("Support ticket not found: {}", ticket_id)))?;
        
        // コメントIDを生成
        let comment_id = format!("COMMENT-{}", Uuid::new_v4());
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // コメントを作成
        let comment = support::TicketComment {
            id: comment_id.clone(),
            author: author.to_string(),
            content: content.to_string(),
            created_at: now,
            is_internal,
            metadata: HashMap::new(),
        };
        
        // コメントを追加
        ticket.comments.push(comment);
        
        // チケットを更新
        ticket.updated_at = now;
        
        info!("Comment added to ticket: {} ({})", ticket_id, comment_id);
        
        Ok(comment_id)
    }
    
    /// サポートチケットのステータスを更新
    pub fn update_ticket_status(
        &mut self,
        ticket_id: &str,
        status: TicketStatus,
        resolution: Option<&str>,
        updated_by: &str,
    ) -> Result<(), Error> {
        // チケットを取得
        let ticket = self.tickets.get_mut(ticket_id)
            .ok_or_else(|| Error::NotFound(format!("Support ticket not found: {}", ticket_id)))?;
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // ステータスを更新
        let old_status = ticket.status.clone();
        ticket.status = status.clone();
        ticket.updated_at = now;
        
        // 解決済みの場合
        if status == TicketStatus::Resolved {
            ticket.resolved_at = Some(now);
            ticket.resolution = resolution.map(|r| r.to_string());
        }
        
        // コメントを追加
        let comment = match (old_status, status.clone()) {
            (TicketStatus::Open, TicketStatus::InProgress) => {
                format!("Ticket status changed from Open to In Progress by {}", updated_by)
            },
            (TicketStatus::Open, TicketStatus::Resolved) => {
                format!("Ticket resolved by {}", updated_by)
            },
            (TicketStatus::InProgress, TicketStatus::Resolved) => {
                format!("Ticket resolved by {}", updated_by)
            },
            (TicketStatus::Resolved, TicketStatus::Reopened) => {
                format!("Ticket reopened by {}", updated_by)
            },
            (old, new) => {
                format!("Ticket status changed from {:?} to {:?} by {}", old, new, updated_by)
            },
        };
        
        self.add_ticket_comment(ticket_id, updated_by, &comment, true)?;
        
        // メトリクスを更新
        self.metrics_collector.increment_counter(&format!("support_tickets_{:?}", status.to_string().to_lowercase()));
        
        info!("Ticket status updated: {} -> {:?}", ticket_id, status);
        
        Ok(())
    }
    
    /// サポートチケットを担当者に割り当て
    pub fn assign_ticket(
        &mut self,
        ticket_id: &str,
        assignee: &str,
        updated_by: &str,
    ) -> Result<(), Error> {
        // チケットを取得
        let ticket = self.tickets.get_mut(ticket_id)
            .ok_or_else(|| Error::NotFound(format!("Support ticket not found: {}", ticket_id)))?;
        
        // 現在時刻を取得
        let now = Utc::now();
        
        // 担当者を更新
        let old_assignee = ticket.assigned_to.clone();
        ticket.assigned_to = Some(assignee.to_string());
        ticket.updated_at = now;
        
        // コメントを追加
        let comment = match old_assignee {
            Some(old) => {
                format!("Ticket reassigned from {} to {} by {}", old, assignee, updated_by)
            },
            None => {
                format!("Ticket assigned to {} by {}", assignee, updated_by)
            },
        };
        
        self.add_ticket_comment(ticket_id, updated_by, &comment, true)?;
        
        info!("Ticket assigned: {} -> {}", ticket_id, assignee);
        
        Ok(())
    }
    
    /// SLAを取得
    pub fn get_agreement(&self, agreement_id: &str) -> Result<&ServiceLevelAgreement, Error> {
        self.agreements.get(agreement_id)
            .ok_or_else(|| Error::NotFound(format!("SLA not found: {}", agreement_id)))
    }
    
    /// SLAリストを取得
    pub fn get_agreements(&self, tier: Option<SLATier>) -> Vec<&ServiceLevelAgreement> {
        self.agreements.values()
            .filter(|a| tier.map_or(true, |t| a.tier == t))
            .collect()
    }
    
    /// メトリクスを取得
    pub fn get_metric(&self, metric_id: &str) -> Result<&SLAMetric, Error> {
        self.metrics.get(metric_id)
            .ok_or_else(|| Error::NotFound(format!("Metric not found: {}", metric_id)))
    }
    
    /// メトリクスリストを取得
    pub fn get_metrics(&self, metric_type: Option<MetricType>) -> Vec<&SLAMetric> {
        self.metrics.values()
            .filter(|m| metric_type.map_or(true, |t| m.metric_type == t))
            .collect()
    }
    
    /// メトリクス履歴を取得
    pub fn get_metric_history(
        &self,
        metric_id: &str,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Result<Vec<&MetricValue>, Error> {
        let history = self.metric_history.get(metric_id)
            .ok_or_else(|| Error::NotFound(format!("Metric history not found: {}", metric_id)))?;
        
        let mut values: Vec<&MetricValue> = history.iter()
            .filter(|v| {
                start_time.map_or(true, |t| v.timestamp >= t) &&
                end_time.map_or(true, |t| v.timestamp <= t)
            })
            .collect();
        
        // 時間順にソート
        values.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        // 上限を適用
        if let Some(limit) = limit {
            if values.len() > limit {
                values.truncate(limit);
            }
        }
        
        Ok(values)
    }
    
    /// レポートを取得
    pub fn get_report(&self, report_id: &str) -> Result<&SLAReport, Error> {
        self.reports.get(report_id)
            .ok_or_else(|| Error::NotFound(format!("Report not found: {}", report_id)))
    }
    
    /// レポートリストを取得
    pub fn get_reports(&self) -> Vec<&SLAReport> {
        self.reports.values().collect()
    }
    
    /// アラートを取得
    pub fn get_alerts(
        &self,
        severity: Option<AlertSeverity>,
        status: Option<AlertStatus>,
    ) -> Vec<&SLAAlert> {
        self.alerts.iter()
            .filter(|a| {
                severity.map_or(true, |s| a.severity == s) &&
                status.map_or(true, |s| a.status == s)
            })
            .collect()
    }
    
    /// アラートを確認
    pub fn acknowledge_alert(
        &mut self,
        alert_id: &str,
        acknowledged_by: &str,
    ) -> Result<(), Error> {
        // アラートを検索
        let alert = self.alerts.iter_mut()
            .find(|a| a.id == alert_id)
            .ok_or_else(|| Error::NotFound(format!("Alert not found: {}", alert_id)))?;
        
        // アラートを確認
        alert.acknowledged = true;
        alert.acknowledged_by = Some(acknowledged_by.to_string());
        alert.acknowledged_at = Some(Utc::now());
        
        info!("Alert acknowledged: {} by {}", alert_id, acknowledged_by);
        
        Ok(())
    }
    
    /// アラートを解決
    pub fn resolve_alert(
        &mut self,
        alert_id: &str,
    ) -> Result<(), Error> {
        // アラートを検索
        let alert = self.alerts.iter_mut()
            .find(|a| a.id == alert_id)
            .ok_or_else(|| Error::NotFound(format!("Alert not found: {}", alert_id)))?;
        
        // アラートを解決
        alert.resolved = true;
        alert.resolved_at = Some(Utc::now());
        alert.status = AlertStatus::Resolved;
        
        info!("Alert resolved: {}", alert_id);
        
        Ok(())
    }
    
    /// チケットを取得
    pub fn get_ticket(&self, ticket_id: &str) -> Result<&SupportTicket, Error> {
        self.tickets.get(ticket_id)
            .ok_or_else(|| Error::NotFound(format!("Ticket not found: {}", ticket_id)))
    }
    
    /// チケットリストを取得
    pub fn get_tickets(
        &self,
        status: Option<TicketStatus>,
        priority: Option<TicketPriority>,
        category: Option<TicketCategory>,
        assignee: Option<&str>,
    ) -> Vec<&SupportTicket> {
        self.tickets.values()
            .filter(|t| {
                status.map_or(true, |s| t.status == s) &&
                priority.map_or(true, |p| t.priority == p) &&
                category.map_or(true, |c| t.category == c) &&
                assignee.map_or(true, |a| t.assigned_to.as_ref().map_or(false, |assigned| assigned == a))
            })
            .collect()
    }
    
    /// 設定を更新
    pub fn update_config(&mut self, config: SLAConfig) {
        self.config = config;
    }
    
    /// 設定を取得
    pub fn get_config(&self) -> &SLAConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sla_manager_initialization() {
        let config = SLAConfig::default();
        let manager = SLAManager::new(config);
        
        assert!(manager.is_initialized());
        
        // デフォルトのメトリクスが初期化されていることを確認
        assert!(manager.metrics.contains_key("METRIC-availability"));
        assert!(manager.metrics.contains_key("METRIC-response_time"));
        assert!(manager.metrics.contains_key("METRIC-throughput"));
        assert!(manager.metrics.contains_key("METRIC-error_rate"));
        assert!(manager.metrics.contains_key("METRIC-data_integrity"));
        
        // デフォルトのSLAが初期化されていることを確認
        assert!(manager.agreements.contains_key("SLA-standard"));
        assert!(manager.agreements.contains_key("SLA-premium"));
        assert!(manager.agreements.contains_key("SLA-enterprise"));
    }
    
    #[test]
    fn test_metric_update() {
        let config = SLAConfig::default();
        let mut manager = SLAManager::new(config);
        
        // メトリクスを更新
        manager.update_metric("availability", 99.95).unwrap();
        
        // メトリクスが更新されたことを確認
        let metric = manager.get_metric("METRIC-availability").unwrap();
        assert_eq!(metric.current_value, 99.95);
        
        // メトリクス履歴が更新されたことを確認
        let history = manager.get_metric_history("METRIC-availability", None, None, None).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].value, 99.95);
    }
    
    #[test]
    fn test_sla_report_generation() {
        let config = SLAConfig::default();
        let mut manager = SLAManager::new(config);
        
        // いくつかのメトリクスを更新
        manager.update_metric("availability", 99.95).unwrap();
        manager.update_metric("response_time", 95.0).unwrap();
        manager.update_metric("throughput", 1200.0).unwrap();
        manager.update_metric("error_rate", 0.05).unwrap();
        manager.update_metric("data_integrity", 99.9995).unwrap();
        
        // 1時間後
        let one_hour_later = Utc::now() + Duration::hours(1);
        
        // 別の値でメトリクスを更新
        manager.update_metric("availability", 99.98).unwrap();
        manager.update_metric("response_time", 90.0).unwrap();
        manager.update_metric("throughput", 1300.0).unwrap();
        manager.update_metric("error_rate", 0.03).unwrap();
        manager.update_metric("data_integrity", 99.9998).unwrap();
        
        // レポートを生成
        let start_time = Utc::now() - Duration::hours(2);
        let end_time = Utc::now() + Duration::hours(2);
        let report = manager.generate_report(start_time, end_time).unwrap();
        
        // レポートをチェック
        assert_eq!(report.overall_status, ComplianceStatus::Compliant);
        assert!(report.metrics.contains_key("METRIC-availability"));
        assert!(report.metrics.contains_key("METRIC-response_time"));
        assert!(report.metrics.contains_key("METRIC-throughput"));
        assert!(report.metrics.contains_key("METRIC-error_rate"));
        assert!(report.metrics.contains_key("METRIC-data_integrity"));
        
        // 可用性メトリクスをチェック
        let availability = &report.metrics["METRIC-availability"];
        assert_eq!(availability["name"], "availability");
        assert_eq!(availability["unit"], "%");
        assert_eq!(availability["data_points"], 2);
        
        // 平均値をチェック（99.95 + 99.98）/ 2 = 99.965
        assert!((availability["avg_value"].as_f64().unwrap() - 99.965).abs() < 0.001);
    }
    
    #[test]
    fn test_support_ticket() {
        let config = SLAConfig::default();
        let mut manager = SLAManager::new(config);
        
        // チケットを作成
        let ticket_id = manager.create_support_ticket(
            "Cannot access dashboard",
            "I'm unable to access the dashboard after the recent update.",
            TicketCategory::Technical,
            TicketPriority::High,
            "customer123",
            "customer@example.com",
        ).unwrap();
        
        // チケットが作成されたことを確認
        let ticket = manager.get_ticket(&ticket_id).unwrap();
        assert_eq!(ticket.title, "Cannot access dashboard");
        assert_eq!(ticket.category, TicketCategory::Technical);
        assert_eq!(ticket.priority, TicketPriority::High);
        assert_eq!(ticket.status, TicketStatus::Open);
        assert_eq!(ticket.customer_id, "customer123");
        assert_eq!(ticket.customer_email, "customer@example.com");
        assert!(ticket.assigned_to.is_none());
        assert!(ticket.comments.is_empty());
        
        // チケットを担当者に割り当て
        manager.assign_ticket(&ticket_id, "support_agent1", "support_manager").unwrap();
        
        // 割り当てが更新されたことを確認
        let ticket = manager.get_ticket(&ticket_id).unwrap();
        assert_eq!(ticket.assigned_to, Some("support_agent1".to_string()));
        assert_eq!(ticket.comments.len(), 1);
        
        // コメントを追加
        manager.add_ticket_comment(
            &ticket_id,
            "support_agent1",
            "I'm investigating this issue. Could you please provide your browser version?",
            false,
        ).unwrap();
        
        // コメントが追加されたことを確認
        let ticket = manager.get_ticket(&ticket_id).unwrap();
        assert_eq!(ticket.comments.len(), 2);
        assert_eq!(ticket.comments[1].author, "support_agent1");
        assert!(!ticket.comments[1].is_internal);
        
        // ステータスを更新
        manager.update_ticket_status(
            &ticket_id,
            TicketStatus::InProgress,
            None,
            "support_agent1",
        ).unwrap();
        
        // ステータスが更新されたことを確認
        let ticket = manager.get_ticket(&ticket_id).unwrap();
        assert_eq!(ticket.status, TicketStatus::InProgress);
        assert_eq!(ticket.comments.len(), 3);
        
        // チケットを解決
        manager.update_ticket_status(
            &ticket_id,
            TicketStatus::Resolved,
            Some("Fixed by clearing browser cache"),
            "support_agent1",
        ).unwrap();
        
        // チケットが解決されたことを確認
        let ticket = manager.get_ticket(&ticket_id).unwrap();
        assert_eq!(ticket.status, TicketStatus::Resolved);
        assert_eq!(ticket.resolution, Some("Fixed by clearing browser cache".to_string()));
        assert!(ticket.resolved_at.is_some());
        assert_eq!(ticket.comments.len(), 4);
    }
    
    #[test]
    fn test_alert_generation() {
        let mut config = SLAConfig::default();
        config.enable_sla_violation_alerts = true;
        let mut manager = SLAManager::new(config);
        
        // 正常値でメトリクスを更新
        manager.update_metric("availability", 99.95).unwrap();
        
        // アラートが生成されていないことを確認
        let alerts = manager.get_alerts(None, None);
        assert!(alerts.is_empty());
        
        // クリティカルしきい値を下回る値でメトリクスを更新
        manager.update_metric("availability", 98.5).unwrap();
        
        // クリティカルアラートが生成されたことを確認
        let alerts = manager.get_alerts(Some(AlertSeverity::Critical), None);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].metric_id, "METRIC-availability");
        assert_eq!(alerts[0].alert_type, AlertType::MetricViolation);
        assert_eq!(alerts[0].severity, AlertSeverity::Critical);
        assert!(!alerts[0].acknowledged);
        assert!(!alerts[0].resolved);
        
        // アラートを確認
        manager.acknowledge_alert(&alerts[0].id, "admin").unwrap();
        
        // アラートが確認されたことを確認
        let acknowledged_alerts = manager.get_alerts(None, None);
        assert_eq!(acknowledged_alerts.len(), 1);
        assert!(acknowledged_alerts[0].acknowledged);
        assert_eq!(acknowledged_alerts[0].acknowledged_by, Some("admin".to_string()));
        
        // 正常値に戻してメトリクスを更新
        manager.update_metric("availability", 99.95).unwrap();
        
        // 回復アラートが生成されたことを確認
        let recovery_alerts = manager.get_alerts(Some(AlertSeverity::Info), None);
        assert_eq!(recovery_alerts.len(), 1);
        assert_eq!(recovery_alerts[0].alert_type, AlertType::MetricRecovery);
    }
}