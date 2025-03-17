use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};

/// ストレージキー
pub type StorageKey = Vec<u8>;

/// ストレージ値
pub type StorageValue = Vec<u8>;

/// ストレージエラー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageError {
    /// キーが見つからない
    KeyNotFound(String),
    /// 無効なキー
    InvalidKey(String),
    /// 無効な値
    InvalidValue(String),
    /// ストレージ容量超過
    StorageCapacityExceeded,
    /// コントラクトが見つからない
    ContractNotFound(String),
    /// 無効なコントラクト
    InvalidContract(String),
    /// コントラクトストレージが見つからない
    ContractStorageNotFound(String),
    /// データベースエラー
    DatabaseError(String),
    /// シリアライズエラー
    SerializationError(String),
    /// デシリアライズエラー
    DeserializationError(String),
    /// I/Oエラー
    IOError(String),
    /// 内部エラー
    InternalError(String),
    /// カスタムエラー
    Custom(String),
}

impl From<StorageError> for crate::error::Error {
    fn from(error: StorageError) -> Self {
        match error {
            StorageError::KeyNotFound(key) => crate::error::Error::NotFound(format!("Key not found: {}", key)),
            StorageError::InvalidKey(key) => crate::error::Error::InvalidInput(format!("Invalid key: {}", key)),
            StorageError::InvalidValue(value) => crate::error::Error::InvalidInput(format!("Invalid value: {}", value)),
            StorageError::StorageCapacityExceeded => crate::error::Error::ResourceExhausted("Storage capacity exceeded".to_string()),
            StorageError::ContractNotFound(address) => crate::error::Error::NotFound(format!("Contract not found: {}", address)),
            StorageError::InvalidContract(address) => crate::error::Error::InvalidInput(format!("Invalid contract: {}", address)),
            StorageError::ContractStorageNotFound(address) => crate::error::Error::NotFound(format!("Contract storage not found: {}", address)),
            StorageError::DatabaseError(msg) => crate::error::Error::Internal(format!("Database error: {}", msg)),
            StorageError::SerializationError(msg) => crate::error::Error::SerializationError(msg),
            StorageError::DeserializationError(msg) => crate::error::Error::DeserializationError(msg),
            StorageError::IOError(msg) => crate::error::Error::IOError(msg),
            StorageError::InternalError(msg) => crate::error::Error::Internal(msg),
            StorageError::Custom(msg) => crate::error::Error::Custom(msg),
        }
    }
}

impl From<crate::error::Error> for StorageError {
    fn from(error: crate::error::Error) -> Self {
        match error {
            crate::error::Error::NotFound(msg) => StorageError::KeyNotFound(msg),
            crate::error::Error::InvalidInput(msg) => StorageError::InvalidKey(msg),
            crate::error::Error::ResourceExhausted(msg) => StorageError::StorageCapacityExceeded,
            crate::error::Error::SerializationError(msg) => StorageError::SerializationError(msg),
            crate::error::Error::DeserializationError(msg) => StorageError::DeserializationError(msg),
            crate::error::Error::IOError(msg) => StorageError::IOError(msg),
            crate::error::Error::Internal(msg) => StorageError::InternalError(msg),
            crate::error::Error::Custom(msg) => StorageError::Custom(msg),
            _ => StorageError::Custom(format!("{:?}", error)),
        }
    }
}

/// コントラクトストレージ
pub trait ContractStorage {
    /// キーに対応する値を取得
    fn get(&self, key: &StorageKey) -> Result<Option<StorageValue>, StorageError>;
    
    /// キーに対応する値を設定
    fn set(&mut self, key: StorageKey, value: StorageValue) -> Result<(), StorageError>;
    
    /// キーに対応する値を削除
    fn delete(&mut self, key: &StorageKey) -> Result<(), StorageError>;
    
    /// キーが存在するか確認
    fn has(&self, key: &StorageKey) -> Result<bool, StorageError>;
    
    /// コントラクトが存在するか確認
    fn has_contract(&self, address: &str) -> Result<bool, StorageError>;
    
