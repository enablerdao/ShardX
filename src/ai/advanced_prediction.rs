use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};

use crate::error::Error;

/// 予測モデル
pub trait PredictionModel {
    /// 価格予測を行う
    fn predict_price(&self, pair: &str, current_price: f64, historical_data: &[PricePoint]) -> Result<Prediction, Error>;
    
    /// 取引推奨を行う
    fn recommend_action(&self, prediction: &Prediction) -> Result<TradingRecommendation, Error>;
}

/// 価格データポイント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// 価格
    pub price: f64,
    /// 取引量
    pub volume: Option<f64>,
}

/// 予測結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    /// 取引ペア
    pub pair: String,
    /// 予測期間
    pub period: String,
    /// 現在の価格
    pub current_price: f64,
    /// 予測価格
    pub predicted_price: f64,
    /// 信頼度（0-1）
    pub confidence: f64,
    /// 予測時刻
    pub timestamp: DateTime<Utc>,
    /// 予測の有効期限
    pub expires_at: DateTime<Utc>,
    /// 予測に使用した履歴データ
    pub historical_data: Vec<PricePoint>,
}

/// 取引推奨
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingRecommendation {
    /// 推奨アクション（買い、売り、保持）
    pub action: TradingAction,
    /// 信頼度（0-1）
    pub confidence: f64,
    /// 推奨理由
    pub reasoning: String,
    /// 予測された価格変動率
    pub predicted_change_percent: f64,
    /// 推奨時刻
    pub timestamp: DateTime<Utc>,
}

/// 取引アクション
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradingAction {
    /// 買い
    Buy,
    /// 売り
    Sell,
    /// 保持
    Hold,
}

/// 統計ベースの予測モデル
pub struct StatisticalModel {
    /// 移動平均ウィンドウサイズ
    window_size: usize,
    /// ボラティリティ係数
    volatility_factor: f64,
}

impl StatisticalModel {
    /// 新しい統計モデルを作成
    pub fn new(window_size: usize, volatility_factor: f64) -> Self {
        Self {
            window_size,
            volatility_factor,
        }
    }
    
    /// 移動平均を計算
    fn calculate_moving_average(&self, data: &[PricePoint]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        
        let start_idx = if data.len() > self.window_size {
            data.len() - self.window_size
        } else {
            0
        };
        
        let sum: f64 = data[start_idx..].iter().map(|point| point.price).sum();
        sum / (data.len() - start_idx) as f64
    }
    
    /// ボラティリティを計算
    fn calculate_volatility(&self, data: &[PricePoint], moving_avg: f64) -> f64 {
        if data.len() <= 1 {
            return 0.0;
        }
        
        let start_idx = if data.len() > self.window_size {
            data.len() - self.window_size
        } else {
            0
        };
        
        let variance: f64 = data[start_idx..]
            .iter()
            .map(|point| {
                let diff = point.price - moving_avg;
                diff * diff
            })
            .sum::<f64>() / (data.len() - start_idx) as f64;
        
        variance.sqrt()
    }
    
    /// トレンドを計算
    fn calculate_trend(&self, data: &[PricePoint]) -> f64 {
        if data.len() <= 1 {
            return 0.0;
        }
        
        let start_idx = if data.len() > self.window_size {
            data.len() - self.window_size
        } else {
            0
        };
        
        let first_price = data[start_idx].price;
        let last_price = data[data.len() - 1].price;
        
        (last_price - first_price) / first_price
    }
}

impl PredictionModel for StatisticalModel {
    fn predict_price(&self, pair: &str, current_price: f64, historical_data: &[PricePoint]) -> Result<Prediction, Error> {
        if historical_data.is_empty() {
            return Err(Error::ValidationError("Historical data is empty".to_string()));
        }
        
        // 移動平均を計算
        let moving_avg = self.calculate_moving_average(historical_data);
        
        // ボラティリティを計算
        let volatility = self.calculate_volatility(historical_data, moving_avg);
        
        // トレンドを計算
        let trend = self.calculate_trend(historical_data);
        
        // 予測価格を計算
        let predicted_change = trend * current_price + self.volatility_factor * volatility * (if trend >= 0.0 { 1.0 } else { -1.0 });
        let predicted_price = current_price + predicted_change;
        
        // 信頼度を計算（トレンドの強さとボラティリティに基づく）
        let trend_strength = trend.abs().min(0.1) * 10.0; // 0-1の範囲に正規化
        let volatility_factor = (0.1 / (volatility + 0.1)).min(1.0); // ボラティリティが低いほど信頼度が高い
        let confidence = (trend_strength * 0.7 + volatility_factor * 0.3).min(1.0);
        
        // 現在時刻と有効期限を設定
        let now = Utc::now();
        let expires_at = now + chrono::Duration::hours(24);
        
        Ok(Prediction {
            pair: pair.to_string(),
            period: "day".to_string(),
            current_price,
            predicted_price,
            confidence,
            timestamp: now,
            expires_at,
            historical_data: historical_data.to_vec(),
        })
    }
    
