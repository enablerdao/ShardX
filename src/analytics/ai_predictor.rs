use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::transaction::{Transaction, TransactionType};
use crate::analytics::metrics::{MetricType, MetricValue};

/// 予測モデル
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PredictionModel {
    /// 線形回帰
    LinearRegression,
    /// ロジスティック回帰
    LogisticRegression,
    /// 決定木
    DecisionTree,
    /// ランダムフォレスト
    RandomForest,
    /// 勾配ブースティング
    GradientBoosting,
    /// サポートベクターマシン
    SVM,
    /// ニューラルネットワーク
    NeuralNetwork,
    /// 長短期記憶ネットワーク
    LSTM,
    /// 畳み込みニューラルネットワーク
    CNN,
    /// 自己回帰和分移動平均
    ARIMA,
    /// 指数平滑法
    ExponentialSmoothing,
    /// プロフェット
    Prophet,
    /// 自己組織化マップ
    SOM,
    /// カスタムモデル
    Custom(String),
}

/// 予測結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    /// 予測値
    pub value: f64,
    /// 信頼区間下限
    pub lower_bound: Option<f64>,
    /// 信頼区間上限
    pub upper_bound: Option<f64>,
    /// 信頼度
    pub confidence: Option<f64>,
    /// 予測時刻
    pub timestamp: DateTime<Utc>,
    /// 予測対象時刻
    pub target_timestamp: DateTime<Utc>,
    /// 特徴量重要度
    pub feature_importance: Option<HashMap<String, f64>>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// 予測設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionConfig {
    /// 予測モデル
    pub model: PredictionModel,
    /// 学習期間（秒）
    pub training_period_seconds: u64,
    /// 予測期間（秒）
    pub prediction_period_seconds: u64,
    /// 特徴量
    pub features: Vec<String>,
    /// ターゲット
    pub target: String,
    /// 信頼区間
    pub confidence_interval: Option<f64>,
    /// 再学習間隔（秒）
    pub retraining_interval_seconds: Option<u64>,
    /// ハイパーパラメータ
    pub hyperparameters: Option<HashMap<String, serde_json::Value>>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

impl Default for PredictionConfig {
    fn default() -> Self {
        Self {
            model: PredictionModel::LSTM,
            training_period_seconds: 604800, // 1週間
            prediction_period_seconds: 86400, // 1日
            features: vec![
                "transaction_count".to_string(),
                "transaction_volume".to_string(),
                "average_fee".to_string(),
                "active_users".to_string(),
            ],
            target: "transaction_count".to_string(),
            confidence_interval: Some(0.95),
            retraining_interval_seconds: Some(86400), // 1日
            hyperparameters: None,
            additional_properties: HashMap::new(),
        }
    }
}

/// トランザクション予測器
pub struct TransactionPredictor {
    /// 設定
    config: PredictionConfig,
    /// モデル
    model: Option<Box<dyn Model>>,
    /// 最後の学習時刻
    last_training_time: Option<DateTime<Utc>>,
    /// 予測履歴
    prediction_history: Vec<PredictionResult>,
    /// 特徴量データ
    feature_data: HashMap<String, Vec<(DateTime<Utc>, f64)>>,
    /// ターゲットデータ
    target_data: Vec<(DateTime<Utc>, f64)>,
}

impl TransactionPredictor {
    /// 新しいトランザクション予測器を作成
    pub fn new(config: PredictionConfig) -> Self {
        Self {
            config,
            model: None,
            last_training_time: None,
            prediction_history: Vec::new(),
            feature_data: HashMap::new(),
            target_data: Vec::new(),
        }
    }
    
    /// 特徴量データを追加
    pub fn add_feature_data(&mut self, feature: &str, timestamp: DateTime<Utc>, value: f64) {
        let feature_data = self.feature_data.entry(feature.to_string()).or_insert_with(Vec::new);
        feature_data.push((timestamp, value));
    }
    
    /// ターゲットデータを追加
    pub fn add_target_data(&mut self, timestamp: DateTime<Utc>, value: f64) {
        self.target_data.push((timestamp, value));
    }
    
