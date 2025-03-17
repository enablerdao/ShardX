use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::crypto::{hash, PublicKey, Signature};
use crate::error::Error;

/// マルチシグウォレット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigWallet {
    /// ID
    pub id: String,
    /// 名前
    pub name: String,
    /// 説明
    pub description: Option<String>,
    /// 所有者
    pub owners: Vec<Owner>,
    /// 閾値
    pub threshold: u32,
    /// トランザクション
    pub transactions: HashMap<String, Transaction>,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 更新日時
    pub updated_at: DateTime<Utc>,
    /// メタデータ
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// 所有者
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
    /// ID
    pub id: String,
    /// 名前
    pub name: Option<String>,
    /// アドレス
    pub address: String,
    /// 公開鍵
    pub public_key: PublicKey,
    /// 重み
    pub weight: u32,
    /// 追加日時
    pub added_at: DateTime<Utc>,
    /// メタデータ
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// トランザクション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// ID
    pub id: String,
    /// 宛先
    pub to: String,
    /// 値
    pub value: u64,
    /// データ
    pub data: Vec<u8>,
    /// ノンス
    pub nonce: u64,
    /// ガス制限
    pub gas_limit: u64,
    /// ガス価格
    pub gas_price: u64,
    /// 署名
    pub signatures: HashMap<String, Signature>,
    /// ステータス
    pub status: TransactionStatus,
    /// 作成者
    pub creator: String,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 更新日時
    pub updated_at: DateTime<Utc>,
    /// 実行日時
    pub executed_at: Option<DateTime<Utc>>,
    /// トランザクションハッシュ
    pub transaction_hash: Option<String>,
    /// メタデータ
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// トランザクションステータス
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TransactionStatus {
    /// 保留中
    Pending,
    /// 実行可能
    Executable,
    /// 実行中
    Executing,
    /// 実行済み
    Executed,
    /// 失敗
    Failed,
    /// キャンセル
    Cancelled,
    /// 期限切れ
    Expired,
}

impl MultisigWallet {
    /// 新しいマルチシグウォレットを作成
    pub fn new(id: String, name: String, threshold: u32) -> Self {
        let now = Utc::now();

        Self {
            id,
            name,
            description: None,
            owners: Vec::new(),
            threshold,
            transactions: HashMap::new(),
            created_at: now,
            updated_at: now,
            metadata: None,
            additional_properties: HashMap::new(),
        }
    }

    /// 所有者を追加
    pub fn add_owner(&mut self, owner: Owner) -> Result<(), Error> {
        // 所有者の重複をチェック
        if self
            .owners
            .iter()
            .any(|o| o.id == owner.id || o.address == owner.address)
        {
            return Err(Error::AlreadyExists(format!(
                "Owner already exists: {}",
                owner.id
            )));
        }

        self.owners.push(owner);
        self.updated_at = Utc::now();

        Ok(())
    }

    /// 所有者を削除
    pub fn remove_owner(&mut self, owner_id: &str) -> Result<(), Error> {
        let initial_len = self.owners.len();
        self.owners.retain(|o| o.id != owner_id);

        if self.owners.len() == initial_len {
            return Err(Error::NotFound(format!("Owner not found: {}", owner_id)));
        }

        // 閾値をチェック
        let total_weight: u32 = self.owners.iter().map(|o| o.weight).sum();
        if self.threshold > total_weight {
            return Err(Error::InvalidState(format!(
                "Threshold ({}) exceeds total weight ({})",
                self.threshold, total_weight
            )));
        }

        self.updated_at = Utc::now();

        Ok(())
    }

