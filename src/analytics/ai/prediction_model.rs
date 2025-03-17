use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::analytics::metrics::{MetricType, MetricValue};
use crate::analytics::ai_predictor::{PredictionModel, PredictionResult, PredictionConfig};

/// 予測モデルタイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModelType {
    /// 線形回帰
    LinearRegression,
    /// 自己回帰和分移動平均
    ARIMA,
    /// 長短期記憶ネットワーク
    LSTM,
    /// 勾配ブースティング
    GradientBoosting,
    /// プロフェット
    Prophet,
    /// ニューラルネットワーク
    NeuralNetwork,
    /// アンサンブル
    Ensemble,
    /// カスタム
    Custom(String),
}

/// 予測対象
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PredictionTarget {
    /// トランザクション数
    TransactionCount,
    /// トランザクション量
    TransactionVolume,
    /// ガス価格
    GasPrice,
    /// ブロック時間
    BlockTime,
    /// アクティブアドレス数
    ActiveAddresses,
    /// ネットワーク使用率
    NetworkUtilization,
    /// 手数料収入
    FeeRevenue,
    /// シャード負荷
    ShardLoad,
    /// カスタム
    Custom(String),
}

/// 予測期間
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PredictionHorizon {
    /// 分単位
    Minutes(u32),
    /// 時間単位
    Hours(u32),
    /// 日単位
    Days(u32),
    /// 週単位
    Weeks(u32),
    /// 月単位
    Months(u32),
    /// カスタム（秒）
    Custom(u64),
}

impl PredictionHorizon {
    /// 秒数に変換
    pub fn to_seconds(&self) -> u64 {
        match self {
            PredictionHorizon::Minutes(m) => *m as u64 * 60,
            PredictionHorizon::Hours(h) => *h as u64 * 3600,
            PredictionHorizon::Days(d) => *d as u64 * 86400,
            PredictionHorizon::Weeks(w) => *w as u64 * 604800,
            PredictionHorizon::Months(m) => *m as u64 * 2592000, // 30日で計算
            PredictionHorizon::Custom(s) => *s,
        }
    }
    
    /// 期間の説明
    pub fn description(&self) -> String {
        match self {
            PredictionHorizon::Minutes(m) => format!("{}分", m),
            PredictionHorizon::Hours(h) => format!("{}時間", h),
            PredictionHorizon::Days(d) => format!("{}日", d),
            PredictionHorizon::Weeks(w) => format!("{}週間", w),
            PredictionHorizon::Months(m) => format!("{}ヶ月", m),
            PredictionHorizon::Custom(s) => format!("{}秒", s),
        }
    }
}

/// 予測間隔
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PredictionInterval {
    /// 分単位
    Minutes(u32),
    /// 時間単位
    Hours(u32),
    /// 日単位
    Days(u32),
    /// カスタム（秒）
    Custom(u64),
}

impl PredictionInterval {
    /// 秒数に変換
    pub fn to_seconds(&self) -> u64 {
        match self {
            PredictionInterval::Minutes(m) => *m as u64 * 60,
            PredictionInterval::Hours(h) => *h as u64 * 3600,
            PredictionInterval::Days(d) => *d as u64 * 86400,
            PredictionInterval::Custom(s) => *s,
        }
    }
}

/// 予測設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedPredictionConfig {
    /// モデルタイプ
    pub model_type: ModelType,
    /// 予測対象
    pub target: PredictionTarget,
    /// 予測期間
    pub horizon: PredictionHorizon,
    /// 予測間隔
    pub interval: PredictionInterval,
    /// 学習期間（秒）
    pub training_period_seconds: u64,
    /// 特徴量
    pub features: Vec<String>,
    /// 信頼区間
    pub confidence_interval: Option<f64>,
    /// 再学習間隔（秒）
    pub retraining_interval_seconds: Option<u64>,
    /// ハイパーパラメータ
    pub hyperparameters: Option<HashMap<String, serde_json::Value>>,
    /// 季節性
    pub seasonality: Option<Seasonality>,
    /// 異常検出
    pub anomaly_detection: Option<bool>,
    /// 変化点検出
    pub changepoint_detection: Option<bool>,
    /// 予測評価メトリクス
    pub evaluation_metrics: Option<Vec<EvaluationMetric>>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// 季節性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Seasonality {
    /// 日次季節性
    pub daily: bool,
    /// 週次季節性
    pub weekly: bool,
    /// 月次季節性
    pub monthly: bool,
    /// 四半期季節性
    pub quarterly: bool,
    /// 年次季節性
    pub yearly: bool,
    /// カスタム季節性（秒）
    pub custom: Option<Vec<u64>>,
}

/// 評価メトリクス
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EvaluationMetric {
    /// 平均絶対誤差
    MAE,
    /// 平均二乗誤差
    MSE,
    /// 平均二乗誤差の平方根
    RMSE,
    /// 平均絶対パーセント誤差
    MAPE,
    /// 対称平均絶対パーセント誤差
    SMAPE,
    /// 決定係数
    R2,
    /// カスタム
    Custom(String),
}

impl Default for AdvancedPredictionConfig {
    fn default() -> Self {
        Self {
            model_type: ModelType::LSTM,
            target: PredictionTarget::TransactionCount,
            horizon: PredictionHorizon::Hours(24),
            interval: PredictionInterval::Hours(1),
            training_period_seconds: 604800, // 1週間
            features: vec![
                "transaction_count".to_string(),
                "transaction_volume".to_string(),
                "average_fee".to_string(),
                "active_users".to_string(),
                "gas_price".to_string(),
                "block_time".to_string(),
            ],
            confidence_interval: Some(0.95),
            retraining_interval_seconds: Some(86400), // 1日
            hyperparameters: None,
            seasonality: Some(Seasonality {
                daily: true,
                weekly: true,
                monthly: false,
                quarterly: false,
                yearly: false,
                custom: None,
            }),
            anomaly_detection: Some(true),
            changepoint_detection: Some(true),
            evaluation_metrics: Some(vec![
                EvaluationMetric::MAE,
                EvaluationMetric::RMSE,
                EvaluationMetric::MAPE,
            ]),
            additional_properties: HashMap::new(),
        }
    }
}

