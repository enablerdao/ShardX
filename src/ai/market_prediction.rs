use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc, NaiveDateTime, TimeZone};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::transaction::{Transaction, TransactionStatus, TransactionType};
use crate::shard::{ShardId, ShardManager};

/// 予測モデルタイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PredictionModelType {
    /// 線形回帰
    LinearRegression,
    /// 移動平均
    MovingAverage,
    /// 指数平滑法
    ExponentialSmoothing,
    /// ARIMA
    ARIMA,
    /// ニューラルネットワーク
    NeuralNetwork,
    /// ランダムフォレスト
    RandomForest,
    /// アンサンブル
    Ensemble,
}

/// 予測対象
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PredictionTarget {
    /// 価格
    Price,
    /// 取引量
    Volume,
    /// ガス料金
    GasFee,
    /// トランザクション数
    TransactionCount,
    /// ブロック生成時間
    BlockTime,
    /// シャード負荷
    ShardLoad,
}

/// 時間枠
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimeFrame {
    /// 分単位
    Minute(u32),
    /// 時間単位
    Hour(u32),
    /// 日単位
    Day(u32),
    /// 週単位
    Week(u32),
    /// 月単位
    Month(u32),
}

impl TimeFrame {
    /// 秒数に変換
    pub fn to_seconds(&self) -> u64 {
        match self {
            TimeFrame::Minute(n) => *n as u64 * 60,
            TimeFrame::Hour(n) => *n as u64 * 3600,
            TimeFrame::Day(n) => *n as u64 * 86400,
            TimeFrame::Week(n) => *n as u64 * 604800,
            TimeFrame::Month(n) => *n as u64 * 2592000, // 30日で計算
        }
    }
    
    /// 文字列表現を取得
    pub fn to_string(&self) -> String {
        match self {
            TimeFrame::Minute(n) => format!("{}分", n),
            TimeFrame::Hour(n) => format!("{}時間", n),
            TimeFrame::Day(n) => format!("{}日", n),
            TimeFrame::Week(n) => format!("{}週間", n),
            TimeFrame::Month(n) => format!("{}ヶ月", n),
        }
    }
}

/// 予測設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionConfig {
    /// モデルタイプ
    pub model_type: PredictionModelType,
    /// 予測対象
    pub target: PredictionTarget,
    /// 時間枠
    pub time_frame: TimeFrame,
    /// 履歴データ期間（日）
    pub history_days: u32,
    /// 予測期間
    pub forecast_period: TimeFrame,
    /// 信頼区間（0.0-1.0）
    pub confidence_interval: f64,
    /// 特徴量
    pub features: Vec<String>,
    /// ハイパーパラメータ
    pub hyperparameters: HashMap<String, f64>,
    /// 自動再学習
    pub auto_retrain: bool,
    /// 再学習間隔
    pub retrain_interval: TimeFrame,
}

impl Default for PredictionConfig {
    fn default() -> Self {
        Self {
            model_type: PredictionModelType::Ensemble,
            target: PredictionTarget::Price,
            time_frame: TimeFrame::Hour(1),
            history_days: 30,
            forecast_period: TimeFrame::Day(1),
            confidence_interval: 0.95,
            features: vec![
                "price".to_string(),
                "volume".to_string(),
                "transaction_count".to_string(),
                "gas_fee".to_string(),
            ],
            hyperparameters: HashMap::new(),
            auto_retrain: true,
            retrain_interval: TimeFrame::Day(1),
        }
    }
}

/// 予測データポイント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionDataPoint {
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// 予測値
    pub value: f64,
    /// 下限値（信頼区間）
    pub lower_bound: Option<f64>,
    /// 上限値（信頼区間）
    pub upper_bound: Option<f64>,
    /// 実際の値（予測後に判明した場合）
    pub actual_value: Option<f64>,
    /// 予測誤差（実際の値が判明した場合）
    pub error: Option<f64>,
}

/// 予測結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    /// 予測ID
    pub id: String,
    /// 予測対象
    pub target: PredictionTarget,
    /// 時間枠
    pub time_frame: TimeFrame,
    /// 予測データポイント
    pub data_points: Vec<PredictionDataPoint>,
    /// 平均絶対誤差
    pub mean_absolute_error: Option<f64>,
    /// 平均二乗誤差
    pub mean_squared_error: Option<f64>,
    /// 平均絶対パーセント誤差
    pub mean_absolute_percentage_error: Option<f64>,
    /// 作成時刻
    pub created_at: DateTime<Utc>,
    /// 更新時刻
    pub updated_at: DateTime<Utc>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// 特徴量データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureData {
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// 特徴量値
    pub values: HashMap<String, f64>,
}

