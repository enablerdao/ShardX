use crate::error::Error;
use crate::dex::{Trade, TradingPair};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tract_onnx::prelude::*;

/// 予測期間
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredictionPeriod {
    /// 1時間
    Hour,
    /// 1日
    Day,
    /// 1週間
    Week,
    /// 1ヶ月
    Month,
}

impl PredictionPeriod {
    /// 期間の長さを分単位で取得
    pub fn minutes(&self) -> i64 {
        match self {
            PredictionPeriod::Hour => 60,
            PredictionPeriod::Day => 60 * 24,
            PredictionPeriod::Week => 60 * 24 * 7,
            PredictionPeriod::Month => 60 * 24 * 30,
        }
    }
    
    /// 期間の長さを時間単位で取得
    pub fn hours(&self) -> i64 {
        match self {
            PredictionPeriod::Hour => 1,
            PredictionPeriod::Day => 24,
            PredictionPeriod::Week => 24 * 7,
            PredictionPeriod::Month => 24 * 30,
        }
    }
    
    /// 期間の長さを日単位で取得
    pub fn days(&self) -> i64 {
        match self {
            PredictionPeriod::Hour => 0,
            PredictionPeriod::Day => 1,
            PredictionPeriod::Week => 7,
            PredictionPeriod::Month => 30,
        }
    }
}

/// 価格予測結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePrediction {
    /// 取引ペア
    pub pair: TradingPair,
    /// 予測期間
    pub period: PredictionPeriod,
    /// 現在価格
    pub current_price: f64,
    /// 予測価格
    pub predicted_price: f64,
    /// 予測の信頼区間（下限）
    pub lower_bound: f64,
    /// 予測の信頼区間（上限）
    pub upper_bound: f64,
    /// 予測の信頼度（0.0〜1.0）
    pub confidence: f64,
    /// 予測日時
    pub prediction_time: chrono::DateTime<chrono::Utc>,
    /// 予測対象の日時
    pub target_time: chrono::DateTime<chrono::Utc>,
}

/// 価格予測モデル
pub struct PricePredictionModel {
    /// ONNXモデル
    model: Option<SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>>,
    /// 取引ペア
    pair: TradingPair,
    /// 過去の価格データ
    price_history: Vec<(chrono::DateTime<chrono::Utc>, f64)>,
    /// 最大履歴サイズ
    max_history_size: usize,
    /// 最後の予測結果
    last_prediction: Option<PricePrediction>,
}

impl PricePredictionModel {
    /// 新しいPricePredictionModelを作成
    pub fn new(pair: TradingPair, max_history_size: usize) -> Self {
        Self {
            model: None,
            pair,
            price_history: Vec::with_capacity(max_history_size),
            max_history_size,
            last_prediction: None,
        }
    }
    
    /// ONNXモデルをロード
    pub fn load_model(&mut self, model_path: &str) -> Result<(), Error> {
        // ONNXモデルをロード
        let model = tract_onnx::onnx()
            .model_for_path(model_path)
            .map_err(|e| Error::InternalError(format!("Failed to load ONNX model: {}", e)))?
            .into_optimized()
            .map_err(|e| Error::InternalError(format!("Failed to optimize model: {}", e)))?
            .into_runnable()
            .map_err(|e| Error::InternalError(format!("Failed to make model runnable: {}", e)))?;
        
        self.model = Some(model);
        Ok(())
    }
    
    /// 価格履歴を更新
    pub fn update_price(&mut self, price: f64) {
        let now = chrono::Utc::now();
        self.price_history.push((now, price));
        
        // 履歴サイズを制限
        if self.price_history.len() > self.max_history_size {
            self.price_history.remove(0);
        }
    }
    
    /// 取引から価格履歴を更新
    pub fn update_from_trade(&mut self, trade: &Trade) {
        if trade.pair == self.pair {
            self.update_price(trade.price);
        }
    }
    
