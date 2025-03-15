use crate::transaction::{Transaction, TransactionStatus};
use crate::wallet::{Account, WalletManager};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// マルチシグ署名の状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureStatus {
    /// 署名待ち
    Pending,
    /// 署名済み
    Signed,
    /// 拒否
    Rejected,
}

/// マルチシグ署名
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    /// 署名者のアカウントID
    pub signer_id: String,
    /// 署名データ
    pub signature: Vec<u8>,
    /// 署名状態
    pub status: SignatureStatus,
    /// 署名日時
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// マルチシグトランザクション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigTransaction {
    /// トランザクションID
    pub id: String,
    /// マルチシグウォレットID
    pub wallet_id: String,
    /// 作成者のアカウントID
    pub creator_id: String,
    /// 必要な署名数
    pub required_signatures: usize,
    /// 署名のマップ（アカウントID -> 署名）
    pub signatures: HashMap<String, Signature>,
    /// トランザクションデータ
    pub transaction_data: Vec<u8>,
    /// 状態
    pub status: TransactionStatus,
    /// 作成日時
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 実行日時
    pub executed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl MultisigTransaction {
    /// 新しいマルチシグトランザクションを作成
    pub fn new(
        wallet_id: String,
        creator_id: String,
        required_signatures: usize,
        transaction_data: Vec<u8>,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        
        let mut signatures = HashMap::new();
        signatures.insert(
            creator_id.clone(),
            Signature {
                signer_id: creator_id.clone(),
                signature: Vec::new(), // 初期状態では空の署名
                status: SignatureStatus::Pending,
                timestamp: now,
            },
        );
        
        Self {
            id,
            wallet_id,
            creator_id,
            required_signatures,
            signatures,
            transaction_data,
            status: TransactionStatus::Pending,
            created_at: now,
            executed_at: None,
        }
    }
    
    /// 署名を追加
    pub fn add_signature(&mut self, signer_id: &str, signature: Vec<u8>) -> Result<(), String> {
        if !self.signatures.contains_key(signer_id) {
            return Err(format!("Signer {} is not authorized for this transaction", signer_id));
        }
        
        if self.status != TransactionStatus::Pending {
            return Err(format!("Transaction is not in pending state: {:?}", self.status));
        }
        
        let sig = self.signatures.get_mut(signer_id).unwrap();
        sig.signature = signature;
        sig.status = SignatureStatus::Signed;
        sig.timestamp = chrono::Utc::now();
        
        Ok(())
    }
    
    /// 署名を拒否
    pub fn reject_signature(&mut self, signer_id: &str) -> Result<(), String> {
        if !self.signatures.contains_key(signer_id) {
            return Err(format!("Signer {} is not authorized for this transaction", signer_id));
        }
        
        if self.status != TransactionStatus::Pending {
            return Err(format!("Transaction is not in pending state: {:?}", self.status));
        }
        
        let sig = self.signatures.get_mut(signer_id).unwrap();
        sig.status = SignatureStatus::Rejected;
        sig.timestamp = chrono::Utc::now();
        
        // 一人でも拒否したら、トランザクションは拒否
        self.status = TransactionStatus::Rejected;
        
        Ok(())
    }
    
    /// 署名が十分かどうかを確認
    pub fn has_enough_signatures(&self) -> bool {
        let signed_count = self.signatures
            .values()
            .filter(|sig| sig.status == SignatureStatus::Signed)
            .count();
        
        signed_count >= self.required_signatures
    }
    
    /// トランザクションを実行
    pub fn execute(&mut self) -> Result<(), String> {
        if self.status != TransactionStatus::Pending {
            return Err(format!("Transaction is not in pending state: {:?}", self.status));
        }
        
        if !self.has_enough_signatures() {
            return Err(format!(
                "Not enough signatures: {}/{} required",
                self.signatures.values().filter(|sig| sig.status == SignatureStatus::Signed).count(),
                self.required_signatures
            ));
        }
        
        self.status = TransactionStatus::Confirmed;
        self.executed_at = Some(chrono::Utc::now());
        
        Ok(())
    }
}

