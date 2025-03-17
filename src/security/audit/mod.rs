// セキュリティ監査モジュール
//
// このモジュールは、ShardXのセキュリティ監査機能を提供します。
// セキュリティ監査は、システムの脆弱性を特定し、セキュリティリスクを
// 評価するためのプロセスです。
//
// 主な機能:
// - コード監査
// - 脆弱性スキャン
// - ペネトレーションテスト
// - セキュリティレポート生成
// - コンプライアンスチェック

mod config;
mod scanner;
mod reporter;
mod compliance;
mod pentester;
mod code_analyzer;
mod risk_assessor;
mod remediation;
mod history;
mod scheduler;

pub use self::config::{AuditConfig, ScannerConfig, ReporterConfig, ComplianceConfig};
pub use self::scanner::{VulnerabilityScanner, ScanResult, VulnerabilityLevel};
pub use self::reporter::{AuditReporter, ReportFormat, AuditReport};
pub use self::compliance::{ComplianceChecker, ComplianceStandard, ComplianceResult};
pub use self::pentester::{PenetrationTester, PenTestResult, PenTestTarget};
pub use self::code_analyzer::{CodeAnalyzer, CodeAnalysisResult, CodeIssue};
pub use self::risk_assessor::{RiskAssessor, RiskLevel, RiskAssessment};
pub use self::remediation::{RemediationPlan, RemediationAction, RemediationStatus};
pub use self::history::{AuditHistory, AuditRecord, AuditComparison};
pub use self::scheduler::{AuditScheduler, ScheduleConfig, AuditTask};

use crate::error::Error;
use crate::metrics::MetricsCollector;
use crate::storage::StorageManager;
use crate::crypto::CryptoManager;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use log::{debug, error, info, warn};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// セキュリティ監査マネージャー
pub struct AuditManager {
    /// 設定
    config: AuditConfig,
    /// 脆弱性スキャナー
    scanner: VulnerabilityScanner,
    /// 監査レポーター
    reporter: AuditReporter,
    /// コンプライアンスチェッカー
    compliance_checker: ComplianceChecker,
    /// ペネトレーションテスター
    pentester: PenetrationTester,
    /// コード分析器
    code_analyzer: CodeAnalyzer,
    /// リスク評価器
    risk_assessor: RiskAssessor,
    /// 修復計画マネージャー
    remediation_manager: RemediationPlan,
    /// 監査履歴
    audit_history: AuditHistory,
    /// 監査スケジューラー
    scheduler: AuditScheduler,
    /// 監査結果
    audit_results: HashMap<AuditId, AuditResult>,
    /// ストレージマネージャー
    storage_manager: Arc<StorageManager>,
    /// 暗号マネージャー
    crypto_manager: Arc<CryptoManager>,
    /// メトリクスコレクター
    metrics: Arc<MetricsCollector>,
    /// 実行中フラグ
    running: bool,
    /// イベント通知チャネル
    event_tx: mpsc::Sender<AuditEvent>,
    /// イベント通知受信チャネル
    event_rx: mpsc::Receiver<AuditEvent>,
}

/// 監査ID
pub type AuditId = String;

/// 監査イベント
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    /// イベントID
    pub id: String,
    /// イベントタイプ
    pub event_type: AuditEventType,
    /// 監査ID
    pub audit_id: Option<AuditId>,
    /// タイムスタンプ
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// データ
    pub data: serde_json::Value,
}

/// 監査イベントタイプ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    /// 監査開始
    AuditStarted,
    /// 監査完了
    AuditCompleted,
    /// 脆弱性検出
    VulnerabilityDetected,
    /// コンプライアンス違反
    ComplianceViolation,
    /// コード問題検出
    CodeIssueDetected,
    /// リスク評価更新
    RiskAssessmentUpdated,
    /// 修復アクション作成
    RemediationActionCreated,
    /// 修復アクション完了
    RemediationActionCompleted,
    /// エラー
    Error,
}