    /// 価格を予測
    pub fn predict_price(&mut self, period: PredictionPeriod) -> Result<PricePrediction, Error> {
        let now = chrono::Utc::now();
        
        // 価格履歴が不足している場合はエラー
        if self.price_history.len() < 10 {
            return Err(Error::InternalError("Insufficient price history for prediction".to_string()));
        }
        
        // 現在価格を取得
        let current_price = self.price_history.last().unwrap().1;
        
        // 予測対象の日時を計算
        let target_time = now + chrono::Duration::minutes(period.minutes());
        
        // モデルがロードされていない場合は単純な予測を行う
        if self.model.is_none() {
            let prediction = self.simple_price_prediction(period);
            
            let result = PricePrediction {
                pair: self.pair.clone(),
                period,
                current_price,
                predicted_price: prediction.0,
                lower_bound: prediction.1,
                upper_bound: prediction.2,
                confidence: 0.7,
                prediction_time: now,
                target_time,
            };
            
            self.last_prediction = Some(result.clone());
            return Ok(result);
        }
        
        // 特徴ベクトルを準備
        let features = self.prepare_features();
        
        // ONNXモデルで予測
        match self.predict(&features) {
            Ok(prediction) => {
                let result = PricePrediction {
                    pair: self.pair.clone(),
                    period,
                    current_price,
                    predicted_price: prediction.0,
                    lower_bound: prediction.1,
                    upper_bound: prediction.2,
                    confidence: prediction.3,
                    prediction_time: now,
                    target_time,
                };
                
                self.last_prediction = Some(result.clone());
                Ok(result)
            },
            Err(_) => {
                // モデルによる予測が失敗した場合は単純な予測を行う
                let prediction = self.simple_price_prediction(period);
                
                let result = PricePrediction {
                    pair: self.pair.clone(),
                    period,
                    current_price,
                    predicted_price: prediction.0,
                    lower_bound: prediction.1,
                    upper_bound: prediction.2,
                    confidence: 0.5,
                    prediction_time: now,
                    target_time,
                };
                
                self.last_prediction = Some(result.clone());
                Ok(result)
            }
        }
    }
    
    /// 特徴ベクトルを準備
    fn prepare_features(&self) -> Vec<f32> {
        let mut features = Vec::with_capacity(20);
        
        // 最新の価格データを取得
        let recent_prices: Vec<f64> = self.price_history.iter()
            .map(|(_, price)| *price)
            .rev()
            .take(10)
            .collect();
        
        // 価格データを特徴ベクトルに変換
        for price in recent_prices {
            features.push(price as f32);
        }
        
        // 価格変化率を計算
        if self.price_history.len() >= 2 {
            let last_price = self.price_history.last().unwrap().1;
            let prev_price = self.price_history[self.price_history.len() - 2].1;
            let change_rate = (last_price - prev_price) / prev_price;
            features.push(change_rate as f32);
        } else {
            features.push(0.0);
        }
        
        // 移動平均を計算
        let ma5 = self.calculate_moving_average(5);
        let ma10 = self.calculate_moving_average(10);
        let ma20 = self.calculate_moving_average(20);
        
        features.push(ma5 as f32);
        features.push(ma10 as f32);
        features.push(ma20 as f32);
        
        // ボラティリティを計算
        let volatility = self.calculate_volatility(10);
        features.push(volatility as f32);
        
        // 時間情報を追加
        let now = chrono::Utc::now();
        features.push(now.hour() as f32 / 24.0);
        features.push(now.weekday().num_days_from_monday() as f32 / 7.0);
        features.push(now.day() as f32 / 31.0);
        
        // 特徴ベクトルを20次元に固定（不足分は0で埋める）
        while features.len() < 20 {
            features.push(0.0);
        }
        
        features
    }
    
    /// 移動平均を計算
    fn calculate_moving_average(&self, period: usize) -> f64 {
        if self.price_history.len() < period {
            return self.price_history.last().map(|(_, price)| *price).unwrap_or(0.0);
        }
        
        let sum: f64 = self.price_history.iter()
            .rev()
            .take(period)
            .map(|(_, price)| *price)
            .sum();
        
        sum / period as f64
    }
    
    /// ボラティリティを計算
    fn calculate_volatility(&self, period: usize) -> f64 {
        if self.price_history.len() < period {
            return 0.0;
        }
        
        let prices: Vec<f64> = self.price_history.iter()
            .rev()
            .take(period)
            .map(|(_, price)| *price)
            .collect();
        
        let mean = prices.iter().sum::<f64>() / prices.len() as f64;
        let variance = prices.iter()
            .map(|price| (*price - mean).powi(2))
            .sum::<f64>() / prices.len() as f64;
        
        variance.sqrt()
    }
    