/// マルチシグウォレット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigWallet {
    /// ウォレットID
    pub id: String,
    /// ウォレット名
    pub name: String,
    /// 所有者のアカウントID
    pub owner_id: String,
    /// 署名者のアカウントID
    pub signers: Vec<String>,
    /// 必要な署名数
    pub required_signatures: usize,
    /// 残高
    pub balance: f64,
    /// トークン残高
    pub token_balances: HashMap<String, f64>,
    /// 作成日時
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl MultisigWallet {
    /// 新しいマルチシグウォレットを作成
    pub fn new(
        name: String,
        owner_id: String,
        signers: Vec<String>,
        required_signatures: usize,
    ) -> Result<Self, String> {
        // 署名者数のバリデーション
        if signers.is_empty() {
            return Err("At least one signer is required".to_string());
        }
        
        if required_signatures > signers.len() {
            return Err(format!(
                "Required signatures ({}) cannot exceed the number of signers ({})",
                required_signatures, signers.len()
            ));
        }
        
        if required_signatures == 0 {
            return Err("At least one signature is required".to_string());
        }
        
        // 署名者の重複チェック
        let unique_signers: HashSet<String> = signers.iter().cloned().collect();
        if unique_signers.len() != signers.len() {
            return Err("Duplicate signers are not allowed".to_string());
        }
        
        // 所有者が署名者に含まれているか確認
        if !signers.contains(&owner_id) {
            return Err("Owner must be included in the signers list".to_string());
        }
        
        let id = Uuid::new_v4().to_string();
        
        Ok(Self {
            id,
            name,
            owner_id,
            signers,
            required_signatures,
            balance: 0.0,
            token_balances: HashMap::new(),
            created_at: chrono::Utc::now(),
        })
    }
    
    /// 署名者を追加
    pub fn add_signer(&mut self, signer_id: String) -> Result<(), String> {
        if self.signers.contains(&signer_id) {
            return Err(format!("Signer {} is already in the wallet", signer_id));
        }
        
        self.signers.push(signer_id);
        Ok(())
    }
    
    /// 署名者を削除
    pub fn remove_signer(&mut self, signer_id: &str) -> Result<(), String> {
        // 所有者は削除できない
        if signer_id == self.owner_id {
            return Err("Cannot remove the owner from signers".to_string());
        }
        
        let position = self.signers.iter().position(|id| id == signer_id);
        if let Some(pos) = position {
            self.signers.remove(pos);
            
            // 署名者が減った場合、必要署名数が署名者数を超えないようにする
            if self.required_signatures > self.signers.len() {
                self.required_signatures = self.signers.len();
            }
            
            Ok(())
        } else {
            Err(format!("Signer {} not found", signer_id))
        }
    }
    
    /// 必要署名数を変更
    pub fn change_required_signatures(&mut self, required: usize) -> Result<(), String> {
        if required == 0 {
            return Err("At least one signature is required".to_string());
        }
        
        if required > self.signers.len() {
            return Err(format!(
                "Required signatures ({}) cannot exceed the number of signers ({})",
                required, self.signers.len()
            ));
        }
        
        self.required_signatures = required;
        Ok(())
    }
    
    /// トークン残高を更新
    pub fn update_token_balance(&mut self, token_id: &str, amount: f64) {
        let balance = self.token_balances.entry(token_id.to_string()).or_insert(0.0);
        *balance += amount;
    }
}

/// マルチシグマネージャー
pub struct MultisigManager {
    /// ウォレットのマップ
    wallets: Mutex<HashMap<String, MultisigWallet>>,
    /// トランザクションのマップ
    transactions: Mutex<HashMap<String, MultisigTransaction>>,
    /// ウォレットマネージャーの参照
    wallet_manager: Arc<WalletManager>,
}

impl MultisigManager {
    /// 新しいMultisigManagerを作成
    pub fn new(wallet_manager: Arc<WalletManager>) -> Self {
        Self {
            wallets: Mutex::new(HashMap::new()),
            transactions: Mutex::new(HashMap::new()),
            wallet_manager,
        }
    }
    
    /// マルチシグウォレットを作成
    pub fn create_wallet(
        &self,
        name: String,
        owner_id: &str,
        signers: Vec<String>,
        required_signatures: usize,
    ) -> Result<MultisigWallet, String> {
        // 所有者アカウントの存在確認
        if self.wallet_manager.get_account(owner_id).is_none() {
            return Err(format!("Owner account {} not found", owner_id));
        }
        
        // 署名者アカウントの存在確認
        for signer_id in &signers {
            if self.wallet_manager.get_account(signer_id).is_none() {
                return Err(format!("Signer account {} not found", signer_id));
            }
        }
        
        // ウォレットを作成
        let wallet = MultisigWallet::new(name, owner_id.to_string(), signers, required_signatures)?;
        
        // ウォレットを保存
        let wallet_id = wallet.id.clone();
        let mut wallets = self.wallets.lock().unwrap();
        wallets.insert(wallet_id, wallet.clone());
        
        info!("Multisig wallet created: {}", wallet.id);
        Ok(wallet)
    }
    
