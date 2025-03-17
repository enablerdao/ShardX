use crate::async_utils::zero_copy::ZeroCopyBuffer;
use crate::error::Error;
use rocksdb::{BlockBasedOptions, Cache, ColumnFamilyDescriptor, Options, SliceTransform, DB};
use std::path::Path;
use std::sync::Arc;

/// 最適化されたRocksDBストレージ
///
/// パフォーマンスを最大化するために最適化されたRocksDBストレージ。
/// ブロックキャッシュ、プレフィックス検索、圧縮設定などを最適化。
pub struct OptimizedRocksDB {
    /// RocksDBインスタンス
    db: Arc<DB>,
    /// オプション
    options: Options,
    /// キャッシュ
    cache: Option<Cache>,
}

impl OptimizedRocksDB {
    /// 新しいOptimizedRocksDBを作成
    pub fn new<P: AsRef<Path>>(path: P, cache_size_mb: Option<usize>) -> Result<Self, Error> {
        // キャッシュを作成
        let cache_size = cache_size_mb.unwrap_or(512) * 1024 * 1024; // MBをバイトに変換
        let cache = Cache::new_lru_cache(cache_size);

        // ブロックベースのオプションを設定
        let mut block_opts = BlockBasedOptions::default();
        block_opts.set_block_cache(&cache);
        block_opts.set_block_size(16 * 1024); // 16KB
        block_opts.set_cache_index_and_filter_blocks(true);
        block_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);
        block_opts.set_format_version(5);

        // オプションを設定
        let mut options = Options::default();
        options.create_if_missing(true);
        options.set_max_open_files(10000);
        options.set_use_fsync(false);
        options.set_keep_log_file_num(10);
        options.set_max_total_wal_size(64 * 1024 * 1024); // 64MB
        options.set_write_buffer_size(64 * 1024 * 1024); // 64MB
        options.set_max_write_buffer_number(3);
        options.set_target_file_size_base(64 * 1024 * 1024); // 64MB
        options.set_level_zero_file_num_compaction_trigger(4);
        options.set_level_zero_slowdown_writes_trigger(20);
        options.set_level_zero_stop_writes_trigger(36);
        options.set_num_levels(7);
        options.set_max_bytes_for_level_base(512 * 1024 * 1024); // 512MB
        options.set_max_bytes_for_level_multiplier(10.0);
        options.set_block_based_table_factory(&block_opts);
        options.set_compression_type(rocksdb::DBCompressionType::Lz4);
        options.set_bottommost_compression_type(rocksdb::DBCompressionType::Zstd);

        // プレフィックス抽出器を設定
        options.set_prefix_extractor(SliceTransform::create_fixed_prefix(8));
        options.set_memtable_prefix_bloom_ratio(0.1);

        // パラレルコンパクションを有効化
        options.set_max_background_jobs(6);
        options.set_max_subcompactions(4);

        // WALリカバリーモードを設定
        options.set_wal_recovery_mode(rocksdb::DBRecoveryMode::PointInTimeRecovery);

        // カラムファミリーを定義
        let cf_names = ["default", "transactions", "accounts", "blocks", "state"];
        let mut cf_descriptors = Vec::new();

        for name in cf_names.iter() {
            let mut cf_opts = Options::default();
            cf_opts.set_block_based_table_factory(&block_opts);
            cf_opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
            cf_opts.set_bottommost_compression_type(rocksdb::DBCompressionType::Zstd);
            cf_opts.set_prefix_extractor(SliceTransform::create_fixed_prefix(8));
            cf_opts.set_memtable_prefix_bloom_ratio(0.1);

            cf_descriptors.push(ColumnFamilyDescriptor::new(name, cf_opts));
        }

        // DBを開く
        let db = DB::open_cf_descriptors(&options, path, cf_descriptors)
            .map_err(|e| Error::StorageError(format!("Failed to open RocksDB: {}", e)))?;

