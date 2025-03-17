use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::crypto::{PublicKey, Signature, KeyPair, hash};
use crate::transaction::{Transaction, TransactionType, TransactionStatus};
use crate::wallet::{Wallet, WalletType, WalletStatus};
use crate::storage::Storage;

/// マルチシグウォレット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigWallet {
    /// ウォレットID
    pub id: String,
    /// ウォレットアドレス
    pub address: String,
    /// 所有者の公開鍵
    pub owners: Vec<PublicKey>,
    /// 必要な署名数
    pub required_signatures: u32,
    /// 作成時刻
    pub created_at: DateTime<Utc>,
    /// 更新時刻
    pub updated_at: DateTime<Utc>,
    /// ステータス
    pub status: WalletStatus,
    /// 残高
    pub balance: u64,
    /// ノンス
    pub nonce: u64,
    /// 保留中のトランザクション
    pub pending_transactions: HashMap<String, MultisigTransaction>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// マルチシグトランザクション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigTransaction {
    /// トランザクションID
    pub id: String,
    /// トランザクション
    pub transaction: Transaction,
    /// 作成時刻
    pub created_at: DateTime<Utc>,
    /// 更新時刻
    pub updated_at: DateTime<Utc>,
    /// 署名
    pub signatures: HashMap<String, Signature>,
    /// 必要な署名数
    pub required_signatures: u32,
    /// ステータス
    pub status: MultisigTransactionStatus,
    /// 実行時刻
    pub executed_at: Option<DateTime<Utc>>,
    /// 有効期限
    pub expires_at: Option<DateTime<Utc>>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// マルチシグトランザクションステータス
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MultisigTransactionStatus {
    /// 保留中
    Pending,
    /// 実行可能
    Ready,
    /// 実行済み
    Executed,
    /// 失敗
    Failed,
    /// キャンセル
    Cancelled,
    /// 期限切れ
    Expired,
}

/// マルチシグウォレットマネージャー
pub struct MultisigWalletManager {
    /// ストレージ
    storage: Box<dyn Storage>,
    /// ウォレットキャッシュ
    wallet_cache: HashMap<String, MultisigWallet>,
}

impl MultisigWalletManager {
    /// 新しいマルチシグウォレットマネージャーを作成
    pub fn new(storage: Box<dyn Storage>) -> Self {
        Self {
            storage,
            wallet_cache: HashMap::new(),
        }
    }
    
