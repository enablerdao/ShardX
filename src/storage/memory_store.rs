use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::error::Error;

/// メモリ内ストレージ（テスト用または小規模ノード用）
pub struct MemoryStorage {
    /// データストア（カラムファミリー -> キー -> 値）
    store: Arc<RwLock<HashMap<String, HashMap<String, Vec<u8>>>>>,
}

impl MemoryStorage {
    /// 新しいMemoryStorageを作成
    pub fn new() -> Self {
        let mut store = HashMap::new();
        
        // 基本的なカラムファミリーを初期化
        store.insert("transactions".to_string(), HashMap::new());
        store.insert("state".to_string(), HashMap::new());
        store.insert("metadata".to_string(), HashMap::new());
        
        Self {
            store: Arc::new(RwLock::new(store)),
        }
    }
    
    /// キーに対応する値を取得
    pub fn get(&self, cf_name: &str, key: &str) -> Result<Option<Vec<u8>>, Error> {
        let store = self.store.read().unwrap();
        
        if let Some(cf) = store.get(cf_name) {
            if let Some(value) = cf.get(key) {
                return Ok(Some(value.clone()));
            }
        }
        
        Ok(None)
    }
    
    /// キーに対応する値を設定
    pub fn put(&self, cf_name: &str, key: &str, value: &[u8]) -> Result<(), Error> {
        let mut store = self.store.write().unwrap();
        
        let cf = store.entry(cf_name.to_string())
            .or_insert_with(HashMap::new);
        
        cf.insert(key.to_string(), value.to_vec());
        
        Ok(())
    }
    
    /// キーに対応する値を削除
    pub fn delete(&self, cf_name: &str, key: &str) -> Result<(), Error> {
        let mut store = self.store.write().unwrap();
        
        if let Some(cf) = store.get_mut(cf_name) {
            cf.remove(key);
        }
        
        Ok(())
    }
    
    /// バッチ書き込み
    pub fn batch_write(&self, operations: Vec<(String, String, Vec<u8>)>) -> Result<(), Error> {
        let mut store = self.store.write().unwrap();
        
        for (cf_name, key, value) in operations {
            let cf = store.entry(cf_name)
                .or_insert_with(HashMap::new);
            
            cf.insert(key, value);
        }
        
        Ok(())
    }
    
    /// プレフィックスに一致するすべてのキーを取得
    pub fn get_by_prefix(&self, cf_name: &str, prefix: &str) -> Result<Vec<(String, Vec<u8>)>, Error> {
        let store = self.store.read().unwrap();
        
        let mut results = Vec::new();
        
        if let Some(cf) = store.get(cf_name) {
            for (key, value) in cf {
                if key.starts_with(prefix) {
                    results.push((key.clone(), value.clone()));
                }
            }
        }
        
        Ok(results)
    }
    
    /// カラムファミリーを作成
    pub fn create_column_family(&self, cf_name: &str) -> Result<(), Error> {
        let mut store = self.store.write().unwrap();
        
        if !store.contains_key(cf_name) {
            store.insert(cf_name.to_string(), HashMap::new());
        }
        
        Ok(())
    }
    
    /// カラムファミリーを削除
    pub fn drop_column_family(&self, cf_name: &str) -> Result<(), Error> {
        let mut store = self.store.write().unwrap();
        
        store.remove(cf_name);
        
        Ok(())
    }
    
    /// すべてのデータをクリア
    pub fn clear(&self) -> Result<(), Error> {
        let mut store = self.store.write().unwrap();
        
        for cf in store.values_mut() {
            cf.clear();
        }
        
        Ok(())
    }
    
    /// カラムファミリーのキー数を取得
    pub fn get_key_count(&self, cf_name: &str) -> Result<usize, Error> {
        let store = self.store.read().unwrap();
        
        if let Some(cf) = store.get(cf_name) {
            return Ok(cf.len());
        }
        
        Ok(0)
    }
    
    /// 全体のキー数を取得
    pub fn get_total_key_count(&self) -> Result<usize, Error> {
        let store = self.store.read().unwrap();
        
        let mut total = 0;
        for cf in store.values() {
            total += cf.len();
        }
        
        Ok(total)
    }
    
    /// メモリ使用量を概算（バイト単位）
    pub fn estimate_memory_usage(&self) -> Result<usize, Error> {
        let store = self.store.read().unwrap();
        
        let mut total = 0;
        
        // カラムファミリー名のサイズ
        for cf_name in store.keys() {
            total += cf_name.len();
        }
        
        // キーと値のサイズ
        for cf in store.values() {
            for (key, value) in cf {
                total += key.len() + value.len();
            }
        }
        
        // ハッシュマップのオーバーヘッド（概算）
        total += store.len() * 32; // カラムファミリーのハッシュマップ
        
        for cf in store.values() {
            total += cf.len() * 32; // キー・値のハッシュマップ
        }
        
        Ok(total)
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_operations() {
        let storage = MemoryStorage::new();
        
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
        let storage = MemoryStorage::new();
        
        // バッチ書き込み
        let operations = vec![
            ("transactions".to_string(), "key1".to_string(), b"value1".to_vec()),
            ("transactions".to_string(), "key2".to_string(), b"value2".to_vec()),
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
        let storage = MemoryStorage::new();
        
        // 値を設定
        storage.put("transactions", "prefix1:key1", b"value1").unwrap();
        storage.put("transactions", "prefix1:key2", b"value2").unwrap();
        storage.put("transactions", "prefix2:key3", b"value3").unwrap();
        
        // プレフィックスで検索
        let results = storage.get_by_prefix("transactions", "prefix1:").unwrap();
        
        // 結果を確認
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|(k, v)| k == "prefix1:key1" && v == b"value1"));
        assert!(results.iter().any(|(k, v)| k == "prefix1:key2" && v == b"value2"));
    }
    
    #[test]
    fn test_column_family_management() {
        let storage = MemoryStorage::new();
        
        // 新しいカラムファミリーを作成
        storage.create_column_family("new_cf").unwrap();
        
        // 値を設定
        storage.put("new_cf", "key1", b"value1").unwrap();
        
        // 値を取得
        let value = storage.get("new_cf", "key1").unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));
        
        // カラムファミリーを削除
        storage.drop_column_family("new_cf").unwrap();
        
        // 削除後に値を取得
        let value = storage.get("new_cf", "key1").unwrap();
        assert_eq!(value, None);
    }
    
    #[test]
    fn test_memory_usage() {
        let storage = MemoryStorage::new();
        
        // 初期メモリ使用量を取得
        let initial_usage = storage.estimate_memory_usage().unwrap();
        
        // データを追加
        for i in 0..100 {
            let key = format!("key{}", i);
            let value = vec![i as u8; 100]; // 100バイトの値
            storage.put("transactions", &key, &value).unwrap();
        }
        
        // 追加後のメモリ使用量を取得
        let final_usage = storage.estimate_memory_usage().unwrap();
        
        // メモリ使用量が増加していることを確認
        assert!(final_usage > initial_usage);
        
        // キー数を確認
        let key_count = storage.get_key_count("transactions").unwrap();
        assert_eq!(key_count, 100);
        
        // 全体のキー数を確認
        let total_key_count = storage.get_total_key_count().unwrap();
        assert_eq!(total_key_count, 100);
    }
}