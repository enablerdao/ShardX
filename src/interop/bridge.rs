use crate::error::Error;
use crate::interop::{ChainType, ChainConfig};
use crate::interop::wrapped_assets::AssetMapping;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};
use tokio::sync::mpsc;
use uuid::Uuid;

/// ブリッジ設定
#[derive(Clone, Debug)]
pub struct BridgeConfig {
    /// ブリッジ名
    pub name: String,
    /// 手数料率（パーセント）
    pub fee_rate: f64,
    /// 最小転送量
    pub min_transfer_amount: u64,
    /// 最大転送量
    pub max_transfer_amount: u64,
    /// 転送遅延（秒）
    pub transfer_delay_seconds: u64,
    /// 自動承認フラグ
    pub auto_approve: bool,
    /// リレイヤーアドレス
    pub relayer_addresses: Vec<String>,
    /// 必要な署名数
    pub required_signatures: u32,
}

/// ブリッジトランザクション
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BridgeTransaction {
    /// トランザクションID
    pub id: String,
    /// ソースチェーン
    pub source_chain: ChainType,
    /// ターゲットチェーン
    pub target_chain: ChainType,
    /// 送信者
    pub sender: String,
    /// 受信者
    pub recipient: String,
    /// アセットID
    pub asset_id: String,
    /// 金額
    pub amount: u64,
    /// 手数料
    pub fee: u64,
    /// ノンス
    pub nonce: u64,
    /// ステータス
    pub status: BridgeTransactionStatus,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 更新日時
    pub updated_at: DateTime<Utc>,
    /// ソーストランザクションID
    pub source_tx_id: Option<String>,
    /// ターゲットトランザクションID
    pub target_tx_id: Option<String>,
    /// 署名
    pub signatures: Vec<String>,
}

/// ブリッジトランザクションステータス
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeTransactionStatus {
    /// 初期化
    Initiated,
    /// ソースチェーンで確認済み
    SourceConfirmed,
    /// 承認済み
    Approved,
    /// ターゲットチェーンで処理中
    Processing,
    /// 完了
    Completed,
    /// 失敗
    Failed,
    /// キャンセル
    Cancelled,
}

/// ブリッジイベント
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BridgeEvent {
    /// イベントID
    pub id: String,
    /// イベントタイプ
    pub event_type: BridgeEventType,
    /// トランザクションID
    pub transaction_id: String,
    /// ソースチェーン
    pub source_chain: ChainType,
    /// ターゲットチェーン
    pub target_chain: ChainType,
    /// 送信者
    pub sender: String,
    /// 受信者
    pub recipient: String,
    /// アセットID
    pub asset_id: String,
    /// 金額
    pub amount: u64,
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// 追加データ
    pub additional_data: Option<serde_json::Value>,
}

/// ブリッジイベントタイプ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeEventType {
    /// デポジット
    Deposit,
    /// 承認
    Approval,
    /// 引き出し
    Withdrawal,
    /// 失敗
    Failure,
}

/// イベントモニター
struct EventMonitor {
    /// モニターID
    id: String,
    /// コールバック送信チャネル
    callback_tx: mpsc::Sender<BridgeEvent>,
    /// 実行中フラグ
    running: bool,
}

/// ブリッジ
pub struct Bridge {
    /// ソースチェーン
    source_chain: ChainType,
    /// ターゲットチェーン
    target_chain: ChainType,
    /// ソースチェーン設定
    source_config: ChainConfig,
    /// ターゲットチェーン設定
    target_config: ChainConfig,
    /// ブリッジ設定
    config: BridgeConfig,
    /// トランザクション
    transactions: Arc<Mutex<HashMap<String, BridgeTransaction>>>,
    /// イベントモニター
    event_monitors: Arc<Mutex<HashMap<String, EventMonitor>>>,
}

