use super::executor::{AsyncExecutor, PriorityAsyncExecutor, TaskPriority};
use crate::error::Error;
use crate::transaction::Transaction;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;

/// タスクスケジューラ
///
/// トランザクション処理のスケジューリングを行う。
/// 依存関係の解決、優先度付け、負荷分散を担当。
pub struct TaskScheduler {
    /// 優先度付き非同期実行エンジン
    executor: PriorityAsyncExecutor,
    /// タスクキュー
    task_queue: Arc<Mutex<TaskQueue>>,
    /// タスク送信チャネル
    task_sender: mpsc::Sender<ScheduledTask>,
    /// 統計情報
    stats: Arc<Mutex<SchedulerStats>>,
}

/// スケジュールされたタスク
struct ScheduledTask {
    /// トランザクション
    transaction: Transaction,
    /// 優先度
    priority: TaskPriority,
    /// スケジュール時刻
    scheduled_at: Instant,
    /// 依存するトランザクションID
    dependencies: Vec<String>,
}

/// タスクキュー
struct TaskQueue {
    /// 優先度キュー
    priority_queue: BinaryHeap<Reverse<PrioritizedTask>>,
    /// 依存関係マップ
    dependency_map: HashMap<String, Vec<String>>,
    /// 完了したトランザクションID
    completed_txs: HashMap<String, Instant>,
}

/// 優先度付きタスク
#[derive(Eq, PartialEq)]
struct PrioritizedTask {
    /// 優先度スコア（低いほど優先）
    score: u64,
    /// トランザクションID
    tx_id: String,
}

impl Ord for PrioritizedTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.cmp(&other.score)
    }
}

impl PartialOrd for PrioritizedTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// スケジューラ統計情報
struct SchedulerStats {
    /// 処理したタスク数
    processed_tasks: usize,
    /// 待機中のタスク数
    pending_tasks: usize,
    /// 平均処理時間
    avg_processing_time: Duration,
    /// 最大処理時間
    max_processing_time: Duration,
    /// 処理時間の合計
    total_processing_time: Duration,
}

impl TaskScheduler {
    /// 新しいTaskSchedulerを作成
    pub fn new(max_concurrency: Option<usize>) -> Result<Self, Error> {
        // 優先度付き非同期実行エンジンを作成
        let executor = PriorityAsyncExecutor::new(max_concurrency)?;

        // タスクキューを作成
        let task_queue = Arc::new(Mutex::new(TaskQueue {
            priority_queue: BinaryHeap::new(),
            dependency_map: HashMap::new(),
            completed_txs: HashMap::new(),
        }));

        // タスクチャネルを作成
        let (task_sender, mut task_receiver) = mpsc::channel::<ScheduledTask>(10000);

        // 統計情報を作成
        let stats = Arc::new(Mutex::new(SchedulerStats {
            processed_tasks: 0,
            pending_tasks: 0,
            avg_processing_time: Duration::from_secs(0),
            max_processing_time: Duration::from_secs(0),
            total_processing_time: Duration::from_secs(0),
        }));

        // タスク処理ループを開始
        let task_queue_clone = task_queue.clone();
        let stats_clone = stats.clone();
        let executor_clone = executor.clone();

        tokio::spawn(async move {
            while let Some(task) = task_receiver.recv().await {
                // 依存関係を解決
                let can_process = {
                    let mut queue = task_queue_clone.lock().unwrap();

                    // 依存するトランザクションがすべて完了しているか確認
                    let all_deps_completed = task
                        .dependencies
                        .iter()
                        .all(|dep_id| queue.completed_txs.contains_key(dep_id));

                    if all_deps_completed {
                        true
                    } else {
                        // 依存関係マップに追加
                        for dep_id in &task.dependencies {
                            queue
                                .dependency_map
                                .entry(dep_id.clone())
                                .or_insert_with(Vec::new)
                                .push(task.transaction.id.clone());
                        }

                        // 優先度キューに追加
                        let score = calculate_priority_score(&task);
                        queue.priority_queue.push(Reverse(PrioritizedTask {
                            score,
                            tx_id: task.transaction.id.clone(),
                        }));

                        // 統計情報を更新
                        let mut stats = stats_clone.lock().unwrap();
                        stats.pending_tasks += 1;

                        false
                    }
                };

                if can_process {
                    // タスクを実行
                    let start_time = Instant::now();
                    let tx = task.transaction.clone();
                    let tx_id = tx.id.clone();
                    let task_queue = task_queue_clone.clone();
                    let stats = stats_clone.clone();

                    let _ = executor_clone.spawn(
                        async move {
                            // トランザクションを処理
                            let result = process_transaction(tx).await;

                            // 処理時間を計測
                            let processing_time = start_time.elapsed();

                            // 完了したトランザクションを記録
                            {
                                let mut queue = task_queue.lock().unwrap();
                                queue.completed_txs.insert(tx_id.clone(), Instant::now());

                                // 依存するタスクを取得
                                if let Some(dependent_txs) = queue.dependency_map.remove(&tx_id) {
                                    // 依存するタスクの依存関係を更新
                                    for dep_tx_id in dependent_txs {
                                        // 優先度キューから取り出して再スケジュール
                                        // 実際の実装ではより効率的な方法が必要
                                    }
                                }
                            }

                            // 統計情報を更新
                            {
                                let mut stats = stats.lock().unwrap();
                                stats.processed_tasks += 1;
                                stats.pending_tasks -= 1;
                                stats.total_processing_time += processing_time;
                                stats.avg_processing_time =
                                    stats.total_processing_time / stats.processed_tasks as u32;
                                if processing_time > stats.max_processing_time {
                                    stats.max_processing_time = processing_time;
                                }
                            }

                            result
                        },
                        task.priority,
                    );
                }
            }
        });

        Ok(Self {
            executor,
            task_queue,
            task_sender,
            stats,
        })
    }