/// 予測結果セット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResultSet {
    /// 予測ID
    pub id: String,
    /// 予測対象
    pub target: PredictionTarget,
    /// モデルタイプ
    pub model_type: ModelType,
    /// 予測期間
    pub horizon: PredictionHorizon,
    /// 予測間隔
    pub interval: PredictionInterval,
    /// 予測時刻
    pub prediction_time: DateTime<Utc>,
    /// 予測結果
    pub results: Vec<PredictionPoint>,
    /// 信頼区間
    pub confidence_interval: Option<f64>,
    /// 評価メトリクス
    pub evaluation: Option<HashMap<EvaluationMetric, f64>>,
    /// 特徴量重要度
    pub feature_importance: Option<HashMap<String, f64>>,
    /// 異常検出結果
    pub anomalies: Option<Vec<Anomaly>>,
    /// 変化点検出結果
    pub changepoints: Option<Vec<Changepoint>>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 予測ポイント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionPoint {
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// 予測値
    pub value: f64,
    /// 下限
    pub lower_bound: Option<f64>,
    /// 上限
    pub upper_bound: Option<f64>,
    /// 実際の値（評価用）
    pub actual_value: Option<f64>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// 異常
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// 値
    pub value: f64,
    /// 期待値
    pub expected_value: f64,
    /// 偏差
    pub deviation: f64,
    /// 重要度
    pub severity: AnomalySeverity,
    /// 説明
    pub description: Option<String>,
}

/// 異常の重要度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AnomalySeverity {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 重大
    Critical,
}

/// 変化点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Changepoint {
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// 変化の大きさ
    pub magnitude: f64,
    /// 方向
    pub direction: ChangepointDirection,
    /// 信頼度
    pub confidence: f64,
    /// 説明
    pub description: Option<String>,
}

/// 変化点の方向
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ChangepointDirection {
    /// 上昇
    Increase,
    /// 下降
    Decrease,
    /// 不明
    Unknown,
}

/// 高度な予測エンジン
pub struct AdvancedPredictionEngine {
    /// 設定
    config: AdvancedPredictionConfig,
    /// モデル
    models: HashMap<PredictionTarget, Box<dyn PredictionModelTrait>>,
    /// 最後の学習時刻
    last_training_time: HashMap<PredictionTarget, DateTime<Utc>>,
    /// 予測履歴
    prediction_history: HashMap<PredictionTarget, Vec<PredictionResultSet>>,
    /// 特徴量データ
    feature_data: HashMap<String, Vec<(DateTime<Utc>, f64)>>,
    /// ターゲットデータ
    target_data: HashMap<PredictionTarget, Vec<(DateTime<Utc>, f64)>>,
    /// 評価結果
    evaluation_results: HashMap<PredictionTarget, HashMap<EvaluationMetric, Vec<f64>>>,
}

impl AdvancedPredictionEngine {
    /// 新しい高度な予測エンジンを作成
    pub fn new(config: AdvancedPredictionConfig) -> Self {
        Self {
            config,
            models: HashMap::new(),
            last_training_time: HashMap::new(),
            prediction_history: HashMap::new(),
            feature_data: HashMap::new(),
            target_data: HashMap::new(),
            evaluation_results: HashMap::new(),
        }
    }
    
    /// 特徴量データを追加
    pub fn add_feature_data(&mut self, feature: &str, timestamp: DateTime<Utc>, value: f64) {
        let feature_data = self.feature_data.entry(feature.to_string()).or_insert_with(Vec::new);
        feature_data.push((timestamp, value));
    }
    
    /// ターゲットデータを追加
    pub fn add_target_data(&mut self, target: PredictionTarget, timestamp: DateTime<Utc>, value: f64) {
        let target_data = self.target_data.entry(target).or_insert_with(Vec::new);
        target_data.push((timestamp, value));
    }
    
    /// メトリクスデータを追加
    pub fn add_metric_data(&mut self, metric_type: &MetricType, timestamp: DateTime<Utc>, value: &MetricValue) {
        let metric_name = format!("{:?}", metric_type);
        
        match value {
            MetricValue::Integer(i) => {
                self.add_feature_data(&metric_name, timestamp, *i as f64);
                
                // 対応するターゲットにも追加
                if let Some(target) = self.metric_to_target(metric_type) {
                    self.add_target_data(target, timestamp, *i as f64);
                }
            },
            MetricValue::Float(f) => {
                self.add_feature_data(&metric_name, timestamp, *f);
                
                if let Some(target) = self.metric_to_target(metric_type) {
                    self.add_target_data(target, timestamp, *f);
                }
            },
            MetricValue::Counter(c) => {
                self.add_feature_data(&metric_name, timestamp, *c as f64);
                
                if let Some(target) = self.metric_to_target(metric_type) {
                    self.add_target_data(target, timestamp, *c as f64);
                }
            },
            _ => {
                // 他のメトリクス型は無視
            }
        }
    }
    
    /// メトリクスタイプを予測対象に変換
    fn metric_to_target(&self, metric_type: &MetricType) -> Option<PredictionTarget> {
        match metric_type {
            MetricType::TransactionCount => Some(PredictionTarget::TransactionCount),
            MetricType::TransactionVolume => Some(PredictionTarget::TransactionVolume),
            MetricType::GasPrice => Some(PredictionTarget::GasPrice),
            MetricType::BlockTime => Some(PredictionTarget::BlockTime),
            MetricType::ActiveAddresses => Some(PredictionTarget::ActiveAddresses),
            MetricType::NetworkUtilization => Some(PredictionTarget::NetworkUtilization),
            MetricType::FeeRevenue => Some(PredictionTarget::FeeRevenue),
            MetricType::ShardLoad => Some(PredictionTarget::ShardLoad),
            _ => None,
        }
    }
    