/// 監査結果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditResult {
    /// 監査ID
    pub id: AuditId,
    /// 監査名
    pub name: String,
    /// 監査対象
    pub target: String,
    /// 監査タイプ
    pub audit_type: AuditType,
    /// 監査ステータス
    pub status: AuditStatus,
    /// 開始時間
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// 完了時間
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// スキャン結果
    pub scan_results: Option<Vec<ScanResult>>,
    /// コンプライアンス結果
    pub compliance_results: Option<Vec<ComplianceResult>>,
    /// コード分析結果
    pub code_analysis_results: Option<Vec<CodeAnalysisResult>>,
    /// ペネトレーションテスト結果
    pub pentest_results: Option<Vec<PenTestResult>>,
    /// リスク評価
    pub risk_assessment: Option<RiskAssessment>,
    /// 修復計画
    pub remediation_plan: Option<Vec<RemediationAction>>,
    /// 総合スコア（0-100）
    pub overall_score: Option<u32>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// 監査タイプ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditType {
    /// 脆弱性スキャン
    VulnerabilityScan,
    /// コンプライアンスチェック
    ComplianceCheck,
    /// コード分析
    CodeAnalysis,
    /// ペネトレーションテスト
    PenetrationTest,
    /// 総合監査
    Comprehensive,
    /// カスタム
    Custom(String),
}

/// 監査ステータス
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditStatus {
    /// 予定
    Scheduled,
    /// 実行中
    Running,
    /// 完了
    Completed,
    /// 失敗
    Failed,
    /// キャンセル
    Cancelled,
}