    /// トランザクションをスケジュール
    pub async fn schedule_transaction(
        &self,
        tx: Transaction,
        priority: Option<TaskPriority>,
    ) -> Result<(), Error> {
        // 依存関係を抽出
        let dependencies = tx.parent_ids.clone();

        // 優先度を決定
        let priority = priority.unwrap_or_else(|| determine_priority(&tx));

        // タスクを作成
        let task = ScheduledTask {
            transaction: tx,
            priority,
            scheduled_at: Instant::now(),
            dependencies,
        };

        // タスクをキューに送信
        self.task_sender
            .send(task)
            .await
            .map_err(|e| Error::InternalError(format!("Failed to send task: {}", e)))
    }

    /// 複数のトランザクションをスケジュール
    pub async fn schedule_batch(
        &self,
        txs: Vec<Transaction>,
        priority: Option<TaskPriority>,
    ) -> Result<(), Error> {
        for tx in txs {
            self.schedule_transaction(tx, priority).await?;
        }

        Ok(())
    }

    /// スケジューラの統計情報を取得
    pub fn get_stats(&self) -> SchedulerStats {
        self.stats.lock().unwrap().clone()
    }

    /// 現在の同時実行数を取得
    pub fn current_concurrency(&self, priority: Option<TaskPriority>) -> usize {
        self.executor.current_concurrency(priority)
    }

    /// 最大同時実行数を取得
    pub fn max_concurrency(&self, priority: Option<TaskPriority>) -> usize {
        self.executor.max_concurrency(priority)
    }

    /// 待機中のタスク数を取得
    pub fn pending_tasks(&self) -> usize {
        self.stats.lock().unwrap().pending_tasks
    }
}

/// トランザクションの優先度を決定
fn determine_priority(tx: &Transaction) -> TaskPriority {
    // 実際の実装ではトランザクションの内容に基づいて優先度を決定
    // ここではダミー実装
    TaskPriority::Normal
}

/// 優先度スコアを計算
fn calculate_priority_score(task: &ScheduledTask) -> u64 {
    // 優先度、待機時間、依存関係などに基づいてスコアを計算
    // 低いスコアほど優先度が高い

    // 基本スコア（優先度に基づく）
    let base_score = match task.priority {
        TaskPriority::Critical => 0,
        TaskPriority::High => 1000,
        TaskPriority::Normal => 10000,
        TaskPriority::Low => 100000,
    };

    // 待機時間による調整（長く待っているほど優先度が上がる）
    let wait_time = Instant::now().duration_since(task.scheduled_at).as_secs();
    let wait_adjustment = if wait_time > 60 {
        // 1分以上待っている場合は優先度を大幅に上げる
        base_score / 2
    } else {
        // 待機時間に応じて少しずつ優先度を上げる
        (base_score as f64 * (1.0 - (wait_time as f64 / 120.0))) as u64
    };

    // 依存関係の数による調整（依存が少ないほど優先度が上がる）
    let dependency_adjustment = task.dependencies.len() as u64 * 100;

    // 最終スコア
    wait_adjustment.saturating_add(dependency_adjustment)
}

