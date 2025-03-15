use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::transaction::{Transaction, TransactionStatus};

/// マルチシグウォレット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigWallet {
    /// ウォレットID
    pub id: String,
    /// ウォレット名
    pub name: String,
    /// 所有者アドレス
    pub owner_id: String,
    /// 署名者アドレス
    pub signers: Vec<String>,
    /// 必要な署名数
    pub required_signatures: usize,
    /// 残高
    pub balance: String,
    /// 作成日時
    pub created_at: u64,
    /// 最終更新日時
    pub updated_at: u64,
    /// ノンス
    pub nonce: u64,
}

/// マルチシグトランザクション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigTransaction {
    /// トランザクションID
    pub id: String,
    /// ウォレットID
    pub wallet_id: String,
    /// 送信先アドレス
    pub to: String,
    /// 金額
    pub amount: String,
    /// データ
    pub data: Option<String>,
    /// 署名
    pub signatures: HashMap<String, String>,
    /// 必要な署名数
    pub required_signatures: usize,
    /// 状態
    pub status: MultisigTransactionStatus,
    /// 作成者
    pub creator: String,
    /// 作成日時
    pub created_at: u64,
    /// 実行日時
    pub executed_at: Option<u64>,
    /// 有効期限
    pub expires_at: u64,
    /// ノンス
    pub nonce: u64,
}

/// マルチシグトランザクションの状態
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MultisigTransactionStatus {
    /// 保留中
    Pending,
    /// 実行済み
    Executed,
    /// 拒否
    Rejected,
    /// 期限切れ
    Expired,
}

/// マルチシグウォレットマネージャー
pub struct MultisigWalletManager {
    /// ウォレット
    wallets: Arc<Mutex<HashMap<String, MultisigWallet>>>,
    /// トランザクション
    transactions: Arc<Mutex<HashMap<String, MultisigTransaction>>>,
}

