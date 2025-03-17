use crate::error::Error;
use crate::transaction::Transaction;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::interval;

/// 異常検出システム
///
/// AIベースの異常検出システム。
/// 不正なトランザクションやネットワーク攻撃を検出し、自動的に対応する。
pub struct AnomalyDetector {
    /// 検出モデル
    model: Box<dyn AnomalyModel + Send + Sync>,
    /// 閾値
    thresholds: HashMap<AnomalyType, f32>,
    /// 履歴データ
    history: Mutex<VecDeque<TransactionMetrics>>,
    /// 検出された異常
    detected_anomalies: Mutex<Vec<DetectedAnomaly>>,
    /// 自動対応ルール
    mitigation_rules: Vec<MitigationRule>,
}

/// 異常の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnomalyType {
    /// 異常なトランザクション量
    UnusualVolume,
    /// 異常なトランザクションパターン
    UnusualPattern,
    /// 悪意のあるアクティビティ
    MaliciousActivity,
    /// ネットワーク分断
    NetworkPartition,
    /// リソース枯渇
    ResourceExhaustion,
    /// 異常なレイテンシ
    UnusualLatency,
}

/// トランザクションメトリクス
#[derive(Debug, Clone)]
pub struct TransactionMetrics {
    /// タイムスタンプ
    timestamp: Instant,
    /// トランザクション数
    transaction_count: usize,
    /// 平均トランザクションサイズ
    avg_transaction_size: usize,
    /// 平均処理時間
    avg_processing_time: Duration,
    /// 拒否されたトランザクション数
    rejected_transaction_count: usize,
    /// 一意のアドレス数
    unique_address_count: usize,
    /// シャード間トランザクション数
    cross_shard_transaction_count: usize,
}

/// 検出された異常
#[derive(Debug, Clone)]
pub struct DetectedAnomaly {
    /// 異常の種類
    anomaly_type: AnomalyType,
    /// 信頼度スコア
    confidence: f32,
    /// 検出時刻
    detected_at: Instant,
    /// 関連するメトリクス
    metrics: TransactionMetrics,
    /// 適用された緩和策
    applied_mitigations: Vec<MitigationAction>,
}

/// 緩和策
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MitigationAction {
    /// トランザクションのスロットリング
    ThrottleTransactions,
    /// 検証閾値の引き上げ
    IncreaseValidationThreshold,
    /// 不審なアドレスのブロック
    BlockSuspiciousAddresses,
    /// 保守的モードへの切り替え
    SwitchToConservativeMode,
    /// 管理者への通知
    AlertAdministrators,
    /// シャードの再バランス
    RebalanceShards,
    /// メモリキャッシュのクリア
    ClearMemoryCache,
}

/// 緩和ルール
struct MitigationRule {
    /// 異常の種類
    anomaly_type: AnomalyType,
    /// 最小信頼度
    min_confidence: f32,
    /// 緩和策
    action: MitigationAction,
}

/// 異常検出モデル
pub trait AnomalyModel {
    /// 異常を検出
    fn detect(
        &self,
        metrics: &TransactionMetrics,
        history: &[TransactionMetrics],
    ) -> HashMap<AnomalyType, f32>;
}

/// 統計ベースの異常検出モデル
pub struct StatisticalModel {
    /// 移動平均ウィンドウサイズ
    window_size: usize,
    /// 標準偏差の倍率
    std_dev_multiplier: f32,
}

impl StatisticalModel {
    /// 新しいStatisticalModelを作成
    pub fn new(window_size: usize, std_dev_multiplier: f32) -> Self {
        Self {
            window_size,
            std_dev_multiplier,
        }
    }
}