/// 予測モデル
pub trait PredictionModel {
    /// モデルを学習
    fn train(&mut self, data: &[FeatureData]) -> Result<(), Error>;
    
    /// 予測を実行
    fn predict(&self, features: &HashMap<String, Vec<f64>>) -> Result<Vec<f64>, Error>;
    
    /// モデルを保存
    fn save(&self, path: &str) -> Result<(), Error>;
    
    /// モデルを読み込み
    fn load(&mut self, path: &str) -> Result<(), Error>;
    
    /// モデルタイプを取得
    fn model_type(&self) -> PredictionModelType;
    
    /// モデル名を取得
    fn name(&self) -> String;
    
    /// モデルの説明を取得
    fn description(&self) -> String;
    
    /// モデルのハイパーパラメータを取得
    fn hyperparameters(&self) -> HashMap<String, f64>;
    
    /// モデルのハイパーパラメータを設定
    fn set_hyperparameters(&mut self, params: HashMap<String, f64>) -> Result<(), Error>;
}

/// 線形回帰モデル
pub struct LinearRegressionModel {
    /// 係数
    coefficients: Vec<f64>,
    /// 切片
    intercept: f64,
    /// 特徴量名
    feature_names: Vec<String>,
    /// ハイパーパラメータ
    hyperparameters: HashMap<String, f64>,
    /// 学習済みかどうか
    trained: bool,
}

impl LinearRegressionModel {
    /// 新しい線形回帰モデルを作成
    pub fn new() -> Self {
        let mut hyperparameters = HashMap::new();
        hyperparameters.insert("learning_rate".to_string(), 0.01);
        hyperparameters.insert("regularization".to_string(), 0.001);
        hyperparameters.insert("max_iterations".to_string(), 1000.0);
        
        Self {
            coefficients: Vec::new(),
            intercept: 0.0,
            feature_names: Vec::new(),
            hyperparameters,
            trained: false,
        }
    }
}

impl PredictionModel for LinearRegressionModel {
    fn train(&mut self, data: &[FeatureData]) -> Result<(), Error> {
        if data.is_empty() {
            return Err(Error::InvalidInput("学習データが空です".to_string()));
        }
        
        // 特徴量名を取得
        self.feature_names = data[0].values.keys().cloned().collect();
        
        // 特徴量行列とターゲットベクトルを作成
        let mut x = Vec::new();
        let mut y = Vec::new();
        
        for feature_data in data {
            let mut row = Vec::new();
            for feature_name in &self.feature_names {
                if let Some(value) = feature_data.values.get(feature_name) {
                    row.push(*value);
                } else {
                    return Err(Error::InvalidInput(format!("特徴量 {} が見つかりません", feature_name)));
                }
            }
            
            if let Some(target_value) = feature_data.values.get("target") {
                x.push(row);
                y.push(*target_value);
            } else {
                return Err(Error::InvalidInput("ターゲット値が見つかりません".to_string()));
            }
        }
        
        // 線形回帰の学習（最小二乗法）
        let learning_rate = *self.hyperparameters.get("learning_rate").unwrap_or(&0.01);
        let regularization = *self.hyperparameters.get("regularization").unwrap_or(&0.001);
        let max_iterations = *self.hyperparameters.get("max_iterations").unwrap_or(&1000.0) as usize;
        
        // 係数を初期化
        self.coefficients = vec![0.0; self.feature_names.len()];
        self.intercept = 0.0;
        
        // 勾配降下法による学習
        for _ in 0..max_iterations {
            let mut intercept_gradient = 0.0;
            let mut coefficient_gradients = vec![0.0; self.feature_names.len()];
            
            for i in 0..x.len() {
                let mut prediction = self.intercept;
                for j in 0..self.coefficients.len() {
                    prediction += self.coefficients[j] * x[i][j];
                }
                
                let error = prediction - y[i];
                
                intercept_gradient += error;
                for j in 0..self.coefficients.len() {
                    coefficient_gradients[j] += error * x[i][j];
                }
            }
            
            // 勾配を平均化
            intercept_gradient /= x.len() as f64;
            for j in 0..coefficient_gradients.len() {
                coefficient_gradients[j] /= x.len() as f64;
                // 正則化項を追加
                coefficient_gradients[j] += regularization * self.coefficients[j];
            }
            
            // パラメータを更新
            self.intercept -= learning_rate * intercept_gradient;
            for j in 0..self.coefficients.len() {
                self.coefficients[j] -= learning_rate * coefficient_gradients[j];
            }
        }
        
        self.trained = true;
        
        Ok(())
    }
    