    /// メトリクスデータを追加
    pub fn add_metric_data(&mut self, metric_type: &MetricType, timestamp: DateTime<Utc>, value: &MetricValue) {
        let metric_name = format!("{:?}", metric_type);
        
        match value {
            MetricValue::Integer(i) => {
                self.add_feature_data(&metric_name, timestamp, *i as f64);
                
                if metric_name == self.config.target {
                    self.add_target_data(timestamp, *i as f64);
                }
            },
            MetricValue::Float(f) => {
                self.add_feature_data(&metric_name, timestamp, *f);
                
                if metric_name == self.config.target {
                    self.add_target_data(timestamp, *f);
                }
            },
            MetricValue::Counter(c) => {
                self.add_feature_data(&metric_name, timestamp, *c as f64);
                
                if metric_name == self.config.target {
                    self.add_target_data(timestamp, *c as f64);
                }
            },
            _ => {
                // 他のメトリクス型は無視
            }
        }
    }
    
    /// トランザクションデータを追加
    pub fn add_transaction_data(&mut self, transactions: &[Transaction], timestamp: DateTime<Utc>) {
        // トランザクション数
        let transaction_count = transactions.len() as f64;
        self.add_feature_data("transaction_count", timestamp, transaction_count);
        
        // トランザクション量
        let transaction_volume: f64 = transactions.iter()
            .map(|tx| tx.amount as f64)
            .sum();
        self.add_feature_data("transaction_volume", timestamp, transaction_volume);
        
        // 平均手数料
        let total_fees: f64 = transactions.iter()
            .map(|tx| tx.fee as f64)
            .sum();
        let average_fee = if transaction_count > 0.0 {
            total_fees / transaction_count
        } else {
            0.0
        };
        self.add_feature_data("average_fee", timestamp, average_fee);
        
        // アクティブユーザー
        let unique_users: std::collections::HashSet<_> = transactions.iter()
            .flat_map(|tx| vec![tx.sender.clone(), tx.receiver.clone()])
            .collect();
        let active_users = unique_users.len() as f64;
        self.add_feature_data("active_users", timestamp, active_users);
        
        // トランザクションタイプごとの数
        let mut type_counts: HashMap<TransactionType, u64> = HashMap::new();
        for tx in transactions {
            *type_counts.entry(tx.transaction_type.clone()).or_insert(0) += 1;
        }
        
        for (tx_type, count) in type_counts {
            let feature_name = format!("transaction_type_{:?}", tx_type);
            self.add_feature_data(&feature_name, timestamp, count as f64);
        }
        
        // ターゲットデータを追加
        if self.config.target == "transaction_count" {
            self.add_target_data(timestamp, transaction_count);
        } else if self.config.target == "transaction_volume" {
            self.add_target_data(timestamp, transaction_volume);
        } else if self.config.target == "average_fee" {
            self.add_target_data(timestamp, average_fee);
        } else if self.config.target == "active_users" {
            self.add_target_data(timestamp, active_users);
        }
    }
    
