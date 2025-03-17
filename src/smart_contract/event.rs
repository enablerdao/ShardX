use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};

/// コントラクトイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEvent {
    /// イベント名
    pub name: String,
    /// トピック
    pub topics: Vec<String>,
    /// データ
    pub data: Vec<u8>,
    /// インデックス
    pub indexed: Vec<bool>,
    /// 匿名フラグ
    pub anonymous: bool,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// イベントログ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLog {
    /// コントラクトアドレス
    pub address: String,
    /// トピック
    pub topics: Vec<String>,
    /// データ
    pub data: Vec<u8>,
    /// ブロック高
    pub block_height: u64,
    /// ブロック時間
    pub block_time: DateTime<Utc>,
    /// トランザクションハッシュ
    pub transaction_hash: String,
    /// トランザクションインデックス
    pub transaction_index: u32,
    /// ログインデックス
    pub log_index: u32,
    /// 削除フラグ
    pub removed: bool,
}

/// イベントフィルタ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    /// アドレス
    pub addresses: Option<Vec<String>>,
    /// トピック
    pub topics: Option<Vec<Option<String>>>,
    /// 開始ブロック
    pub from_block: Option<u64>,
    /// 終了ブロック
    pub to_block: Option<u64>,
    /// ブロックハッシュ
    pub block_hash: Option<String>,
}

/// イベント購読
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSubscription {
    /// 購読ID
    pub id: String,
    /// フィルタ
    pub filter: EventFilter,
    /// 作成時刻
    pub created_at: DateTime<Utc>,
    /// 更新時刻
    pub updated_at: DateTime<Utc>,
    /// 最後の通知時刻
    pub last_notification_at: Option<DateTime<Utc>>,
    /// 最後の通知ブロック
    pub last_notification_block: Option<u64>,
    /// 通知数
    pub notification_count: u64,
    /// 有効フラグ
    pub enabled: bool,
    /// メタデータ
    pub metadata: Option<HashMap<String, String>>,
}

/// イベントマネージャー
pub struct EventManager {
    /// イベントログ
    logs: Vec<EventLog>,
    /// 購読
    subscriptions: HashMap<String, EventSubscription>,
    /// 最大ログ数
    max_logs: usize,
}

impl EventManager {
    /// 新しいイベントマネージャーを作成
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: Vec::new(),
            subscriptions: HashMap::new(),
            max_logs,
        }
    }
    
    /// イベントログを追加
    pub fn add_log(&mut self, log: EventLog) {
        self.logs.push(log);
        
        // 最大ログ数を超えた場合、古いログを削除
        if self.logs.len() > self.max_logs {
            self.logs.remove(0);
        }
        
        // 購読者に通知
        self.notify_subscribers();
    }
    
    /// イベントログを取得
    pub fn get_logs(&self, filter: Option<&EventFilter>) -> Vec<&EventLog> {
        if let Some(filter) = filter {
            // フィルタを適用
            self.logs.iter()
                .filter(|log| {
                    // アドレスフィルタ
                    if let Some(addresses) = &filter.addresses {
                        if !addresses.contains(&log.address) {
                            return false;
                        }
                    }
                    
                    // トピックフィルタ
                    if let Some(topics) = &filter.topics {
                        for (i, topic) in topics.iter().enumerate() {
                            if let Some(topic) = topic {
                                if i >= log.topics.len() || log.topics[i] != *topic {
                                    return false;
                                }
                            }
                        }
                    }
                    
                    // ブロック高さフィルタ
                    if let Some(from_block) = filter.from_block {
                        if log.block_height < from_block {
                            return false;
                        }
                    }
                    
                    if let Some(to_block) = filter.to_block {
                        if log.block_height > to_block {
                            return false;
                        }
                    }
                    
                    // ブロックハッシュフィルタ
                    if let Some(block_hash) = &filter.block_hash {
                        // 実際の実装では、ブロックハッシュを比較する
                        // ここでは簡易的に常にtrueを返す
                        true
                    } else {
                        true
                    }
                })
                .collect()
        } else {
            // フィルタなし
            self.logs.iter().collect()
        }
    }
    
    /// 購読を追加
    pub fn subscribe(&mut self, filter: EventFilter) -> String {
        let id = format!("subscription_{}", self.subscriptions.len());
        let subscription = EventSubscription {
            id: id.clone(),
            filter,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_notification_at: None,
            last_notification_block: None,
            notification_count: 0,
            enabled: true,
            metadata: None,
        };
        
        self.subscriptions.insert(id.clone(), subscription);
        
        id
    }
    
    /// 購読を解除
    pub fn unsubscribe(&mut self, id: &str) -> bool {
        self.subscriptions.remove(id).is_some()
    }
    
    /// 購読を取得
    pub fn get_subscription(&self, id: &str) -> Option<&EventSubscription> {
        self.subscriptions.get(id)
    }
    
    /// 購読を更新
    pub fn update_subscription(&mut self, id: &str, filter: EventFilter) -> bool {
        if let Some(subscription) = self.subscriptions.get_mut(id) {
            subscription.filter = filter;
            subscription.updated_at = Utc::now();
            true
        } else {
            false
        }
    }
    
    /// 購読を有効化
    pub fn enable_subscription(&mut self, id: &str) -> bool {
        if let Some(subscription) = self.subscriptions.get_mut(id) {
            subscription.enabled = true;
            subscription.updated_at = Utc::now();
            true
        } else {
            false
        }
    }
    
    /// 購読を無効化
    pub fn disable_subscription(&mut self, id: &str) -> bool {
        if let Some(subscription) = self.subscriptions.get_mut(id) {
            subscription.enabled = false;
            subscription.updated_at = Utc::now();
            true
        } else {
            false
        }
    }
    
    /// 購読者に通知
    fn notify_subscribers(&mut self) {
        let now = Utc::now();
        
        for (id, subscription) in self.subscriptions.iter_mut() {
            if !subscription.enabled {
                continue;
            }
            
            // 最後の通知以降のログを取得
            let last_block = subscription.last_notification_block.unwrap_or(0);
            let logs = self.get_logs(Some(&EventFilter {
                addresses: subscription.filter.addresses.clone(),
                topics: subscription.filter.topics.clone(),
                from_block: Some(last_block + 1),
                to_block: None,
                block_hash: None,
            }));
            
            if !logs.is_empty() {
                // 通知を送信（実際の実装では、通知を送信する処理を実装する）
                debug!("Notifying subscription {} with {} logs", id, logs.len());
                
                // 最後の通知情報を更新
                subscription.last_notification_at = Some(now);
                subscription.last_notification_block = logs.last().map(|log| log.block_height);
                subscription.notification_count += logs.len() as u64;
                subscription.updated_at = now;
            }
        }
    }
    
    /// イベントログをクリア
    pub fn clear_logs(&mut self) {
        self.logs.clear();
    }
    
    /// 購読をクリア
    pub fn clear_subscriptions(&mut self) {
        self.subscriptions.clear();
    }
}