use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::error::Error;
use crate::shard::{ShardId, ShardManager};
use crate::transaction::{Transaction, TransactionStatus, TransactionType};

/// トランザクション分析結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionAnalysis {
    /// トランザクションID
    pub transaction_id: String,
    /// 処理時間（ミリ秒）
    pub processing_time_ms: u64,
    /// 経由したシャード
    pub shards_traversed: Vec<ShardId>,
    /// 処理ステップ
    pub processing_steps: Vec<ProcessingStep>,
    /// 合計コスト
    pub total_cost: f64,
    /// ステータス
    pub status: TransactionStatus,
    /// エラー（存在する場合）
    pub error: Option<String>,
    /// パフォーマンススコア（0-100）
    pub performance_score: u8,
    /// 最適化提案
    pub optimization_suggestions: Vec<String>,
    /// 異常フラグ
    pub anomaly_detected: bool,
    /// 異常の詳細
    pub anomaly_details: Option<String>,
    /// 関連トランザクション
    pub related_transactions: Vec<String>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// 処理ステップ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStep {
    /// ステップ名
    pub name: String,
    /// 開始時刻
    pub start_time: DateTime<Utc>,
    /// 終了時刻
    pub end_time: DateTime<Utc>,
    /// 処理時間（ミリ秒）
    pub duration_ms: u64,
    /// 処理シャード
    pub shard_id: Option<ShardId>,
    /// ステータス
    pub status: StepStatus,
    /// 詳細
    pub details: Option<String>,
}

/// ステップステータス
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepStatus {
    /// 成功
    Success,
    /// 警告
    Warning,
    /// エラー
    Error,
    /// スキップ
    Skipped,
}

/// トランザクション分析設定
#[derive(Debug, Clone)]
pub struct TransactionAnalyzerConfig {
    /// 詳細分析を有効にするかどうか
    pub detailed_analysis: bool,
    /// 履歴保持期間（秒）
    pub history_retention_seconds: u64,
    /// 異常検出を有効にするかどうか
    pub anomaly_detection: bool,
    /// 最適化提案を有効にするかどうか
    pub optimization_suggestions: bool,
    /// パフォーマンススコアリングを有効にするかどうか
    pub performance_scoring: bool,
    /// 関連トランザクション分析を有効にするかどうか
    pub related_transaction_analysis: bool,
    /// メタデータ収集を有効にするかどうか
    pub metadata_collection: bool,
}

impl Default for TransactionAnalyzerConfig {
    fn default() -> Self {
        Self {
            detailed_analysis: true,
            history_retention_seconds: 86400 * 7, // 1週間
            anomaly_detection: true,
            optimization_suggestions: true,
            performance_scoring: true,
            related_transaction_analysis: true,
            metadata_collection: true,
        }
    }
}

/// トランザクション分析統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionAnalyticsSummary {
    /// 分析したトランザクション数
    pub transactions_analyzed: u64,
    /// 平均処理時間（ミリ秒）
    pub avg_processing_time_ms: f64,
    /// 最大処理時間（ミリ秒）
    pub max_processing_time_ms: u64,
    /// 最小処理時間（ミリ秒）
    pub min_processing_time_ms: u64,
    /// 成功率（%）
    pub success_rate: f64,
    /// トランザクションタイプ別の統計
    pub stats_by_type: HashMap<TransactionType, TypeStats>,
    /// シャード別の統計
    pub stats_by_shard: HashMap<ShardId, ShardStats>,
    /// 時間帯別の統計
    pub stats_by_hour: HashMap<u8, HourlyStats>,
    /// 異常検出数
    pub anomalies_detected: u64,
    /// 平均パフォーマンススコア
    pub avg_performance_score: f64,
}

/// トランザクションタイプ別の統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeStats {
    /// トランザクション数
    pub count: u64,
    /// 平均処理時間（ミリ秒）
    pub avg_processing_time_ms: f64,
    /// 成功率（%）
    pub success_rate: f64,
    /// 平均コスト
    pub avg_cost: f64,
}

/// シャード別の統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardStats {
    /// 処理したトランザクション数
    pub transactions_processed: u64,
    /// 平均処理時間（ミリ秒）
    pub avg_processing_time_ms: f64,
    /// 最大処理時間（ミリ秒）
    pub max_processing_time_ms: u64,
    /// エラー数
    pub errors: u64,
    /// 負荷スコア（0-100）
    pub load_score: u8,
}

