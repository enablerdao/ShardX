use crate::transaction::{Transaction, TransactionStatus};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// ウォレットのアカウント情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// アカウントID
    pub id: String,
    /// 公開鍵
    pub public_key: String,
    /// 秘密鍵（実際の実装では暗号化して保存）
    #[serde(skip_serializing)]
    pub private_key: String,
    /// アカウント名
    pub name: String,
    /// 残高
    pub balance: f64,
    /// トークン残高
    pub token_balances: HashMap<String, f64>,
    /// 作成日時
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Account {
    /// 新しいアカウントを作成
    pub fn new(name: String) -> Self {
        // 実際の実装では、暗号学的に安全な鍵ペアを生成
        let id = Uuid::new_v4().to_string();
        let private_key = generate_private_key();
        let public_key = derive_public_key(&private_key);
        
        Self {
            id,
            public_key,
            private_key,
            name,
            balance: 0.0,
            token_balances: HashMap::new(),
            created_at: chrono::Utc::now(),
        }
    }
    
    /// トランザクションに署名
    pub fn sign_transaction(&self, payload: &[u8]) -> Vec<u8> {
        // 実際の実装では、秘密鍵を使用して署名
        // 簡略化のため、ここではSHA-256ハッシュを使用
        let mut hasher = Sha256::new();
        hasher.update(&self.private_key);
        hasher.update(payload);
        hasher.finalize().to_vec()
    }
    
    /// トークン残高を更新
    pub fn update_token_balance(&mut self, token_id: &str, amount: f64) {
        let balance = self.token_balances.entry(token_id.to_string()).or_insert(0.0);
        *balance += amount;
    }
}

/// ウォレットマネージャー
pub struct WalletManager {
    /// アカウントのマップ
    accounts: Mutex<HashMap<String, Account>>,
}

impl WalletManager {
    /// 新しいウォレットマネージャーを作成
    pub fn new() -> Self {
        Self {
            accounts: Mutex::new(HashMap::new()),
        }
    }
    
    /// アカウントを作成
    pub fn create_account(&self, name: String) -> Result<Account, String> {
        let account = Account::new(name);
        let account_id = account.id.clone();
        
        let mut accounts = self.accounts.lock().unwrap();
        accounts.insert(account_id, account.clone());
        
        info!("Account created: {}", account.id);
        Ok(account)
    }
    
    /// アカウントを取得
    pub fn get_account(&self, account_id: &str) -> Option<Account> {
        let accounts = self.accounts.lock().unwrap();
        accounts.get(account_id).cloned()
    }
    
    /// すべてのアカウントを取得
    pub fn get_all_accounts(&self) -> Vec<Account> {
        let accounts = self.accounts.lock().unwrap();
        accounts.values().cloned().collect()
    }
    
