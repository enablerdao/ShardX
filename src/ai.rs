use crate::transaction::Transaction;
use log::info;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex};

/// トランザクションの優先度を表す構造体
#[derive(Debug, Clone, PartialEq, Eq)]
struct PrioritizedTransaction {
    /// トランザクションの参照
    tx: Transaction,
    /// 優先スコア（高いほど優先）
    score: u32,
}

impl PartialOrd for PrioritizedTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedTransaction {
    fn cmp(&self, other: &Self) -> Ordering {
        // スコアの高い順（降順）でソート
        self.score.cmp(&other.score).reverse()
    }
}

/// AIベースのトランザクション優先度マネージャー
pub struct AIPriorityManager {
    /// 優先キュー
    queue: Arc<Mutex<BinaryHeap<PrioritizedTransaction>>>,
}

impl AIPriorityManager {
    /// 新しい優先度マネージャーを作成
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(BinaryHeap::new())),
        }
    }
    
    /// トランザクションの優先度を計算
    fn calculate_priority(&self, tx: &Transaction) -> u32 {
        // 実際の実装では、機械学習モデルを使用して優先度を計算
        // 簡略化のため、以下の要素に基づいて優先度を計算:
        // 1. ペイロードサイズ（小さいほど優先）
        // 2. 親トランザクションの数（多いほど優先）
        // 3. タイムスタンプ（古いほど優先）
        
        let size_score = 1000 - tx.payload.len().min(1000) as u32;
        let parent_score = tx.parent_ids.len() as u32 * 100;
        let time_score = (chrono::Utc::now().timestamp_millis() as u64 - tx.timestamp).min(1000) as u32;
        
        size_score + parent_score + time_score
    }
    
    /// トランザクションをキューに追加
    pub fn enqueue(&self, tx: Transaction) {
        let score = self.calculate_priority(&tx);
        let prioritized_tx = PrioritizedTransaction { tx: tx.clone(), score };
        
        let mut queue = self.queue.lock().unwrap();
        queue.push(prioritized_tx);
        
        info!("Transaction {} added to queue with priority {}", tx.id, score);
    }
    
    /// 最も優先度の高いトランザクションを取得
    pub fn dequeue(&self) -> Option<Transaction> {
        let mut queue = self.queue.lock().unwrap();
        queue.pop().map(|prioritized_tx| {
            info!(
                "Transaction {} dequeued with priority {}",
                prioritized_tx.tx.id, prioritized_tx.score
            );
            prioritized_tx.tx
        })
    }
    
    /// キューのサイズを取得
    pub fn queue_size(&self) -> usize {
        let queue = self.queue.lock().unwrap();
        queue.len()
    }
}