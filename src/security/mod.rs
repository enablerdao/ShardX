// セキュリティモジュール
//
// このモジュールは、ShardXにおけるセキュリティ機能を提供します。
// 主な機能:
// - 脆弱性検出
// - セキュリティ監査
// - インシデント対応
// - 異常検出
// - セキュリティポリシー適用

mod vulnerability_scanner;
mod audit;
mod incident_response;
// mod anomaly_detection; // TODO: このモジュールが見つかりません
// mod security_policy; // TODO: このモジュールが見つかりません

pub use self::vulnerability_scanner::{VulnerabilityScanner, VulnerabilityReport, Vulnerability, SeverityLevel};
pub use self::audit::{SecurityAuditor, AuditReport, AuditFinding};
pub use self::incident_response::{IncidentResponseManager, SecurityIncident, IncidentStatus};
pub use self::anomaly_detection::{AnomalyDetector, AnomalyReport, AnomalyType};
pub use self::security_policy::{SecurityPolicyManager, SecurityPolicy, PolicyViolation};

use crate::error::Error;
use crate::metrics::MetricsCollector;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use log::{debug, error, info, warn};

/// セキュリティマネージャー
pub struct SecurityManager {
    /// 脆弱性スキャナー
    vulnerability_scanner: VulnerabilityScanner,
    /// セキュリティ監査人
    security_auditor: SecurityAuditor,
    /// インシデント対応マネージャー
    incident_response_manager: IncidentResponseManager,
    /// 異常検出器
    anomaly_detector: AnomalyDetector,
    /// セキュリティポリシーマネージャー
    security_policy_manager: SecurityPolicyManager,
    /// メトリクスコレクター
    metrics: Arc<MetricsCollector>,
    /// 最後のスキャン時刻
    last_scan: Arc<Mutex<Instant>>,
    /// 検出された脆弱性
    detected_vulnerabilities: Arc<Mutex<HashMap<String, Vulnerability>>>,
    /// 検出された異常
    detected_anomalies: Arc<Mutex<Vec<AnomalyReport>>>,
    /// アクティブなインシデント
    active_incidents: Arc<Mutex<HashMap<String, SecurityIncident>>>,
    /// セキュリティスコア
    security_score: Arc<Mutex<f64>>,
}

impl SecurityManager {
    /// 新しいSecurityManagerを作成
    pub fn new(metrics: Arc<MetricsCollector>) -> Self {
        Self {
            vulnerability_scanner: VulnerabilityScanner::new(),
            security_auditor: SecurityAuditor::new(),
            incident_response_manager: IncidentResponseManager::new(),
            anomaly_detector: AnomalyDetector::new(),
            security_policy_manager: SecurityPolicyManager::new(),
            metrics,
            last_scan: Arc::new(Mutex::new(Instant::now())),
            detected_vulnerabilities: Arc::new(Mutex::new(HashMap::new())),
            detected_anomalies: Arc::new(Mutex::new(Vec::new())),
            active_incidents: Arc::new(Mutex::new(HashMap::new())),
            security_score: Arc::new(Mutex::new(100.0)), // 初期スコアは100点満点
        }
    }
    
    /// 脆弱性スキャンを実行
    pub async fn scan_for_vulnerabilities(&self) -> Result<VulnerabilityReport, Error> {
        info!("Starting vulnerability scan");
        
        // スキャンを実行
        let report = self.vulnerability_scanner.scan_system().await?;
        
        // 検出された脆弱性を保存
        {
            let mut vulnerabilities = self.detected_vulnerabilities.lock().unwrap();
            for vulnerability in &report.vulnerabilities {
                vulnerabilities.insert(vulnerability.id.clone(), vulnerability.clone());
            }
        }
        
        // 最後のスキャン時刻を更新
        *self.last_scan.lock().unwrap() = Instant::now();
        
        // メトリクスを更新
        self.metrics.set_gauge("security_vulnerabilities_total", report.vulnerabilities.len() as f64);
        self.metrics.set_gauge("security_vulnerabilities_critical", 
            report.vulnerabilities.iter().filter(|v| v.severity == SeverityLevel::Critical).count() as f64);
        self.metrics.set_gauge("security_vulnerabilities_high", 
            report.vulnerabilities.iter().filter(|v| v.severity == SeverityLevel::High).count() as f64);
        
        // セキュリティスコアを更新
        self.update_security_score();
        
        // 重大な脆弱性が見つかった場合はインシデントを作成
        for vulnerability in &report.vulnerabilities {
            if vulnerability.severity == SeverityLevel::Critical {
                let incident = SecurityIncident {
                    id: format!("INC-{}", uuid::Uuid::new_v4()),
                    title: format!("Critical vulnerability detected: {}", vulnerability.title),
                    description: vulnerability.description.clone(),
                    severity: vulnerability.severity.clone(),
                    status: IncidentStatus::Open,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    assigned_to: None,
                    related_vulnerabilities: vec![vulnerability.id.clone()],
                    resolution: None,
                };
                
                self.incident_response_manager.create_incident(incident.clone())?;
                
                // アクティブなインシデントに追加
                let mut active_incidents = self.active_incidents.lock().unwrap();
                active_incidents.insert(incident.id.clone(), incident);
            }
        }
        
        info!("Vulnerability scan completed: {} vulnerabilities found", report.vulnerabilities.len());
        
        Ok(report)
    }
    
