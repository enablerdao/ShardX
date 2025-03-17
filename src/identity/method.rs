use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::crypto::{PublicKey, PrivateKey, KeyPair, hash};
use crate::identity::did::{DID, DIDDocument, DIDMethod, DIDVerificationMethod, DIDAuthentication, DIDService, DIDProof};

/// メソッドメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodMetadata {
    /// メソッド名
    pub name: String,
    /// メソッド説明
    pub description: String,
    /// メソッドバージョン
    pub version: String,
    /// メソッド作成者
    pub author: String,
    /// メソッドウェブサイト
    pub website: Option<String>,
    /// メソッドソースコード
    pub source_code: Option<String>,
    /// メソッドライセンス
    pub license: Option<String>,
    /// メソッド仕様
    pub specification: Option<String>,
    /// メソッドサポートされている機能
    pub supported_features: Vec<String>,
    /// メソッド制限
    pub limitations: Option<Vec<String>>,
    /// メソッド作成日時
    pub created_at: DateTime<Utc>,
    /// メソッド更新日時
    pub updated_at: DateTime<Utc>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// メソッドオプション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodOptions {
    /// キー生成アルゴリズム
    pub key_algorithm: String,
    /// キー長
    pub key_length: usize,
    /// キーの目的
    pub key_purpose: Vec<String>,
    /// キーの有効期限（秒）
    pub key_expiry_seconds: Option<u64>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

impl Default for MethodOptions {
    fn default() -> Self {
        Self {
            key_algorithm: "Ed25519".to_string(),
            key_length: 256,
            key_purpose: vec!["authentication".to_string()],
            key_expiry_seconds: None,
            additional_properties: HashMap::new(),
        }
    }
}

/// メソッド
pub trait Method {
    /// メソッド名を取得
    fn name(&self) -> &str;
    
    /// メソッドメタデータを取得
    fn metadata(&self) -> MethodMetadata;
    
    /// DIDを生成
    fn generate(&self, options: Option<MethodOptions>) -> Result<(DID, DIDDocument, KeyPair), Error>;
    
    /// DIDを検証
    fn validate(&self, did: &DID) -> Result<bool, Error>;
    
    /// DIDドキュメントを解決
    fn resolve(&self, did: &DID) -> Result<DIDDocument, Error>;
    
    /// DIDドキュメントを更新
    fn update(&self, did: &DID, document: &DIDDocument, key_pair: &KeyPair) -> Result<DIDDocument, Error>;
    
    /// DIDを無効化
    fn deactivate(&self, did: &DID, key_pair: &KeyPair) -> Result<(), Error>;
}

/// Key DIDメソッド
pub struct KeyMethod {
    /// メタデータ
    metadata: MethodMetadata,
    /// ストレージ
    storage: HashMap<String, DIDDocument>,
}

impl KeyMethod {
    /// 新しいKey DIDメソッドを作成
    pub fn new() -> Self {
        let metadata = MethodMetadata {
            name: "key".to_string(),
            description: "The did:key method".to_string(),
            version: "1.0.0".to_string(),
            author: "ShardX".to_string(),
            website: Some("https://shardx.org".to_string()),
            source_code: Some("https://github.com/enablerdao/ShardX".to_string()),
            license: Some("MIT".to_string()),
            specification: Some("https://w3c-ccg.github.io/did-method-key/".to_string()),
            supported_features: vec![
                "create".to_string(),
                "resolve".to_string(),
            ],
            limitations: Some(vec![
                "No update".to_string(),
                "No deactivate".to_string(),
            ]),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            additional_properties: HashMap::new(),
        };
        
        Self {
            metadata,
            storage: HashMap::new(),
        }
    }
    
    /// 公開鍵をマルチベースエンコード
    fn encode_public_key(&self, public_key: &PublicKey) -> String {
        // 実際の実装では、公開鍵をマルチベースエンコードする
        // ここでは簡易的な実装を提供
        format!("z{}", hex::encode(public_key.as_bytes()))
    }
    
    /// マルチベースエンコードされた公開鍵をデコード
    fn decode_public_key(&self, encoded: &str) -> Result<PublicKey, Error> {
        // 実際の実装では、マルチベースエンコードされた公開鍵をデコードする
        // ここでは簡易的な実装を提供
        if !encoded.starts_with('z') {
            return Err(Error::InvalidInput(format!("Invalid multibase encoding: {}", encoded)));
        }
        
        let hex_str = &encoded[1..];
        let bytes = hex::decode(hex_str)
            .map_err(|e| Error::InvalidInput(format!("Invalid hex encoding: {}", e)))?;
        
        PublicKey::from_bytes(&bytes)
            .map_err(|e| Error::InvalidInput(format!("Invalid public key: {}", e)))
    }
}