    /// マルチシグウォレットを取得
    pub fn get_wallet(&self, wallet_id: &str) -> Option<MultisigWallet> {
        let wallets = self.wallets.lock().unwrap();
        wallets.get(wallet_id).cloned()
    }
    
    /// アカウントに関連するマルチシグウォレットを取得
    pub fn get_wallets_by_account(&self, account_id: &str) -> Vec<MultisigWallet> {
        let wallets = self.wallets.lock().unwrap();
        wallets
            .values()
            .filter(|wallet| wallet.signers.contains(&account_id.to_string()))
            .cloned()
            .collect()
    }
    
    /// マルチシグトランザクションを作成
    pub fn create_transaction(
        &self,
        wallet_id: &str,
        creator_id: &str,
        transaction_data: Vec<u8>,
    ) -> Result<MultisigTransaction, String> {
        // ウォレットの存在確認
        let wallet = self.get_wallet(wallet_id)
            .ok_or_else(|| format!("Wallet {} not found", wallet_id))?;
        
        // 作成者が署名者かどうか確認
        if !wallet.signers.contains(&creator_id.to_string()) {
            return Err(format!("Creator {} is not a signer of wallet {}", creator_id, wallet_id));
        }
        
        // トランザクションを作成
        let tx = MultisigTransaction::new(
            wallet_id.to_string(),
            creator_id.to_string(),
            wallet.required_signatures,
            transaction_data,
        );
        
        // トランザクションを保存
        let tx_id = tx.id.clone();
        let mut transactions = self.transactions.lock().unwrap();
        transactions.insert(tx_id, tx.clone());
        
        info!("Multisig transaction created: {}", tx.id);
        Ok(tx)
    }
    
    /// マルチシグトランザクションを取得
    pub fn get_transaction(&self, tx_id: &str) -> Option<MultisigTransaction> {
        let transactions = self.transactions.lock().unwrap();
        transactions.get(tx_id).cloned()
    }
    
    /// ウォレットに関連するマルチシグトランザクションを取得
    pub fn get_transactions_by_wallet(&self, wallet_id: &str) -> Vec<MultisigTransaction> {
        let transactions = self.transactions.lock().unwrap();
        transactions
            .values()
            .filter(|tx| tx.wallet_id == wallet_id)
            .cloned()
            .collect()
    }
    
    /// アカウントに関連するマルチシグトランザクションを取得
    pub fn get_transactions_by_account(&self, account_id: &str) -> Vec<MultisigTransaction> {
        let wallets = self.get_wallets_by_account(account_id);
        let wallet_ids: HashSet<String> = wallets.iter().map(|w| w.id.clone()).collect();
        
        let transactions = self.transactions.lock().unwrap();
        transactions
            .values()
            .filter(|tx| wallet_ids.contains(&tx.wallet_id) && tx.signatures.contains_key(account_id))
            .cloned()
            .collect()
    }
    
