use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::crypto::{PublicKey, PrivateKey, Signature, KeyPair, hash};
use crate::transaction::{Transaction, TransactionStatus, TransactionType};
use crate::wallet::Wallet;
use crate::wallet::multisig::config::MultisigConfig;
use crate::wallet::multisig::transaction::{
    MultisigTransaction, MultisigTransactionStatus, 
    TransactionStep, TransactionStepStatus, 
    TransactionHistoryEntry, TransactionAction
};

/// マルチシグウォレット
pub struct MultisigWallet {
    /// ウォレットID
    pub id: String,
    /// ウォレット名
    pub name: String,
    /// 設定
    pub config: MultisigConfig,
    /// 公開キーリスト
    pub public_keys: Vec<PublicKey>,
    /// 秘密キー（所有している場合）
    pub private_key: Option<PrivateKey>,
    /// トランザクション履歴
    pub transactions: Arc<Mutex<HashMap<String, MultisigTransaction>>>,
    /// 1日あたりの送金額履歴
    pub daily_spending: Arc<Mutex<HashMap<String, f64>>>,
    /// トランザクション処理ステップ
    pub transaction_steps: Arc<Mutex<HashMap<String, Vec<TransactionStep>>>>,
    /// トランザクション履歴
    pub transaction_history: Arc<Mutex<HashMap<String, Vec<TransactionHistoryEntry>>>>,
    /// 基本ウォレット
    pub base_wallet: Arc<Wallet>,
}

impl MultisigWallet {
    /// 新しいマルチシグウォレットを作成
    pub fn new(
        id: String,
        name: String,
        config: MultisigConfig,
        public_keys: Vec<PublicKey>,
        private_key: Option<PrivateKey>,
        base_wallet: Arc<Wallet>,
    ) -> Result<Self, Error> {
        // 設定の検証
        if config.required_signatures > config.total_keys {
            return Err(Error::InvalidConfig("必要な署名数が合計キー数を超えています".to_string()));
        }
        
        if config.total_keys != public_keys.len() {
            return Err(Error::InvalidConfig(format!(
                "合計キー数 ({}) と公開キーリストの長さ ({}) が一致しません",
                config.total_keys, public_keys.len()
            )));
        }
        
        // 階層設定の検証
        if let Some(hierarchy) = &config.approval_hierarchy {
            let mut total_keys = 0;
            for level in &hierarchy.levels {
                total_keys += level.keys.len();
            }
            
            if total_keys != public_keys.len() {
                return Err(Error::InvalidConfig(format!(
                    "階層内のキー数の合計 ({}) と公開キーリストの長さ ({}) が一致しません",
                    total_keys, public_keys.len()
                )));
            }
        }
        
        Ok(Self {
            id,
            name,
            config,
            public_keys,
            private_key,
            transactions: Arc::new(Mutex::new(HashMap::new())),
            daily_spending: Arc::new(Mutex::new(HashMap::new())),
            transaction_steps: Arc::new(Mutex::new(HashMap::new())),
            transaction_history: Arc::new(Mutex::new(HashMap::new())),
            base_wallet,
        })
    }
    