impl AnomalyModel for StatisticalModel {
    fn detect(
        &self,
        metrics: &TransactionMetrics,
        history: &[TransactionMetrics],
    ) -> HashMap<AnomalyType, f32> {
        let mut anomalies = HashMap::new();

        // 履歴データが不十分な場合は異常を検出しない
        if history.len() < self.window_size {
            return anomalies;
        }

        // 移動平均と標準偏差を計算
        let recent_history = &history[history.len() - self.window_size..];

        // トランザクション数の統計
        let tx_counts: Vec<usize> = recent_history.iter().map(|m| m.transaction_count).collect();

        let tx_count_mean = mean(&tx_counts);
        let tx_count_std_dev = std_dev(&tx_counts, tx_count_mean);

        // 異常なトランザクション量を検出
        let tx_count_z_score = z_score(
            metrics.transaction_count as f32,
            tx_count_mean,
            tx_count_std_dev,
        );

        if tx_count_z_score.abs() > self.std_dev_multiplier {
            let confidence = confidence_from_z_score(tx_count_z_score, self.std_dev_multiplier);
            anomalies.insert(AnomalyType::UnusualVolume, confidence);
        }

        // 拒否されたトランザクション数の統計
        let rejected_counts: Vec<usize> = recent_history
            .iter()
            .map(|m| m.rejected_transaction_count)
            .collect();

        let rejected_count_mean = mean(&rejected_counts);
        let rejected_count_std_dev = std_dev(&rejected_counts, rejected_count_mean);

        // 悪意のあるアクティビティを検出
        let rejected_count_z_score = z_score(
            metrics.rejected_transaction_count as f32,
            rejected_count_mean,
            rejected_count_std_dev,
        );

        if rejected_count_z_score > self.std_dev_multiplier * 1.5 {
            let confidence =
                confidence_from_z_score(rejected_count_z_score, self.std_dev_multiplier * 1.5);
            anomalies.insert(AnomalyType::MaliciousActivity, confidence);
        }

        // 処理時間の統計
        let processing_times: Vec<f32> = recent_history
            .iter()
            .map(|m| m.avg_processing_time.as_secs_f32())
            .collect();

        let processing_time_mean = mean(&processing_times);
        let processing_time_std_dev = std_dev(&processing_times, processing_time_mean);

        // 異常なレイテンシを検出
        let processing_time_z_score = z_score(
            metrics.avg_processing_time.as_secs_f32(),
            processing_time_mean,
            processing_time_std_dev,
        );

        if processing_time_z_score > self.std_dev_multiplier * 2.0 {
            let confidence =
                confidence_from_z_score(processing_time_z_score, self.std_dev_multiplier * 2.0);
            anomalies.insert(AnomalyType::UnusualLatency, confidence);
        }

        // シャード間トランザクション数の統計
        let cross_shard_counts: Vec<usize> = recent_history
            .iter()
            .map(|m| m.cross_shard_transaction_count)
            .collect();

        let cross_shard_mean = mean(&cross_shard_counts);
        let cross_shard_std_dev = std_dev(&cross_shard_counts, cross_shard_mean);

        // ネットワーク分断を検出
        let cross_shard_z_score = z_score(
            metrics.cross_shard_transaction_count as f32,
            cross_shard_mean,
            cross_shard_std_dev,
        );

        if cross_shard_z_score < -self.std_dev_multiplier * 3.0 {
            let confidence =
                confidence_from_z_score(-cross_shard_z_score, self.std_dev_multiplier * 3.0);
            anomalies.insert(AnomalyType::NetworkPartition, confidence);
        }

        anomalies
    }
}

/// 機械学習ベースの異常検出モデル
pub struct MachineLearningModel {
    /// モデルの重み
    weights: HashMap<String, f32>,
    /// バイアス
    bias: f32,
}

impl MachineLearningModel {
    /// 新しいMachineLearningModelを作成
    pub fn new() -> Self {
        // 実際の実装では、訓練済みモデルをロードする
        // ここでは簡易的な実装として、ハードコードされた重みを使用

        let mut weights = HashMap::new();
        weights.insert("transaction_count".to_string(), 0.8);
        weights.insert("avg_transaction_size".to_string(), 0.2);
        weights.insert("avg_processing_time".to_string(), 0.5);
        weights.insert("rejected_transaction_count".to_string(), 0.9);
        weights.insert("unique_address_count".to_string(), 0.3);
        weights.insert("cross_shard_transaction_count".to_string(), 0.4);

        Self {
            weights,
            bias: -0.5,
        }
    }