/// トランザクションを処理
async fn process_transaction(tx: Transaction) -> Result<(), Error> {
    // 実際の実装ではトランザクションの処理ロジックを実装
    // ここではダミー実装

    // 少し待機してトランザクション処理をシミュレート
    sleep(Duration::from_millis(100)).await;

    Ok(())
}

// SchedulerStatsのClone実装
impl Clone for SchedulerStats {
    fn clone(&self) -> Self {
        Self {
            processed_tasks: self.processed_tasks,
            pending_tasks: self.pending_tasks,
            avg_processing_time: self.avg_processing_time,
            max_processing_time: self.max_processing_time,
            total_processing_time: self.total_processing_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::TransactionStatus;

    #[tokio::test]
    async fn test_schedule_transaction() {
        let scheduler = TaskScheduler::new(Some(4)).unwrap();

        // テスト用のトランザクション
        let tx = Transaction {
            id: "tx1".to_string(),
            parent_ids: vec![],
            timestamp: 12345,
            payload: vec![1, 2, 3],
            signature: vec![4, 5, 6],
            status: TransactionStatus::Pending,
        };

        // トランザクションをスケジュール
        let result = scheduler
            .schedule_transaction(tx, Some(TaskPriority::Normal))
            .await;

        // スケジュールが成功したことを確認
        assert!(result.is_ok());

        // 少し待機して処理が完了するのを待つ
        sleep(Duration::from_millis(500)).await;

        // 統計情報を確認
        let stats = scheduler.get_stats();
        assert!(stats.processed_tasks > 0);
    }

    #[tokio::test]
    async fn test_schedule_batch() {
        let scheduler = TaskScheduler::new(Some(4)).unwrap();

        // テスト用のトランザクション
        let txs = vec![
            Transaction {
                id: "tx1".to_string(),
                parent_ids: vec![],
                timestamp: 12345,
                payload: vec![1, 2, 3],
                signature: vec![4, 5, 6],
                status: TransactionStatus::Pending,
            },
            Transaction {
                id: "tx2".to_string(),
                parent_ids: vec!["tx1".to_string()],
                timestamp: 12346,
                payload: vec![7, 8, 9],
                signature: vec![10, 11, 12],
                status: TransactionStatus::Pending,
            },
        ];

        // トランザクションをバッチでスケジュール
        let result = scheduler
            .schedule_batch(txs, Some(TaskPriority::High))
            .await;

        // スケジュールが成功したことを確認
        assert!(result.is_ok());

        // 少し待機して処理が完了するのを待つ
        sleep(Duration::from_millis(500)).await;

        // 統計情報を確認
        let stats = scheduler.get_stats();
        assert!(stats.processed_tasks > 0);
    }

    #[tokio::test]
    async fn test_dependency_resolution() {
        let scheduler = TaskScheduler::new(Some(4)).unwrap();

        // 依存関係のあるトランザクション
        let tx1 = Transaction {
            id: "tx1".to_string(),
            parent_ids: vec![],
            timestamp: 12345,
            payload: vec![1, 2, 3],
            signature: vec![4, 5, 6],
            status: TransactionStatus::Pending,
        };

        let tx2 = Transaction {
            id: "tx2".to_string(),
            parent_ids: vec!["tx1".to_string()],
            timestamp: 12346,
            payload: vec![7, 8, 9],
            signature: vec![10, 11, 12],
            status: TransactionStatus::Pending,
        };

        // tx2を先にスケジュール
        scheduler
            .schedule_transaction(tx2, Some(TaskPriority::Normal))
            .await
            .unwrap();

        // tx1をスケジュール
        scheduler
            .schedule_transaction(tx1, Some(TaskPriority::Normal))
            .await
            .unwrap();

        // 少し待機して処理が完了するのを待つ
        sleep(Duration::from_millis(500)).await;

        // 統計情報を確認
        let stats = scheduler.get_stats();
        assert_eq!(stats.processed_tasks, 2);
        assert_eq!(stats.pending_tasks, 0);
    }
}