    /// モデルを学習
    pub fn train(&mut self, target: Option<PredictionTarget>) -> Result<(), Error> {
        let now = Utc::now();
        
        // 対象のターゲットを決定
        let targets = if let Some(target) = target {
            vec![target]
        } else {
            vec![self.config.target.clone()]
        };
        
        for target in targets {
            // 再学習間隔をチェック
            if let Some(last_training_time) = self.last_training_time.get(&target) {
                if let Some(retraining_interval) = self.config.retraining_interval_seconds {
                    let elapsed = now.timestamp() - last_training_time.timestamp();
                    if elapsed < retraining_interval as i64 {
                        continue;
                    }
                }
            }
            
            // 学習期間のデータを取得
            let training_start = now - Duration::seconds(self.config.training_period_seconds as i64);
            
            // 特徴量データを準備
            let mut features: HashMap<String, Vec<f64>> = HashMap::new();
            let mut timestamps: Vec<DateTime<Utc>> = Vec::new();
            
            for feature_name in &self.config.features {
                if let Some(feature_data) = self.feature_data.get(feature_name) {
                    let filtered_data: Vec<_> = feature_data.iter()
                        .filter(|(ts, _)| *ts >= training_start)
                        .collect();
                    
                    if filtered_data.is_empty() {
                        warn!("No data for feature: {}", feature_name);
                        continue;
                    }
                    
                    // タイムスタンプを記録
                    if timestamps.is_empty() {
                        timestamps = filtered_data.iter().map(|(ts, _)| *ts).collect();
                    }
                    
                    // 特徴量値を記録
                    features.insert(feature_name.clone(), filtered_data.iter().map(|(_, v)| *v).collect());
                } else {
                    warn!("Feature not found: {}", feature_name);
                }
            }
            
            if features.is_empty() {
                return Err(Error::InvalidState("No feature data available for training".to_string()));
            }
            
            // ターゲットデータを準備
            let target_data = self.target_data.entry(target.clone()).or_insert_with(Vec::new);
            let filtered_target: Vec<_> = target_data.iter()
                .filter(|(ts, _)| *ts >= training_start)
                .collect();
            
            if filtered_target.is_empty() {
                return Err(Error::InvalidState(format!("No data for target: {:?}", target)));
            }
            
            let target_values: Vec<f64> = filtered_target.iter().map(|(_, v)| *v).collect();
            
            // モデルを作成または取得
            let model = self.get_or_create_model(&target);
            
            // モデルを学習
            model.train(&features, &target_values, self.config.hyperparameters.as_ref())?;
            
            // 最後の学習時刻を更新
            self.last_training_time.insert(target.clone(), now);
        }
        
        Ok(())
    }
    
    /// モデルを取得または作成
    fn get_or_create_model(&mut self, target: &PredictionTarget) -> &mut Box<dyn PredictionModelTrait> {
        if !self.models.contains_key(target) {
            let model: Box<dyn PredictionModelTrait> = match self.config.model_type {
                ModelType::LinearRegression => Box::new(LinearRegressionModel::new()),
                ModelType::ARIMA => Box::new(ARIMAModel::new()),
                ModelType::LSTM => Box::new(LSTMModel::new()),
                ModelType::GradientBoosting => Box::new(GradientBoostingModel::new()),
                ModelType::Prophet => Box::new(ProphetModel::new()),
                ModelType::NeuralNetwork => Box::new(NeuralNetworkModel::new()),
                ModelType::Ensemble => Box::new(EnsembleModel::new()),
                ModelType::Custom(_) => Box::new(CustomModel::new()),
            };
            
            self.models.insert(target.clone(), model);
        }
        
        self.models.get_mut(target).unwrap()
    }
    
    /// 予測を実行
    pub fn predict(&mut self, target: Option<PredictionTarget>) -> Result<PredictionResultSet, Error> {
        let now = Utc::now();
        
        // 対象のターゲットを決定
        let target = target.unwrap_or_else(|| self.config.target.clone());
        
        // モデルをチェック
        if !self.models.contains_key(&target) {
            self.train(Some(target.clone()))?;
        }
        
        let model = self.models.get(&target).ok_or_else(|| {
            Error::InvalidState(format!("Model not trained for target: {:?}", target))
        })?;
        
        // 予測期間を計算
        let horizon_seconds = self.config.horizon.to_seconds();
        let interval_seconds = self.config.interval.to_seconds();
        
        // 予測間隔の数を計算
        let intervals = (horizon_seconds / interval_seconds) as usize;
        if intervals == 0 {
            return Err(Error::InvalidInput("Prediction horizon must be greater than interval".to_string()));
        }
        
        // 最新の特徴量データを取得
        let mut latest_features: HashMap<String, f64> = HashMap::new();
        
        for feature_name in &self.config.features {
            if let Some(feature_data) = self.feature_data.get(feature_name) {
                if let Some((_, value)) = feature_data.last() {
                    latest_features.insert(feature_name.clone(), *value);
                } else {
                    warn!("No data for feature: {}", feature_name);
                }
            } else {
                warn!("Feature not found: {}", feature_name);
            }
        }
        
        if latest_features.is_empty() {
            return Err(Error::InvalidState("No feature data available for prediction".to_string()));
        }
        
        // 予測を実行
        let mut prediction_points = Vec::new();
        
        for i in 1..=intervals {
            let target_timestamp = now + Duration::seconds((i * interval_seconds as usize) as i64);
            
            // 予測を実行
            let prediction = model.predict(&latest_features)?;
            
            // 信頼区間を計算
            let (lower_bound, upper_bound) = if let Some(confidence_interval) = self.config.confidence_interval {
                let std_dev = 0.1 * prediction; // 仮の標準偏差
                let z_score = match confidence_interval {
                    0.90 => 1.645,
                    0.95 => 1.96,
                    0.99 => 2.576,
                    _ => 1.96,
                };
                
                let margin = z_score * std_dev;
                (Some(prediction - margin), Some(prediction + margin))
            } else {
                (None, None)
            };
            
            // 予測ポイントを作成
            let point = PredictionPoint {
                timestamp: target_timestamp,
                value: prediction,
                lower_bound,
                upper_bound,
                actual_value: None,
                additional_properties: HashMap::new(),
            };
            
            prediction_points.push(point);
            
            // 次の予測のために特徴量を更新
            latest_features.insert(format!("{:?}", target), prediction);
        }
        
        // 予測結果セットを作成
        let prediction_id = format!("prediction_{}_{}", target_timestamp_to_string(&now), target_to_string(&target));
        
        let result_set = PredictionResultSet {
            id: prediction_id,
            target: target.clone(),
            model_type: self.config.model_type.clone(),
            horizon: self.config.horizon.clone(),
            interval: self.config.interval.clone(),
            prediction_time: now,
            results: prediction_points,
            confidence_interval: self.config.confidence_interval,
            evaluation: None,
            feature_importance: model.feature_importance(),
            anomalies: None,
            changepoints: None,
            metadata: None,
        };
        
        // 予測履歴に追加
        let history = self.prediction_history.entry(target.clone()).or_insert_with(Vec::new);
        history.push(result_set.clone());
        
        // 異常検出を実行
        if let Some(true) = self.config.anomaly_detection {
            // 実際の実装では、異常検出アルゴリズムを実行
        }
        
        // 変化点検出を実行
        if let Some(true) = self.config.changepoint_detection {
            // 実際の実装では、変化点検出アルゴリズムを実行
        }
        
        Ok(result_set)
    }
    