    fn predict(&self, features: &HashMap<String, Vec<f64>>) -> Result<Vec<f64>, Error> {
        if !self.trained {
            return Err(Error::InvalidState("モデルが学習されていません".to_string()));
        }
        
        // 特徴量の長さを確認
        let n_samples = features.values().next().map(|v| v.len()).unwrap_or(0);
        if n_samples == 0 {
            return Err(Error::InvalidInput("特徴量が空です".to_string()));
        }
        
        // 全ての特徴量が同じ長さであることを確認
        for (name, values) in features {
            if values.len() != n_samples {
                return Err(Error::InvalidInput(format!(
                    "特徴量 {} の長さが一致しません: {} != {}",
                    name, values.len(), n_samples
                )));
            }
        }
        
        // 予測を実行
        let mut predictions = vec![0.0; n_samples];
        
        for i in 0..n_samples {
            let mut prediction = self.intercept;
            
            for (j, feature_name) in self.feature_names.iter().enumerate() {
                if let Some(values) = features.get(feature_name) {
                    prediction += self.coefficients[j] * values[i];
                } else {
                    return Err(Error::InvalidInput(format!("特徴量 {} が見つかりません", feature_name)));
                }
            }
            
            predictions[i] = prediction;
        }
        
        Ok(predictions)
    }
    
    fn save(&self, path: &str) -> Result<(), Error> {
        let model_data = serde_json::json!({
            "model_type": "LinearRegression",
            "coefficients": self.coefficients,
            "intercept": self.intercept,
            "feature_names": self.feature_names,
            "hyperparameters": self.hyperparameters,
            "trained": self.trained,
        });
        
        std::fs::write(path, serde_json::to_string_pretty(&model_data)?)
            .map_err(|e| Error::IOError(format!("モデルの保存に失敗しました: {}", e)))?;
        
        Ok(())
    }
    
    fn load(&mut self, path: &str) -> Result<(), Error> {
        let model_data = std::fs::read_to_string(path)
            .map_err(|e| Error::IOError(format!("モデルの読み込みに失敗しました: {}", e)))?;
        
        let model_json: serde_json::Value = serde_json::from_str(&model_data)
            .map_err(|e| Error::ParseError(format!("モデルデータの解析に失敗しました: {}", e)))?;
        
        // モデルタイプを確認
        let model_type = model_json["model_type"].as_str()
            .ok_or_else(|| Error::ParseError("モデルタイプが見つかりません".to_string()))?;
        
        if model_type != "LinearRegression" {
            return Err(Error::InvalidInput(format!(
                "モデルタイプが一致しません: {} != LinearRegression",
                model_type
            )));
        }
        
        // パラメータを読み込み
        self.coefficients = serde_json::from_value(model_json["coefficients"].clone())
            .map_err(|e| Error::ParseError(format!("係数の解析に失敗しました: {}", e)))?;
        
        self.intercept = model_json["intercept"].as_f64()
            .ok_or_else(|| Error::ParseError("切片が見つかりません".to_string()))?;
        
        self.feature_names = serde_json::from_value(model_json["feature_names"].clone())
            .map_err(|e| Error::ParseError(format!("特徴量名の解析に失敗しました: {}", e)))?;
        
        self.hyperparameters = serde_json::from_value(model_json["hyperparameters"].clone())
            .map_err(|e| Error::ParseError(format!("ハイパーパラメータの解析に失敗しました: {}", e)))?;
        
        self.trained = model_json["trained"].as_bool()
            .ok_or_else(|| Error::ParseError("学習状態が見つかりません".to_string()))?;
        
        Ok(())
    }
    
    fn model_type(&self) -> PredictionModelType {
        PredictionModelType::LinearRegression
    }
    
    fn name(&self) -> String {
        "線形回帰モデル".to_string()
    }
    
    fn description(&self) -> String {
        "線形回帰を用いた予測モデル。特徴量の線形結合によって予測値を計算します。".to_string()
    }
    
    fn hyperparameters(&self) -> HashMap<String, f64> {
        self.hyperparameters.clone()
    }
    
    fn set_hyperparameters(&mut self, params: HashMap<String, f64>) -> Result<(), Error> {
        self.hyperparameters = params;
        Ok(())
    }
}

/// 移動平均モデル
pub struct MovingAverageModel {
    /// ウィンドウサイズ
    window_size: usize,
    /// 履歴データ
    history: VecDeque<f64>,
    /// ハイパーパラメータ
    hyperparameters: HashMap<String, f64>,
    /// 学習済みかどうか
    trained: bool,
}