    /// トランザクションを作成
    pub fn create_transaction(
        &self,
        from_account_id: &str,
        to_account_id: &str,
        amount: f64,
        token_id: Option<String>,
        parent_ids: Vec<String>,
    ) -> Result<Transaction, String> {
        let accounts = self.accounts.lock().unwrap();
        
        // 送信元アカウントを確認
        let from_account = accounts.get(from_account_id)
            .ok_or_else(|| format!("From account {} not found", from_account_id))?;
        
        // 送信先アカウントを確認
        let to_account = accounts.get(to_account_id)
            .ok_or_else(|| format!("To account {} not found", to_account_id))?;
        
        // 残高を確認
        if let Some(token) = &token_id {
            let balance = from_account.token_balances.get(token).unwrap_or(&0.0);
            if *balance < amount {
                return Err(format!("Insufficient token balance: {} < {}", balance, amount));
            }
        } else {
            if from_account.balance < amount {
                return Err(format!("Insufficient balance: {} < {}", from_account.balance, amount));
            }
        }
        
        // トランザクションデータを作成
        let tx_data = TransactionData {
            from: from_account.id.clone(),
            to: to_account.id.clone(),
            amount,
            token_id,
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        // JSONにシリアライズ
        let payload = serde_json::to_vec(&tx_data)
            .map_err(|e| format!("Failed to serialize transaction data: {}", e))?;
        
        // 署名
        let signature = from_account.sign_transaction(&payload);
        
        // トランザクションを作成
        let transaction = Transaction::new(parent_ids, payload, signature);
        
        debug!("Transaction created: {}", transaction.id);
        Ok(transaction)
    }
    
    /// トランザクションを処理
    pub fn process_transaction(&self, transaction: &Transaction) -> Result<(), String> {
        // ペイロードをデシリアライズ
        let tx_data: TransactionData = serde_json::from_slice(&transaction.payload)
            .map_err(|e| format!("Failed to deserialize transaction data: {}", e))?;
        
        let mut accounts = self.accounts.lock().unwrap();
        
        // アカウントが存在するか確認
        if !accounts.contains_key(&tx_data.from) {
            return Err(format!("From account {} not found", tx_data.from));
        }
        if !accounts.contains_key(&tx_data.to) {
            return Err(format!("To account {} not found", tx_data.to));
        }
        
        // 残高を更新
        if let Some(token_id) = &tx_data.token_id {
            // トークン送金の場合
            // 送信元アカウントのトークン残高を確認
            {
                let from_account = accounts.get(&tx_data.from).unwrap();
                let balance = from_account.token_balances.get(token_id).unwrap_or(&0.0);
                if *balance < tx_data.amount {
                    return Err(format!("Insufficient token balance: {} < {}", balance, tx_data.amount));
                }
                *balance
            };
            
            // 送信元アカウントのトークン残高を減らす
            {
                let from_account = accounts.get_mut(&tx_data.from).unwrap();
                let balance = from_account.token_balances.entry(token_id.clone()).or_insert(0.0);
                *balance -= tx_data.amount;
            }
            
            // 送信先アカウントのトークン残高を増やす
            {
                let to_account = accounts.get_mut(&tx_data.to).unwrap();
                let balance = to_account.token_balances.entry(token_id.clone()).or_insert(0.0);
                *balance += tx_data.amount;
            }
            
            info!("Token transfer: {} {} from {} to {}", tx_data.amount, token_id, tx_data.from, tx_data.to);
        } else {
            // 通常の送金の場合
            // 送信元アカウントの残高を確認
            {
                let from_account = accounts.get(&tx_data.from).unwrap();
                if from_account.balance < tx_data.amount {
                    return Err(format!("Insufficient balance: {} < {}", from_account.balance, tx_data.amount));
                }
                from_account.balance
            };
            
            // 送信元アカウントの残高を減らす
            {
                let from_account = accounts.get_mut(&tx_data.from).unwrap();
                from_account.balance -= tx_data.amount;
            }
            
            // 送信先アカウントの残高を増やす
            {
                let to_account = accounts.get_mut(&tx_data.to).unwrap();
                to_account.balance += tx_data.amount;
            }
            
            info!("Transfer: {} from {} to {}", tx_data.amount, tx_data.from, tx_data.to);
        }
        
        Ok(())
    }
    
    /// トレードを処理
    pub fn process_trade(
        &self,
        buyer_id: &str,
        seller_id: &str,
        base_token: &str,
        _quote_token: &str,  // 未使用変数に_を追加
        price: f64,
        amount: f64,
    ) -> Result<(), String> {
        let mut accounts = self.accounts.lock().unwrap();
        
        // アカウントが存在するか確認
        if !accounts.contains_key(buyer_id) {
            return Err(format!("Buyer account {} not found", buyer_id));
        }
        if !accounts.contains_key(seller_id) {
            return Err(format!("Seller account {} not found", seller_id));
        }
        
        // 取引金額を計算
        let total_price = price * amount;
        
        // 買い手の残高を確認
        {
            let buyer = accounts.get(buyer_id).unwrap();
            if buyer.balance < total_price {
                return Err(format!("Buyer has insufficient balance: {} < {}", buyer.balance, total_price));
            }
        }
        
        // 売り手のトークン残高を確認
        {
            let seller = accounts.get(seller_id).unwrap();
            let seller_token_balance = seller.token_balances.get(base_token).unwrap_or(&0.0);
            if *seller_token_balance < amount {
                return Err(format!("Seller has insufficient token balance: {} < {}", seller_token_balance, amount));
            }
        }
        
        // 買い手の残高を更新
        {
            let buyer = accounts.get_mut(buyer_id).unwrap();
            buyer.balance -= total_price;
            let buyer_token_balance = buyer.token_balances.entry(base_token.to_string()).or_insert(0.0);
            *buyer_token_balance += amount;
        }
        
        // 売り手の残高を更新
        {
            let seller = accounts.get_mut(seller_id).unwrap();
            seller.balance += total_price;
            let seller_token_balance = seller.token_balances.entry(base_token.to_string()).or_insert(0.0);
            *seller_token_balance -= amount;
        }
        
        info!("Trade executed: {} {} from {} to {} at price {}", 
            amount, base_token, seller_id, buyer_id, price);
        
        Ok(())
    }
}

/// トランザクションデータ
#[derive(Debug, Serialize, Deserialize)]
struct TransactionData {
    /// 送信元アカウントID
    from: String,
    /// 送信先アカウントID
    to: String,
    /// 金額
    amount: f64,
    /// トークンID（Noneの場合はネイティブトークン）
    token_id: Option<String>,
    /// タイムスタンプ
    timestamp: i64,
}

/// 秘密鍵を生成（実際の実装では暗号学的に安全な方法を使用）
fn generate_private_key() -> String {
    let mut hasher = Sha256::new();
    hasher.update(Uuid::new_v4().to_string());
    hex::encode(hasher.finalize())
}

/// 公開鍵を導出（実際の実装では暗号学的に正しい方法を使用）
fn derive_public_key(private_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(private_key);
    hex::encode(&hasher.finalize()[..16])
}