    /// モデルを学習
    pub fn train(&mut self) -> Result<(), Error> {
        let now = Utc::now();
        
        // 再学習間隔をチェック
        if let Some(last_training_time) = self.last_training_time {
            if let Some(retraining_interval) = self.config.retraining_interval_seconds {
                let elapsed = now.timestamp() - last_training_time.timestamp();
                if elapsed < retraining_interval as i64 {
                    return Ok(());
                }
            }
        }
        
        // 学習期間のデータを取得
        let training_start = now - chrono::Duration::seconds(self.config.training_period_seconds as i64);
        
        // 特徴量データを準備
        let mut features: HashMap<String, Vec<f64>> = HashMap::new();
        let mut timestamps: Vec<DateTime<Utc>> = Vec::new();
        
        for feature_name in &self.config.features {
            if let Some(feature_data) = self.feature_data.get(feature_name) {
                let filtered_data: Vec<_> = feature_data.iter()
                    .filter(|(ts, _)| *ts >= training_start)
                    .collect();
                
                if filtered_data.is_empty() {
                    return Err(Error::InvalidState(format!("No data for feature: {}", feature_name)));
                }
                
                // タイムスタンプを記録
                if timestamps.is_empty() {
                    timestamps = filtered_data.iter().map(|(ts, _)| *ts).collect();
                }
                
                // 特徴量値を記録
                features.insert(feature_name.clone(), filtered_data.iter().map(|(_, v)| *v).collect());
            } else {
                return Err(Error::InvalidState(format!("Feature not found: {}", feature_name)));
            }
        }
        
        // ターゲットデータを準備
        let filtered_target: Vec<_> = self.target_data.iter()
            .filter(|(ts, _)| *ts >= training_start)
            .collect();
        
        if filtered_target.is_empty() {
            return Err(Error::InvalidState(format!("No data for target: {}", self.config.target)));
        }
        
        let target_values: Vec<f64> = filtered_target.iter().map(|(_, v)| *v).collect();
        
        // モデルを作成
        let mut model: Box<dyn Model> = match self.config.model {
            PredictionModel::LinearRegression => Box::new(LinearRegressionModel::new()),
            PredictionModel::LSTM => Box::new(LSTMModel::new()),
            PredictionModel::ARIMA => Box::new(ARIMAModel::new()),
            _ => return Err(Error::NotImplemented(format!("Model not implemented: {:?}", self.config.model))),
        };
        
        // モデルを学習
        model.train(&features, &target_values, self.config.hyperparameters.as_ref())?;
        
        // モデルを保存
        self.model = Some(model);
        self.last_training_time = Some(now);
        
        Ok(())
    }
    
    /// 予測を実行
    pub fn predict(&mut self) -> Result<Vec<PredictionResult>, Error> {
        let now = Utc::now();
        
        // モデルをチェック
        if self.model.is_none() {
            self.train()?;
        }
        
        let model = self.model.as_ref().ok_or_else(|| {
            Error::InvalidState("Model not trained".to_string())
        })?;
        
        // 予測期間を計算
        let prediction_end = now + chrono::Duration::seconds(self.config.prediction_period_seconds as i64);
        
        // 予測間隔を計算（1時間ごと）
        let interval_seconds = 3600;
        let intervals = (self.config.prediction_period_seconds / interval_seconds) as usize;
        
        // 最新の特徴量データを取得
        let mut latest_features: HashMap<String, f64> = HashMap::new();
        
        for feature_name in &self.config.features {
            if let Some(feature_data) = self.feature_data.get(feature_name) {
                if let Some((_, value)) = feature_data.last() {
                    latest_features.insert(feature_name.clone(), *value);
                } else {
                    return Err(Error::InvalidState(format!("No data for feature: {}", feature_name)));
                }
            } else {
                return Err(Error::InvalidState(format!("Feature not found: {}", feature_name)));
            }
        }
        
        // 予測を実行
        let mut predictions = Vec::new();
        
        for i in 1..=intervals {
            let target_timestamp = now + chrono::Duration::seconds((i * interval_seconds) as i64);
            
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
            
            // 予測結果を作成
            let result = PredictionResult {
                value: prediction,
                lower_bound,
                upper_bound,
                confidence: self.config.confidence_interval,
                timestamp: now,
                target_timestamp,
                feature_importance: None,
                additional_properties: HashMap::new(),
            };
            
            predictions.push(result.clone());
            
            // 予測履歴に追加
            self.prediction_history.push(result);
            
            // 次の予測のために特徴量を更新
            latest_features.insert(self.config.target.clone(), prediction);
        }
        
        Ok(predictions)
    }
    