    fn recommend_action(&self, prediction: &Prediction) -> Result<TradingRecommendation, Error> {
        let price_change = prediction.predicted_price - prediction.current_price;
        let price_change_percent = price_change / prediction.current_price * 100.0;
        
        // アクションを決定
        let (action, reasoning) = if price_change_percent > 5.0 && prediction.confidence > 0.6 {
            (
                TradingAction::Buy,
                format!(
                    "Strong buy signal with {:.1}% predicted increase and {:.0}% confidence",
                    price_change_percent,
                    prediction.confidence * 100.0
                )
            )
        } else if price_change_percent < -5.0 && prediction.confidence > 0.6 {
            (
                TradingAction::Sell,
                format!(
                    "Strong sell signal with {:.1}% predicted decrease and {:.0}% confidence",
                    -price_change_percent,
                    prediction.confidence * 100.0
                )
            )
        } else if price_change_percent.abs() < 2.0 || prediction.confidence < 0.4 {
            (
                TradingAction::Hold,
                format!(
                    "Hold recommendation due to small predicted change ({:.1}%) or low confidence ({:.0}%)",
                    price_change_percent,
                    prediction.confidence * 100.0
                )
            )
        } else if price_change_percent > 0.0 {
            (
                TradingAction::Buy,
                format!(
                    "Moderate buy signal with {:.1}% predicted increase and {:.0}% confidence",
                    price_change_percent,
                    prediction.confidence * 100.0
                )
            )
        } else {
            (
                TradingAction::Sell,
                format!(
                    "Moderate sell signal with {:.1}% predicted decrease and {:.0}% confidence",
                    -price_change_percent,
                    prediction.confidence * 100.0
                )
            )
        };
        
        Ok(TradingRecommendation {
            action,
            confidence: prediction.confidence,
            reasoning,
            predicted_change_percent: price_change_percent,
            timestamp: Utc::now(),
        })
    }
}

/// 機械学習ベースの予測モデル
pub struct MachineLearningModel {
    /// モデルの重み
    weights: HashMap<String, f64>,
    /// バイアス
    bias: f64,
}

impl MachineLearningModel {
    /// 新しい機械学習モデルを作成
    pub fn new() -> Self {
        // 実際の実装では、訓練済みモデルをロードする
        // ここでは簡易的な実装として、ハードコードされた重みを使用
        
        let mut weights = HashMap::new();
        weights.insert("price_1d".to_string(), 0.8);
        weights.insert("price_7d".to_string(), 0.5);
        weights.insert("price_30d".to_string(), 0.3);
        weights.insert("volume_1d".to_string(), 0.4);
        weights.insert("volume_7d".to_string(), 0.2);
        weights.insert("trend_1d".to_string(), 0.9);
        weights.insert("trend_7d".to_string(), 0.6);
        weights.insert("volatility".to_string(), -0.3);
        
        Self {
            weights,
            bias: -0.1,
        }
    }
    