    /// 特徴量を抽出
    fn extract_features(&self, metrics: &TransactionMetrics) -> HashMap<String, f32> {
        let mut features = HashMap::new();

        features.insert(
            "transaction_count".to_string(),
            metrics.transaction_count as f32,
        );
        features.insert(
            "avg_transaction_size".to_string(),
            metrics.avg_transaction_size as f32,
        );
        features.insert(
            "avg_processing_time".to_string(),
            metrics.avg_processing_time.as_secs_f32(),
        );
        features.insert(
            "rejected_transaction_count".to_string(),
            metrics.rejected_transaction_count as f32,
        );
        features.insert(
            "unique_address_count".to_string(),
            metrics.unique_address_count as f32,
        );
        features.insert(
            "cross_shard_transaction_count".to_string(),
            metrics.cross_shard_transaction_count as f32,
        );

        features
    }
}

impl AnomalyModel for MachineLearningModel {
    fn detect(
        &self,
        metrics: &TransactionMetrics,
        _history: &[TransactionMetrics],
    ) -> HashMap<AnomalyType, f32> {
        let mut anomalies = HashMap::new();

        // 特徴量を抽出
        let features = self.extract_features(metrics);

        // 異常スコアを計算
        let mut unusual_volume_score = self.bias;
        let mut unusual_pattern_score = self.bias;
        let mut malicious_activity_score = self.bias;
        let mut network_partition_score = self.bias;
        let mut resource_exhaustion_score = self.bias;

        for (feature, value) in &features {
            if let Some(weight) = self.weights.get(feature) {
                unusual_volume_score += weight * value;
                unusual_pattern_score += weight * value * 0.8;
                malicious_activity_score += weight * value * 1.2;
                network_partition_score += weight * value * 0.7;
                resource_exhaustion_score += weight * value * 0.9;
            }
        }

        // シグモイド関数を適用して0-1の範囲に変換
        let unusual_volume_confidence = sigmoid(unusual_volume_score);
        let unusual_pattern_confidence = sigmoid(unusual_pattern_score);
        let malicious_activity_confidence = sigmoid(malicious_activity_score);
        let network_partition_confidence = sigmoid(network_partition_score);
        let resource_exhaustion_confidence = sigmoid(resource_exhaustion_score);

        // 閾値を超える異常を検出
        if unusual_volume_confidence > 0.7 {
            anomalies.insert(AnomalyType::UnusualVolume, unusual_volume_confidence);
        }

        if unusual_pattern_confidence > 0.75 {
            anomalies.insert(AnomalyType::UnusualPattern, unusual_pattern_confidence);
        }

        if malicious_activity_confidence > 0.65 {
            anomalies.insert(
                AnomalyType::MaliciousActivity,
                malicious_activity_confidence,
            );
        }

        if network_partition_confidence > 0.8 {
            anomalies.insert(AnomalyType::NetworkPartition, network_partition_confidence);
        }

        if resource_exhaustion_confidence > 0.7 {
            anomalies.insert(
                AnomalyType::ResourceExhaustion,
                resource_exhaustion_confidence,
            );
        }

        anomalies
    }
}

impl AnomalyDetector {
    /// 新しいAnomalyDetectorを作成
    pub fn new(model: Box<dyn AnomalyModel + Send + Sync>) -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert(AnomalyType::UnusualVolume, 0.7);
        thresholds.insert(AnomalyType::UnusualPattern, 0.75);
        thresholds.insert(AnomalyType::MaliciousActivity, 0.65);
        thresholds.insert(AnomalyType::NetworkPartition, 0.8);
        thresholds.insert(AnomalyType::ResourceExhaustion, 0.7);
        thresholds.insert(AnomalyType::UnusualLatency, 0.75);

        let mitigation_rules = vec![
            MitigationRule {
                anomaly_type: AnomalyType::UnusualVolume,
                min_confidence: 0.7,
                action: MitigationAction::ThrottleTransactions,
            },
            MitigationRule {
                anomaly_type: AnomalyType::UnusualPattern,
                min_confidence: 0.75,
                action: MitigationAction::IncreaseValidationThreshold,
            },
            MitigationRule {
                anomaly_type: AnomalyType::MaliciousActivity,
                min_confidence: 0.65,
                action: MitigationAction::BlockSuspiciousAddresses,
            },
            MitigationRule {
                anomaly_type: AnomalyType::MaliciousActivity,
                min_confidence: 0.9,
                action: MitigationAction::AlertAdministrators,
            },
            MitigationRule {
                anomaly_type: AnomalyType::NetworkPartition,
                min_confidence: 0.8,
                action: MitigationAction::SwitchToConservativeMode,
            },
            MitigationRule {
                anomaly_type: AnomalyType::NetworkPartition,
                min_confidence: 0.9,
                action: MitigationAction::AlertAdministrators,
            },
            MitigationRule {
                anomaly_type: AnomalyType::ResourceExhaustion,
                min_confidence: 0.7,
                action: MitigationAction::ClearMemoryCache,
            },
            MitigationRule {
                anomaly_type: AnomalyType::ResourceExhaustion,
                min_confidence: 0.85,
                action: MitigationAction::RebalanceShards,
            },
            MitigationRule {
                anomaly_type: AnomalyType::UnusualLatency,
                min_confidence: 0.75,
                action: MitigationAction::RebalanceShards,
            },
        ];

