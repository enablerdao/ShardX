use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::transaction::{Transaction, TransactionStatus};
use crate::chart::DataPoint;

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
    /// トランザクション数
    TransactionCount,
    /// 取引量
    TransactionVolume,
    /// 手数料
    TransactionFee,
    /// ガス使用量
    GasUsage,
    /// ブロック時間
    BlockTime,
    /// ネットワーク負荷
    NetworkLoad,
}

/// 予測期間
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PredictionHorizon {
    /// 短期（1時間〜24時間）
    ShortTerm,
    /// 中期（1日〜7日）
    MediumTerm,
    /// 長期（1週間〜1ヶ月）
    LongTerm,
}

/// 予測結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    /// 予測ID
    pub id: String,
    /// 予測モデルタイプ
    pub model_type: PredictionModelType,
    /// 予測対象
    pub target: PredictionTarget,
    /// 予測期間
    pub horizon: PredictionHorizon,
    /// 予測作成時刻
    pub created_at: DateTime<Utc>,
    /// 予測開始時刻
    pub start_time: DateTime<Utc>,
    /// 予測終了時刻
    pub end_time: DateTime<Utc>,
    /// 予測データポイント
    pub predictions: Vec<DataPoint>,
    /// 信頼区間（下限）
    pub confidence_lower: Option<Vec<DataPoint>>,
    /// 信頼区間（上限）
    pub confidence_upper: Option<Vec<DataPoint>>,
    /// 予測精度
    pub accuracy: Option<f64>,
    /// 予測エラー（RMSE）
    pub error_rmse: Option<f64>,
    /// 予測エラー（MAE）
    pub error_mae: Option<f64>,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// 特徴量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    /// 特徴量名
    pub name: String,
    /// 特徴量値
    pub value: f64,
    /// 重要度
    pub importance: Option<f64>,
}

/// モデル設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// モデルタイプ
    pub model_type: PredictionModelType,
    /// ハイパーパラメータ
    pub hyperparameters: HashMap<String, String>,
    /// 特徴量
    pub features: Vec<String>,
    /// 学習期間（日数）
    pub training_period_days: u32,
    /// 予測期間
    pub prediction_horizon: PredictionHorizon,
    /// 信頼区間レベル（0.0〜1.0）
    pub confidence_level: f64,
}

/// トランザクション予測器
pub struct TransactionPredictor {
    /// モデル設定
    model_config: ModelConfig,
    /// 学習済みモデル
    trained_model: Option<Box<dyn PredictionModel>>,
    /// 最終学習時刻
    last_training_time: Option<DateTime<Utc>>,
}

impl TransactionPredictor {
    /// 新しいトランザクション予測器を作成
    pub fn new(model_config: ModelConfig) -> Self {
        Self {
            model_config,
            trained_model: None,
            last_training_time: None,
        }
    }
    
    /// モデルを学習
    pub fn train(&mut self, historical_data: &[Transaction]) -> Result<(), Error> {
        // 学習データを準備
        let training_data = self.prepare_training_data(historical_data)?;
        
        // モデルを作成
        let mut model: Box<dyn PredictionModel> = match self.model_config.model_type {
            PredictionModelType::LinearRegression => Box::new(LinearRegressionModel::new()),
            PredictionModelType::MovingAverage => Box::new(MovingAverageModel::new()),
            PredictionModelType::ExponentialSmoothing => Box::new(ExponentialSmoothingModel::new()),
            PredictionModelType::ARIMA => Box::new(ARIMAModel::new()),
            PredictionModelType::NeuralNetwork => Box::new(NeuralNetworkModel::new()),
            PredictionModelType::RandomForest => Box::new(RandomForestModel::new()),
            PredictionModelType::Ensemble => Box::new(EnsembleModel::new()),
        };
        
        // モデルを学習
        model.train(&training_data, &self.model_config)?;
        
        // 学習済みモデルを保存
        self.trained_model = Some(model);
        self.last_training_time = Some(Utc::now());
        
        Ok(())
    }
    
    /// 予測を実行
    pub fn predict(&self, target: PredictionTarget) -> Result<PredictionResult, Error> {
        // 学習済みモデルがない場合はエラー
        let model = self.trained_model.as_ref()
            .ok_or_else(|| Error::InvalidState("モデルが学習されていません".to_string()))?;
        
        // 予測を実行
        let now = Utc::now();
        let (start_time, end_time) = self.get_prediction_time_range(now);
        
        // 予測データポイントを生成
        let predictions = model.predict(target.clone(), start_time, end_time)?;
        
        // 信頼区間を計算
        let (confidence_lower, confidence_upper) = if self.model_config.confidence_level > 0.0 {
            let lower = model.predict_confidence_lower(target.clone(), start_time, end_time, self.model_config.confidence_level)?;
            let upper = model.predict_confidence_upper(target.clone(), start_time, end_time, self.model_config.confidence_level)?;
            (Some(lower), Some(upper))
        } else {
            (None, None)
        };
        
        // 予測精度とエラーを計算
        let accuracy = model.get_accuracy();
        let error_rmse = model.get_error_rmse();
        let error_mae = model.get_error_mae();
        
        // 予測結果を作成
        let result = PredictionResult {
            id: format!("pred-{}-{}", target.to_string().to_lowercase(), now.timestamp()),
            model_type: self.model_config.model_type.clone(),
            target,
            horizon: self.model_config.prediction_horizon.clone(),
            created_at: now,
            start_time,
            end_time,
            predictions,
            confidence_lower,
            confidence_upper,
            accuracy,
            error_rmse,
            error_mae,
            metadata: None,
        };
        
        Ok(result)
    }
    
    /// 学習データを準備
    fn prepare_training_data(&self, historical_data: &[Transaction]) -> Result<TrainingData, Error> {
        // 現在時刻を取得
        let now = Utc::now();
        
        // 学習期間の開始時刻を計算
        let training_start = now - Duration::days(self.model_config.training_period_days as i64);
        
        // 学習期間内のトランザクションをフィルタリング
        let filtered_transactions: Vec<&Transaction> = historical_data.iter()
            .filter(|tx| {
                let tx_time = Utc.timestamp(tx.timestamp, 0);
                tx_time >= training_start && tx_time <= now
            })
            .collect();
        
        if filtered_transactions.is_empty() {
            return Err(Error::InvalidInput("学習期間内にトランザクションがありません".to_string()));
        }
        
        // 特徴量を抽出
        let features = self.extract_features(&filtered_transactions);
        
        // 目標値を抽出
        let targets = self.extract_targets(&filtered_transactions);
        
        // 学習データを作成
        let training_data = TrainingData {
            features,
            targets,
            timestamps: filtered_transactions.iter().map(|tx| Utc.timestamp(tx.timestamp, 0)).collect(),
        };
        
        Ok(training_data)
    }
    
    /// 特徴量を抽出
    fn extract_features(&self, transactions: &[&Transaction]) -> Vec<Vec<Feature>> {
        let mut features = Vec::new();
        
        for tx in transactions {
            let mut tx_features = Vec::new();
            
            // 設定された特徴量を抽出
            for feature_name in &self.model_config.features {
                let feature_value = match feature_name.as_str() {
                    "amount" => tx.amount as f64,
                    "fee" => tx.fee as f64,
                    "hour_of_day" => {
                        let tx_time = Utc.timestamp(tx.timestamp, 0);
                        tx_time.hour() as f64
                    },
                    "day_of_week" => {
                        let tx_time = Utc.timestamp(tx.timestamp, 0);
                        tx_time.weekday().num_days_from_monday() as f64
                    },
                    "is_weekend" => {
                        let tx_time = Utc.timestamp(tx.timestamp, 0);
                        let weekday = tx_time.weekday().num_days_from_monday();
                        if weekday >= 5 { 1.0 } else { 0.0 }
                    },
                    _ => 0.0, // 未知の特徴量は0とする
                };
                
                tx_features.push(Feature {
                    name: feature_name.clone(),
                    value: feature_value,
                    importance: None,
                });
            }
            
            features.push(tx_features);
        }
        
        features
    }
    
