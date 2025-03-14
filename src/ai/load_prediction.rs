use tract_onnx::prelude::*;
use crate::error::Error;

/// 負荷予測モデル
pub struct LoadPredictor {
    /// ONNXモデル
    model: Option<SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>>,
    /// 過去の負荷履歴
    load_history: Vec<f32>,
    /// 最大履歴サイズ
    max_history_size: usize,
}

impl LoadPredictor {
    /// 新しいLoadPredictorを作成
    pub fn new(max_history_size: usize) -> Self {
        Self {
            model: None,
            load_history: Vec::with_capacity(max_history_size),
            max_history_size,
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
    
    /// 負荷履歴を更新
    pub fn update_load(&mut self, current_load: f32) {
        self.load_history.push(current_load);
        
        // 履歴サイズを制限
        if self.load_history.len() > self.max_history_size {
            self.load_history.remove(0);
        }
    }
    
    /// 将来の負荷を予測
    pub fn predict_future_load(&self, time_steps: usize) -> Result<Vec<f32>, Error> {
        if self.load_history.is_empty() {
            return Ok(vec![0.0; time_steps]);
        }
        
        // モデルがロードされていない場合は単純な予測を行う
        if self.model.is_none() {
            return Ok(self.simple_load_prediction(time_steps));
        }
        
        // 履歴データを特徴ベクトルに変換
        let features = self.prepare_features();
        
        // ONNXモデルで予測
        match self.predict(&features) {
            Ok(predictions) => Ok(predictions),
            Err(_) => Ok(self.simple_load_prediction(time_steps)),
        }
    }
    
    /// 特徴ベクトルを準備
    fn prepare_features(&self) -> Vec<f32> {
        // 最新のload_history_sizeエントリを使用
        let history_size = self.load_history.len().min(self.max_history_size);
        let start_idx = self.load_history.len() - history_size;
        
        self.load_history[start_idx..].to_vec()
    }
    
    /// 特徴ベクトルを使用して予測
    fn predict(&self, features: &[f32]) -> Result<Vec<f32>, Error> {
        if let Some(model) = &self.model {
            // 入力テンソルを作成
            let input = tract_ndarray::Array::from_vec(features.to_vec())
                .into_shape((1, features.len()))
                .map_err(|e| Error::InternalError(format!("Failed to create input tensor: {}", e)))?;
            
            // 推論を実行
            let result = model
                .run(tvec!(input.into()))
                .map_err(|e| Error::InternalError(format!("Failed to run inference: {}", e)))?;
            
            // 結果を取得
            let output = result[0]
                .to_array_view::<f32>()
                .map_err(|e| Error::InternalError(format!("Failed to get output: {}", e)))?;
            
            // 出力を1次元ベクトルに変換
            let predictions: Vec<f32> = output.iter().copied().collect();
            
            Ok(predictions)
        } else {
            Err(Error::InternalError("Model not loaded".to_string()))
        }
    }
    
    /// 単純な負荷予測（モデルがない場合のフォールバック）
    fn simple_load_prediction(&self, time_steps: usize) -> Vec<f32> {
        let mut predictions = Vec::with_capacity(time_steps);
        
        if self.load_history.is_empty() {
            return vec![0.0; time_steps];
        }
        
        // 直近の平均負荷を計算
        let window_size = 5.min(self.load_history.len());
        let start_idx = self.load_history.len() - window_size;
        let recent_loads = &self.load_history[start_idx..];
        let avg_load: f32 = recent_loads.iter().sum::<f32>() / window_size as f32;
        
        // 直近の傾向（増加/減少）を計算
        let trend = if window_size >= 2 {
            (recent_loads[window_size - 1] - recent_loads[0]) / (window_size - 1) as f32
        } else {
            0.0
        };
        
        // 予測を生成
        for i in 0..time_steps {
            let predicted_load = avg_load + trend * (i + 1) as f32;
            predictions.push(predicted_load.max(0.0));
        }
        
        predictions
    }
    
    /// 最適なシャード数を推定
    pub fn estimate_optimal_shards(&self, current_shards: usize) -> usize {
        // 将来の負荷を予測
        let future_loads = self.predict_future_load(5).unwrap_or_else(|_| vec![0.0; 5]);
        
        // 最大予測負荷を取得
        let max_predicted_load = future_loads.iter().fold(0.0, |a, &b| a.max(b));
        
        // 負荷に基づいてシャード数を調整
        let optimal_shards = if max_predicted_load > 0.8 {
            // 高負荷予測: シャード数を増やす
            (current_shards as f32 * 1.5).round() as usize
        } else if max_predicted_load < 0.3 && current_shards > 10 {
            // 低負荷予測: シャード数を減らす
            (current_shards as f32 * 0.8).round() as usize
        } else {
            // 中程度の負荷: 現在のシャード数を維持
            current_shards
        };
        
        // 最小10、最大100のシャード数に制限
        optimal_shards.max(10).min(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_load_history_update() {
        let mut predictor = LoadPredictor::new(10);
        
        // 負荷履歴を更新
        predictor.update_load(0.5);
        predictor.update_load(0.6);
        predictor.update_load(0.7);
        
        // 履歴サイズを確認
        assert_eq!(predictor.load_history.len(), 3);
        
        // 履歴の値を確認
        assert_eq!(predictor.load_history[0], 0.5);
        assert_eq!(predictor.load_history[1], 0.6);
        assert_eq!(predictor.load_history[2], 0.7);
    }
    
    #[test]
    fn test_simple_load_prediction() {
        let mut predictor = LoadPredictor::new(10);
        
        // 負荷履歴を更新（増加傾向）
        predictor.update_load(0.1);
        predictor.update_load(0.2);
        predictor.update_load(0.3);
        predictor.update_load(0.4);
        predictor.update_load(0.5);
        
        // 将来の負荷を予測
        let predictions = predictor.simple_load_prediction(3);
        
        // 予測サイズを確認
        assert_eq!(predictions.len(), 3);
        
        // 増加傾向が継続することを確認
        assert!(predictions[0] > 0.5);
        assert!(predictions[1] > predictions[0]);
        assert!(predictions[2] > predictions[1]);
    }
    
    #[test]
    fn test_estimate_optimal_shards() {
        let mut predictor = LoadPredictor::new(10);
        
        // 高負荷シナリオ
        for i in 0..10 {
            predictor.update_load(0.7 + i as f32 * 0.02);
        }
        
        // 現在のシャード数が20の場合
        let optimal_shards_high = predictor.estimate_optimal_shards(20);
        
        // シャード数が増加することを確認
        assert!(optimal_shards_high > 20);
        
        // 低負荷シナリオ
        let mut predictor = LoadPredictor::new(10);
        for i in 0..10 {
            predictor.update_load(0.3 - i as f32 * 0.02);
        }
        
        // 現在のシャード数が50の場合
        let optimal_shards_low = predictor.estimate_optimal_shards(50);
        
        // シャード数が減少することを確認
        assert!(optimal_shards_low < 50);
    }
}