    /// 予測精度を評価
    pub fn evaluate(&mut self, target: Option<PredictionTarget>) -> Result<HashMap<EvaluationMetric, f64>, Error> {
        let target = target.unwrap_or_else(|| self.config.target.clone());
        
        // 予測履歴を取得
        let history = self.prediction_history.get(&target).ok_or_else(|| {
            Error::NotFound(format!("No prediction history for target: {:?}", target))
        })?;
        
        if history.is_empty() {
            return Err(Error::InvalidState("No predictions to evaluate".to_string()));
        }
        
        // ターゲットデータを取得
        let target_data = self.target_data.get(&target).ok_or_else(|| {
            Error::NotFound(format!("No target data for: {:?}", target))
        })?;
        
        if target_data.is_empty() {
            return Err(Error::InvalidState("No target data to evaluate against".to_string()));
        }
        
        // 評価メトリクスを初期化
        let mut metrics = HashMap::new();
        let evaluation_metrics = self.config.evaluation_metrics.clone().unwrap_or_else(|| {
            vec![EvaluationMetric::MAE, EvaluationMetric::RMSE, EvaluationMetric::MAPE]
        });
        
        // 各メトリクスの累積値
        let mut mae_sum = 0.0;
        let mut mse_sum = 0.0;
        let mut mape_sum = 0.0;
        let mut smape_sum = 0.0;
        let mut count = 0;
        
        // 最新の予測結果セットを取得
        let latest_prediction = history.last().unwrap();
        
        // 各予測ポイントに対して実際の値を検索し、誤差を計算
        for point in &latest_prediction.results {
            // 予測対象時刻に最も近い実際の値を検索
            let actual = target_data.iter()
                .min_by_key(|(ts, _)| {
                    let diff = (*ts - point.timestamp).num_seconds().abs();
                    diff as u64
                });
            
            if let Some((_, actual_value)) = actual {
                let error = (point.value - *actual_value).abs();
                let squared_error = error * error;
                let percentage_error = if *actual_value != 0.0 {
                    error / actual_value.abs()
                } else {
                    0.0
                };
                let symmetric_percentage_error = if point.value.abs() + actual_value.abs() > 0.0 {
                    2.0 * error / (point.value.abs() + actual_value.abs())
                } else {
                    0.0
                };
                
                mae_sum += error;
                mse_sum += squared_error;
                mape_sum += percentage_error;
                smape_sum += symmetric_percentage_error;
                count += 1;
            }
        }
        
        if count == 0 {
            return Err(Error::InvalidState("No matching data points for evaluation".to_string()));
        }
        
        // 平均値を計算
        let mae = mae_sum / count as f64;
        let mse = mse_sum / count as f64;
        let rmse = mse.sqrt();
        let mape = mape_sum / count as f64 * 100.0; // パーセントに変換
        let smape = smape_sum / count as f64 * 100.0; // パーセントに変換
        
        // R2スコアを計算（実際の実装では、より複雑な計算が必要）
        let r2 = 0.0; // 仮の値
        
        // 評価結果を格納
        for metric in &evaluation_metrics {
            match metric {
                EvaluationMetric::MAE => {
                    metrics.insert(EvaluationMetric::MAE, mae);
                },
                EvaluationMetric::MSE => {
                    metrics.insert(EvaluationMetric::MSE, mse);
                },
                EvaluationMetric::RMSE => {
                    metrics.insert(EvaluationMetric::RMSE, rmse);
                },
                EvaluationMetric::MAPE => {
                    metrics.insert(EvaluationMetric::MAPE, mape);
                },
                EvaluationMetric::SMAPE => {
                    metrics.insert(EvaluationMetric::SMAPE, smape);
                },
                EvaluationMetric::R2 => {
                    metrics.insert(EvaluationMetric::R2, r2);
                },
                EvaluationMetric::Custom(_) => {
                    // カスタムメトリクスは未実装
                },
            }
        }
        
        // 評価結果を履歴に保存
        let eval_history = self.evaluation_results.entry(target.clone()).or_insert_with(HashMap::new);
        
        for (metric, value) in &metrics {
            let history = eval_history.entry(metric.clone()).or_insert_with(Vec::new);
            history.push(*value);
        }
        
        Ok(metrics)
    }
    
    /// 設定を取得
    pub fn get_config(&self) -> &AdvancedPredictionConfig {
        &self.config
    }
    
    /// 設定を更新
    pub fn update_config(&mut self, config: AdvancedPredictionConfig) {
        self.config = config;
        self.models.clear(); // モデルを再学習するためにリセット
    }
    
    /// 予測履歴を取得
    pub fn get_prediction_history(&self, target: Option<PredictionTarget>) -> Option<&Vec<PredictionResultSet>> {
        let target = target.unwrap_or_else(|| self.config.target.clone());
        self.prediction_history.get(&target)
    }
    
    /// 特徴量データを取得
    pub fn get_feature_data(&self, feature: &str) -> Option<&[(DateTime<Utc>, f64)]> {
        self.feature_data.get(feature).map(|data| data.as_slice())
    }
    