        Self {
            model,
            thresholds,
            history: Mutex::new(VecDeque::with_capacity(1000)),
            detected_anomalies: Mutex::new(Vec::new()),
            mitigation_rules,
        }
    }

    /// 統計ベースの異常検出器を作成
    pub fn new_statistical(window_size: usize, std_dev_multiplier: f32) -> Self {
        let model = Box::new(StatisticalModel::new(window_size, std_dev_multiplier));
        Self::new(model)
    }

    /// 機械学習ベースの異常検出器を作成
    pub fn new_ml() -> Self {
        let model = Box::new(MachineLearningModel::new());
        Self::new(model)
    }

    /// メトリクスを追加
    pub fn add_metrics(&self, metrics: TransactionMetrics) {
        let mut history = self.history.lock().unwrap();

        history.push_back(metrics.clone());

        if history.len() > 1000 {
            history.pop_front();
        }
    }

    /// 異常を検出
    pub fn detect(&self, metrics: &TransactionMetrics) -> Vec<(AnomalyType, f32)> {
        let history = self.history.lock().unwrap();
        let history_vec: Vec<TransactionMetrics> = history.iter().cloned().collect();

        let anomalies = self.model.detect(metrics, &history_vec);

        let mut result = Vec::new();

        for (anomaly_type, confidence) in anomalies {
            let threshold = self.thresholds.get(&anomaly_type).unwrap_or(&0.7);

            if confidence > *threshold {
                result.push((anomaly_type, confidence));
            }
        }

        // 検出された異常を記録
        if !result.is_empty() {
            let mitigations = self.determine_mitigations(&result);

            let detected_anomaly = DetectedAnomaly {
                anomaly_type: result[0].0, // 最初の異常タイプを使用
                confidence: result[0].1,   // 最初の信頼度を使用
                detected_at: Instant::now(),
                metrics: metrics.clone(),
                applied_mitigations: mitigations,
            };

            let mut detected_anomalies = self.detected_anomalies.lock().unwrap();
            detected_anomalies.push(detected_anomaly);

            // 最大1000件まで保存
            if detected_anomalies.len() > 1000 {
                detected_anomalies.remove(0);
            }
        }

        result
    }

    /// 緩和策を決定
    fn determine_mitigations(&self, anomalies: &[(AnomalyType, f32)]) -> Vec<MitigationAction> {
        let mut actions = Vec::new();

        for (anomaly_type, confidence) in anomalies {
            for rule in &self.mitigation_rules {
                if rule.anomaly_type == *anomaly_type && *confidence >= rule.min_confidence {
                    if !actions.contains(&rule.action) {
                        actions.push(rule.action.clone());
                    }
                }
            }
        }

        actions
    }

    /// 検出された異常を取得
    pub fn get_detected_anomalies(&self) -> Vec<DetectedAnomaly> {
        let detected_anomalies = self.detected_anomalies.lock().unwrap();
        detected_anomalies.clone()
    }

    /// 最近の異常を取得
    pub fn get_recent_anomalies(&self, duration: Duration) -> Vec<DetectedAnomaly> {
        let detected_anomalies = self.detected_anomalies.lock().unwrap();
        let now = Instant::now();

        detected_anomalies
            .iter()
            .filter(|anomaly| now.duration_since(anomaly.detected_at) < duration)
            .cloned()
            .collect()
    }

    /// 閾値を設定
    pub fn set_threshold(&mut self, anomaly_type: AnomalyType, threshold: f32) {
        self.thresholds.insert(anomaly_type, threshold);
    }

    /// 緩和ルールを追加
    pub fn add_mitigation_rule(
        &mut self,
        anomaly_type: AnomalyType,
        min_confidence: f32,
        action: MitigationAction,
    ) {
        self.mitigation_rules.push(MitigationRule {
            anomaly_type,
            min_confidence,
            action,
        });
    }

    /// 監視ループを開始
    pub async fn start_monitoring_loop(
        detector: Arc<Self>,
        metrics_provider: Arc<dyn MetricsProvider + Send + Sync>,
    ) -> Result<(), Error> {
        let mut interval = interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            // メトリクスを取得
            let metrics = metrics_provider.get_current_metrics().await?;

            // メトリクスを追加
            detector.add_metrics(metrics.clone());

            // 異常を検出
            let anomalies = detector.detect(&metrics);

            // 異常が検出された場合は対応
            if !anomalies.is_empty() {
                let mitigations = detector.determine_mitigations(&anomalies);

                for mitigation in mitigations {
                    metrics_provider.apply_mitigation(mitigation).await?;
                }
            }
        }
    }
}