impl MovingAverageModel {
    /// 新しい移動平均モデルを作成
    pub fn new(window_size: usize) -> Self {
        let mut hyperparameters = HashMap::new();
        hyperparameters.insert("window_size".to_string(), window_size as f64);
        
        Self {
            window_size,
            history: VecDeque::new(),
            hyperparameters,
            trained: false,
        }
    }
}

impl PredictionModel for MovingAverageModel {
    fn train(&mut self, data: &[FeatureData]) -> Result<(), Error> {
        if data.is_empty() {
            return Err(Error::InvalidInput("学習データが空です".to_string()));
        }
        
        // ターゲット値を抽出
        let mut target_values = Vec::new();
        
        for feature_data in data {
            if let Some(target_value) = feature_data.values.get("target") {
                target_values.push(*target_value);
            } else {
                return Err(Error::InvalidInput("ターゲット値が見つかりません".to_string()));
            }
        }
        
        // 履歴データを初期化
        self.history.clear();
        for value in target_values {
            self.history.push_back(value);
            if self.history.len() > self.window_size {
                self.history.pop_front();
            }
        }
        
        self.trained = true;
        
        Ok(())
    }
    
    fn predict(&self, features: &HashMap<String, Vec<f64>>) -> Result<Vec<f64>, Error> {
        if !self.trained {
            return Err(Error::InvalidState("モデルが学習されていません".to_string()));
        }
        
        if self.history.is_empty() {
            return Err(Error::InvalidState("履歴データが空です".to_string()));
        }
        
        // 予測期間の長さを取得
        let n_samples = features.values().next().map(|v| v.len()).unwrap_or(0);
        if n_samples == 0 {
            return Err(Error::InvalidInput("特徴量が空です".to_string()));
        }
        
        // 移動平均を計算
        let avg = self.history.iter().sum::<f64>() / self.history.len() as f64;
        
        // 全ての予測値に同じ値を設定
        let predictions = vec![avg; n_samples];
        
        Ok(predictions)
    }
    
    fn save(&self, path: &str) -> Result<(), Error> {
        let model_data = serde_json::json!({
            "model_type": "MovingAverage",
            "window_size": self.window_size,
            "history": Vec::from(self.history.clone()),
            "hyperparameters": self.hyperparameters,
            "trained": self.trained,
        });
        
        std::fs::write(path, serde_json::to_string_pretty(&model_data)?)
            .map_err(|e| Error::IOError(format!("モデルの保存に失敗しました: {}", e)))?;
        
        Ok(())
    }
    
    fn load(&mut self, path: &str) -> Result<(), Error> {
        let model_data = std::fs::read_to_string(path)
            .map_err(|e| Error::IOError(format!("モデルの読み込みに失敗しました: {}", e)))?;
        
        let model_json: serde_json::Value = serde_json::from_str(&model_data)
            .map_err(|e| Error::ParseError(format!("モデルデータの解析に失敗しました: {}", e)))?;
        
        // モデルタイプを確認
        let model_type = model_json["model_type"].as_str()
            .ok_or_else(|| Error::ParseError("モデルタイプが見つかりません".to_string()))?;
        
        if model_type != "MovingAverage" {
            return Err(Error::InvalidInput(format!(
                "モデルタイプが一致しません: {} != MovingAverage",
                model_type
            )));
        }
        
        // パラメータを読み込み
        self.window_size = model_json["window_size"].as_u64()
            .ok_or_else(|| Error::ParseError("ウィンドウサイズが見つかりません".to_string()))? as usize;
        
        let history: Vec<f64> = serde_json::from_value(model_json["history"].clone())
            .map_err(|e| Error::ParseError(format!("履歴データの解析に失敗しました: {}", e)))?;
        
        self.history = VecDeque::from(history);
        
        self.hyperparameters = serde_json::from_value(model_json["hyperparameters"].clone())
            .map_err(|e| Error::ParseError(format!("ハイパーパラメータの解析に失敗しました: {}", e)))?;
        
        self.trained = model_json["trained"].as_bool()
            .ok_or_else(|| Error::ParseError("学習状態が見つかりません".to_string()))?;
        
        Ok(())
    }
    
    fn model_type(&self) -> PredictionModelType {
        PredictionModelType::MovingAverage
    }
    
    fn name(&self) -> String {
        "移動平均モデル".to_string()
    }
    
    fn description(&self) -> String {
        format!("過去 {} 期間の移動平均を用いた予測モデル。", self.window_size)
    }
    