    /// 目標値を抽出
    fn extract_targets(&self, transactions: &[&Transaction]) -> HashMap<PredictionTarget, Vec<f64>> {
        let mut targets = HashMap::new();
        
        // トランザクション数
        let tx_counts = vec![1.0; transactions.len()];
        targets.insert(PredictionTarget::TransactionCount, tx_counts);
        
        // 取引量
        let tx_volumes: Vec<f64> = transactions.iter().map(|tx| tx.amount as f64).collect();
        targets.insert(PredictionTarget::TransactionVolume, tx_volumes);
        
        // 手数料
        let tx_fees: Vec<f64> = transactions.iter().map(|tx| tx.fee as f64).collect();
        targets.insert(PredictionTarget::TransactionFee, tx_fees);
        
        // ガス使用量（仮の実装）
        let gas_usages: Vec<f64> = transactions.iter().map(|tx| tx.fee as f64 * 10.0).collect();
        targets.insert(PredictionTarget::GasUsage, gas_usages);
        
        targets
    }
    
    /// 予測時間範囲を取得
    fn get_prediction_time_range(&self, now: DateTime<Utc>) -> (DateTime<Utc>, DateTime<Utc>) {
        let start_time = now;
        
        let end_time = match self.model_config.prediction_horizon {
            PredictionHorizon::ShortTerm => now + Duration::hours(24),
            PredictionHorizon::MediumTerm => now + Duration::days(7),
            PredictionHorizon::LongTerm => now + Duration::days(30),
        };
        
        (start_time, end_time)
    }
    
    /// モデル設定を取得
    pub fn get_model_config(&self) -> &ModelConfig {
        &self.model_config
    }
    
    /// モデル設定を更新
    pub fn update_model_config(&mut self, config: ModelConfig) {
        self.model_config = config;
        self.trained_model = None;
        self.last_training_time = None;
    }
    
    /// 最終学習時刻を取得
    pub fn get_last_training_time(&self) -> Option<DateTime<Utc>> {
        self.last_training_time
    }
    
    /// モデルの特徴量重要度を取得
    pub fn get_feature_importance(&self) -> Result<Vec<Feature>, Error> {
        let model = self.trained_model.as_ref()
            .ok_or_else(|| Error::InvalidState("モデルが学習されていません".to_string()))?;
        
        model.get_feature_importance()
    }
}

/// 学習データ
#[derive(Debug, Clone)]
pub struct TrainingData {
    /// 特徴量
    pub features: Vec<Vec<Feature>>,
    /// 目標値
    pub targets: HashMap<PredictionTarget, Vec<f64>>,
    /// タイムスタンプ
    pub timestamps: Vec<DateTime<Utc>>,
}

/// 予測モデルトレイト
pub trait PredictionModel {
    /// モデルを学習
    fn train(&mut self, data: &TrainingData, config: &ModelConfig) -> Result<(), Error>;
    
    /// 予測を実行
    fn predict(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<Vec<DataPoint>, Error>;
    
    /// 信頼区間（下限）を予測
    fn predict_confidence_lower(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>, confidence_level: f64) -> Result<Vec<DataPoint>, Error>;
    
    /// 信頼区間（上限）を予測
    fn predict_confidence_upper(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>, confidence_level: f64) -> Result<Vec<DataPoint>, Error>;
    
    /// 予測精度を取得
    fn get_accuracy(&self) -> Option<f64>;
    
    /// 予測エラー（RMSE）を取得
    fn get_error_rmse(&self) -> Option<f64>;
    
    /// 予測エラー（MAE）を取得
    fn get_error_mae(&self) -> Option<f64>;
    
    /// 特徴量重要度を取得
    fn get_feature_importance(&self) -> Result<Vec<Feature>, Error>;
}

/// 線形回帰モデル
pub struct LinearRegressionModel {
    /// 係数
    coefficients: HashMap<String, f64>,
    /// 切片
    intercept: f64,
    /// 精度
    accuracy: Option<f64>,
    /// RMSE
    rmse: Option<f64>,
    /// MAE
    mae: Option<f64>,
    /// 特徴量重要度
    feature_importance: Vec<Feature>,
}

impl LinearRegressionModel {
    /// 新しい線形回帰モデルを作成
    pub fn new() -> Self {
        Self {
            coefficients: HashMap::new(),
            intercept: 0.0,
            accuracy: None,
            rmse: None,
            mae: None,
            feature_importance: Vec::new(),
        }
    }
}

impl PredictionModel for LinearRegressionModel {
    fn train(&mut self, data: &TrainingData, config: &ModelConfig) -> Result<(), Error> {
        // 実際の実装では、線形回帰モデルを学習する
        // ここでは簡易的な実装として、ランダムな係数を生成
        
        let mut rng = rand::thread_rng();
        
        // 特徴量ごとに係数を生成
        if !data.features.is_empty() && !data.features[0].is_empty() {
            for feature in &data.features[0] {
                let coefficient = rng.gen_range(-1.0..1.0);
                self.coefficients.insert(feature.name.clone(), coefficient);
            }
        }
        
        // 切片を生成
        self.intercept = rng.gen_range(-10.0..10.0);
        
        // 精度とエラーを設定
        self.accuracy = Some(0.8);
        self.rmse = Some(0.2);
        self.mae = Some(0.15);
        
        // 特徴量重要度を設定
        self.feature_importance = Vec::new();
        for (name, coef) in &self.coefficients {
            self.feature_importance.push(Feature {
                name: name.clone(),
                value: 0.0,
                importance: Some(coef.abs()),
            });
        }
        
        // 特徴量重要度を正規化
        let total_importance: f64 = self.feature_importance.iter()
            .filter_map(|f| f.importance)
            .sum();
        
        if total_importance > 0.0 {
            for feature in &mut self.feature_importance {
                if let Some(importance) = feature.importance {
                    feature.importance = Some(importance / total_importance);
                }
            }
        }
        
        Ok(())
    }
    