    /// トランザクションを作成
    pub fn create_transaction(
        &self,
        transaction: Transaction,
        creator_key: &PrivateKey,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<MultisigTransaction, Error> {
        // 作成者の公開キーを取得
        let creator_public_key = creator_key.to_public_key()?;
        
        // 作成者がウォレットのメンバーであることを確認
        if !self.public_keys.contains(&creator_public_key) {
            return Err(Error::Unauthorized("作成者はウォレットのメンバーではありません".to_string()));
        }
        
        // トランザクションの検証
        self.validate_transaction(&transaction)?;
        
        // トランザクションIDを生成
        let transaction_id = format!("multisig-{}", hash(&format!("{:?}-{}", transaction, Utc::now())));
        
        // 署名を作成
        let signature = creator_key.sign(&transaction_id)?;
        let mut signatures = HashMap::new();
        signatures.insert(creator_public_key.clone(), signature);
        
        // タイムロック解除時刻を計算
        let timelock_release_at = self.config.timelock_seconds.map(|seconds| {
            Utc::now() + chrono::Duration::seconds(seconds as i64)
        });
        
        // マルチシグトランザクションを作成
        let multisig_tx = MultisigTransaction {
            id: transaction_id.clone(),
            creator: creator_public_key.clone(),
            created_at: Utc::now(),
            executed_at: None,
            timelock_release_at,
            signatures,
            rejections: HashMap::new(),
            status: if timelock_release_at.is_some() {
                MultisigTransactionStatus::TimeLocked
            } else {
                MultisigTransactionStatus::Pending
            },
            transaction,
            metadata: metadata.unwrap_or_default(),
        };
        
        // トランザクションを保存
        let mut transactions = self.transactions.lock().unwrap();
        transactions.insert(transaction_id.clone(), multisig_tx.clone());
        
        // トランザクション処理ステップを初期化
        let mut steps = Vec::new();
        steps.push(TransactionStep {
            name: "作成".to_string(),
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            shard_id: None,
            status: TransactionStepStatus::Completed,
            details: Some("トランザクションが作成されました".to_string()),
        });
        
        if timelock_release_at.is_some() {
            steps.push(TransactionStep {
                name: "タイムロック".to_string(),
                start_time: Utc::now(),
                end_time: None,
                shard_id: None,
                status: TransactionStepStatus::InProgress,
                details: Some(format!(
                    "タイムロック解除時刻: {}",
                    timelock_release_at.unwrap().to_rfc3339()
                )),
            });
        }
        
        steps.push(TransactionStep {
            name: "署名収集".to_string(),
            start_time: Utc::now(),
            end_time: None,
            shard_id: None,
            status: TransactionStepStatus::InProgress,
            details: Some(format!(
                "署名: 1/{} ({})",
                self.config.required_signatures,
                creator_public_key
            )),
        });
        
        let mut transaction_steps = self.transaction_steps.lock().unwrap();
        transaction_steps.insert(transaction_id.clone(), steps);
        
        // トランザクション履歴を記録
        let history_entry = TransactionHistoryEntry {
            timestamp: Utc::now(),
            action: TransactionAction::Created,
            actor: Some(creator_public_key),
            details: Some("トランザクションが作成されました".to_string()),
        };
        
        let mut transaction_history = self.transaction_history.lock().unwrap();
        transaction_history.insert(transaction_id.clone(), vec![history_entry]);
        
        // 通知を送信
        if self.config.notification_settings.on_transaction_created {
            self.send_notification(
                "マルチシグトランザクションが作成されました",
                &format!("トランザクションID: {}", transaction_id),
            )?;
        }
        
        Ok(multisig_tx)
    }
    
    /// トランザクションに署名
    pub fn sign_transaction(
        &self,
        transaction_id: &str,
        signer_key: &PrivateKey,
    ) -> Result<MultisigTransaction, Error> {
        // 署名者の公開キーを取得
        let signer_public_key = signer_key.to_public_key()?;
        
        // 署名者がウォレットのメンバーであることを確認
        if !self.public_keys.contains(&signer_public_key) {
            return Err(Error::Unauthorized("署名者はウォレットのメンバーではありません".to_string()));
        }
        
        // トランザクションを取得
        let mut transactions = self.transactions.lock().unwrap();
        let multisig_tx = transactions.get_mut(transaction_id).ok_or_else(|| {
            Error::NotFound(format!("トランザクションID {} が見つかりません", transaction_id))
        })?;
        
        // トランザクションのステータスを確認
        if multisig_tx.status != MultisigTransactionStatus::Pending && 
           multisig_tx.status != MultisigTransactionStatus::TimeLocked {
            return Err(Error::InvalidState(format!(
                "トランザクションは {} 状態であり、署名できません",
                format!("{:?}", multisig_tx.status)
            )));
        }
        
        // 既に署名済みかどうかを確認
        if multisig_tx.signatures.contains_key(&signer_public_key) {
            return Err(Error::AlreadyExists("既に署名済みです".to_string()));
        }
        
        // 署名を作成
        let signature = signer_key.sign(transaction_id)?;
        multisig_tx.signatures.insert(signer_public_key.clone(), signature);
        
        // トランザクション処理ステップを更新
        let mut transaction_steps = self.transaction_steps.lock().unwrap();
        let steps = transaction_steps.get_mut(transaction_id).ok_or_else(|| {
            Error::NotFound(format!("トランザクションステップが見つかりません: {}", transaction_id))
        })?;
        
        // 署名収集ステップを更新
        for step in steps.iter_mut() {
            if step.name == "署名収集" {
                step.details = Some(format!(
                    "署名: {}/{} (最新の署名者: {})",
                    multisig_tx.signatures.len(),
                    self.config.required_signatures,
                    signer_public_key
                ));
                
                // 必要な署名数が集まった場合
                if multisig_tx.signatures.len() >= self.config.required_signatures {
                    step.end_time = Some(Utc::now());
                    step.status = TransactionStepStatus::Completed;
                }
                
                break;
            }
        }
        
        // トランザクション履歴を記録
        let history_entry = TransactionHistoryEntry {
            timestamp: Utc::now(),
            action: TransactionAction::SignatureAdded,
            actor: Some(signer_public_key),
            details: Some(format!(
                "署名が追加されました ({}/{})",
                multisig_tx.signatures.len(),
                self.config.required_signatures
            )),
        };
        
        let mut transaction_history = self.transaction_history.lock().unwrap();
        let history = transaction_history.entry(transaction_id.to_string())
            .or_insert_with(Vec::new);
        history.push(history_entry);
        
        // 通知を送信
        if self.config.notification_settings.on_signature_added {
            self.send_notification(
                "マルチシグトランザクションに署名が追加されました",
                &format!("トランザクションID: {}", transaction_id),
            )?;
        }
        
        // 必要な署名数が集まったかどうかを確認
        if self.has_required_signatures(multisig_tx) {
            // タイムロックを確認
            if let Some(release_time) = multisig_tx.timelock_release_at {
                if Utc::now() < release_time {
                    // まだタイムロック中
                    return Ok(multisig_tx.clone());
                }
            }
            
            // トランザクションを実行
            self.execute_transaction(multisig_tx)?;
        }
        
        Ok(multisig_tx.clone())
    }
    
    /// トランザクションを拒否
    pub fn reject_transaction(
        &self,
        transaction_id: &str,
        rejecter_key: &PrivateKey,
        reason: Option<String>,
    ) -> Result<MultisigTransaction, Error> {
        // 拒否者の公開キーを取得
        let rejecter_public_key = rejecter_key.to_public_key()?;
        
        // 拒否者がウォレットのメンバーであることを確認
        if !self.public_keys.contains(&rejecter_public_key) {
            return Err(Error::Unauthorized("拒否者はウォレットのメンバーではありません".to_string()));
        }
        
        // トランザクションを取得
        let mut transactions = self.transactions.lock().unwrap();
        let multisig_tx = transactions.get_mut(transaction_id).ok_or_else(|| {
            Error::NotFound(format!("トランザクションID {} が見つかりません", transaction_id))
        })?;
        
        // トランザクションのステータスを確認
        if multisig_tx.status != MultisigTransactionStatus::Pending && 
           multisig_tx.status != MultisigTransactionStatus::TimeLocked {
            return Err(Error::InvalidState(format!(
                "トランザクションは {} 状態であり、拒否できません",
                format!("{:?}", multisig_tx.status)
            )));
        }
        
        // 拒否理由を設定
        let rejection_reason = reason.unwrap_or_else(|| "拒否理由なし".to_string());
        multisig_tx.rejections.insert(rejecter_public_key.clone(), rejection_reason.clone());
        
        // 拒否数が（合計キー数 - 必要な署名数 + 1）以上になった場合、トランザクションを拒否
        let rejection_threshold = self.config.total_keys - self.config.required_signatures + 1;
        if multisig_tx.rejections.len() >= rejection_threshold {
            multisig_tx.status = MultisigTransactionStatus::Rejected;
            
            // トランザクション処理ステップを更新
            let mut transaction_steps = self.transaction_steps.lock().unwrap();
            let steps = transaction_steps.get_mut(transaction_id).ok_or_else(|| {
                Error::NotFound(format!("トランザクションステップが見つかりません: {}", transaction_id))
            })?;
            
            // 署名収集ステップを更新
            for step in steps.iter_mut() {
                if step.name == "署名収集" {
                    step.end_time = Some(Utc::now());
                    step.status = TransactionStepStatus::Failed;
                    step.details = Some(format!(
                        "トランザクションが拒否されました: {}",
                        rejection_reason
                    ));
                    break;
                }
            }
            
            // 拒否ステップを追加
            steps.push(TransactionStep {
                name: "拒否".to_string(),
                start_time: Utc::now(),
                end_time: Some(Utc::now()),
                shard_id: None,
                status: TransactionStepStatus::Completed,
                details: Some(format!(
                    "トランザクションが拒否されました: {}",
                    rejection_reason
                )),
            });
            
            // トランザクション履歴を記録
            let history_entry = TransactionHistoryEntry {
                timestamp: Utc::now(),
                action: TransactionAction::Rejected,
                actor: Some(rejecter_public_key),
                details: Some(format!(
                    "トランザクションが拒否されました: {}",
                    rejection_reason
                )),
            };
            
            let mut transaction_history = self.transaction_history.lock().unwrap();
            let history = transaction_history.entry(transaction_id.to_string())
                .or_insert_with(Vec::new);
            history.push(history_entry);
            
            // 通知を送信
            if self.config.notification_settings.on_transaction_rejected {
                self.send_notification(
                    "マルチシグトランザクションが拒否されました",
                    &format!("トランザクションID: {}", transaction_id),
                )?;
            }
        } else {
            // トランザクション履歴を記録
            let history_entry = TransactionHistoryEntry {
                timestamp: Utc::now(),
                action: TransactionAction::Rejected,
                actor: Some(rejecter_public_key),
                details: Some(format!(
                    "拒否が追加されました: {} ({}/{})",
                    rejection_reason,
                    multisig_tx.rejections.len(),
                    rejection_threshold
                )),
            };
            
            let mut transaction_history = self.transaction_history.lock().unwrap();
            let history = transaction_history.entry(transaction_id.to_string())
                .or_insert_with(Vec::new);
            history.push(history_entry);
        }
        
        Ok(multisig_tx.clone())
    }
    
    /// トランザクションをキャンセル
    pub fn cancel_transaction(
        &self,
        transaction_id: &str,
        canceller_key: &PrivateKey,
    ) -> Result<MultisigTransaction, Error> {
        // キャンセル者の公開キーを取得
        let canceller_public_key = canceller_key.to_public_key()?;
        
        // キャンセル者がウォレットのメンバーであることを確認
        if !self.public_keys.contains(&canceller_public_key) {
            return Err(Error::Unauthorized("キャンセル者はウォレットのメンバーではありません".to_string()));
        }
        
        // トランザクションを取得
        let mut transactions = self.transactions.lock().unwrap();
        let multisig_tx = transactions.get_mut(transaction_id).ok_or_else(|| {
            Error::NotFound(format!("トランザクションID {} が見つかりません", transaction_id))
        })?;
        
        // トランザクションのステータスを確認
        if multisig_tx.status != MultisigTransactionStatus::Pending && 
           multisig_tx.status != MultisigTransactionStatus::TimeLocked {
            return Err(Error::InvalidState(format!(
                "トランザクションは {} 状態であり、キャンセルできません",
                format!("{:?}", multisig_tx.status)
            )));
        }
        
        // 作成者またはすでに署名した人のみがキャンセル可能
        if multisig_tx.creator != canceller_public_key && !multisig_tx.signatures.contains_key(&canceller_public_key) {
            return Err(Error::Unauthorized("キャンセル権限がありません".to_string()));
        }
        
        // トランザクションをキャンセル
        multisig_tx.status = MultisigTransactionStatus::Cancelled;
        
        // トランザクション処理ステップを更新
        let mut transaction_steps = self.transaction_steps.lock().unwrap();
        let steps = transaction_steps.get_mut(transaction_id).ok_or_else(|| {
            Error::NotFound(format!("トランザクションステップが見つかりません: {}", transaction_id))
        })?;
        
        // 署名収集ステップを更新
        for step in steps.iter_mut() {
            if step.name == "署名収集" || step.name == "タイムロック" {
                if step.status == TransactionStepStatus::InProgress {
                    step.end_time = Some(Utc::now());
                    step.status = TransactionStepStatus::Skipped;
                    step.details = Some("トランザクションがキャンセルされました".to_string());
                }
            }
        }
        
        // キャンセルステップを追加
        steps.push(TransactionStep {
            name: "キャンセル".to_string(),
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            shard_id: None,
            status: TransactionStepStatus::Completed,
            details: Some("トランザクションがキャンセルされました".to_string()),
        });
        
        // トランザクション履歴を記録
        let history_entry = TransactionHistoryEntry {
            timestamp: Utc::now(),
            action: TransactionAction::Cancelled,
            actor: Some(canceller_public_key),
            details: Some("トランザクションがキャンセルされました".to_string()),
        };
        
        let mut transaction_history = self.transaction_history.lock().unwrap();
        let history = transaction_history.entry(transaction_id.to_string())
            .or_insert_with(Vec::new);
        history.push(history_entry);
        
        // 通知を送信
        self.send_notification(
            "マルチシグトランザクションがキャンセルされました",
            &format!("トランザクションID: {}", transaction_id),
        )?;
        
        Ok(multisig_tx.clone())
    }
    
    /// トランザクションを取得
    pub fn get_transaction(&self, transaction_id: &str) -> Result<MultisigTransaction, Error> {
        let transactions = self.transactions.lock().unwrap();
        let multisig_tx = transactions.get(transaction_id).ok_or_else(|| {
            Error::NotFound(format!("トランザクションID {} が見つかりません", transaction_id))
        })?;
        
        Ok(multisig_tx.clone())
    }
    
    /// 全トランザクションを取得
    pub fn get_all_transactions(&self) -> Result<Vec<MultisigTransaction>, Error> {
        let transactions = self.transactions.lock().unwrap();
        let all_transactions = transactions.values().cloned().collect();
        
        Ok(all_transactions)
    }
    
    /// 保留中のトランザクションを取得
    pub fn get_pending_transactions(&self) -> Result<Vec<MultisigTransaction>, Error> {
        let transactions = self.transactions.lock().unwrap();
        let pending_transactions = transactions.values()
            .filter(|tx| tx.status == MultisigTransactionStatus::Pending || tx.status == MultisigTransactionStatus::TimeLocked)
            .cloned()
            .collect();
        
        Ok(pending_transactions)
    }
    
    /// トランザクション処理ステップを取得
    pub fn get_transaction_steps(&self, transaction_id: &str) -> Result<Vec<TransactionStep>, Error> {
        let transaction_steps = self.transaction_steps.lock().unwrap();
        let steps = transaction_steps.get(transaction_id).ok_or_else(|| {
            Error::NotFound(format!("トランザクションステップが見つかりません: {}", transaction_id))
        })?;
        
        Ok(steps.clone())
    }
    
    /// トランザクション履歴を取得
    pub fn get_transaction_history(&self, transaction_id: &str) -> Result<Vec<TransactionHistoryEntry>, Error> {
        let transaction_history = self.transaction_history.lock().unwrap();
        let history = transaction_history.get(transaction_id).ok_or_else(|| {
            Error::NotFound(format!("トランザクション履歴が見つかりません: {}", transaction_id))
        })?;
        
        Ok(history.clone())
    }
    
    /// タイムロックの解除を確認
    pub fn check_timelock_releases(&self) -> Result<Vec<MultisigTransaction>, Error> {
        let mut transactions = self.transactions.lock().unwrap();
        let now = Utc::now();
        let mut released_transactions = Vec::new();
        
        for multisig_tx in transactions.values_mut() {
            // タイムロック中のトランザクションを確認
            if multisig_tx.status == MultisigTransactionStatus::TimeLocked {
                if let Some(release_time) = multisig_tx.timelock_release_at {
                    if now >= release_time {
                        // タイムロックが解除された
                        multisig_tx.status = MultisigTransactionStatus::Pending;
                        
                        // トランザクション処理ステップを更新
                        let mut transaction_steps = self.transaction_steps.lock().unwrap();
                        if let Some(steps) = transaction_steps.get_mut(&multisig_tx.id) {
                            for step in steps.iter_mut() {
                                if step.name == "タイムロック" {
                                    step.end_time = Some(now);
                                    step.status = TransactionStepStatus::Completed;
                                    step.details = Some("タイムロックが解除されました".to_string());
                                    break;
                                }
                            }
                        }
                        
                        // トランザクション履歴を記録
                        let history_entry = TransactionHistoryEntry {
                            timestamp: now,
                            action: TransactionAction::TimeLockReleased,
                            actor: None,
                            details: Some("タイムロックが解除されました".to_string()),
                        };
                        
                        let mut transaction_history = self.transaction_history.lock().unwrap();
                        let history = transaction_history.entry(multisig_tx.id.clone())
                            .or_insert_with(Vec::new);
                        history.push(history_entry);
                        
                        released_transactions.push(multisig_tx.clone());
                        
                        // 必要な署名数が集まっている場合は実行
                        if self.has_required_signatures(multisig_tx) {
                            self.execute_transaction(multisig_tx)?;
                        }
                        
                        // 通知を送信
                        if self.config.notification_settings.on_timelock_released {
                            self.send_notification(
                                "マルチシグトランザクションのタイムロックが解除されました",
                                &format!("トランザクションID: {}", multisig_tx.id),
                            )?;
                        }
                    }
                }
            }
        }
        
        Ok(released_transactions)
    }
    
    /// 日次送金額の更新
    pub fn update_daily_spending(&self, amount: f64) -> Result<(), Error> {
        let mut daily_spending = self.daily_spending.lock().unwrap();
        let today = Utc::now().format("%Y-%m-%d").to_string();
        
        let current_amount = daily_spending.get(&today).cloned().unwrap_or(0.0);
        daily_spending.insert(today, current_amount + amount);
        
        Ok(())
    }
    
    /// 日次送金額の確認
    pub fn check_daily_limit(&self, amount: f64) -> Result<bool, Error> {
        if let Some(daily_limit) = self.config.daily_limit {
            let daily_spending = self.daily_spending.lock().unwrap();
            let today = Utc::now().format("%Y-%m-%d").to_string();
            
            let current_amount = daily_spending.get(&today).cloned().unwrap_or(0.0);
            if current_amount + amount > daily_limit {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// トランザクション送金額の確認
    pub fn check_transaction_limit(&self, amount: f64) -> Result<bool, Error> {
        if let Some(transaction_limit) = self.config.transaction_limit {
            if amount > transaction_limit {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// 必要な署名数が集まっているかどうかを確認
    fn has_required_signatures(&self, transaction: &MultisigTransaction) -> bool {
        // 階層的承認が設定されている場合
        if let Some(hierarchy) = &self.config.approval_hierarchy {
            // 各階層レベルで必要な署名数を確認
            for level in &hierarchy.levels {
                let level_signatures = transaction.signatures.keys()
                    .filter(|key| level.keys.contains(key))
                    .count();
                
                if level_signatures < level.required_signatures {
                    return false;
                }
                
                // 金額制限を確認
                if let Some(max_amount) = level.max_amount {
                    if transaction.transaction.amount > max_amount {
                        // このレベルでは承認できない金額
                        continue;
                    }
                }
            }
            
            return true;
        } else {
            // 通常のM-of-N承認
            return transaction.signatures.len() >= self.config.required_signatures;
        }
    }
    
    /// トランザクションを実行
    fn execute_transaction(&self, transaction: &mut MultisigTransaction) -> Result<(), Error> {
        // トランザクションのステータスを確認
        if transaction.status != MultisigTransactionStatus::Pending && 
           transaction.status != MultisigTransactionStatus::TimeLocked {
            return Err(Error::InvalidState(format!(
                "トランザクションは {} 状態であり、実行できません",
                format!("{:?}", transaction.status)
            )));
        }
        
        // 基本ウォレットを使用してトランザクションを送信
        self.base_wallet.send_transaction(&transaction.transaction)?;
        
        // トランザクションのステータスを更新
        transaction.status = MultisigTransactionStatus::Executed;
        transaction.executed_at = Some(Utc::now());
        
        // トランザクション処理ステップを更新
        let mut transaction_steps = self.transaction_steps.lock().unwrap();
        if let Some(steps) = transaction_steps.get_mut(&transaction.id) {
            // 実行ステップを追加
            steps.push(TransactionStep {
                name: "実行".to_string(),
                start_time: Utc::now(),
                end_time: Some(Utc::now()),
                shard_id: Some(transaction.transaction.shard_id.clone()),
                status: TransactionStepStatus::Completed,
                details: Some("トランザクションが実行されました".to_string()),
            });
        }
        
        // トランザクション履歴を記録
        let history_entry = TransactionHistoryEntry {
            timestamp: Utc::now(),
            action: TransactionAction::Executed,
            actor: None,
            details: Some("トランザクションが実行されました".to_string()),
        };
        
        let mut transaction_history = self.transaction_history.lock().unwrap();
        let history = transaction_history.entry(transaction.id.clone())
            .or_insert_with(Vec::new);
        history.push(history_entry);
        
        // 日次送金額を更新
        self.update_daily_spending(transaction.transaction.amount)?;
        
        // 通知を送信
        if self.config.notification_settings.on_transaction_executed {
            self.send_notification(
                "マルチシグトランザクションが実行されました",
                &format!("トランザクションID: {}", transaction.id),
            )?;
        }
        
        Ok(())
    }
    
    /// トランザクションを検証
    fn validate_transaction(&self, transaction: &Transaction) -> Result<(), Error> {
        // 送金額の制限を確認
        if !self.check_transaction_limit(transaction.amount)? {
            return Err(Error::LimitExceeded(format!(
                "トランザクション金額 {} が制限を超えています",
                transaction.amount
            )));
        }
        
        // 日次送金額の制限を確認
        if !self.check_daily_limit(transaction.amount)? {
            return Err(Error::LimitExceeded("日次送金額の制限を超えています".to_string()));
        }
        
        // 拒否ルールを確認
        for rule in &self.config.rejection_rules {
            let mut should_reject = true;
            
            // 送信先アドレスを確認
            if let Some(addr) = &rule.destination_address {
                if &transaction.to_address != addr {
                    should_reject = false;
                }
            }
            
            // 最小金額を確認
            if let Some(min_amount) = rule.min_amount {
                if transaction.amount < min_amount {
                    should_reject = false;
                }
            }
            
            // トランザクションタイプを確認
            if let Some(tx_type) = &rule.transaction_type {
                if &transaction.transaction_type != tx_type {
                    should_reject = false;
                }
            }
            
            if should_reject {
                return Err(Error::RejectionRule(format!(
                    "トランザクションは拒否ルール '{}' に一致します",
                    rule.name
                )));
            }
        }
        
        Ok(())
    }
    
    /// 通知を送信
    fn send_notification(&self, title: &str, message: &str) -> Result<(), Error> {
        // 実際の実装では、設定された通知先に通知を送信
        // ここでは簡易的な実装としてログに出力
        info!("通知: {} - {}", title, message);
        
        for destination in &self.config.notification_settings.notification_destinations {
            match destination.notification_type {
                crate::wallet::multisig::config::NotificationType::Email => {
                    // Eメール送信処理
                    debug!("Eメール通知: {} -> {}", destination.destination, message);
                }
                crate::wallet::multisig::config::NotificationType::Webhook => {
                    // Webhook送信処理
                    debug!("Webhook通知: {} -> {}", destination.destination, message);
                }
                crate::wallet::multisig::config::NotificationType::OnChainMessage => {
                    // オンチェーンメッセージ送信処理
                    debug!("オンチェーン通知: {} -> {}", destination.destination, message);
                }
            }
        }
        
        Ok(())
    }
}