    fn hyperparameters(&self) -> HashMap<String, f64> {
        self.hyperparameters.clone()
    }
    
    fn set_hyperparameters(&mut self, params: HashMap<String, f64>) -> Result<(), Error> {
        if let Some(window_size) = params.get("window_size") {
            self.window_size = *window_size as usize;
        }
        
        self.hyperparameters = params;
        Ok(())
    }
}

/// アンサンブルモデル
pub struct EnsembleModel {
    /// 内部モデル
    models: Vec<Box<dyn PredictionModel>>,
    /// モデルの重み
    weights: Vec<f64>,
    /// ハイパーパラメータ
    hyperparameters: HashMap<String, f64>,
    /// 学習済みかどうか
    trained: bool,
}

impl EnsembleModel {
    /// 新しいアンサンブルモデルを作成
    pub fn new() -> Self {
        let mut hyperparameters = HashMap::new();
        hyperparameters.insert("equal_weights".to_string(), 1.0);
        
        Self {
            models: Vec::new(),
            weights: Vec::new(),
            hyperparameters,
            trained: false,
        }
    }
    
    /// モデルを追加
    pub fn add_model(&mut self, model: Box<dyn PredictionModel>, weight: f64) {
        self.models.push(model);
        self.weights.push(weight);
        
        // 重みを正規化
        let sum = self.weights.iter().sum::<f64>();
        if sum > 0.0 {
            for w in &mut self.weights {
                *w /= sum;
            }
        }
    }
}

impl PredictionModel for EnsembleModel {
    fn train(&mut self, data: &[FeatureData]) -> Result<(), Error> {
        if data.is_empty() {
            return Err(Error::InvalidInput("学習データが空です".to_string()));
        }
        
        if self.models.is_empty() {
            return Err(Error::InvalidState("モデルが追加されていません".to_string()));
        }
        
        // 各モデルを学習
        for model in &mut self.models {
            model.train(data)?;
        }
        
        self.trained = true;
        
        Ok(())
    }
    
    fn predict(&self, features: &HashMap<String, Vec<f64>>) -> Result<Vec<f64>, Error> {
        if !self.trained {
            return Err(Error::InvalidState("モデルが学習されていません".to_string()));
        }
        
        if self.models.is_empty() {
            return Err(Error::InvalidState("モデルが追加されていません".to_string()));
        }
        
        // 予測期間の長さを取得
        let n_samples = features.values().next().map(|v| v.len()).unwrap_or(0);
        if n_samples == 0 {
            return Err(Error::InvalidInput("特徴量が空です".to_string()));
        }
        
        // 各モデルの予測を取得
        let mut all_predictions = Vec::new();
        
        for model in &self.models {
            let predictions = model.predict(features)?;
            all_predictions.push(predictions);
        }
        
        // 重み付き平均を計算
        let mut ensemble_predictions = vec![0.0; n_samples];
        
        for i in 0..n_samples {
            let mut weighted_sum = 0.0;
            
            for (j, predictions) in all_predictions.iter().enumerate() {
                weighted_sum += predictions[i] * self.weights[j];
            }
            
            ensemble_predictions[i] = weighted_sum;
        }
        
        Ok(ensemble_predictions)
    }
    
    fn save(&self, path: &str) -> Result<(), Error> {
        // 各モデルを個別に保存
        for (i, model) in self.models.iter().enumerate() {
            let model_path = format!("{}_model_{}", path, i);
            model.save(&model_path)?;
        }
        
        // アンサンブル設定を保存
        let model_data = serde_json::json!({
            "model_type": "Ensemble",
            "model_count": self.models.len(),
            "weights": self.weights,
            "hyperparameters": self.hyperparameters,
            "trained": self.trained,
        });
        
        std::fs::write(path, serde_json::to_string_pretty(&model_data)?)
            .map_err(|e| Error::IOError(format!("モデルの保存に失敗しました: {}", e)))?;
        
        Ok(())
    }
    