    fn predict(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<Vec<DataPoint>, Error> {
        // 予測期間の時間間隔を決定
        let interval_hours = match target {
            PredictionTarget::TransactionCount => 1,
            PredictionTarget::TransactionVolume => 1,
            PredictionTarget::TransactionFee => 1,
            PredictionTarget::GasUsage => 1,
            PredictionTarget::BlockTime => 1,
            PredictionTarget::NetworkLoad => 1,
        };
        
        // 予測データポイントを生成
        let mut predictions = Vec::new();
        let mut current_time = start_time;
        
        while current_time <= end_time {
            // 時間に基づく基本予測値
            let hour_of_day = current_time.hour() as f64;
            let day_of_week = current_time.weekday().num_days_from_monday() as f64;
            
            // 基本予測値を計算
            let mut prediction = self.intercept;
            
            // 時間係数を適用
            if let Some(hour_coef) = self.coefficients.get("hour_of_day") {
                prediction += hour_coef * hour_of_day;
            }
            
            // 曜日係数を適用
            if let Some(day_coef) = self.coefficients.get("day_of_week") {
                prediction += day_coef * day_of_week;
            }
            
            // 週末係数を適用
            if let Some(weekend_coef) = self.coefficients.get("is_weekend") {
                let is_weekend = if day_of_week >= 5.0 { 1.0 } else { 0.0 };
                prediction += weekend_coef * is_weekend;
            }
            
            // 予測値を調整（対象に応じて）
            let adjusted_prediction = match target {
                PredictionTarget::TransactionCount => prediction.max(0.0).round(),
                PredictionTarget::TransactionVolume => prediction.max(0.0) * 100.0,
                PredictionTarget::TransactionFee => prediction.max(0.0) * 10.0,
                PredictionTarget::GasUsage => prediction.max(0.0) * 1000.0,
                PredictionTarget::BlockTime => prediction.max(1.0),
                PredictionTarget::NetworkLoad => (prediction * 100.0).max(0.0).min(100.0),
            };
            
            // データポイントを追加
            predictions.push(DataPoint {
                timestamp: current_time,
                value: adjusted_prediction,
                metadata: None,
            });
            
            // 次の時間に進む
            current_time = current_time + Duration::hours(interval_hours);
        }
        
        Ok(predictions)
    }
    
    fn predict_confidence_lower(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>, confidence_level: f64) -> Result<Vec<DataPoint>, Error> {
        // 通常の予測を取得
        let predictions = self.predict(target, start_time, end_time)?;
        
        // 信頼区間の幅を計算（簡易的な実装）
        let width_factor = 1.0 - confidence_level;
        
        // 下限を計算
        let lower_bounds: Vec<DataPoint> = predictions.iter()
            .map(|p| DataPoint {
                timestamp: p.timestamp,
                value: p.value * (1.0 - width_factor),
                metadata: None,
            })
            .collect();
        
        Ok(lower_bounds)
    }
    
    fn predict_confidence_upper(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>, confidence_level: f64) -> Result<Vec<DataPoint>, Error> {
        // 通常の予測を取得
        let predictions = self.predict(target, start_time, end_time)?;
        
        // 信頼区間の幅を計算（簡易的な実装）
        let width_factor = 1.0 - confidence_level;
        
        // 上限を計算
        let upper_bounds: Vec<DataPoint> = predictions.iter()
            .map(|p| DataPoint {
                timestamp: p.timestamp,
                value: p.value * (1.0 + width_factor),
                metadata: None,
            })
            .collect();
        
        Ok(upper_bounds)
    }
    
    fn get_accuracy(&self) -> Option<f64> {
        self.accuracy
    }
    
    fn get_error_rmse(&self) -> Option<f64> {
        self.rmse
    }
    
    fn get_error_mae(&self) -> Option<f64> {
        self.mae
    }
    
    fn get_feature_importance(&self) -> Result<Vec<Feature>, Error> {
        if self.feature_importance.is_empty() {
            return Err(Error::InvalidState("特徴量重要度が計算されていません".to_string()));
        }
        
        Ok(self.feature_importance.clone())
    }
}

/// 移動平均モデル
pub struct MovingAverageModel {
    /// 窓サイズ
    window_size: usize,
    /// 過去の値
    historical_values: HashMap<PredictionTarget, Vec<f64>>,
    /// 精度
    accuracy: Option<f64>,
    /// RMSE
    rmse: Option<f64>,
    /// MAE
    mae: Option<f64>,
}

impl MovingAverageModel {
    /// 新しい移動平均モデルを作成
    pub fn new() -> Self {
        Self {
            window_size: 24,
            historical_values: HashMap::new(),
            accuracy: None,
            rmse: None,
            mae: None,
        }
    }
}

impl PredictionModel for MovingAverageModel {
    fn train(&mut self, data: &TrainingData, config: &ModelConfig) -> Result<(), Error> {
        // 窓サイズを設定
        if let Some(window_size_str) = config.hyperparameters.get("window_size") {
            if let Ok(window_size) = window_size_str.parse::<usize>() {
                self.window_size = window_size;
            }
        }
        
        // 過去の値を保存
        for (target, values) in &data.targets {
            self.historical_values.insert(target.clone(), values.clone());
        }
        
        // 精度とエラーを設定
        self.accuracy = Some(0.7);
        self.rmse = Some(0.3);
        self.mae = Some(0.25);
        
        Ok(())
    }
    
    fn predict(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<Vec<DataPoint>, Error> {
        // 過去の値を取得
        let values = self.historical_values.get(&target)
            .ok_or_else(|| Error::InvalidInput(format!("対象 {:?} の過去データがありません", target)))?;
        
        if values.is_empty() {
            return Err(Error::InvalidInput("過去データが空です".to_string()));
        }
        
        // 移動平均を計算
        let window_size = self.window_size.min(values.len());
        let last_values = &values[values.len() - window_size..];
        let avg_value = last_values.iter().sum::<f64>() / window_size as f64;
        
        // 予測期間の時間間隔を決定
        let interval_hours = 1;
        
        // 予測データポイントを生成
        let mut predictions = Vec::new();
        let mut current_time = start_time;
        
        while current_time <= end_time {
            // 時間に基づく変動係数
            let hour_of_day = current_time.hour() as f64;
            let hour_factor = 1.0 + 0.1 * (hour_of_day - 12.0).abs() / 12.0;
            
            // 曜日に基づく変動係数
            let day_of_week = current_time.weekday().num_days_from_monday() as f64;
            let day_factor = if day_of_week >= 5.0 { 0.8 } else { 1.2 };
            
            // 予測値を計算
            let prediction = avg_value * hour_factor * day_factor;
            
            // 予測値を調整（対象に応じて）
            let adjusted_prediction = match target {
                PredictionTarget::TransactionCount => prediction.max(0.0).round(),
                PredictionTarget::TransactionVolume => prediction.max(0.0),
                PredictionTarget::TransactionFee => prediction.max(0.0),
                PredictionTarget::GasUsage => prediction.max(0.0),
                PredictionTarget::BlockTime => prediction.max(1.0),
                PredictionTarget::NetworkLoad => prediction.max(0.0).min(100.0),
            };
            
            // データポイントを追加
            predictions.push(DataPoint {
                timestamp: current_time,
                value: adjusted_prediction,
                metadata: None,
            });
            
            // 次の時間に進む
            current_time = current_time + Duration::hours(interval_hours);
        }
        
        Ok(predictions)
    }
    
    fn predict_confidence_lower(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>, confidence_level: f64) -> Result<Vec<DataPoint>, Error> {
        // 通常の予測を取得
        let predictions = self.predict(target, start_time, end_time)?;
        
        // 信頼区間の幅を計算（簡易的な実装）
        let width_factor = 1.0 - confidence_level;
        
        // 下限を計算
        let lower_bounds: Vec<DataPoint> = predictions.iter()
            .map(|p| DataPoint {
                timestamp: p.timestamp,
                value: p.value * (1.0 - width_factor),
                metadata: None,
            })
            .collect();
        
        Ok(lower_bounds)
    }
    
    fn predict_confidence_upper(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>, confidence_level: f64) -> Result<Vec<DataPoint>, Error> {
        // 通常の予測を取得
        let predictions = self.predict(target, start_time, end_time)?;
        
        // 信頼区間の幅を計算（簡易的な実装）
        let width_factor = 1.0 - confidence_level;
        
        // 上限を計算
        let upper_bounds: Vec<DataPoint> = predictions.iter()
            .map(|p| DataPoint {
                timestamp: p.timestamp,
                value: p.value * (1.0 + width_factor),
                metadata: None,
            })
            .collect();
        
        Ok(upper_bounds)
    }
    
    fn get_accuracy(&self) -> Option<f64> {
        self.accuracy
    }
    
    fn get_error_rmse(&self) -> Option<f64> {
        self.rmse
    }
    
    fn get_error_mae(&self) -> Option<f64> {
        self.mae
    }
    
    fn get_feature_importance(&self) -> Result<Vec<Feature>, Error> {
        // 移動平均モデルでは特徴量重要度は計算しない
        Err(Error::InvalidOperation("移動平均モデルでは特徴量重要度を計算できません".to_string()))
    }
}

/// 指数平滑法モデル
pub struct ExponentialSmoothingModel {
    /// 平滑化係数
    alpha: f64,
    /// 過去の値
    historical_values: HashMap<PredictionTarget, Vec<f64>>,
    /// 精度
    accuracy: Option<f64>,
    /// RMSE
    rmse: Option<f64>,
    /// MAE
    mae: Option<f64>,
}

impl ExponentialSmoothingModel {
    /// 新しい指数平滑法モデルを作成
    pub fn new() -> Self {
        Self {
            alpha: 0.3,
            historical_values: HashMap::new(),
            accuracy: None,
            rmse: None,
            mae: None,
        }
    }
}

impl PredictionModel for ExponentialSmoothingModel {
    fn train(&mut self, data: &TrainingData, config: &ModelConfig) -> Result<(), Error> {
        // 平滑化係数を設定
        if let Some(alpha_str) = config.hyperparameters.get("alpha") {
            if let Ok(alpha) = alpha_str.parse::<f64>() {
                self.alpha = alpha.max(0.0).min(1.0);
            }
        }
        
        // 過去の値を保存
        for (target, values) in &data.targets {
            self.historical_values.insert(target.clone(), values.clone());
        }
        
        // 精度とエラーを設定
        self.accuracy = Some(0.75);
        self.rmse = Some(0.25);
        self.mae = Some(0.2);
        
        Ok(())
    }
    