        Ok(Self {
            db: Arc::new(db),
            options,
            cache: Some(cache),
        })
    }

    /// キーに対応する値を取得
    pub fn get(&self, cf_name: &str, key: &[u8]) -> Result<Option<ZeroCopyBuffer>, Error> {
        let cf = self
            .db
            .cf_handle(cf_name)
            .ok_or_else(|| Error::StorageError(format!("Column family not found: {}", cf_name)))?;

        self.db
            .get_cf(cf, key)
            .map(|opt| opt.map(ZeroCopyBuffer::new))
            .map_err(|e| Error::StorageError(format!("Failed to get value: {}", e)))
    }

    /// キーに対応する値を設定
    pub fn put(&self, cf_name: &str, key: &[u8], value: &[u8]) -> Result<(), Error> {
        let cf = self
            .db
            .cf_handle(cf_name)
            .ok_or_else(|| Error::StorageError(format!("Column family not found: {}", cf_name)))?;

        self.db
            .put_cf(cf, key, value)
            .map_err(|e| Error::StorageError(format!("Failed to put value: {}", e)))
    }

    /// キーに対応する値を削除
    pub fn delete(&self, cf_name: &str, key: &[u8]) -> Result<(), Error> {
        let cf = self
            .db
            .cf_handle(cf_name)
            .ok_or_else(|| Error::StorageError(format!("Column family not found: {}", cf_name)))?;

        self.db
            .delete_cf(cf, key)
            .map_err(|e| Error::StorageError(format!("Failed to delete value: {}", e)))
    }

    /// バッチ書き込み
    pub fn write_batch(&self, batch: rocksdb::WriteBatch) -> Result<(), Error> {
        self.db
            .write(batch)
            .map_err(|e| Error::StorageError(format!("Failed to write batch: {}", e)))
    }

    /// プレフィックスに一致するキーを検索
    pub fn prefix_scan(
        &self,
        cf_name: &str,
        prefix: &[u8],
    ) -> Result<Vec<(Vec<u8>, ZeroCopyBuffer)>, Error> {
        let cf = self
            .db
            .cf_handle(cf_name)
            .ok_or_else(|| Error::StorageError(format!("Column family not found: {}", cf_name)))?;

        let mut iter = self.db.prefix_iterator_cf(cf, prefix);
        let mut results = Vec::new();

        iter.seek(prefix.to_vec());

        while iter.valid() {
            let key = iter
                .key()
                .ok_or_else(|| Error::StorageError("Failed to get key".to_string()))?;
            let value = iter
                .value()
                .ok_or_else(|| Error::StorageError("Failed to get value".to_string()))?;

            results.push((key.to_vec(), ZeroCopyBuffer::new(value.to_vec())));

            iter.next();
        }

        Ok(results)
    }

    /// 範囲検索
    pub fn range_scan(
        &self,
        cf_name: &str,
        start: &[u8],
        end: &[u8],
    ) -> Result<Vec<(Vec<u8>, ZeroCopyBuffer)>, Error> {
        let cf = self
            .db
            .cf_handle(cf_name)
            .ok_or_else(|| Error::StorageError(format!("Column family not found: {}", cf_name)))?;

        let mut iter = self.db.iterator_cf(
            cf,
            rocksdb::IteratorMode::From(start, rocksdb::Direction::Forward),
        );
        let mut results = Vec::new();

        while iter.valid() {
            let key = iter
                .key()
                .ok_or_else(|| Error::StorageError("Failed to get key".to_string()))?;

            if key > end {
                break;
            }

            let value = iter
                .value()
                .ok_or_else(|| Error::StorageError("Failed to get value".to_string()))?;

            results.push((key.to_vec(), ZeroCopyBuffer::new(value.to_vec())));

            iter.next();
        }

        Ok(results)
    }

    /// フラッシュ
    pub fn flush(&self) -> Result<(), Error> {
        self.db
            .flush()
            .map_err(|e| Error::StorageError(format!("Failed to flush: {}", e)))
    }

    /// コンパクション
    pub fn compact_range(
        &self,
        cf_name: &str,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
    ) -> Result<(), Error> {
        let cf = self
            .db
            .cf_handle(cf_name)
            .ok_or_else(|| Error::StorageError(format!("Column family not found: {}", cf_name)))?;

        self.db.compact_range_cf(cf, start, end);

        Ok(())
    }

    /// 統計情報を取得
    pub fn get_statistics(&self) -> String {
        self.db
            .property_value("rocksdb.stats")
            .unwrap_or_else(|| "Statistics not available".to_string())
    }

    /// 推定ファイルサイズを取得
    pub fn get_estimated_file_size(&self, cf_name: &str) -> Result<u64, Error> {
        let cf = self
            .db
            .cf_handle(cf_name)
            .ok_or_else(|| Error::StorageError(format!("Column family not found: {}", cf_name)))?;

        self.db
            .property_int_value_cf(cf, "rocksdb.estimate-live-data-size")
            .map_err(|e| Error::StorageError(format!("Failed to get estimated file size: {}", e)))?
            .ok_or_else(|| Error::StorageError("Property value not available".to_string()))
    }

    /// キャッシュ使用量を取得
    pub fn get_cache_usage(&self) -> Option<usize> {
        self.cache.as_ref().map(|cache| cache.get_usage())
    }

    /// キャッシュ容量を取得
    pub fn get_cache_capacity(&self) -> Option<usize> {
        self.cache.as_ref().map(|cache| cache.get_capacity())
    }
}

