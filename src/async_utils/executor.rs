use crate::error::Error;
use futures::future::join_all;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::{mpsc, Semaphore};
use tokio::task::JoinHandle;

/// 非同期タスク実行エンジン
///
/// バックプレッシャー機構を備えた高性能な非同期タスク実行エンジン。
/// 並列度の制御、タスクのプライオリティ付け、リソース管理を行う。
pub struct AsyncExecutor {
    /// Tokioランタイム
    runtime: Runtime,
    /// タスク送信チャネル
    task_sender: mpsc::Sender<Task>,
    /// 同時実行数を制限するセマフォ
    semaphore: Arc<Semaphore>,
    /// 最大同時実行数
    max_concurrency: usize,
}

/// 非同期タスク
type Task = Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>;

impl AsyncExecutor {
    /// 新しいAsyncExecutorを作成
    pub fn new(max_concurrency: Option<usize>) -> Result<Self, Error> {
        // 最大同時実行数（デフォルトはCPUコア数）
        let max_concurrency = max_concurrency.unwrap_or_else(num_cpus::get);

        // Tokioランタイムを作成
        let runtime = Builder::new_multi_thread()
            .worker_threads(max_concurrency)
            .thread_name("shardx-worker")
            .thread_stack_size(3 * 1024 * 1024) // 3MB
            .enable_all()
            .build()
            .map_err(|e| Error::InternalError(format!("Failed to create runtime: {}", e)))?;

        // タスクチャネルを作成
        let (task_sender, mut task_receiver) = mpsc::channel::<Task>(10000);

        // セマフォを作成
        let semaphore = Arc::new(Semaphore::new(max_concurrency));

        // タスク処理ループを開始
        let semaphore_clone = semaphore.clone();
        runtime.spawn(async move {
            while let Some(task) = task_receiver.recv().await {
                let permit = semaphore_clone.clone().acquire_owned().await.unwrap();

                tokio::spawn(async move {
                    let _permit = permit; // permitをドロップするとセマフォが解放される
                    let _ = task.await;
                });
            }
        });

        Ok(Self {
            runtime,
            task_sender,
            semaphore,
            max_concurrency,
        })
    }

    /// タスクを実行キューに追加
    pub fn spawn<F>(&self, future: F) -> Result<(), Error>
    where
        F: Future<Output = Result<(), Error>> + Send + 'static,
    {
        let sender = self.task_sender.clone();

        self.runtime.block_on(async {
            sender
                .send(Box::pin(future))
                .await
                .map_err(|e| Error::InternalError(format!("Failed to send task: {}", e)))
        })
    }

    /// 複数のタスクを実行し、すべての結果を待機
    pub fn spawn_batch<F, I>(&self, futures: I) -> Vec<Result<(), Error>>
    where
        F: Future<Output = Result<(), Error>> + Send + 'static,
        I: IntoIterator<Item = F>,
    {
        let futures: Vec<_> = futures.into_iter().collect();

        self.runtime.block_on(async {
            let mut handles = Vec::with_capacity(futures.len());

            for future in futures {
                let semaphore = self.semaphore.clone();

                let handle: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
                    // セマフォを取得
                    let permit = semaphore.acquire_owned().await.map_err(|e| {
                        Error::InternalError(format!("Failed to acquire semaphore: {}", e))
                    })?;

                    // タスクを実行
                    let result = future.await;

                    // permitをドロップしてセマフォを解放
                    drop(permit);

                    result
                });

                handles.push(handle);
            }

            // すべてのタスクの完了を待機
            let results = join_all(handles).await;

            // 結果を変換
            results
                .into_iter()
                .map(|r| match r {
                    Ok(result) => result,
                    Err(e) => Err(Error::InternalError(format!("Task panicked: {}", e))),
                })
                .collect()
        })
    }

    /// 現在の同時実行数を取得
    pub fn current_concurrency(&self) -> usize {
        self.max_concurrency - self.semaphore.available_permits()
    }

    /// 最大同時実行数を取得
    pub fn max_concurrency(&self) -> usize {
        self.max_concurrency
    }

    /// 実行キューのサイズを取得
    pub fn queue_size(&self) -> usize {
        // 現在のチャネルの長さを取得
        self.task_sender.capacity().unwrap_or(0)
    }
}

/// 非同期タスクの優先度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// 優先度付き非同期実行エンジン
pub struct PriorityAsyncExecutor {
    /// 優先度ごとの実行エンジン
    executors: [AsyncExecutor; 4],
}