    fn load(&mut self, path: &str) -> Result<(), Error> {
        let model_data = std::fs::read_to_string(path)
            .map_err(|e| Error::IOError(format!("モデルの読み込みに失敗しました: {}", e)))?;
        
        let model_json: serde_json::Value = serde_json::from_str(&model_data)
            .map_err(|e| Error::ParseError(format!("モデルデータの解析に失敗しました: {}", e)))?;
        
        // モデルタイプを確認
        let model_type = model_json["model_type"].as_str()
            .ok_or_else(|| Error::ParseError("モデルタイプが見つかりません".to_string()))?;
        
        if model_type != "Ensemble" {
            return Err(Error::InvalidInput(format!(
                "モデルタイプが一致しません: {} != Ensemble",
                model_type
            )));
        }
        
        // パラメータを読み込み
        let model_count = model_json["model_count"].as_u64()
            .ok_or_else(|| Error::ParseError("モデル数が見つかりません".to_string()))? as usize;
        
        self.weights = serde_json::from_value(model_json["weights"].clone())
            .map_err(|e| Error::ParseError(format!("重みの解析に失敗しました: {}", e)))?;
        
        self.hyperparameters = serde_json::from_value(model_json["hyperparameters"].clone())
            .map_err(|e| Error::ParseError(format!("ハイパーパラメータの解析に失敗しました: {}", e)))?;
        
        self.trained = model_json["trained"].as_bool()
            .ok_or_else(|| Error::ParseError("学習状態が見つかりません".to_string()))?;
        
        // 各モデルを読み込み
        self.models.clear();
        
        for i in 0..model_count {
            let model_path = format!("{}_model_{}", path, i);
            
            // モデルタイプを確認
            let model_data = std::fs::read_to_string(&model_path)
                .map_err(|e| Error::IOError(format!("モデルの読み込みに失敗しました: {}", e)))?;
            
            let model_json: serde_json::Value = serde_json::from_str(&model_data)
                .map_err(|e| Error::ParseError(format!("モデルデータの解析に失敗しました: {}", e)))?;
            
            let sub_model_type = model_json["model_type"].as_str()
                .ok_or_else(|| Error::ParseError("モデルタイプが見つかりません".to_string()))?;
            
            // モデルタイプに応じてモデルを作成
            let mut model: Box<dyn PredictionModel> = match sub_model_type {
                "LinearRegression" => Box::new(LinearRegressionModel::new()),
                "MovingAverage" => {
                    let window_size = model_json["window_size"].as_u64()
                        .ok_or_else(|| Error::ParseError("ウィンドウサイズが見つかりません".to_string()))? as usize;
                    Box::new(MovingAverageModel::new(window_size))
                },
                _ => return Err(Error::InvalidInput(format!("未対応のモデルタイプです: {}", sub_model_type))),
            };
            
            // モデルを読み込み
            model.load(&model_path)?;
            
            // モデルを追加
            self.models.push(model);
        }
        
        Ok(())
    }
    
    fn model_type(&self) -> PredictionModelType {
        PredictionModelType::Ensemble
    }
    
    fn name(&self) -> String {
        "アンサンブルモデル".to_string()
    }
    
    fn description(&self) -> String {
        format!("{} 個のモデルを組み合わせたアンサンブルモデル。", self.models.len())
    }
    
    fn hyperparameters(&self) -> HashMap<String, f64> {
        self.hyperparameters.clone()
    }
    
    fn set_hyperparameters(&mut self, params: HashMap<String, f64>) -> Result<(), Error> {
        // 等重みフラグをチェック
        if let Some(equal_weights) = params.get("equal_weights") {
            if *equal_weights > 0.5 && !self.models.is_empty() {
                // 全てのモデルに等しい重みを設定
                let weight = 1.0 / self.models.len() as f64;
                self.weights = vec![weight; self.models.len()];
            }
        }
        
        self.hyperparameters = params;
        Ok(())
    }
}

/// 予測サービス
pub struct MarketPredictionService {
    /// 設定
    config: PredictionConfig,
    /// モデル
    model: Box<dyn PredictionModel>,
    /// 特徴量データ
    feature_data: Arc<Mutex<Vec<FeatureData>>>,
    /// 予測結果
    predictions: Arc<RwLock<HashMap<String, PredictionResult>>>,
    /// 最終学習時刻
    last_trained: Arc<Mutex<DateTime<Utc>>>,
}

impl MarketPredictionService {
    /// 新しい予測サービスを作成
    pub fn new(config: PredictionConfig) -> Result<Self, Error> {
        // モデルを作成
        let model: Box<dyn PredictionModel> = match config.model_type {
            PredictionModelType::LinearRegression => Box::new(LinearRegressionModel::new()),
            PredictionModelType::MovingAverage => Box::new(MovingAverageModel::new(24)),
            PredictionModelType::Ensemble => Box::new(EnsembleModel::new()),
            _ => return Err(Error::InvalidInput(format!("未対応のモデルタイプです: {:?}", config.model_type))),
        };
        
        Ok(Self {
            config,
            model,
            feature_data: Arc::new(Mutex::new(Vec::new())),
            predictions: Arc::new(RwLock::new(HashMap::new())),
            last_trained: Arc::new(Mutex::new(Utc::now())),
        })
    }
    