    /// トランザクションに署名
    pub fn sign_transaction(
        &self,
        tx_id: &str,
        signer_id: &str,
        signature: Vec<u8>,
    ) -> Result<MultisigTransaction, String> {
        // 入力検証
        if tx_id.is_empty() {
            return Err("Transaction ID cannot be empty".to_string());
        }
        
        if signer_id.is_empty() {
            return Err("Signer ID cannot be empty".to_string());
        }
        
        if signature.is_empty() {
            return Err("Signature cannot be empty".to_string());
        }
        
        // レート制限チェック
        self.check_rate_limit(signer_id)?;
        
        // トランザクションロックを取得
        let tx_lock = self.get_transaction_lock(tx_id)?;
        let _guard = tx_lock.lock().unwrap();
        
        // トランザクションの存在確認
        let mut transactions = self.transactions.lock().unwrap();
        let tx = transactions.get_mut(tx_id)
            .ok_or_else(|| format!("Transaction {} not found", tx_id))?;
        
        // ウォレットの存在確認
        let wallets = self.wallets.lock().unwrap();
        let wallet = wallets.get(&tx.wallet_id)
            .ok_or_else(|| format!("Wallet {} not found", tx.wallet_id))?;
        
        // 署名者がウォレットの署名者リストに含まれているか確認
        if !wallet.signers.contains(&signer_id.to_string()) {
            // セキュリティログを記録
            warn!("Unauthorized signature attempt: signer={}, tx={}", signer_id, tx_id);
            return Err(format!("Signer {} is not authorized for this transaction", signer_id));
        }
        
        // 署名の検証
        self.verify_signature(signer_id, tx, &signature)?;
        
        // 署名を追加
        tx.add_signature(signer_id, signature)?;
        
        // 署名イベントを記録
        info!("Transaction signed: tx={}, signer={}, wallet={}", tx_id, signer_id, wallet.id);
        
        // 署名が十分な場合、トランザクションを実行
        if tx.has_enough_signatures() {
            // 二重実行防止のためのチェック
            if tx.executed_at.is_some() {
                warn!("Transaction already executed: tx={}", tx_id);
                return Ok(tx.clone());
            }
            
            // トランザクションを実行
            match tx.execute() {
                Ok(_) => {
                    info!("Transaction executed successfully: tx={}", tx_id);
                    
                    // 実際のトランザクションを処理
                    match self.process_transaction(tx) {
                        Ok(_) => {},
                        Err(e) => {
                            error!("Transaction processing failed: tx={}, error={}", tx_id, e);
                            return Err(e);
                        }
                    }
                },
                Err(e) => {
                    error!("Transaction execution failed: tx={}, error={}", tx_id, e);
                    return Err(e);
                }
            }
        }
        
        Ok(tx.clone())
    }
    
    /// 署名の検証
    fn verify_signature(&self, signer_id: &str, tx: &MultisigTransaction, signature: &[u8]) -> Result<(), String> {
        // アカウントを取得
        let account = self.wallet_manager.get_account(signer_id)
            .ok_or_else(|| format!("Account {} not found", signer_id))?;
        
        // 実際の実装では、公開鍵を使用して署名を検証
        // ここでは簡略化のため、署名の長さだけチェック
        if signature.len() < 32 {
            warn!("Invalid signature format: signer={}, tx={}", signer_id, tx.id);
            return Err("Signature too short".to_string());
        }
        
        // TODO: 実際の署名検証ロジックを実装
        // let message = tx.id.as_bytes();
        // let public_key = hex::decode(&account.public_key)?;
        // let is_valid = crypto::verify(message, signature, &public_key)?;
        // if !is_valid {
        //     return Err("Signature verification failed".to_string());
        // }
        
        Ok(())
    }
    
    /// レート制限チェック
    fn check_rate_limit(&self, user_id: &str) -> Result<(), String> {
        // レート制限の実装
        // 実際の実装では、ユーザーごとの操作回数を追跡し、一定期間内の操作回数を制限
        
        // 簡略化のため、ここではダミー実装
        Ok(())
    }
    
    /// トランザクションロックを取得
    fn get_transaction_lock(&self, tx_id: &str) -> Result<Arc<Mutex<()>>, String> {
        // トランザクションごとのロックを管理
        // 同じトランザクションに対する並行操作を防止
        
        // 簡略化のため、ここではダミー実装
        Ok(Arc::new(Mutex::new(())))
    }
    
    /// トランザクションを拒否
    pub fn reject_transaction(
        &self,
        tx_id: &str,
        signer_id: &str,
    ) -> Result<MultisigTransaction, String> {
        // トランザクションの存在確認
        let mut transactions = self.transactions.lock().unwrap();
        let tx = transactions.get_mut(tx_id)
            .ok_or_else(|| format!("Transaction {} not found", tx_id))?;
        
        // 署名を拒否
        tx.reject_signature(signer_id)?;
        
        info!("Transaction {} rejected by {}", tx_id, signer_id);
        Ok(tx.clone())
    }
    