/// メトリクスプロバイダー
#[async_trait::async_trait]
pub trait MetricsProvider {
    /// 現在のメトリクスを取得
    async fn get_current_metrics(&self) -> Result<TransactionMetrics, Error>;

    /// 緩和策を適用
    async fn apply_mitigation(&self, action: MitigationAction) -> Result<(), Error>;
}

/// 平均値を計算
fn mean<T>(values: &[T]) -> f32
where
    T: Copy + Into<f32>,
{
    if values.is_empty() {
        return 0.0;
    }

    let sum: f32 = values.iter().map(|&v| v.into()).sum();
    sum / values.len() as f32
}

/// 標準偏差を計算
fn std_dev<T>(values: &[T], mean: f32) -> f32
where
    T: Copy + Into<f32>,
{
    if values.len() <= 1 {
        return 0.0;
    }

    let variance: f32 = values
        .iter()
        .map(|&v| {
            let diff = v.into() - mean;
            diff * diff
        })
        .sum::<f32>()
        / (values.len() - 1) as f32;

    variance.sqrt()
}

/// Z-スコアを計算
fn z_score(value: f32, mean: f32, std_dev: f32) -> f32 {
    if std_dev == 0.0 {
        return 0.0;
    }

    (value - mean) / std_dev
}

/// Z-スコアから信頼度を計算
fn confidence_from_z_score(z_score: f32, threshold: f32) -> f32 {
    let abs_z = z_score.abs();

    if abs_z <= threshold {
        return 0.0;
    }

    let confidence = (abs_z - threshold) / (5.0 - threshold);
    confidence.min(1.0)
}