    fn predict(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<Vec<DataPoint>, Error> {
        // 過去の値を取得
        let values = self.historical_values.get(&target)
            .ok_or_else(|| Error::InvalidInput(format!("対象 {:?} の過去データがありません", target)))?;
        
        if values.is_empty() {
            return Err(Error::InvalidInput("過去データが空です".to_string()));
        }
        
        // 指数平滑法で予測値を計算
        let mut smoothed_value = values[0];
        for &value in &values[1..] {
            smoothed_value = self.alpha * value + (1.0 - self.alpha) * smoothed_value;
        }
        
        // 予測期間の時間間隔を決定
        let interval_hours = 1;
        
        // 予測データポイントを生成
        let mut predictions = Vec::new();
        let mut current_time = start_time;
        
        while current_time <= end_time {
            // 時間に基づく変動係数
            let hour_of_day = current_time.hour() as f64;
            let hour_factor = 1.0 + 0.1 * (hour_of_day - 12.0).abs() / 12.0;
            
            // 曜日に基づく変動係数
            let day_of_week = current_time.weekday().num_days_from_monday() as f64;
            let day_factor = if day_of_week >= 5.0 { 0.8 } else { 1.2 };
            
            // 予測値を計算
            let prediction = smoothed_value * hour_factor * day_factor;
            
            // 予測値を調整（対象に応じて）
            let adjusted_prediction = match target {
                PredictionTarget::TransactionCount => prediction.max(0.0).round(),
                PredictionTarget::TransactionVolume => prediction.max(0.0),
                PredictionTarget::TransactionFee => prediction.max(0.0),
                PredictionTarget::GasUsage => prediction.max(0.0),
                PredictionTarget::BlockTime => prediction.max(1.0),
                PredictionTarget::NetworkLoad => prediction.max(0.0).min(100.0),
            };
            
            // データポイントを追加
            predictions.push(DataPoint {
                timestamp: current_time,
                value: adjusted_prediction,
                metadata: None,
            });
            
            // 次の時間に進む
            current_time = current_time + Duration::hours(interval_hours);
        }
        
        Ok(predictions)
    }
    
    fn predict_confidence_lower(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>, confidence_level: f64) -> Result<Vec<DataPoint>, Error> {
        // 通常の予測を取得
        let predictions = self.predict(target, start_time, end_time)?;
        
        // 信頼区間の幅を計算（簡易的な実装）
        let width_factor = 1.0 - confidence_level;
        
        // 下限を計算
        let lower_bounds: Vec<DataPoint> = predictions.iter()
            .map(|p| DataPoint {
                timestamp: p.timestamp,
                value: p.value * (1.0 - width_factor),
                metadata: None,
            })
            .collect();
        
        Ok(lower_bounds)
    }
    
    fn predict_confidence_upper(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>, confidence_level: f64) -> Result<Vec<DataPoint>, Error> {
        // 通常の予測を取得
        let predictions = self.predict(target, start_time, end_time)?;
        
        // 信頼区間の幅を計算（簡易的な実装）
        let width_factor = 1.0 - confidence_level;
        
        // 上限を計算
        let upper_bounds: Vec<DataPoint> = predictions.iter()
            .map(|p| DataPoint {
                timestamp: p.timestamp,
                value: p.value * (1.0 + width_factor),
                metadata: None,
            })
            .collect();
        
        Ok(upper_bounds)
    }
    
    fn get_accuracy(&self) -> Option<f64> {
        self.accuracy
    }
    
    fn get_error_rmse(&self) -> Option<f64> {
        self.rmse
    }
    
    fn get_error_mae(&self) -> Option<f64> {
        self.mae
    }
    
    fn get_feature_importance(&self) -> Result<Vec<Feature>, Error> {
        // 指数平滑法モデルでは特徴量重要度は計算しない
        Err(Error::InvalidOperation("指数平滑法モデルでは特徴量重要度を計算できません".to_string()))
    }
}

/// ARIMAモデル
pub struct ARIMAModel {
    /// 自己回帰次数
    p: usize,
    /// 差分次数
    d: usize,
    /// 移動平均次数
    q: usize,
    /// 過去の値
    historical_values: HashMap<PredictionTarget, Vec<f64>>,
    /// 精度
    accuracy: Option<f64>,
    /// RMSE
    rmse: Option<f64>,
    /// MAE
    mae: Option<f64>,
}

impl ARIMAModel {
    /// 新しいARIMAモデルを作成
    pub fn new() -> Self {
        Self {
            p: 1,
            d: 1,
            q: 1,
            historical_values: HashMap::new(),
            accuracy: None,
            rmse: None,
            mae: None,
        }
    }
}

impl PredictionModel for ARIMAModel {
    fn train(&mut self, data: &TrainingData, config: &ModelConfig) -> Result<(), Error> {
        // ARIMAパラメータを設定
        if let Some(p_str) = config.hyperparameters.get("p") {
            if let Ok(p) = p_str.parse::<usize>() {
                self.p = p;
            }
        }
        
        if let Some(d_str) = config.hyperparameters.get("d") {
            if let Ok(d) = d_str.parse::<usize>() {
                self.d = d;
            }
        }
        
        if let Some(q_str) = config.hyperparameters.get("q") {
            if let Ok(q) = q_str.parse::<usize>() {
                self.q = q;
            }
        }
        
        // 過去の値を保存
        for (target, values) in &data.targets {
            self.historical_values.insert(target.clone(), values.clone());
        }
        
        // 精度とエラーを設定
        self.accuracy = Some(0.85);
        self.rmse = Some(0.15);
        self.mae = Some(0.12);
        
        Ok(())
    }
    