    /// ターゲットデータを取得
    pub fn get_target_data(&self, target: &PredictionTarget) -> Option<&[(DateTime<Utc>, f64)]> {
        self.target_data.get(target).map(|data| data.as_slice())
    }
    
    /// 評価結果を取得
    pub fn get_evaluation_results(&self, target: &PredictionTarget) -> Option<&HashMap<EvaluationMetric, Vec<f64>>> {
        self.evaluation_results.get(target)
    }
    
    /// 最新の評価結果を取得
    pub fn get_latest_evaluation(&self, target: &PredictionTarget) -> Option<HashMap<EvaluationMetric, f64>> {
        if let Some(eval_results) = self.evaluation_results.get(target) {
            let mut latest = HashMap::new();
            
            for (metric, values) in eval_results {
                if let Some(value) = values.last() {
                    latest.insert(metric.clone(), *value);
                }
            }
            
            if !latest.is_empty() {
                return Some(latest);
            }
        }
        
        None
    }
    
    /// 異常を検出
    pub fn detect_anomalies(&self, target: &PredictionTarget, lookback_days: u32) -> Result<Vec<Anomaly>, Error> {
        // 実際の実装では、異常検出アルゴリズムを実行
        // ここでは簡易的な実装を提供
        
        let target_data = self.target_data.get(target).ok_or_else(|| {
            Error::NotFound(format!("No target data for: {:?}", target))
        })?;
        
        if target_data.is_empty() {
            return Err(Error::InvalidState("No target data for anomaly detection".to_string()));
        }
        
        let now = Utc::now();
        let lookback_period = Duration::days(lookback_days as i64);
        let lookback_start = now - lookback_period;
        
        // 期間内のデータをフィルタリング
        let filtered_data: Vec<_> = target_data.iter()
            .filter(|(ts, _)| *ts >= lookback_start)
            .collect();
        
        if filtered_data.is_empty() {
            return Err(Error::InvalidState("No data in lookback period for anomaly detection".to_string()));
        }
        
        // 平均と標準偏差を計算
        let values: Vec<f64> = filtered_data.iter().map(|(_, v)| *v).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        
        let variance = values.iter()
            .map(|v| (*v - mean) * (*v - mean))
            .sum::<f64>() / values.len() as f64;
        
        let std_dev = variance.sqrt();
        
        // 異常を検出（平均から3標準偏差以上離れた値）
        let threshold = 3.0;
        let mut anomalies = Vec::new();
        
        for (ts, value) in filtered_data {
            let deviation = (*value - mean).abs() / std_dev;
            
            if deviation > threshold {
                let severity = if deviation > 5.0 {
                    AnomalySeverity::Critical
                } else if deviation > 4.0 {
                    AnomalySeverity::High
                } else if deviation > 3.5 {
                    AnomalySeverity::Medium
                } else {
                    AnomalySeverity::Low
                };
                
                anomalies.push(Anomaly {
                    timestamp: *ts,
                    value: *value,
                    expected_value: mean,
                    deviation,
                    severity,
                    description: Some(format!("Value deviates {:.2} standard deviations from mean", deviation)),
                });
            }
        }
        
        Ok(anomalies)
    }
    
    /// 変化点を検出
    pub fn detect_changepoints(&self, target: &PredictionTarget, lookback_days: u32) -> Result<Vec<Changepoint>, Error> {
        // 実際の実装では、変化点検出アルゴリズムを実行
        // ここでは簡易的な実装を提供
        
        let target_data = self.target_data.get(target).ok_or_else(|| {
            Error::NotFound(format!("No target data for: {:?}", target))
        })?;
        
        if target_data.is_empty() {
            return Err(Error::InvalidState("No target data for changepoint detection".to_string()));
        }
        
        let now = Utc::now();
        let lookback_period = Duration::days(lookback_days as i64);
        let lookback_start = now - lookback_period;
        
        // 期間内のデータをフィルタリング
        let filtered_data: Vec<_> = target_data.iter()
            .filter(|(ts, _)| *ts >= lookback_start)
            .collect();
        
        if filtered_data.len() < 10 {
            return Err(Error::InvalidState("Insufficient data for changepoint detection".to_string()));
        }
        
        // 変化点を検出（隣接する値の差が大きい点）
        let mut changepoints = Vec::new();
        let window_size = 5;
        
        for i in window_size..(filtered_data.len() - window_size) {
            let before_avg = filtered_data[i-window_size..i].iter()
                .map(|(_, v)| *v)
                .sum::<f64>() / window_size as f64;
            
            let after_avg = filtered_data[i..i+window_size].iter()
                .map(|(_, v)| *v)
                .sum::<f64>() / window_size as f64;
            
            let change = after_avg - before_avg;
            let magnitude = change.abs();
            
            // 変化の大きさが閾値を超える場合
            if magnitude > before_avg * 0.2 { // 20%以上の変化
                let direction = if change > 0.0 {
                    ChangepointDirection::Increase
                } else if change < 0.0 {
                    ChangepointDirection::Decrease
                } else {
                    ChangepointDirection::Unknown
                };
                
                let confidence = 0.8; // 仮の値
                
                changepoints.push(Changepoint {
                    timestamp: filtered_data[i].0,
                    magnitude,
                    direction,
                    confidence,
                    description: Some(format!("{:.2}% change detected", (change / before_avg * 100.0).abs())),
                });
            }
        }
        
        Ok(changepoints)
    }
}

/// 予測モデルトレイト
trait PredictionModelTrait {
    /// モデルを学習
    fn train(&mut self, features: &HashMap<String, Vec<f64>>, target: &[f64], hyperparameters: Option<&HashMap<String, serde_json::Value>>) -> Result<(), Error>;
    
    /// 予測を実行
    fn predict(&self, features: &HashMap<String, f64>) -> Result<f64, Error>;
    
    /// 特徴量重要度を取得
    fn feature_importance(&self) -> Option<HashMap<String, f64>> {
        None
    }
}

/// 線形回帰モデル
struct LinearRegressionModel {
    /// 係数
    coefficients: HashMap<String, f64>,
    /// 切片
    intercept: f64,
    /// 特徴量重要度
    importance: HashMap<String, f64>,
}