impl Method for KeyMethod {
    fn name(&self) -> &str {
        "key"
    }
    
    fn metadata(&self) -> MethodMetadata {
        self.metadata.clone()
    }
    
    fn generate(&self, options: Option<MethodOptions>) -> Result<(DID, DIDDocument, KeyPair), Error> {
        let options = options.unwrap_or_default();
        
        // キーペアを生成
        let key_pair = KeyPair::generate();
        
        // 公開鍵をエンコード
        let encoded_public_key = self.encode_public_key(&key_pair.public_key());
        
        // DIDを作成
        let did = DID::new("key".to_string(), encoded_public_key.clone());
        
        // DIDドキュメントを作成
        let mut document = DIDDocument::new(&did);
        
        // 検証メソッドを追加
        let verification_method = DIDVerificationMethod {
            id: format!("{}#keys-1", did.to_string()),
            type_: "Ed25519VerificationKey2018".to_string(),
            controller: did.to_string(),
            public_key_jwk: None,
            public_key_base58: None,
            public_key_multibase: Some(encoded_public_key),
            additional_properties: HashMap::new(),
        };
        
        document.add_verification_method(verification_method.clone());
        
        // 認証を追加
        let authentication = DIDAuthentication::Embedded(verification_method);
        document.add_authentication(authentication);
        
        // ドキュメントをストレージに保存
        let mut key_method = self.clone();
        key_method.storage.insert(did.to_string(), document.clone());
        
        Ok((did, document, key_pair))
    }
    
    fn validate(&self, did: &DID) -> Result<bool, Error> {
        // メソッドをチェック
        if did.method != "key" {
            return Err(Error::InvalidInput(format!("Invalid method: {}", did.method)));
        }
        
        // 識別子をチェック
        if !did.id.starts_with('z') {
            return Err(Error::InvalidInput(format!("Invalid identifier: {}", did.id)));
        }
        
        // 公開鍵をデコード
        let _ = self.decode_public_key(&did.id)?;
        
        Ok(true)
    }
    
    fn resolve(&self, did: &DID) -> Result<DIDDocument, Error> {
        // DIDを検証
        self.validate(did)?;
        
        // ストレージからドキュメントを取得
        if let Some(document) = self.storage.get(&did.to_string()) {
            return Ok(document.clone());
        }
        
        // ストレージにない場合は、DIDから動的に生成
        let public_key = self.decode_public_key(&did.id)?;
        
        // DIDドキュメントを作成
        let mut document = DIDDocument::new(did);
        
        // 検証メソッドを追加
        let verification_method = DIDVerificationMethod {
            id: format!("{}#keys-1", did.to_string()),
            type_: "Ed25519VerificationKey2018".to_string(),
            controller: did.to_string(),
            public_key_jwk: None,
            public_key_base58: None,
            public_key_multibase: Some(did.id.clone()),
            additional_properties: HashMap::new(),
        };
        
        document.add_verification_method(verification_method.clone());
        
        // 認証を追加
        let authentication = DIDAuthentication::Embedded(verification_method);
        document.add_authentication(authentication);
        
        Ok(document)
    }
    
    fn update(&self, did: &DID, document: &DIDDocument, key_pair: &KeyPair) -> Result<DIDDocument, Error> {
        // did:keyメソッドは更新をサポートしていない
        Err(Error::NotImplemented("did:key method does not support update".to_string()))
    }
    
    fn deactivate(&self, did: &DID, key_pair: &KeyPair) -> Result<(), Error> {
        // did:keyメソッドは無効化をサポートしていない
        Err(Error::NotImplemented("did:key method does not support deactivate".to_string()))
    }
}

/// Web DIDメソッド
pub struct WebMethod {
    /// メタデータ
    metadata: MethodMetadata,
    /// ストレージ
    storage: HashMap<String, DIDDocument>,
}

impl WebMethod {
    /// 新しいWeb DIDメソッドを作成
    pub fn new() -> Self {
        let metadata = MethodMetadata {
            name: "web".to_string(),
            description: "The did:web method".to_string(),
            version: "1.0.0".to_string(),
            author: "ShardX".to_string(),
            website: Some("https://shardx.org".to_string()),
            source_code: Some("https://github.com/enablerdao/ShardX".to_string()),
            license: Some("MIT".to_string()),
            specification: Some("https://w3c-ccg.github.io/did-method-web/".to_string()),
            supported_features: vec![
                "create".to_string(),
                "resolve".to_string(),
                "update".to_string(),
                "deactivate".to_string(),
            ],
            limitations: Some(vec![
                "Requires web server".to_string(),
            ]),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            additional_properties: HashMap::new(),
        };
        
        Self {
            metadata,
            storage: HashMap::new(),
        }
    }
    
