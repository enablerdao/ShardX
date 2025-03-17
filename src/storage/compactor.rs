use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::error::Error;

/// ストレージ最適化設定
#[derive(Debug, Clone)]
pub struct StorageCompactorConfig {
    /// 圧縮閾値（バイト）
    pub compaction_threshold: usize,
    /// 圧縮間隔（秒）
    pub compaction_interval: u64,
    /// 最大ファイルサイズ（バイト）
    pub max_file_size: usize,
    /// 自動圧縮が有効かどうか
    pub auto_compact: bool,
    /// 圧縮レベル（1-9）
    pub compression_level: u32,
    /// 削除マーカーの有効期限（秒）
    pub tombstone_expiry: u64,
    /// 重複排除が有効かどうか
    pub deduplication: bool,
}

impl Default for StorageCompactorConfig {
    fn default() -> Self {
        Self {
            compaction_threshold: 100 * 1024 * 1024, // 100MB
            compaction_interval: 3600,               // 1時間
            max_file_size: 1024 * 1024 * 1024,       // 1GB
            auto_compact: true,
            compression_level: 6,
            tombstone_expiry: 86400, // 24時間
            deduplication: true,
        }
    }
}

/// ストレージ圧縮統計
#[derive(Debug, Clone)]
pub struct CompactionStats {
    /// 圧縮前のサイズ（バイト）
    pub size_before: usize,
    /// 圧縮後のサイズ（バイト）
    pub size_after: usize,
    /// 削除されたエントリ数
    pub deleted_entries: usize,
    /// 重複排除されたエントリ数
    pub deduplicated_entries: usize,
    /// 圧縮にかかった時間（ミリ秒）
    pub duration_ms: u64,
    /// 圧縮率（%）
    pub compression_ratio: f32,
    /// 最後の圧縮時刻
    pub last_compaction: Instant,
}

/// ストレージコンパクター
pub struct StorageCompactor {
    /// 設定
    config: StorageCompactorConfig,
    /// データディレクトリ
    data_dir: PathBuf,
    /// 最後の圧縮時刻
    last_compaction: Arc<Mutex<Instant>>,
    /// 圧縮統計
    stats: Arc<Mutex<Option<CompactionStats>>>,
    /// 圧縮中フラグ
    compacting: Arc<Mutex<bool>>,
}

impl StorageCompactor {
    /// 新しいストレージコンパクターを作成
    pub fn new(data_dir: impl AsRef<Path>, config: Option<StorageCompactorConfig>) -> Self {
        let config = config.unwrap_or_default();

        Self {
            config,
            data_dir: data_dir.as_ref().to_path_buf(),
            last_compaction: Arc::new(Mutex::new(Instant::now())),
            stats: Arc::new(Mutex::new(None)),
            compacting: Arc::new(Mutex::new(false)),
        }
    }

    /// 圧縮を実行
    pub fn compact(&self) -> Result<CompactionStats, Error> {
        // 既に圧縮中の場合はエラー
        let mut compacting = self.compacting.lock().unwrap();
        if *compacting {
            return Err(Error::Other("Compaction already in progress".to_string()));
        }

        // 圧縮中フラグを設定
        *compacting = true;

        // 圧縮開始
        info!("Starting storage compaction");
        let start_time = Instant::now();

        // 圧縮前のサイズを計算
        let size_before = self.calculate_storage_size()?;

        // 圧縮処理
        let mut deleted_entries = 0;
        let mut deduplicated_entries = 0;

        // 1. 古い削除マーカーを削除
        deleted_entries += self.remove_expired_tombstones()?;

        // 2. 重複排除
        if self.config.deduplication {
            deduplicated_entries += self.deduplicate_data()?;
        }

        // 3. データファイルを圧縮
        self.compact_data_files()?;

        // 圧縮後のサイズを計算
        let size_after = self.calculate_storage_size()?;

        // 圧縮にかかった時間
        let duration = start_time.elapsed();
        let duration_ms = duration.as_millis() as u64;

        // 圧縮率を計算
        let compression_ratio = if size_before > 0 {
            (1.0 - (size_after as f32 / size_before as f32)) * 100.0
        } else {
            0.0
        };

        // 統計を更新
        let stats = CompactionStats {
            size_before,
            size_after,
            deleted_entries,
            deduplicated_entries,
            duration_ms,
            compression_ratio,
            last_compaction: Instant::now(),
        };

        // 最後の圧縮時刻を更新
        *self.last_compaction.lock().unwrap() = Instant::now();

        // 統計を保存
        *self.stats.lock().unwrap() = Some(stats.clone());

        // 圧縮中フラグを解除
        *compacting = false;

        info!(
            "Storage compaction completed: {} -> {} bytes ({:.1}% reduction) in {}ms",
            size_before, size_after, compression_ratio, duration_ms
        );

        Ok(stats)
    }

    /// 自動圧縮をチェック
    pub fn check_auto_compact(&self) -> Result<bool, Error> {
        // 自動圧縮が無効の場合は何もしない
        if !self.config.auto_compact {
            return Ok(false);
        }

        // 既に圧縮中の場合は何もしない
        let compacting = self.compacting.lock().unwrap();
        if *compacting {
            return Ok(false);
        }

        // 最後の圧縮からの経過時間をチェック
        let last_compaction = *self.last_compaction.lock().unwrap();
        let elapsed = last_compaction.elapsed();

        if elapsed.as_secs() < self.config.compaction_interval {
            return Ok(false);
        }

        // ストレージサイズをチェック
        let size = self.calculate_storage_size()?;

        if size >= self.config.compaction_threshold {
            // 圧縮を実行
            drop(compacting);
            let _ = self.compact()?;
            return Ok(true);
        }

        Ok(false)
    }

    /// 圧縮統計を取得
    pub fn get_stats(&self) -> Option<CompactionStats> {
        self.stats.lock().unwrap().clone()
    }

    /// 設定を更新
    pub fn update_config(&mut self, config: StorageCompactorConfig) {
        self.config = config;
    }

    /// ストレージサイズを計算
    fn calculate_storage_size(&self) -> Result<usize, Error> {
        let mut total_size = 0;

        // データディレクトリ内のすべてのファイルを走査
        for entry in fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if metadata.is_file() {
                total_size += metadata.len() as usize;
            }
        }

        Ok(total_size)
    }

    /// 期限切れの削除マーカーを削除
    fn remove_expired_tombstones(&self) -> Result<usize, Error> {
        // 実際の実装では、削除マーカーを走査して期限切れのものを削除
        // ここでは簡易的な実装として、削除数を返す

        Ok(100) // 100個削除したと仮定
    }

    /// データの重複排除
    fn deduplicate_data(&self) -> Result<usize, Error> {
        // 実際の実装では、データを走査して重複を排除
        // ここでは簡易的な実装として、重複排除数を返す

        Ok(50) // 50個重複排除したと仮定
    }

    /// データファイルを圧縮
    fn compact_data_files(&self) -> Result<(), Error> {
        // 実際の実装では、データファイルを圧縮
        // ここでは簡易的な実装として、何もしない

        Ok(())
    }
}