    /// コントラクトを取得
    fn get_contract(&self, address: &str) -> Result<Option<Vec<u8>>, StorageError>;
    
    /// コントラクトを設定
    fn set_contract(&mut self, address: &str, code: Vec<u8>) -> Result<(), StorageError>;
    
    /// コントラクトを削除
    fn delete_contract(&mut self, address: &str) -> Result<(), StorageError>;
    
    /// コントラクトストレージから値を取得
    fn get_contract_storage(&self, address: &str, key: &StorageKey) -> Result<Option<StorageValue>, StorageError>;
    
    /// コントラクトストレージに値を設定
    fn set_contract_storage(&mut self, address: &str, key: StorageKey, value: StorageValue) -> Result<(), StorageError>;
    
    /// コントラクトストレージから値を削除
    fn delete_contract_storage(&mut self, address: &str, key: &StorageKey) -> Result<(), StorageError>;
    
    /// コントラクトストレージにキーが存在するか確認
    fn has_contract_storage(&self, address: &str, key: &StorageKey) -> Result<bool, StorageError>;
    
    /// コントラクトストレージのキーを取得
    fn get_contract_storage_keys(&self, address: &str) -> Result<Vec<StorageKey>, StorageError>;
    
    /// コントラクトストレージをクリア
    fn clear_contract_storage(&mut self, address: &str) -> Result<(), StorageError>;
}

/// インメモリコントラクトストレージ
pub struct InMemoryContractStorage {
    /// グローバルストレージ
    global_storage: HashMap<Vec<u8>, Vec<u8>>,
    /// コントラクトコード
    contract_code: HashMap<String, Vec<u8>>,
    /// コントラクトストレージ
    contract_storage: HashMap<String, HashMap<Vec<u8>, Vec<u8>>>,
}

impl InMemoryContractStorage {
    /// 新しいインメモリコントラクトストレージを作成
    pub fn new() -> Self {
        Self {
            global_storage: HashMap::new(),
            contract_code: HashMap::new(),
            contract_storage: HashMap::new(),
        }
    }
}

impl ContractStorage for InMemoryContractStorage {
    fn get(&self, key: &StorageKey) -> Result<Option<StorageValue>, StorageError> {
        Ok(self.global_storage.get(key).cloned())
    }
    
    fn set(&mut self, key: StorageKey, value: StorageValue) -> Result<(), StorageError> {
        self.global_storage.insert(key, value);
        Ok(())
    }
    
    fn delete(&mut self, key: &StorageKey) -> Result<(), StorageError> {
        self.global_storage.remove(key);
        Ok(())
    }
    
    fn has(&self, key: &StorageKey) -> Result<bool, StorageError> {
        Ok(self.global_storage.contains_key(key))
    }
    
    fn has_contract(&self, address: &str) -> Result<bool, StorageError> {
        Ok(self.contract_code.contains_key(address))
    }
    
    fn get_contract(&self, address: &str) -> Result<Option<Vec<u8>>, StorageError> {
        Ok(self.contract_code.get(address).cloned())
    }
    
    fn set_contract(&mut self, address: &str, code: Vec<u8>) -> Result<(), StorageError> {
        self.contract_code.insert(address.to_string(), code);
        Ok(())
    }
    
    fn delete_contract(&mut self, address: &str) -> Result<(), StorageError> {
        self.contract_code.remove(address);
        self.contract_storage.remove(address);
        Ok(())
    }
    
    fn get_contract_storage(&self, address: &str, key: &StorageKey) -> Result<Option<StorageValue>, StorageError> {
        if let Some(storage) = self.contract_storage.get(address) {
            Ok(storage.get(key).cloned())
        } else {
            Ok(None)
        }
    }
    
    fn set_contract_storage(&mut self, address: &str, key: StorageKey, value: StorageValue) -> Result<(), StorageError> {
        let storage = self.contract_storage.entry(address.to_string()).or_insert_with(HashMap::new);
        storage.insert(key, value);
        Ok(())
    }
    
