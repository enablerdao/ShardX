use crate::transaction::Transaction;
use log::{debug, info};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// シャーディングマネージャー
pub struct ShardingManager {
    /// 現在のシャード数
    shard_count: AtomicU32,
    /// 負荷閾値（この値を超えるとシャード数を増やす）
    load_threshold: u32,
}

impl ShardingManager {
    /// 新しいシャーディングマネージャーを作成
    pub fn new(initial_shards: u32, load_threshold: u32) -> Self {
        Self {
            shard_count: AtomicU32::new(initial_shards),
            load_threshold,
        }
    }
    
    /// トランザクションが属するシャードを決定
    pub fn assign_shard(&self, tx: &Transaction) -> u32 {
        let mut hasher = Sha256::new();
        hasher.update(&tx.id);
        let hash = hasher.finalize();
        
        // ハッシュの最初の4バイトを取り出してu32に変換
        let hash_value = u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]);
        
        // 現在のシャード数で割った余りがシャードID
        hash_value % self.shard_count.load(Ordering::Relaxed)
    }
    
    /// 現在のシャード数を取得
    pub fn get_shard_count(&self) -> u32 {
        self.shard_count.load(Ordering::Relaxed)
    }
    
    /// 負荷に応じてシャード数を調整
    pub fn adjust_shards(&self, current_load: u32) {
        let current_shards = self.shard_count.load(Ordering::Relaxed);
        
        if current_load > self.load_threshold && current_shards < 512 {
            // 負荷が高い場合、シャード数を2倍に増やす（最大512）
            let new_shards = (current_shards * 2).min(512);
            self.shard_count.store(new_shards, Ordering::Relaxed);
            info!("Increased shard count from {} to {}", current_shards, new_shards);
        } else if current_load < self.load_threshold / 2 && current_shards > 256 {
            // 負荷が低い場合、シャード数を半分に減らす（最小256）
            let new_shards = (current_shards / 2).max(256);
            self.shard_count.store(new_shards, Ordering::Relaxed);
            info!("Decreased shard count from {} to {}", current_shards, new_shards);
        }
    }
}

/// クロスシャード通信マネージャー
pub struct CrossShardManager {
    /// シャーディングマネージャーの参照
    sharding_manager: Arc<ShardingManager>,
}

impl CrossShardManager {
    /// 新しいクロスシャード通信マネージャーを作成
    pub fn new(sharding_manager: Arc<ShardingManager>) -> Self {
        Self { sharding_manager }
    }
    
    /// トランザクションを別のシャードに送信
    pub async fn send_cross_shard(&self, tx: Transaction, target_shard: u32) -> Result<(), String> {
        // 実際の実装では、Redisやメッセージキューを使用してトランザクションを送信
        // 簡略化のため、ログ出力のみ
        debug!(
            "Sending transaction {} to shard {}",
            tx.id, target_shard
        );
        
        // 成功を返す
        Ok(())
    }
    
    /// トランザクションが属するシャードを確認し、必要に応じて転送
    pub async fn route_transaction(&self, tx: Transaction) -> Result<Option<u32>, String> {
        let target_shard = self.sharding_manager.assign_shard(&tx);
        let current_shard = 0; // 現在のノードのシャードID（実際の実装では設定から取得）
        
        if target_shard != current_shard {
            // 別のシャードに転送
            self.send_cross_shard(tx, target_shard).await?;
            Ok(Some(target_shard))
        } else {
            // このシャードで処理
            Ok(None)
        }
    }
}