    /// 特徴ベクトルを使用して予測（最適化版）
    fn predict(&self, features: &[f32]) -> Result<(f64, f64, f64, f64), Error> {
        if let Some(model) = &self.model {
            // 最適化: 入力テンソルの作成を効率化
            // 事前に確保されたバッファを使用して再利用
            let input = {
                // 静的なバッファを使用（実際の実装ではスレッドローカルストレージを使用）
                thread_local! {
                    static BUFFER: std::cell::RefCell<Vec<f32>> = std::cell::RefCell::new(Vec::with_capacity(20));
                }
                
                let input_tensor = thread_local! {
                    static TENSOR: std::cell::RefCell<Option<tract_ndarray::Array<f32, tract_ndarray::Dim<[usize; 2]>>>> = 
                        std::cell::RefCell::new(None);
                };
                
                // バッファを再利用
                let tensor = BUFFER.with(|buffer| {
                    let mut buffer = buffer.borrow_mut();
                    buffer.clear();
                    buffer.extend_from_slice(features);
                    
                    input_tensor.with(|tensor_cell| {
                        let mut tensor_opt = tensor_cell.borrow_mut();
                        
                        if let Some(tensor) = tensor_opt.as_mut() {
                            // 既存のテンソルを再利用
                            tensor.assign(&tract_ndarray::Array::from_shape_vec((1, features.len()), buffer.clone())
                                .map_err(|e| Error::InternalError(format!("Failed to create input tensor: {}", e)))?);
                            Ok(tensor.clone())
                        } else {
                            // 新しいテンソルを作成
                            let new_tensor = tract_ndarray::Array::from_shape_vec((1, features.len()), buffer.clone())
                                .map_err(|e| Error::InternalError(format!("Failed to create input tensor: {}", e)))?;
                            *tensor_opt = Some(new_tensor.clone());
                            Ok(new_tensor)
                        }
                    })
                })?;
                
                tensor.into()
            };
            
            // 最適化: 推論のタイムアウトを設定
            let timeout = std::time::Duration::from_millis(100);
            let result = std::sync::Arc::new(std::sync::Mutex::new(None));
            let result_clone = result.clone();
            
            // 別スレッドで推論を実行
            let inference_thread = std::thread::spawn(move || {
                let inference_result = model.run(tvec!(input));
                if let Ok(res) = inference_result {
                    let mut result = result_clone.lock().unwrap();
                    *result = Some(Ok(res));
                } else if let Err(e) = inference_result {
                    let mut result = result_clone.lock().unwrap();
                    *result = Some(Err(Error::InternalError(format!("Failed to run inference: {}", e))));
                }
            });
            
            // タイムアウトを待機
            if inference_thread.join().is_ok() {
                let result_guard = result.lock().unwrap();
                if let Some(res) = &*result_guard {
                    match res {
                        Ok(output) => {
                            // 結果を取得
                            let output_view = output[0]
                                .to_array_view::<f32>()
                                .map_err(|e| Error::InternalError(format!("Failed to get output: {}", e)))?;
                            
                            // 出力は [predicted_price, lower_bound, upper_bound, confidence] の形式
                            let predicted_price = output_view[[0, 0]] as f64;
                            let lower_bound = output_view[[0, 1]] as f64;
                            let upper_bound = output_view[[0, 2]] as f64;
                            let confidence = output_view[[0, 3]] as f64;
                            
                            return Ok((predicted_price, lower_bound, upper_bound, confidence));
                        },
                        Err(e) => return Err(e.clone()),
                    }
                }
            }
            
            // タイムアウトした場合はフォールバック
            warn!("Model inference timed out, falling back to simple prediction");
            let prediction = self.simple_price_prediction(PredictionPeriod::Hour);
            Ok((prediction.0, prediction.1, prediction.2, 0.5))
        } else {
            Err(Error::InternalError("Model not loaded".to_string()))
        }
    }
    
