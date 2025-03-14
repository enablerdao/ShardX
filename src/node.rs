use crate::ai::AIPriorityManager;
use crate::consensus::{ProofOfFlow, SimpleValidator, Validator};
use crate::sharding::{CrossShardManager, ShardingManager};
use crate::transaction::{DAG, Transaction};
use log::{error, info};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use uuid::Uuid;

/// ノードの設定
pub struct NodeConfig {
    /// ノードID
    pub node_id: String,
    /// APIポート
    pub port: u16,
    /// データディレクトリ
    pub data_dir: String,
    /// 初期シャード数
    pub shard_count: u32,
    /// 負荷閾値
    pub load_threshold: u32,
    /// バリデータの数
    pub validator_count: usize,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            node_id: Uuid::new_v4().to_string(),
            port: 54868,
            data_dir: "./data".to_string(),
            shard_count: 256,
            load_threshold: 10000,
            validator_count: 4,
        }
    }
}

/// ノードの状態
#[derive(Debug, Clone, Copy)]
pub enum NodeStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
}

/// ShardXノード
pub struct Node {
    /// ノードID
    pub id: String,
    /// ノードの状態
    pub status: NodeStatus,
    /// DAG
    pub dag: Arc<DAG>,
    /// コンセンサスエンジン
    pub consensus: Arc<ProofOfFlow>,
    /// シャーディングマネージャー
    pub sharding_manager: Arc<ShardingManager>,
    /// クロスシャードマネージャー
    pub cross_shard_manager: Arc<CrossShardManager>,
    /// AI優先度マネージャー
    pub priority_manager: Arc<AIPriorityManager>,
    /// トランザクション処理チャネル
    tx_sender: mpsc::Sender<Transaction>,
    tx_receiver: Option<mpsc::Receiver<Transaction>>,
}

impl Node {
    /// 新しいノードを作成
    pub fn new(config: NodeConfig) -> Self {
        let dag = Arc::new(DAG::new());
        
        // バリデータを作成
        let mut validators: Vec<Arc<dyn Validator>> = Vec::new();
        for i in 0..config.validator_count {
            validators.push(Arc::new(SimpleValidator {
                id: format!("validator-{}", i),
            }));
        }
        
        // コンセンサスエンジンを作成
        let consensus = Arc::new(ProofOfFlow::new(Arc::clone(&dag), validators));
        
        // シャーディングマネージャーを作成
        let sharding_manager = Arc::new(ShardingManager::new(
            config.shard_count,
            config.load_threshold,
        ));
        
        // クロスシャードマネージャーを作成
        let cross_shard_manager = Arc::new(CrossShardManager::new(Arc::clone(&sharding_manager)));
        
        // AI優先度マネージャーを作成
        let priority_manager = Arc::new(AIPriorityManager::new());
        
        // トランザクション処理チャネルを作成
        let (tx_sender, tx_receiver) = mpsc::channel(1000);
        
        Self {
            id: config.node_id,
            status: NodeStatus::Stopped,
            dag,
            consensus,
            sharding_manager,
            cross_shard_manager,
            priority_manager,
            tx_sender,
            tx_receiver: Some(tx_receiver),
        }
    }
    
    /// ノードを起動
    pub async fn start(&mut self) {
        info!("Starting node {}", self.id);
        self.status = NodeStatus::Starting;
        
        // トランザクション処理ループを開始
        let consensus = Arc::clone(&self.consensus);
        let cross_shard_manager = Arc::clone(&self.cross_shard_manager);
        let priority_manager = Arc::clone(&self.priority_manager);
        let mut rx = self.tx_receiver.take().unwrap();
        
        tokio::spawn(async move {
            while let Some(tx) = rx.recv().await {
                // AIによる優先度付け
                priority_manager.enqueue(tx.clone());
                
                // 優先度の高いトランザクションを処理
                if let Some(prioritized_tx) = priority_manager.dequeue() {
                    // シャーディングによるルーティング
                    match cross_shard_manager.route_transaction(prioritized_tx.clone()).await {
                        Ok(Some(target_shard)) => {
                            info!("Transaction {} routed to shard {}", prioritized_tx.id, target_shard);
                        }
                        Ok(None) => {
                            // このシャードで処理
                            match consensus.process_transaction(prioritized_tx.clone()).await {
                                Ok(_) => {
                                    info!("Transaction {} processed successfully", prioritized_tx.id);
                                }
                                Err(e) => {
                                    error!("Failed to process transaction {}: {}", prioritized_tx.id, e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to route transaction {}: {}", prioritized_tx.id, e);
                        }
                    }
                }
            }
        });
        
        // 負荷監視ループを開始
        let sharding_manager = Arc::clone(&self.sharding_manager);
        let priority_manager = Arc::clone(&self.priority_manager);
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                // キューサイズを負荷として使用
                let current_load = priority_manager.queue_size() as u32;
                
                // シャード数を調整
                sharding_manager.adjust_shards(current_load);
            }
        });
        
        self.status = NodeStatus::Running;
        info!("Node {} started", self.id);
    }
    
    /// トランザクションを送信
    pub async fn submit_transaction(&self, tx: Transaction) -> Result<(), String> {
        if let Err(e) = self.tx_sender.send(tx.clone()).await {
            error!("Failed to submit transaction {}: {}", tx.id, e);
            Err(format!("Failed to submit transaction: {}", e))
        } else {
            Ok(())
        }
    }
    
    /// ノードを停止
    pub async fn stop(&mut self) {
        info!("Stopping node {}", self.id);
        self.status = NodeStatus::Stopping;
        
        // 実際の実装では、ここでクリーンアップ処理を行う
        
        self.status = NodeStatus::Stopped;
        info!("Node {} stopped", self.id);
    }
    
    /// ノードの状態を取得
    pub fn get_status(&self) -> NodeStatus {
        self.status
    }
    
    /// 現在のTPS（1秒あたりのトランザクション数）を計算
    pub fn get_tps(&self) -> f64 {
        self.consensus.calculate_tps(10)
    }
    
    /// 現在のシャード数を取得
    pub fn get_shard_count(&self) -> u32 {
        self.sharding_manager.get_shard_count()
    }
}