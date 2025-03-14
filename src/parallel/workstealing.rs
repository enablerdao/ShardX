use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use std::sync::mpsc::channel;
use rayon::prelude::*;
use crate::error::Error;

/// 負荷制限付きワークスティーリングスケジューラ
pub struct WorkStealingScheduler {
    pool: rayon::ThreadPool,
    cpu_limit: AtomicU32,
}

impl WorkStealingScheduler {
    /// 新しいWorkStealingSchedulerを作成
    pub fn new() -> Self {
        // Rayonスレッドプールを設定
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_cpus::get())
            .build()
            .unwrap();
        
        Self {
            pool,
            cpu_limit: AtomicU32::new(50), // デフォルトでCPU使用率50%上限
        }
    }
    
    /// CPU使用率上限を設定
    pub fn set_cpu_limit(&self, limit_percent: u32) {
        self.cpu_limit.store(limit_percent, Ordering::SeqCst);
    }
    
    /// バッチ処理を実行
    pub fn process_batch<T, F>(&self, items: Vec<T>, processor: F) -> Vec<Result<(), Error>>
    where
        T: Send + Clone,
        F: Fn(T) -> Result<(), Error> + Send + Sync + Clone,
    {
        // CPUモニタリングスレッドを起動
        let cpu_limit = self.cpu_limit.load(Ordering::SeqCst);
        let (pause_sender, pause_receiver) = channel();
        let monitor_handle = std::thread::spawn(move || {
            loop {
                let cpu_usage = get_cpu_usage();
                if cpu_usage > cpu_limit as f32 {
                    // CPU使用率が上限を超えたら一時停止信号を送信
                    let _ = pause_sender.send(true);
                    std::thread::sleep(Duration::from_millis(100));
                } else {
                    let _ = pause_sender.send(false);
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        });
        
        // アイテムをチャンク（小タスク）に分割
        let chunk_size = if num_cpus::get() >= 8 {
            50 // 高スペックノードは大タスク
        } else {
            10 // 低スペックノードは小タスク
        };
        
        let chunks: Vec<Vec<T>> = items
            .chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect();
        
        // 結果を格納する配列
        let mut results = Vec::with_capacity(items.len());
        
        // 各チャンクを並列処理
        self.pool.install(|| {
            let chunk_results: Vec<Vec<Result<(), Error>>> = chunks
                .par_iter()
                .map(|chunk| {
                    // CPU使用率をチェック
                    if pause_receiver.try_recv().unwrap_or(false) {
                        // 一時停止信号を受信したら少し待機
                        std::thread::sleep(Duration::from_millis(50));
                    }
                    
                    // チャンク内のアイテムを処理
                    chunk.iter().map(|item| processor.clone()(item.clone())).collect()
                })
                .collect();
            
            // 結果を平坦化
            for chunk_result in chunk_results {
                results.extend(chunk_result);
            }
        });
        
        // モニタリングスレッドを終了
        monitor_handle.thread().unpark();
        
        results
    }
}

/// CPU使用率を取得
fn get_cpu_usage() -> f32 {
    // 実際の実装ではシステムのCPU使用率を取得
    // ここではダミー実装
    30.0 // 30%と仮定
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_workstealing_scheduler() {
        let scheduler = WorkStealingScheduler::new();
        
        // テスト用のアイテム
        let items: Vec<u32> = (0..100).collect();
        
        // テスト用の処理関数
        let processor = |item: u32| -> Result<(), Error> {
            // 単純な処理
            let _ = item * 2;
            Ok(())
        };
        
        // バッチ処理を実行
        let results = scheduler.process_batch(items, processor);
        
        // 結果を確認
        assert_eq!(results.len(), 100);
        for result in results {
            assert!(result.is_ok());
        }
    }
    
    #[test]
    fn test_cpu_limit() {
        let scheduler = WorkStealingScheduler::new();
        
        // CPU使用率上限を設定
        scheduler.set_cpu_limit(70);
        
        // 設定が反映されていることを確認
        assert_eq!(scheduler.cpu_limit.load(Ordering::SeqCst), 70);
    }
}