impl Bridge {
    /// 新しいBridgeを作成
    pub fn new(
        source_chain: ChainType,
        target_chain: ChainType,
        source_config: ChainConfig,
        target_config: ChainConfig,
        config: BridgeConfig,
    ) -> Result<Self, Error> {
        Ok(Self {
            source_chain,
            target_chain,
            source_config,
            target_config,
            config,
            transactions: Arc::new(Mutex::new(HashMap::new())),
            event_monitors: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// アセットをラップ
    pub async fn wrap_asset(
        &self,
        asset_mapping: &AssetMapping,
        amount: u64,
        sender: &str,
        recipient: &str,
    ) -> Result<String, Error> {
        // 金額の検証
        if amount < self.config.min_transfer_amount {
            return Err(Error::InvalidArgument(format!(
                "Amount is below minimum: {} < {}",
                amount,
                self.config.min_transfer_amount
            )));
        }
        
        if amount > self.config.max_transfer_amount {
            return Err(Error::InvalidArgument(format!(
                "Amount exceeds maximum: {} > {}",
                amount,
                self.config.max_transfer_amount
            )));
        }
        
        // 手数料の計算
        let fee = (amount as f64 * self.config.fee_rate / 100.0) as u64;
        
        // トランザクションの作成
        let tx_id = Uuid::new_v4().to_string();
        let transaction = BridgeTransaction {
            id: tx_id.clone(),
            source_chain: self.source_chain.clone(),
            target_chain: self.target_chain.clone(),
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            asset_id: asset_mapping.source_asset_id.clone(),
            amount,
            fee,
            nonce: rand::random::<u64>(),
            status: BridgeTransactionStatus::Initiated,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            source_tx_id: None,
            target_tx_id: None,
            signatures: Vec::new(),
        };
        
        // トランザクションを保存
        {
            let mut transactions = self.transactions.lock().unwrap();
            transactions.insert(tx_id.clone(), transaction.clone());
        }
        
        // ソースチェーンでのトランザクション実行
        let source_tx_id = self.execute_source_transaction(&transaction).await?;
        
        // トランザクションを更新
        {
            let mut transactions = self.transactions.lock().unwrap();
            if let Some(tx) = transactions.get_mut(&tx_id) {
                tx.source_tx_id = Some(source_tx_id);
                tx.status = BridgeTransactionStatus::SourceConfirmed;
                tx.updated_at = Utc::now();
            }
        }
        
        // イベントを発行
        let event = BridgeEvent {
            id: Uuid::new_v4().to_string(),
            event_type: BridgeEventType::Deposit,
            transaction_id: tx_id.clone(),
            source_chain: self.source_chain.clone(),
            target_chain: self.target_chain.clone(),
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            asset_id: asset_mapping.source_asset_id.clone(),
            amount,
            timestamp: Utc::now(),
            additional_data: None,
        };
        
        self.emit_event(event).await?;
        
        // 自動承認の場合は処理を続行
        if self.config.auto_approve {
            self.process_transaction(&tx_id).await?;
        }
        
        Ok(tx_id)
    }
    
    /// アセットをアンラップ
    pub async fn unwrap_asset(
        &self,
        asset_mapping: &AssetMapping,
        amount: u64,
        sender: &str,
        recipient: &str,
    ) -> Result<String, Error> {
        // 金額の検証
        if amount < self.config.min_transfer_amount {
            return Err(Error::InvalidArgument(format!(
                "Amount is below minimum: {} < {}",
                amount,
                self.config.min_transfer_amount
            )));
        }
        
        if amount > self.config.max_transfer_amount {
            return Err(Error::InvalidArgument(format!(
                "Amount exceeds maximum: {} > {}",
                amount,
                self.config.max_transfer_amount
            )));
        }
        
        // 手数料の計算
        let fee = (amount as f64 * self.config.fee_rate / 100.0) as u64;
        
        // トランザクションの作成
        let tx_id = Uuid::new_v4().to_string();
        let transaction = BridgeTransaction {
            id: tx_id.clone(),
            source_chain: self.target_chain.clone(), // 逆方向
            target_chain: self.source_chain.clone(), // 逆方向
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            asset_id: asset_mapping.target_asset_id.clone(),
            amount,
            fee,
            nonce: rand::random::<u64>(),
            status: BridgeTransactionStatus::Initiated,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            source_tx_id: None,
            target_tx_id: None,
            signatures: Vec::new(),
        };
        
        // トランザクションを保存
        {
            let mut transactions = self.transactions.lock().unwrap();
            transactions.insert(tx_id.clone(), transaction.clone());
        }
        
        // ソースチェーンでのトランザクション実行
        let source_tx_id = self.execute_source_transaction(&transaction).await?;
        
        // トランザクションを更新
        {
            let mut transactions = self.transactions.lock().unwrap();
            if let Some(tx) = transactions.get_mut(&tx_id) {
                tx.source_tx_id = Some(source_tx_id);
                tx.status = BridgeTransactionStatus::SourceConfirmed;
                tx.updated_at = Utc::now();
            }
        }
        
        // イベントを発行
        let event = BridgeEvent {
            id: Uuid::new_v4().to_string(),
            event_type: BridgeEventType::Deposit,
            transaction_id: tx_id.clone(),
            source_chain: self.target_chain.clone(), // 逆方向
            target_chain: self.source_chain.clone(), // 逆方向
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            asset_id: asset_mapping.target_asset_id.clone(),
            amount,
            timestamp: Utc::now(),
            additional_data: None,
        };
        
        self.emit_event(event).await?;
        
        // 自動承認の場合は処理を続行
        if self.config.auto_approve {
            self.process_transaction(&tx_id).await?;
        }
        
        Ok(tx_id)
    }
    
    /// トランザクションを承認
    pub async fn approve_transaction(
        &self,
        transaction_id: &str,
        approver: &str,
        signature: &str,
    ) -> Result<(), Error> {
        // トランザクションを取得
        let mut transaction = {
            let transactions = self.transactions.lock().unwrap();
            transactions.get(transaction_id)
                .cloned()
                .ok_or_else(|| Error::NotFound(format!("Transaction not found: {}", transaction_id)))?
        };
        
        // ステータスをチェック
        if transaction.status != BridgeTransactionStatus::SourceConfirmed {
            return Err(Error::InvalidState(format!(
                "Transaction is not in SourceConfirmed state: {:?}",
                transaction.status
            )));
        }
        
        // 署名を検証
        if !self.verify_signature(&transaction, approver, signature)? {
            return Err(Error::AuthenticationError("Invalid signature".to_string()));
        }
        
        // 署名を追加
        transaction.signatures.push(signature.to_string());
        
        // 必要な署名数をチェック
        if transaction.signatures.len() >= self.config.required_signatures as usize {
            transaction.status = BridgeTransactionStatus::Approved;
        }
        
        // トランザクションを更新
        {
            let mut transactions = self.transactions.lock().unwrap();
            if let Some(tx) = transactions.get_mut(transaction_id) {
                tx.signatures = transaction.signatures.clone();
                tx.status = transaction.status.clone();
                tx.updated_at = Utc::now();
            }
        }
        
        // イベントを発行
        let event = BridgeEvent {
            id: Uuid::new_v4().to_string(),
            event_type: BridgeEventType::Approval,
            transaction_id: transaction_id.to_string(),
            source_chain: transaction.source_chain.clone(),
            target_chain: transaction.target_chain.clone(),
            sender: transaction.sender.clone(),
            recipient: transaction.recipient.clone(),
            asset_id: transaction.asset_id.clone(),
            amount: transaction.amount,
            timestamp: Utc::now(),
            additional_data: Some(serde_json::json!({
                "approver": approver,
                "signature_count": transaction.signatures.len(),
                "required_signatures": self.config.required_signatures,
            })),
        };
        
        self.emit_event(event).await?;
        
        // 承認済みの場合は処理を続行
        if transaction.status == BridgeTransactionStatus::Approved {
            self.process_transaction(transaction_id).await?;
        }
        
        Ok(())
    }
    
    /// トランザクションを処理
    async fn process_transaction(&self, transaction_id: &str) -> Result<(), Error> {
        // トランザクションを取得
        let mut transaction = {
            let transactions = self.transactions.lock().unwrap();
            transactions.get(transaction_id)
                .cloned()
                .ok_or_else(|| Error::NotFound(format!("Transaction not found: {}", transaction_id)))?
        };
        
        // ステータスをチェック
        if transaction.status != BridgeTransactionStatus::Approved {
            return Err(Error::InvalidState(format!(
                "Transaction is not in Approved state: {:?}",
                transaction.status
            )));
        }
        
        // ステータスを更新
        transaction.status = BridgeTransactionStatus::Processing;
        
        // トランザクションを更新
        {
            let mut transactions = self.transactions.lock().unwrap();
            if let Some(tx) = transactions.get_mut(transaction_id) {
                tx.status = transaction.status.clone();
                tx.updated_at = Utc::now();
            }
        }
        
        // 遅延を適用
        if self.config.transfer_delay_seconds > 0 {
            tokio::time::sleep(tokio::time::Duration::from_secs(self.config.transfer_delay_seconds)).await;
        }
        
        // ターゲットチェーンでのトランザクション実行
        match self.execute_target_transaction(&transaction).await {
            Ok(target_tx_id) => {
                // トランザクションを更新
                {
                    let mut transactions = self.transactions.lock().unwrap();
                    if let Some(tx) = transactions.get_mut(transaction_id) {
                        tx.target_tx_id = Some(target_tx_id);
                        tx.status = BridgeTransactionStatus::Completed;
                        tx.updated_at = Utc::now();
                    }
                }
                
                // イベントを発行
                let event = BridgeEvent {
                    id: Uuid::new_v4().to_string(),
                    event_type: BridgeEventType::Withdrawal,
                    transaction_id: transaction_id.to_string(),
                    source_chain: transaction.source_chain.clone(),
                    target_chain: transaction.target_chain.clone(),
                    sender: transaction.sender.clone(),
                    recipient: transaction.recipient.clone(),
                    asset_id: transaction.asset_id.clone(),
                    amount: transaction.amount,
                    timestamp: Utc::now(),
                    additional_data: Some(serde_json::json!({
                        "target_tx_id": target_tx_id,
                    })),
                };
                
                self.emit_event(event).await?;
            },
            Err(e) => {
                // トランザクションを更新
                {
                    let mut transactions = self.transactions.lock().unwrap();
                    if let Some(tx) = transactions.get_mut(transaction_id) {
                        tx.status = BridgeTransactionStatus::Failed;
                        tx.updated_at = Utc::now();
                    }
                }
                
                // イベントを発行
                let event = BridgeEvent {
                    id: Uuid::new_v4().to_string(),
                    event_type: BridgeEventType::Failure,
                    transaction_id: transaction_id.to_string(),
                    source_chain: transaction.source_chain.clone(),
                    target_chain: transaction.target_chain.clone(),
                    sender: transaction.sender.clone(),
                    recipient: transaction.recipient.clone(),
                    asset_id: transaction.asset_id.clone(),
                    amount: transaction.amount,
                    timestamp: Utc::now(),
                    additional_data: Some(serde_json::json!({
                        "error": e.to_string(),
                    })),
                };
                
                self.emit_event(event).await?;
                
                return Err(e);
            }
        }
        
        Ok(())
    }
    
    /// ソースチェーンでのトランザクション実行
    async fn execute_source_transaction(&self, transaction: &BridgeTransaction) -> Result<String, Error> {
        // 実際の実装では、ソースチェーンに対してトランザクションを実行
        // ここでは、テスト用のダミーデータを返す
        
        info!(
            "Executing source transaction: {} -> {}, asset: {}, amount: {}",
            transaction.source_chain.to_string(),
            transaction.target_chain.to_string(),
            transaction.asset_id,
            transaction.amount
        );
        
        // ダミーのトランザクションID
        let tx_id = format!("src_tx_{}", Uuid::new_v4());
        
        Ok(tx_id)
    }
    
    /// ターゲットチェーンでのトランザクション実行
    async fn execute_target_transaction(&self, transaction: &BridgeTransaction) -> Result<String, Error> {
        // 実際の実装では、ターゲットチェーンに対してトランザクションを実行
        // ここでは、テスト用のダミーデータを返す
        
        info!(
            "Executing target transaction: {} -> {}, asset: {}, amount: {}",
            transaction.source_chain.to_string(),
            transaction.target_chain.to_string(),
            transaction.asset_id,
            transaction.amount
        );
        
        // ダミーのトランザクションID
        let tx_id = format!("tgt_tx_{}", Uuid::new_v4());
        
        Ok(tx_id)
    }
    
    /// 署名を検証
    fn verify_signature(&self, transaction: &BridgeTransaction, approver: &str, signature: &str) -> Result<bool, Error> {
        // 実際の実装では、署名を暗号学的に検証
        // ここでは、テスト用のダミーロジックを提供
        
        // リレイヤーアドレスをチェック
        if !self.config.relayer_addresses.contains(&approver.to_string()) {
            return Ok(false);
        }
        
        // 署名が既に存在するかチェック
        if transaction.signatures.contains(&signature.to_string()) {
            return Ok(false);
        }
        
        // 署名の長さをチェック
        if signature.len() < 64 {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// イベントを発行
    async fn emit_event(&self, event: BridgeEvent) -> Result<(), Error> {
        let event_monitors = self.event_monitors.lock().unwrap();
        
        for monitor in event_monitors.values() {
            if monitor.running {
                // イベントを送信
                if let Err(e) = monitor.callback_tx.send(event.clone()).await {
                    error!("Failed to send event to monitor: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// イベントを監視
    pub async fn monitor_events<F>(
        &self,
        callback: Box<F>,
    ) -> Result<String, Error>
    where
        F: Fn(BridgeEvent) -> Result<(), Error> + Send + Sync + 'static,
    {
        // モニターIDを生成
        let monitor_id = Uuid::new_v4().to_string();
        
        // チャネルを作成
        let (tx, mut rx) = mpsc::channel(100);
        
        // モニターを作成
        let monitor = EventMonitor {
            id: monitor_id.clone(),
            callback_tx: tx,
            running: true,
        };
        
        // モニターを登録
        {
            let mut event_monitors = self.event_monitors.lock().unwrap();
            event_monitors.insert(monitor_id.clone(), monitor);
        }
        
        // イベント処理タスクを開始
        let monitor_id_clone = monitor_id.clone();
        let event_monitors_clone = self.event_monitors.clone();
        
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                match callback(event) {
                    Ok(_) => {},
                    Err(e) => {
                        error!("Error in event callback: {}", e);
                        
                        // エラーが発生した場合はモニターを停止
                        let mut event_monitors = event_monitors_clone.lock().unwrap();
                        if let Some(monitor) = event_monitors.get_mut(&monitor_id_clone) {
                            monitor.running = false;
                        }
                        
                        break;
                    }
                }
            }
        });
        
        Ok(monitor_id)
    }
    
    /// イベント監視を停止
    pub async fn stop_monitoring(&self, monitor_id: &str) -> Result<(), Error> {
        let mut event_monitors = self.event_monitors.lock().unwrap();
        
        if let Some(monitor) = event_monitors.get_mut(monitor_id) {
            monitor.running = false;
        } else {
            return Err(Error::NotFound(format!("Monitor not found: {}", monitor_id)));
        }
        
        Ok(())
    }
    
    /// トランザクションを取得
    pub fn get_transaction(&self, transaction_id: &str) -> Result<BridgeTransaction, Error> {
        let transactions = self.transactions.lock().unwrap();
        
        transactions.get(transaction_id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Transaction not found: {}", transaction_id)))
    }
    
    /// トランザクションリストを取得
    pub fn get_transactions(&self) -> Vec<BridgeTransaction> {
        let transactions = self.transactions.lock().unwrap();
        transactions.values().cloned().collect()
    }
    
    /// ブリッジ設定を取得
    pub fn get_config(&self) -> &BridgeConfig {
        &self.config
    }
    
    /// ソースチェーンを取得
    pub fn get_source_chain(&self) -> &ChainType {
        &self.source_chain
    }
    
    /// ターゲットチェーンを取得
    pub fn get_target_chain(&self) -> &ChainType {
        &self.target_chain
    }
}

impl std::fmt::Display for ChainType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainType::Ethereum => write!(f, "Ethereum"),
            ChainType::Bitcoin => write!(f, "Bitcoin"),
            ChainType::Polkadot => write!(f, "Polkadot"),
            ChainType::Cosmos => write!(f, "Cosmos"),
            ChainType::Solana => write!(f, "Solana"),
            ChainType::Avalanche => write!(f, "Avalanche"),
            ChainType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interop::wrapped_assets::AssetMapping;
    
    #[tokio::test]
    async fn test_bridge_creation() {
        // チェーン設定
        let source_config = ChainConfig {
            chain_type: ChainType::Ethereum,
            chain_id: "1".to_string(),
            endpoint_url: "https://mainnet.infura.io/v3/your-api-key".to_string(),
            contract_address: Some("0x1234567890abcdef1234567890abcdef12345678".to_string()),
            confirmation_blocks: 12,
            timeout_ms: 30000,
        };
        
        let target_config = ChainConfig {
            chain_type: ChainType::Polkadot,
            chain_id: "polkadot".to_string(),
            endpoint_url: "wss://rpc.polkadot.io".to_string(),
            contract_address: None,
            confirmation_blocks: 6,
            timeout_ms: 30000,
        };
        
        // ブリッジ設定
        let bridge_config = BridgeConfig {
            name: "Ethereum-Polkadot Bridge".to_string(),
            fee_rate: 0.1,
            min_transfer_amount: 100,
            max_transfer_amount: 1000000,
            transfer_delay_seconds: 0,
            auto_approve: true,
            relayer_addresses: vec!["relayer1".to_string(), "relayer2".to_string()],
            required_signatures: 1,
        };
        
        // ブリッジを作成
        let bridge = Bridge::new(
            ChainType::Ethereum,
            ChainType::Polkadot,
            source_config,
            target_config,
            bridge_config,
        );
        
        assert!(bridge.is_ok());
    }
    
    #[tokio::test]
    async fn test_wrap_asset() {
        // チェーン設定
        let source_config = ChainConfig {
            chain_type: ChainType::Ethereum,
            chain_id: "1".to_string(),
            endpoint_url: "https://mainnet.infura.io/v3/your-api-key".to_string(),
            contract_address: Some("0x1234567890abcdef1234567890abcdef12345678".to_string()),
            confirmation_blocks: 12,
            timeout_ms: 30000,
        };
        
        let target_config = ChainConfig {
            chain_type: ChainType::Polkadot,
            chain_id: "polkadot".to_string(),
            endpoint_url: "wss://rpc.polkadot.io".to_string(),
            contract_address: None,
            confirmation_blocks: 6,
            timeout_ms: 30000,
        };
        
        // ブリッジ設定
        let bridge_config = BridgeConfig {
            name: "Ethereum-Polkadot Bridge".to_string(),
            fee_rate: 0.1,
            min_transfer_amount: 100,
            max_transfer_amount: 1000000,
            transfer_delay_seconds: 0,
            auto_approve: true,
            relayer_addresses: vec!["relayer1".to_string(), "relayer2".to_string()],
            required_signatures: 1,
        };
        
        // ブリッジを作成
        let bridge = Bridge::new(
            ChainType::Ethereum,
            ChainType::Polkadot,
            source_config,
            target_config,
            bridge_config,
        ).unwrap();
        
        // アセットマッピング
        let asset_mapping = AssetMapping {
            source_chain: ChainType::Ethereum,
            target_chain: ChainType::Polkadot,
            source_asset_id: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC on Ethereum
            target_asset_id: "usdc".to_string(), // USDC on Polkadot
        };
        
        // アセットをラップ
        let tx_id = bridge.wrap_asset(
            &asset_mapping,
            1000,
            "0x1234567890abcdef1234567890abcdef12345678",
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        ).await;
        
        assert!(tx_id.is_ok());
        
        // トランザクションを取得
        let transaction = bridge.get_transaction(&tx_id.unwrap()).unwrap();
        
        // トランザクションの検証
        assert_eq!(transaction.source_chain, ChainType::Ethereum);
        assert_eq!(transaction.target_chain, ChainType::Polkadot);
        assert_eq!(transaction.amount, 1000);
        assert_eq!(transaction.status, BridgeTransactionStatus::Completed); // auto_approve = true
    }
}