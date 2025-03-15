use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use chrono::Utc;
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::crypto::{PublicKey, Signature, KeyPair, hash};
use crate::transaction::{Transaction, TransactionStatus};
use crate::wallet::WalletId;
use crate::wallet::multisig::threshold::ThresholdPolicy;
use crate::wallet::multisig::enhanced_wallet::{EnhancedMultisigWallet, WalletStatus};
use crate::wallet::multisig::enhanced_transaction::{EnhancedMultisigTransaction, MultisigTransactionState};

/// マルチシグウォレットマネージャー
pub struct EnhancedMultisigManager {
    /// ウォレット
    wallets: RwLock<HashMap<WalletId, EnhancedMultisigWallet>>,
    /// トランザクション
    transactions: RwLock<HashMap<String, EnhancedMultisigTransaction>>,
    /// 公開鍵とウォレットのマッピング
    key_to_wallets: RwLock<HashMap<PublicKey, HashSet<WalletId>>>,
}

impl EnhancedMultisigManager {
    /// 新しいマルチシグウォレットマネージャーを作成
    pub fn new() -> Self {
        Self {
            wallets: RwLock::new(HashMap::new()),
            transactions: RwLock::new(HashMap::new()),
            key_to_wallets: RwLock::new(HashMap::new()),
        }
    }
    
    /// ウォレットを作成
    pub fn create_wallet(&self, name: String, policy: ThresholdPolicy) -> Result<EnhancedMultisigWallet, Error> {
        if !policy.is_valid() {
            return Err(Error::InvalidInput("無効なポリシーです".to_string()));
        }
        
        let wallet = EnhancedMultisigWallet::new(name, policy.clone());
        
        // ウォレットを保存
        let mut wallets = self.wallets.write().unwrap();
        wallets.insert(wallet.id.clone(), wallet.clone());
        
        // 公開鍵とウォレットのマッピングを更新
        let mut key_to_wallets = self.key_to_wallets.write().unwrap();
        for key in &policy.allowed_public_keys {
            let wallet_set = key_to_wallets.entry(key.clone()).or_insert_with(HashSet::new);
            wallet_set.insert(wallet.id.clone());
        }
        
        Ok(wallet)
    }
    