    /// マルチシグウォレットを作成
    pub fn create_wallet(
        &mut self,
        owners: Vec<PublicKey>,
        required_signatures: u32,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<MultisigWallet, Error> {
        // 入力の検証
        if owners.is_empty() {
            return Err(Error::InvalidInput("Owners list cannot be empty".to_string()));
        }
        
        if required_signatures == 0 || required_signatures as usize > owners.len() {
            return Err(Error::InvalidInput(format!(
                "Required signatures must be between 1 and {}", owners.len()
            )));
        }
        
        // 重複する所有者を削除
        let unique_owners: Vec<PublicKey> = owners.clone().into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        
        if unique_owners.len() != owners.len() {
            warn!("Duplicate owners were removed from the wallet");
        }
        
        // ウォレットアドレスを生成
        let address = generate_multisig_address(&unique_owners, required_signatures);
        
        // ウォレットIDを生成
        let wallet_id = format!("msw_{}", Utc::now().timestamp_nanos());
        
        // マルチシグウォレットを作成
        let wallet = MultisigWallet {
            id: wallet_id.clone(),
            address: address.clone(),
            owners: unique_owners,
            required_signatures,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: WalletStatus::Active,
            balance: 0,
            nonce: 0,
            pending_transactions: HashMap::new(),
            metadata: metadata.unwrap_or_default(),
        };
        
        // ストレージに保存
        self.storage.save_multisig_wallet(&wallet)?;
        
        // キャッシュに追加
        self.wallet_cache.insert(wallet_id.clone(), wallet.clone());
        
        Ok(wallet)
    }
    
    /// ウォレットを取得
    pub fn get_wallet(&mut self, wallet_id: &str) -> Result<MultisigWallet, Error> {
        // キャッシュをチェック
        if let Some(wallet) = self.wallet_cache.get(wallet_id) {
            return Ok(wallet.clone());
        }
        
        // ストレージから取得
        let wallet = self.storage.get_multisig_wallet(wallet_id)?;
        
        // キャッシュに追加
        self.wallet_cache.insert(wallet_id.to_string(), wallet.clone());
        
        Ok(wallet)
    }
    
    /// ウォレットをアドレスで取得
    pub fn get_wallet_by_address(&mut self, address: &str) -> Result<MultisigWallet, Error> {
        // キャッシュをチェック
        for wallet in self.wallet_cache.values() {
            if wallet.address == address {
                return Ok(wallet.clone());
            }
        }
        
        // ストレージから取得
        let wallet = self.storage.get_multisig_wallet_by_address(address)?;
        
        // キャッシュに追加
        self.wallet_cache.insert(wallet.id.clone(), wallet.clone());
        
        Ok(wallet)
    }
    
    /// ウォレットを更新
    pub fn update_wallet(&mut self, wallet: &MultisigWallet) -> Result<(), Error> {
        // ストレージに保存
        self.storage.save_multisig_wallet(wallet)?;
        
        // キャッシュを更新
        self.wallet_cache.insert(wallet.id.clone(), wallet.clone());
        
        Ok(())
    }
    
    /// ウォレットを削除
    pub fn delete_wallet(&mut self, wallet_id: &str) -> Result<(), Error> {
        // ストレージから削除
        self.storage.delete_multisig_wallet(wallet_id)?;
        
        // キャッシュから削除
        self.wallet_cache.remove(wallet_id);
        
        Ok(())
    }
    
    /// トランザクションを作成
    pub fn create_transaction(
        &mut self,
        wallet_id: &str,
        transaction_type: TransactionType,
        amount: u64,
        recipient: &str,
        data: Option<Vec<u8>>,
        expires_in_seconds: Option<u64>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<MultisigTransaction, Error> {
        // ウォレットを取得
        let mut wallet = self.get_wallet(wallet_id)?;
        
        // ウォレットのステータスをチェック
        if wallet.status != WalletStatus::Active {
            return Err(Error::InvalidState(format!(
                "Wallet is not active: {:?}", wallet.status
            )));
        }
        
        // 残高をチェック
        if amount > wallet.balance {
            return Err(Error::InsufficientFunds(format!(
                "Insufficient funds: {} < {}", wallet.balance, amount
            )));
        }
        
        // トランザクションを作成
        let transaction = Transaction {
            id: format!("tx_{}", Utc::now().timestamp_nanos()),
            transaction_type,
            sender: wallet.address.clone(),
            recipient: recipient.to_string(),
            amount,
            fee: calculate_fee(&transaction_type, amount),
            nonce: wallet.nonce,
            data: data.unwrap_or_default(),
            timestamp: Utc::now(),
            signature: None,
            status: TransactionStatus::Pending,
            block_id: None,
            shard_id: "".to_string(), // シャードIDは後で設定
        };
        
        // マルチシグトランザクションを作成
        let multisig_tx_id = format!("mstx_{}", Utc::now().timestamp_nanos());
        
        let expires_at = expires_in_seconds.map(|seconds| {
            Utc::now() + chrono::Duration::seconds(seconds as i64)
        });
        
        let multisig_tx = MultisigTransaction {
            id: multisig_tx_id.clone(),
            transaction,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            signatures: HashMap::new(),
            required_signatures: wallet.required_signatures,
            status: MultisigTransactionStatus::Pending,
            executed_at: None,
            expires_at,
            metadata: metadata.unwrap_or_default(),
        };
        
        // ウォレットに追加
        wallet.pending_transactions.insert(multisig_tx_id.clone(), multisig_tx.clone());
        wallet.updated_at = Utc::now();
        
        // ウォレットを更新
        self.update_wallet(&wallet)?;
        
        Ok(multisig_tx)
    }
    
    /// トランザクションに署名
    pub fn sign_transaction(
        &mut self,
        wallet_id: &str,
        transaction_id: &str,
        key_pair: &KeyPair,
    ) -> Result<MultisigTransaction, Error> {
        // ウォレットを取得
        let mut wallet = self.get_wallet(wallet_id)?;
        
        // トランザクションを取得
        let mut multisig_tx = wallet.pending_transactions.get(transaction_id).ok_or_else(|| {
            Error::NotFound(format!("Transaction not found: {}", transaction_id))
        })?.clone();
        
        // ウォレットのステータスをチェック
        if wallet.status != WalletStatus::Active {
            return Err(Error::InvalidState(format!(
                "Wallet is not active: {:?}", wallet.status
            )));
        }
        
        // トランザクションのステータスをチェック
        if multisig_tx.status != MultisigTransactionStatus::Pending {
            return Err(Error::InvalidState(format!(
                "Transaction is not pending: {:?}", multisig_tx.status
            )));
        }
        
        // 有効期限をチェック
        if let Some(expires_at) = multisig_tx.expires_at {
            if Utc::now() > expires_at {
                // 期限切れ
                multisig_tx.status = MultisigTransactionStatus::Expired;
                wallet.pending_transactions.insert(transaction_id.to_string(), multisig_tx.clone());
                self.update_wallet(&wallet)?;
                
                return Err(Error::TransactionExpired(format!(
                    "Transaction expired at {}", expires_at
                )));
            }
        }
        
        // 署名者が所有者かチェック
        let public_key = key_pair.public_key();
        
        if !wallet.owners.contains(&public_key) {
            return Err(Error::Unauthorized(format!(
                "Signer is not an owner of the wallet: {}", public_key
            )));
        }
        
        // 既に署名済みかチェック
        let signer_id = public_key.to_string();
        
        if multisig_tx.signatures.contains_key(&signer_id) {
            return Err(Error::InvalidState(format!(
                "Transaction already signed by {}", signer_id
            )));
        }
        
        // トランザクションに署名
        let signature = key_pair.sign(&multisig_tx.transaction.id.as_bytes())?;
        
        // 署名を追加
        multisig_tx.signatures.insert(signer_id, signature);
        multisig_tx.updated_at = Utc::now();
        
        // 必要な署名数に達したかチェック
        if multisig_tx.signatures.len() as u32 >= multisig_tx.required_signatures {
            multisig_tx.status = MultisigTransactionStatus::Ready;
        }
        
        // ウォレットを更新
        wallet.pending_transactions.insert(transaction_id.to_string(), multisig_tx.clone());
        wallet.updated_at = Utc::now();
        
        self.update_wallet(&wallet)?;
        
        Ok(multisig_tx)
    }
    
    /// トランザクションを実行
    pub fn execute_transaction(
        &mut self,
        wallet_id: &str,
        transaction_id: &str,
    ) -> Result<Transaction, Error> {
        // ウォレットを取得
        let mut wallet = self.get_wallet(wallet_id)?;
        
        // トランザクションを取得
        let mut multisig_tx = wallet.pending_transactions.get(transaction_id).ok_or_else(|| {
            Error::NotFound(format!("Transaction not found: {}", transaction_id))
        })?.clone();
        
        // ウォレットのステータスをチェック
        if wallet.status != WalletStatus::Active {
            return Err(Error::InvalidState(format!(
                "Wallet is not active: {:?}", wallet.status
            )));
        }
        
        // トランザクションのステータスをチェック
        if multisig_tx.status != MultisigTransactionStatus::Ready {
            return Err(Error::InvalidState(format!(
                "Transaction is not ready: {:?}", multisig_tx.status
            )));
        }
        
        // 有効期限をチェック
        if let Some(expires_at) = multisig_tx.expires_at {
            if Utc::now() > expires_at {
                // 期限切れ
                multisig_tx.status = MultisigTransactionStatus::Expired;
                wallet.pending_transactions.insert(transaction_id.to_string(), multisig_tx.clone());
                self.update_wallet(&wallet)?;
                
                return Err(Error::TransactionExpired(format!(
                    "Transaction expired at {}", expires_at
                )));
            }
        }
        
        // 残高をチェック
        let total_amount = multisig_tx.transaction.amount + multisig_tx.transaction.fee;
        
        if total_amount > wallet.balance {
            return Err(Error::InsufficientFunds(format!(
                "Insufficient funds: {} < {}", wallet.balance, total_amount
            )));
        }
        
        // トランザクションを実行
        let mut transaction = multisig_tx.transaction.clone();
        
        // 複数の署名を組み合わせた署名を作成
        let combined_signature = combine_signatures(&multisig_tx.signatures.values().cloned().collect());
        transaction.signature = Some(combined_signature);
        
        // トランザクションを送信
        // 実際の実装では、トランザクションをブロックチェーンに送信
        // ここでは簡易的な実装を提供
        
        // ウォレットの残高を更新
        wallet.balance -= total_amount;
        wallet.nonce += 1;
        
        // トランザクションのステータスを更新
        multisig_tx.status = MultisigTransactionStatus::Executed;
        multisig_tx.executed_at = Some(Utc::now());
        multisig_tx.updated_at = Utc::now();
        
        // ウォレットを更新
        wallet.pending_transactions.insert(transaction_id.to_string(), multisig_tx);
        wallet.updated_at = Utc::now();
        
        self.update_wallet(&wallet)?;
        
        Ok(transaction)
    }
    
    /// トランザクションをキャンセル
    pub fn cancel_transaction(
        &mut self,
        wallet_id: &str,
        transaction_id: &str,
        key_pair: &KeyPair,
    ) -> Result<MultisigTransaction, Error> {
        // ウォレットを取得
        let mut wallet = self.get_wallet(wallet_id)?;
        
        // トランザクションを取得
        let mut multisig_tx = wallet.pending_transactions.get(transaction_id).ok_or_else(|| {
            Error::NotFound(format!("Transaction not found: {}", transaction_id))
        })?.clone();
        
        // ウォレットのステータスをチェック
        if wallet.status != WalletStatus::Active {
            return Err(Error::InvalidState(format!(
                "Wallet is not active: {:?}", wallet.status
            )));
        }
        
        // トランザクションのステータスをチェック
        if multisig_tx.status != MultisigTransactionStatus::Pending && multisig_tx.status != MultisigTransactionStatus::Ready {
            return Err(Error::InvalidState(format!(
                "Transaction cannot be cancelled: {:?}", multisig_tx.status
            )));
        }
        
        // 署名者が所有者かチェック
        let public_key = key_pair.public_key();
        
        if !wallet.owners.contains(&public_key) {
            return Err(Error::Unauthorized(format!(
                "Signer is not an owner of the wallet: {}", public_key
            )));
        }
        
        // トランザクションをキャンセル
        multisig_tx.status = MultisigTransactionStatus::Cancelled;
        multisig_tx.updated_at = Utc::now();
        
        // ウォレットを更新
        wallet.pending_transactions.insert(transaction_id.to_string(), multisig_tx.clone());
        wallet.updated_at = Utc::now();
        
        self.update_wallet(&wallet)?;
        
        Ok(multisig_tx)
    }
    
    /// トランザクションを取得
    pub fn get_transaction(
        &mut self,
        wallet_id: &str,
        transaction_id: &str,
    ) -> Result<MultisigTransaction, Error> {
        // ウォレットを取得
        let wallet = self.get_wallet(wallet_id)?;
        
        // トランザクションを取得
        let multisig_tx = wallet.pending_transactions.get(transaction_id).ok_or_else(|| {
            Error::NotFound(format!("Transaction not found: {}", transaction_id))
        })?.clone();
        
        Ok(multisig_tx)
    }
    
    /// ウォレットの保留中のトランザクションを取得
    pub fn get_pending_transactions(
        &mut self,
        wallet_id: &str,
    ) -> Result<Vec<MultisigTransaction>, Error> {
        // ウォレットを取得
        let wallet = self.get_wallet(wallet_id)?;
        
        // 保留中のトランザクションを取得
        let pending_txs: Vec<MultisigTransaction> = wallet.pending_transactions.values()
            .filter(|tx| tx.status == MultisigTransactionStatus::Pending || tx.status == MultisigTransactionStatus::Ready)
            .cloned()
            .collect();
        
        Ok(pending_txs)
    }
    
    /// ウォレットの実行済みトランザクションを取得
    pub fn get_executed_transactions(
        &mut self,
        wallet_id: &str,
    ) -> Result<Vec<MultisigTransaction>, Error> {
        // ウォレットを取得
        let wallet = self.get_wallet(wallet_id)?;
        
        // 実行済みのトランザクションを取得
        let executed_txs: Vec<MultisigTransaction> = wallet.pending_transactions.values()
            .filter(|tx| tx.status == MultisigTransactionStatus::Executed)
            .cloned()
            .collect();
        
        Ok(executed_txs)
    }
    
    /// ウォレットの所有者を追加
    pub fn add_owner(
        &mut self,
        wallet_id: &str,
        new_owner: PublicKey,
        key_pairs: &[KeyPair],
    ) -> Result<MultisigWallet, Error> {
        // ウォレットを取得
        let mut wallet = self.get_wallet(wallet_id)?;
        
        // ウォレットのステータスをチェック
        if wallet.status != WalletStatus::Active {
            return Err(Error::InvalidState(format!(
                "Wallet is not active: {:?}", wallet.status
            )));
        }
        
        // 既に所有者かチェック
        if wallet.owners.contains(&new_owner) {
            return Err(Error::InvalidInput(format!(
                "Already an owner: {}", new_owner
            )));
        }
        
        // 署名者が所有者かチェック
        let mut signatures = 0;
        
        for key_pair in key_pairs {
            let public_key = key_pair.public_key();
            
            if wallet.owners.contains(&public_key) {
                signatures += 1;
            }
        }
        
        // 必要な署名数をチェック
        if signatures < wallet.required_signatures {
            return Err(Error::Unauthorized(format!(
                "Not enough signatures: {} < {}", signatures, wallet.required_signatures
            )));
        }
        
        // 所有者を追加
        wallet.owners.push(new_owner);
        wallet.updated_at = Utc::now();
        
        // ウォレットを更新
        self.update_wallet(&wallet)?;
        
        Ok(wallet)
    }
    
    /// ウォレットの所有者を削除
    pub fn remove_owner(
        &mut self,
        wallet_id: &str,
        owner_to_remove: &PublicKey,
        key_pairs: &[KeyPair],
    ) -> Result<MultisigWallet, Error> {
        // ウォレットを取得
        let mut wallet = self.get_wallet(wallet_id)?;
        
        // ウォレットのステータスをチェック
        if wallet.status != WalletStatus::Active {
            return Err(Error::InvalidState(format!(
                "Wallet is not active: {:?}", wallet.status
            )));
        }
        
        // 所有者かチェック
        if !wallet.owners.contains(owner_to_remove) {
            return Err(Error::InvalidInput(format!(
                "Not an owner: {}", owner_to_remove
            )));
        }
        
        // 最低1人の所有者が必要
        if wallet.owners.len() <= 1 {
            return Err(Error::InvalidInput("Cannot remove the last owner".to_string()));
        }
        
        // 署名者が所有者かチェック
        let mut signatures = 0;
        
        for key_pair in key_pairs {
            let public_key = key_pair.public_key();
            
            if wallet.owners.contains(&public_key) && &public_key != owner_to_remove {
                signatures += 1;
            }
        }
        
        // 必要な署名数をチェック
        if signatures < wallet.required_signatures {
            return Err(Error::Unauthorized(format!(
                "Not enough signatures: {} < {}", signatures, wallet.required_signatures
            )));
        }
        
        // 所有者を削除
        wallet.owners.retain(|owner| owner != owner_to_remove);
        wallet.updated_at = Utc::now();
        
        // 必要な署名数が所有者数を超えないようにする
        if wallet.required_signatures as usize > wallet.owners.len() {
            wallet.required_signatures = wallet.owners.len() as u32;
        }
        
        // ウォレットを更新
        self.update_wallet(&wallet)?;
        
        Ok(wallet)
    }
    
    /// 必要な署名数を変更
    pub fn change_required_signatures(
        &mut self,
        wallet_id: &str,
        new_required_signatures: u32,
        key_pairs: &[KeyPair],
    ) -> Result<MultisigWallet, Error> {
        // ウォレットを取得
        let mut wallet = self.get_wallet(wallet_id)?;
        
        // ウォレットのステータスをチェック
        if wallet.status != WalletStatus::Active {
            return Err(Error::InvalidState(format!(
                "Wallet is not active: {:?}", wallet.status
            )));
        }
        
        // 新しい必要署名数をチェック
        if new_required_signatures == 0 || new_required_signatures as usize > wallet.owners.len() {
            return Err(Error::InvalidInput(format!(
                "Required signatures must be between 1 and {}", wallet.owners.len()
            )));
        }
        
        // 署名者が所有者かチェック
        let mut signatures = 0;
        
        for key_pair in key_pairs {
            let public_key = key_pair.public_key();
            
            if wallet.owners.contains(&public_key) {
                signatures += 1;
            }
        }
        
        // 必要な署名数をチェック
        if signatures < wallet.required_signatures {
            return Err(Error::Unauthorized(format!(
                "Not enough signatures: {} < {}", signatures, wallet.required_signatures
            )));
        }
        
        // 必要署名数を変更
        wallet.required_signatures = new_required_signatures;
        wallet.updated_at = Utc::now();
        
        // ウォレットを更新
        self.update_wallet(&wallet)?;
        
        Ok(wallet)
    }
    
    /// ウォレットを凍結
    pub fn freeze_wallet(
        &mut self,
        wallet_id: &str,
        key_pairs: &[KeyPair],
    ) -> Result<MultisigWallet, Error> {
        // ウォレットを取得
        let mut wallet = self.get_wallet(wallet_id)?;
        
        // ウォレットのステータスをチェック
        if wallet.status != WalletStatus::Active {
            return Err(Error::InvalidState(format!(
                "Wallet is not active: {:?}", wallet.status
            )));
        }
        
        // 署名者が所有者かチェック
        let mut signatures = 0;
        
        for key_pair in key_pairs {
            let public_key = key_pair.public_key();
            
            if wallet.owners.contains(&public_key) {
                signatures += 1;
            }
        }
        
        // 必要な署名数をチェック
        if signatures < wallet.required_signatures {
            return Err(Error::Unauthorized(format!(
                "Not enough signatures: {} < {}", signatures, wallet.required_signatures
            )));
        }
        
        // ウォレットを凍結
        wallet.status = WalletStatus::Frozen;
        wallet.updated_at = Utc::now();
        
        // ウォレットを更新
        self.update_wallet(&wallet)?;
        
        Ok(wallet)
    }
    
    /// ウォレットを解凍
    pub fn unfreeze_wallet(
        &mut self,
        wallet_id: &str,
        key_pairs: &[KeyPair],
    ) -> Result<MultisigWallet, Error> {
        // ウォレットを取得
        let mut wallet = self.get_wallet(wallet_id)?;
        
        // ウォレットのステータスをチェック
        if wallet.status != WalletStatus::Frozen {
            return Err(Error::InvalidState(format!(
                "Wallet is not frozen: {:?}", wallet.status
            )));
        }
        
        // 署名者が所有者かチェック
        let mut signatures = 0;
        
        for key_pair in key_pairs {
            let public_key = key_pair.public_key();
            
            if wallet.owners.contains(&public_key) {
                signatures += 1;
            }
        }
        
        // 必要な署名数をチェック
        if signatures < wallet.required_signatures {
            return Err(Error::Unauthorized(format!(
                "Not enough signatures: {} < {}", signatures, wallet.required_signatures
            )));
        }
        
        // ウォレットを解凍
        wallet.status = WalletStatus::Active;
        wallet.updated_at = Utc::now();
        
        // ウォレットを更新
        self.update_wallet(&wallet)?;
        
        Ok(wallet)
    }
    
    /// ウォレットを閉鎖
    pub fn close_wallet(
        &mut self,
        wallet_id: &str,
        key_pairs: &[KeyPair],
    ) -> Result<MultisigWallet, Error> {
        // ウォレットを取得
        let mut wallet = self.get_wallet(wallet_id)?;
        
        // ウォレットのステータスをチェック
        if wallet.status == WalletStatus::Closed {
            return Err(Error::InvalidState("Wallet is already closed".to_string()));
        }
        
        // 署名者が所有者かチェック
        let mut signatures = 0;
        
        for key_pair in key_pairs {
            let public_key = key_pair.public_key();
            
            if wallet.owners.contains(&public_key) {
                signatures += 1;
            }
        }
        
        // 必要な署名数をチェック
        if signatures < wallet.required_signatures {
            return Err(Error::Unauthorized(format!(
                "Not enough signatures: {} < {}", signatures, wallet.required_signatures
            )));
        }
        
        // 残高をチェック
        if wallet.balance > 0 {
            return Err(Error::InvalidState(format!(
                "Wallet still has balance: {}", wallet.balance
            )));
        }
        
        // ウォレットを閉鎖
        wallet.status = WalletStatus::Closed;
        wallet.updated_at = Utc::now();
        
        // ウォレットを更新
        self.update_wallet(&wallet)?;
        
        Ok(wallet)
    }
    
    /// ウォレットの残高を更新
    pub fn update_balance(
        &mut self,
        wallet_id: &str,
        new_balance: u64,
    ) -> Result<MultisigWallet, Error> {
        // ウォレットを取得
        let mut wallet = self.get_wallet(wallet_id)?;
        
        // 残高を更新
        wallet.balance = new_balance;
        wallet.updated_at = Utc::now();
        
        // ウォレットを更新
        self.update_wallet(&wallet)?;
        
        Ok(wallet)
    }
    
    /// ウォレットのメタデータを更新
    pub fn update_metadata(
        &mut self,
        wallet_id: &str,
        metadata: HashMap<String, String>,
        key_pairs: &[KeyPair],
    ) -> Result<MultisigWallet, Error> {
        // ウォレットを取得
        let mut wallet = self.get_wallet(wallet_id)?;
        
        // ウォレットのステータスをチェック
        if wallet.status != WalletStatus::Active {
            return Err(Error::InvalidState(format!(
                "Wallet is not active: {:?}", wallet.status
            )));
        }
        
        // 署名者が所有者かチェック
        let mut signatures = 0;
        
        for key_pair in key_pairs {
            let public_key = key_pair.public_key();
            
            if wallet.owners.contains(&public_key) {
                signatures += 1;
            }
        }
        
        // 必要な署名数をチェック
        if signatures < wallet.required_signatures {
            return Err(Error::Unauthorized(format!(
                "Not enough signatures: {} < {}", signatures, wallet.required_signatures
            )));
        }
        
        // メタデータを更新
        wallet.metadata = metadata;
        wallet.updated_at = Utc::now();
        
        // ウォレットを更新
        self.update_wallet(&wallet)?;
        
        Ok(wallet)
    }
    
    /// キャッシュをクリア
    pub fn clear_cache(&mut self) {
        self.wallet_cache.clear();
    }
}

/// マルチシグアドレスを生成
fn generate_multisig_address(owners: &[PublicKey], required_signatures: u32) -> String {
    // 所有者の公開鍵をソート
    let mut sorted_owners = owners.to_vec();
    sorted_owners.sort();
    
    // 所有者の公開鍵と必要署名数を連結
    let mut data = Vec::new();
    
    for owner in &sorted_owners {
        data.extend_from_slice(owner.as_bytes());
    }
    
    data.extend_from_slice(&required_signatures.to_le_bytes());
    
    // ハッシュを計算
    let hash = hash(&data);
    
    // アドレスを生成
    format!("msw_{}", hex::encode(hash))
}

/// 手数料を計算
fn calculate_fee(transaction_type: &TransactionType, amount: u64) -> u64 {
    match transaction_type {
        TransactionType::Transfer => amount / 1000, // 0.1%
        TransactionType::SmartContract => amount / 500, // 0.2%
        _ => amount / 2000, // 0.05%
    }
}

/// 署名を組み合わせる
fn combine_signatures(signatures: &[Signature]) -> Signature {
    // 実際の実装では、署名を組み合わせる方法が必要
    // ここでは簡易的な実装を提供
    if signatures.is_empty() {
        return Signature::from_bytes(&[0; 64]).unwrap();
    }
    
    signatures[0].clone()
}