    /// トランザクションを処理
    fn process_transaction(&self, tx: &mut MultisigTransaction) -> Result<(), String> {
        // トランザクションデータをデシリアライズ
        let tx_data: TransactionData = serde_json::from_slice(&tx.transaction_data)
            .map_err(|e| format!("Failed to deserialize transaction data: {}", e))?;
        
        // ウォレットの存在確認
        let mut wallets = self.wallets.lock().unwrap();
        let wallet = wallets.get_mut(&tx.wallet_id)
            .ok_or_else(|| format!("Wallet {} not found", tx.wallet_id))?;
        
        match tx_data.operation {
            // 送金操作
            Operation::Transfer { to, amount, token_id } => {
                // 残高確認
                if let Some(token) = &token_id {
                    let balance = wallet.token_balances.get(token).unwrap_or(&0.0);
                    if *balance < amount {
                        return Err(format!("Insufficient token balance: {} < {}", balance, amount));
                    }
                    
                    // トークン残高を更新
                    wallet.update_token_balance(token, -amount);
                    
                    // 送信先アカウントを取得
                    if let Some(to_account) = self.wallet_manager.get_account(&to) {
                        // 送信先アカウントのトークン残高を更新
                        let mut to_account = to_account;
                        to_account.update_token_balance(token, amount);
                    } else {
                        // 送信先がマルチシグウォレットの場合
                        if let Some(to_wallet) = wallets.get_mut(&to) {
                            to_wallet.update_token_balance(token, amount);
                        } else {
                            return Err(format!("Recipient {} not found", to));
                        }
                    }
                } else {
                    if wallet.balance < amount {
                        return Err(format!("Insufficient balance: {} < {}", wallet.balance, amount));
                    }
                    
                    // 残高を更新
                    wallet.balance -= amount;
                    
                    // 送信先アカウントを取得
                    if let Some(to_account) = self.wallet_manager.get_account(&to) {
                        // 送信先アカウントの残高を更新
                        let mut to_account = to_account;
                        to_account.balance += amount;
                    } else {
                        // 送信先がマルチシグウォレットの場合
                        if let Some(to_wallet) = wallets.get_mut(&to) {
                            to_wallet.balance += amount;
                        } else {
                            return Err(format!("Recipient {} not found", to));
                        }
                    }
                }
                
                info!("Transfer executed: {} from wallet {} to {}", amount, tx.wallet_id, to);
            },
            
            // 署名者追加操作
            Operation::AddSigner { signer_id } => {
                wallet.add_signer(signer_id.clone())?;
                info!("Signer {} added to wallet {}", signer_id, tx.wallet_id);
            },
            
            // 署名者削除操作
            Operation::RemoveSigner { signer_id } => {
                wallet.remove_signer(&signer_id)?;
                info!("Signer {} removed from wallet {}", signer_id, tx.wallet_id);
            },
            
            // 必要署名数変更操作
            Operation::ChangeRequiredSignatures { required } => {
                wallet.change_required_signatures(required)?;
                info!("Required signatures changed to {} for wallet {}", required, tx.wallet_id);
            },
        }
        
        Ok(())
    }
}

/// トランザクション操作
#[derive(Debug, Serialize, Deserialize)]
pub enum Operation {
    /// 送金
    Transfer {
        /// 送信先ID（アカウントまたはウォレット）
        to: String,
        /// 金額
        amount: f64,
        /// トークンID（Noneの場合はネイティブトークン）
        token_id: Option<String>,
    },
    /// 署名者追加
    AddSigner {
        /// 追加する署名者のアカウントID
        signer_id: String,
    },
    /// 署名者削除
    RemoveSigner {
        /// 削除する署名者のアカウントID
        signer_id: String,
    },
    /// 必要署名数変更
    ChangeRequiredSignatures {
        /// 新しい必要署名数
        required: usize,
    },
}

