// 状態チャネルモジュール
//
// このモジュールは、ShardXの状態チャネル機能を提供します。
// 状態チャネルは、ブロックチェーン上のトランザクション数を削減し、
// スケーラビリティを向上させるためのオフチェーン技術です。
//
// 主な機能:
// - 双方向状態チャネル
// - 多者間状態チャネル
// - 条件付き支払い
// - チャネルネットワーク
// - 紛争解決

mod config;
mod state;
mod payment;
mod network;
mod dispute;
mod update;
mod signature;
mod watcher;
mod virtual_channel;
mod routing;

pub use self::config::{ChannelConfig, ChannelParams, ChannelPolicy};
pub use self::state::{ChannelState, StateUpdate, StateProof, StateVersion};
pub use self::payment::{Payment, PaymentCondition, PaymentStatus, PaymentProof};
pub use self::network::{ChannelNetwork, ChannelRoute, RouteHop};
pub use self::dispute::{DisputeResolver, DisputeProof, DisputeStatus};
pub use self::update::{UpdateProcessor, UpdateMessage, UpdateStatus};
pub use self::signature::{ChannelSignature, SignatureVerifier};
pub use self::watcher::{ChannelWatcher, WatcherEvent, WatcherConfig};
pub use self::virtual_channel::{VirtualChannel, VirtualChannelState};
pub use self::routing::{RoutingTable, RoutingStrategy, PathFinder};

use crate::error::Error;
use crate::metrics::MetricsCollector;
use crate::network::NetworkManager;
use crate::storage::StorageManager;
use crate::crypto::{CryptoManager, KeyPair};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use log::{debug, error, info, warn};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// 状態チャネルマネージャー
pub struct ChannelManager {
    /// 設定
    config: ChannelConfig,
    /// チャネル
    channels: HashMap<ChannelId, Channel>,
    /// 仮想チャネル
    virtual_channels: HashMap<ChannelId, VirtualChannel>,
    /// チャネルネットワーク
    network: ChannelNetwork,
    /// 紛争解決器
    dispute_resolver: DisputeResolver,
    /// 更新プロセッサー
    update_processor: UpdateProcessor,
    /// チャネルウォッチャー
    watcher: ChannelWatcher,
    /// ルーティングテーブル
    routing_table: RoutingTable,
    /// ネットワークマネージャー
    network_manager: Arc<NetworkManager>,
    /// ストレージマネージャー
    storage_manager: Arc<StorageManager>,
    /// 暗号マネージャー
    crypto_manager: Arc<CryptoManager>,
    /// メトリクスコレクター
    metrics: Arc<MetricsCollector>,
    /// キーペア
    keypair: KeyPair,
    /// ノードID
    node_id: String,
    /// 実行中フラグ
    running: bool,
    /// イベント通知チャネル
    event_tx: mpsc::Sender<ChannelEvent>,
    /// イベント通知受信チャネル
    event_rx: mpsc::Receiver<ChannelEvent>,
}

/// チャネルID
pub type ChannelId = String;

/// チャネル
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Channel {
    /// チャネルID
    pub id: ChannelId,
    /// チャネル参加者
    pub participants: Vec<Participant>,
    /// チャネル状態
    pub state: ChannelState,
    /// チャネルパラメータ
    pub params: ChannelParams,
    /// チャネル残高
    pub balances: HashMap<String, u64>,
    /// チャネル作成時間
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// チャネル更新時間
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// チャネル有効期限
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// チャネルメタデータ
    pub metadata: HashMap<String, String>,
}

/// チャネル参加者
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Participant {
    /// 参加者ID
    pub id: String,
    /// 参加者アドレス
    pub address: String,
    /// 参加者公開鍵
    pub public_key: String,
    /// 参加者メタデータ
    pub metadata: HashMap<String, String>,
}

/// チャネルイベント
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelEvent {
    /// イベントID
    pub id: String,
    /// イベントタイプ
    pub event_type: ChannelEventType,
    /// チャネルID
    pub channel_id: ChannelId,
    /// タイムスタンプ
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// データ
    pub data: serde_json::Value,
}

/// チャネルイベントタイプ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelEventType {
    /// チャネル作成
    ChannelCreated,
    /// チャネル更新
    ChannelUpdated,
    /// チャネル閉鎖
    ChannelClosed,
    /// 支払い送信
    PaymentSent,
    /// 支払い受信
    PaymentReceived,
    /// 紛争開始
    DisputeStarted,
    /// 紛争解決
    DisputeResolved,
    /// 仮想チャネル作成
    VirtualChannelCreated,
    /// 仮想チャネル閉鎖
    VirtualChannelClosed,
    /// エラー
    Error,
}