/// 時間帯別の統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyStats {
    /// トランザクション数
    pub count: u64,
    /// 平均処理時間（ミリ秒）
    pub avg_processing_time_ms: f64,
    /// 成功率（%）
    pub success_rate: f64,
}

/// トランザクションフロー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionFlow {
    /// トランザクションID
    pub transaction_id: String,
    /// 送信元アドレス
    pub from_address: String,
    /// 送信先アドレス
    pub to_address: String,
    /// 金額
    pub amount: f64,
    /// 送信元シャード
    pub from_shard: ShardId,
    /// 送信先シャード
    pub to_shard: ShardId,
    /// 経由シャード
    pub via_shards: Vec<ShardId>,
    /// 開始時刻
    pub start_time: DateTime<Utc>,
    /// 終了時刻
    pub end_time: DateTime<Utc>,
    /// ステータス
    pub status: TransactionStatus,
    /// 処理ステップ
    pub steps: Vec<ProcessingStep>,
}

/// トランザクション分析器
pub struct TransactionAnalyzer {
    /// 設定
    config: TransactionAnalyzerConfig,
    /// 分析結果履歴
    analysis_history: Arc<Mutex<HashMap<String, TransactionAnalysis>>>,
    /// 統計情報
    stats: Arc<Mutex<TransactionAnalyticsSummary>>,
    /// シャードマネージャ
    shard_manager: Arc<ShardManager>,
    /// 異常検出モデル
    anomaly_detector: Option<AnomalyDetector>,
    /// 最適化エンジン
    optimization_engine: Option<OptimizationEngine>,
}

/// 異常検出モデル（簡易実装）
struct AnomalyDetector {
    /// 処理時間の閾値（ミリ秒）
    processing_time_threshold_ms: u64,
    /// エラー率の閾値（%）
    error_rate_threshold: f64,
    /// 異常パターン
    anomaly_patterns: Vec<String>,
}

/// 最適化エンジン（簡易実装）
struct OptimizationEngine {
    /// 最適化ルール
    optimization_rules: Vec<OptimizationRule>,
}

/// 最適化ルール
struct OptimizationRule {
    /// ルール名
    name: String,
    /// 条件
    condition: Box<dyn Fn(&TransactionAnalysis) -> bool + Send + Sync>,
    /// 提案
    suggestion: String,
}