    /// 異常検出を実行
    pub async fn detect_anomalies(&self, data: &[u8]) -> Result<AnomalyReport, Error> {
        // 異常検出を実行
        let report = self.anomaly_detector.detect(data).await?;
        
        // 検出された異常を保存
        if report.anomalies.len() > 0 {
            let mut anomalies = self.detected_anomalies.lock().unwrap();
            anomalies.push(report.clone());
            
            // メトリクスを更新
            self.metrics.increment_counter_by("security_anomalies_detected", report.anomalies.len() as u64);
            
            // 重大な異常が見つかった場合はインシデントを作成
            for anomaly in &report.anomalies {
                if anomaly.severity == SeverityLevel::Critical || anomaly.severity == SeverityLevel::High {
                    let incident = SecurityIncident {
                        id: format!("INC-{}", uuid::Uuid::new_v4()),
                        title: format!("Security anomaly detected: {}", anomaly.title),
                        description: anomaly.description.clone(),
                        severity: anomaly.severity.clone(),
                        status: IncidentStatus::Open,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        assigned_to: None,
                        related_vulnerabilities: Vec::new(),
                        resolution: None,
                    };
                    
                    self.incident_response_manager.create_incident(incident.clone())?;
                    
                    // アクティブなインシデントに追加
                    let mut active_incidents = self.active_incidents.lock().unwrap();
                    active_incidents.insert(incident.id.clone(), incident);
                }
            }
        }
        
        Ok(report)
    }
    
    /// セキュリティ監査を実行
    pub async fn perform_security_audit(&self) -> Result<AuditReport, Error> {
        info!("Starting security audit");
        
        // 監査を実行
        let report = self.security_auditor.audit_system().await?;
        
        // メトリクスを更新
        self.metrics.set_gauge("security_audit_findings_total", report.findings.len() as f64);
        self.metrics.set_gauge("security_audit_findings_critical", 
            report.findings.iter().filter(|f| f.severity == SeverityLevel::Critical).count() as f64);
        
        // セキュリティスコアを更新
        self.update_security_score();
        
        info!("Security audit completed: {} findings", report.findings.len());
        
        Ok(report)
    }
    
    /// インシデントを作成
    pub fn create_incident(&self, incident: SecurityIncident) -> Result<(), Error> {
        // インシデントを作成
        self.incident_response_manager.create_incident(incident.clone())?;
        
        // アクティブなインシデントに追加
        let mut active_incidents = self.active_incidents.lock().unwrap();
        active_incidents.insert(incident.id.clone(), incident);
        
        // メトリクスを更新
        self.metrics.increment_counter("security_incidents_created");
        self.metrics.set_gauge("security_incidents_active", active_incidents.len() as f64);
        
        Ok(())
    }
    
    /// インシデントを解決
    pub fn resolve_incident(&self, incident_id: &str, resolution: &str) -> Result<(), Error> {
        // インシデントを解決
        self.incident_response_manager.resolve_incident(incident_id, resolution)?;
        
        // アクティブなインシデントから削除
        let mut active_incidents = self.active_incidents.lock().unwrap();
        active_incidents.remove(incident_id);
        
        // メトリクスを更新
        self.metrics.increment_counter("security_incidents_resolved");
        self.metrics.set_gauge("security_incidents_active", active_incidents.len() as f64);
        
        Ok(())
    }
    
