use crate::error::Error;
use lru::LruCache;
use rocksdb::{ColumnFamilyDescriptor, Options, WriteBatch, DB};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// RocksDBを使用した最適化ストレージ
pub struct OptimizedStorage {
    /// RocksDBインスタンス
    db: DB,
    /// LRUキャッシュ
    cache: Arc<Mutex<LruCache<String, Vec<u8>>>>,
    /// キャッシュヒット数
    cache_hits: Arc<Mutex<u64>>,
    /// キャッシュミス数
    cache_misses: Arc<Mutex<u64>>,
}

impl OptimizedStorage {
    /// 新しいOptimizedStorageを作成
    pub fn new<P: AsRef<Path>>(path: P, cache_size: usize) -> Result<Self, Error> {
        // RocksDBのオプションを最適化
        let mut opts = Options::default();

        // 書き込み最適化
        opts.create_if_missing(true);
        opts.set_write_buffer_size(64 * 1024 * 1024); // 64MB
        opts.set_max_write_buffer_number(4);
        opts.set_min_write_buffer_number_to_merge(2);
        opts.set_level_zero_file_num_compaction_trigger(4);
        opts.set_level_zero_slowdown_writes_trigger(20);
        opts.set_level_zero_stop_writes_trigger(36);
        opts.set_max_background_jobs(4);

        // ブルームフィルタでルックアップを高速化
        opts.set_bloom_filter(10, false);

        // カラムファミリーを定義
        let cf_names = vec!["transactions", "state", "metadata"];
        let cf_descriptors: Vec<ColumnFamilyDescriptor> = cf_names
            .iter()
            .map(|name| {
                let mut cf_opts = Options::default();
                cf_opts.set_write_buffer_size(32 * 1024 * 1024); // 32MB
                cf_opts.set_bloom_filter(10, false);
                ColumnFamilyDescriptor::new(*name, cf_opts)
            })
            .collect();

        // DBを開く
        let db = if path.as_ref().exists() {
            // 既存のDBを開く
            DB::open_cf_descriptors(&opts, path, cf_descriptors)
                .map_err(|e| Error::StorageError(format!("Failed to open RocksDB: {}", e)))?
        } else {
            // 新しいDBを作成
            let db = DB::open(&opts, &path)
                .map_err(|e| Error::StorageError(format!("Failed to create RocksDB: {}", e)))?;

            // カラムファミリーを作成
            for name in cf_names {
                db.create_cf(name, &Options::default()).map_err(|e| {
                    Error::StorageError(format!("Failed to create column family: {}", e))
                })?;
            }

            db
        };

        // LRUキャッシュを作成（90%ヒット率を目標）
        let cache = Arc::new(Mutex::new(LruCache::new(cache_size)));

        Ok(Self {
            db,
            cache,
            cache_hits: Arc::new(Mutex::new(0)),
            cache_misses: Arc::new(Mutex::new(0)),
        })
    }

    /// キーに対応する値を取得
    pub fn get(&self, cf_name: &str, key: &str) -> Result<Option<Vec<u8>>, Error> {
        // まずキャッシュをチェック
        let cache_key = format!("{}:{}", cf_name, key);

        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(value) = cache.get(&cache_key) {
                // キャッシュヒット
                let mut hits = self.cache_hits.lock().unwrap();
                *hits += 1;
                return Ok(Some(value.clone()));
            }
        }

        // キャッシュミス
        {
            let mut misses = self.cache_misses.lock().unwrap();
            *misses += 1;
        }

        // DBから取得
        let cf = self
            .db
            .cf_handle(cf_name)
            .ok_or_else(|| Error::StorageError(format!("Column family not found: {}", cf_name)))?;

        let result = self
            .db
            .get_cf(cf, key.as_bytes())
            .map_err(|e| Error::StorageError(format!("Failed to get value: {}", e)))?;

        // 結果をキャッシュに保存
        if let Some(ref value) = result {
            let mut cache = self.cache.lock().unwrap();
            cache.put(cache_key, value.clone());
        }