    /// 特徴量を抽出
    fn extract_features(&self, current_price: f64, historical_data: &[PricePoint]) -> HashMap<String, f64> {
        let mut features = HashMap::new();
        
        if historical_data.is_empty() {
            return features;
        }
        
        // 現在の価格
        features.insert("current_price".to_string(), current_price);
        
        // 1日前の価格
        if historical_data.len() > 24 {
            features.insert("price_1d".to_string(), historical_data[historical_data.len() - 24].price);
        }
        
        // 7日前の価格
        if historical_data.len() > 168 {
            features.insert("price_7d".to_string(), historical_data[historical_data.len() - 168].price);
        }
        
        // 30日前の価格
        if historical_data.len() > 720 {
            features.insert("price_30d".to_string(), historical_data[historical_data.len() - 720].price);
        }
        
        // 1日の取引量
        let volume_1d: f64 = historical_data
            .iter()
            .rev()
            .take(24)
            .filter_map(|point| point.volume)
            .sum();
        features.insert("volume_1d".to_string(), volume_1d);
        
        // 7日の取引量
        let volume_7d: f64 = historical_data
            .iter()
            .rev()
            .take(168)
            .filter_map(|point| point.volume)
            .sum();
        features.insert("volume_7d".to_string(), volume_7d);
        
        // 1日のトレンド
        if historical_data.len() > 24 {
            let price_1d = historical_data[historical_data.len() - 24].price;
            let trend_1d = (current_price - price_1d) / price_1d;
            features.insert("trend_1d".to_string(), trend_1d);
        }
        
        // 7日のトレンド
        if historical_data.len() > 168 {
            let price_7d = historical_data[historical_data.len() - 168].price;
            let trend_7d = (current_price - price_7d) / price_7d;
            features.insert("trend_7d".to_string(), trend_7d);
        }
        
        // ボラティリティ
        if historical_data.len() > 24 {
            let prices: Vec<f64> = historical_data.iter().rev().take(24).map(|point| point.price).collect();
            let mean = prices.iter().sum::<f64>() / prices.len() as f64;
            let variance = prices.iter().map(|&price| (price - mean).powi(2)).sum::<f64>() / prices.len() as f64;
            let volatility = variance.sqrt();
            features.insert("volatility".to_string(), volatility);
        }
        
        features
    }
    
    /// 予測値を計算
    fn calculate_prediction(&self, features: &HashMap<String, f64>) -> f64 {
        let mut prediction = self.bias;
        
        for (feature, weight) in &self.weights {
            if let Some(value) = features.get(feature) {
                prediction += weight * value;
            }
        }
        
        prediction
    }
    
    /// シグモイド関数
    fn sigmoid(&self, x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }
}

impl PredictionModel for MachineLearningModel {
    fn predict_price(&self, pair: &str, current_price: f64, historical_data: &[PricePoint]) -> Result<Prediction, Error> {
        if historical_data.is_empty() {
            return Err(Error::ValidationError("Historical data is empty".to_string()));
        }
        
        // 特徴量を抽出
        let features = self.extract_features(current_price, historical_data);
        
        // 予測値を計算
        let prediction_factor = self.calculate_prediction(&features);
        
        // 予測価格を計算
        let predicted_change_percent = prediction_factor * 10.0; // スケーリング
        let predicted_price = current_price * (1.0 + predicted_change_percent / 100.0);
        
        // 信頼度を計算
        let confidence = self.sigmoid(prediction_factor.abs() * 2.0).min(1.0);
        
        // 現在時刻と有効期限を設定
        let now = Utc::now();
        let expires_at = now + chrono::Duration::hours(24);
        
        Ok(Prediction {
            pair: pair.to_string(),
            period: "day".to_string(),
            current_price,
            predicted_price,
            confidence,
            timestamp: now,
            expires_at,
            historical_data: historical_data.to_vec(),
        })
    }
    
    fn recommend_action(&self, prediction: &Prediction) -> Result<TradingRecommendation, Error> {
        let price_change = prediction.predicted_price - prediction.current_price;
        let price_change_percent = price_change / prediction.current_price * 100.0;
        
        // アクションを決定
        let (action, reasoning) = if price_change_percent > 5.0 && prediction.confidence > 0.7 {
            (
                TradingAction::Buy,
                format!(
                    "Strong buy signal with {:.1}% predicted increase and {:.0}% confidence based on ML model",
                    price_change_percent,
                    prediction.confidence * 100.0
                )
            )
        } else if price_change_percent < -5.0 && prediction.confidence > 0.7 {
            (
                TradingAction::Sell,
                format!(
                    "Strong sell signal with {:.1}% predicted decrease and {:.0}% confidence based on ML model",
                    -price_change_percent,
                    prediction.confidence * 100.0
                )
            )
        } else if price_change_percent.abs() < 2.0 || prediction.confidence < 0.5 {
            (
                TradingAction::Hold,
                format!(
                    "Hold recommendation due to small predicted change ({:.1}%) or low confidence ({:.0}%) based on ML model",
                    price_change_percent,
                    prediction.confidence * 100.0
                )
            )
        } else if price_change_percent > 0.0 {
            (
                TradingAction::Buy,
                format!(
                    "Moderate buy signal with {:.1}% predicted increase and {:.0}% confidence based on ML model",
                    price_change_percent,
                    prediction.confidence * 100.0
                )
            )
        } else {
            (
                TradingAction::Sell,
                format!(
                    "Moderate sell signal with {:.1}% predicted decrease and {:.0}% confidence based on ML model",
                    -price_change_percent,
                    prediction.confidence * 100.0
                )
            )
        };
        
        Ok(TradingRecommendation {
            action,
            confidence: prediction.confidence,
            reasoning,
            predicted_change_percent: price_change_percent,
            timestamp: Utc::now(),
        })
    }
}

