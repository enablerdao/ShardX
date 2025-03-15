use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::error::Error;
use crate::crypto::{PublicKey, Signature};

/// マルチシグ閾値ポリシー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdPolicy {
    /// 必要な署名数
    pub required_signatures: usize,
    /// 許可された公開鍵のリスト
    pub allowed_public_keys: Vec<PublicKey>,
    /// 公開鍵の重み（オプション）
    pub weights: Option<HashMap<PublicKey, u32>>,
    /// 有効期限（オプション）
    pub expiration: Option<DateTime<Utc>>,
}

impl ThresholdPolicy {
    /// 新しい閾値ポリシーを作成
    pub fn new(required_signatures: usize, allowed_public_keys: Vec<PublicKey>) -> Self {
        Self {
            required_signatures,
            allowed_public_keys,
            weights: None,
            expiration: None,
        }
    }
    
    /// 重み付きポリシーを作成
    pub fn with_weights(allowed_public_keys: Vec<PublicKey>, weights: HashMap<PublicKey, u32>, threshold: u32) -> Self {
        let required_signatures = allowed_public_keys.len(); // 実際には重みで判断するため、最大値を設定
        
        Self {
            required_signatures,
            allowed_public_keys,
            weights: Some(weights),
            expiration: None,
        }
    }
    
    /// 有効期限付きポリシーを作成
    pub fn with_expiration(required_signatures: usize, allowed_public_keys: Vec<PublicKey>, expiration: DateTime<Utc>) -> Self {
        Self {
            required_signatures,
            allowed_public_keys,
            weights: None,
            expiration: Some(expiration),
        }
    }
    
    /// ポリシーが有効かどうかを確認
    pub fn is_valid(&self) -> bool {
        // 有効期限をチェック
        if let Some(expiration) = self.expiration {
            if Utc::now() > expiration {
                return false;
            }
        }
        
        // 必要な署名数が許可された公開鍵の数以下であることを確認
        if self.required_signatures > self.allowed_public_keys.len() {
            return false;
        }
        
        // 重みが設定されている場合、全ての公開鍵に重みが設定されていることを確認
        if let Some(weights) = &self.weights {
            for key in &self.allowed_public_keys {
                if !weights.contains_key(key) {
                    return false;
                }
            }
        }
        
        true
    }
    
    /// 公開鍵が許可されているかどうかを確認
    pub fn is_allowed(&self, public_key: &PublicKey) -> bool {
        self.allowed_public_keys.contains(public_key)
    }
    
    /// 署名が閾値を満たしているかどうかを確認
    pub fn is_threshold_met(&self, signatures: &HashMap<PublicKey, Signature>) -> bool {
        if !self.is_valid() {
            return false;
        }
        
        // 有効な署名の数をカウント
        let valid_signatures: Vec<&PublicKey> = signatures.keys()
            .filter(|key| self.is_allowed(key))
            .collect();
        
        // 重みが設定されている場合
        if let Some(weights) = &self.weights {
            let total_weight: u32 = valid_signatures.iter()
                .filter_map(|key| weights.get(key))
                .sum();
            
            // 閾値は required_signatures フィールドに格納されていると仮定
            return total_weight >= self.required_signatures as u32;
        }
        
        // 重みが設定されていない場合は単純に署名数をチェック
        valid_signatures.len() >= self.required_signatures
    }
    
    /// 残りの必要署名数を取得
    pub fn remaining_signatures(&self, signatures: &HashMap<PublicKey, Signature>) -> usize {
        if !self.is_valid() {
            return self.required_signatures;
        }
        
        // 有効な署名の数をカウント
        let valid_signatures: Vec<&PublicKey> = signatures.keys()
            .filter(|key| self.is_allowed(key))
            .collect();
        
        // 重みが設定されている場合
        if let Some(weights) = &self.weights {
            let total_weight: u32 = valid_signatures.iter()
                .filter_map(|key| weights.get(key))
                .sum();
            
            let threshold = self.required_signatures as u32;
            if total_weight >= threshold {
                return 0;
            }
            
            // 残りの重みを計算（簡易的な実装）
            return (threshold - total_weight) as usize;
        }
        
        // 重みが設定されていない場合は単純に署名数をチェック
        if valid_signatures.len() >= self.required_signatures {
            return 0;
        }
        
        self.required_signatures - valid_signatures.len()
    }
    