        Ok(result)
    }

    /// キーに対応する値を設定
    pub fn put(&self, cf_name: &str, key: &str, value: &[u8]) -> Result<(), Error> {
        // DBに書き込み
        let cf = self
            .db
            .cf_handle(cf_name)
            .ok_or_else(|| Error::StorageError(format!("Column family not found: {}", cf_name)))?;

        self.db
            .put_cf(cf, key.as_bytes(), value)
            .map_err(|e| Error::StorageError(format!("Failed to put value: {}", e)))?;

        // キャッシュを更新
        let cache_key = format!("{}:{}", cf_name, key);
        let mut cache = self.cache.lock().unwrap();
        cache.put(cache_key, value.to_vec());

        Ok(())
    }

    /// キーに対応する値を削除
    pub fn delete(&self, cf_name: &str, key: &str) -> Result<(), Error> {
        // DBから削除
        let cf = self
            .db
            .cf_handle(cf_name)
            .ok_or_else(|| Error::StorageError(format!("Column family not found: {}", cf_name)))?;

        self.db
            .delete_cf(cf, key.as_bytes())
            .map_err(|e| Error::StorageError(format!("Failed to delete value: {}", e)))?;

        // キャッシュから削除
        let cache_key = format!("{}:{}", cf_name, key);
        let mut cache = self.cache.lock().unwrap();
        cache.pop(&cache_key);

        Ok(())
    }

    /// バッチ書き込み
    pub fn batch_write(&self, operations: Vec<(String, String, Vec<u8>)>) -> Result<(), Error> {
        // 一括書き込み用のバッチを作成
        let mut batch = WriteBatch::default();

        // キャッシュ更新用のエントリを収集
        let mut cache_updates = Vec::with_capacity(operations.len());

        for (cf_name, key, value) in operations {
            let cf = self.db.cf_handle(&cf_name).ok_or_else(|| {
                Error::StorageError(format!("Column family not found: {}", cf_name))
            })?;

            batch.put_cf(cf, key.as_bytes(), &value);

            // キャッシュ更新用にエントリを追加
            let cache_key = format!("{}:{}", cf_name, key);
            cache_updates.push((cache_key, value));
        }

        // バッチを書き込み
        self.db
            .write(batch)
            .map_err(|e| Error::StorageError(format!("Failed to write batch: {}", e)))?;

        // キャッシュを更新
        let mut cache = self.cache.lock().unwrap();
        for (key, value) in cache_updates {
            cache.put(key, value);
        }

        Ok(())
    }

    /// プレフィックスに一致するすべてのキーを取得
    pub fn get_by_prefix(
        &self,
        cf_name: &str,
        prefix: &str,
    ) -> Result<Vec<(String, Vec<u8>)>, Error> {
        let cf = self
            .db
            .cf_handle(cf_name)
            .ok_or_else(|| Error::StorageError(format!("Column family not found: {}", cf_name)))?;

        let mut results = Vec::new();
        let prefix_bytes = prefix.as_bytes();

        // イテレータを作成
        let iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::Start);

        // プレフィックスに一致するキーを収集
        for item in iter {
            let (key, value) =
                item.map_err(|e| Error::StorageError(format!("Failed to iterate: {}", e)))?;

            let key_str = String::from_utf8_lossy(&key).to_string();

            if key_str.starts_with(prefix) {
                results.push((key_str.to_string(), value.to_vec()));

                // キャッシュを更新
                let cache_key = format!("{}:{}", cf_name, key_str);
                let mut cache = self.cache.lock().unwrap();
                cache.put(cache_key, value.to_vec());
            }
        }

        Ok(results)
    }

    /// キャッシュヒット率を取得
    pub fn get_cache_hit_ratio(&self) -> f64 {
        let hits = *self.cache_hits.lock().unwrap();
        let misses = *self.cache_misses.lock().unwrap();

        if hits + misses == 0 {
            return 0.0;
        }

        hits as f64 / (hits + misses) as f64
    }

    /// キャッシュをクリア
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();

        let mut hits = self.cache_hits.lock().unwrap();
        *hits = 0;

        let mut misses = self.cache_misses.lock().unwrap();
        *misses = 0;
    }

    /// DBをフラッシュ
    pub fn flush(&self) -> Result<(), Error> {
        self.db
            .flush()
            .map_err(|e| Error::StorageError(format!("Failed to flush database: {}", e)))
    }

    /// DBをコンパクション
    pub fn compact(&self, cf_name: &str) -> Result<(), Error> {
        let cf = self
            .db
            .cf_handle(cf_name)
            .ok_or_else(|| Error::StorageError(format!("Column family not found: {}", cf_name)))?;

        self.db.compact_range_cf(cf, None::<&[u8]>, None::<&[u8]>);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_basic_operations() {
        // 一時ディレクトリを作成
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        // ストレージを作成
        let storage = OptimizedStorage::new(path, 1000).unwrap();

        // 値を設定
        storage.put("transactions", "key1", b"value1").unwrap();

        // 値を取得
        let value = storage.get("transactions", "key1").unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));

        // 値を削除
        storage.delete("transactions", "key1").unwrap();

        // 削除後に値を取得
        let value = storage.get("transactions", "key1").unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn test_batch_write() {
        // 一時ディレクトリを作成
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        // ストレージを作成
        let storage = OptimizedStorage::new(path, 1000).unwrap();

        // バッチ書き込み
        let operations = vec![
            (
                "transactions".to_string(),
                "key1".to_string(),
                b"value1".to_vec(),
            ),
            (
                "transactions".to_string(),
                "key2".to_string(),
                b"value2".to_vec(),
            ),
            ("state".to_string(), "key3".to_string(), b"value3".to_vec()),
        ];

        storage.batch_write(operations).unwrap();

        // 値を取得
        let value1 = storage.get("transactions", "key1").unwrap();
        let value2 = storage.get("transactions", "key2").unwrap();
        let value3 = storage.get("state", "key3").unwrap();

        assert_eq!(value1, Some(b"value1".to_vec()));
        assert_eq!(value2, Some(b"value2".to_vec()));
        assert_eq!(value3, Some(b"value3".to_vec()));
    }

    #[test]
    fn test_prefix_scan() {
        // 一時ディレクトリを作成
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        // ストレージを作成
        let storage = OptimizedStorage::new(path, 1000).unwrap();

        // 値を設定
        storage
            .put("transactions", "prefix1:key1", b"value1")
            .unwrap();
        storage
            .put("transactions", "prefix1:key2", b"value2")
            .unwrap();
        storage
            .put("transactions", "prefix2:key3", b"value3")
            .unwrap();

        // プレフィックスで検索
        let results = storage.get_by_prefix("transactions", "prefix1:").unwrap();

        // 結果を確認
        assert_eq!(results.len(), 2);
        assert!(results
            .iter()
            .any(|(k, v)| k == "prefix1:key1" && v == b"value1"));
        assert!(results
            .iter()
            .any(|(k, v)| k == "prefix1:key2" && v == b"value2"));
    }

    #[test]
    fn test_cache() {
        // 一時ディレクトリを作成
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        // ストレージを作成
        let storage = OptimizedStorage::new(path, 1000).unwrap();

        // 値を設定
        storage.put("transactions", "key1", b"value1").unwrap();

        // 値を2回取得
        let _ = storage.get("transactions", "key1").unwrap();
        let _ = storage.get("transactions", "key1").unwrap();

        // キャッシュヒット率を確認
        let hit_ratio = storage.get_cache_hit_ratio();
        assert_eq!(hit_ratio, 0.5); // 1回目はミス、2回目はヒット

        // キャッシュをクリア
        storage.clear_cache();

        // キャッシュヒット率をリセット後に確認
        let hit_ratio = storage.get_cache_hit_ratio();
        assert_eq!(hit_ratio, 0.0);
    }
}