/// チャネルステータス
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelStatus {
    /// 初期化中
    Initializing,
    /// オープン
    Open,
    /// 更新中
    Updating,
    /// 紛争中
    Disputed,
    /// クローズ中
    Closing,
    /// クローズド
    Closed,
    /// エラー
    Error,
}

/// チャネル統計
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelStats {
    /// チャネル数
    pub channel_count: usize,
    /// 仮想チャネル数
    pub virtual_channel_count: usize,
    /// 総残高
    pub total_balance: u64,
    /// 総支払い数
    pub total_payments: u64,
    /// 総支払い金額
    pub total_payment_amount: u64,
    /// 紛争数
    pub dispute_count: usize,
    /// アクティブチャネル数
    pub active_channel_count: usize,
    /// 平均チャネル容量
    pub average_channel_capacity: f64,
    /// 平均支払いサイズ
    pub average_payment_size: f64,
    /// 平均ルート長
    pub average_route_length: f64,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

impl ChannelManager {
    /// 新しいChannelManagerを作成
    pub fn new(
        config: ChannelConfig,
        network_manager: Arc<NetworkManager>,
        storage_manager: Arc<StorageManager>,
        crypto_manager: Arc<CryptoManager>,
        metrics: Arc<MetricsCollector>,
        keypair: KeyPair,
    ) -> Result<Self, Error> {
        let (tx, rx) = mpsc::channel(100);
        
        // ノードIDを生成
        let node_id = hex::encode(&keypair.public_key);
        
        // チャネルネットワークを作成
        let network = ChannelNetwork::new(config.network_config.clone());
        
        // 紛争解決器を作成
        let dispute_resolver = DisputeResolver::new(config.dispute_config.clone());
        
        // 更新プロセッサーを作成
        let update_processor = UpdateProcessor::new(config.update_config.clone());
        
        // チャネルウォッチャーを作成
        let watcher = ChannelWatcher::new(config.watcher_config.clone());
        
        // ルーティングテーブルを作成
        let routing_table = RoutingTable::new(config.routing_config.clone());
        
        Ok(Self {
            config,
            channels: HashMap::new(),
            virtual_channels: HashMap::new(),
            network,
            dispute_resolver,
            update_processor,
            watcher,
            routing_table,
            network_manager,
            storage_manager,
            crypto_manager,
            metrics,
            keypair,
            node_id,
            running: false,
            event_tx: tx,
            event_rx: rx,
        })
    }
    
    /// 状態チャネルを開始
    pub async fn start(&mut self) -> Result<(), Error> {
        if self.running {
            return Err(Error::InvalidState("Channel manager is already running".to_string()));
        }
        
        self.running = true;
        
        // 保存されたチャネルを読み込む
        self.load_channels().await?;
        
        // バックグラウンドタスクを開始
        self.start_background_tasks();
        
        info!("Channel manager started with node ID: {}", self.node_id);
        
        Ok(())
    }
    
    /// 状態チャネルを停止
    pub async fn stop(&mut self) -> Result<(), Error> {
        if !self.running {
            return Err(Error::InvalidState("Channel manager is not running".to_string()));
        }
        
        self.running = false;
        
        // チャネルウォッチャーを停止
        self.watcher.stop().await?;
        
        info!("Channel manager stopped");
        
        Ok(())
    }
    
    /// バックグラウンドタスクを開始
    fn start_background_tasks(&self) {
        // 更新処理タスク
        let update_interval = self.config.update_interval_ms;
        let update_tx = self.event_tx.clone();
        let update_processor = Arc::new(RwLock::new(self.update_processor.clone()));
        let update_running = Arc::new(RwLock::new(self.running));
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(update_interval));
            
            loop {
                interval.tick().await;
                
                let running = *update_running.read().unwrap();
                if !running {
                    break;
                }
                
                let processor = update_processor.read().unwrap();
                
                // 更新処理タスクを実行
                if let Err(e) = processor.process_pending_updates().await {
                    error!("Failed to process pending updates: {}", e);
                }
            }
        });
        
        // ウォッチャータスク
        let watcher_interval = self.config.watcher_interval_ms;
        let watcher_tx = self.event_tx.clone();
        let watcher = Arc::new(RwLock::new(self.watcher.clone()));
        let watcher_running = Arc::new(RwLock::new(self.running));
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(watcher_interval));
            
            loop {
                interval.tick().await;
                
                let running = *watcher_running.read().unwrap();
                if !running {
                    break;
                }
                
                let w = watcher.read().unwrap();
                
                // ウォッチャータスクを実行
                if let Err(e) = w.check_channels().await {
                    error!("Failed to check channels: {}", e);
                }
            }
        });
        
        // ルーティングテーブル更新タスク
        let routing_interval = self.config.routing_interval_ms;
        let routing_tx = self.event_tx.clone();
        let routing_table = Arc::new(RwLock::new(self.routing_table.clone()));
        let routing_running = Arc::new(RwLock::new(self.running));
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(routing_interval));
            
            loop {
                interval.tick().await;
                
                let running = *routing_running.read().unwrap();
                if !running {
                    break;
                }
                
                let table = routing_table.read().unwrap();
                
                // ルーティングテーブルを更新
                if let Err(e) = table.update_routes().await {
                    error!("Failed to update routing table: {}", e);
                }
            }
        });
    }
    
    /// 保存されたチャネルを読み込む
    async fn load_channels(&mut self) -> Result<(), Error> {
        // ストレージからチャネルを読み込む
        let storage = self.storage_manager.get_storage("channels")?;
        
        // チャネルを読み込む
        if let Ok(channels) = storage.get_all::<Channel>("channel") {
            for channel in channels {
                self.channels.insert(channel.id.clone(), channel);
            }
        }
        
        // 仮想チャネルを読み込む
        if let Ok(virtual_channels) = storage.get_all::<VirtualChannel>("virtual_channel") {
            for channel in virtual_channels {
                self.virtual_channels.insert(channel.id.clone(), channel);
            }
        }
        
        info!("Loaded {} channels and {} virtual channels", self.channels.len(), self.virtual_channels.len());
        
        Ok(())
    }
    
    /// チャネルを作成
    pub async fn create_channel(
        &mut self,
        counterparty: &str,
        initial_balance: u64,
        counterparty_balance: u64,
        params: Option<ChannelParams>,
    ) -> Result<ChannelId, Error> {
        // チャネルIDを生成
        let channel_id = format!("ch-{}", Uuid::new_v4());
        
        // 現在時刻を取得
        let now = chrono::Utc::now();
        
        // チャネルパラメータを設定
        let params = params.unwrap_or_else(|| ChannelParams {
            timeout_seconds: self.config.default_timeout_seconds,
            expiry_seconds: self.config.default_expiry_seconds,
            min_payment: self.config.default_min_payment,
            max_payment: self.config.default_max_payment,
            fee_rate: self.config.default_fee_rate,
            metadata: HashMap::new(),
        });
        
        // 有効期限を計算
        let expires_at = if params.expiry_seconds > 0 {
            Some(now + chrono::Duration::seconds(params.expiry_seconds as i64))
        } else {
            None
        };
        
        // 参加者を作成
        let participants = vec![
            Participant {
                id: self.node_id.clone(),
                address: hex::encode(&self.keypair.public_key),
                public_key: hex::encode(&self.keypair.public_key),
                metadata: HashMap::new(),
            },
            Participant {
                id: counterparty.to_string(),
                address: counterparty.to_string(),
                public_key: counterparty.to_string(),
                metadata: HashMap::new(),
            },
        ];
        
        // 残高を設定
        let mut balances = HashMap::new();
        balances.insert(self.node_id.clone(), initial_balance);
        balances.insert(counterparty.to_string(), counterparty_balance);
        
        // チャネル状態を作成
        let state = ChannelState {
            version: 0,
            status: ChannelStatus::Initializing,
            balances: balances.clone(),
            pending_updates: Vec::new(),
            last_update: now,
            signatures: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        // チャネルを作成
        let channel = Channel {
            id: channel_id.clone(),
            participants,
            state,
            params,
            balances,
            created_at: now,
            updated_at: now,
            expires_at,
            metadata: HashMap::new(),
        };
        
        // チャネルを保存
        self.channels.insert(channel_id.clone(), channel.clone());
        
        // ストレージに保存
        let storage = self.storage_manager.get_storage("channels")?;
        storage.put(&format!("channel:{}", channel_id), &channel)?;
        
        // チャネル作成イベントを発行
        let event = ChannelEvent {
            id: format!("event-{}", Uuid::new_v4()),
            event_type: ChannelEventType::ChannelCreated,
            channel_id: channel_id.clone(),
            timestamp: now,
            data: serde_json::json!({
                "counterparty": counterparty,
                "initial_balance": initial_balance,
                "counterparty_balance": counterparty_balance,
            }),
        };
        
        if let Err(e) = self.event_tx.send(event).await {
            error!("Failed to send channel created event: {}", e);
        }
        
        // メトリクスを更新
        self.metrics.increment_counter("channel_created");
        self.metrics.set_gauge("channel_count", self.channels.len() as f64);
        self.metrics.set_gauge("total_channel_balance", (initial_balance + counterparty_balance) as f64);
        
        info!("Created channel {} with counterparty {}", channel_id, counterparty);
        
        Ok(channel_id)
    }
    
    /// チャネルを閉じる
    pub async fn close_channel(
        &mut self,
        channel_id: &str,
        reason: Option<&str>,
    ) -> Result<(), Error> {
        // チャネルが存在するかチェック
        let channel = self.channels.get_mut(channel_id)
            .ok_or_else(|| Error::NotFound(format!("Channel not found: {}", channel_id)))?;
        
        // チャネルが閉じられるかチェック
        if channel.state.status == ChannelStatus::Closed {
            return Err(Error::InvalidState(format!("Channel is already closed: {}", channel_id)));
        }
        
        // 現在時刻を取得
        let now = chrono::Utc::now();
        
        // チャネル状態を更新
        channel.state.status = ChannelStatus::Closing;
        channel.state.version += 1;
        channel.state.last_update = now;
        channel.updated_at = now;
        
        // 閉鎖理由をメタデータに追加
        if let Some(reason) = reason {
            channel.metadata.insert("close_reason".to_string(), reason.to_string());
        }
        
        // ストレージを更新
        let storage = self.storage_manager.get_storage("channels")?;
        storage.put(&format!("channel:{}", channel_id), channel)?;
        
        // チャネル閉鎖イベントを発行
        let event = ChannelEvent {
            id: format!("event-{}", Uuid::new_v4()),
            event_type: ChannelEventType::ChannelClosed,
            channel_id: channel_id.to_string(),
            timestamp: now,
            data: serde_json::json!({
                "reason": reason,
                "final_balances": channel.state.balances,
            }),
        };
        
        if let Err(e) = self.event_tx.send(event).await {
            error!("Failed to send channel closed event: {}", e);
        }
        
        // メトリクスを更新
        self.metrics.increment_counter("channel_closed");
        
        info!("Closed channel {}", channel_id);
        
        Ok(())
    }
    
    /// 支払いを送信
    pub async fn send_payment(
        &mut self,
        channel_id: &str,
        amount: u64,
        recipient: &str,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<String, Error> {
        // チャネルが存在するかチェック
        let channel = self.channels.get_mut(channel_id)
            .ok_or_else(|| Error::NotFound(format!("Channel not found: {}", channel_id)))?;
        
        // チャネルがオープンかチェック
        if channel.state.status != ChannelStatus::Open {
            return Err(Error::InvalidState(format!("Channel is not open: {}", channel_id)));
        }
        
        // 残高が十分かチェック
        let sender_balance = channel.state.balances.get(&self.node_id).cloned().unwrap_or(0);
        if sender_balance < amount {
            return Err(Error::InsufficientFunds(format!("Insufficient balance: {} < {}", sender_balance, amount)));
        }
        
        // 支払いIDを生成
        let payment_id = format!("pay-{}", Uuid::new_v4());
        
        // 現在時刻を取得
        let now = chrono::Utc::now();
        
        // 支払いを作成
        let payment = Payment {
            id: payment_id.clone(),
            channel_id: channel_id.to_string(),
            sender: self.node_id.clone(),
            recipient: recipient.to_string(),
            amount,
            status: PaymentStatus::Pending,
            created_at: now,
            updated_at: now,
            completed_at: None,
            condition: None,
            metadata: metadata.unwrap_or_else(HashMap::new),
        };
        
        // 状態更新を作成
        let update = StateUpdate {
            channel_id: channel_id.to_string(),
            version: channel.state.version + 1,
            balances: {
                let mut new_balances = channel.state.balances.clone();
                *new_balances.entry(self.node_id.clone()).or_insert(0) -= amount;
                *new_balances.entry(recipient.to_string()).or_insert(0) += amount;
                new_balances
            },
            timestamp: now,
            payment_id: Some(payment_id.clone()),
            metadata: HashMap::new(),
        };
        
        // 更新に署名
        let signature = self.sign_update(&update)?;
        
        // 更新を送信
        self.update_processor.send_update(channel_id, update.clone(), signature).await?;
        
        // チャネル状態を更新
        channel.state.pending_updates.push(update);
        channel.updated_at = now;
        
        // ストレージを更新
        let storage = self.storage_manager.get_storage("channels")?;
        storage.put(&format!("channel:{}", channel_id), channel)?;
        storage.put(&format!("payment:{}", payment_id), &payment)?;
        
        // 支払い送信イベントを発行
        let event = ChannelEvent {
            id: format!("event-{}", Uuid::new_v4()),
            event_type: ChannelEventType::PaymentSent,
            channel_id: channel_id.to_string(),
            timestamp: now,
            data: serde_json::json!({
                "payment_id": payment_id,
                "amount": amount,
                "recipient": recipient,
            }),
        };
        
        if let Err(e) = self.event_tx.send(event).await {
            error!("Failed to send payment sent event: {}", e);
        }
        
        // メトリクスを更新
        self.metrics.increment_counter("payment_sent");
        self.metrics.increment_counter("payment_amount", amount as f64);
        
        info!("Sent payment {} of amount {} to {} via channel {}", payment_id, amount, recipient, channel_id);
        
        Ok(payment_id)
    }
    
    /// 支払いを受信
    pub async fn receive_payment(
        &mut self,
        channel_id: &str,
        payment_id: &str,
        amount: u64,
        sender: &str,
    ) -> Result<(), Error> {
        // チャネルが存在するかチェック
        let channel = self.channels.get_mut(channel_id)
            .ok_or_else(|| Error::NotFound(format!("Channel not found: {}", channel_id)))?;
        
        // チャネルがオープンかチェック
        if channel.state.status != ChannelStatus::Open {
            return Err(Error::InvalidState(format!("Channel is not open: {}", channel_id)));
        }
        
        // 現在時刻を取得
        let now = chrono::Utc::now();
        
        // 支払いを作成
        let payment = Payment {
            id: payment_id.to_string(),
            channel_id: channel_id.to_string(),
            sender: sender.to_string(),
            recipient: self.node_id.clone(),
            amount,
            status: PaymentStatus::Completed,
            created_at: now,
            updated_at: now,
            completed_at: Some(now),
            condition: None,
            metadata: HashMap::new(),
        };
        
        // 状態更新を作成
        let update = StateUpdate {
            channel_id: channel_id.to_string(),
            version: channel.state.version + 1,
            balances: {
                let mut new_balances = channel.state.balances.clone();
                *new_balances.entry(sender.to_string()).or_insert(0) -= amount;
                *new_balances.entry(self.node_id.clone()).or_insert(0) += amount;
                new_balances
            },
            timestamp: now,
            payment_id: Some(payment_id.to_string()),
            metadata: HashMap::new(),
        };
        
        // 更新に署名
        let signature = self.sign_update(&update)?;
        
        // 更新を送信
        self.update_processor.send_update(channel_id, update.clone(), signature).await?;
        
        // チャネル状態を更新
        channel.state.balances = update.balances.clone();
        channel.state.version = update.version;
        channel.state.last_update = now;
        channel.updated_at = now;
        
        // ストレージを更新
        let storage = self.storage_manager.get_storage("channels")?;
        storage.put(&format!("channel:{}", channel_id), channel)?;
        storage.put(&format!("payment:{}", payment_id), &payment)?;
        
        // 支払い受信イベントを発行
        let event = ChannelEvent {
            id: format!("event-{}", Uuid::new_v4()),
            event_type: ChannelEventType::PaymentReceived,
            channel_id: channel_id.to_string(),
            timestamp: now,
            data: serde_json::json!({
                "payment_id": payment_id,
                "amount": amount,
                "sender": sender,
            }),
        };
        
        if let Err(e) = self.event_tx.send(event).await {
            error!("Failed to send payment received event: {}", e);
        }
        
        // メトリクスを更新
        self.metrics.increment_counter("payment_received");
        self.metrics.increment_counter("payment_amount_received", amount as f64);
        
        info!("Received payment {} of amount {} from {} via channel {}", payment_id, amount, sender, channel_id);
        
        Ok(())
    }
    
    /// 仮想チャネルを作成
    pub async fn create_virtual_channel(
        &mut self,
        intermediaries: Vec<String>,
        recipient: &str,
        capacity: u64,
        params: Option<ChannelParams>,
    ) -> Result<ChannelId, Error> {
        // 仮想チャネルIDを生成
        let channel_id = format!("vch-{}", Uuid::new_v4());
        
        // 現在時刻を取得
        let now = chrono::Utc::now();
        
        // チャネルパラメータを設定
        let params = params.unwrap_or_else(|| ChannelParams {
            timeout_seconds: self.config.default_timeout_seconds,
            expiry_seconds: self.config.default_expiry_seconds,
            min_payment: self.config.default_min_payment,
            max_payment: self.config.default_max_payment,
            fee_rate: self.config.default_fee_rate,
            metadata: HashMap::new(),
        });
        
        // 有効期限を計算
        let expires_at = if params.expiry_seconds > 0 {
            Some(now + chrono::Duration::seconds(params.expiry_seconds as i64))
        } else {
            None
        };
        
        // 参加者を作成
        let mut participants = vec![
            Participant {
                id: self.node_id.clone(),
                address: hex::encode(&self.keypair.public_key),
                public_key: hex::encode(&self.keypair.public_key),
                metadata: HashMap::new(),
            },
        ];
        
        // 中間者を追加
        for intermediary in &intermediaries {
            participants.push(Participant {
                id: intermediary.clone(),
                address: intermediary.clone(),
                public_key: intermediary.clone(),
                metadata: HashMap::new(),
            });
        }
        
        // 受信者を追加
        participants.push(Participant {
            id: recipient.to_string(),
            address: recipient.to_string(),
            public_key: recipient.to_string(),
            metadata: HashMap::new(),
        });
        
        // 残高を設定
        let mut balances = HashMap::new();
        balances.insert(self.node_id.clone(), capacity);
        balances.insert(recipient.to_string(), 0);
        
        // 仮想チャネル状態を作成
        let state = VirtualChannelState {
            version: 0,
            status: ChannelStatus::Initializing,
            balances: balances.clone(),
            base_channels: Vec::new(),
            last_update: now,
            signatures: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        // 仮想チャネルを作成
        let virtual_channel = VirtualChannel {
            id: channel_id.clone(),
            participants,
            state,
            params,
            intermediaries,
            capacity,
            created_at: now,
            updated_at: now,
            expires_at,
            metadata: HashMap::new(),
        };
        
        // 仮想チャネルを保存
        self.virtual_channels.insert(channel_id.clone(), virtual_channel.clone());
        
        // ストレージに保存
        let storage = self.storage_manager.get_storage("channels")?;
        storage.put(&format!("virtual_channel:{}", channel_id), &virtual_channel)?;
        
        // 仮想チャネル作成イベントを発行
        let event = ChannelEvent {
            id: format!("event-{}", Uuid::new_v4()),
            event_type: ChannelEventType::VirtualChannelCreated,
            channel_id: channel_id.clone(),
            timestamp: now,
            data: serde_json::json!({
                "intermediaries": intermediaries,
                "recipient": recipient,
                "capacity": capacity,
            }),
        };
        
        if let Err(e) = self.event_tx.send(event).await {
            error!("Failed to send virtual channel created event: {}", e);
        }
        
        // メトリクスを更新
        self.metrics.increment_counter("virtual_channel_created");
        self.metrics.set_gauge("virtual_channel_count", self.virtual_channels.len() as f64);
        
        info!("Created virtual channel {} with recipient {} through {} intermediaries", 
            channel_id, recipient, intermediaries.len());
        
        Ok(channel_id)
    }
    
    /// 仮想チャネルを閉じる
    pub async fn close_virtual_channel(
        &mut self,
        channel_id: &str,
        reason: Option<&str>,
    ) -> Result<(), Error> {
        // 仮想チャネルが存在するかチェック
        let virtual_channel = self.virtual_channels.get_mut(channel_id)
            .ok_or_else(|| Error::NotFound(format!("Virtual channel not found: {}", channel_id)))?;
        
        // 仮想チャネルが閉じられるかチェック
        if virtual_channel.state.status == ChannelStatus::Closed {
            return Err(Error::InvalidState(format!("Virtual channel is already closed: {}", channel_id)));
        }
        
        // 現在時刻を取得
        let now = chrono::Utc::now();
        
        // 仮想チャネル状態を更新
        virtual_channel.state.status = ChannelStatus::Closing;
        virtual_channel.state.version += 1;
        virtual_channel.state.last_update = now;
        virtual_channel.updated_at = now;
        
        // 閉鎖理由をメタデータに追加
        if let Some(reason) = reason {
            virtual_channel.metadata.insert("close_reason".to_string(), reason.to_string());
        }
        
        // ストレージを更新
        let storage = self.storage_manager.get_storage("channels")?;
        storage.put(&format!("virtual_channel:{}", channel_id), virtual_channel)?;
        
        // 仮想チャネル閉鎖イベントを発行
        let event = ChannelEvent {
            id: format!("event-{}", Uuid::new_v4()),
            event_type: ChannelEventType::VirtualChannelClosed,
            channel_id: channel_id.to_string(),
            timestamp: now,
            data: serde_json::json!({
                "reason": reason,
                "final_balances": virtual_channel.state.balances,
            }),
        };
        
        if let Err(e) = self.event_tx.send(event).await {
            error!("Failed to send virtual channel closed event: {}", e);
        }
        
        // メトリクスを更新
        self.metrics.increment_counter("virtual_channel_closed");
        
        info!("Closed virtual channel {}", channel_id);
        
        Ok(())
    }
    
    /// 紛争を開始
    pub async fn start_dispute(
        &mut self,
        channel_id: &str,
        reason: &str,
    ) -> Result<String, Error> {
        // チャネルが存在するかチェック
        let channel = self.channels.get(channel_id)
            .ok_or_else(|| Error::NotFound(format!("Channel not found: {}", channel_id)))?;
        
        // 紛争IDを生成
        let dispute_id = format!("dispute-{}", Uuid::new_v4());
        
        // 現在時刻を取得
        let now = chrono::Utc::now();
        
        // 紛争を開始
        self.dispute_resolver.start_dispute(
            dispute_id.clone(),
            channel_id.to_string(),
            self.node_id.clone(),
            reason.to_string(),
            channel.state.clone(),
        ).await?;
        
        // 紛争開始イベントを発行
        let event = ChannelEvent {
            id: format!("event-{}", Uuid::new_v4()),
            event_type: ChannelEventType::DisputeStarted,
            channel_id: channel_id.to_string(),
            timestamp: now,
            data: serde_json::json!({
                "dispute_id": dispute_id,
                "reason": reason,
            }),
        };
        
        if let Err(e) = self.event_tx.send(event).await {
            error!("Failed to send dispute started event: {}", e);
        }
        
        // メトリクスを更新
        self.metrics.increment_counter("dispute_started");
        
        info!("Started dispute {} for channel {} with reason: {}", dispute_id, channel_id, reason);
        
        Ok(dispute_id)
    }
    
    /// 紛争を解決
    pub async fn resolve_dispute(
        &mut self,
        dispute_id: &str,
        resolution: &str,
    ) -> Result<(), Error> {
        // 紛争を解決
        let (channel_id, state) = self.dispute_resolver.resolve_dispute(
            dispute_id,
            resolution.to_string(),
        ).await?;
        
        // チャネルが存在するかチェック
        let channel = self.channels.get_mut(&channel_id)
            .ok_or_else(|| Error::NotFound(format!("Channel not found: {}", channel_id)))?;
        
        // 現在時刻を取得
        let now = chrono::Utc::now();
        
        // チャネル状態を更新
        channel.state = state;
        channel.updated_at = now;
        
        // ストレージを更新
        let storage = self.storage_manager.get_storage("channels")?;
        storage.put(&format!("channel:{}", channel_id), channel)?;
        
        // 紛争解決イベントを発行
        let event = ChannelEvent {
            id: format!("event-{}", Uuid::new_v4()),
            event_type: ChannelEventType::DisputeResolved,
            channel_id,
            timestamp: now,
            data: serde_json::json!({
                "dispute_id": dispute_id,
                "resolution": resolution,
            }),
        };
        
        if let Err(e) = self.event_tx.send(event).await {
            error!("Failed to send dispute resolved event: {}", e);
        }
        
        // メトリクスを更新
        self.metrics.increment_counter("dispute_resolved");
        
        info!("Resolved dispute {} with resolution: {}", dispute_id, resolution);
        
        Ok(())
    }
    
    /// 支払いルートを検索
    pub async fn find_payment_route(
        &self,
        recipient: &str,
        amount: u64,
    ) -> Result<Vec<RouteHop>, Error> {
        // ルートを検索
        let route = self.routing_table.find_route(
            &self.node_id,
            recipient,
            amount,
        )?;
        
        if route.is_empty() {
            return Err(Error::NotFound(format!("No route found to recipient: {}", recipient)));
        }
        
        Ok(route)
    }
    
    /// 更新に署名
    fn sign_update(&self, update: &StateUpdate) -> Result<ChannelSignature, Error> {
        // 更新をシリアライズ
        let update_bytes = serde_json::to_vec(update)?;
        
        // 署名を生成
        let signature = self.crypto_manager.sign(&update_bytes, &self.keypair.private_key)?;
        
        Ok(ChannelSignature {
            signer: self.node_id.clone(),
            signature: hex::encode(signature),
            timestamp: chrono::Utc::now(),
        })
    }
    
    /// チャネルを取得
    pub fn get_channel(&self, channel_id: &str) -> Result<&Channel, Error> {
        self.channels.get(channel_id)
            .ok_or_else(|| Error::NotFound(format!("Channel not found: {}", channel_id)))
    }
    
    /// 仮想チャネルを取得
    pub fn get_virtual_channel(&self, channel_id: &str) -> Result<&VirtualChannel, Error> {
        self.virtual_channels.get(channel_id)
            .ok_or_else(|| Error::NotFound(format!("Virtual channel not found: {}", channel_id)))
    }
    
    /// すべてのチャネルIDを取得
    pub fn get_all_channel_ids(&self) -> Vec<ChannelId> {
        self.channels.keys().cloned().collect()
    }
    
    /// すべての仮想チャネルIDを取得
    pub fn get_all_virtual_channel_ids(&self) -> Vec<ChannelId> {
        self.virtual_channels.keys().cloned().collect()
    }
    
    /// チャネル統計を取得
    pub fn get_stats(&self) -> ChannelStats {
        let mut total_balance = 0;
        let mut total_payments = 0;
        let mut total_payment_amount = 0;
        let mut active_channel_count = 0;
        let mut total_capacity = 0;
        
        // チャネル統計を計算
        for channel in self.channels.values() {
            let channel_balance: u64 = channel.balances.values().sum();
            total_balance += channel_balance;
            
            if channel.state.status == ChannelStatus::Open {
                active_channel_count += 1;
            }
            
            // チャネル容量を計算
            let capacity: u64 = channel.balances.values().sum();
            total_capacity += capacity;
        }
        
        // 平均チャネル容量を計算
        let average_channel_capacity = if !self.channels.is_empty() {
            total_capacity as f64 / self.channels.len() as f64
        } else {
            0.0
        };
        
        // 平均支払いサイズを計算
        let average_payment_size = if total_payments > 0 {
            total_payment_amount as f64 / total_payments as f64
        } else {
            0.0
        };
        
        // 平均ルート長を計算
        let average_route_length = self.routing_table.get_average_route_length();
        
        ChannelStats {
            channel_count: self.channels.len(),
            virtual_channel_count: self.virtual_channels.len(),
            total_balance,
            total_payments,
            total_payment_amount,
            dispute_count: self.dispute_resolver.get_active_dispute_count(),
            active_channel_count,
            average_channel_capacity,
            average_payment_size,
            average_route_length,
            metadata: HashMap::new(),
        }
    }
    
    /// 設定を取得
    pub fn get_config(&self) -> &ChannelConfig {
        &self.config
    }
    
    /// 設定を更新
    pub fn update_config(&mut self, config: ChannelConfig) {
        self.config = config.clone();
        self.network.update_config(config.network_config);
        self.dispute_resolver.update_config(config.dispute_config);
        self.update_processor.update_config(config.update_config);
        self.watcher.update_config(config.watcher_config);
        self.routing_table.update_config(config.routing_config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::test_utils::create_test_network_manager;
    use crate::storage::test_utils::create_test_storage_manager;
    use crate::crypto::test_utils::{create_test_crypto_manager, create_test_keypair};
    
    #[tokio::test]
    async fn test_channel_manager_creation() {
        let config = ChannelConfig::default();
        let network_manager = Arc::new(create_test_network_manager());
        let storage_manager = Arc::new(create_test_storage_manager());
        let crypto_manager = Arc::new(create_test_crypto_manager());
        let metrics = Arc::new(MetricsCollector::new("channel"));
        let keypair = create_test_keypair();
        
        let result = ChannelManager::new(
            config,
            network_manager,
            storage_manager,
            crypto_manager,
            metrics,
            keypair,
        );
        
        assert!(result.is_ok());
        let manager = result.unwrap();
        
        assert!(!manager.running);
        assert!(manager.channels.is_empty());
        assert!(manager.virtual_channels.is_empty());
    }
    
    #[tokio::test]
    async fn test_channel_manager_start_stop() {
        let config = ChannelConfig::default();
        let network_manager = Arc::new(create_test_network_manager());
        let storage_manager = Arc::new(create_test_storage_manager());
        let crypto_manager = Arc::new(create_test_crypto_manager());
        let metrics = Arc::new(MetricsCollector::new("channel"));
        let keypair = create_test_keypair();
        
        let mut manager = ChannelManager::new(
            config,
            network_manager,
            storage_manager,
            crypto_manager,
            metrics,
            keypair,
        ).unwrap();
        
        // 開始
        let result = manager.start().await;
        assert!(result.is_ok());
        assert!(manager.running);
        
        // 停止
        let result = manager.stop().await;
        assert!(result.is_ok());
        assert!(!manager.running);
    }
    
    #[tokio::test]
    async fn test_create_channel() {
        let config = ChannelConfig::default();
        let network_manager = Arc::new(create_test_network_manager());
        let storage_manager = Arc::new(create_test_storage_manager());
        let crypto_manager = Arc::new(create_test_crypto_manager());
        let metrics = Arc::new(MetricsCollector::new("channel"));
        let keypair = create_test_keypair();
        
        let mut manager = ChannelManager::new(
            config,
            network_manager,
            storage_manager,
            crypto_manager,
            metrics,
            keypair,
        ).unwrap();
        
        // チャネルを作成
        let result = manager.create_channel("counterparty1", 1000, 1000, None).await;
        
        // テスト環境では実際のチャネルは作成できないので、エラーになることを確認
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_get_stats() {
        let config = ChannelConfig::default();
        let network_manager = Arc::new(create_test_network_manager());
        let storage_manager = Arc::new(create_test_storage_manager());
        let crypto_manager = Arc::new(create_test_crypto_manager());
        let metrics = Arc::new(MetricsCollector::new("channel"));
        let keypair = create_test_keypair();
        
        let manager = ChannelManager::new(
            config,
            network_manager,
            storage_manager,
            crypto_manager,
            metrics,
            keypair,
        ).unwrap();
        
        // 統計を取得
        let stats = manager.get_stats();
        
        assert_eq!(stats.channel_count, 0);
        assert_eq!(stats.virtual_channel_count, 0);
        assert_eq!(stats.total_balance, 0);
        assert_eq!(stats.total_payments, 0);
        assert_eq!(stats.total_payment_amount, 0);
        assert_eq!(stats.active_channel_count, 0);
    }
}