    /// 予測精度を評価
    pub fn evaluate_accuracy(&self) -> Result<f64, Error> {
        // 過去の予測と実際の値を比較
        let mut total_error = 0.0;
        let mut count = 0;
        
        for prediction in &self.prediction_history {
            // 予測対象時刻に最も近い実際の値を検索
            let actual = self.target_data.iter()
                .min_by_key(|(ts, _)| {
                    let diff = (*ts - prediction.target_timestamp).num_seconds().abs();
                    diff as u64
                });
            
            if let Some((_, actual_value)) = actual {
                let error = (prediction.value - *actual_value).abs();
                total_error += error;
                count += 1;
            }
        }
        
        if count == 0 {
            return Err(Error::InvalidState("No predictions to evaluate".to_string()));
        }
        
        // 平均絶対誤差を計算
        let mae = total_error / count as f64;
        
        // 精度を計算（1 - 正規化された誤差）
        let max_value = self.target_data.iter().map(|(_, v)| *v).fold(0.0, f64::max);
        let min_value = self.target_data.iter().map(|(_, v)| *v).fold(f64::INFINITY, f64::min);
        let range = max_value - min_value;
        
        if range <= 0.0 {
            return Ok(0.0);
        }
        
        let normalized_error = mae / range;
        let accuracy = 1.0 - normalized_error;
        
        Ok(accuracy)
    }
    
    /// 設定を取得
    pub fn get_config(&self) -> &PredictionConfig {
        &self.config
    }
    
    /// 設定を更新
    pub fn update_config(&mut self, config: PredictionConfig) {
        self.config = config;
        self.model = None; // モデルを再学習するためにリセット
    }
    
    /// 予測履歴を取得
    pub fn get_prediction_history(&self) -> &[PredictionResult] {
        &self.prediction_history
    }
    
    /// 特徴量データを取得
    pub fn get_feature_data(&self, feature: &str) -> Option<&[(DateTime<Utc>, f64)]> {
        self.feature_data.get(feature).map(|data| data.as_slice())
    }
    
    /// ターゲットデータを取得
    pub fn get_target_data(&self) -> &[(DateTime<Utc>, f64)] {
        &self.target_data
    }
}

/// モデルトレイト
trait Model {
    /// モデルを学習
    fn train(&mut self, features: &HashMap<String, Vec<f64>>, target: &[f64], hyperparameters: Option<&HashMap<String, serde_json::Value>>) -> Result<(), Error>;
    
    /// 予測を実行
    fn predict(&self, features: &HashMap<String, f64>) -> Result<f64, Error>;
}

/// 線形回帰モデル
struct LinearRegressionModel {
    /// 係数
    coefficients: HashMap<String, f64>,
    /// 切片
    intercept: f64,
}

impl LinearRegressionModel {
    /// 新しい線形回帰モデルを作成
    fn new() -> Self {
        Self {
            coefficients: HashMap::new(),
            intercept: 0.0,
        }
    }
}

impl Model for LinearRegressionModel {
    fn train(&mut self, features: &HashMap<String, Vec<f64>>, target: &[f64], _hyperparameters: Option<&HashMap<String, serde_json::Value>>) -> Result<(), Error> {
        // 実際の実装では、線形回帰モデルを学習する
        // ここでは簡易的な実装を提供
        
        // 各特徴量の平均値を係数として使用
        for (feature_name, values) in features {
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            self.coefficients.insert(feature_name.clone(), mean);
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
}

/// LSTMモデル
struct LSTMModel {
    /// 重み
    weights: HashMap<String, f64>,
    /// バイアス
    bias: f64,
}

impl LSTMModel {
    /// 新しいLSTMモデルを作成
    fn new() -> Self {
        Self {
            weights: HashMap::new(),
            bias: 0.0,
        }
    }
}

impl Model for LSTMModel {
    fn train(&mut self, features: &HashMap<String, Vec<f64>>, target: &[f64], _hyperparameters: Option<&HashMap<String, serde_json::Value>>) -> Result<(), Error> {
        // 実際の実装では、LSTMモデルを学習する
        // ここでは簡易的な実装を提供
        
        // 各特徴量の最新値を重みとして使用
        for (feature_name, values) in features {
            if let Some(last_value) = values.last() {
                self.weights.insert(feature_name.clone(), *last_value);
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

impl Model for ARIMAModel {
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