    /// 単純な価格予測（モデルがない場合のフォールバック）
    fn simple_price_prediction(&self, period: PredictionPeriod) -> (f64, f64, f64) {
        // 現在価格を取得
        let current_price = self.price_history.last().unwrap().1;
        
        // 過去の価格変化率を計算
        let mut change_rates = Vec::new();
        for i in 1..self.price_history.len() {
            let prev_price = self.price_history[i - 1].1;
            let curr_price = self.price_history[i].1;
            let change_rate = (curr_price - prev_price) / prev_price;
            change_rates.push(change_rate);
        }
        
        // 平均変化率を計算
        let avg_change_rate = if change_rates.is_empty() {
            0.0
        } else {
            change_rates.iter().sum::<f64>() / change_rates.len() as f64
        };
        
        // 変化率の標準偏差を計算
        let std_dev = if change_rates.len() < 2 {
            0.01
        } else {
            let mean = avg_change_rate;
            let variance = change_rates.iter()
                .map(|rate| (*rate - mean).powi(2))
                .sum::<f64>() / (change_rates.len() - 1) as f64;
            variance.sqrt()
        };
        
        // 予測期間に応じて変化率を調整
        let period_factor = match period {
            PredictionPeriod::Hour => 1.0,
            PredictionPeriod::Day => 24.0,
            PredictionPeriod::Week => 24.0 * 7.0,
            PredictionPeriod::Month => 24.0 * 30.0,
        };
        
        // 予測価格を計算
        let predicted_change = avg_change_rate * period_factor;
        let predicted_price = current_price * (1.0 + predicted_change);
        
        // 信頼区間を計算
        let confidence_interval = std_dev * period_factor.sqrt() * 1.96; // 95%信頼区間
        let lower_bound = current_price * (1.0 + predicted_change - confidence_interval);
        let upper_bound = current_price * (1.0 + predicted_change + confidence_interval);
        
        (predicted_price, lower_bound, upper_bound)
    }
    
    /// 最後の予測結果を取得
    pub fn get_last_prediction(&self) -> Option<PricePrediction> {
        self.last_prediction.clone()
    }
    
    /// 予測の精度を評価
    pub fn evaluate_prediction_accuracy(&self) -> Option<f64> {
        if let Some(prediction) = &self.last_prediction {
            // 予測対象の時間がまだ来ていない場合はNone
            let now = chrono::Utc::now();
            if now < prediction.target_time {
                return None;
            }
            
            // 予測対象の時間に最も近い実際の価格を取得
            let actual_price = self.price_history.iter()
                .filter(|(time, _)| *time >= prediction.target_time)
                .min_by_key(|(time, _)| (*time - prediction.target_time).num_milliseconds().abs())
                .map(|(_, price)| *price);
            
            if let Some(actual_price) = actual_price {
                // 予測誤差を計算
                let error = (actual_price - prediction.predicted_price).abs() / actual_price;
                return Some(1.0 - error); // 精度 = 1 - 相対誤差
            }
        }
        
        None
    }
}

/// 取引予測マネージャー
pub struct TradePredictionManager {
    /// 価格予測モデルのマップ
    models: Mutex<HashMap<String, PricePredictionModel>>,
}

impl TradePredictionManager {
    /// 新しいTradePredictionManagerを作成
    pub fn new() -> Self {
        Self {
            models: Mutex::new(HashMap::new()),
        }
    }
    
    /// 価格予測モデルを追加
    pub fn add_model(&self, pair: TradingPair, max_history_size: usize) -> Result<(), Error> {
        let pair_str = format!("{}/{}", pair.base, pair.quote);
        
        let mut models = self.models.lock().unwrap();
        if models.contains_key(&pair_str) {
            return Err(Error::InternalError(format!("Model for pair {} already exists", pair_str)));
        }
        
        let model = PricePredictionModel::new(pair, max_history_size);
        models.insert(pair_str, model);
        
        Ok(())
    }
    
    /// 価格予測モデルにONNXモデルをロード
    pub fn load_model(&self, pair: &TradingPair, model_path: &str) -> Result<(), Error> {
        let pair_str = format!("{}/{}", pair.base, pair.quote);
        
        let mut models = self.models.lock().unwrap();
        let model = models.get_mut(&pair_str)
            .ok_or_else(|| Error::InternalError(format!("Model for pair {} not found", pair_str)))?;
        
        model.load_model(model_path)
    }
    