impl PriorityAsyncExecutor {
    /// 新しいPriorityAsyncExecutorを作成
    pub fn new(max_concurrency: Option<usize>) -> Result<Self, Error> {
        let max_concurrency = max_concurrency.unwrap_or_else(num_cpus::get);

        // 優先度ごとの同時実行数を計算
        // Critical: 40%, High: 30%, Normal: 20%, Low: 10%
        let critical_concurrency = (max_concurrency * 4) / 10;
        let high_concurrency = (max_concurrency * 3) / 10;
        let normal_concurrency = (max_concurrency * 2) / 10;
        let low_concurrency =
            max_concurrency - critical_concurrency - high_concurrency - normal_concurrency;

        // 各優先度の実行エンジンを作成
        let executors = [
            AsyncExecutor::new(Some(low_concurrency))?,
            AsyncExecutor::new(Some(normal_concurrency))?,
            AsyncExecutor::new(Some(high_concurrency))?,
            AsyncExecutor::new(Some(critical_concurrency))?,
        ];

        Ok(Self { executors })
    }

    /// 優先度付きタスクを実行キューに追加
    pub fn spawn<F>(&self, future: F, priority: TaskPriority) -> Result<(), Error>
    where
        F: Future<Output = Result<(), Error>> + Send + 'static,
    {
        let executor = &self.executors[priority as usize];
        executor.spawn(future)
    }

    /// 複数の優先度付きタスクを実行し、すべての結果を待機
    pub fn spawn_batch<F, I>(&self, futures: I, priority: TaskPriority) -> Vec<Result<(), Error>>
    where
        F: Future<Output = Result<(), Error>> + Send + 'static,
        I: IntoIterator<Item = F>,
    {
        let executor = &self.executors[priority as usize];
        executor.spawn_batch(futures)
    }

    /// 現在の同時実行数を取得
    pub fn current_concurrency(&self, priority: Option<TaskPriority>) -> usize {
        match priority {
            Some(p) => self.executors[p as usize].current_concurrency(),
            None => self.executors.iter().map(|e| e.current_concurrency()).sum(),
        }
    }

    /// 最大同時実行数を取得
    pub fn max_concurrency(&self, priority: Option<TaskPriority>) -> usize {
        match priority {
            Some(p) => self.executors[p as usize].max_concurrency(),
            None => self.executors.iter().map(|e| e.max_concurrency()).sum(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[test]
    fn test_async_executor() {
        let executor = AsyncExecutor::new(Some(4)).unwrap();

        // カウンタを作成
        let counter = Arc::new(AtomicUsize::new(0));

        // 10個のタスクを作成
        let mut futures = Vec::new();
        for i in 0..10 {
            let counter_clone = counter.clone();

            futures.push(async move {
                // 少し待機
                tokio::time::sleep(Duration::from_millis(100)).await;

                // カウンタをインクリメント
                counter_clone.fetch_add(1, Ordering::SeqCst);

                println!("Task {} completed", i);
                Ok(())
            });
        }

        // タスクを実行
        let results = executor.spawn_batch(futures);

        // すべてのタスクが成功したことを確認
        for result in results {
            assert!(result.is_ok());
        }

        // カウンタが10になっていることを確認
        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_priority_executor() {
        let executor = PriorityAsyncExecutor::new(Some(4)).unwrap();

        // カウンタを作成
        let counter = Arc::new(AtomicUsize::new(0));

        // 各優先度のタスクを作成
        let mut low_futures = Vec::new();
        let mut normal_futures = Vec::new();
        let mut high_futures = Vec::new();
        let mut critical_futures = Vec::new();

        for i in 0..5 {
            let counter_clone = counter.clone();

            low_futures.push(async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                counter_clone.fetch_add(1, Ordering::SeqCst);
                println!("Low priority task {} completed", i);
                Ok(())
            });

            let counter_clone = counter.clone();
            normal_futures.push(async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                counter_clone.fetch_add(1, Ordering::SeqCst);
                println!("Normal priority task {} completed", i);
                Ok(())
            });

            let counter_clone = counter.clone();
            high_futures.push(async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                counter_clone.fetch_add(1, Ordering::SeqCst);
                println!("High priority task {} completed", i);
                Ok(())
            });

            let counter_clone = counter.clone();
            critical_futures.push(async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                counter_clone.fetch_add(1, Ordering::SeqCst);
                println!("Critical priority task {} completed", i);
                Ok(())
            });
        }

        // タスクを実行
        let low_results = executor.spawn_batch(low_futures, TaskPriority::Low);
        let normal_results = executor.spawn_batch(normal_futures, TaskPriority::Normal);
        let high_results = executor.spawn_batch(high_futures, TaskPriority::High);
        let critical_results = executor.spawn_batch(critical_futures, TaskPriority::Critical);

        // すべてのタスクが成功したことを確認
        for result in low_results
            .iter()
            .chain(normal_results.iter())
            .chain(high_results.iter())
            .chain(critical_results.iter())
        {
            assert!(result.is_ok());
        }

        // カウンタが20になっていることを確認
        assert_eq!(counter.load(Ordering::SeqCst), 20);
    }
}
