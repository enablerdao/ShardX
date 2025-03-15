use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

use crate::error::Error;

/// シャードID
pub type ShardId = String;

/// シャード情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardInfo {
    /// シャードID
    pub id: ShardId,
    /// シャード名
    pub name: String,
    /// バリデータ数
    pub validators: usize,
    /// 現在のブロック高
    pub height: u64,
    /// 1秒あたりのトランザクション数
    pub tps: f64,
    /// シャードの状態
    pub status: ShardStatus,
}

/// シャードの状態
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShardStatus {
    /// アクティブ
    Active,
    /// 非アクティブ
    Inactive,
    /// 同期中
    Syncing,
}

/// シャードマネージャー
pub struct ShardManager {
    /// シャード情報
    shards: RwLock<HashMap<ShardId, ShardInfo>>,
    /// シャード間の接続情報
    connections: Mutex<HashMap<(ShardId, ShardId), ConnectionInfo>>,
}

/// シャード間の接続情報
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// 接続元シャードID
    pub from: ShardId,
    /// 接続先シャードID
    pub to: ShardId,
    /// レイテンシ（ミリ秒）
    pub latency: u64,
    /// 帯域幅（バイト/秒）
    pub bandwidth: u64,
    /// 最後に更新された時刻
    pub last_updated: u64,
}

impl ShardManager {
    /// 新しいシャードマネージャーを作成
    pub fn new() -> Self {
        Self {
            shards: RwLock::new(HashMap::new()),
            connections: Mutex::new(HashMap::new()),
        }
    }
    
    /// シャードを追加
    pub async fn add_shard(&self, shard: ShardInfo) -> Result<(), Error> {
        let mut shards = self.shards.write().await;
        
        if shards.contains_key(&shard.id) {
            return Err(Error::ShardError(format!("Shard already exists: {}", shard.id)));
        }
        
        shards.insert(shard.id.clone(), shard);
        
        Ok(())
    }
    
    /// シャードを更新
    pub async fn update_shard(&self, shard: ShardInfo) -> Result<(), Error> {
        let mut shards = self.shards.write().await;
        
        if !shards.contains_key(&shard.id) {
            return Err(Error::ShardError(format!("Shard not found: {}", shard.id)));
        }
        
        shards.insert(shard.id.clone(), shard);
        
        Ok(())
    }
    
    /// シャードを削除
    pub async fn remove_shard(&self, shard_id: &ShardId) -> Result<(), Error> {
        let mut shards = self.shards.write().await;
        
        if !shards.contains_key(shard_id) {
            return Err(Error::ShardError(format!("Shard not found: {}", shard_id)));
        }
        
        shards.remove(shard_id);
        
        // 関連する接続情報も削除
        let mut connections = self.connections.lock().unwrap();
        
        connections.retain(|&(from, to), _| from != *shard_id && to != *shard_id);
        
        Ok(())
    }
    
    /// シャードを取得
    pub async fn get_shard(&self, shard_id: &ShardId) -> Result<ShardInfo, Error> {
        let shards = self.shards.read().await;
        
        shards.get(shard_id)
            .cloned()
            .ok_or_else(|| Error::ShardError(format!("Shard not found: {}", shard_id)))
    }
    
    /// すべてのシャードを取得
    pub async fn get_all_shards(&self) -> Result<Vec<ShardInfo>, Error> {
        let shards = self.shards.read().await;
        
        Ok(shards.values().cloned().collect())
    }
    
    /// アクティブなシャードを取得
    pub async fn get_active_shards(&self) -> Result<Vec<ShardInfo>, Error> {
        let shards = self.shards.read().await;
        
        Ok(shards.values()
            .filter(|shard| shard.status == ShardStatus::Active)
            .cloned()
            .collect())
    }
    
    /// シャード間の接続情報を追加
    pub fn add_connection(&self, connection: ConnectionInfo) -> Result<(), Error> {
        let mut connections = self.connections.lock().unwrap();
        
        let key = (connection.from.clone(), connection.to.clone());
        
        connections.insert(key, connection);
        
        Ok(())
    }
    
    /// シャード間の接続情報を更新
    pub fn update_connection(&self, connection: ConnectionInfo) -> Result<(), Error> {
        let mut connections = self.connections.lock().unwrap();
        
        let key = (connection.from.clone(), connection.to.clone());
        
        if !connections.contains_key(&key) {
            return Err(Error::ShardError(format!(
                "Connection not found: {} -> {}",
                connection.from,
                connection.to
            )));
        }
        
        connections.insert(key, connection);
        
        Ok(())
    }
    
    /// シャード間の接続情報を削除
    pub fn remove_connection(&self, from: &ShardId, to: &ShardId) -> Result<(), Error> {
        let mut connections = self.connections.lock().unwrap();
        
        let key = (from.clone(), to.clone());
        
        if !connections.contains_key(&key) {
            return Err(Error::ShardError(format!(
                "Connection not found: {} -> {}",
                from,
                to
            )));
        }
        
        connections.remove(&key);
        
        Ok(())
    }
    
    /// シャード間の接続情報を取得
    pub fn get_connection(&self, from: &ShardId, to: &ShardId) -> Result<ConnectionInfo, Error> {
        let connections = self.connections.lock().unwrap();
        
        let key = (from.clone(), to.clone());
        
        connections.get(&key)
            .cloned()
            .ok_or_else(|| Error::ShardError(format!(
                "Connection not found: {} -> {}",
                from,
                to
            )))
    }
    
    /// シャードからの接続情報をすべて取得
    pub fn get_connections_from(&self, from: &ShardId) -> Result<Vec<ConnectionInfo>, Error> {
        let connections = self.connections.lock().unwrap();
        
        Ok(connections.iter()
            .filter(|&((f, _), _)| f == from)
            .map(|(_, conn)| conn.clone())
            .collect())
    }
    
    /// シャードへの接続情報をすべて取得
    pub fn get_connections_to(&self, to: &ShardId) -> Result<Vec<ConnectionInfo>, Error> {
        let connections = self.connections.lock().unwrap();
        
        Ok(connections.iter()
            .filter(|&((_, t), _)| t == to)
            .map(|(_, conn)| conn.clone())
            .collect())
    }
    
    /// すべての接続情報を取得
    pub fn get_all_connections(&self) -> Result<Vec<ConnectionInfo>, Error> {
        let connections = self.connections.lock().unwrap();
        
        Ok(connections.values().cloned().collect())
    }
}

impl Default for ShardManager {
    fn default() -> Self {
        Self::new()
    }
}