use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use log::{debug, error, info, warn};
use tokio::time;

use crate::error::Error;
use crate::storage::{StorageManager, StorageKey, StorageValue};
use crate::metrics::MetricsCollector;

/// ストレージ最適化器
/// 
/// ストレージの効率を改善するための最適化を行う。
/// - データ圧縮
/// - キャッシュ最適化
/// - インデックス最適化
/// - ストレージレイアウト最適化
/// - ガベージコレクション
pub struct StorageOptimizer {
    /// ストレージマネージャー
    storage_manager: Arc<StorageManager>,
    /// アクセス頻度カウンター
    access_counter: Arc<Mutex<HashMap<StorageKey, u32>>>,
    /// 最後の書き込み時刻
    last_write: Arc<Mutex<HashMap<StorageKey, Instant>>>,
    /// ホットキー
    hot_keys: Arc<Mutex<HashSet<StorageKey>>>,
    /// コールドキー
    cold_keys: Arc<Mutex<HashSet<StorageKey>>>,
    /// 圧縮済みキー
    compressed_keys: Arc<Mutex<HashSet<StorageKey>>>,
    /// メトリクスコレクター
    metrics: Arc<MetricsCollector>,
    /// 最後の最適化時刻
    last_optimization: Arc<Mutex<Instant>>,
    /// 最適化間隔（秒）
    optimization_interval_secs: u64,
    /// ホットキー閾値
    hot_key_threshold: u32,
    /// コールドキー閾値（秒）
    cold_key_threshold_secs: u64,
    /// 圧縮閾値（バイト）
    compression_threshold: usize,
    /// 圧縮レベル（0-9）
    compression_level: u32,
    /// 実行中フラグ
    running: Arc<Mutex<bool>>,
}

/// 最適化統計
#[derive(Debug, Clone)]
pub struct OptimizationStats {
    /// 圧縮されたキーの数
    pub compressed_keys: usize,
    /// 圧縮前の合計サイズ（バイト）
    pub original_size: usize,
    /// 圧縮後の合計サイズ（バイト）
    pub compressed_size: usize,
    /// 削除されたキーの数
    pub deleted_keys: usize,
    /// 解放されたストレージ（バイト）
    pub freed_storage: usize,
    /// 最適化にかかった時間（秒）
    pub optimization_time: f64,
    /// ホットキーの数
    pub hot_keys: usize,
    /// コールドキーの数
    pub cold_keys: usize,
}