/// 監査統計
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditStats {
    /// 総監査数
    pub total_audits: usize,
    /// 完了監査数
    pub completed_audits: usize,
    /// 失敗監査数
    pub failed_audits: usize,
    /// 予定監査数
    pub scheduled_audits: usize,
    /// 実行中監査数
    pub running_audits: usize,
    /// 検出された脆弱性数
    pub detected_vulnerabilities: usize,
    /// 高リスク脆弱性数
    pub high_risk_vulnerabilities: usize,
    /// 中リスク脆弱性数
    pub medium_risk_vulnerabilities: usize,
    /// 低リスク脆弱性数
    pub low_risk_vulnerabilities: usize,
    /// コンプライアンス違反数
    pub compliance_violations: usize,
    /// コード問題数
    pub code_issues: usize,
    /// 修復アクション数
    pub remediation_actions: usize,
    /// 完了した修復アクション数
    pub completed_remediation_actions: usize,
    /// 平均監査スコア
    pub average_audit_score: f64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

impl AuditManager {
    /// 新しいAuditManagerを作成
    pub fn new(
        config: AuditConfig,
        storage_manager: Arc<StorageManager>,
        crypto_manager: Arc<CryptoManager>,
        metrics: Arc<MetricsCollector>,
    ) -> Self {
        let (tx, rx) = mpsc::channel(100);
        
        let scanner = VulnerabilityScanner::new(config.scanner_config.clone());
        let reporter = AuditReporter::new(config.reporter_config.clone());
        let compliance_checker = ComplianceChecker::new(config.compliance_config.clone());
        let pentester = PenetrationTester::new(config.pentester_config.clone());
        let code_analyzer = CodeAnalyzer::new(config.code_analyzer_config.clone());
        let risk_assessor = RiskAssessor::new(config.risk_assessor_config.clone());
        let remediation_manager = RemediationPlan::new(config.remediation_config.clone());
        let audit_history = AuditHistory::new(config.history_config.clone());
        let scheduler = AuditScheduler::new(config.scheduler_config.clone());
        
        Self {
            config,
            scanner,
            reporter,
            compliance_checker,
            pentester,
            code_analyzer,
            risk_assessor,
            remediation_manager,
            audit_history,
            scheduler,
            audit_results: HashMap::new(),
            storage_manager,
            crypto_manager,
            metrics,
            running: false,
            event_tx: tx,
            event_rx: rx,
        }
    }
    
    /// 監査マネージャーを開始
    pub async fn start(&mut self) -> Result<(), Error> {
        if self.running {
            return Err(Error::InvalidState("Audit manager is already running".to_string()));
        }
        
        self.running = true;
        
        // 保存された監査結果を読み込む
        self.load_audit_results().await?;
        
        // バックグラウンドタスクを開始
        self.start_background_tasks();
        
        info!("Audit manager started");
        
        Ok(())
    }
    
    /// 監査マネージャーを停止
    pub async fn stop(&mut self) -> Result<(), Error> {
        if !self.running {
            return Err(Error::InvalidState("Audit manager is not running".to_string()));
        }
        
        self.running = false;
        
        // 実行中の監査を停止
        for (audit_id, audit) in &mut self.audit_results {
            if audit.status == AuditStatus::Running {
                info!("Stopping audit: {}", audit_id);
                audit.status = AuditStatus::Cancelled;
                
                // 監査結果を保存
                let storage = self.storage_manager.get_storage("audit_results")?;
                storage.put(&format!("audit:{}", audit_id), audit)?;
            }
        }
        
        info!("Audit manager stopped");
        
        Ok(())
    }
    
    /// バックグラウンドタスクを開始
    fn start_background_tasks(&self) {
        // スケジューラータスク
        let scheduler_interval = self.config.scheduler_interval_ms;
        let scheduler_tx = self.event_tx.clone();
        let scheduler = Arc::new(RwLock::new(self.scheduler.clone()));
        let scheduler_running = Arc::new(RwLock::new(self.running));
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(scheduler_interval));
            
            loop {
                interval.tick().await;
                
                let running = *scheduler_running.read().unwrap();
                if !running {
                    break;
                }
                
                let s = scheduler.read().unwrap();
                
                // スケジュールされた監査を実行
                if let Err(e) = s.process_scheduled_audits().await {
                    error!("Failed to process scheduled audits: {}", e);
                }
            }
        });
    }
    
    /// 保存された監査結果を読み込む
    async fn load_audit_results(&mut self) -> Result<(), Error> {
        // ストレージから監査結果を読み込む
        let storage = self.storage_manager.get_storage("audit_results")?;
        
        if let Ok(results) = storage.get_all::<AuditResult>("audit") {
            for result in results {
                self.audit_results.insert(result.id.clone(), result);
            }
        }
        
        info!("Loaded {} audit results", self.audit_results.len());
        
        Ok(())
    }
    
    /// 監査を作成
    pub async fn create_audit(
        &mut self,
        name: &str,
        target: &str,
        audit_type: AuditType,
        scheduled_time: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<AuditId, Error> {
        // 監査IDを生成
        let audit_id = format!("audit-{}", Uuid::new_v4());
        
        // 現在時刻を取得
        let now = chrono::Utc::now();
        
        // 監査ステータスを決定
        let status = if scheduled_time.is_some() {
            AuditStatus::Scheduled
        } else {
            AuditStatus::Running
        };
        
        // 監査結果を作成
        let audit_result = AuditResult {
            id: audit_id.clone(),
            name: name.to_string(),
            target: target.to_string(),
            audit_type,
            status,
            started_at: now,
            completed_at: None,
            scan_results: None,
            compliance_results: None,
            code_analysis_results: None,
            pentest_results: None,
            risk_assessment: None,
            remediation_plan: None,
            overall_score: None,
            metadata: HashMap::new(),
        };
        
        // 監査結果を保存
        self.audit_results.insert(audit_id.clone(), audit_result.clone());
        
        // ストレージに保存
        let storage = self.storage_manager.get_storage("audit_results")?;
        storage.put(&format!("audit:{}", audit_id), &audit_result)?;
        
        // スケジュールされた監査の場合
        if let Some(scheduled_time) = scheduled_time {
            // 監査をスケジュール
            self.scheduler.schedule_audit(
                audit_id.clone(),
                name.to_string(),
                target.to_string(),
                audit_type.clone(),
                scheduled_time,
            ).await?;
            
            info!("Scheduled audit {} ({}) for {}", audit_id, name, scheduled_time);
        } else {
            // 監査開始イベントを発行
            let event = AuditEvent {
                id: format!("event-{}", Uuid::new_v4()),
                event_type: AuditEventType::AuditStarted,
                audit_id: Some(audit_id.clone()),
                timestamp: now,
                data: serde_json::json!({
                    "name": name,
                    "target": target,
                    "audit_type": format!("{:?}", audit_type),
                }),
            };
            
            if let Err(e) = self.event_tx.send(event).await {
                error!("Failed to send audit started event: {}", e);
            }
            
            // 監査を開始
            self.start_audit(&audit_id).await?;
        }
        
        // メトリクスを更新
        self.metrics.increment_counter("security_audits_created");
        
        Ok(audit_id)
    }
    
    /// 監査を開始
    async fn start_audit(&mut self, audit_id: &str) -> Result<(), Error> {
        // 監査が存在するかチェック
        let audit = self.audit_results.get_mut(audit_id)
            .ok_or_else(|| Error::NotFound(format!("Audit not found: {}", audit_id)))?;
        
        // 監査が開始可能かチェック
        if audit.status != AuditStatus::Scheduled && audit.status != AuditStatus::Running {
            return Err(Error::InvalidState(format!("Audit is not in scheduled or running state: {}", audit_id)));
        }
        
        // 監査タイプに基づいて処理を実行
        match audit.audit_type {
            AuditType::VulnerabilityScan => {
                // 脆弱性スキャンを実行
                let scan_results = self.scanner.scan_target(&audit.target).await?;
                audit.scan_results = Some(scan_results);
            },
            AuditType::ComplianceCheck => {
                // コンプライアンスチェックを実行
                let compliance_results = self.compliance_checker.check_compliance(&audit.target).await?;
                audit.compliance_results = Some(compliance_results);
            },
            AuditType::CodeAnalysis => {
                // コード分析を実行
                let code_analysis_results = self.code_analyzer.analyze_code(&audit.target).await?;
                audit.code_analysis_results = Some(code_analysis_results);
            },
            AuditType::PenetrationTest => {
                // ペネトレーションテストを実行
                let pentest_results = self.pentester.run_pentest(&audit.target).await?;
                audit.pentest_results = Some(pentest_results);
            },
            AuditType::Comprehensive => {
                // 総合監査を実行
                let scan_results = self.scanner.scan_target(&audit.target).await?;
                let compliance_results = self.compliance_checker.check_compliance(&audit.target).await?;
                let code_analysis_results = self.code_analyzer.analyze_code(&audit.target).await?;
                let pentest_results = self.pentester.run_pentest(&audit.target).await?;
                
                audit.scan_results = Some(scan_results);
                audit.compliance_results = Some(compliance_results);
                audit.code_analysis_results = Some(code_analysis_results);
                audit.pentest_results = Some(pentest_results);
            },
            AuditType::Custom(ref custom_type) => {
                // カスタム監査を実行
                info!("Running custom audit type: {}", custom_type);
                
                // カスタム監査の実装に応じて処理
                let scan_results = self.scanner.scan_target(&audit.target).await?;
                audit.scan_results = Some(scan_results);
            },
        }
        
        // リスク評価を実行
        let risk_assessment = self.assess_risk(audit).await?;
        audit.risk_assessment = Some(risk_assessment);
        
        // 修復計画を作成
        let remediation_actions = self.create_remediation_plan(audit).await?;
        audit.remediation_plan = Some(remediation_actions);
        
        // 総合スコアを計算
        audit.overall_score = Some(self.calculate_overall_score(audit));
        
        // 監査を完了
        audit.status = AuditStatus::Completed;
        audit.completed_at = Some(chrono::Utc::now());
        
        // ストレージに保存
        let storage = self.storage_manager.get_storage("audit_results")?;
        storage.put(&format!("audit:{}", audit_id), audit)?;
        
        // 監査履歴に追加
        self.audit_history.add_audit_record(audit_id.to_string(), audit.clone()).await?;
        
        // 監査完了イベントを発行
        let event = AuditEvent {
            id: format!("event-{}", Uuid::new_v4()),
            event_type: AuditEventType::AuditCompleted,
            audit_id: Some(audit_id.to_string()),
            timestamp: chrono::Utc::now(),
            data: serde_json::json!({
                "name": audit.name,
                "target": audit.target,
                "audit_type": format!("{:?}", audit.audit_type),
                "overall_score": audit.overall_score,
            }),
        };
        
        if let Err(e) = self.event_tx.send(event).await {
            error!("Failed to send audit completed event: {}", e);
        }
        
        // メトリクスを更新
        self.metrics.increment_counter("security_audits_completed");
        
        info!("Completed audit: {} with score: {:?}", audit_id, audit.overall_score);
        
        Ok(())
    }
    
    /// リスク評価を実行
    async fn assess_risk(&self, audit: &AuditResult) -> Result<RiskAssessment, Error> {
        // リスク評価の入力を準備
        let mut vulnerabilities = Vec::new();
        let mut compliance_violations = Vec::new();
        let mut code_issues = Vec::new();
        
        // 脆弱性スキャン結果を追加
        if let Some(scan_results) = &audit.scan_results {
            for result in scan_results {
                vulnerabilities.push(result.clone());
            }
        }
        
        // コンプライアンス結果を追加
        if let Some(compliance_results) = &audit.compliance_results {
            for result in compliance_results {
                if !result.compliant {
                    compliance_violations.push(result.clone());
                }
            }
        }
        
        // コード分析結果を追加
        if let Some(code_analysis_results) = &audit.code_analysis_results {
            for result in code_analysis_results {
                code_issues.extend(result.issues.clone());
            }
        }
        
        // リスク評価を実行
        self.risk_assessor.assess_risk(
            &audit.target,
            &vulnerabilities,
            &compliance_violations,
            &code_issues,
        ).await
    }
    
    /// 修復計画を作成
    async fn create_remediation_plan(&self, audit: &AuditResult) -> Result<Vec<RemediationAction>, Error> {
        // 修復計画の入力を準備
        let mut vulnerabilities = Vec::new();
        let mut compliance_violations = Vec::new();
        let mut code_issues = Vec::new();
        
        // 脆弱性スキャン結果を追加
        if let Some(scan_results) = &audit.scan_results {
            for result in scan_results {
                vulnerabilities.push(result.clone());
            }
        }
        
        // コンプライアンス結果を追加
        if let Some(compliance_results) = &audit.compliance_results {
            for result in compliance_results {
                if !result.compliant {
                    compliance_violations.push(result.clone());
                }
            }
        }
        
        // コード分析結果を追加
        if let Some(code_analysis_results) = &audit.code_analysis_results {
            for result in code_analysis_results {
                code_issues.extend(result.issues.clone());
            }
        }
        
        // 修復計画を作成
        self.remediation_manager.create_plan(
            &audit.id,
            &audit.target,
            &vulnerabilities,
            &compliance_violations,
            &code_issues,
        ).await
    }
    
    /// 総合スコアを計算
    fn calculate_overall_score(&self, audit: &AuditResult) -> u32 {
        let mut score = 100;
        
        // 脆弱性スキャン結果からスコアを減算
        if let Some(scan_results) = &audit.scan_results {
            for result in scan_results {
                match result.level {
                    VulnerabilityLevel::Critical => score = score.saturating_sub(20),
                    VulnerabilityLevel::High => score = score.saturating_sub(10),
                    VulnerabilityLevel::Medium => score = score.saturating_sub(5),
                    VulnerabilityLevel::Low => score = score.saturating_sub(2),
                    VulnerabilityLevel::Info => score = score.saturating_sub(0),
                }
            }
        }
        
        // コンプライアンス結果からスコアを減算
        if let Some(compliance_results) = &audit.compliance_results {
            for result in compliance_results {
                if !result.compliant {
                    match result.severity {
                        RiskLevel::Critical => score = score.saturating_sub(15),
                        RiskLevel::High => score = score.saturating_sub(10),
                        RiskLevel::Medium => score = score.saturating_sub(5),
                        RiskLevel::Low => score = score.saturating_sub(2),
                        RiskLevel::Info => score = score.saturating_sub(0),
                    }
                }
            }
        }
        
        // コード分析結果からスコアを減算
        if let Some(code_analysis_results) = &audit.code_analysis_results {
            for result in code_analysis_results {
                for issue in &result.issues {
                    match issue.severity {
                        RiskLevel::Critical => score = score.saturating_sub(10),
                        RiskLevel::High => score = score.saturating_sub(7),
                        RiskLevel::Medium => score = score.saturating_sub(4),
                        RiskLevel::Low => score = score.saturating_sub(1),
                        RiskLevel::Info => score = score.saturating_sub(0),
                    }
                }
            }
        }
        
        score
    }
    
    /// 監査レポートを生成
    pub async fn generate_report(
        &self,
        audit_id: &str,
        format: ReportFormat,
    ) -> Result<Vec<u8>, Error> {
        // 監査が存在するかチェック
        let audit = self.audit_results.get(audit_id)
            .ok_or_else(|| Error::NotFound(format!("Audit not found: {}", audit_id)))?;
        
        // 監査が完了しているかチェック
        if audit.status != AuditStatus::Completed {
            return Err(Error::InvalidState(format!("Audit is not completed: {}", audit_id)));
        }
        
        // レポートを生成
        let report = self.reporter.generate_report(audit, format).await?;
        
        info!("Generated {} report for audit: {}", format, audit_id);
        
        Ok(report)
    }
    
    /// 修復アクションを完了
    pub async fn complete_remediation_action(
        &mut self,
        audit_id: &str,
        action_id: &str,
        notes: Option<&str>,
    ) -> Result<(), Error> {
        // 監査が存在するかチェック
        let audit = self.audit_results.get_mut(audit_id)
            .ok_or_else(|| Error::NotFound(format!("Audit not found: {}", audit_id)))?;
        
        // 修復計画が存在するかチェック
        let remediation_plan = audit.remediation_plan.as_mut()
            .ok_or_else(|| Error::InvalidState(format!("Audit has no remediation plan: {}", audit_id)))?;
        
        // アクションを検索
        let action = remediation_plan.iter_mut()
            .find(|a| a.id == action_id)
            .ok_or_else(|| Error::NotFound(format!("Remediation action not found: {}", action_id)))?;
        
        // アクションを完了
        action.status = RemediationStatus::Completed;
        action.completed_at = Some(chrono::Utc::now());
        
        if let Some(notes) = notes {
            action.notes = Some(notes.to_string());
        }
        
        // ストレージに保存
        let storage = self.storage_manager.get_storage("audit_results")?;
        storage.put(&format!("audit:{}", audit_id), audit)?;
        
        info!("Completed remediation action: {} for audit: {}", action_id, audit_id);
        
        Ok(())
    }
    
    /// 監査を取得
    pub fn get_audit(&self, audit_id: &str) -> Result<&AuditResult, Error> {
        self.audit_results.get(audit_id)
            .ok_or_else(|| Error::NotFound(format!("Audit not found: {}", audit_id)))
    }
    
    /// すべての監査IDを取得
    pub fn get_all_audit_ids(&self) -> Vec<AuditId> {
        self.audit_results.keys().cloned().collect()
    }
    
    /// 統計を取得
    pub fn get_stats(&self) -> AuditStats {
        let mut completed_audits = 0;
        let mut failed_audits = 0;
        let mut scheduled_audits = 0;
        let mut running_audits = 0;
        let mut detected_vulnerabilities = 0;
        let mut high_risk_vulnerabilities = 0;
        let mut medium_risk_vulnerabilities = 0;
        let mut low_risk_vulnerabilities = 0;
        let mut compliance_violations = 0;
        let mut code_issues = 0;
        let mut remediation_actions = 0;
        let mut completed_remediation_actions = 0;
        let mut total_score = 0;
        let mut score_count = 0;
        
        // 監査統計を計算
        for audit in self.audit_results.values() {
            match audit.status {
                AuditStatus::Completed => completed_audits += 1,
                AuditStatus::Failed => failed_audits += 1,
                AuditStatus::Scheduled => scheduled_audits += 1,
                AuditStatus::Running => running_audits += 1,
                _ => {}
            }
            
            // 脆弱性統計
            if let Some(scan_results) = &audit.scan_results {
                detected_vulnerabilities += scan_results.len();
                
                for result in scan_results {
                    match result.level {
                        VulnerabilityLevel::Critical | VulnerabilityLevel::High => high_risk_vulnerabilities += 1,
                        VulnerabilityLevel::Medium => medium_risk_vulnerabilities += 1,
                        VulnerabilityLevel::Low | VulnerabilityLevel::Info => low_risk_vulnerabilities += 1,
                    }
                }
            }
            
            // コンプライアンス統計
            if let Some(compliance_results) = &audit.compliance_results {
                for result in compliance_results {
                    if !result.compliant {
                        compliance_violations += 1;
                    }
                }
            }
            
            // コード問題統計
            if let Some(code_analysis_results) = &audit.code_analysis_results {
                for result in code_analysis_results {
                    code_issues += result.issues.len();
                }
            }
            
            // 修復アクション統計
            if let Some(remediation_plan) = &audit.remediation_plan {
                remediation_actions += remediation_plan.len();
                
                for action in remediation_plan {
                    if action.status == RemediationStatus::Completed {
                        completed_remediation_actions += 1;
                    }
                }
            }
            
            // スコア統計
            if let Some(score) = audit.overall_score {
                total_score += score;
                score_count += 1;
            }
        }
        
        // 平均スコアを計算
        let average_audit_score = if score_count > 0 {
            total_score as f64 / score_count as f64
        } else {
            0.0
        };
        
        AuditStats {
            total_audits: self.audit_results.len(),
            completed_audits,
            failed_audits,
            scheduled_audits,
            running_audits,
            detected_vulnerabilities,
            high_risk_vulnerabilities,
            medium_risk_vulnerabilities,
            low_risk_vulnerabilities,
            compliance_violations,
            code_issues,
            remediation_actions,
            completed_remediation_actions,
            average_audit_score,
            metadata: HashMap::new(),
        }
    }
    
    /// 設定を取得
    pub fn get_config(&self) -> &AuditConfig {
        &self.config
    }
    
    /// 設定を更新
    pub fn update_config(&mut self, config: AuditConfig) {
        self.config = config.clone();
        self.scanner.update_config(config.scanner_config);
        self.reporter.update_config(config.reporter_config);
        self.compliance_checker.update_config(config.compliance_config);
        self.pentester.update_config(config.pentester_config);
        self.code_analyzer.update_config(config.code_analyzer_config);
        self.risk_assessor.update_config(config.risk_assessor_config);
        self.remediation_manager.update_config(config.remediation_config);
        self.audit_history.update_config(config.history_config);
        self.scheduler.update_config(config.scheduler_config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::test_utils::create_test_storage_manager;
    use crate::crypto::test_utils::create_test_crypto_manager;
    
    #[test]
    fn test_audit_manager_creation() {
        let config = AuditConfig::default();
        let storage_manager = Arc::new(create_test_storage_manager());
        let crypto_manager = Arc::new(create_test_crypto_manager());
        let metrics = Arc::new(MetricsCollector::new("audit"));
        
        let manager = AuditManager::new(
            config,
            storage_manager,
            crypto_manager,
            metrics,
        );
        
        assert!(!manager.running);
        assert!(manager.audit_results.is_empty());
    }
    
    #[tokio::test]
    async fn test_calculate_overall_score() {
        let config = AuditConfig::default();
        let storage_manager = Arc::new(create_test_storage_manager());
        let crypto_manager = Arc::new(create_test_crypto_manager());
        let metrics = Arc::new(MetricsCollector::new("audit"));
        
        let manager = AuditManager::new(
            config,
            storage_manager,
            crypto_manager,
            metrics,
        );
        
        // テスト用の監査結果を作成
        let mut audit = AuditResult {
            id: "test-audit".to_string(),
            name: "Test Audit".to_string(),
            target: "test-target".to_string(),
            audit_type: AuditType::Comprehensive,
            status: AuditStatus::Completed,
            started_at: chrono::Utc::now(),
            completed_at: Some(chrono::Utc::now()),
            scan_results: Some(vec![
                ScanResult {
                    id: "vuln-1".to_string(),
                    name: "Critical Vulnerability".to_string(),
                    description: "A critical vulnerability".to_string(),
                    level: VulnerabilityLevel::Critical,
                    location: "system/core".to_string(),
                    details: HashMap::new(),
                },
                ScanResult {
                    id: "vuln-2".to_string(),
                    name: "High Vulnerability".to_string(),
                    description: "A high vulnerability".to_string(),
                    level: VulnerabilityLevel::High,
                    location: "system/auth".to_string(),
                    details: HashMap::new(),
                },
            ]),
            compliance_results: None,
            code_analysis_results: None,
            pentest_results: None,
            risk_assessment: None,
            remediation_plan: None,
            overall_score: None,
            metadata: HashMap::new(),
        };
        
        // スコアを計算
        let score = manager.calculate_overall_score(&audit);
        
        // 期待されるスコア: 100 - 20 (Critical) - 10 (High) = 70
        assert_eq!(score, 70);
        
        // コンプライアンス結果を追加
        audit.compliance_results = Some(vec![
            ComplianceResult {
                standard: ComplianceStandard::PCI_DSS,
                requirement: "Requirement 1.1".to_string(),
                compliant: false,
                severity: RiskLevel::High,
                details: "Failed compliance check".to_string(),
            },
        ]);
        
        // スコアを再計算
        let score = manager.calculate_overall_score(&audit);
        
        // 期待されるスコア: 100 - 20 (Critical) - 10 (High) - 10 (High Compliance) = 60
        assert_eq!(score, 60);
    }
    
    #[tokio::test]
    async fn test_get_stats() {
        let config = AuditConfig::default();
        let storage_manager = Arc::new(create_test_storage_manager());
        let crypto_manager = Arc::new(create_test_crypto_manager());
        let metrics = Arc::new(MetricsCollector::new("audit"));
        
        let manager = AuditManager::new(
            config,
            storage_manager,
            crypto_manager,
            metrics,
        );
        
        // 統計を取得
        let stats = manager.get_stats();
        
        assert_eq!(stats.total_audits, 0);
        assert_eq!(stats.completed_audits, 0);
        assert_eq!(stats.failed_audits, 0);
        assert_eq!(stats.scheduled_audits, 0);
        assert_eq!(stats.running_audits, 0);
        assert_eq!(stats.detected_vulnerabilities, 0);
    }
}