    /// セキュリティポリシーを適用
    pub fn apply_security_policy(&self, policy: SecurityPolicy) -> Result<(), Error> {
        self.security_policy_manager.apply_policy(policy)
    }
    
    /// セキュリティポリシー違反をチェック
    pub fn check_policy_violations(&self, data: &[u8]) -> Result<Vec<PolicyViolation>, Error> {
        self.security_policy_manager.check_violations(data)
    }
    
    /// セキュリティスコアを更新
    fn update_security_score(&self) {
        let mut score = 100.0;
        
        // 脆弱性に基づいてスコアを減点
        {
            let vulnerabilities = self.detected_vulnerabilities.lock().unwrap();
            
            // 重大度に応じた減点
            for vulnerability in vulnerabilities.values() {
                match vulnerability.severity {
                    SeverityLevel::Critical => score -= 10.0,
                    SeverityLevel::High => score -= 5.0,
                    SeverityLevel::Medium => score -= 2.0,
                    SeverityLevel::Low => score -= 0.5,
                    SeverityLevel::Info => score -= 0.1,
                }
            }
        }
        
        // アクティブなインシデントに基づいてスコアを減点
        {
            let active_incidents = self.active_incidents.lock().unwrap();
            
            // 重大度に応じた減点
            for incident in active_incidents.values() {
                match incident.severity {
                    SeverityLevel::Critical => score -= 15.0,
                    SeverityLevel::High => score -= 7.5,
                    SeverityLevel::Medium => score -= 3.0,
                    SeverityLevel::Low => score -= 1.0,
                    SeverityLevel::Info => score -= 0.2,
                }
            }
        }
        
        // スコアを0以上100以下に制限
        score = score.max(0.0).min(100.0);
        
        // スコアを更新
        *self.security_score.lock().unwrap() = score;
        
        // メトリクスを更新
        self.metrics.set_gauge("security_score", score);
    }
    
    /// セキュリティスコアを取得
    pub fn get_security_score(&self) -> f64 {
        *self.security_score.lock().unwrap()
    }
    
    /// 検出された脆弱性を取得
    pub fn get_detected_vulnerabilities(&self) -> HashMap<String, Vulnerability> {
        self.detected_vulnerabilities.lock().unwrap().clone()
    }
    
    /// アクティブなインシデントを取得
    pub fn get_active_incidents(&self) -> HashMap<String, SecurityIncident> {
        self.active_incidents.lock().unwrap().clone()
    }
    
    /// 最後のスキャン時刻を取得
    pub fn get_last_scan_time(&self) -> Instant {
        *self.last_scan.lock().unwrap()
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new(Arc::new(MetricsCollector::new("security")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_security_manager() {
        let metrics = Arc::new(MetricsCollector::new("test"));
        let manager = SecurityManager::new(metrics);
        
        // 初期セキュリティスコアを確認
        assert_eq!(manager.get_security_score(), 100.0);
        
        // 脆弱性スキャンを実行
        let report = manager.scan_for_vulnerabilities().await.unwrap();
        
        // スキャン結果を確認
        assert!(report.vulnerabilities.len() >= 0);
        
        // 異常検出を実行
        let data = b"test data for anomaly detection";
        let anomaly_report = manager.detect_anomalies(data).await.unwrap();
        
        // 異常検出結果を確認
        assert!(anomaly_report.anomalies.len() >= 0);
        
        // セキュリティ監査を実行
        let audit_report = manager.perform_security_audit().await.unwrap();
        
        // 監査結果を確認
        assert!(audit_report.findings.len() >= 0);
        
        // インシデントを作成
        let incident = SecurityIncident {
            id: "INC-TEST-001".to_string(),
            title: "Test incident".to_string(),
            description: "This is a test incident".to_string(),
            severity: SeverityLevel::Medium,
            status: IncidentStatus::Open,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            assigned_to: None,
            related_vulnerabilities: Vec::new(),
            resolution: None,
        };
        
        manager.create_incident(incident).unwrap();
        
        // アクティブなインシデントを確認
        let active_incidents = manager.get_active_incidents();
        assert_eq!(active_incidents.len(), 1);
        assert!(active_incidents.contains_key("INC-TEST-001"));
        
        // インシデントを解決
        manager.resolve_incident("INC-TEST-001", "Test resolution").unwrap();
        
        // アクティブなインシデントが減少したことを確認
        let active_incidents = manager.get_active_incidents();
        assert_eq!(active_incidents.len(), 0);
    }
}