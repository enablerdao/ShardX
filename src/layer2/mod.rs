// レイヤー2モジュール
//
// このモジュールは、ShardXのレイヤー2ソリューションを提供します。
// レイヤー2ソリューションは、メインチェーン（レイヤー1）の上に構築された
// スケーラビリティソリューションで、トランザクションスループットを向上させ、
// 手数料を削減し、メインチェーンの負荷を軽減します。
//
// 主な機能:
// - ロールアップ（Optimistic/ZK）
// - サイドチェーン
// - プラズマチェーン
// - ステートチャネル
// - バリデータネットワーク

mod config;
// mod rollup; // TODO: このモジュールが見つかりません
// mod sidechain; // TODO: このモジュールが見つかりません
// mod plasma; // TODO: このモジュールが見つかりません
// mod validator; // TODO: このモジュールが見つかりません
// mod bridge; // TODO: このモジュールが見つかりません
// mod sync; // TODO: このモジュールが見つかりません
// mod proof; // TODO: このモジュールが見つかりません
// mod challenge; // TODO: このモジュールが見つかりません
// mod batch; // TODO: このモジュールが見つかりません

pub use self::config::{Layer2Config, RollupConfig, SidechainConfig, PlasmaConfig, ValidatorConfig};
pub use self::rollup::{Rollup, RollupType, RollupState, OptimisticRollup, ZKRollup};
pub use self::sidechain::{Sidechain, SidechainState, SidechainBlock, SidechainTransaction};
pub use self::plasma::{PlasmaChain, PlasmaBlock, PlasmaTransaction, PlasmaExit};
pub use self::validator::{ValidatorNetwork, ValidatorNode, ValidatorConsensus};
pub use self::bridge::{Bridge, BridgeTransaction, BridgeState, TokenBridge};
pub use self::sync::{SyncManager, SyncState, SyncMode};
pub use self::proof::{Proof, ProofVerifier, ProofGenerator, ZKProof, FraudProof};
pub use self::challenge::{Challenge, ChallengeManager, ChallengeState, ChallengeVerifier};
pub use self::batch::{Batch, BatchProcessor, BatchSubmitter, BatchVerifier};

use crate::error::Error;
use crate::metrics::MetricsCollector;
use crate::network::NetworkManager;
use crate::storage::StorageManager;
use crate::crypto::CryptoManager;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use log::{debug, error, info, warn};
use serde::{Serialize, Deserialize};

/// レイヤー2マネージャー
pub struct Layer2Manager {
    /// 設定
    config: Layer2Config,
    /// ロールアップ
    rollups: HashMap<String, Box<dyn Rollup>>,
    /// サイドチェーン
    sidechains: HashMap<String, Sidechain>,
    /// プラズマチェーン
    plasma_chains: HashMap<String, PlasmaChain>,
    /// バリデータネットワーク
    validator_networks: HashMap<String, ValidatorNetwork>,
    /// ブリッジ
    bridges: HashMap<String, Bridge>,
    /// 同期マネージャー
    sync_manager: SyncManager,
    /// チャレンジマネージャー
    challenge_manager: ChallengeManager,
    /// バッチプロセッサー
    batch_processor: BatchProcessor,
    /// ネットワークマネージャー
    network_manager: Arc<NetworkManager>,
    /// ストレージマネージャー
    storage_manager: Arc<StorageManager>,
    /// 暗号マネージャー
    crypto_manager: Arc<CryptoManager>,
    /// メトリクスコレクター
    metrics: Arc<MetricsCollector>,
    /// 実行中フラグ
    running: bool,
    /// イベント通知チャネル
    event_tx: mpsc::Sender<Layer2Event>,
    /// イベント通知受信チャネル
    event_rx: mpsc::Receiver<Layer2Event>,
}

/// レイヤー2イベント
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layer2Event {
    /// イベントID
    pub id: String,
    /// イベントタイプ
    pub event_type: Layer2EventType,
    /// レイヤー2 ID
    pub layer2_id: String,
    /// レイヤー2タイプ
    pub layer2_type: Layer2Type,
    /// タイムスタンプ
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// データ
    pub data: serde_json::Value,
}

/// レイヤー2イベントタイプ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Layer2EventType {
    /// ブロック作成
    BlockCreated,
    /// バッチ提出
    BatchSubmitted,
    /// 状態更新
    StateUpdated,
    /// チャレンジ開始
    ChallengeStarted,
    /// チャレンジ解決
    ChallengeResolved,
    /// 証明生成
    ProofGenerated,
    /// 証明検証
    ProofVerified,
    /// 出金開始
    WithdrawalStarted,
    /// 出金完了
    WithdrawalCompleted,
    /// デポジット受信
    DepositReceived,
    /// エラー
    Error,
}