impl LinearRegressionModel {
    /// 新しい線形回帰モデルを作成
    fn new() -> Self {
        Self {
            coefficients: HashMap::new(),
            intercept: 0.0,
            importance: HashMap::new(),
        }
    }
}

impl PredictionModelTrait for LinearRegressionModel {
    fn train(&mut self, features: &HashMap<String, Vec<f64>>, target: &[f64], _hyperparameters: Option<&HashMap<String, serde_json::Value>>) -> Result<(), Error> {
        // 実際の実装では、線形回帰モデルを学習する
        // ここでは簡易的な実装を提供
        
        // 各特徴量の平均値を係数として使用
        for (feature_name, values) in features {
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            self.coefficients.insert(feature_name.clone(), mean);
            
            // 特徴量重要度を計算（仮の実装）
            let importance = mean.abs() / 10.0;
            self.importance.insert(feature_name.clone(), importance);
        }
        
        // 切片を0に設定
        self.intercept = 0.0;
        
        Ok(())
    }
    
    fn predict(&self, features: &HashMap<String, f64>) -> Result<f64, Error> {
        // 線形回帰モデルによる予測
        let mut prediction = self.intercept;
        
        for (feature_name, value) in features {
            if let Some(coefficient) = self.coefficients.get(feature_name) {
                prediction += coefficient * value;
            }
        }
        
        Ok(prediction)
    }
    
    fn feature_importance(&self) -> Option<HashMap<String, f64>> {
        if self.importance.is_empty() {
            None
        } else {
            Some(self.importance.clone())
        }
    }
}

/// LSTMモデル
struct LSTMModel {
    /// 重み
    weights: HashMap<String, f64>,
    /// バイアス
    bias: f64,
    /// 特徴量重要度
    importance: HashMap<String, f64>,
}

impl LSTMModel {
    /// 新しいLSTMモデルを作成
    fn new() -> Self {
        Self {
            weights: HashMap::new(),
            bias: 0.0,
            importance: HashMap::new(),
        }
    }
}

impl PredictionModelTrait for LSTMModel {
    fn train(&mut self, features: &HashMap<String, Vec<f64>>, target: &[f64], _hyperparameters: Option<&HashMap<String, serde_json::Value>>) -> Result<(), Error> {
        // 実際の実装では、LSTMモデルを学習する
        // ここでは簡易的な実装を提供
        
        // 各特徴量の最新値を重みとして使用
        for (feature_name, values) in features {
            if let Some(last_value) = values.last() {
                self.weights.insert(feature_name.clone(), *last_value);
                
                // 特徴量重要度を計算（仮の実装）
                let importance = last_value.abs() / 10.0;
                self.importance.insert(feature_name.clone(), importance);
            }
        }
        
        // バイアスを0に設定
        self.bias = 0.0;
        
        Ok(())
    }
    
    fn predict(&self, features: &HashMap<String, f64>) -> Result<f64, Error> {
        // LSTMモデルによる予測
        let mut prediction = self.bias;
        
        for (feature_name, value) in features {
            if let Some(weight) = self.weights.get(feature_name) {
                prediction += weight * value;
            }
        }
        
        Ok(prediction)
    }
    
    fn feature_importance(&self) -> Option<HashMap<String, f64>> {
        if self.importance.is_empty() {
            None
        } else {
            Some(self.importance.clone())
        }
    }
}

/// ARIMAモデル
struct ARIMAModel {
    /// 自己回帰係数
    ar_coefficients: Vec<f64>,
    /// 移動平均係数
    ma_coefficients: Vec<f64>,
    /// 差分次数
    d: usize,
    /// 定数項
    constant: f64,
}

impl ARIMAModel {
    /// 新しいARIMAモデルを作成
    fn new() -> Self {
        Self {
            ar_coefficients: Vec::new(),
            ma_coefficients: Vec::new(),
            d: 0,
            constant: 0.0,
        }
    }
}

impl PredictionModelTrait for ARIMAModel {
    fn train(&mut self, features: &HashMap<String, Vec<f64>>, target: &[f64], hyperparameters: Option<&HashMap<String, serde_json::Value>>) -> Result<(), Error> {
        // 実際の実装では、ARIMAモデルを学習する
        // ここでは簡易的な実装を提供
        
        // ハイパーパラメータを取得
        let p = hyperparameters.and_then(|h| h.get("p")).and_then(|v| v.as_u64()).unwrap_or(1) as usize;
        let d = hyperparameters.and_then(|h| h.get("d")).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let q = hyperparameters.and_then(|h| h.get("q")).and_then(|v| v.as_u64()).unwrap_or(1) as usize;
        
        // 自己回帰係数を初期化
        self.ar_coefficients = vec![0.5; p];
        
        // 移動平均係数を初期化
        self.ma_coefficients = vec![0.5; q];
        
        // 差分次数を設定
        self.d = d;
        
        // 定数項を設定
        self.constant = target.iter().sum::<f64>() / target.len() as f64;
        
        Ok(())
    }
    
    fn predict(&self, features: &HashMap<String, f64>) -> Result<f64, Error> {
        // ARIMAモデルによる予測
        // 実際の実装では、過去の値を使用して予測する
        // ここでは簡易的な実装を提供
        
        let target_feature = "transaction_count";
        let value = features.get(target_feature).cloned().unwrap_or(0.0);
        
        // 自己回帰成分
        let ar_component = self.ar_coefficients.iter().sum::<f64>() * value;
        
        // 移動平均成分
        let ma_component = self.ma_coefficients.iter().sum::<f64>() * 0.1;
        
        // 予測値
        let prediction = self.constant + ar_component + ma_component;
        
        Ok(prediction)
    }
}

/// 勾配ブースティングモデル
struct GradientBoostingModel {
    /// 木の数
    n_trees: usize,
    /// 学習率
    learning_rate: f64,
    /// 特徴量重要度
    importance: HashMap<String, f64>,
}

impl GradientBoostingModel {
    /// 新しい勾配ブースティングモデルを作成
    fn new() -> Self {
        Self {
            n_trees: 100,
            learning_rate: 0.1,
            importance: HashMap::new(),
        }
    }
}

