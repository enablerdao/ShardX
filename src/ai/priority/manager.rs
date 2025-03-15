//! AI優先度マネージャー
//! 
//! トランザクションの優先度を管理するAIベースのマネージャー

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::error::Error;
use crate::transaction::Transaction;

/// AI優先度マネージャー
pub struct AIPriorityManager {
    /// トランザクション優先度マップ
    priorities: Mutex<HashMap<String, f64>>,
    /// 優先度計算モデル
    model: Option<PriorityModel>,
    /// 設定
    config: PriorityConfig,
}

/// 優先度計算モデル
struct PriorityModel {
    /// モデルの重み
    weights: HashMap<String, f64>,
    /// バイアス
    bias: f64,
}

/// 優先度設定
#[derive(Debug, Clone)]
pub struct PriorityConfig {
    /// 手数料の重み
    pub fee_weight: f64,
    /// 待機時間の重み
    pub wait_time_weight: f64,
    /// トランザクションサイズの重み
    pub size_weight: f64,
    /// ユーザー評価の重み
    pub user_rating_weight: f64,
    /// 最小優先度
    pub min_priority: f64,
    /// 最大優先度
    pub max_priority: f64,
}

impl AIPriorityManager {
    /// 新しいAI優先度マネージャーを作成
    pub fn new(config: PriorityConfig) -> Self {
        let model = PriorityModel {
            weights: HashMap::new(),
            bias: 0.0,
        };
        
        Self {
            priorities: Mutex::new(HashMap::new()),
            model: Some(model),
            config,
        }
    }
    
    /// トランザクションの優先度を計算
    pub fn calculate_priority(&self, tx: &Transaction) -> Result<f64, Error> {
        // 基本的な特徴量を抽出
        let features = self.extract_features(tx);
        
        // モデルがある場合はモデルを使用
        if let Some(model) = &self.model {
            let mut priority = model.bias;
            
            for (feature, value) in &features {
                if let Some(weight) = model.weights.get(feature) {
                    priority += weight * value;
                }
            }
            
            // 優先度を範囲内に収める
            priority = priority.max(self.config.min_priority).min(self.config.max_priority);
            
            // 優先度を保存
            let mut priorities = self.priorities.lock().unwrap();
            priorities.insert(tx.id().to_string(), priority);
            
            Ok(priority)
        } else {
            // モデルがない場合は手数料ベースの単純な優先度を計算
            let fee = tx.fee.parse::<f64>().unwrap_or(0.0);
            let priority = fee * self.config.fee_weight;
            
            // 優先度を範囲内に収める
            let priority = priority.max(self.config.min_priority).min(self.config.max_priority);
            
            // 優先度を保存
            let mut priorities = self.priorities.lock().unwrap();
            priorities.insert(tx.id().to_string(), priority);
            
            Ok(priority)
        }
    }
    
    /// トランザクションから特徴量を抽出
    fn extract_features(&self, tx: &Transaction) -> HashMap<String, f64> {
        let mut features = HashMap::new();
        
        // 手数料
        let fee = tx.fee.parse::<f64>().unwrap_or(0.0);
        features.insert("fee".to_string(), fee);
        
        // トランザクションサイズ（ペイロードサイズで近似）
        let size = tx.payload.len() as f64;
        features.insert("size".to_string(), size);
        
        // その他の特徴量を追加
        
        features
    }
    
    /// トランザクションの優先度を取得
    pub fn get_priority(&self, tx_id: &str) -> Option<f64> {
        let priorities = self.priorities.lock().unwrap();
        priorities.get(tx_id).cloned()
    }
    
    /// 優先度に基づいてトランザクションをソート
    pub fn sort_transactions(&self, txs: &mut [Transaction]) {
        txs.sort_by(|a, b| {
            let a_priority = self.get_priority(a.id()).unwrap_or(0.0);
            let b_priority = self.get_priority(b.id()).unwrap_or(0.0);
            b_priority.partial_cmp(&a_priority).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    
    /// モデルを更新
    pub fn update_model(&mut self, weights: HashMap<String, f64>, bias: f64) {
        let model = PriorityModel {
            weights,
            bias,
        };
        
        self.model = Some(model);
    }
    
    /// 設定を更新
    pub fn update_config(&mut self, config: PriorityConfig) {
        self.config = config;
    }
}

impl Default for PriorityConfig {
    fn default() -> Self {
        Self {
            fee_weight: 0.7,
            wait_time_weight: 0.1,
            size_weight: -0.1,
            user_rating_weight: 0.3,
            min_priority: 0.0,
            max_priority: 10.0,
        }
    }
}