    /// 特徴量データを追加
    pub fn add_feature_data(&self, data: FeatureData) -> Result<(), Error> {
        let mut feature_data = self.feature_data.lock().unwrap();
        feature_data.push(data);
        
        // 古いデータを削除
        let cutoff = Utc::now() - chrono::Duration::days(self.config.history_days as i64);
        feature_data.retain(|d| d.timestamp >= cutoff);
        
        // 自動再学習
        if self.config.auto_retrain {
            let mut last_trained = self.last_trained.lock().unwrap();
            let now = Utc::now();
            let retrain_seconds = self.config.retrain_interval.to_seconds();
            
            if (now - *last_trained).num_seconds() > retrain_seconds as i64 {
                drop(last_trained); // ロックを解放
                drop(feature_data); // ロックを解放
                
                self.train()?;
                
                let mut last_trained = self.last_trained.lock().unwrap();
                *last_trained = now;
            }
        }
        
        Ok(())
    }
    
    /// モデルを学習
    pub fn train(&self) -> Result<(), Error> {
        let feature_data = self.feature_data.lock().unwrap();
        
        if feature_data.is_empty() {
            return Err(Error::InvalidInput("学習データが空です".to_string()));
        }
        
        // モデルを学習
        let mut model = self.model.clone();
        model.train(&feature_data)?;
        
        // モデルを更新
        let mut model_mut = &mut self.model;
        *model_mut = model;
        
        Ok(())
    }
    
    /// 予測を実行
    pub fn predict(&self) -> Result<PredictionResult, Error> {
        // 特徴量データを取得
        let feature_data = self.feature_data.lock().unwrap();
        
        if feature_data.is_empty() {
            return Err(Error::InvalidInput("特徴量データが空です".to_string()));
        }
        
        // 予測期間を計算
        let now = Utc::now();
        let forecast_seconds = self.config.forecast_period.to_seconds();
        let time_frame_seconds = self.config.time_frame.to_seconds();
        
        let n_periods = (forecast_seconds / time_frame_seconds) as usize;
        if n_periods == 0 {
            return Err(Error::InvalidInput("予測期間が時間枠より短いです".to_string()));
        }
        
        // 特徴量を準備
        let mut features: HashMap<String, Vec<f64>> = HashMap::new();
        
        for feature_name in &self.config.features {
            let mut values = Vec::new();
            
            // 過去の値を取得
            for data in feature_data.iter().rev().take(10) {
                if let Some(value) = data.values.get(feature_name) {
                    values.push(*value);
                }
            }
            
            // 値を反転して時系列順にする
            values.reverse();
            
            // 予測期間分の値を追加（ダミー値）
            let last_value = values.last().cloned().unwrap_or(0.0);
            for _ in 0..n_periods {
                values.push(last_value);
            }
            
            features.insert(feature_name.clone(), values);
        }
        
        // 予測を実行
        let predictions = self.model.predict(&features)?;
        
        // 予測結果を作成
        let mut data_points = Vec::new();
        
        for i in 0..n_periods {
            let timestamp = now + chrono::Duration::seconds((i as u64 * time_frame_seconds) as i64);
            
            // 信頼区間を計算
            let confidence_interval = self.config.confidence_interval;
            let std_dev = 0.1 * predictions[i].abs(); // 仮の標準偏差
            let z_score = 1.96; // 95%信頼区間のz値
            let margin = z_score * std_dev;
            
            let data_point = PredictionDataPoint {
                timestamp,
                value: predictions[i],
                lower_bound: Some(predictions[i] - margin),
                upper_bound: Some(predictions[i] + margin),
                actual_value: None,
                error: None,
            };
            
            data_points.push(data_point);
        }
        
        // 予測結果を作成
        let prediction_id = format!("pred-{}", Utc::now().timestamp());
        let prediction_result = PredictionResult {
            id: prediction_id.clone(),
            target: self.config.target.clone(),
            time_frame: self.config.time_frame.clone(),
            data_points,
            mean_absolute_error: None,
            mean_squared_error: None,
            mean_absolute_percentage_error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };
        
        // 予測結果を保存
        let mut predictions = self.predictions.write().unwrap();
        predictions.insert(prediction_id.clone(), prediction_result.clone());
        
        Ok(prediction_result)
    }
    