impl Clone for OptimizedRocksDB {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            options: self.options.clone(),
            cache: self.cache.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_rocksdb_basic_operations() {
        // 一時ディレクトリを作成
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path();

        // DBを開く
        let db = OptimizedRocksDB::new(db_path, Some(64)).unwrap();

        // 値を書き込み
        db.put("default", b"key1", b"value1").unwrap();
        db.put("default", b"key2", b"value2").unwrap();

        // 値を読み取り
        let value1 = db.get("default", b"key1").unwrap().unwrap();
        let value2 = db.get("default", b"key2").unwrap().unwrap();

        assert_eq!(value1.as_bytes(), b"value1");
        assert_eq!(value2.as_bytes(), b"value2");

        // 値を削除
        db.delete("default", b"key1").unwrap();

        // 削除された値を確認
        let value1 = db.get("default", b"key1").unwrap();
        assert!(value1.is_none());

        // 存在する値を確認
        let value2 = db.get("default", b"key2").unwrap().unwrap();
        assert_eq!(value2.as_bytes(), b"value2");
    }

    #[test]
    fn test_rocksdb_batch_write() {
        // 一時ディレクトリを作成
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path();

        // DBを開く
        let db = OptimizedRocksDB::new(db_path, Some(64)).unwrap();

        // バッチを作成
        let mut batch = rocksdb::WriteBatch::default();

        // デフォルトのカラムファミリーハンドルを取得
        let cf = db.db.cf_handle("default").unwrap();

        // バッチに操作を追加
        batch.put_cf(cf, b"key1", b"value1");
        batch.put_cf(cf, b"key2", b"value2");
        batch.put_cf(cf, b"key3", b"value3");

        // バッチを書き込み
        db.write_batch(batch).unwrap();

        // 値を確認
        let value1 = db.get("default", b"key1").unwrap().unwrap();
        let value2 = db.get("default", b"key2").unwrap().unwrap();
        let value3 = db.get("default", b"key3").unwrap().unwrap();

        assert_eq!(value1.as_bytes(), b"value1");
        assert_eq!(value2.as_bytes(), b"value2");
        assert_eq!(value3.as_bytes(), b"value3");
    }

    #[test]
    fn test_rocksdb_prefix_scan() {
        // 一時ディレクトリを作成
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path();

        // DBを開く
        let db = OptimizedRocksDB::new(db_path, Some(64)).unwrap();

        // プレフィックス付きのキーを書き込み
        db.put("default", b"prefix1_key1", b"value1").unwrap();
        db.put("default", b"prefix1_key2", b"value2").unwrap();
        db.put("default", b"prefix2_key1", b"value3").unwrap();
        db.put("default", b"prefix2_key2", b"value4").unwrap();

        // プレフィックス検索
        let results = db.prefix_scan("default", b"prefix1").unwrap();

        // 結果を確認
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, b"prefix1_key1");
        assert_eq!(results[0].1.as_bytes(), b"value1");
        assert_eq!(results[1].0, b"prefix1_key2");
        assert_eq!(results[1].1.as_bytes(), b"value2");
    }

    #[test]
    fn test_rocksdb_range_scan() {
        // 一時ディレクトリを作成
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path();

        // DBを開く
        let db = OptimizedRocksDB::new(db_path, Some(64)).unwrap();

        // キーを書き込み
        db.put("default", b"key1", b"value1").unwrap();
        db.put("default", b"key2", b"value2").unwrap();
        db.put("default", b"key3", b"value3").unwrap();
        db.put("default", b"key4", b"value4").unwrap();
        db.put("default", b"key5", b"value5").unwrap();

        // 範囲検索
        let results = db.range_scan("default", b"key2", b"key4").unwrap();

        // 結果を確認
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, b"key2");
        assert_eq!(results[0].1.as_bytes(), b"value2");
        assert_eq!(results[1].0, b"key3");
        assert_eq!(results[1].1.as_bytes(), b"value3");
        assert_eq!(results[2].0, b"key4");
        assert_eq!(results[2].1.as_bytes(), b"value4");
    }
}