impl TransactionAnalyzer {
    /// 新しいトランザクション分析器を作成
    pub fn new(
        shard_manager: Arc<ShardManager>,
        config: Option<TransactionAnalyzerConfig>,
    ) -> Self {
        let config = config.unwrap_or_default();

        // 初期統計情報
        let initial_stats = TransactionAnalyticsSummary {
            transactions_analyzed: 0,
            avg_processing_time_ms: 0.0,
            max_processing_time_ms: 0,
            min_processing_time_ms: u64::MAX,
            success_rate: 0.0,
            stats_by_type: HashMap::new(),
            stats_by_shard: HashMap::new(),
            stats_by_hour: HashMap::new(),
            anomalies_detected: 0,
            avg_performance_score: 0.0,
        };

        // 異常検出モデル
        let anomaly_detector = if config.anomaly_detection {
            Some(AnomalyDetector {
                processing_time_threshold_ms: 5000, // 5秒
                error_rate_threshold: 10.0,         // 10%
                anomaly_patterns: vec![
                    "multiple_failures".to_string(),
                    "timeout".to_string(),
                    "unexpected_state".to_string(),
                ],
            })
        } else {
            None
        };

        // 最適化エンジン
        let optimization_engine = if config.optimization_suggestions {
            let mut engine = OptimizationEngine {
                optimization_rules: Vec::new(),
            };

            // ルールを追加
            engine.optimization_rules.push(OptimizationRule {
                name: "high_latency".to_string(),
                condition: Box::new(|analysis: &TransactionAnalysis| {
                    analysis.processing_time_ms > 1000
                }),
                suggestion: "シャード間の通信を最適化することで、レイテンシを削減できます。"
                    .to_string(),
            });

            engine.optimization_rules.push(OptimizationRule {
                name: "many_shards".to_string(),
                condition: Box::new(|analysis: &TransactionAnalysis| {
                    analysis.shards_traversed.len() > 3
                }),
                suggestion: "トランザクションが多くのシャードを経由しています。データの局所性を高めることを検討してください。".to_string(),
            });

            Some(engine)
        } else {
            None
        };

        Self {
            config,
            analysis_history: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(initial_stats)),
            shard_manager,
            anomaly_detector,
            optimization_engine,
        }
    }

    /// トランザクションを分析
    pub fn analyze_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<TransactionAnalysis, Error> {
        let start_time = Instant::now();

        // 基本情報を収集
        let transaction_id = transaction.id.clone();
        let status = transaction.status.clone();

        // 処理ステップを収集
        let processing_steps = self.collect_processing_steps(transaction)?;

        // 経由したシャードを収集
        let shards_traversed = self.collect_shards_traversed(transaction)?;

        // 処理時間を計算
        let processing_time_ms = if let (Some(first), Some(last)) =
            (processing_steps.first(), processing_steps.last())
        {
            (last.end_time - first.start_time).num_milliseconds() as u64
        } else {
            0
        };

        // コストを計算
        let total_cost = self.calculate_cost(transaction, &processing_steps)?;

        // 異常を検出
        let (anomaly_detected, anomaly_details) =
            self.detect_anomalies(transaction, &processing_steps)?;

        // パフォーマンススコアを計算
        let performance_score =
            self.calculate_performance_score(transaction, processing_time_ms, &shards_traversed)?;

        // 最適化提案を生成
        let optimization_suggestions = self.generate_optimization_suggestions(
            transaction,
            processing_time_ms,
            &shards_traversed,
            &processing_steps,
        )?;

        // 関連トランザクションを検索
        let related_transactions = self.find_related_transactions(transaction)?;

        // メタデータを収集
        let metadata = if self.config.metadata_collection {
            self.collect_metadata(transaction)?
        } else {
            HashMap::new()
        };

        // 分析結果を作成
        let analysis = TransactionAnalysis {
            transaction_id,
            processing_time_ms,
            shards_traversed,
            processing_steps,
            total_cost,
            status,
            error: transaction.error.clone(),
            performance_score,
            optimization_suggestions,
            anomaly_detected,
            anomaly_details,
            related_transactions,
            metadata,
        };

        // 履歴に追加
        if self.config.detailed_analysis {
            let mut history = self.analysis_history.lock().unwrap();
            history.insert(transaction.id.clone(), analysis.clone());

            // 古いエントリを削除
            let retention_time =
                chrono::Duration::seconds(self.config.history_retention_seconds as i64);
            let cutoff_time = Utc::now() - retention_time;

            history.retain(|_, analysis| {
                if let Some(step) = analysis.processing_steps.first() {
                    step.start_time > cutoff_time
                } else {
                    true
                }
            });
        }

        // 統計情報を更新
        self.update_statistics(&analysis)?;

        Ok(analysis)
    }

    /// 処理ステップを収集
    fn collect_processing_steps(
        &self,
        transaction: &Transaction,
    ) -> Result<Vec<ProcessingStep>, Error> {
        // 実際の実装では、トランザクションのログから処理ステップを収集
        // ここでは簡易的な実装として、ダミーデータを返す

        let mut steps = Vec::new();

        // 検証ステップ
        let verification_start = Utc::now() - chrono::Duration::seconds(5);
        let verification_end = verification_start + chrono::Duration::milliseconds(100);
        steps.push(ProcessingStep {
            name: "検証".to_string(),
            start_time: verification_start,
            end_time: verification_end,
            duration_ms: 100,
            shard_id: Some(transaction.shard_id.clone()),
            status: StepStatus::Success,
            details: Some("署名検証成功".to_string()),
        });

        // 実行ステップ
        let execution_start = verification_end;
        let execution_end = execution_start + chrono::Duration::milliseconds(300);
        steps.push(ProcessingStep {
            name: "実行".to_string(),
            start_time: execution_start,
            end_time: execution_end,
            duration_ms: 300,
            shard_id: Some(transaction.shard_id.clone()),
            status: StepStatus::Success,
            details: Some("トランザクション実行成功".to_string()),
        });

        // コミットステップ
        let commit_start = execution_end;
        let commit_end = commit_start + chrono::Duration::milliseconds(200);
        steps.push(ProcessingStep {
            name: "コミット".to_string(),
            start_time: commit_start,
            end_time: commit_end,
            duration_ms: 200,
            shard_id: Some(transaction.shard_id.clone()),
            status: StepStatus::Success,
            details: Some("状態更新コミット成功".to_string()),
        });

        Ok(steps)
    }

    /// 経由したシャードを収集
    fn collect_shards_traversed(&self, transaction: &Transaction) -> Result<Vec<ShardId>, Error> {
        // 実際の実装では、トランザクションのログから経由したシャードを収集
        // ここでは簡易的な実装として、ダミーデータを返す

        Ok(vec![transaction.shard_id.clone()])
    }

    /// コストを計算
    fn calculate_cost(
        &self,
        transaction: &Transaction,
        steps: &[ProcessingStep],
    ) -> Result<f64, Error> {
        // 実際の実装では、トランザクションの複雑さや処理時間からコストを計算
        // ここでは簡易的な実装として、ダミーデータを返す

        let base_cost = 0.001; // 基本コスト
        let time_cost = steps
            .iter()
            .map(|s| s.duration_ms as f64 * 0.0001)
            .sum::<f64>(); // 時間コスト
        let complexity_cost = 0.002; // 複雑さコスト

        Ok(base_cost + time_cost + complexity_cost)
    }

    /// 異常を検出
    fn detect_anomalies(
        &self,
        transaction: &Transaction,
        steps: &[ProcessingStep],
    ) -> Result<(bool, Option<String>), Error> {
        // 異常検出が無効の場合は何もしない
        if !self.config.anomaly_detection || self.anomaly_detector.is_none() {
            return Ok((false, None));
        }

        let detector = self.anomaly_detector.as_ref().unwrap();

        // 処理時間の異常を検出
        let total_time = steps.iter().map(|s| s.duration_ms).sum::<u64>();
        if total_time > detector.processing_time_threshold_ms {
            return Ok((
                true,
                Some(format!("処理時間が閾値を超えています: {}ms", total_time)),
            ));
        }

        // エラーステップを検出
        let error_steps = steps
            .iter()
            .filter(|s| s.status == StepStatus::Error)
            .count();
        if error_steps > 0 {
            return Ok((
                true,
                Some(format!("{}個のエラーステップがあります", error_steps)),
            ));
        }

        Ok((false, None))
    }

    /// パフォーマンススコアを計算
    fn calculate_performance_score(
        &self,
        transaction: &Transaction,
        processing_time_ms: u64,
        shards_traversed: &[ShardId],
    ) -> Result<u8, Error> {
        // パフォーマンススコアリングが無効の場合は100を返す
        if !self.config.performance_scoring {
            return Ok(100);
        }

        // 基本スコア
        let mut score = 100;

        // 処理時間によるスコア調整
        if processing_time_ms > 5000 {
            score -= 50;
        } else if processing_time_ms > 1000 {
            score -= 20;
        } else if processing_time_ms > 500 {
            score -= 10;
        }

        // 経由シャード数によるスコア調整
        if shards_traversed.len() > 5 {
            score -= 30;
        } else if shards_traversed.len() > 3 {
            score -= 15;
        } else if shards_traversed.len() > 1 {
            score -= 5;
        }

        // エラーによるスコア調整
        if transaction.error.is_some() {
            score -= 40;
        }

        // スコアの範囲を0-100に制限
        score = score.max(0).min(100);

        Ok(score as u8)
    }

    /// 最適化提案を生成
    fn generate_optimization_suggestions(
        &self,
        transaction: &Transaction,
        processing_time_ms: u64,
        shards_traversed: &[ShardId],
        steps: &[ProcessingStep],
    ) -> Result<Vec<String>, Error> {
        // 最適化提案が無効の場合は空のリストを返す
        if !self.config.optimization_suggestions || self.optimization_engine.is_none() {
            return Ok(Vec::new());
        }

        let engine = self.optimization_engine.as_ref().unwrap();
        let mut suggestions = Vec::new();

        // ダミーの分析結果を作成
        let dummy_analysis = TransactionAnalysis {
            transaction_id: transaction.id.clone(),
            processing_time_ms,
            shards_traversed: shards_traversed.to_vec(),
            processing_steps: steps.to_vec(),
            total_cost: 0.0,
            status: transaction.status.clone(),
            error: transaction.error.clone(),
            performance_score: 0,
            optimization_suggestions: Vec::new(),
            anomaly_detected: false,
            anomaly_details: None,
            related_transactions: Vec::new(),
            metadata: HashMap::new(),
        };

        // 各ルールをチェック
        for rule in &engine.optimization_rules {
            if (rule.condition)(&dummy_analysis) {
                suggestions.push(rule.suggestion.clone());
            }
        }

        Ok(suggestions)
    }

    /// 関連トランザクションを検索
    fn find_related_transactions(&self, transaction: &Transaction) -> Result<Vec<String>, Error> {
        // 関連トランザクション分析が無効の場合は空のリストを返す
        if !self.config.related_transaction_analysis {
            return Ok(Vec::new());
        }

        // 実際の実装では、トランザクションの送信元/送信先や時間的な近さから関連トランザクションを検索
        // ここでは簡易的な実装として、ダミーデータを返す

        Ok(Vec::new())
    }

    /// メタデータを収集
    fn collect_metadata(
        &self,
        transaction: &Transaction,
    ) -> Result<HashMap<String, String>, Error> {
        // メタデータ収集が無効の場合は空のマップを返す
        if !self.config.metadata_collection {
            return Ok(HashMap::new());
        }

        // 実際の実装では、トランザクションの詳細情報からメタデータを収集
        // ここでは簡易的な実装として、ダミーデータを返す

        let mut metadata = HashMap::new();
        metadata.insert(
            "transaction_type".to_string(),
            format!("{:?}", transaction.transaction_type),
        );
        metadata.insert("timestamp".to_string(), Utc::now().to_rfc3339());

        Ok(metadata)
    }

    /// 統計情報を更新
    fn update_statistics(&self, analysis: &TransactionAnalysis) -> Result<(), Error> {
        let mut stats = self.stats.lock().unwrap();

        // トランザクション数を更新
        stats.transactions_analyzed += 1;

        // 処理時間の統計を更新
        let old_avg = stats.avg_processing_time_ms;
        let old_count = stats.transactions_analyzed - 1;
        stats.avg_processing_time_ms = (old_avg * old_count as f64
            + analysis.processing_time_ms as f64)
            / stats.transactions_analyzed as f64;
        stats.max_processing_time_ms = stats
            .max_processing_time_ms
            .max(analysis.processing_time_ms);
        if analysis.processing_time_ms > 0 {
            stats.min_processing_time_ms = stats
                .min_processing_time_ms
                .min(analysis.processing_time_ms);
        }

        // 成功率を更新
        let success_count = if analysis.status == TransactionStatus::Confirmed {
            1
        } else {
            0
        };
        let old_success_rate = stats.success_rate;
        stats.success_rate = (old_success_rate * old_count as f64 + success_count as f64)
            / stats.transactions_analyzed as f64
            * 100.0;

        // 異常検出数を更新
        if analysis.anomaly_detected {
            stats.anomalies_detected += 1;
        }

        // パフォーマンススコアを更新
        let old_avg_score = stats.avg_performance_score;
        stats.avg_performance_score = (old_avg_score * old_count as f64
            + analysis.performance_score as f64)
            / stats.transactions_analyzed as f64;

        Ok(())
    }

    /// 統計情報を取得
    pub fn get_statistics(&self) -> Result<TransactionAnalyticsSummary, Error> {
        let stats = self.stats.lock().unwrap();
        Ok(stats.clone())
    }

    /// トランザクションフローを取得
    pub fn get_transaction_flow(
        &self,
        transaction_id: &str,
    ) -> Result<Option<TransactionFlow>, Error> {
        // 履歴から分析結果を取得
        let history = self.analysis_history.lock().unwrap();
        let analysis = match history.get(transaction_id) {
            Some(a) => a,
            None => return Ok(None),
        };

        // トランザクションの詳細情報を取得
        // 実際の実装では、トランザクションの詳細情報を取得する処理が必要
        // ここでは簡易的な実装として、ダミーデータを返す

        let flow = TransactionFlow {
            transaction_id: transaction_id.to_string(),
            from_address: "0x1234...".to_string(),
            to_address: "0x5678...".to_string(),
            amount: 1.0,
            from_shard: analysis
                .shards_traversed
                .first()
                .cloned()
                .unwrap_or_default(),
            to_shard: analysis
                .shards_traversed
                .last()
                .cloned()
                .unwrap_or_default(),
            via_shards: analysis.shards_traversed.clone(),
            start_time: analysis
                .processing_steps
                .first()
                .map(|s| s.start_time)
                .unwrap_or_else(Utc::now),
            end_time: analysis
                .processing_steps
                .last()
                .map(|s| s.end_time)
                .unwrap_or_else(Utc::now),
            status: analysis.status.clone(),
            steps: analysis.processing_steps.clone(),
        };

        Ok(Some(flow))
    }

    /// 分析結果を取得
    pub fn get_analysis(&self, transaction_id: &str) -> Result<Option<TransactionAnalysis>, Error> {
        let history = self.analysis_history.lock().unwrap();
        Ok(history.get(transaction_id).cloned())
    }

    /// 設定を更新
    pub fn update_config(&mut self, config: TransactionAnalyzerConfig) {
        self.config = config;
    }
}