    /// 閾値を変更
    pub fn change_threshold(&mut self, threshold: u32) -> Result<(), Error> {
        let total_weight: u32 = self.owners.iter().map(|o| o.weight).sum();

        if threshold > total_weight {
            return Err(Error::InvalidInput(format!(
                "Threshold ({}) exceeds total weight ({})",
                threshold, total_weight
            )));
        }

        if threshold == 0 {
            return Err(Error::InvalidInput(
                "Threshold must be greater than 0".to_string(),
            ));
        }

        self.threshold = threshold;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// トランザクションを作成
    pub fn create_transaction(
        &mut self,
        to: String,
        value: u64,
        data: Vec<u8>,
        gas_limit: u64,
        gas_price: u64,
        creator: String,
    ) -> Result<String, Error> {
        // 作成者が所有者であることを確認
        if !self.owners.iter().any(|o| o.id == creator) {
            return Err(Error::InvalidInput(format!(
                "Creator is not an owner: {}",
                creator
            )));
        }

        // トランザクションIDを生成
        let id = format!("tx_{}", Utc::now().timestamp_nanos());

        // ノンスを計算
        let nonce = self.transactions.len() as u64;

        // トランザクションを作成
        let transaction = Transaction {
            id: id.clone(),
            to,
            value,
            data,
            nonce,
            gas_limit,
            gas_price,
            signatures: HashMap::new(),
            status: TransactionStatus::Pending,
            creator,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            executed_at: None,
            transaction_hash: None,
            metadata: None,
            additional_properties: HashMap::new(),
        };

        // トランザクションを保存
        self.transactions.insert(id.clone(), transaction);
        self.updated_at = Utc::now();

        Ok(id)
    }

    /// トランザクションに署名
    pub fn sign_transaction(
        &mut self,
        transaction_id: &str,
        owner_id: &str,
        signature: Signature,
    ) -> Result<(), Error> {
        // トランザクションを取得
        let transaction = self
            .transactions
            .get_mut(transaction_id)
            .ok_or_else(|| Error::NotFound(format!("Transaction not found: {}", transaction_id)))?;

        // トランザクションのステータスをチェック
        if transaction.status != TransactionStatus::Pending {
            return Err(Error::InvalidState(format!(
                "Transaction is not pending: {:?}",
                transaction.status
            )));
        }

        // 所有者をチェック
        let owner = self
            .owners
            .iter()
            .find(|o| o.id == owner_id)
            .ok_or_else(|| Error::NotFound(format!("Owner not found: {}", owner_id)))?;

        // 署名を検証
        let transaction_data = self.get_transaction_data(transaction);
        let valid = owner
            .public_key
            .verify(&transaction_data, &signature)
            .map_err(|e| Error::CryptoError(format!("Signature verification failed: {}", e)))?;

        if !valid {
            return Err(Error::InvalidInput("Invalid signature".to_string()));
        }

        // 署名を追加
        transaction
            .signatures
            .insert(owner_id.to_string(), signature);
        transaction.updated_at = Utc::now();

        // 署名の重みを計算
        let signed_weight: u32 = transaction
            .signatures
            .keys()
            .filter_map(|id| self.owners.iter().find(|o| o.id == *id))
            .map(|o| o.weight)
            .sum();

        // 閾値に達したかチェック
        if signed_weight >= self.threshold {
            transaction.status = TransactionStatus::Executable;
        }

        self.updated_at = Utc::now();

        Ok(())
    }

    /// トランザクションを実行
    pub fn execute_transaction(&mut self, transaction_id: &str) -> Result<(), Error> {
        // トランザクションを取得
        let transaction = self
            .transactions
            .get_mut(transaction_id)
            .ok_or_else(|| Error::NotFound(format!("Transaction not found: {}", transaction_id)))?;

        // トランザクションのステータスをチェック
        if transaction.status != TransactionStatus::Executable {
            return Err(Error::InvalidState(format!(
                "Transaction is not executable: {:?}",
                transaction.status
            )));
        }

        // 署名の重みを計算
        let signed_weight: u32 = transaction
            .signatures
            .keys()
            .filter_map(|id| self.owners.iter().find(|o| o.id == *id))
            .map(|o| o.weight)
            .sum();

        // 閾値に達しているかチェック
        if signed_weight < self.threshold {
            return Err(Error::InvalidState(format!(
                "Insufficient signatures: {} < {}",
                signed_weight, self.threshold
            )));
        }

        // トランザクションを実行
        transaction.status = TransactionStatus::Executing;
        transaction.updated_at = Utc::now();

        // 実際の実装では、ブロックチェーンにトランザクションを送信する
        // ここでは簡易的に成功したとみなす
        let now = Utc::now();
        transaction.status = TransactionStatus::Executed;
        transaction.executed_at = Some(now);
        transaction.transaction_hash = Some(format!(
            "0x{}",
            hex::encode(hash(&self.get_transaction_data(transaction)))
        ));
        transaction.updated_at = now;

        self.updated_at = now;

        Ok(())
    }

    /// トランザクションをキャンセル
    pub fn cancel_transaction(
        &mut self,
        transaction_id: &str,
        owner_id: &str,
    ) -> Result<(), Error> {
        // トランザクションを取得
        let transaction = self
            .transactions
            .get_mut(transaction_id)
            .ok_or_else(|| Error::NotFound(format!("Transaction not found: {}", transaction_id)))?;

        // トランザクションのステータスをチェック
        if transaction.status != TransactionStatus::Pending
            && transaction.status != TransactionStatus::Executable
        {
            return Err(Error::InvalidState(format!(
                "Transaction cannot be cancelled: {:?}",
                transaction.status
            )));
        }

        // 所有者をチェック
        if !self.owners.iter().any(|o| o.id == owner_id) {
            return Err(Error::NotFound(format!("Owner not found: {}", owner_id)));
        }

        // 作成者または署名者のみがキャンセル可能
        if transaction.creator != owner_id && !transaction.signatures.contains_key(owner_id) {
            return Err(Error::InvalidState(format!(
                "Only creator or signers can cancel the transaction"
            )));
        }

        // トランザクションをキャンセル
        transaction.status = TransactionStatus::Cancelled;
        transaction.updated_at = Utc::now();

        self.updated_at = Utc::now();

        Ok(())
    }

    /// トランザクションデータを取得
    fn get_transaction_data(&self, transaction: &Transaction) -> Vec<u8> {
        // 実際の実装では、トランザクションデータをシリアライズする
        // ここでは簡易的な実装を提供
        let mut data = Vec::new();

        // トランザクションID
        data.extend_from_slice(transaction.id.as_bytes());

        // 宛先
        data.extend_from_slice(transaction.to.as_bytes());

        // 値
        data.extend_from_slice(&transaction.value.to_be_bytes());

        // データ
        data.extend_from_slice(&transaction.data);

        // ノンス
        data.extend_from_slice(&transaction.nonce.to_be_bytes());

        // ガス制限
        data.extend_from_slice(&transaction.gas_limit.to_be_bytes());

        // ガス価格
        data.extend_from_slice(&transaction.gas_price.to_be_bytes());

        data
    }

    /// トランザクションの署名を検証
    pub fn verify_transaction_signature(
        &self,
        transaction_id: &str,
        owner_id: &str,
    ) -> Result<bool, Error> {
        // トランザクションを取得
        let transaction = self
            .transactions
            .get(transaction_id)
            .ok_or_else(|| Error::NotFound(format!("Transaction not found: {}", transaction_id)))?;

        // 所有者をチェック
        let owner = self
            .owners
            .iter()
            .find(|o| o.id == owner_id)
            .ok_or_else(|| Error::NotFound(format!("Owner not found: {}", owner_id)))?;

        // 署名をチェック
        let signature = transaction.signatures.get(owner_id).ok_or_else(|| {
            Error::NotFound(format!("Signature not found for owner: {}", owner_id))
        })?;

        // 署名を検証
        let transaction_data = self.get_transaction_data(transaction);
        owner
            .public_key
            .verify(&transaction_data, signature)
            .map_err(|e| Error::CryptoError(format!("Signature verification failed: {}", e)))
    }

    /// トランザクションの署名重みを計算
    pub fn get_transaction_signature_weight(&self, transaction_id: &str) -> Result<u32, Error> {
        // トランザクションを取得
        let transaction = self
            .transactions
            .get(transaction_id)
            .ok_or_else(|| Error::NotFound(format!("Transaction not found: {}", transaction_id)))?;

        // 署名の重みを計算
        let signed_weight: u32 = transaction
            .signatures
            .keys()
            .filter_map(|id| self.owners.iter().find(|o| o.id == *id))
            .map(|o| o.weight)
            .sum();

        Ok(signed_weight)
    }

    /// トランザクションが実行可能か確認
    pub fn is_transaction_executable(&self, transaction_id: &str) -> Result<bool, Error> {
        // トランザクションを取得
        let transaction = self
            .transactions
            .get(transaction_id)
            .ok_or_else(|| Error::NotFound(format!("Transaction not found: {}", transaction_id)))?;

        // ステータスをチェック
        if transaction.status != TransactionStatus::Pending
            && transaction.status != TransactionStatus::Executable
        {
            return Ok(false);
        }

        // 署名の重みを計算
        let signed_weight = self.get_transaction_signature_weight(transaction_id)?;

        // 閾値に達しているかチェック
        Ok(signed_weight >= self.threshold)
    }

    /// 所有者の重みを変更
    pub fn change_owner_weight(&mut self, owner_id: &str, weight: u32) -> Result<(), Error> {
        // 所有者を取得
        let owner = self
            .owners
            .iter_mut()
            .find(|o| o.id == owner_id)
            .ok_or_else(|| Error::NotFound(format!("Owner not found: {}", owner_id)))?;

        // 重みを変更
        owner.weight = weight;

        // 閾値をチェック
        let total_weight: u32 = self.owners.iter().map(|o| o.weight).sum();
        if self.threshold > total_weight {
            return Err(Error::InvalidState(format!(
                "Threshold ({}) exceeds total weight ({})",
                self.threshold, total_weight
            )));
        }

        self.updated_at = Utc::now();

        Ok(())
    }

    /// 所有者の公開鍵を更新
    pub fn update_owner_public_key(
        &mut self,
        owner_id: &str,
        public_key: PublicKey,
    ) -> Result<(), Error> {
        // 所有者を取得
        let owner = self
            .owners
            .iter_mut()
            .find(|o| o.id == owner_id)
            .ok_or_else(|| Error::NotFound(format!("Owner not found: {}", owner_id)))?;

        // 公開鍵を更新
        owner.public_key = public_key;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// 所有者のアドレスを更新
    pub fn update_owner_address(&mut self, owner_id: &str, address: String) -> Result<(), Error> {
        // アドレスの重複をチェック
        if self
            .owners
            .iter()
            .any(|o| o.id != owner_id && o.address == address)
        {
            return Err(Error::AlreadyExists(format!(
                "Address already in use: {}",
                address
            )));
        }

        // 所有者を取得
        let owner = self
            .owners
            .iter_mut()
            .find(|o| o.id == owner_id)
            .ok_or_else(|| Error::NotFound(format!("Owner not found: {}", owner_id)))?;

        // アドレスを更新
        owner.address = address;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// 所有者の名前を更新
    pub fn update_owner_name(&mut self, owner_id: &str, name: Option<String>) -> Result<(), Error> {
        // 所有者を取得
        let owner = self
            .owners
            .iter_mut()
            .find(|o| o.id == owner_id)
            .ok_or_else(|| Error::NotFound(format!("Owner not found: {}", owner_id)))?;

        // 名前を更新
        owner.name = name;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// ウォレットの説明を更新
    pub fn update_description(&mut self, description: Option<String>) -> Result<(), Error> {
        self.description = description;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// ウォレットの名前を更新
    pub fn update_name(&mut self, name: String) -> Result<(), Error> {
        self.name = name;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// ウォレットのメタデータを更新
    pub fn update_metadata(
        &mut self,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<(), Error> {
        self.metadata = metadata;
        self.updated_at = Utc::now();

        Ok(())
    }
}