impl PredictionModelTrait for GradientBoostingModel {
    fn train(&mut self, features: &HashMap<String, Vec<f64>>, target: &[f64], hyperparameters: Option<&HashMap<String, serde_json::Value>>) -> Result<(), Error> {
        // 実際の実装では、勾配ブースティングモデルを学習する
        // ここでは簡易的な実装を提供
        
        // ハイパーパラメータを取得
        if let Some(params) = hyperparameters {
            if let Some(n_trees) = params.get("n_trees").and_then(|v| v.as_u64()) {
                self.n_trees = n_trees as usize;
            }
            
            if let Some(learning_rate) = params.get("learning_rate").and_then(|v| v.as_f64()) {
                self.learning_rate = learning_rate;
            }
        }
        
        // 特徴量重要度を計算（仮の実装）
        for (feature_name, values) in features {
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let importance = mean.abs() / 5.0;
            self.importance.insert(feature_name.clone(), importance);
        }
        
        Ok(())
    }
    
    fn predict(&self, features: &HashMap<String, f64>) -> Result<f64, Error> {
        // 勾配ブースティングモデルによる予測
        // 実際の実装では、複数の決定木の予測を組み合わせる
        // ここでは簡易的な実装を提供
        
        let mut prediction = 0.0;
        
        for (feature_name, value) in features {
            if let Some(importance) = self.importance.get(feature_name) {
                prediction += importance * value;
            }
        }
        
        prediction *= self.learning_rate;
        
        Ok(prediction)
    }
    
    fn feature_importance(&self) -> Option<HashMap<String, f64>> {
        if self.importance.is_empty() {
            None
        } else {
            Some(self.importance.clone())
        }
    }
}

/// Prophetモデル
struct ProphetModel {
    /// 成長トレンド
    growth: String,
    /// 季節性
    seasonality_mode: String,
    /// 変化点
    changepoints: Vec<DateTime<Utc>>,
}

impl ProphetModel {
    /// 新しいProphetモデルを作成
    fn new() -> Self {
        Self {
            growth: "linear".to_string(),
            seasonality_mode: "additive".to_string(),
            changepoints: Vec::new(),
        }
    }
}

impl PredictionModelTrait for ProphetModel {
    fn train(&mut self, features: &HashMap<String, Vec<f64>>, target: &[f64], hyperparameters: Option<&HashMap<String, serde_json::Value>>) -> Result<(), Error> {
        // 実際の実装では、Prophetモデルを学習する
        // ここでは簡易的な実装を提供
        
        // ハイパーパラメータを取得
        if let Some(params) = hyperparameters {
            if let Some(growth) = params.get("growth").and_then(|v| v.as_str()) {
                self.growth = growth.to_string();
            }
            
            if let Some(seasonality_mode) = params.get("seasonality_mode").and_then(|v| v.as_str()) {
                self.seasonality_mode = seasonality_mode.to_string();
            }
        }
        
        Ok(())
    }
    
    fn predict(&self, features: &HashMap<String, f64>) -> Result<f64, Error> {
        // Prophetモデルによる予測
        // 実際の実装では、トレンド、季節性、休日効果を組み合わせる
        // ここでは簡易的な実装を提供
        
        let target_feature = "transaction_count";
        let base_value = features.get(target_feature).cloned().unwrap_or(0.0);
        
        // トレンド成分
        let trend = if self.growth == "linear" {
            base_value * 1.05 // 5%の成長
        } else {
            base_value * 1.1 // 10%の成長
        };
        
        // 季節性成分
        let now = Utc::now();
        let hour_of_day = now.hour() as f64;
        let day_of_week = now.weekday().num_days_from_monday() as f64;
        
        let hourly_seasonality = (2.0 * std::f64::consts::PI * hour_of_day / 24.0).sin() * 0.1;
        let weekly_seasonality = (2.0 * std::f64::consts::PI * day_of_week / 7.0).sin() * 0.2;
        
        let seasonality = if self.seasonality_mode == "additive" {
            hourly_seasonality + weekly_seasonality
        } else {
            hourly_seasonality * weekly_seasonality
        };
        
        // 予測値
        let prediction = trend + seasonality * base_value;
        
        Ok(prediction)
    }
}

/// ニューラルネットワークモデル
struct NeuralNetworkModel {
    /// 隠れ層のサイズ
    hidden_layers: Vec<usize>,
    /// 活性化関数
    activation: String,
    /// 特徴量重要度
    importance: HashMap<String, f64>,
}

impl NeuralNetworkModel {
    /// 新しいニューラルネットワークモデルを作成
    fn new() -> Self {
        Self {
            hidden_layers: vec![64, 32],
            activation: "relu".to_string(),
            importance: HashMap::new(),
        }
    }
}

impl PredictionModelTrait for NeuralNetworkModel {
    fn train(&mut self, features: &HashMap<String, Vec<f64>>, target: &[f64], hyperparameters: Option<&HashMap<String, serde_json::Value>>) -> Result<(), Error> {
        // 実際の実装では、ニューラルネットワークモデルを学習する
        // ここでは簡易的な実装を提供
        
        // ハイパーパラメータを取得
        if let Some(params) = hyperparameters {
            if let Some(hidden_layers) = params.get("hidden_layers").and_then(|v| v.as_array()) {
                self.hidden_layers = hidden_layers.iter()
                    .filter_map(|v| v.as_u64().map(|n| n as usize))
                    .collect();
            }
            
            if let Some(activation) = params.get("activation").and_then(|v| v.as_str()) {
                self.activation = activation.to_string();
            }
        }
        
        // 特徴量重要度を計算（仮の実装）
        for (feature_name, values) in features {
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let std_dev = values.iter()
                .map(|v| (*v - mean) * (*v - mean))
                .sum::<f64>() / values.len() as f64;
            
            let importance = std_dev.sqrt() / 3.0;
            self.importance.insert(feature_name.clone(), importance);
        }
        
        Ok(())
    }
    