/// トランザクションデータ
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionData {
    /// 操作
    pub operation: Operation,
    /// メモ
    pub memo: Option<String>,
    /// タイムスタンプ
    pub timestamp: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::Account;
    
    fn create_test_account(id: &str, name: &str) -> Account {
        Account {
            id: id.to_string(),
            public_key: format!("pub_{}", id),
            private_key: format!("priv_{}", id),
            name: name.to_string(),
            balance: 1000.0,
            token_balances: HashMap::new(),
            created_at: chrono::Utc::now(),
        }
    }
    
    #[test]
    fn test_multisig_wallet_creation() {
        let wallet = MultisigWallet::new(
            "Test Wallet".to_string(),
            "owner1".to_string(),
            vec!["owner1".to_string(), "signer2".to_string(), "signer3".to_string()],
            2,
        );
        
        assert!(wallet.is_ok());
        let wallet = wallet.unwrap();
        
        assert_eq!(wallet.name, "Test Wallet");
        assert_eq!(wallet.owner_id, "owner1");
        assert_eq!(wallet.signers.len(), 3);
        assert_eq!(wallet.required_signatures, 2);
        assert_eq!(wallet.balance, 0.0);
        assert!(wallet.token_balances.is_empty());
    }
    
    #[test]
    fn test_multisig_wallet_validation() {
        // 署名者が空の場合
        let wallet = MultisigWallet::new(
            "Test Wallet".to_string(),
            "owner1".to_string(),
            vec![],
            1,
        );
        assert!(wallet.is_err());
        
        // 必要署名数が署名者数を超える場合
        let wallet = MultisigWallet::new(
            "Test Wallet".to_string(),
            "owner1".to_string(),
            vec!["owner1".to_string(), "signer2".to_string()],
            3,
        );
        assert!(wallet.is_err());
        
        // 必要署名数が0の場合
        let wallet = MultisigWallet::new(
            "Test Wallet".to_string(),
            "owner1".to_string(),
            vec!["owner1".to_string(), "signer2".to_string()],
            0,
        );
        assert!(wallet.is_err());
        
        // 署名者に重複がある場合
        let wallet = MultisigWallet::new(
            "Test Wallet".to_string(),
            "owner1".to_string(),
            vec!["owner1".to_string(), "signer2".to_string(), "signer2".to_string()],
            2,
        );
        assert!(wallet.is_err());
        
        // 所有者が署名者に含まれていない場合
        let wallet = MultisigWallet::new(
            "Test Wallet".to_string(),
            "owner1".to_string(),
            vec!["signer2".to_string(), "signer3".to_string()],
            1,
        );
        assert!(wallet.is_err());
    }
    
    #[test]
    fn test_multisig_transaction() {
        // ウォレットを作成
        let wallet = MultisigWallet::new(
            "Test Wallet".to_string(),
            "owner1".to_string(),
            vec!["owner1".to_string(), "signer2".to_string(), "signer3".to_string()],
            2,
        ).unwrap();
        
        // トランザクションデータを作成
        let tx_data = TransactionData {
            operation: Operation::Transfer {
                to: "recipient".to_string(),
                amount: 100.0,
                token_id: None,
            },
            memo: Some("Test transfer".to_string()),
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        let tx_data_bytes = serde_json::to_vec(&tx_data).unwrap();
        
        // マルチシグトランザクションを作成
        let mut tx = MultisigTransaction::new(
            wallet.id.clone(),
            "owner1".to_string(),
            wallet.required_signatures,
            tx_data_bytes,
        );
        
        // 初期状態の確認
        assert_eq!(tx.status, TransactionStatus::Pending);
        assert_eq!(tx.signatures.len(), 1);
        assert!(tx.signatures.contains_key("owner1"));
        assert_eq!(tx.signatures["owner1"].status, SignatureStatus::Pending);
        
        // 署名を追加
        tx.add_signature("owner1", vec![1, 2, 3]).unwrap();
        assert_eq!(tx.signatures["owner1"].status, SignatureStatus::Signed);
        
        // 署名が不足している状態で実行
        let result = tx.execute();
        assert!(result.is_err());
        
        // 2人目の署名を追加
        tx.add_signature("signer2", vec![4, 5, 6]).unwrap();
        assert_eq!(tx.signatures["signer2"].status, SignatureStatus::Signed);
        
        // 署名が十分な状態で実行
        let result = tx.execute();
        assert!(result.is_ok());
        assert_eq!(tx.status, TransactionStatus::Confirmed);
        assert!(tx.executed_at.is_some());
    }
    
    #[test]
    fn test_multisig_transaction_rejection() {
        // ウォレットを作成
        let wallet = MultisigWallet::new(
            "Test Wallet".to_string(),
            "owner1".to_string(),
            vec!["owner1".to_string(), "signer2".to_string(), "signer3".to_string()],
            2,
        ).unwrap();
        
        // トランザクションデータを作成
        let tx_data = TransactionData {
            operation: Operation::Transfer {
                to: "recipient".to_string(),
                amount: 100.0,
                token_id: None,
            },
            memo: Some("Test transfer".to_string()),
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        let tx_data_bytes = serde_json::to_vec(&tx_data).unwrap();
        
        // マルチシグトランザクションを作成
        let mut tx = MultisigTransaction::new(
            wallet.id.clone(),
            "owner1".to_string(),
            wallet.required_signatures,
            tx_data_bytes,
        );
        
        // 署名を拒否
        tx.reject_signature("owner1").unwrap();
        assert_eq!(tx.signatures["owner1"].status, SignatureStatus::Rejected);
        assert_eq!(tx.status, TransactionStatus::Rejected);
        
        // 拒否された状態で署名を追加
        let result = tx.add_signature("signer2", vec![4, 5, 6]);
        assert!(result.is_err());
        
        // 拒否された状態で実行
        let result = tx.execute();
        assert!(result.is_err());
    }
}