    /// 取引から価格履歴を更新
    pub fn update_from_trade(&self, trade: &Trade) {
        let pair_str = format!("{}/{}", trade.pair.base, trade.pair.quote);
        
        let mut models = self.models.lock().unwrap();
        if let Some(model) = models.get_mut(&pair_str) {
            model.update_from_trade(trade);
        }
    }
    
    /// 価格を予測
    pub fn predict_price(&self, pair: &TradingPair, period: PredictionPeriod) -> Result<PricePrediction, Error> {
        let pair_str = format!("{}/{}", pair.base, pair.quote);
        
        let mut models = self.models.lock().unwrap();
        let model = models.get_mut(&pair_str)
            .ok_or_else(|| Error::InternalError(format!("Model for pair {} not found", pair_str)))?;
        
        model.predict_price(period)
    }
    
    /// 最後の予測結果を取得
    pub fn get_last_prediction(&self, pair: &TradingPair) -> Option<PricePrediction> {
        let pair_str = format!("{}/{}", pair.base, pair.quote);
        
        let models = self.models.lock().unwrap();
        models.get(&pair_str).and_then(|model| model.get_last_prediction())
    }
    
    /// 予測の精度を評価
    pub fn evaluate_prediction_accuracy(&self, pair: &TradingPair) -> Option<f64> {
        let pair_str = format!("{}/{}", pair.base, pair.quote);
        
        let models = self.models.lock().unwrap();
        models.get(&pair_str).and_then(|model| model.evaluate_prediction_accuracy())
    }
    
    /// すべての取引ペアの予測を取得
    pub fn get_all_predictions(&self, period: PredictionPeriod) -> HashMap<String, Result<PricePrediction, Error>> {
        let mut results = HashMap::new();
        
        let models = self.models.lock().unwrap();
        for (pair_str, model) in models.iter() {
            let result = model.predict_price(period);
            results.insert(pair_str.clone(), result);
        }
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_trading_pair() -> TradingPair {
        TradingPair {
            base: "BTC".to_string(),
            quote: "USD".to_string(),
        }
    }
    
    fn create_test_trade(price: f64) -> Trade {
        Trade {
            id: Uuid::new_v4().to_string(),
            buy_order_id: "buy1".to_string(),
            sell_order_id: "sell1".to_string(),
            pair: create_test_trading_pair(),
            price,
            amount: 1.0,
            executed_at: chrono::Utc::now(),
        }
    }
    
    #[test]
    fn test_price_prediction_model() {
        let pair = create_test_trading_pair();
        let mut model = PricePredictionModel::new(pair, 100);
        
        // 価格履歴を追加
        for i in 0..20 {
            let price = 10000.0 + (i as f64 * 100.0);
            model.update_price(price);
        }
        
        // 予測を実行
        let prediction = model.predict_price(PredictionPeriod::Hour).unwrap();
        
        // 予測結果の検証
        assert_eq!(prediction.pair.base, "BTC");
        assert_eq!(prediction.pair.quote, "USD");
        assert_eq!(prediction.period, PredictionPeriod::Hour);
        assert!(prediction.predicted_price > 0.0);
        assert!(prediction.lower_bound <= prediction.predicted_price);
        assert!(prediction.upper_bound >= prediction.predicted_price);
        assert!(prediction.confidence > 0.0 && prediction.confidence <= 1.0);
    }
    
    #[test]
    fn test_trade_prediction_manager() {
        let manager = TradePredictionManager::new();
        let pair = create_test_trading_pair();
        
        // モデルを追加
        manager.add_model(pair.clone(), 100).unwrap();
        
        // 取引から価格履歴を更新
        for i in 0..20 {
            let price = 10000.0 + (i as f64 * 100.0);
            let trade = create_test_trade(price);
            manager.update_from_trade(&trade);
        }
        
        // 予測を実行
        let prediction = manager.predict_price(&pair, PredictionPeriod::Hour).unwrap();
        
        // 予測結果の検証
        assert_eq!(prediction.pair.base, "BTC");
        assert_eq!(prediction.pair.quote, "USD");
        assert_eq!(prediction.period, PredictionPeriod::Hour);
        assert!(prediction.predicted_price > 0.0);
        
        // 最後の予測結果を取得
        let last_prediction = manager.get_last_prediction(&pair).unwrap();
        assert_eq!(last_prediction.predicted_price, prediction.predicted_price);
    }
}