/// シグモイド関数
fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistical_model() {
        // 統計モデルを作成
        let model = StatisticalModel::new(5, 2.0);

        // 履歴データを作成
        let mut history = Vec::new();

        for i in 0..10 {
            let metrics = TransactionMetrics {
                timestamp: Instant::now(),
                transaction_count: 100 + i * 10,
                avg_transaction_size: 200,
                avg_processing_time: Duration::from_millis(50),
                rejected_transaction_count: 5,
                unique_address_count: 50,
                cross_shard_transaction_count: 20,
            };

            history.push(metrics);
        }

        // 正常なメトリクス
        let normal_metrics = TransactionMetrics {
            timestamp: Instant::now(),
            transaction_count: 200,
            avg_transaction_size: 200,
            avg_processing_time: Duration::from_millis(50),
            rejected_transaction_count: 5,
            unique_address_count: 50,
            cross_shard_transaction_count: 20,
        };

        // 異常なメトリクス
        let abnormal_metrics = TransactionMetrics {
            timestamp: Instant::now(),
            transaction_count: 500, // 異常に高い
            avg_transaction_size: 200,
            avg_processing_time: Duration::from_millis(50),
            rejected_transaction_count: 5,
            unique_address_count: 50,
            cross_shard_transaction_count: 20,
        };

        // 正常なメトリクスでは異常が検出されないことを確認
        let normal_anomalies = model.detect(&normal_metrics, &history);
        assert!(normal_anomalies.is_empty());

        // 異常なメトリクスでは異常が検出されることを確認
        let abnormal_anomalies = model.detect(&abnormal_metrics, &history);
        assert!(!abnormal_anomalies.is_empty());
        assert!(abnormal_anomalies.contains_key(&AnomalyType::UnusualVolume));
    }

    #[test]
    fn test_machine_learning_model() {
        // 機械学習モデルを作成
        let model = MachineLearningModel::new();

        // 正常なメトリクス
        let normal_metrics = TransactionMetrics {
            timestamp: Instant::now(),
            transaction_count: 100,
            avg_transaction_size: 200,
            avg_processing_time: Duration::from_millis(50),
            rejected_transaction_count: 5,
            unique_address_count: 50,
            cross_shard_transaction_count: 20,
        };

        // 異常なメトリクス（悪意のあるアクティビティ）
        let malicious_metrics = TransactionMetrics {
            timestamp: Instant::now(),
            transaction_count: 100,
            avg_transaction_size: 200,
            avg_processing_time: Duration::from_millis(50),
            rejected_transaction_count: 50, // 異常に高い
            unique_address_count: 50,
            cross_shard_transaction_count: 20,
        };

        // 正常なメトリクスでの検出結果
        let normal_anomalies = model.detect(&normal_metrics, &[]);

        // 異常なメトリクスでの検出結果
        let malicious_anomalies = model.detect(&malicious_metrics, &[]);

        // 異常なメトリクスでは悪意のあるアクティビティが検出されることを確認
        assert!(
            malicious_anomalies
                .get(&AnomalyType::MaliciousActivity)
                .unwrap_or(&0.0)
                > normal_anomalies
                    .get(&AnomalyType::MaliciousActivity)
                    .unwrap_or(&0.0)
        );
    }

    #[test]
    fn test_anomaly_detector() {
        // 統計ベースの異常検出器を作成
        let detector = AnomalyDetector::new_statistical(5, 2.0);

        // 正常なメトリクス
        let normal_metrics = TransactionMetrics {
            timestamp: Instant::now(),
            transaction_count: 100,
            avg_transaction_size: 200,
            avg_processing_time: Duration::from_millis(50),
            rejected_transaction_count: 5,
            unique_address_count: 50,
            cross_shard_transaction_count: 20,
        };

        // メトリクスを追加
        for _ in 0..10 {
            detector.add_metrics(normal_metrics.clone());
        }

        // 異常なメトリクス
        let abnormal_metrics = TransactionMetrics {
            timestamp: Instant::now(),
            transaction_count: 500, // 異常に高い
            avg_transaction_size: 200,
            avg_processing_time: Duration::from_millis(50),
            rejected_transaction_count: 5,
            unique_address_count: 50,
            cross_shard_transaction_count: 20,
        };

        // 正常なメトリクスでは異常が検出されないことを確認
        let normal_anomalies = detector.detect(&normal_metrics);
        assert!(normal_anomalies.is_empty());

        // 異常なメトリクスでは異常が検出されることを確認
        let abnormal_anomalies = detector.detect(&abnormal_metrics);
        assert!(!abnormal_anomalies.is_empty());

        // 検出された異常の種類を確認
        assert_eq!(abnormal_anomalies[0].0, AnomalyType::UnusualVolume);
    }

    #[test]
    fn test_mitigation_rules() {
        // 異常検出器を作成
        let mut detector = AnomalyDetector::new_ml();

        // 緩和ルールを追加
        detector.add_mitigation_rule(
            AnomalyType::UnusualVolume,
            0.5,
            MitigationAction::ThrottleTransactions,
        );

        // 異常なメトリクス
        let abnormal_metrics = TransactionMetrics {
            timestamp: Instant::now(),
            transaction_count: 1000, // 異常に高い
            avg_transaction_size: 200,
            avg_processing_time: Duration::from_millis(50),
            rejected_transaction_count: 5,
            unique_address_count: 50,
            cross_shard_transaction_count: 20,
        };

        // 異常を検出
        let anomalies = detector.detect(&abnormal_metrics);

        // 緩和策を決定
        let mitigations = detector.determine_mitigations(&anomalies);

        // 緩和策が適用されることを確認
        assert!(!mitigations.is_empty());
        assert!(mitigations.contains(&MitigationAction::ThrottleTransactions));
    }
}