/// 予測サービス
pub struct PredictionService {
    /// 統計モデル
    statistical_model: StatisticalModel,
    /// 機械学習モデル
    ml_model: MachineLearningModel,
    /// キャッシュされた予測
    predictions_cache: Arc<Mutex<HashMap<String, Prediction>>>,
}

impl PredictionService {
    /// 新しい予測サービスを作成
    pub fn new() -> Self {
        Self {
            statistical_model: StatisticalModel::new(24, 0.5),
            ml_model: MachineLearningModel::new(),
            predictions_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// 価格予測を取得
    pub fn get_prediction(&self, pair: &str, current_price: f64, historical_data: &[PricePoint]) -> Result<Prediction, Error> {
        // キャッシュをチェック
        let cache_key = format!("{}:{}", pair, Utc::now().date_naive());
        
        {
            let predictions = self.predictions_cache.lock().unwrap();
            
            if let Some(prediction) = predictions.get(&cache_key) {
                if prediction.expires_at > Utc::now() {
                    return Ok(prediction.clone());
                }
            }
        }
        
        // 統計モデルと機械学習モデルの両方で予測
        let stat_prediction = self.statistical_model.predict_price(pair, current_price, historical_data)?;
        let ml_prediction = self.ml_model.predict_price(pair, current_price, historical_data)?;
        
        // 信頼度に基づいて重み付け
        let stat_weight = stat_prediction.confidence;
        let ml_weight = ml_prediction.confidence * 1.2; // MLモデルに少し重みを付ける
        let total_weight = stat_weight + ml_weight;
        
        let weighted_price = (stat_prediction.predicted_price * stat_weight + ml_prediction.predicted_price * ml_weight) / total_weight;
        let weighted_confidence = (stat_prediction.confidence * stat_weight + ml_prediction.confidence * ml_weight) / total_weight;
        
        // 最終的な予測を作成
        let prediction = Prediction {
            pair: pair.to_string(),
            period: "day".to_string(),
            current_price,
            predicted_price: weighted_price,
            confidence: weighted_confidence,
            timestamp: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(24),
            historical_data: historical_data.to_vec(),
        };
        
        // キャッシュに保存
        {
            let mut predictions = self.predictions_cache.lock().unwrap();
            predictions.insert(cache_key, prediction.clone());
        }
        
        Ok(prediction)
    }
    
    /// 取引推奨を取得
    pub fn get_recommendation(&self, prediction: &Prediction) -> Result<TradingRecommendation, Error> {
        // 統計モデルと機械学習モデルの両方で推奨を取得
        let stat_recommendation = self.statistical_model.recommend_action(prediction)?;
        let ml_recommendation = self.ml_model.recommend_action(prediction)?;
        
        // 信頼度に基づいて選択
        if ml_recommendation.confidence > stat_recommendation.confidence {
            Ok(ml_recommendation)
        } else {
            Ok(stat_recommendation)
        }
    }
    
    /// キャッシュをクリア
    pub fn clear_cache(&self) {
        let mut predictions = self.predictions_cache.lock().unwrap();
        predictions.clear();
    }
    
    /// 期限切れの予測をクリア
    pub fn clear_expired_predictions(&self) {
        let mut predictions = self.predictions_cache.lock().unwrap();
        let now = Utc::now();
        
        predictions.retain(|_, prediction| prediction.expires_at > now);
    }
}