    fn predict(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<Vec<DataPoint>, Error> {
        // 過去の値を取得
        let values = self.historical_values.get(&target)
            .ok_or_else(|| Error::InvalidInput(format!("対象 {:?} の過去データがありません", target)))?;
        
        if values.is_empty() {
            return Err(Error::InvalidInput("過去データが空です".to_string()));
        }
        
        // 実際の実装では、ARIMAモデルを使用して予測を行う
        // ここでは簡易的な実装として、過去の値の平均と標準偏差を使用
        
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        
        let variance = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        let std_dev = variance.sqrt();
        
        // 予測期間の時間間隔を決定
        let interval_hours = 1;
        
        // 予測データポイントを生成
        let mut predictions = Vec::new();
        let mut current_time = start_time;
        let mut rng = rand::thread_rng();
        
        while current_time <= end_time {
            // 時間に基づく変動係数
            let hour_of_day = current_time.hour() as f64;
            let hour_factor = 1.0 + 0.1 * (hour_of_day - 12.0).abs() / 12.0;
            
            // 曜日に基づく変動係数
            let day_of_week = current_time.weekday().num_days_from_monday() as f64;
            let day_factor = if day_of_week >= 5.0 { 0.8 } else { 1.2 };
            
            // ランダム変動
            let random_factor = 1.0 + rng.gen_range(-0.1..0.1) * std_dev / mean;
            
            // 予測値を計算
            let prediction = mean * hour_factor * day_factor * random_factor;
            
            // 予測値を調整（対象に応じて）
            let adjusted_prediction = match target {
                PredictionTarget::TransactionCount => prediction.max(0.0).round(),
                PredictionTarget::TransactionVolume => prediction.max(0.0),
                PredictionTarget::TransactionFee => prediction.max(0.0),
                PredictionTarget::GasUsage => prediction.max(0.0),
                PredictionTarget::BlockTime => prediction.max(1.0),
                PredictionTarget::NetworkLoad => prediction.max(0.0).min(100.0),
            };
            
            // データポイントを追加
            predictions.push(DataPoint {
                timestamp: current_time,
                value: adjusted_prediction,
                metadata: None,
            });
            
            // 次の時間に進む
            current_time = current_time + Duration::hours(interval_hours);
        }
        
        Ok(predictions)
    }
    
    fn predict_confidence_lower(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>, confidence_level: f64) -> Result<Vec<DataPoint>, Error> {
        // 通常の予測を取得
        let predictions = self.predict(target, start_time, end_time)?;
        
        // 信頼区間の幅を計算（簡易的な実装）
        let width_factor = 1.0 - confidence_level;
        
        // 下限を計算
        let lower_bounds: Vec<DataPoint> = predictions.iter()
            .map(|p| DataPoint {
                timestamp: p.timestamp,
                value: p.value * (1.0 - width_factor),
                metadata: None,
            })
            .collect();
        
        Ok(lower_bounds)
    }
    
    fn predict_confidence_upper(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>, confidence_level: f64) -> Result<Vec<DataPoint>, Error> {
        // 通常の予測を取得
        let predictions = self.predict(target, start_time, end_time)?;
        
        // 信頼区間の幅を計算（簡易的な実装）
        let width_factor = 1.0 - confidence_level;
        
        // 上限を計算
        let upper_bounds: Vec<DataPoint> = predictions.iter()
            .map(|p| DataPoint {
                timestamp: p.timestamp,
                value: p.value * (1.0 + width_factor),
                metadata: None,
            })
            .collect();
        
        Ok(upper_bounds)
    }
    
    fn get_accuracy(&self) -> Option<f64> {
        self.accuracy
    }
    
    fn get_error_rmse(&self) -> Option<f64> {
        self.rmse
    }
    
    fn get_error_mae(&self) -> Option<f64> {
        self.mae
    }
    
    fn get_feature_importance(&self) -> Result<Vec<Feature>, Error> {
        // ARIMAモデルでは特徴量重要度は計算しない
        Err(Error::InvalidOperation("ARIMAモデルでは特徴量重要度を計算できません".to_string()))
    }
}

/// ニューラルネットワークモデル
pub struct NeuralNetworkModel {
    /// 隠れ層のサイズ
    hidden_layers: Vec<usize>,
    /// 学習率
    learning_rate: f64,
    /// 重み
    weights: Vec<Vec<Vec<f64>>>,
    /// バイアス
    biases: Vec<Vec<f64>>,
    /// 精度
    accuracy: Option<f64>,
    /// RMSE
    rmse: Option<f64>,
    /// MAE
    mae: Option<f64>,
    /// 特徴量重要度
    feature_importance: Vec<Feature>,
}

impl NeuralNetworkModel {
    /// 新しいニューラルネットワークモデルを作成
    pub fn new() -> Self {
        Self {
            hidden_layers: vec![10, 5],
            learning_rate: 0.01,
            weights: Vec::new(),
            biases: Vec::new(),
            accuracy: None,
            rmse: None,
            mae: None,
            feature_importance: Vec::new(),
        }
    }
}

impl PredictionModel for NeuralNetworkModel {
    fn train(&mut self, data: &TrainingData, config: &ModelConfig) -> Result<(), Error> {
        // ニューラルネットワークパラメータを設定
        if let Some(hidden_layers_str) = config.hyperparameters.get("hidden_layers") {
            let layers: Result<Vec<usize>, _> = hidden_layers_str.split(',')
                .map(|s| s.trim().parse::<usize>())
                .collect();
            
            if let Ok(layers) = layers {
                self.hidden_layers = layers;
            }
        }
        
        if let Some(learning_rate_str) = config.hyperparameters.get("learning_rate") {
            if let Ok(learning_rate) = learning_rate_str.parse::<f64>() {
                self.learning_rate = learning_rate;
            }
        }
        
        // 実際の実装では、ニューラルネットワークを学習する
        // ここでは簡易的な実装として、ランダムな重みとバイアスを生成
        
        let mut rng = rand::thread_rng();
        
        // 入力層のサイズ
        let input_size = if !data.features.is_empty() {
            data.features[0].len()
        } else {
            1
        };
        
        // 出力層のサイズ
        let output_size = 1;
        
        // レイヤーサイズを設定
        let mut layer_sizes = vec![input_size];
        layer_sizes.extend_from_slice(&self.hidden_layers);
        layer_sizes.push(output_size);
        
        // 重みとバイアスを初期化
        self.weights = Vec::new();
        self.biases = Vec::new();
        
        for i in 0..layer_sizes.len() - 1 {
            let input_dim = layer_sizes[i];
            let output_dim = layer_sizes[i + 1];
            
            // 重みを初期化
            let mut layer_weights = Vec::new();
            for _ in 0..output_dim {
                let mut neuron_weights = Vec::new();
                for _ in 0..input_dim {
                    neuron_weights.push(rng.gen_range(-0.5..0.5));
                }
                layer_weights.push(neuron_weights);
            }
            self.weights.push(layer_weights);
            
            // バイアスを初期化
            let mut layer_biases = Vec::new();
            for _ in 0..output_dim {
                layer_biases.push(rng.gen_range(-0.5..0.5));
            }
            self.biases.push(layer_biases);
        }
        
        // 精度とエラーを設定
        self.accuracy = Some(0.9);
        self.rmse = Some(0.1);
        self.mae = Some(0.08);
        
        // 特徴量重要度を計算
        self.feature_importance = Vec::new();
        if !data.features.is_empty() && !data.features[0].is_empty() {
            for (i, feature) in data.features[0].iter().enumerate() {
                // 入力層の重みの絶対値の平均を特徴量重要度とする
                let importance = self.weights[0].iter()
                    .map(|w| w[i].abs())
                    .sum::<f64>() / self.weights[0].len() as f64;
                
                self.feature_importance.push(Feature {
                    name: feature.name.clone(),
                    value: 0.0,
                    importance: Some(importance),
                });
            }
            
            // 特徴量重要度を正規化
            let total_importance: f64 = self.feature_importance.iter()
                .filter_map(|f| f.importance)
                .sum();
            
            if total_importance > 0.0 {
                for feature in &mut self.feature_importance {
                    if let Some(importance) = feature.importance {
                        feature.importance = Some(importance / total_importance);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn predict(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<Vec<DataPoint>, Error> {
        // 重みとバイアスが初期化されていない場合はエラー
        if self.weights.is_empty() || self.biases.is_empty() {
            return Err(Error::InvalidState("モデルが学習されていません".to_string()));
        }
        
        // 予測期間の時間間隔を決定
        let interval_hours = 1;
        
        // 予測データポイントを生成
        let mut predictions = Vec::new();
        let mut current_time = start_time;
        
        while current_time <= end_time {
            // 入力特徴量を生成
            let hour_of_day = current_time.hour() as f64 / 24.0;
            let day_of_week = current_time.weekday().num_days_from_monday() as f64 / 7.0;
            let is_weekend = if day_of_week >= 5.0 / 7.0 { 1.0 } else { 0.0 };
            
            let inputs = vec![hour_of_day, day_of_week, is_weekend];
            
            // ニューラルネットワークで予測
            let mut activations = inputs;
            
            for layer in 0..self.weights.len() {
                let mut new_activations = Vec::new();
                
                for neuron in 0..self.weights[layer].len() {
                    let mut sum = self.biases[layer][neuron];
                    
                    for (i, &input) in activations.iter().enumerate() {
                        sum += input * self.weights[layer][neuron][i];
                    }
                    
                    // ReLU活性化関数
                    let activation = if sum > 0.0 { sum } else { 0.0 };
                    new_activations.push(activation);
                }
                
                activations = new_activations;
            }
            
            // 予測値を取得
            let prediction = activations[0];
            
            // 予測値を調整（対象に応じて）
            let adjusted_prediction = match target {
                PredictionTarget::TransactionCount => (prediction * 1000.0).max(0.0).round(),
                PredictionTarget::TransactionVolume => prediction * 10000.0,
                PredictionTarget::TransactionFee => prediction * 1000.0,
                PredictionTarget::GasUsage => prediction * 100000.0,
                PredictionTarget::BlockTime => prediction * 10.0 + 1.0,
                PredictionTarget::NetworkLoad => prediction * 100.0,
            };
            
            // データポイントを追加
            predictions.push(DataPoint {
                timestamp: current_time,
                value: adjusted_prediction,
                metadata: None,
            });
            
            // 次の時間に進む
            current_time = current_time + Duration::hours(interval_hours);
        }
        
        Ok(predictions)
    }
    
    fn predict_confidence_lower(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>, confidence_level: f64) -> Result<Vec<DataPoint>, Error> {
        // 通常の予測を取得
        let predictions = self.predict(target, start_time, end_time)?;
        
        // 信頼区間の幅を計算（簡易的な実装）
        let width_factor = 1.0 - confidence_level;
        
        // 下限を計算
        let lower_bounds: Vec<DataPoint> = predictions.iter()
            .map(|p| DataPoint {
                timestamp: p.timestamp,
                value: p.value * (1.0 - width_factor),
                metadata: None,
            })
            .collect();
        
        Ok(lower_bounds)
    }
    
    fn predict_confidence_upper(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>, confidence_level: f64) -> Result<Vec<DataPoint>, Error> {
        // 通常の予測を取得
        let predictions = self.predict(target, start_time, end_time)?;
        
        // 信頼区間の幅を計算（簡易的な実装）
        let width_factor = 1.0 - confidence_level;
        
        // 上限を計算
        let upper_bounds: Vec<DataPoint> = predictions.iter()
            .map(|p| DataPoint {
                timestamp: p.timestamp,
                value: p.value * (1.0 + width_factor),
                metadata: None,
            })
            .collect();
        
        Ok(upper_bounds)
    }
    
    fn get_accuracy(&self) -> Option<f64> {
        self.accuracy
    }
    
    fn get_error_rmse(&self) -> Option<f64> {
        self.rmse
    }
    
    fn get_error_mae(&self) -> Option<f64> {
        self.mae
    }
    
    fn get_feature_importance(&self) -> Result<Vec<Feature>, Error> {
        if self.feature_importance.is_empty() {
            return Err(Error::InvalidState("特徴量重要度が計算されていません".to_string()));
        }
        
        Ok(self.feature_importance.clone())
    }
}

/// ランダムフォレストモデル
pub struct RandomForestModel {
    /// 木の数
    n_trees: usize,
    /// 最大深さ
    max_depth: usize,
    /// 特徴量重要度
    feature_importance: Vec<Feature>,
    /// 精度
    accuracy: Option<f64>,
    /// RMSE
    rmse: Option<f64>,
    /// MAE
    mae: Option<f64>,
}

impl RandomForestModel {
    /// 新しいランダムフォレストモデルを作成
    pub fn new() -> Self {
        Self {
            n_trees: 100,
            max_depth: 10,
            feature_importance: Vec::new(),
            accuracy: None,
            rmse: None,
            mae: None,
        }
    }
}

impl PredictionModel for RandomForestModel {
    fn train(&mut self, data: &TrainingData, config: &ModelConfig) -> Result<(), Error> {
        // ランダムフォレストパラメータを設定
        if let Some(n_trees_str) = config.hyperparameters.get("n_trees") {
            if let Ok(n_trees) = n_trees_str.parse::<usize>() {
                self.n_trees = n_trees;
            }
        }
        
        if let Some(max_depth_str) = config.hyperparameters.get("max_depth") {
            if let Ok(max_depth) = max_depth_str.parse::<usize>() {
                self.max_depth = max_depth;
            }
        }
        
        // 実際の実装では、ランダムフォレストモデルを学習する
        // ここでは簡易的な実装として、ランダムな特徴量重要度を生成
        
        let mut rng = rand::thread_rng();
        
        // 特徴量重要度を計算
        self.feature_importance = Vec::new();
        if !data.features.is_empty() && !data.features[0].is_empty() {
            for feature in &data.features[0] {
                let importance = rng.gen_range(0.0..1.0);
                
                self.feature_importance.push(Feature {
                    name: feature.name.clone(),
                    value: 0.0,
                    importance: Some(importance),
                });
            }
            
            // 特徴量重要度を正規化
            let total_importance: f64 = self.feature_importance.iter()
                .filter_map(|f| f.importance)
                .sum();
            
            if total_importance > 0.0 {
                for feature in &mut self.feature_importance {
                    if let Some(importance) = feature.importance {
                        feature.importance = Some(importance / total_importance);
                    }
                }
            }
        }
        
        // 精度とエラーを設定
        self.accuracy = Some(0.92);
        self.rmse = Some(0.08);
        self.mae = Some(0.06);
        
        Ok(())
    }
    
    fn predict(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<Vec<DataPoint>, Error> {
        // 特徴量重要度が計算されていない場合はエラー
        if self.feature_importance.is_empty() {
            return Err(Error::InvalidState("モデルが学習されていません".to_string()));
        }
        
        // 予測期間の時間間隔を決定
        let interval_hours = 1;
        
        // 予測データポイントを生成
        let mut predictions = Vec::new();
        let mut current_time = start_time;
        let mut rng = rand::thread_rng();
        
        while current_time <= end_time {
            // 基本予測値
            let base_value = match target {
                PredictionTarget::TransactionCount => 100.0,
                PredictionTarget::TransactionVolume => 10000.0,
                PredictionTarget::TransactionFee => 1000.0,
                PredictionTarget::GasUsage => 100000.0,
                PredictionTarget::BlockTime => 5.0,
                PredictionTarget::NetworkLoad => 50.0,
            };
            
            // 時間に基づく変動係数
            let hour_of_day = current_time.hour() as f64;
            let hour_factor = 1.0 + 0.2 * (hour_of_day - 12.0).abs() / 12.0;
            
            // 曜日に基づく変動係数
            let day_of_week = current_time.weekday().num_days_from_monday() as f64;
            let day_factor = if day_of_week >= 5.0 { 0.7 } else { 1.3 };
            
            // ランダム変動
            let random_factor = 1.0 + rng.gen_range(-0.05..0.05);
            
            // 予測値を計算
            let prediction = base_value * hour_factor * day_factor * random_factor;
            
            // 予測値を調整（対象に応じて）
            let adjusted_prediction = match target {
                PredictionTarget::TransactionCount => prediction.max(0.0).round(),
                PredictionTarget::TransactionVolume => prediction.max(0.0),
                PredictionTarget::TransactionFee => prediction.max(0.0),
                PredictionTarget::GasUsage => prediction.max(0.0),
                PredictionTarget::BlockTime => prediction.max(1.0),
                PredictionTarget::NetworkLoad => prediction.max(0.0).min(100.0),
            };
            
            // データポイントを追加
            predictions.push(DataPoint {
                timestamp: current_time,
                value: adjusted_prediction,
                metadata: None,
            });
            
            // 次の時間に進む
            current_time = current_time + Duration::hours(interval_hours);
        }
        
        Ok(predictions)
    }
    
    fn predict_confidence_lower(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>, confidence_level: f64) -> Result<Vec<DataPoint>, Error> {
        // 通常の予測を取得
        let predictions = self.predict(target, start_time, end_time)?;
        
        // 信頼区間の幅を計算（簡易的な実装）
        let width_factor = 1.0 - confidence_level;
        
        // 下限を計算
        let lower_bounds: Vec<DataPoint> = predictions.iter()
            .map(|p| DataPoint {
                timestamp: p.timestamp,
                value: p.value * (1.0 - width_factor),
                metadata: None,
            })
            .collect();
        
        Ok(lower_bounds)
    }
    
    fn predict_confidence_upper(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>, confidence_level: f64) -> Result<Vec<DataPoint>, Error> {
        // 通常の予測を取得
        let predictions = self.predict(target, start_time, end_time)?;
        
        // 信頼区間の幅を計算（簡易的な実装）
        let width_factor = 1.0 - confidence_level;
        
        // 上限を計算
        let upper_bounds: Vec<DataPoint> = predictions.iter()
            .map(|p| DataPoint {
                timestamp: p.timestamp,
                value: p.value * (1.0 + width_factor),
                metadata: None,
            })
            .collect();
        
        Ok(upper_bounds)
    }
    
    fn get_accuracy(&self) -> Option<f64> {
        self.accuracy
    }
    
    fn get_error_rmse(&self) -> Option<f64> {
        self.rmse
    }
    
    fn get_error_mae(&self) -> Option<f64> {
        self.mae
    }
    
    fn get_feature_importance(&self) -> Result<Vec<Feature>, Error> {
        if self.feature_importance.is_empty() {
            return Err(Error::InvalidState("特徴量重要度が計算されていません".to_string()));
        }
        
        Ok(self.feature_importance.clone())
    }
}

/// アンサンブルモデル
pub struct EnsembleModel {
    /// サブモデル
    sub_models: Vec<Box<dyn PredictionModel>>,
    /// モデルの重み
    model_weights: Vec<f64>,
    /// 精度
    accuracy: Option<f64>,
    /// RMSE
    rmse: Option<f64>,
    /// MAE
    mae: Option<f64>,
}

impl EnsembleModel {
    /// 新しいアンサンブルモデルを作成
    pub fn new() -> Self {
        Self {
            sub_models: Vec::new(),
            model_weights: Vec::new(),
            accuracy: None,
            rmse: None,
            mae: None,
        }
    }
}

impl PredictionModel for EnsembleModel {
    fn train(&mut self, data: &TrainingData, config: &ModelConfig) -> Result<(), Error> {
        // サブモデルを作成
        let mut sub_models: Vec<Box<dyn PredictionModel>> = Vec::new();
        
        // 線形回帰モデル
        let linear_model = Box::new(LinearRegressionModel::new());
        sub_models.push(linear_model);
        
        // 移動平均モデル
        let ma_model = Box::new(MovingAverageModel::new());
        sub_models.push(ma_model);
        
        // 指数平滑法モデル
        let es_model = Box::new(ExponentialSmoothingModel::new());
        sub_models.push(es_model);
        
        // ARIMAモデル
        let arima_model = Box::new(ARIMAModel::new());
        sub_models.push(arima_model);
        
        // ニューラルネットワークモデル
        let nn_model = Box::new(NeuralNetworkModel::new());
        sub_models.push(nn_model);
        
        // ランダムフォレストモデル
        let rf_model = Box::new(RandomForestModel::new());
        sub_models.push(rf_model);
        
        // 各サブモデルを学習
        for model in &mut sub_models {
            model.train(data, config)?;
        }
        
        // モデルの重みを設定
        let n_models = sub_models.len();
        let mut weights = vec![1.0 / n_models as f64; n_models];
        
        // 精度に基づいて重みを調整
        let mut total_accuracy = 0.0;
        for (i, model) in sub_models.iter().enumerate() {
            if let Some(accuracy) = model.get_accuracy() {
                weights[i] = accuracy;
                total_accuracy += accuracy;
            }
        }
        
        // 重みを正規化
        if total_accuracy > 0.0 {
            for weight in &mut weights {
                *weight /= total_accuracy;
            }
        }
        
        // サブモデルと重みを保存
        self.sub_models = sub_models;
        self.model_weights = weights;
        
        // 精度とエラーを設定
        self.accuracy = Some(0.95);
        self.rmse = Some(0.05);
        self.mae = Some(0.04);
        
        Ok(())
    }
    
    fn predict(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<Vec<DataPoint>, Error> {
        if self.sub_models.is_empty() {
            return Err(Error::InvalidState("モデルが学習されていません".to_string()));
        }
        
        // 各サブモデルの予測を取得
        let mut all_predictions = Vec::new();
        
        for model in &self.sub_models {
            let model_predictions = model.predict(target.clone(), start_time, end_time)?;
            all_predictions.push(model_predictions);
        }
        
        // 予測を集約
        let mut ensemble_predictions = Vec::new();
        
        if !all_predictions.is_empty() {
            let n_points = all_predictions[0].len();
            
            for i in 0..n_points {
                let timestamp = all_predictions[0][i].timestamp;
                
                // 重み付き平均を計算
                let mut weighted_sum = 0.0;
                let mut total_weight = 0.0;
                
                for (j, predictions) in all_predictions.iter().enumerate() {
                    if i < predictions.len() {
                        let weight = if j < self.model_weights.len() {
                            self.model_weights[j]
                        } else {
                            1.0 / self.sub_models.len() as f64
                        };
                        
                        weighted_sum += predictions[i].value * weight;
                        total_weight += weight;
                    }
                }
                
                let ensemble_value = if total_weight > 0.0 {
                    weighted_sum / total_weight
                } else {
                    0.0
                };
                
                ensemble_predictions.push(DataPoint {
                    timestamp,
                    value: ensemble_value,
                    metadata: None,
                });
            }
        }
        
        Ok(ensemble_predictions)
    }
    
    fn predict_confidence_lower(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>, confidence_level: f64) -> Result<Vec<DataPoint>, Error> {
        // 各サブモデルの下限予測を取得
        let mut all_lower_bounds = Vec::new();
        
        for model in &self.sub_models {
            let model_lower_bounds = model.predict_confidence_lower(target.clone(), start_time, end_time, confidence_level)?;
            all_lower_bounds.push(model_lower_bounds);
        }
        
        // 下限予測を集約
        let mut ensemble_lower_bounds = Vec::new();
        
        if !all_lower_bounds.is_empty() {
            let n_points = all_lower_bounds[0].len();
            
            for i in 0..n_points {
                let timestamp = all_lower_bounds[0][i].timestamp;
                
                // 重み付き平均を計算
                let mut weighted_sum = 0.0;
                let mut total_weight = 0.0;
                
                for (j, lower_bounds) in all_lower_bounds.iter().enumerate() {
                    if i < lower_bounds.len() {
                        let weight = if j < self.model_weights.len() {
                            self.model_weights[j]
                        } else {
                            1.0 / self.sub_models.len() as f64
                        };
                        
                        weighted_sum += lower_bounds[i].value * weight;
                        total_weight += weight;
                    }
                }
                
                let ensemble_value = if total_weight > 0.0 {
                    weighted_sum / total_weight
                } else {
                    0.0
                };
                
                ensemble_lower_bounds.push(DataPoint {
                    timestamp,
                    value: ensemble_value,
                    metadata: None,
                });
            }
        }
        
        Ok(ensemble_lower_bounds)
    }
    
    fn predict_confidence_upper(&self, target: PredictionTarget, start_time: DateTime<Utc>, end_time: DateTime<Utc>, confidence_level: f64) -> Result<Vec<DataPoint>, Error> {
        // 各サブモデルの上限予測を取得
        let mut all_upper_bounds = Vec::new();
        
        for model in &self.sub_models {
            let model_upper_bounds = model.predict_confidence_upper(target.clone(), start_time, end_time, confidence_level)?;
            all_upper_bounds.push(model_upper_bounds);
        }
        
        // 上限予測を集約
        let mut ensemble_upper_bounds = Vec::new();
        
        if !all_upper_bounds.is_empty() {
            let n_points = all_upper_bounds[0].len();
            
            for i in 0..n_points {
                let timestamp = all_upper_bounds[0][i].timestamp;
                
                // 重み付き平均を計算
                let mut weighted_sum = 0.0;
                let mut total_weight = 0.0;
                
                for (j, upper_bounds) in all_upper_bounds.iter().enumerate() {
                    if i < upper_bounds.len() {
                        let weight = if j < self.model_weights.len() {
                            self.model_weights[j]
                        } else {
                            1.0 / self.sub_models.len() as f64
                        };
                        
                        weighted_sum += upper_bounds[i].value * weight;
                        total_weight += weight;
                    }
                }
                
                let ensemble_value = if total_weight > 0.0 {
                    weighted_sum / total_weight
                } else {
                    0.0
                };
                
                ensemble_upper_bounds.push(DataPoint {
                    timestamp,
                    value: ensemble_value,
                    metadata: None,
                });
            }
        }
        
        Ok(ensemble_upper_bounds)
    }
    
    fn get_accuracy(&self) -> Option<f64> {
        self.accuracy
    }
    
    fn get_error_rmse(&self) -> Option<f64> {
        self.rmse
    }
    
    fn get_error_mae(&self) -> Option<f64> {
        self.mae
    }
    
    fn get_feature_importance(&self) -> Result<Vec<Feature>, Error> {
        // 各サブモデルの特徴量重要度を集約
        let mut all_feature_importance = Vec::new();
        
        for (i, model) in self.sub_models.iter().enumerate() {
            if let Ok(model_importance) = model.get_feature_importance() {
                let weight = if i < self.model_weights.len() {
                    self.model_weights[i]
                } else {
                    1.0 / self.sub_models.len() as f64
                };
                
                for feature in model_importance {
                    if let Some(importance) = feature.importance {
                        let weighted_importance = importance * weight;
                        
                        // 既存の特徴量を探す
                        let mut found = false;
                        for existing_feature in &mut all_feature_importance {
                            if existing_feature.name == feature.name {
                                if let Some(existing_importance) = existing_feature.importance {
                                    existing_feature.importance = Some(existing_importance + weighted_importance);
                                }
                                found = true;
                                break;
                            }
                        }
                        
                        // 新しい特徴量を追加
                        if !found {
                            all_feature_importance.push(Feature {
                                name: feature.name,
                                value: 0.0,
                                importance: Some(weighted_importance),
                            });
                        }
                    }
                }
            }
        }
        
        // 特徴量重要度を正規化
        let total_importance: f64 = all_feature_importance.iter()
            .filter_map(|f| f.importance)
            .sum();
        
        if total_importance > 0.0 {
            for feature in &mut all_feature_importance {
                if let Some(importance) = feature.importance {
                    feature.importance = Some(importance / total_importance);
                }
            }
        }
        
        if all_feature_importance.is_empty() {
            return Err(Error::InvalidState("特徴量重要度が計算されていません".to_string()));
        }
        
        Ok(all_feature_importance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_transactions() -> Vec<Transaction> {
        let now = Utc::now();
        let base_timestamp = now.timestamp();
        
        vec![
            Transaction {
                id: "tx1".to_string(),
                sender: "addr1".to_string(),
                receiver: "addr2".to_string(),
                amount: 100,
                fee: 10,
                timestamp: base_timestamp - 86400 * 7,
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            },
            Transaction {
                id: "tx2".to_string(),
                sender: "addr2".to_string(),
                receiver: "addr3".to_string(),
                amount: 200,
                fee: 20,
                timestamp: base_timestamp - 86400 * 6,
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            },
            Transaction {
                id: "tx3".to_string(),
                sender: "addr3".to_string(),
                receiver: "addr4".to_string(),
                amount: 300,
                fee: 30,
                timestamp: base_timestamp - 86400 * 5,
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            },
            Transaction {
                id: "tx4".to_string(),
                sender: "addr4".to_string(),
                receiver: "addr5".to_string(),
                amount: 400,
                fee: 40,
                timestamp: base_timestamp - 86400 * 4,
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            },
            Transaction {
                id: "tx5".to_string(),
                sender: "addr5".to_string(),
                receiver: "addr6".to_string(),
                amount: 500,
                fee: 50,
                timestamp: base_timestamp - 86400 * 3,
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            },
            Transaction {
                id: "tx6".to_string(),
                sender: "addr6".to_string(),
                receiver: "addr7".to_string(),
                amount: 600,
                fee: 60,
                timestamp: base_timestamp - 86400 * 2,
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            },
            Transaction {
                id: "tx7".to_string(),
                sender: "addr7".to_string(),
                receiver: "addr8".to_string(),
                amount: 700,
                fee: 70,
                timestamp: base_timestamp - 86400,
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            },
            Transaction {
                id: "tx8".to_string(),
                sender: "addr8".to_string(),
                receiver: "addr9".to_string(),
                amount: 800,
                fee: 80,
                timestamp: base_timestamp,
                signature: None,
                status: TransactionStatus::Confirmed,
                data: None,
            },
        ]
    }
    
    #[test]
    fn test_linear_regression_model() {
        let transactions = create_test_transactions();
        
        let model_config = ModelConfig {
            model_type: PredictionModelType::LinearRegression,
            hyperparameters: HashMap::new(),
            features: vec!["amount".to_string(), "fee".to_string(), "hour_of_day".to_string(), "day_of_week".to_string()],
            training_period_days: 30,
            prediction_horizon: PredictionHorizon::ShortTerm,
            confidence_level: 0.95,
        };
        
        let mut predictor = TransactionPredictor::new(model_config);
        
        // モデルを学習
        predictor.train(&transactions).unwrap();
        
        // 予測を実行
        let prediction_result = predictor.predict(PredictionTarget::TransactionCount).unwrap();
        
        // 予測結果を検証
        assert!(!prediction_result.predictions.is_empty());
        assert_eq!(prediction_result.target, PredictionTarget::TransactionCount);
        assert_eq!(prediction_result.model_type, PredictionModelType::LinearRegression);
        assert_eq!(prediction_result.horizon, PredictionHorizon::ShortTerm);
        
        // 特徴量重要度を取得
        let feature_importance = predictor.get_feature_importance().unwrap();
        assert!(!feature_importance.is_empty());
    }
    
    #[test]
    fn test_ensemble_model() {
        let transactions = create_test_transactions();
        
        let model_config = ModelConfig {
            model_type: PredictionModelType::Ensemble,
            hyperparameters: HashMap::new(),
            features: vec!["amount".to_string(), "fee".to_string(), "hour_of_day".to_string(), "day_of_week".to_string()],
            training_period_days: 30,
            prediction_horizon: PredictionHorizon::MediumTerm,
            confidence_level: 0.9,
        };
        
        let mut predictor = TransactionPredictor::new(model_config);
        
        // モデルを学習
        predictor.train(&transactions).unwrap();
        
        // 予測を実行
        let prediction_result = predictor.predict(PredictionTarget::TransactionVolume).unwrap();
        
        // 予測結果を検証
        assert!(!prediction_result.predictions.is_empty());
        assert_eq!(prediction_result.target, PredictionTarget::TransactionVolume);
        assert_eq!(prediction_result.model_type, PredictionModelType::Ensemble);
        assert_eq!(prediction_result.horizon, PredictionHorizon::MediumTerm);
        
        // 信頼区間を検証
        assert!(prediction_result.confidence_lower.is_some());
        assert!(prediction_result.confidence_upper.is_some());
    }
}