/// レイヤー2タイプ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Layer2Type {
    /// Optimisticロールアップ
    OptimisticRollup,
    /// ZKロールアップ
    ZKRollup,
    /// サイドチェーン
    Sidechain,
    /// プラズマチェーン
    PlasmaChain,
    /// バリデータネットワーク
    ValidatorNetwork,
}

/// レイヤー2ステータス
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Layer2Status {
    /// 初期化中
    Initializing,
    /// 同期中
    Syncing,
    /// アクティブ
    Active,
    /// 一時停止
    Paused,
    /// エラー
    Error,
    /// シャットダウン
    Shutdown,
}

/// レイヤー2統計
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layer2Stats {
    /// レイヤー2 ID
    pub layer2_id: String,
    /// レイヤー2タイプ
    pub layer2_type: Layer2Type,
    /// ステータス
    pub status: Layer2Status,
    /// トランザクション数
    pub transaction_count: u64,
    /// ブロック数
    pub block_count: u64,
    /// バッチ数
    pub batch_count: u64,
    /// TPS（1秒あたりのトランザクション数）
    pub tps: f64,
    /// 最終ブロック時間
    pub last_block_time: Option<chrono::DateTime<chrono::Utc>>,
    /// 最終バッチ時間
    pub last_batch_time: Option<chrono::DateTime<chrono::Utc>>,
    /// 最終同期時間
    pub last_sync_time: Option<chrono::DateTime<chrono::Utc>>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

impl Layer2Manager {
    /// 新しいLayer2Managerを作成
    pub fn new(
        config: Layer2Config,
        network_manager: Arc<NetworkManager>,
        storage_manager: Arc<StorageManager>,
        crypto_manager: Arc<CryptoManager>,
        metrics: Arc<MetricsCollector>,
    ) -> Self {
        let (tx, rx) = mpsc::channel(100);
        
        let sync_manager = SyncManager::new(config.sync_config.clone());
        let challenge_manager = ChallengeManager::new(config.challenge_config.clone());
        let batch_processor = BatchProcessor::new(config.batch_config.clone());
        
        Self {
            config,
            rollups: HashMap::new(),
            sidechains: HashMap::new(),
            plasma_chains: HashMap::new(),
            validator_networks: HashMap::new(),
            bridges: HashMap::new(),
            sync_manager,
            challenge_manager,
            batch_processor,
            network_manager,
            storage_manager,
            crypto_manager,
            metrics,
            running: false,
            event_tx: tx,
            event_rx: rx,
        }
    }
    
    /// レイヤー2を開始
    pub async fn start(&mut self) -> Result<(), Error> {
        if self.running {
            return Err(Error::InvalidState("Layer2 manager is already running".to_string()));
        }
        
        self.running = true;
        
        // 保存されたレイヤー2設定を読み込む
        self.load_layer2_configs().await?;
        
        // バックグラウンドタスクを開始
        self.start_background_tasks();
        
        info!("Layer2 manager started");
        
        Ok(())
    }
    
    /// レイヤー2を停止
    pub async fn stop(&mut self) -> Result<(), Error> {
        if !self.running {
            return Err(Error::InvalidState("Layer2 manager is not running".to_string()));
        }
        
        self.running = false;
        
        // すべてのレイヤー2を停止
        for (id, rollup) in &mut self.rollups {
            if let Err(e) = rollup.stop().await {
                error!("Failed to stop rollup {}: {}", id, e);
            }
        }
        
        for (id, sidechain) in &mut self.sidechains {
            if let Err(e) = sidechain.stop().await {
                error!("Failed to stop sidechain {}: {}", id, e);
            }
        }
        
        for (id, plasma) in &mut self.plasma_chains {
            if let Err(e) = plasma.stop().await {
                error!("Failed to stop plasma chain {}: {}", id, e);
            }
        }
        
        for (id, validator) in &mut self.validator_networks {
            if let Err(e) = validator.stop().await {
                error!("Failed to stop validator network {}: {}", id, e);
            }
        }
        
        info!("Layer2 manager stopped");
        
        Ok(())
    }
    
    /// バックグラウンドタスクを開始
    fn start_background_tasks(&self) {
        // 同期タスク
        let sync_interval = self.config.sync_interval_ms;
        let sync_tx = self.event_tx.clone();
        let sync_manager = Arc::new(RwLock::new(self.sync_manager.clone()));
        let sync_running = Arc::new(RwLock::new(self.running));
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(sync_interval));
            
            loop {
                interval.tick().await;
                
                let running = *sync_running.read().unwrap();
                if !running {
                    break;
                }
                
                let manager = sync_manager.read().unwrap();
                
                // 同期タスクを実行
                if let Err(e) = manager.run_sync_tasks().await {
                    error!("Failed to run sync tasks: {}", e);
                }
            }
        });
        
        // バッチ処理タスク
        let batch_interval = self.config.batch_interval_ms;
        let batch_tx = self.event_tx.clone();
        let batch_processor = Arc::new(RwLock::new(self.batch_processor.clone()));
        let batch_running = Arc::new(RwLock::new(self.running));
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(batch_interval));
            
            loop {
                interval.tick().await;
                
                let running = *batch_running.read().unwrap();
                if !running {
                    break;
                }
                
                let processor = batch_processor.read().unwrap();
                
                // バッチ処理タスクを実行
                if let Err(e) = processor.process_pending_batches().await {
                    error!("Failed to process pending batches: {}", e);
                }
            }
        });
        
        // チャレンジ監視タスク
        let challenge_interval = self.config.challenge_interval_ms;
        let challenge_tx = self.event_tx.clone();
        let challenge_manager = Arc::new(RwLock::new(self.challenge_manager.clone()));
        let challenge_running = Arc::new(RwLock::new(self.running));
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(challenge_interval));
            
            loop {
                interval.tick().await;
                
                let running = *challenge_running.read().unwrap();
                if !running {
                    break;
                }
                
                let manager = challenge_manager.read().unwrap();
                
                // チャレンジを監視
                if let Err(e) = manager.monitor_challenges().await {
                    error!("Failed to monitor challenges: {}", e);
                }
            }
        });
    }
    
    /// 保存されたレイヤー2設定を読み込む
    async fn load_layer2_configs(&mut self) -> Result<(), Error> {
        // ストレージからレイヤー2設定を読み込む
        let storage = self.storage_manager.get_storage("layer2_configs")?;
        
        // Optimisticロールアップ設定を読み込む
        if let Ok(configs) = storage.get_all::<RollupConfig>("optimistic_rollup") {
            for config in configs {
                if let Err(e) = self.create_optimistic_rollup(&config.id, config).await {
                    error!("Failed to create optimistic rollup: {}", e);
                }
            }
        }
        
        // ZKロールアップ設定を読み込む
        if let Ok(configs) = storage.get_all::<RollupConfig>("zk_rollup") {
            for config in configs {
                if let Err(e) = self.create_zk_rollup(&config.id, config).await {
                    error!("Failed to create ZK rollup: {}", e);
                }
            }
        }
        
        // サイドチェーン設定を読み込む
        if let Ok(configs) = storage.get_all::<SidechainConfig>("sidechain") {
            for config in configs {
                if let Err(e) = self.create_sidechain(&config.id, config).await {
                    error!("Failed to create sidechain: {}", e);
                }
            }
        }
        
        // プラズマチェーン設定を読み込む
        if let Ok(configs) = storage.get_all::<PlasmaConfig>("plasma") {
            for config in configs {
                if let Err(e) = self.create_plasma_chain(&config.id, config).await {
                    error!("Failed to create plasma chain: {}", e);
                }
            }
        }
        
        // バリデータネットワーク設定を読み込む
        if let Ok(configs) = storage.get_all::<ValidatorConfig>("validator") {
            for config in configs {
                if let Err(e) = self.create_validator_network(&config.id, config).await {
                    error!("Failed to create validator network: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Optimisticロールアップを作成
    pub async fn create_optimistic_rollup(
        &mut self,
        id: &str,
        config: RollupConfig,
    ) -> Result<(), Error> {
        // IDが既に存在するかチェック
        if self.rollups.contains_key(id) {
            return Err(Error::AlreadyExists(format!("Rollup with ID {} already exists", id)));
        }
        
        // Optimisticロールアップを作成
        let rollup = OptimisticRollup::new(
            id.to_string(),
            config.clone(),
            self.network_manager.clone(),
            self.storage_manager.clone(),
            self.crypto_manager.clone(),
            self.event_tx.clone(),
        )?;
        
        // ロールアップを開始
        rollup.start().await?;
        
        // ロールアップを保存
        self.rollups.insert(id.to_string(), Box::new(rollup));
        
        // 設定を保存
        let storage = self.storage_manager.get_storage("layer2_configs")?;
        storage.put(&format!("optimistic_rollup:{}", id), &config)?;
        
        // メトリクスを更新
        self.metrics.increment_counter("layer2_optimistic_rollups_created");
        
        info!("Created optimistic rollup: {}", id);
        
        Ok(())
    }
    
    /// ZKロールアップを作成
    pub async fn create_zk_rollup(
        &mut self,
        id: &str,
        config: RollupConfig,
    ) -> Result<(), Error> {
        // IDが既に存在するかチェック
        if self.rollups.contains_key(id) {
            return Err(Error::AlreadyExists(format!("Rollup with ID {} already exists", id)));
        }
        
        // ZKロールアップを作成
        let rollup = ZKRollup::new(
            id.to_string(),
            config.clone(),
            self.network_manager.clone(),
            self.storage_manager.clone(),
            self.crypto_manager.clone(),
            self.event_tx.clone(),
        )?;
        
        // ロールアップを開始
        rollup.start().await?;
        
        // ロールアップを保存
        self.rollups.insert(id.to_string(), Box::new(rollup));
        
        // 設定を保存
        let storage = self.storage_manager.get_storage("layer2_configs")?;
        storage.put(&format!("zk_rollup:{}", id), &config)?;
        
        // メトリクスを更新
        self.metrics.increment_counter("layer2_zk_rollups_created");
        
        info!("Created ZK rollup: {}", id);
        
        Ok(())
    }
    
    /// サイドチェーンを作成
    pub async fn create_sidechain(
        &mut self,
        id: &str,
        config: SidechainConfig,
    ) -> Result<(), Error> {
        // IDが既に存在するかチェック
        if self.sidechains.contains_key(id) {
            return Err(Error::AlreadyExists(format!("Sidechain with ID {} already exists", id)));
        }
        
        // サイドチェーンを作成
        let sidechain = Sidechain::new(
            id.to_string(),
            config.clone(),
            self.network_manager.clone(),
            self.storage_manager.clone(),
            self.crypto_manager.clone(),
            self.event_tx.clone(),
        )?;
        
        // サイドチェーンを開始
        sidechain.start().await?;
        
        // サイドチェーンを保存
        self.sidechains.insert(id.to_string(), sidechain);
        
        // 設定を保存
        let storage = self.storage_manager.get_storage("layer2_configs")?;
        storage.put(&format!("sidechain:{}", id), &config)?;
        
        // メトリクスを更新
        self.metrics.increment_counter("layer2_sidechains_created");
        
        info!("Created sidechain: {}", id);
        
        Ok(())
    }
    
    /// プラズマチェーンを作成
    pub async fn create_plasma_chain(
        &mut self,
        id: &str,
        config: PlasmaConfig,
    ) -> Result<(), Error> {
        // IDが既に存在するかチェック
        if self.plasma_chains.contains_key(id) {
            return Err(Error::AlreadyExists(format!("Plasma chain with ID {} already exists", id)));
        }
        
        // プラズマチェーンを作成
        let plasma = PlasmaChain::new(
            id.to_string(),
            config.clone(),
            self.network_manager.clone(),
            self.storage_manager.clone(),
            self.crypto_manager.clone(),
            self.event_tx.clone(),
        )?;
        
        // プラズマチェーンを開始
        plasma.start().await?;
        
        // プラズマチェーンを保存
        self.plasma_chains.insert(id.to_string(), plasma);
        
        // 設定を保存
        let storage = self.storage_manager.get_storage("layer2_configs")?;
        storage.put(&format!("plasma:{}", id), &config)?;
        
        // メトリクスを更新
        self.metrics.increment_counter("layer2_plasma_chains_created");
        
        info!("Created plasma chain: {}", id);
        
        Ok(())
    }
    
    /// バリデータネットワークを作成
    pub async fn create_validator_network(
        &mut self,
        id: &str,
        config: ValidatorConfig,
    ) -> Result<(), Error> {
        // IDが既に存在するかチェック
        if self.validator_networks.contains_key(id) {
            return Err(Error::AlreadyExists(format!("Validator network with ID {} already exists", id)));
        }
        
        // バリデータネットワークを作成
        let validator = ValidatorNetwork::new(
            id.to_string(),
            config.clone(),
            self.network_manager.clone(),
            self.storage_manager.clone(),
            self.crypto_manager.clone(),
            self.event_tx.clone(),
        )?;
        
        // バリデータネットワークを開始
        validator.start().await?;
        
        // バリデータネットワークを保存
        self.validator_networks.insert(id.to_string(), validator);
        
        // 設定を保存
        let storage = self.storage_manager.get_storage("layer2_configs")?;
        storage.put(&format!("validator:{}", id), &config)?;
        
        // メトリクスを更新
        self.metrics.increment_counter("layer2_validator_networks_created");
        
        info!("Created validator network: {}", id);
        
        Ok(())
    }
    
    /// ブリッジを作成
    pub async fn create_bridge(
        &mut self,
        id: &str,
        source_layer2_id: &str,
        target_layer2_id: &str,
    ) -> Result<(), Error> {
        // IDが既に存在するかチェック
        if self.bridges.contains_key(id) {
            return Err(Error::AlreadyExists(format!("Bridge with ID {} already exists", id)));
        }
        
        // ソースレイヤー2が存在するかチェック
        let source_type = self.get_layer2_type(source_layer2_id)?;
        
        // ターゲットレイヤー2が存在するかチェック
        let target_type = self.get_layer2_type(target_layer2_id)?;
        
        // ブリッジを作成
        let bridge = Bridge::new(
            id.to_string(),
            source_layer2_id.to_string(),
            target_layer2_id.to_string(),
            source_type,
            target_type,
            self.network_manager.clone(),
            self.storage_manager.clone(),
            self.crypto_manager.clone(),
            self.event_tx.clone(),
        )?;
        
        // ブリッジを開始
        bridge.start().await?;
        
        // ブリッジを保存
        self.bridges.insert(id.to_string(), bridge);
        
        // メトリクスを更新
        self.metrics.increment_counter("layer2_bridges_created");
        
        info!("Created bridge: {} (from {} to {})", id, source_layer2_id, target_layer2_id);
        
        Ok(())
    }
    
    /// レイヤー2タイプを取得
    fn get_layer2_type(&self, layer2_id: &str) -> Result<Layer2Type, Error> {
        if self.rollups.contains_key(layer2_id) {
            let rollup = self.rollups.get(layer2_id).unwrap();
            Ok(match rollup.get_rollup_type() {
                RollupType::Optimistic => Layer2Type::OptimisticRollup,
                RollupType::ZK => Layer2Type::ZKRollup,
            })
        } else if self.sidechains.contains_key(layer2_id) {
            Ok(Layer2Type::Sidechain)
        } else if self.plasma_chains.contains_key(layer2_id) {
            Ok(Layer2Type::PlasmaChain)
        } else if self.validator_networks.contains_key(layer2_id) {
            Ok(Layer2Type::ValidatorNetwork)
        } else {
            Err(Error::NotFound(format!("Layer2 with ID {} not found", layer2_id)))
        }
    }
    
    /// トランザクションを送信
    pub async fn send_transaction(
        &self,
        layer2_id: &str,
        transaction: &[u8],
    ) -> Result<String, Error> {
        // レイヤー2が存在するかチェック
        let layer2_type = self.get_layer2_type(layer2_id)?;
        
        // レイヤー2タイプに基づいてトランザクションを送信
        match layer2_type {
            Layer2Type::OptimisticRollup | Layer2Type::ZKRollup => {
                let rollup = self.rollups.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Rollup with ID {} not found", layer2_id)))?;
                
                rollup.send_transaction(transaction).await
            },
            Layer2Type::Sidechain => {
                let sidechain = self.sidechains.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Sidechain with ID {} not found", layer2_id)))?;
                
                sidechain.send_transaction(transaction).await
            },
            Layer2Type::PlasmaChain => {
                let plasma = self.plasma_chains.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Plasma chain with ID {} not found", layer2_id)))?;
                
                plasma.send_transaction(transaction).await
            },
            Layer2Type::ValidatorNetwork => {
                let validator = self.validator_networks.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Validator network with ID {} not found", layer2_id)))?;
                
                validator.send_transaction(transaction).await
            },
        }
    }
    
    /// トランザクションステータスを取得
    pub async fn get_transaction_status(
        &self,
        layer2_id: &str,
        transaction_id: &str,
    ) -> Result<TransactionStatus, Error> {
        // レイヤー2が存在するかチェック
        let layer2_type = self.get_layer2_type(layer2_id)?;
        
        // レイヤー2タイプに基づいてトランザクションステータスを取得
        match layer2_type {
            Layer2Type::OptimisticRollup | Layer2Type::ZKRollup => {
                let rollup = self.rollups.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Rollup with ID {} not found", layer2_id)))?;
                
                rollup.get_transaction_status(transaction_id).await
            },
            Layer2Type::Sidechain => {
                let sidechain = self.sidechains.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Sidechain with ID {} not found", layer2_id)))?;
                
                sidechain.get_transaction_status(transaction_id).await
            },
            Layer2Type::PlasmaChain => {
                let plasma = self.plasma_chains.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Plasma chain with ID {} not found", layer2_id)))?;
                
                plasma.get_transaction_status(transaction_id).await
            },
            Layer2Type::ValidatorNetwork => {
                let validator = self.validator_networks.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Validator network with ID {} not found", layer2_id)))?;
                
                validator.get_transaction_status(transaction_id).await
            },
        }
    }
    
    /// デポジットを実行
    pub async fn deposit(
        &self,
        layer2_id: &str,
        from: &str,
        to: &str,
        token: &str,
        amount: u64,
    ) -> Result<String, Error> {
        // レイヤー2が存在するかチェック
        let layer2_type = self.get_layer2_type(layer2_id)?;
        
        // レイヤー2タイプに基づいてデポジットを実行
        match layer2_type {
            Layer2Type::OptimisticRollup | Layer2Type::ZKRollup => {
                let rollup = self.rollups.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Rollup with ID {} not found", layer2_id)))?;
                
                rollup.deposit(from, to, token, amount).await
            },
            Layer2Type::Sidechain => {
                let sidechain = self.sidechains.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Sidechain with ID {} not found", layer2_id)))?;
                
                sidechain.deposit(from, to, token, amount).await
            },
            Layer2Type::PlasmaChain => {
                let plasma = self.plasma_chains.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Plasma chain with ID {} not found", layer2_id)))?;
                
                plasma.deposit(from, to, token, amount).await
            },
            Layer2Type::ValidatorNetwork => {
                let validator = self.validator_networks.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Validator network with ID {} not found", layer2_id)))?;
                
                validator.deposit(from, to, token, amount).await
            },
        }
    }
    
    /// 出金を開始
    pub async fn withdraw(
        &self,
        layer2_id: &str,
        from: &str,
        to: &str,
        token: &str,
        amount: u64,
    ) -> Result<String, Error> {
        // レイヤー2が存在するかチェック
        let layer2_type = self.get_layer2_type(layer2_id)?;
        
        // レイヤー2タイプに基づいて出金を開始
        match layer2_type {
            Layer2Type::OptimisticRollup | Layer2Type::ZKRollup => {
                let rollup = self.rollups.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Rollup with ID {} not found", layer2_id)))?;
                
                rollup.withdraw(from, to, token, amount).await
            },
            Layer2Type::Sidechain => {
                let sidechain = self.sidechains.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Sidechain with ID {} not found", layer2_id)))?;
                
                sidechain.withdraw(from, to, token, amount).await
            },
            Layer2Type::PlasmaChain => {
                let plasma = self.plasma_chains.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Plasma chain with ID {} not found", layer2_id)))?;
                
                plasma.withdraw(from, to, token, amount).await
            },
            Layer2Type::ValidatorNetwork => {
                let validator = self.validator_networks.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Validator network with ID {} not found", layer2_id)))?;
                
                validator.withdraw(from, to, token, amount).await
            },
        }
    }
    
    /// 出金ステータスを取得
    pub async fn get_withdrawal_status(
        &self,
        layer2_id: &str,
        withdrawal_id: &str,
    ) -> Result<WithdrawalStatus, Error> {
        // レイヤー2が存在するかチェック
        let layer2_type = self.get_layer2_type(layer2_id)?;
        
        // レイヤー2タイプに基づいて出金ステータスを取得
        match layer2_type {
            Layer2Type::OptimisticRollup | Layer2Type::ZKRollup => {
                let rollup = self.rollups.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Rollup with ID {} not found", layer2_id)))?;
                
                rollup.get_withdrawal_status(withdrawal_id).await
            },
            Layer2Type::Sidechain => {
                let sidechain = self.sidechains.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Sidechain with ID {} not found", layer2_id)))?;
                
                sidechain.get_withdrawal_status(withdrawal_id).await
            },
            Layer2Type::PlasmaChain => {
                let plasma = self.plasma_chains.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Plasma chain with ID {} not found", layer2_id)))?;
                
                plasma.get_withdrawal_status(withdrawal_id).await
            },
            Layer2Type::ValidatorNetwork => {
                let validator = self.validator_networks.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Validator network with ID {} not found", layer2_id)))?;
                
                validator.get_withdrawal_status(withdrawal_id).await
            },
        }
    }
    
    /// レイヤー2統計を取得
    pub async fn get_layer2_stats(&self, layer2_id: &str) -> Result<Layer2Stats, Error> {
        // レイヤー2が存在するかチェック
        let layer2_type = self.get_layer2_type(layer2_id)?;
        
        // レイヤー2タイプに基づいて統計を取得
        match layer2_type {
            Layer2Type::OptimisticRollup | Layer2Type::ZKRollup => {
                let rollup = self.rollups.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Rollup with ID {} not found", layer2_id)))?;
                
                let stats = rollup.get_stats().await?;
                let status = rollup.get_status().await?;
                
                Ok(Layer2Stats {
                    layer2_id: layer2_id.to_string(),
                    layer2_type,
                    status,
                    transaction_count: stats.transaction_count,
                    block_count: stats.block_count,
                    batch_count: stats.batch_count,
                    tps: stats.tps,
                    last_block_time: stats.last_block_time,
                    last_batch_time: stats.last_batch_time,
                    last_sync_time: stats.last_sync_time,
                    metadata: stats.metadata,
                })
            },
            Layer2Type::Sidechain => {
                let sidechain = self.sidechains.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Sidechain with ID {} not found", layer2_id)))?;
                
                let stats = sidechain.get_stats().await?;
                let status = sidechain.get_status().await?;
                
                Ok(Layer2Stats {
                    layer2_id: layer2_id.to_string(),
                    layer2_type,
                    status,
                    transaction_count: stats.transaction_count,
                    block_count: stats.block_count,
                    batch_count: stats.batch_count,
                    tps: stats.tps,
                    last_block_time: stats.last_block_time,
                    last_batch_time: stats.last_batch_time,
                    last_sync_time: stats.last_sync_time,
                    metadata: stats.metadata,
                })
            },
            Layer2Type::PlasmaChain => {
                let plasma = self.plasma_chains.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Plasma chain with ID {} not found", layer2_id)))?;
                
                let stats = plasma.get_stats().await?;
                let status = plasma.get_status().await?;
                
                Ok(Layer2Stats {
                    layer2_id: layer2_id.to_string(),
                    layer2_type,
                    status,
                    transaction_count: stats.transaction_count,
                    block_count: stats.block_count,
                    batch_count: stats.batch_count,
                    tps: stats.tps,
                    last_block_time: stats.last_block_time,
                    last_batch_time: stats.last_batch_time,
                    last_sync_time: stats.last_sync_time,
                    metadata: stats.metadata,
                })
            },
            Layer2Type::ValidatorNetwork => {
                let validator = self.validator_networks.get(layer2_id)
                    .ok_or_else(|| Error::NotFound(format!("Validator network with ID {} not found", layer2_id)))?;
                
                let stats = validator.get_stats().await?;
                let status = validator.get_status().await?;
                
                Ok(Layer2Stats {
                    layer2_id: layer2_id.to_string(),
                    layer2_type,
                    status,
                    transaction_count: stats.transaction_count,
                    block_count: stats.block_count,
                    batch_count: stats.batch_count,
                    tps: stats.tps,
                    last_block_time: stats.last_block_time,
                    last_batch_time: stats.last_batch_time,
                    last_sync_time: stats.last_sync_time,
                    metadata: stats.metadata,
                })
            },
        }
    }
    
    /// すべてのレイヤー2 IDを取得
    pub fn get_all_layer2_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        
        // ロールアップIDを追加
        for id in self.rollups.keys() {
            ids.push(id.clone());
        }
        
        // サイドチェーンIDを追加
        for id in self.sidechains.keys() {
            ids.push(id.clone());
        }
        
        // プラズマチェーンIDを追加
        for id in self.plasma_chains.keys() {
            ids.push(id.clone());
        }
        
        // バリデータネットワークIDを追加
        for id in self.validator_networks.keys() {
            ids.push(id.clone());
        }
        
        ids
    }
    
    /// すべてのブリッジIDを取得
    pub fn get_all_bridge_ids(&self) -> Vec<String> {
        self.bridges.keys().cloned().collect()
    }
    
    /// 設定を取得
    pub fn get_config(&self) -> &Layer2Config {
        &self.config
    }
    
    /// 設定を更新
    pub fn update_config(&mut self, config: Layer2Config) {
        self.config = config.clone();
        self.sync_manager.update_config(config.sync_config);
        self.challenge_manager.update_config(config.challenge_config);
        self.batch_processor.update_config(config.batch_config);
    }
}

/// トランザクションステータス
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// 保留中
    Pending,
    /// 処理中
    Processing,
    /// 確認済み
    Confirmed,
    /// 失敗
    Failed,
    /// 不明
    Unknown,
}

/// 出金ステータス
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WithdrawalStatus {
    /// 開始
    Initiated,
    /// 処理中
    Processing,
    /// 証明待ち
    AwaitingProof,
    /// チャレンジ期間
    ChallengePeriod,
    /// 完了
    Completed,
    /// 失敗
    Failed,
    /// チャレンジされた
    Challenged,
    /// 不明
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::test_utils::create_test_network_manager;
    use crate::storage::test_utils::create_test_storage_manager;
    use crate::crypto::test_utils::create_test_crypto_manager;
    
    #[tokio::test]
    async fn test_layer2_manager_creation() {
        let config = Layer2Config::default();
        let network_manager = Arc::new(create_test_network_manager());
        let storage_manager = Arc::new(create_test_storage_manager());
        let crypto_manager = Arc::new(create_test_crypto_manager());
        let metrics = Arc::new(MetricsCollector::new("layer2"));
        
        let manager = Layer2Manager::new(
            config,
            network_manager,
            storage_manager,
            crypto_manager,
            metrics,
        );
        
        assert!(!manager.running);
        assert!(manager.rollups.is_empty());
        assert!(manager.sidechains.is_empty());
        assert!(manager.plasma_chains.is_empty());
        assert!(manager.validator_networks.is_empty());
        assert!(manager.bridges.is_empty());
    }
    
    #[tokio::test]
    async fn test_layer2_manager_start_stop() {
        let config = Layer2Config::default();
        let network_manager = Arc::new(create_test_network_manager());
        let storage_manager = Arc::new(create_test_storage_manager());
        let crypto_manager = Arc::new(create_test_crypto_manager());
        let metrics = Arc::new(MetricsCollector::new("layer2"));
        
        let mut manager = Layer2Manager::new(
            config,
            network_manager,
            storage_manager,
            crypto_manager,
            metrics,
        );
        
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
    async fn test_create_optimistic_rollup() {
        let config = Layer2Config::default();
        let network_manager = Arc::new(create_test_network_manager());
        let storage_manager = Arc::new(create_test_storage_manager());
        let crypto_manager = Arc::new(create_test_crypto_manager());
        let metrics = Arc::new(MetricsCollector::new("layer2"));
        
        let mut manager = Layer2Manager::new(
            config,
            network_manager,
            storage_manager,
            crypto_manager,
            metrics,
        );
        
        // Optimisticロールアップを作成
        let rollup_config = RollupConfig::default();
        let result = manager.create_optimistic_rollup("test_rollup", rollup_config).await;
        
        // テスト環境では実際のロールアップは作成できないので、エラーになることを確認
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_get_layer2_type() {
        let config = Layer2Config::default();
        let network_manager = Arc::new(create_test_network_manager());
        let storage_manager = Arc::new(create_test_storage_manager());
        let crypto_manager = Arc::new(create_test_crypto_manager());
        let metrics = Arc::new(MetricsCollector::new("layer2"));
        
        let manager = Layer2Manager::new(
            config,
            network_manager,
            storage_manager,
            crypto_manager,
            metrics,
        );
        
        // 存在しないレイヤー2のタイプを取得
        let result = manager.get_layer2_type("non_existent");
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_get_all_layer2_ids() {
        let config = Layer2Config::default();
        let network_manager = Arc::new(create_test_network_manager());
        let storage_manager = Arc::new(create_test_storage_manager());
        let crypto_manager = Arc::new(create_test_crypto_manager());
        let metrics = Arc::new(MetricsCollector::new("layer2"));
        
        let manager = Layer2Manager::new(
            config,
            network_manager,
            storage_manager,
            crypto_manager,
            metrics,
        );
        
        // すべてのレイヤー2 IDを取得
        let ids = manager.get_all_layer2_ids();
        assert!(ids.is_empty());
    }
}