    /// ウォレットを取得
    pub fn get_wallet(&self, wallet_id: &WalletId) -> Result<EnhancedMultisigWallet, Error> {
        let wallets = self.wallets.read().unwrap();
        
        wallets.get(wallet_id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("ウォレット {} が見つかりません", wallet_id)))
    }
    
    /// ウォレットを更新
    pub fn update_wallet(&self, wallet: EnhancedMultisigWallet) -> Result<(), Error> {
        let mut wallets = self.wallets.write().unwrap();
        
        if !wallets.contains_key(&wallet.id) {
            return Err(Error::NotFound(format!("ウォレット {} が見つかりません", wallet.id)));
        }
        
        // 古いウォレットを取得
        let old_wallet = wallets.get(&wallet.id).unwrap();
        
        // ポリシーが変更された場合、公開鍵とウォレットのマッピングを更新
        if old_wallet.policy.allowed_public_keys != wallet.policy.allowed_public_keys {
            let mut key_to_wallets = self.key_to_wallets.write().unwrap();
            
            // 古いマッピングを削除
            for key in &old_wallet.policy.allowed_public_keys {
                if let Some(wallet_set) = key_to_wallets.get_mut(key) {
                    wallet_set.remove(&wallet.id);
                    if wallet_set.is_empty() {
                        key_to_wallets.remove(key);
                    }
                }
            }
            
            // 新しいマッピングを追加
            for key in &wallet.policy.allowed_public_keys {
                let wallet_set = key_to_wallets.entry(key.clone()).or_insert_with(HashSet::new);
                wallet_set.insert(wallet.id.clone());
            }
        }
        
        // ウォレットを更新
        wallets.insert(wallet.id.clone(), wallet);
        
        Ok(())
    }
    
    /// ウォレットを削除
    pub fn delete_wallet(&self, wallet_id: &WalletId) -> Result<(), Error> {
        let mut wallets = self.wallets.write().unwrap();
        
        // ウォレットが存在するか確認
        let wallet = wallets.get(wallet_id)
            .ok_or_else(|| Error::NotFound(format!("ウォレット {} が見つかりません", wallet_id)))?;
        
        // 公開鍵とウォレットのマッピングを更新
        let mut key_to_wallets = self.key_to_wallets.write().unwrap();
        for key in &wallet.policy.allowed_public_keys {
            if let Some(wallet_set) = key_to_wallets.get_mut(key) {
                wallet_set.remove(wallet_id);
                if wallet_set.is_empty() {
                    key_to_wallets.remove(key);
                }
            }
        }
        
        // ウォレットを削除
        wallets.remove(wallet_id);
        
        Ok(())
    }
    
    /// 全ウォレットを取得
    pub fn get_all_wallets(&self) -> Vec<EnhancedMultisigWallet> {
        let wallets = self.wallets.read().unwrap();
        wallets.values().cloned().collect()
    }
    
    /// 公開鍵に関連するウォレットを取得
    pub fn get_wallets_by_key(&self, public_key: &PublicKey) -> Vec<EnhancedMultisigWallet> {
        let key_to_wallets = self.key_to_wallets.read().unwrap();
        let wallets = self.wallets.read().unwrap();
        
        if let Some(wallet_ids) = key_to_wallets.get(public_key) {
            wallet_ids.iter()
                .filter_map(|id| wallets.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// アクティブなウォレットを取得
    pub fn get_active_wallets(&self) -> Vec<EnhancedMultisigWallet> {
        let wallets = self.wallets.read().unwrap();
        
        wallets.values()
            .filter(|wallet| wallet.status == WalletStatus::Active)
            .cloned()
            .collect()
    }
    
    /// タグでウォレットを検索
    pub fn find_wallets_by_tag(&self, tag: &str) -> Vec<EnhancedMultisigWallet> {
        let wallets = self.wallets.read().unwrap();
        
        wallets.values()
            .filter(|wallet| wallet.tags.contains(&tag.to_string()))
            .cloned()
            .collect()
    }
    
    /// トランザクションを作成
    pub fn create_transaction(&self, wallet_id: &WalletId, transaction: Transaction) -> Result<EnhancedMultisigTransaction, Error> {
        // ウォレットを取得
        let wallet = self.get_wallet(wallet_id)?;
        
        // ウォレットがアクティブかどうかを確認
        if !wallet.is_active() {
            return Err(Error::InvalidState(format!(
                "ウォレット {} はアクティブではありません",
                wallet_id
            )));
        }
        
        // 残高をチェック
        if transaction.amount > wallet.balance {
            return Err(Error::InsufficientFunds(format!(
                "残高不足: 必要 {}, 残高 {}",
                transaction.amount,
                wallet.balance
            )));
        }
        
        // 日次制限をチェック
        if !wallet.is_within_limit(transaction.amount) {
            return Err(Error::LimitExceeded("日次取引制限を超えています".to_string()));
        }
        
        // マルチシグトランザクションを作成
        let multisig_tx = EnhancedMultisigTransaction::new(wallet_id.clone(), transaction, wallet.policy.clone());
        
        // トランザクションを保存
        let mut transactions = self.transactions.write().unwrap();
        transactions.insert(multisig_tx.id.clone(), multisig_tx.clone());
        
        Ok(multisig_tx)
    }
    
    /// トランザクションを取得
    pub fn get_transaction(&self, transaction_id: &str) -> Result<EnhancedMultisigTransaction, Error> {
        let transactions = self.transactions.read().unwrap();
        
        transactions.get(transaction_id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("トランザクション {} が見つかりません", transaction_id)))
    }
    
    /// トランザクションを更新
    pub fn update_transaction(&self, transaction: EnhancedMultisigTransaction) -> Result<(), Error> {
        let mut transactions = self.transactions.write().unwrap();
        
        if !transactions.contains_key(&transaction.id) {
            return Err(Error::NotFound(format!("トランザクション {} が見つかりません", transaction.id)));
        }
        
        transactions.insert(transaction.id.clone(), transaction);
        
        Ok(())
    }
    
    /// トランザクションを削除
    pub fn delete_transaction(&self, transaction_id: &str) -> Result<(), Error> {
        let mut transactions = self.transactions.write().unwrap();
        
        if transactions.remove(transaction_id).is_none() {
            return Err(Error::NotFound(format!("トランザクション {} が見つかりません", transaction_id)));
        }
        
        Ok(())
    }
    
    /// ウォレットの全トランザクションを取得
    pub fn get_wallet_transactions(&self, wallet_id: &WalletId) -> Vec<EnhancedMultisigTransaction> {
        let transactions = self.transactions.read().unwrap();
        
        transactions.values()
            .filter(|tx| &tx.wallet_id == wallet_id)
            .cloned()
            .collect()
    }
    
    /// 署名を追加
    pub fn add_signature(&self, transaction_id: &str, public_key: PublicKey, signature: Signature) -> Result<EnhancedMultisigTransaction, Error> {
        // トランザクションを取得
        let mut transaction = self.get_transaction(transaction_id)?;
        
        // 署名を追加
        transaction.add_signature(public_key, signature)?;
        
        // トランザクションを更新
        self.update_transaction(transaction.clone())?;
        
        Ok(transaction)
    }
    
    /// トランザクションを実行
    pub fn execute_transaction(&self, transaction_id: &str) -> Result<EnhancedMultisigTransaction, Error> {
        // トランザクションを取得
        let mut transaction = self.get_transaction(transaction_id)?;
        
        // トランザクションを実行
        transaction.execute()?;
        
        // ウォレットの残高を更新
        let mut wallet = self.get_wallet(&transaction.wallet_id)?;
        wallet.decrease_balance(transaction.transaction.amount)?;
        self.update_wallet(wallet)?;
        
        // トランザクションを更新
        self.update_transaction(transaction.clone())?;
        
        Ok(transaction)
    }
    
    /// トランザクションを拒否
    pub fn reject_transaction(&self, transaction_id: &str, reason: Option<String>) -> Result<EnhancedMultisigTransaction, Error> {
        // トランザクションを取得
        let mut transaction = self.get_transaction(transaction_id)?;
        
        // トランザクションを拒否
        transaction.reject(reason)?;
        
        // トランザクションを更新
        self.update_transaction(transaction.clone())?;
        
        Ok(transaction)
    }
    
    /// トランザクションをキャンセル
    pub fn cancel_transaction(&self, transaction_id: &str) -> Result<EnhancedMultisigTransaction, Error> {
        // トランザクションを取得
        let mut transaction = self.get_transaction(transaction_id)?;
        
        // トランザクションをキャンセル
        transaction.cancel()?;
        
        // トランザクションを更新
        self.update_transaction(transaction.clone())?;
        
        Ok(transaction)
    }
    
    /// 期限切れのトランザクションをクリーンアップ
    pub fn cleanup_expired_transactions(&self) -> usize {
        let mut transactions = self.transactions.write().unwrap();
        let now = Utc::now();
        
        let mut expired_count = 0;
        
        // 期限切れのトランザクションを特定
        let expired_ids: Vec<String> = transactions.values()
            .filter(|tx| tx.check_expiration())
            .map(|tx| tx.id.clone())
            .collect();
        
        // 期限切れのトランザクションを更新
        for id in &expired_ids {
            if let Some(tx) = transactions.get_mut(id) {
                tx.state = MultisigTransactionState::Expired;
                tx.updated_at = now;
                expired_count += 1;
            }
        }
        
        expired_count
    }
    
    /// 日次制限をリセット
    pub fn reset_daily_limits(&self) -> usize {
        let mut wallets = self.wallets.write().unwrap();
        let mut reset_count = 0;
        
        for wallet in wallets.values_mut() {
            let before = wallet.remaining_daily_limit;
            wallet.reset_daily_limit_if_needed();
            let after = wallet.remaining_daily_limit;
            
            if before != after {
                reset_count += 1;
            }
        }
        
        reset_count
    }
    
    /// ウォレットの統計情報を取得
    pub fn get_wallet_stats(&self, wallet_id: &WalletId) -> Result<WalletStats, Error> {
        // ウォレットを取得
        let wallet = self.get_wallet(wallet_id)?;
        
        // トランザクションを取得
        let transactions = self.get_wallet_transactions(wallet_id);
        
        // 統計情報を計算
        let total_transactions = transactions.len();
        
        let pending_transactions = transactions.iter()
            .filter(|tx| tx.state == MultisigTransactionState::Created || tx.state == MultisigTransactionState::PartiallyApproved)
            .count();
        
        let executed_transactions = transactions.iter()
            .filter(|tx| tx.state == MultisigTransactionState::Executed)
            .count();
        
        let rejected_transactions = transactions.iter()
            .filter(|tx| tx.state == MultisigTransactionState::Rejected)
            .count();
        
        let expired_transactions = transactions.iter()
            .filter(|tx| tx.state == MultisigTransactionState::Expired)
            .count();
        
        let total_volume: u64 = transactions.iter()
            .filter(|tx| tx.state == MultisigTransactionState::Executed)
            .map(|tx| tx.transaction.amount)
            .sum();
        
        let avg_approval_time = if executed_transactions > 0 {
            let total_time: i64 = transactions.iter()
                .filter(|tx| tx.state == MultisigTransactionState::Executed && tx.executed_at.is_some())
                .map(|tx| (tx.executed_at.unwrap() - tx.created_at).num_seconds())
                .sum();
            
            Some(total_time / executed_transactions as i64)
        } else {
            None
        };
        
        Ok(WalletStats {
            wallet_id: wallet_id.clone(),
            balance: wallet.balance,
            total_transactions,
            pending_transactions,
            executed_transactions,
            rejected_transactions,
            expired_transactions,
            total_volume,
            avg_approval_time,
        })
    }
}

/// ウォレットの統計情報
#[derive(Debug, Clone)]
pub struct WalletStats {
    /// ウォレットID
    pub wallet_id: WalletId,
    /// 残高
    pub balance: u64,
    /// 総トランザクション数
    pub total_transactions: usize,
    /// 保留中のトランザクション数
    pub pending_transactions: usize,
    /// 実行済みのトランザクション数
    pub executed_transactions: usize,
    /// 拒否されたトランザクション数
    pub rejected_transactions: usize,
    /// 期限切れのトランザクション数
    pub expired_transactions: usize,
    /// 総取引量
    pub total_volume: u64,
    /// 平均承認時間（秒）
    pub avg_approval_time: Option<i64>,
}

/// マルチシグウォレットファクトリー
pub struct EnhancedMultisigFactory {
    /// マルチシグウォレットマネージャー
    manager: Arc<EnhancedMultisigManager>,
}

impl EnhancedMultisigFactory {
    /// 新しいマルチシグウォレットファクトリーを作成
    pub fn new(manager: Arc<EnhancedMultisigManager>) -> Self {
        Self { manager }
    }
    
    /// 2-of-3マルチシグウォレットを作成
    pub fn create_2of3_wallet(&self, name: String, key1: PublicKey, key2: PublicKey, key3: PublicKey) -> Result<EnhancedMultisigWallet, Error> {
        let policy = ThresholdPolicy::new(2, vec![key1, key2, key3]);
        self.manager.create_wallet(name, policy)
    }
    
    /// 3-of-5マルチシグウォレットを作成
    pub fn create_3of5_wallet(&self, name: String, keys: Vec<PublicKey>) -> Result<EnhancedMultisigWallet, Error> {
        if keys.len() != 5 {
            return Err(Error::InvalidInput("5つの公開鍵が必要です".to_string()));
        }
        
        let policy = ThresholdPolicy::new(3, keys);
        self.manager.create_wallet(name, policy)
    }
    
    /// 重み付きマルチシグウォレットを作成
    pub fn create_weighted_wallet(&self, name: String, keys: Vec<PublicKey>, weights: HashMap<PublicKey, u32>, threshold: u32) -> Result<EnhancedMultisigWallet, Error> {
        let policy = ThresholdPolicy::with_weights(keys, weights, threshold);
        self.manager.create_wallet(name, policy)
    }
    
    /// 有効期限付きマルチシグウォレットを作成
    pub fn create_time_locked_wallet(&self, name: String, required_signatures: usize, keys: Vec<PublicKey>, expiration: chrono::DateTime<Utc>) -> Result<EnhancedMultisigWallet, Error> {
        let policy = ThresholdPolicy::with_expiration(required_signatures, keys, expiration);
        self.manager.create_wallet(name, policy)
    }
    
    /// 日次制限付きウォレットを作成
    pub fn create_limited_wallet(&self, name: String, policy: ThresholdPolicy, daily_limit: u64) -> Result<EnhancedMultisigWallet, Error> {
        let mut wallet = self.manager.create_wallet(name, policy)?;
        wallet.set_daily_limit(daily_limit);
        self.manager.update_wallet(wallet.clone())?;
        Ok(wallet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_keypair;
    
    fn create_test_transaction() -> Transaction {
        Transaction {
            id: "tx1".to_string(),
            sender: "sender".to_string(),
            receiver: "receiver".to_string(),
            amount: 100,
            fee: 10,
            timestamp: Utc::now().timestamp(),
            signature: None,
            status: TransactionStatus::Pending,
            data: None,
        }
    }
    
    #[test]
    fn test_multisig_manager() {
        // キーペアを生成
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let keypair3 = generate_keypair();
        
        // 2-of-3ポリシーを作成
        let policy = ThresholdPolicy::new(
            2,
            vec![keypair1.public.clone(), keypair2.public.clone(), keypair3.public.clone()]
        );
        
        // マネージャーを作成
        let manager = EnhancedMultisigManager::new();
        
        // ウォレットを作成
        let wallet = manager.create_wallet("テストウォレット".to_string(), policy).unwrap();
        
        // ウォレットを取得
        let retrieved_wallet = manager.get_wallet(&wallet.id).unwrap();
        assert_eq!(retrieved_wallet.name, "テストウォレット");
        
        // 残高を更新
        let mut updated_wallet = retrieved_wallet.clone();
        updated_wallet.update_balance(1000);
        manager.update_wallet(updated_wallet).unwrap();
        
        // 更新されたウォレットを取得
        let wallet_with_balance = manager.get_wallet(&wallet.id).unwrap();
        assert_eq!(wallet_with_balance.balance, 1000);
        
        // トランザクションを作成
        let transaction = create_test_transaction();
        let multisig_tx = manager.create_transaction(&wallet.id, transaction).unwrap();
        
        // 署名を追加
        let tx1 = manager.add_signature(&multisig_tx.id, keypair1.public.clone(), "sig1".to_string()).unwrap();
        assert_eq!(tx1.state, MultisigTransactionState::PartiallyApproved);
        
        let tx2 = manager.add_signature(&multisig_tx.id, keypair2.public.clone(), "sig2".to_string()).unwrap();
        assert_eq!(tx2.state, MultisigTransactionState::Approved);
        
        // トランザクションを実行
        let tx3 = manager.execute_transaction(&multisig_tx.id).unwrap();
        assert_eq!(tx3.state, MultisigTransactionState::Executed);
        
        // 残高が更新されていることを確認
        let wallet_after_tx = manager.get_wallet(&wallet.id).unwrap();
        assert_eq!(wallet_after_tx.balance, 900); // 1000 - 100
        
        // ウォレットの統計情報を取得
        let stats = manager.get_wallet_stats(&wallet.id).unwrap();
        assert_eq!(stats.total_transactions, 1);
        assert_eq!(stats.executed_transactions, 1);
        assert_eq!(stats.total_volume, 100);
    }
    
    #[test]
    fn test_multisig_factory() {
        // キーペアを生成
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let keypair3 = generate_keypair();
        let keypair4 = generate_keypair();
        let keypair5 = generate_keypair();
        
        // マネージャーを作成
        let manager = Arc::new(EnhancedMultisigManager::new());
        
        // ファクトリーを作成
        let factory = EnhancedMultisigFactory::new(manager.clone());
        
        // 2-of-3ウォレットを作成
        let wallet_2of3 = factory.create_2of3_wallet(
            "2-of-3ウォレット".to_string(),
            keypair1.public.clone(),
            keypair2.public.clone(),
            keypair3.public.clone()
        ).unwrap();
        
        assert_eq!(wallet_2of3.name, "2-of-3ウォレット");
        assert_eq!(wallet_2of3.policy.required_signatures, 2);
        assert_eq!(wallet_2of3.policy.allowed_public_keys.len(), 3);
        
        // 3-of-5ウォレットを作成
        let wallet_3of5 = factory.create_3of5_wallet(
            "3-of-5ウォレット".to_string(),
            vec![
                keypair1.public.clone(),
                keypair2.public.clone(),
                keypair3.public.clone(),
                keypair4.public.clone(),
                keypair5.public.clone()
            ]
        ).unwrap();
        
        assert_eq!(wallet_3of5.name, "3-of-5ウォレット");
        assert_eq!(wallet_3of5.policy.required_signatures, 3);
        assert_eq!(wallet_3of5.policy.allowed_public_keys.len(), 5);
        
        // 重み付きウォレットを作成
        let mut weights = HashMap::new();
        weights.insert(keypair1.public.clone(), 3);
        weights.insert(keypair2.public.clone(), 2);
        weights.insert(keypair3.public.clone(), 1);
        
        let weighted_wallet = factory.create_weighted_wallet(
            "重み付きウォレット".to_string(),
            vec![keypair1.public.clone(), keypair2.public.clone(), keypair3.public.clone()],
            weights,
            4
        ).unwrap();
        
        assert_eq!(weighted_wallet.name, "重み付きウォレット");
        assert!(weighted_wallet.policy.weights.is_some());
        
        // 日次制限付きウォレットを作成
        let policy = ThresholdPolicy::new(
            1,
            vec![keypair1.public.clone()]
        );
        
        let limited_wallet = factory.create_limited_wallet(
            "制限付きウォレット".to_string(),
            policy,
            500
        ).unwrap();
        
        assert_eq!(limited_wallet.name, "制限付きウォレット");
        assert_eq!(limited_wallet.daily_limit, Some(500));
        assert_eq!(limited_wallet.remaining_daily_limit, Some(500));
    }
}