impl StorageOptimizer {
    /// 新しいStorageOptimizerを作成
    pub fn new(
        storage_manager: Arc<StorageManager>,
        metrics: Arc<MetricsCollector>,
    ) -> Self {
        Self {
            storage_manager,
            access_counter: Arc::new(Mutex::new(HashMap::new())),
            last_write: Arc::new(Mutex::new(HashMap::new())),
            hot_keys: Arc::new(Mutex::new(HashSet::new())),
            cold_keys: Arc::new(Mutex::new(HashSet::new())),
            compressed_keys: Arc::new(Mutex::new(HashSet::new())),
            metrics,
            last_optimization: Arc::new(Mutex::new(Instant::now())),
            optimization_interval_secs: 3600, // 1時間ごとに最適化
            hot_key_threshold: 100, // 100回以上アクセスされたキーをホットキーとする
            cold_key_threshold_secs: 86400 * 7, // 7日間アクセスされていないキーをコールドキーとする
            compression_threshold: 1024, // 1KB以上のデータを圧縮
            compression_level: 6, // 中程度の圧縮レベル
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    /// キーへのアクセスを記録
    pub fn record_access(&self, key: &StorageKey) {
        let mut access_counter = self.access_counter.lock().unwrap();
        let count = access_counter.entry(key.clone()).or_insert(0);
        *count += 1;
        
        // ホットキーを更新
        if *count >= self.hot_key_threshold {
            let mut hot_keys = self.hot_keys.lock().unwrap();
            hot_keys.insert(key.clone());
        }
    }
    
    /// キーへの書き込みを記録
    pub fn record_write(&self, key: &StorageKey) {
        let mut last_write = self.last_write.lock().unwrap();
        last_write.insert(key.clone(), Instant::now());
    }
    
    /// 最適化処理を開始
    pub async fn start_optimization(&self) -> Result<(), Error> {
        // 既に実行中かチェック
        let mut running = self.running.lock().unwrap();
        if *running {
            return Err(Error::InvalidState("Storage optimizer is already running".to_string()));
        }
        
        *running = true;
        drop(running);
        
        // 最適化タスクを開始
        let access_counter = self.access_counter.clone();
        let last_write = self.last_write.clone();
        let hot_keys = self.hot_keys.clone();
        let cold_keys = self.cold_keys.clone();
        let compressed_keys = self.compressed_keys.clone();
        let metrics = self.metrics.clone();
        let last_optimization = self.last_optimization.clone();
        let optimization_interval_secs = self.optimization_interval_secs;
        let hot_key_threshold = self.hot_key_threshold;
        let cold_key_threshold_secs = self.cold_key_threshold_secs;
        let compression_threshold = self.compression_threshold;
        let compression_level = self.compression_level;
        let storage_manager = self.storage_manager.clone();
        let running = self.running.clone();
        
        tokio::spawn(async move {
            while *running.lock().unwrap() {
                // 最適化間隔をチェック
                let should_optimize = {
                    let last_opt = last_optimization.lock().unwrap();
                    last_opt.elapsed().as_secs() >= optimization_interval_secs
                };
                
                if should_optimize {
                    // 最適化を実行
                    info!("Starting storage optimization");
                    let start_time = Instant::now();
                    
                    // コールドキーを検出
                    let now = Instant::now();
                    let mut cold_keys_detected = Vec::new();
                    
                    {
                        let last_write = last_write.lock().unwrap();
                        
                        for (key, timestamp) in last_write.iter() {
                            if timestamp.elapsed().as_secs() >= cold_key_threshold_secs {
                                cold_keys_detected.push(key.clone());
                            }
                        }
                    }
                    
                    // コールドキーを更新
                    {
                        let mut cold_keys = cold_keys.lock().unwrap();
                        for key in cold_keys_detected {
                            cold_keys.insert(key);
                        }
                    }
                    
                    // 圧縮対象のキーを検出
                    let mut compression_candidates = Vec::new();
                    let mut original_size = 0;
                    let mut compressed_size = 0;
                    let mut compressed_count = 0;
                    
                    // 圧縮済みキーを取得
                    let compressed_keys_set = {
                        let compressed_keys = compressed_keys.lock().unwrap();
                        compressed_keys.clone()
                    };
                    
                    // ストレージからすべてのキーを取得
                    let all_keys = storage_manager.get_all_keys().unwrap_or_default();
                    
                    for key in all_keys {
                        // 既に圧縮済みのキーはスキップ
                        if compressed_keys_set.contains(&key) {
                            continue;
                        }
                        
                        // キーの値を取得
                        if let Ok(Some(value)) = storage_manager.get(&key) {
                            // 圧縮閾値を超えるデータのみ圧縮
                            if value.len() >= compression_threshold {
                                compression_candidates.push((key, value));
                            }
                        }
                    }
                    
                    // 圧縮を実行
                    for (key, value) in compression_candidates {
                        // データを圧縮
                        let compressed_value = Self::compress_data(&value, compression_level);
                        
                        // 圧縮率をチェック
                        if compressed_value.len() < value.len() {
                            // 圧縮されたデータを保存
                            if let Err(e) = storage_manager.put(&key, &compressed_value) {
                                error!("Failed to store compressed data: {}", e);
                                continue;
                            }
                            
                            // 圧縮済みキーに追加
                            {
                                let mut compressed_keys = compressed_keys.lock().unwrap();
                                compressed_keys.insert(key);
                            }
                            
                            // 統計を更新
                            original_size += value.len();
                            compressed_size += compressed_value.len();
                            compressed_count += 1;
                        }
                    }
                    
                    // ガベージコレクションを実行
                    let mut deleted_keys = 0;
                    let mut freed_storage = 0;
                    
                    // 削除対象のキーを検出
                    let garbage_keys = {
                        let cold_keys = cold_keys.lock().unwrap();
                        let access_counter = access_counter.lock().unwrap();
                        
                        cold_keys.iter()
                            .filter(|key| {
                                // アクセス頻度が低いキーのみ削除
                                access_counter.get(*key).cloned().unwrap_or(0) < hot_key_threshold / 10
                            })
                            .cloned()
                            .collect::<Vec<_>>()
                    };
                    
                    // キーを削除
                    for key in garbage_keys {
                        // キーの値を取得（サイズ計算用）
                        if let Ok(Some(value)) = storage_manager.get(&key) {
                            freed_storage += value.len();
                        }
                        
                        // キーを削除
                        if let Err(e) = storage_manager.delete(&key) {
                            error!("Failed to delete key: {}", e);
                            continue;
                        }
                        
                        // 各種マップから削除
                        {
                            let mut access_counter = access_counter.lock().unwrap();
                            let mut last_write = last_write.lock().unwrap();
                            let mut hot_keys = hot_keys.lock().unwrap();
                            let mut cold_keys = cold_keys.lock().unwrap();
                            let mut compressed_keys = compressed_keys.lock().unwrap();
                            
                            access_counter.remove(&key);
                            last_write.remove(&key);
                            hot_keys.remove(&key);
                            cold_keys.remove(&key);
                            compressed_keys.remove(&key);
                        }
                        
                        deleted_keys += 1;
                    }
                    
                    // 最適化統計を作成
                    let stats = OptimizationStats {
                        compressed_keys: compressed_count,
                        original_size,
                        compressed_size,
                        deleted_keys,
                        freed_storage,
                        optimization_time: start_time.elapsed().as_secs_f64(),
                        hot_keys: hot_keys.lock().unwrap().len(),
                        cold_keys: cold_keys.lock().unwrap().len(),
                    };
                    
                    // 最適化結果をログに出力
                    info!("Storage optimization completed: {:?}", stats);
                    
                    // メトリクスを更新
                    metrics.set_gauge("storage_hot_keys", stats.hot_keys as f64);
                    metrics.set_gauge("storage_cold_keys", stats.cold_keys as f64);
                    metrics.set_gauge("storage_compressed_keys", compressed_keys.lock().unwrap().len() as f64);
                    metrics.increment_counter_by("storage_compressed_keys_count", compressed_count as u64);
                    metrics.increment_counter_by("storage_deleted_keys_count", deleted_keys as u64);
                    metrics.increment_counter_by("storage_freed_bytes", freed_storage as u64);
                    
                    if compressed_count > 0 {
                        let compression_ratio = original_size as f64 / compressed_size as f64;
                        metrics.observe_histogram("storage_compression_ratio", compression_ratio);
                    }
                    
                    metrics.observe_histogram("storage_optimization_time", stats.optimization_time);
                    
                    // 最後の最適化時刻を更新
                    *last_optimization.lock().unwrap() = Instant::now();
                }
                
                // 1分待機
                time::sleep(Duration::from_secs(60)).await;
            }
        });
        
        Ok(())
    }
    
    /// 処理を停止
    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }
    
    /// データを圧縮
    fn compress_data(data: &[u8], level: u32) -> Vec<u8> {
        use flate2::{Compression, write::ZlibEncoder};
        use std::io::Write;
        
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(level));
        encoder.write_all(data).unwrap_or_default();
        encoder.finish().unwrap_or_default()
    }
    