    /// 有効期限までの残り時間（秒）を取得
    pub fn time_remaining(&self) -> Option<i64> {
        self.expiration.map(|expiration| {
            let now = Utc::now();
            if now >= expiration {
                return 0;
            }
            
            (expiration - now).num_seconds()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_keypair;
    
    #[test]
    fn test_threshold_policy() {
        // キーペアを生成
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let keypair3 = generate_keypair();
        
        // 2-of-3ポリシーを作成
        let policy = ThresholdPolicy::new(
            2,
            vec![keypair1.public.clone(), keypair2.public.clone(), keypair3.public.clone()]
        );
        
        assert!(policy.is_valid());
        assert!(policy.is_allowed(&keypair1.public));
        assert!(!policy.is_allowed(&generate_keypair().public));
        
        // 署名マップを作成
        let mut signatures = HashMap::new();
        signatures.insert(keypair1.public.clone(), "sig1".to_string());
        
        // 閾値を満たしていないことを確認
        assert!(!policy.is_threshold_met(&signatures));
        assert_eq!(policy.remaining_signatures(&signatures), 1);
        
        // 署名を追加
        signatures.insert(keypair2.public.clone(), "sig2".to_string());
        
        // 閾値を満たしていることを確認
        assert!(policy.is_threshold_met(&signatures));
        assert_eq!(policy.remaining_signatures(&signatures), 0);
    }
    
    #[test]
    fn test_weighted_policy() {
        // キーペアを生成
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let keypair3 = generate_keypair();
        
        // 重みを設定
        let mut weights = HashMap::new();
        weights.insert(keypair1.public.clone(), 3);
        weights.insert(keypair2.public.clone(), 2);
        weights.insert(keypair3.public.clone(), 1);
        
        // 重み付きポリシーを作成（閾値4）
        let policy = ThresholdPolicy::with_weights(
            vec![keypair1.public.clone(), keypair2.public.clone(), keypair3.public.clone()],
            weights,
            4
        );
        
        assert!(policy.is_valid());
        
        // 署名マップを作成
        let mut signatures = HashMap::new();
        signatures.insert(keypair1.public.clone(), "sig1".to_string());
        
        // 閾値を満たしていないことを確認（重み3 < 閾値4）
        assert!(!policy.is_threshold_met(&signatures));
        
        // 署名を追加
        signatures.insert(keypair2.public.clone(), "sig2".to_string());
        
        // 閾値を満たしていることを確認（重み3+2=5 > 閾値4）
        assert!(policy.is_threshold_met(&signatures));
    }
    
    #[test]
    fn test_expiration() {
        // キーペアを生成
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        
        // 過去の日時を設定
        let past = Utc::now() - chrono::Duration::days(1);
        
        // 有効期限切れのポリシーを作成
        let expired_policy = ThresholdPolicy::with_expiration(
            1,
            vec![keypair1.public.clone(), keypair2.public.clone()],
            past
        );
        
        assert!(!expired_policy.is_valid());
        
        // 未来の日時を設定
        let future = Utc::now() + chrono::Duration::days(1);
        
        // 有効なポリシーを作成
        let valid_policy = ThresholdPolicy::with_expiration(
            1,
            vec![keypair1.public.clone(), keypair2.public.clone()],
            future
        );
        
        assert!(valid_policy.is_valid());
        assert!(valid_policy.time_remaining().unwrap() > 0);
    }
}