impl MultisigWalletManager {
    /// 新しいマルチシグウォレットマネージャーを作成
    pub fn new() -> Self {
        Self {
            wallets: Arc::new(Mutex::new(HashMap::new())),
            transactions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// マルチシグウォレットを作成
    pub fn create_wallet(
        &self,
        name: &str,
        owner_id: &str,
        signers: Vec<String>,
        required_signatures: usize,
    ) -> Result<MultisigWallet, Error> {
        // 入力を検証
        if name.is_empty() {
            return Err(Error::ValidationError("Wallet name is required".to_string()));
        }
        
        if signers.is_empty() {
            return Err(Error::ValidationError("At least one signer is required".to_string()));
        }
        
        if required_signatures == 0 || required_signatures > signers.len() {
            return Err(Error::ValidationError(format!(
                "Required signatures must be between 1 and {}",
                signers.len()
            )));
        }
        
        // 重複する署名者をチェック
        let unique_signers: HashSet<String> = signers.iter().cloned().collect();
        if unique_signers.len() != signers.len() {
            return Err(Error::ValidationError("Duplicate signers are not allowed".to_string()));
        }
        
        // ウォレットIDを生成
        let id = generate_wallet_id(owner_id, &signers, required_signatures);
        
        // 現在のタイムスタンプを取得
        let now = chrono::Utc::now().timestamp() as u64;
        
        // ウォレットを作成
        let wallet = MultisigWallet {
            id,
            name: name.to_string(),
            owner_id: owner_id.to_string(),
            signers,
            required_signatures,
            balance: "0".to_string(),
            created_at: now,
            updated_at: now,
            nonce: 0,
        };
        
        // ウォレットを保存
        let mut wallets = self.wallets.lock().unwrap();
        wallets.insert(wallet.id.clone(), wallet.clone());
        
        Ok(wallet)
    }
    
    /// マルチシグウォレットを取得
    pub fn get_wallet(&self, wallet_id: &str) -> Result<MultisigWallet, Error> {
        let wallets = self.wallets.lock().unwrap();
        
        wallets.get(wallet_id)
            .cloned()
            .ok_or_else(|| Error::ValidationError(format!("Wallet not found: {}", wallet_id)))
    }
    
    /// 所有者のマルチシグウォレットを取得
    pub fn get_wallets_by_owner(&self, owner_id: &str) -> Vec<MultisigWallet> {
        let wallets = self.wallets.lock().unwrap();
        
        wallets.values()
            .filter(|wallet| wallet.owner_id == owner_id)
            .cloned()
            .collect()
    }
    
    /// 署名者のマルチシグウォレットを取得
    pub fn get_wallets_by_signer(&self, signer: &str) -> Vec<MultisigWallet> {
        let wallets = self.wallets.lock().unwrap();
        
        wallets.values()
            .filter(|wallet| wallet.signers.contains(&signer.to_string()))
            .cloned()
            .collect()
    }
    
    /// マルチシグトランザクションを作成
    pub fn create_transaction(
        &self,
        wallet_id: &str,
        creator: &str,
        to: &str,
        amount: &str,
        data: Option<String>,
        signature: &str,
    ) -> Result<MultisigTransaction, Error> {
        // ウォレットを取得
        let mut wallets = self.wallets.lock().unwrap();
        
        let wallet = wallets.get_mut(wallet_id)
            .ok_or_else(|| Error::ValidationError(format!("Wallet not found: {}", wallet_id)))?;
        
        // 署名者をチェック
        if !wallet.signers.contains(&creator.to_string()) {
            return Err(Error::ValidationError(format!(
                "Creator is not a signer for this wallet: {}",
                creator
            )));
        }
        
        // 入力を検証
        if to.is_empty() {
            return Err(Error::ValidationError("Recipient address is required".to_string()));
        }
        
        if amount.is_empty() {
            return Err(Error::ValidationError("Amount is required".to_string()));
        }
        
        // 金額を検証
        let amount_value = amount.parse::<f64>()
            .map_err(|_| Error::ValidationError("Invalid amount".to_string()))?;
        
        if amount_value <= 0.0 {
            return Err(Error::ValidationError("Amount must be greater than 0".to_string()));
        }
        
        // 残高を検証
        let balance = wallet.balance.parse::<f64>()
            .map_err(|_| Error::ValidationError("Invalid wallet balance".to_string()))?;
        
        if amount_value > balance {
            return Err(Error::ValidationError("Insufficient balance".to_string()));
        }
        
        // 現在のタイムスタンプを取得
        let now = chrono::Utc::now().timestamp() as u64;
        
        // トランザクションIDを生成
        let id = generate_transaction_id(wallet_id, &to, amount, wallet.nonce);
        
        // 署名を検証
        let message = format!("{}:{}:{}:{}", wallet_id, to, amount, wallet.nonce);
        if !verify_signature(creator, &message, signature) {
            return Err(Error::ValidationError("Invalid signature".to_string()));
        }
        
        // 署名を保存
        let mut signatures = HashMap::new();
        signatures.insert(creator.to_string(), signature.to_string());
        
        // 有効期限を設定（1週間）
        let expires_at = now + 7 * 24 * 60 * 60;
        
        // トランザクションを作成
        let transaction = MultisigTransaction {
            id,
            wallet_id: wallet_id.to_string(),
            to: to.to_string(),
            amount: amount.to_string(),
            data,
            signatures,
            required_signatures: wallet.required_signatures,
            status: MultisigTransactionStatus::Pending,
            creator: creator.to_string(),
            created_at: now,
            executed_at: None,
            expires_at,
            nonce: wallet.nonce,
        };
        
        // ウォレットのノンスをインクリメント
        wallet.nonce += 1;
        wallet.updated_at = now;
        
        // トランザクションを保存
        let mut transactions = self.transactions.lock().unwrap();
        transactions.insert(transaction.id.clone(), transaction.clone());
        
        Ok(transaction)
    }
    
    /// マルチシグトランザクションに署名
    pub fn sign_transaction(
        &self,
        transaction_id: &str,
        signer: &str,
        signature: &str,
    ) -> Result<MultisigTransaction, Error> {
        // トランザクションを取得
        let mut transactions = self.transactions.lock().unwrap();
        
        let transaction = transactions.get_mut(transaction_id)
            .ok_or_else(|| Error::ValidationError(format!("Transaction not found: {}", transaction_id)))?;
        
        // トランザクションの状態をチェック
        if transaction.status != MultisigTransactionStatus::Pending {
            return Err(Error::ValidationError(format!(
                "Transaction is not pending: {}",
                transaction_id
            )));
        }
        
        // 有効期限をチェック
        let now = chrono::Utc::now().timestamp() as u64;
        if now > transaction.expires_at {
            transaction.status = MultisigTransactionStatus::Expired;
            return Err(Error::ValidationError(format!(
                "Transaction has expired: {}",
                transaction_id
            )));
        }
        
        // ウォレットを取得
        let wallets = self.wallets.lock().unwrap();
        
        let wallet = wallets.get(&transaction.wallet_id)
            .ok_or_else(|| Error::ValidationError(format!("Wallet not found: {}", transaction.wallet_id)))?;
        
        // 署名者をチェック
        if !wallet.signers.contains(&signer.to_string()) {
            return Err(Error::ValidationError(format!(
                "Signer is not authorized for this wallet: {}",
                signer
            )));
        }
        
        // 既に署名済みかチェック
        if transaction.signatures.contains_key(signer) {
            return Err(Error::ValidationError(format!(
                "Signer has already signed this transaction: {}",
                signer
            )));
        }
        
        // 署名を検証
        let message = format!("{}:{}:{}:{}", transaction.wallet_id, transaction.to, transaction.amount, transaction.nonce);
        if !verify_signature(signer, &message, signature) {
            return Err(Error::ValidationError("Invalid signature".to_string()));
        }
        
        // 署名を追加
        transaction.signatures.insert(signer.to_string(), signature.to_string());
        
        // 必要な署名数に達したかチェック
        if transaction.signatures.len() >= transaction.required_signatures {
            // トランザクションを実行
            drop(transactions);
            drop(wallets);
            
            match self.execute_transaction(transaction_id) {
                Ok(executed_tx) => return Ok(executed_tx),
                Err(e) => {
                    error!("Failed to execute transaction: {}", e);
                    // エラーが発生した場合でも、署名は追加されたトランザクションを返す
                    let transactions = self.transactions.lock().unwrap();
                    return Ok(transactions.get(transaction_id).unwrap().clone());
                }
            }
        }
        
        Ok(transaction.clone())
    }
    
    /// マルチシグトランザクションを実行
    pub fn execute_transaction(&self, transaction_id: &str) -> Result<MultisigTransaction, Error> {
        // トランザクションを取得
        let mut transactions = self.transactions.lock().unwrap();
        
        let transaction = transactions.get_mut(transaction_id)
            .ok_or_else(|| Error::ValidationError(format!("Transaction not found: {}", transaction_id)))?;
        
        // トランザクションの状態をチェック
        if transaction.status != MultisigTransactionStatus::Pending {
            return Err(Error::ValidationError(format!(
                "Transaction is not pending: {}",
                transaction_id
            )));
        }
        
        // 有効期限をチェック
        let now = chrono::Utc::now().timestamp() as u64;
        if now > transaction.expires_at {
            transaction.status = MultisigTransactionStatus::Expired;
            return Err(Error::ValidationError(format!(
                "Transaction has expired: {}",
                transaction_id
            )));
        }
        
        // 必要な署名数をチェック
        if transaction.signatures.len() < transaction.required_signatures {
            return Err(Error::ValidationError(format!(
                "Not enough signatures: {}/{}",
                transaction.signatures.len(),
                transaction.required_signatures
            )));
        }
        
        // ウォレットを取得
        let mut wallets = self.wallets.lock().unwrap();
        
        let wallet = wallets.get_mut(&transaction.wallet_id)
            .ok_or_else(|| Error::ValidationError(format!("Wallet not found: {}", transaction.wallet_id)))?;
        
        // 残高をチェック
        let amount = transaction.amount.parse::<f64>()
            .map_err(|_| Error::ValidationError("Invalid amount".to_string()))?;
        
        let balance = wallet.balance.parse::<f64>()
            .map_err(|_| Error::ValidationError("Invalid wallet balance".to_string()))?;
        
        if amount > balance {
            return Err(Error::ValidationError("Insufficient balance".to_string()));
        }
        
        // 残高を更新
        let new_balance = balance - amount;
        wallet.balance = format!("{:.8}", new_balance);
        wallet.updated_at = now;
        
        // トランザクションを実行済みに更新
        transaction.status = MultisigTransactionStatus::Executed;
        transaction.executed_at = Some(now);
        
        // 実際のトランザクションを作成する処理はここに実装
        // （ブロックチェーンへのトランザクション送信など）
        
        Ok(transaction.clone())
    }
    
    /// マルチシグトランザクションを拒否
    pub fn reject_transaction(
        &self,
        transaction_id: &str,
        signer: &str,
    ) -> Result<MultisigTransaction, Error> {
        // トランザクションを取得
        let mut transactions = self.transactions.lock().unwrap();
        
        let transaction = transactions.get_mut(transaction_id)
            .ok_or_else(|| Error::ValidationError(format!("Transaction not found: {}", transaction_id)))?;
        
        // トランザクションの状態をチェック
        if transaction.status != MultisigTransactionStatus::Pending {
            return Err(Error::ValidationError(format!(
                "Transaction is not pending: {}",
                transaction_id
            )));
        }
        
        // ウォレットを取得
        let wallets = self.wallets.lock().unwrap();
        
        let wallet = wallets.get(&transaction.wallet_id)
            .ok_or_else(|| Error::ValidationError(format!("Wallet not found: {}", transaction.wallet_id)))?;
        
        // 署名者をチェック
        if !wallet.signers.contains(&signer.to_string()) {
            return Err(Error::ValidationError(format!(
                "Signer is not authorized for this wallet: {}",
                signer
            )));
        }
        
        // トランザクションを拒否
        transaction.status = MultisigTransactionStatus::Rejected;
        
        Ok(transaction.clone())
    }
    
    /// マルチシグトランザクションを取得
    pub fn get_transaction(&self, transaction_id: &str) -> Result<MultisigTransaction, Error> {
        let transactions = self.transactions.lock().unwrap();
        
        transactions.get(transaction_id)
            .cloned()
            .ok_or_else(|| Error::ValidationError(format!("Transaction not found: {}", transaction_id)))
    }
    
    /// ウォレットのトランザクションを取得
    pub fn get_wallet_transactions(&self, wallet_id: &str) -> Vec<MultisigTransaction> {
        let transactions = self.transactions.lock().unwrap();
        
        transactions.values()
            .filter(|tx| tx.wallet_id == wallet_id)
            .cloned()
            .collect()
    }
    
    /// 署名者のトランザクションを取得
    pub fn get_signer_transactions(&self, signer: &str) -> Vec<MultisigTransaction> {
        // 署名者のウォレットを取得
        let signer_wallets = self.get_wallets_by_signer(signer);
        let wallet_ids: HashSet<String> = signer_wallets.iter().map(|w| w.id.clone()).collect();
        
        let transactions = self.transactions.lock().unwrap();
        
        transactions.values()
            .filter(|tx| wallet_ids.contains(&tx.wallet_id))
            .cloned()
            .collect()
    }
    
    /// 保留中のトランザクションをクリーンアップ
    pub fn cleanup_expired_transactions(&self) -> usize {
        let mut transactions = self.transactions.lock().unwrap();
        let now = chrono::Utc::now().timestamp() as u64;
        
        let mut expired_count = 0;
        
        for tx in transactions.values_mut() {
            if tx.status == MultisigTransactionStatus::Pending && now > tx.expires_at {
                tx.status = MultisigTransactionStatus::Expired;
                expired_count += 1;
            }
        }
        
        expired_count
    }
    
    /// ウォレットの残高を更新
    pub fn update_wallet_balance(&self, wallet_id: &str, balance: &str) -> Result<MultisigWallet, Error> {
        let mut wallets = self.wallets.lock().unwrap();
        
        let wallet = wallets.get_mut(wallet_id)
            .ok_or_else(|| Error::ValidationError(format!("Wallet not found: {}", wallet_id)))?;
        
        // 残高を検証
        let _ = balance.parse::<f64>()
            .map_err(|_| Error::ValidationError("Invalid balance".to_string()))?;
        
        wallet.balance = balance.to_string();
        wallet.updated_at = chrono::Utc::now().timestamp() as u64;
        
        Ok(wallet.clone())
    }
}

/// ウォレットIDを生成
fn generate_wallet_id(owner_id: &str, signers: &[String], required_signatures: usize) -> String {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(owner_id.as_bytes());
    
    for signer in signers {
        hasher.update(signer.as_bytes());
    }
    
    hasher.update(required_signatures.to_string().as_bytes());
    hasher.update(chrono::Utc::now().timestamp().to_string().as_bytes());
    
    let result = hasher.finalize();
    format!("mw{}", hex::encode(result))
}

/// トランザクションIDを生成
fn generate_transaction_id(wallet_id: &str, to: &str, amount: &str, nonce: u64) -> String {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(wallet_id.as_bytes());
    hasher.update(to.as_bytes());
    hasher.update(amount.as_bytes());
    hasher.update(nonce.to_string().as_bytes());
    hasher.update(chrono::Utc::now().timestamp().to_string().as_bytes());
    
    let result = hasher.finalize();
    format!("mt{}", hex::encode(result))
}

/// 署名を検証
fn verify_signature(signer: &str, message: &str, signature: &str) -> bool {
    // 実際の実装では、公開鍵暗号を使用して署名を検証
    // ここでは簡易的な実装として、常にtrueを返す
    true
}