    /// データを解凍
    fn decompress_data(compressed_data: &[u8]) -> Result<Vec<u8>, Error> {
        use flate2::read::ZlibDecoder;
        use std::io::Read;
        
        let mut decoder = ZlibDecoder::new(compressed_data);
        let mut decompressed = Vec::new();
        
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| Error::DecompressionError(format!("Failed to decompress data: {}", e)))?;
        
        Ok(decompressed)
    }
    
    /// ホットキーの数を取得
    pub fn get_hot_keys_count(&self) -> usize {
        self.hot_keys.lock().unwrap().len()
    }
    
    /// コールドキーの数を取得
    pub fn get_cold_keys_count(&self) -> usize {
        self.cold_keys.lock().unwrap().len()
    }
    
    /// 圧縮済みキーの数を取得
    pub fn get_compressed_keys_count(&self) -> usize {
        self.compressed_keys.lock().unwrap().len()
    }
    
    /// ホットキー閾値を設定
    pub fn set_hot_key_threshold(&mut self, threshold: u32) {
        self.hot_key_threshold = threshold;
    }
    
    /// コールドキー閾値を設定
    pub fn set_cold_key_threshold_secs(&mut self, threshold_secs: u64) {
        self.cold_key_threshold_secs = threshold_secs;
    }
    
    /// 圧縮閾値を設定
    pub fn set_compression_threshold(&mut self, threshold: usize) {
        self.compression_threshold = threshold;
    }
    
    /// 圧縮レベルを設定
    pub fn set_compression_level(&mut self, level: u32) {
        self.compression_level = level.min(9);
    }
    
    /// 最適化間隔を設定
    pub fn set_optimization_interval_secs(&mut self, interval_secs: u64) {
        self.optimization_interval_secs = interval_secs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{StorageManager, StorageKey, StorageValue};
    use tempfile::tempdir;
    
    #[test]
    fn test_record_access() {
        let temp_dir = tempdir().unwrap();
        let storage_manager = Arc::new(StorageManager::new(temp_dir.path()).unwrap());
        let metrics = Arc::new(MetricsCollector::new("test"));
        
        let optimizer = StorageOptimizer::new(storage_manager, metrics);
        
        // アクセスを記録
        let key = "test_key".to_string();
        
        for _ in 0..optimizer.hot_key_threshold {
            optimizer.record_access(&key);
        }
        
        // ホットキーになっていることを確認
        assert!(optimizer.hot_keys.lock().unwrap().contains(&key));
        assert_eq!(optimizer.get_hot_keys_count(), 1);
    }
    
    #[test]
    fn test_record_write() {
        let temp_dir = tempdir().unwrap();
        let storage_manager = Arc::new(StorageManager::new(temp_dir.path()).unwrap());
        let metrics = Arc::new(MetricsCollector::new("test"));
        
        let optimizer = StorageOptimizer::new(storage_manager, metrics);
        
        // 書き込みを記録
        let key = "test_key".to_string();
        optimizer.record_write(&key);
        
        // 最終書き込み時刻が記録されていることを確認
        assert!(optimizer.last_write.lock().unwrap().contains_key(&key));
    }
    
    #[test]
    fn test_compression() {
        // テストデータ
        let data = vec![0; 10000]; // 圧縮率の高いデータ
        
        // 圧縮
        let compressed = StorageOptimizer::compress_data(&data, 6);
        
        // 圧縮されていることを確認
        assert!(compressed.len() < data.len());
        
        // 解凍
        let decompressed = StorageOptimizer::decompress_data(&compressed).unwrap();
        
        // 元のデータと一致することを確認
        assert_eq!(decompressed, data);
    }
    
    #[tokio::test]
    async fn test_optimization_cycle() {
        let temp_dir = tempdir().unwrap();
        let storage_manager = Arc::new(StorageManager::new(temp_dir.path()).unwrap());
        let metrics = Arc::new(MetricsCollector::new("test"));
        
        let mut optimizer = StorageOptimizer::new(storage_manager.clone(), metrics);
        
        // 最適化間隔を短く設定
        optimizer.set_optimization_interval_secs(1);
        
        // テストデータを保存
        for i in 0..10 {
            let key = format!("key{}", i);
            let value = vec![i as u8; 2000]; // 圧縮閾値を超えるデータ
            
            storage_manager.put(&key, &value).unwrap();
            
            // アクセスと書き込みを記録
            optimizer.record_access(&key);
            optimizer.record_write(&key);
        }
        
        // 最適化を開始
        optimizer.start_optimization().await.unwrap();
        
        // 少し待機して最適化が実行されるのを待つ
        time::sleep(Duration::from_secs(2)).await;
        
        // 処理を停止
        optimizer.stop();
        
        // 圧縮されたキーがあることを確認
        assert!(optimizer.get_compressed_keys_count() > 0);
    }
}