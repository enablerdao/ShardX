use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::identity::did::{DID, DIDDocument, DIDMethod};

/// リゾルバオプション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolverOptions {
    /// 解決タイムアウト（ミリ秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    /// キャッシュを使用するフラグ
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_cache: Option<bool>,
    /// キャッシュの有効期限（秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_ttl_seconds: Option<u64>,
    /// 検証フラグ
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify: Option<bool>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

impl Default for ResolverOptions {
    fn default() -> Self {
        Self {
            timeout_ms: Some(30000),
            use_cache: Some(true),
            cache_ttl_seconds: Some(3600),
            verify: Some(true),
            additional_properties: HashMap::new(),
        }
    }
}

/// リゾルバ結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolverResult {
    /// DIDドキュメント
    #[serde(rename = "didDocument")]
    pub did_document: Option<DIDDocument>,
    /// メタデータ
    pub metadata: ResolverMetadata,
    /// エラー
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// リゾルバメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolverMetadata {
    /// 解決日時
    #[serde(rename = "resolvedAt")]
    pub resolved_at: DateTime<Utc>,
    /// DIDメソッド
    #[serde(rename = "didMethod")]
    pub did_method: String,
    /// ドライバID
    #[serde(rename = "driverId", skip_serializing_if = "Option::is_none")]
    pub driver_id: Option<String>,
    /// ドライババージョン
    #[serde(rename = "driverVersion", skip_serializing_if = "Option::is_none")]
    pub driver_version: Option<String>,
    /// キャッシュヒットフラグ
    #[serde(rename = "cacheHit", skip_serializing_if = "Option::is_none")]
    pub cache_hit: Option<bool>,
    /// キャッシュ有効期限
    #[serde(rename = "cacheExpires", skip_serializing_if = "Option::is_none")]
    pub cache_expires: Option<DateTime<Utc>>,
    /// 解決時間（ミリ秒）
    #[serde(rename = "resolutionTimeMs", skip_serializing_if = "Option::is_none")]
    pub resolution_time_ms: Option<u64>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// リゾルバ
pub trait Resolver {
    /// DIDを解決
    fn resolve(&self, did: &DID) -> Result<ResolverResult, Error>;
    
    /// DIDを解決（オプション付き）
    fn resolve_with_options(&self, did: &DID, options: ResolverOptions) -> Result<ResolverResult, Error>;
    
    /// DIDメソッドをサポートしているか確認
    fn supports_method(&self, method: &str) -> bool;
    
    /// サポートしているDIDメソッドのリストを取得
    fn supported_methods(&self) -> Vec<String>;
    
    /// DIDメソッドを追加
    fn add_method(&mut self, method: Box<dyn DIDMethod>);
    
    /// DIDメソッドを削除
    fn remove_method(&mut self, method_name: &str) -> bool;
}

/// ユニバーサルリゾルバ
pub struct UniversalResolver {
    /// DIDメソッド
    methods: HashMap<String, Box<dyn DIDMethod>>,
    /// キャッシュ
    cache: HashMap<String, (DIDDocument, DateTime<Utc>)>,
    /// デフォルトオプション
    default_options: ResolverOptions,
}

impl UniversalResolver {
    /// 新しいユニバーサルリゾルバを作成
    pub fn new() -> Self {
        Self {
            methods: HashMap::new(),
            cache: HashMap::new(),
            default_options: ResolverOptions::default(),
        }
    }
    
    /// デフォルトオプションを設定
    pub fn set_default_options(&mut self, options: ResolverOptions) {
        self.default_options = options;
    }
    
    /// キャッシュをクリア
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
    
    /// キャッシュから取得
    fn get_from_cache(&self, did: &DID, options: &ResolverOptions) -> Option<(DIDDocument, bool)> {
        let use_cache = options.use_cache.unwrap_or(self.default_options.use_cache.unwrap_or(true));
        
        if !use_cache {
            return None;
        }
        
        let cache_key = did.to_string();
        
        if let Some((document, expires)) = self.cache.get(&cache_key) {
            let now = Utc::now();
            
            if now < *expires {
                return Some((document.clone(), true));
            }
        }
        
        None
    }
    
    /// キャッシュに保存
    fn store_in_cache(&mut self, did: &DID, document: DIDDocument, options: &ResolverOptions) {
        let use_cache = options.use_cache.unwrap_or(self.default_options.use_cache.unwrap_or(true));
        
        if !use_cache {
            return;
        }
        
        let cache_key = did.to_string();
        let ttl = options.cache_ttl_seconds.unwrap_or(self.default_options.cache_ttl_seconds.unwrap_or(3600));
        let expires = Utc::now() + chrono::Duration::seconds(ttl as i64);
        
        self.cache.insert(cache_key, (document, expires));
    }
}

impl Resolver for UniversalResolver {
    fn resolve(&self, did: &DID) -> Result<ResolverResult, Error> {
        self.resolve_with_options(did, self.default_options.clone())
    }
    
    fn resolve_with_options(&self, did: &DID, options: ResolverOptions) -> Result<ResolverResult, Error> {
        let start_time = std::time::Instant::now();
        let method_name = did.method.clone();
        
        // キャッシュをチェック
        let (document, cache_hit) = if let Some((document, cache_hit)) = self.get_from_cache(did, &options) {
            (Some(document), Some(cache_hit))
        } else {
            // メソッドを取得
            let method = self.methods.get(&method_name).ok_or_else(|| {
                Error::NotFound(format!("DID method not found: {}", method_name))
            })?;
            
            // DIDを解決
            match method.resolve(did) {
                Ok(document) => {
                    // キャッシュに保存
                    let mut resolver = self.clone();
                    resolver.store_in_cache(did, document.clone(), &options);
                    
                    (Some(document), Some(false))
                },
                Err(e) => {
                    (None, None)
                }
            }
        };
        
        let resolution_time_ms = start_time.elapsed().as_millis() as u64;
        
        // メタデータを作成
        let metadata = ResolverMetadata {
            resolved_at: Utc::now(),
            did_method: method_name,
            driver_id: None,
            driver_version: None,
            cache_hit,
            cache_expires: None,
            resolution_time_ms: Some(resolution_time_ms),
            additional_properties: HashMap::new(),
        };
        
        // 結果を作成
        let result = ResolverResult {
            did_document: document,
            metadata,
            error: None,
        };
        
        Ok(result)
    }
    
    fn supports_method(&self, method: &str) -> bool {
        self.methods.contains_key(method)
    }
    
    fn supported_methods(&self) -> Vec<String> {
        self.methods.keys().cloned().collect()
    }
    
    fn add_method(&mut self, method: Box<dyn DIDMethod>) {
        let method_name = method.name().to_string();
        self.methods.insert(method_name, method);
    }
    
    fn remove_method(&mut self, method_name: &str) -> bool {
        self.methods.remove(method_name).is_some()
    }
}