    /// ドメインからDIDを作成
    fn create_did_from_domain(&self, domain: &str) -> DID {
        let id = domain.replace('.', ':');
        DID::new("web".to_string(), id)
    }
    
    /// DIDからドメインを取得
    fn get_domain_from_did(&self, did: &DID) -> String {
        did.id.replace(':', ".")
    }
}

impl Method for WebMethod {
    fn name(&self) -> &str {
        "web"
    }
    
    fn metadata(&self) -> MethodMetadata {
        self.metadata.clone()
    }
    
    fn generate(&self, options: Option<MethodOptions>) -> Result<(DID, DIDDocument, KeyPair), Error> {
        let options = options.unwrap_or_default();
        
        // オプションからドメインを取得
        let domain = options.additional_properties.get("domain")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::InvalidInput("Domain is required".to_string()))?;
        
        // キーペアを生成
        let key_pair = KeyPair::generate();
        
        // DIDを作成
        let did = self.create_did_from_domain(domain);
        
        // DIDドキュメントを作成
        let mut document = DIDDocument::new(&did);
        
        // 検証メソッドを追加
        let verification_method = DIDVerificationMethod {
            id: format!("{}#keys-1", did.to_string()),
            type_: "Ed25519VerificationKey2018".to_string(),
            controller: did.to_string(),
            public_key_jwk: None,
            public_key_base58: Some(base58::encode(&key_pair.public_key().as_bytes())),
            public_key_multibase: None,
            additional_properties: HashMap::new(),
        };
        
        document.add_verification_method(verification_method.clone());
        
        // 認証を追加
        let authentication = DIDAuthentication::Embedded(verification_method);
        document.add_authentication(authentication);
        
        // サービスを追加
        let service = DIDService {
            id: format!("{}#website", did.to_string()),
            type_: "Website".to_string(),
            service_endpoint: crate::identity::did::ServiceEndpoint::Uri(format!("https://{}", domain)),
            additional_properties: HashMap::new(),
        };
        
        document.add_service(service);
        
        // ドキュメントをストレージに保存
        let mut web_method = self.clone();
        web_method.storage.insert(did.to_string(), document.clone());
        
        Ok((did, document, key_pair))
    }
    
    fn validate(&self, did: &DID) -> Result<bool, Error> {
        // メソッドをチェック
        if did.method != "web" {
            return Err(Error::InvalidInput(format!("Invalid method: {}", did.method)));
        }
        
        // 識別子をチェック
        if did.id.is_empty() {
            return Err(Error::InvalidInput("Empty identifier".to_string()));
        }
        
        Ok(true)
    }
    
    fn resolve(&self, did: &DID) -> Result<DIDDocument, Error> {
        // DIDを検証
        self.validate(did)?;
        
        // ストレージからドキュメントを取得
        if let Some(document) = self.storage.get(&did.to_string()) {
            return Ok(document.clone());
        }
        
        // ストレージにない場合は、Webからドキュメントを取得
        let domain = self.get_domain_from_did(did);
        let url = format!("https://{}/.well-known/did.json", domain);
        
        // 実際の実装では、HTTPリクエストを送信してドキュメントを取得する
        // ここでは簡易的な実装を提供
        Err(Error::NotFound(format!("DID document not found: {}", did.to_string())))
    }
    
    fn update(&self, did: &DID, document: &DIDDocument, key_pair: &KeyPair) -> Result<DIDDocument, Error> {
        // DIDを検証
        self.validate(did)?;
        
        // ドキュメントを更新
        let mut updated_document = document.clone();
        updated_document.update_timestamp();
        
        // ドキュメントをストレージに保存
        let mut web_method = self.clone();
        web_method.storage.insert(did.to_string(), updated_document.clone());
        
        Ok(updated_document)
    }
    
    fn deactivate(&self, did: &DID, key_pair: &KeyPair) -> Result<(), Error> {
        // DIDを検証
        self.validate(did)?;
        
        // ドキュメントをストレージから削除
        let mut web_method = self.clone();
        web_method.storage.remove(&did.to_string());
        
        Ok(())
    }
}