    /// 予測結果を取得
    pub fn get_prediction(&self, prediction_id: &str) -> Result<PredictionResult, Error> {
        let predictions = self.predictions.read().unwrap();
        
        predictions.get(prediction_id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("予測 {} が見つかりません", prediction_id)))
    }
    
    /// 全予測結果を取得
    pub fn get_all_predictions(&self) -> Vec<PredictionResult> {
        let predictions = self.predictions.read().unwrap();
        predictions.values().cloned().collect()
    }
    
    /// 予測結果を更新（実際の値を追加）
    pub fn update_prediction(&self, prediction_id: &str, timestamp: DateTime<Utc>, actual_value: f64) -> Result<(), Error> {
        let mut predictions = self.predictions.write().unwrap();
        
        let prediction = predictions.get_mut(prediction_id)
            .ok_or_else(|| Error::NotFound(format!("予測 {} が見つかりません", prediction_id)))?;
        
        // データポイントを更新
        for data_point in &mut prediction.data_points {
            if (data_point.timestamp - timestamp).num_seconds().abs() < 60 {
                data_point.actual_value = Some(actual_value);
                data_point.error = Some((data_point.value - actual_value).abs());
                break;
            }
        }
        
        // 誤差指標を更新
        let mut errors = Vec::new();
        let mut squared_errors = Vec::new();
        let mut percentage_errors = Vec::new();
        
        for data_point in &prediction.data_points {
            if let (Some(actual), Some(error)) = (data_point.actual_value, data_point.error) {
                errors.push(error);
                squared_errors.push(error * error);
                
                if actual.abs() > 1e-10 {
                    percentage_errors.push(error / actual.abs());
                }
            }
        }
        
        if !errors.is_empty() {
            prediction.mean_absolute_error = Some(errors.iter().sum::<f64>() / errors.len() as f64);
            prediction.mean_squared_error = Some(squared_errors.iter().sum::<f64>() / squared_errors.len() as f64);
        }
        
        if !percentage_errors.is_empty() {
            prediction.mean_absolute_percentage_error = Some(percentage_errors.iter().sum::<f64>() / percentage_errors.len() as f64);
        }
        
        prediction.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// 設定を更新
    pub fn update_config(&mut self, config: PredictionConfig) -> Result<(), Error> {
        // モデルタイプが変更された場合は新しいモデルを作成
        if config.model_type != self.config.model_type {
            let model: Box<dyn PredictionModel> = match config.model_type {
                PredictionModelType::LinearRegression => Box::new(LinearRegressionModel::new()),
                PredictionModelType::MovingAverage => Box::new(MovingAverageModel::new(24)),
                PredictionModelType::Ensemble => Box::new(EnsembleModel::new()),
                _ => return Err(Error::InvalidInput(format!("未対応のモデルタイプです: {:?}", config.model_type))),
            };
            
            self.model = model;
        }
        
        self.config = config;
        
        Ok(())
    }
    
    /// モデルを保存
    pub fn save_model(&self, path: &str) -> Result<(), Error> {
        self.model.save(path)
    }
    
    /// モデルを読み込み
    pub fn load_model(&mut self, path: &str) -> Result<(), Error> {
        let mut model = self.model.clone();
        model.load(path)?;
        
        self.model = model;
        
        Ok(())
    }
}

/// 予測サービスマネージャー
pub struct MarketPredictionServiceManager {
    /// 予測サービス
    services: Arc<RwLock<HashMap<String, Arc<MarketPredictionService>>>>,
}

impl MarketPredictionServiceManager {
    /// 新しい予測サービスマネージャーを作成
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 予測サービスを作成
    pub fn create_service(&self, name: &str, config: PredictionConfig) -> Result<(), Error> {
        let mut services = self.services.write().unwrap();
        
        if services.contains_key(name) {
            return Err(Error::AlreadyExists(format!("予測サービス {} は既に存在します", name)));
        }
        
        let service = Arc::new(MarketPredictionService::new(config)?);
        services.insert(name.to_string(), service);
        
        Ok(())
    }
    
    /// 予測サービスを取得
    pub fn get_service(&self, name: &str) -> Result<Arc<MarketPredictionService>, Error> {
        let services = self.services.read().unwrap();
        
        services.get(name)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("予測サービス {} が見つかりません", name)))
    }
    
    /// 予測サービスを削除
    pub fn delete_service(&self, name: &str) -> Result<(), Error> {
        let mut services = self.services.write().unwrap();
        
        if services.remove(name).is_none() {
            return Err(Error::NotFound(format!("予測サービス {} が見つかりません", name)));
        }
        
        Ok(())
    }
    
    /// 全予測サービスを取得
    pub fn get_all_services(&self) -> Vec<String> {
        let services = self.services.read().unwrap();
        services.keys().cloned().collect()
    }
}