    fn delete_contract_storage(&mut self, address: &str, key: &StorageKey) -> Result<(), StorageError> {
        if let Some(storage) = self.contract_storage.get_mut(address) {
            storage.remove(key);
        }
        Ok(())
    }
    
    fn has_contract_storage(&self, address: &str, key: &StorageKey) -> Result<bool, StorageError> {
        if let Some(storage) = self.contract_storage.get(address) {
            Ok(storage.contains_key(key))
        } else {
            Ok(false)
        }
    }
    
    fn get_contract_storage_keys(&self, address: &str) -> Result<Vec<StorageKey>, StorageError> {
        if let Some(storage) = self.contract_storage.get(address) {
            Ok(storage.keys().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }
    
    fn clear_contract_storage(&mut self, address: &str) -> Result<(), StorageError> {
        if let Some(storage) = self.contract_storage.get_mut(address) {
            storage.clear();
        }
        Ok(())
    }
}

/// 永続化コントラクトストレージ
pub struct PersistentContractStorage {
    /// ストレージパス
    path: String,
    /// インメモリキャッシュ
    cache: InMemoryContractStorage,
}

impl PersistentContractStorage {
    /// 新しい永続化コントラクトストレージを作成
    pub fn new(path: String) -> Self {
        Self {
            path,
            cache: InMemoryContractStorage::new(),
        }
    }
    
    /// ストレージを同期
    pub fn sync(&self) -> Result<(), StorageError> {
        // 実際の実装では、キャッシュをディスクに書き込む
        Ok(())
    }
}

impl ContractStorage for PersistentContractStorage {
    fn get(&self, key: &StorageKey) -> Result<Option<StorageValue>, StorageError> {
        self.cache.get(key)
    }
    
    fn set(&mut self, key: StorageKey, value: StorageValue) -> Result<(), StorageError> {
        self.cache.set(key, value)?;
        self.sync()?;
        Ok(())
    }
    
    fn delete(&mut self, key: &StorageKey) -> Result<(), StorageError> {
        self.cache.delete(key)?;
        self.sync()?;
        Ok(())
    }
    
    fn has(&self, key: &StorageKey) -> Result<bool, StorageError> {
        self.cache.has(key)
    }
    
    fn has_contract(&self, address: &str) -> Result<bool, StorageError> {
        self.cache.has_contract(address)
    }
    
    fn get_contract(&self, address: &str) -> Result<Option<Vec<u8>>, StorageError> {
        self.cache.get_contract(address)
    }
    
    fn set_contract(&mut self, address: &str, code: Vec<u8>) -> Result<(), StorageError> {
        self.cache.set_contract(address, code)?;
        self.sync()?;
        Ok(())
    }
    
    fn delete_contract(&mut self, address: &str) -> Result<(), StorageError> {
        self.cache.delete_contract(address)?;
        self.sync()?;
        Ok(())
    }
    
    fn get_contract_storage(&self, address: &str, key: &StorageKey) -> Result<Option<StorageValue>, StorageError> {
        self.cache.get_contract_storage(address, key)
    }
    
    fn set_contract_storage(&mut self, address: &str, key: StorageKey, value: StorageValue) -> Result<(), StorageError> {
        self.cache.set_contract_storage(address, key, value)?;
        self.sync()?;
        Ok(())
    }
    
    fn delete_contract_storage(&mut self, address: &str, key: &StorageKey) -> Result<(), StorageError> {
        self.cache.delete_contract_storage(address, key)?;
        self.sync()?;
        Ok(())
    }
    
    fn has_contract_storage(&self, address: &str, key: &StorageKey) -> Result<bool, StorageError> {
        self.cache.has_contract_storage(address, key)
    }
    
    fn get_contract_storage_keys(&self, address: &str) -> Result<Vec<StorageKey>, StorageError> {
        self.cache.get_contract_storage_keys(address)
    }
    
    fn clear_contract_storage(&mut self, address: &str) -> Result<(), StorageError> {
        self.cache.clear_contract_storage(address)?;
        self.sync()?;
        Ok(())
    }
}