    fn predict(&self, features: &HashMap<String, f64>) -> Result<f64, Error> {
        // ニューラルネットワークモデルによる予測
        // 実際の実装では、順伝播計算を行う
        // ここでは簡易的な実装を提供
        
        let mut prediction = 0.0;
        
        for (feature_name, value) in features {
            if let Some(importance) = self.importance.get(feature_name) {
                prediction += importance * value;
            }
        }
        
        // 活性化関数を適用
        prediction = match self.activation.as_str() {
            "relu" => prediction.max(0.0),
            "sigmoid" => 1.0 / (1.0 + (-prediction).exp()),
            "tanh" => prediction.tanh(),
            _ => prediction,
        };
        
        Ok(prediction)
    }
    
    fn feature_importance(&self) -> Option<HashMap<String, f64>> {
        if self.importance.is_empty() {
            None
        } else {
            Some(self.importance.clone())
        }
    }
}

/// アンサンブルモデル
struct EnsembleModel {
    /// サブモデル
    models: Vec<Box<dyn PredictionModelTrait>>,
    /// モデルの重み
    weights: Vec<f64>,
    /// 特徴量重要度
    importance: HashMap<String, f64>,
}

impl EnsembleModel {
    /// 新しいアンサンブルモデルを作成
    fn new() -> Self {
        Self {
            models: Vec::new(),
            weights: Vec::new(),
            importance: HashMap::new(),
        }
    }
}

impl PredictionModelTrait for EnsembleModel {
    fn train(&mut self, features: &HashMap<String, Vec<f64>>, target: &[f64], hyperparameters: Option<&HashMap<String, serde_json::Value>>) -> Result<(), Error> {
        // 実際の実装では、複数のモデルを学習し、アンサンブルする
        // ここでは簡易的な実装を提供
        
        // サブモデルを作成
        let mut models: Vec<Box<dyn PredictionModelTrait>> = Vec::new();
        models.push(Box::new(LinearRegressionModel::new()));
        models.push(Box::new(LSTMModel::new()));
        models.push(Box::new(GradientBoostingModel::new()));
        
        // 各モデルを学習
        for model in &mut models {
            model.train(features, target, hyperparameters)?;
        }
        
        // モデルの重みを設定（均等）
        let weight = 1.0 / models.len() as f64;
        let weights = vec![weight; models.len()];
        
        // 特徴量重要度を集約
        let mut importance = HashMap::new();
        
        for model in &models {
            if let Some(model_importance) = model.feature_importance() {
                for (feature, value) in model_importance {
                    let entry = importance.entry(feature).or_insert(0.0);
                    *entry += value / models.len() as f64;
                }
            }
        }
        
        self.models = models;
        self.weights = weights;
        self.importance = importance;
        
        Ok(())
    }
    
    fn predict(&self, features: &HashMap<String, f64>) -> Result<f64, Error> {
        // アンサンブルモデルによる予測
        // 各サブモデルの予測を重み付けして組み合わせる
        
        if self.models.is_empty() {
            return Err(Error::InvalidState("No models in ensemble".to_string()));
        }
        
        let mut weighted_sum = 0.0;
        let mut weight_sum = 0.0;
        
        for (i, model) in self.models.iter().enumerate() {
            let weight = if i < self.weights.len() {
                self.weights[i]
            } else {
                1.0 / self.models.len() as f64
            };
            
            let prediction = model.predict(features)?;
            weighted_sum += prediction * weight;
            weight_sum += weight;
        }
        
        if weight_sum == 0.0 {
            return Err(Error::InvalidState("Sum of weights is zero".to_string()));
        }
        
        Ok(weighted_sum / weight_sum)
    }
    
    fn feature_importance(&self) -> Option<HashMap<String, f64>> {
        if self.importance.is_empty() {
            None
        } else {
            Some(self.importance.clone())
        }
    }
}

/// カスタムモデル
struct CustomModel {
    /// 特徴量重要度
    importance: HashMap<String, f64>,
}

impl CustomModel {
    /// 新しいカスタムモデルを作成
    fn new() -> Self {
        Self {
            importance: HashMap::new(),
        }
    }
}

impl PredictionModelTrait for CustomModel {
    fn train(&mut self, features: &HashMap<String, Vec<f64>>, target: &[f64], _hyperparameters: Option<&HashMap<String, serde_json::Value>>) -> Result<(), Error> {
        // カスタムモデルの学習ロジック
        // 実際の実装では、特定のドメイン知識に基づいたモデルを実装
        
        // 特徴量重要度を計算（仮の実装）
        for (feature_name, values) in features {
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let importance = mean.abs() / 8.0;
            self.importance.insert(feature_name.clone(), importance);
        }
        
        Ok(())
    }
    
    fn predict(&self, features: &HashMap<String, f64>) -> Result<f64, Error> {
        // カスタムモデルによる予測
        // 実際の実装では、特定のドメイン知識に基づいた予測ロジックを実装
        
        let mut prediction = 0.0;
        
        for (feature_name, value) in features {
            if let Some(importance) = self.importance.get(feature_name) {
                prediction += importance * value;
            }
        }
        
        Ok(prediction)
    }
    
    fn feature_importance(&self) -> Option<HashMap<String, f64>> {
        if self.importance.is_empty() {
            None
        } else {
            Some(self.importance.clone())
        }
    }
}

/// ターゲットを文字列に変換
fn target_to_string(target: &PredictionTarget) -> String {
    match target {
        PredictionTarget::TransactionCount => "transaction_count".to_string(),
        PredictionTarget::TransactionVolume => "transaction_volume".to_string(),
        PredictionTarget::GasPrice => "gas_price".to_string(),
        PredictionTarget::BlockTime => "block_time".to_string(),
        PredictionTarget::ActiveAddresses => "active_addresses".to_string(),
        PredictionTarget::NetworkUtilization => "network_utilization".to_string(),
        PredictionTarget::FeeRevenue => "fee_revenue".to_string(),
        PredictionTarget::ShardLoad => "shard_load".to_string(),
        PredictionTarget::Custom(name) => name.clone(),
    }
}

/// タイムスタンプを文字列に変換
fn target_timestamp_to_string(timestamp: &DateTime<Utc>) -> String {
    timestamp.format("%Y%m%d%H%M%S").to_string()
}