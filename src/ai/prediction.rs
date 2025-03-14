use std::collections::BinaryHeap;
use std::cmp::Ordering;
use tract_onnx::prelude::*;
use crate::transaction::Transaction;
use crate::error::Error;

/// 優先度付きトランザクション
#[derive(Debug, Clone)]
pub struct PrioritizedTransaction {
    /// トランザクション
    pub transaction: Transaction,
    /// 優先度スコア
    pub priority: f32,
}

impl PartialEq for PrioritizedTransaction {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for PrioritizedTransaction {}

impl PartialOrd for PrioritizedTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl Ord for PrioritizedTransaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.partial_cmp(&other.priority)
            .unwrap_or(Ordering::Equal)
    }
}

/// ONNXモデルを使用したトランザクション予測器
pub struct TransactionPredictor {
    /// ONNXモデル
    model: Option<SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>>,
    /// 優先度キュー
    priority_queue: BinaryHeap<PrioritizedTransaction>,
}

impl TransactionPredictor {
    /// 新しいTransactionPredictorを作成
    pub fn new() -> Self {
        Self {
            model: None,
            priority_queue: BinaryHeap::new(),
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
    
    /// トランザクションの優先度を予測
    pub fn predict_priority(&self, tx: &Transaction) -> f32 {
        // モデルがロードされていない場合はデフォルト値を返す
        if self.model.is_none() {
            return self.calculate_basic_priority(tx);
        }
        
        // トランザクションの特徴を抽出
        let features = self.extract_features(tx);
        
        // ONNXモデルで予測
        match self.predict(&features) {
            Ok(priority) => priority,
            Err(_) => self.calculate_basic_priority(tx),
        }
    }
    
    /// 特徴ベクトルを使用して予測
    fn predict(&self, features: &[f32]) -> Result<f32, Error> {
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
            
            Ok(output[[0, 0]])
        } else {
            Err(Error::InternalError("Model not loaded".to_string()))
        }
    }
    
    /// トランザクションの特徴を抽出
    pub fn extract_features(&self, tx: &Transaction) -> Vec<f32> {
        let mut features = Vec::with_capacity(10);
        
        // 1. トランザクションサイズ
        features.push(tx.payload.len() as f32 / 1000.0);
        
        // 2. 親トランザクション数
        features.push(tx.parent_ids.len() as f32);
        
        // 3. タイムスタンプ（現在時刻との差）
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as f32;
        let tx_time = tx.timestamp as f32;
        features.push((current_time - tx_time) / 3600.0); // 時間単位
        
        // 4. 緊急フラグ（ペイロードの最初のバイトで簡易判定）
        let is_urgent = if !tx.payload.is_empty() && tx.payload[0] > 200 { 1.0 } else { 0.0 };
        features.push(is_urgent);
        
        // 5. 金額（ペイロードから簡易抽出）
        let amount = self.extract_amount_from_payload(&tx.payload);
        features.push(amount / 1000.0); // 1000単位
        
        // 特徴ベクトルを10次元に固定（不足分は0で埋める）
        while features.len() < 10 {
            features.push(0.0);
        }
        
        features
    }
    
    /// ペイロードから金額を抽出（簡易実装）
    fn extract_amount_from_payload(&self, payload: &[u8]) -> f32 {
        if payload.len() < 4 {
            return 0.0;
        }
        
        // 簡易実装：最初の4バイトを金額として解釈
        let mut amount_bytes = [0u8; 4];
        amount_bytes.copy_from_slice(&payload[0..4]);
        let amount = u32::from_le_bytes(amount_bytes) as f32;
        
        amount
    }
    
    /// 基本的な優先度計算（モデルがない場合のフォールバック）
    fn calculate_basic_priority(&self, tx: &Transaction) -> f32 {
        let mut priority = 0.0;
        
        // 1. ペイロードサイズ（小さいほど優先）
        let size_factor = 1.0 - (tx.payload.len() as f32 / 10000.0).min(1.0);
        priority += size_factor * 100.0;
        
        // 2. 親トランザクション数（多いほど優先）
        let parent_factor = (tx.parent_ids.len() as f32 / 10.0).min(1.0);
        priority += parent_factor * 200.0;
        
        // 3. 金額（大きいほど優先）
        let amount = self.extract_amount_from_payload(&tx.payload);
        if amount >= 1000.0 {
            priority += 500.0; // $1000以上は+500
        }
        
        // 4. 緊急フラグ
        if !tx.payload.is_empty() && tx.payload[0] > 200 {
            priority += 300.0; // 緊急は+300
        }
        
        priority
    }
    
    /// トランザクションを優先度キューに追加
    pub fn prioritize(&mut self, tx: Transaction) {
        let priority = self.predict_priority(&tx);
        
        self.priority_queue.push(PrioritizedTransaction {
            transaction: tx,
            priority,
        });
    }
    
    /// 次のバッチを取得
    pub fn get_next_batch(&mut self, batch_size: usize) -> Vec<Transaction> {
        let mut batch = Vec::with_capacity(batch_size);
        
        for _ in 0..batch_size {
            if let Some(prioritized_tx) = self.priority_queue.pop() {
                batch.push(prioritized_tx.transaction);
            } else {
                break;
            }
        }
        
        batch
    }
    
    /// キューサイズを取得
    pub fn queue_size(&self) -> usize {
        self.priority_queue.len()
    }
    
    /// キューをクリア
    pub fn clear_queue(&mut self) {
        self.priority_queue.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::TransactionStatus;
    
    fn create_test_transaction(id: &str, payload: Vec<u8>) -> Transaction {
        Transaction {
            id: id.to_string(),
            parent_ids: vec!["parent1".to_string()],
            timestamp: 12345,
            payload,
            signature: vec![4, 5, 6],
            status: TransactionStatus::Pending,
        }
    }
    
    #[test]
    fn test_extract_features() {
        let predictor = TransactionPredictor::new();
        
        // テスト用のトランザクション
        let tx = create_test_transaction("tx1", vec![1, 2, 3, 4]);
        
        // 特徴抽出
        let features = predictor.extract_features(&tx);
        
        // 特徴ベクトルのサイズを確認
        assert_eq!(features.len(), 10);
        
        // 特徴値の範囲を確認
        for feature in &features {
            assert!(!feature.is_nan());
        }
    }
    
    #[test]
    fn test_basic_priority_calculation() {
        let predictor = TransactionPredictor::new();
        
        // 通常のトランザクション
        let tx1 = create_test_transaction("tx1", vec![1, 2, 3, 4]);
        let priority1 = predictor.calculate_basic_priority(&tx1);
        
        // 大きな金額のトランザクション
        let mut amount_bytes = 1000_u32.to_le_bytes().to_vec();
        amount_bytes.extend_from_slice(&[5, 6, 7, 8]);
        let tx2 = create_test_transaction("tx2", amount_bytes);
        let priority2 = predictor.calculate_basic_priority(&tx2);
        
        // 緊急フラグ付きトランザクション
        let tx3 = create_test_transaction("tx3", vec![201, 2, 3, 4]);
        let priority3 = predictor.calculate_basic_priority(&tx3);
        
        // 優先度の比較
        assert!(priority2 > priority1); // 大きな金額は優先度が高い
        assert!(priority3 > priority1); // 緊急フラグは優先度が高い
    }
    
    #[test]
    fn test_prioritize_and_get_batch() {
        let mut predictor = TransactionPredictor::new();
        
        // 複数のトランザクションを追加
        let tx1 = create_test_transaction("tx1", vec![1, 2, 3, 4]);
        let tx2 = create_test_transaction("tx2", vec![201, 2, 3, 4]); // 緊急フラグ付き
        let tx3 = create_test_transaction("tx3", vec![1, 2, 3, 4]);
        
        predictor.prioritize(tx1);
        predictor.prioritize(tx2);
        predictor.prioritize(tx3);
        
        // キューサイズを確認
        assert_eq!(predictor.queue_size(), 3);
        
        // バッチを取得
        let batch = predictor.get_next_batch(2);
        
        // バッチサイズを確認
        assert_eq!(batch.len(), 2);
        
        // 残りのキューサイズを確認
        assert_eq!(predictor.queue_size(), 1);
        
        // 優先度の高いトランザクション（緊急フラグ付き）が最初に取得されることを確認
        assert_eq!(batch[0].id, "tx2");
    }
}