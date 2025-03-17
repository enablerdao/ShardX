use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::crypto::{hash, PublicKey, Signature};
use crate::error::Error;

/// 分散型識別子（DID）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DID {
    /// DIDスキーム（常に "did"）
    pub scheme: String,
    /// DIDメソッド
    pub method: String,
    /// DID識別子
    pub id: String,
    /// DIDパス
    pub path: Option<String>,
    /// DIDクエリ
    pub query: Option<String>,
    /// DIDフラグメント
    pub fragment: Option<String>,
}

impl DID {
    /// 新しいDIDを作成
    pub fn new(method: String, id: String) -> Self {
        Self {
            scheme: "did".to_string(),
            method,
            id,
            path: None,
            query: None,
            fragment: None,
        }
    }

    /// 文字列からDIDを解析
    pub fn parse(did_str: &str) -> Result<Self, Error> {
        // DIDスキームをチェック
        if !did_str.starts_with("did:") {
            return Err(Error::InvalidInput(format!(
                "Invalid DID scheme: {}",
                did_str
            )));
        }

        // DIDの各部分を解析
        let parts: Vec<&str> = did_str.splitn(4, ':').collect();
        if parts.len() < 3 {
            return Err(Error::InvalidInput(format!(
                "Invalid DID format: {}",
                did_str
            )));
        }

        let scheme = parts[0].to_string();
        let method = parts[1].to_string();
        let mut id = parts[2].to_string();

        let mut path = None;
        let mut query = None;
        let mut fragment = None;

        // パス、クエリ、フラグメントを解析
        if parts.len() > 3 {
            let rest = parts[3];

            // フラグメントを解析
            let fragment_parts: Vec<&str> = rest.split('#').collect();
            if fragment_parts.len() > 1 {
                fragment = Some(fragment_parts[1].to_string());
            }

            // クエリを解析
            let query_parts: Vec<&str> = fragment_parts[0].split('?').collect();
            if query_parts.len() > 1 {
                query = Some(query_parts[1].to_string());
            }

            // パスを解析
            let path_parts: Vec<&str> = query_parts[0].split('/').collect();
            if path_parts.len() > 1 {
                path = Some(format!("/{}", path_parts[1..].join("/")));
            }
        }

        Ok(Self {
            scheme,
            method,
            id,
            path,
            query,
            fragment,
        })
    }

    /// DIDを文字列に変換
    pub fn to_string(&self) -> String {
        let mut result = format!("{}:{}:{}", self.scheme, self.method, self.id);

        if let Some(path) = &self.path {
            result.push_str(path);
        }

        if let Some(query) = &self.query {
            result.push_str(&format!("?{}", query));
        }

        if let Some(fragment) = &self.fragment {
            result.push_str(&format!("#{}", fragment));
        }

        result
    }

    /// DIDのURLを作成
    pub fn to_url(
        &self,
        path: Option<&str>,
        query: Option<&str>,
        fragment: Option<&str>,
    ) -> String {
        let mut result = format!("{}:{}:{}", self.scheme, self.method, self.id);

        if let Some(path) = path {
            result.push_str(&format!("/{}", path));
        } else if let Some(path) = &self.path {
            result.push_str(path);
        }

        if let Some(query) = query {
            result.push_str(&format!("?{}", query));
        } else if let Some(query) = &self.query {
            result.push_str(&format!("?{}", query));
        }

        if let Some(fragment) = fragment {
            result.push_str(&format!("#{}", fragment));
        } else if let Some(fragment) = &self.fragment {
            result.push_str(&format!("#{}", fragment));
        }

        result
    }

    /// DIDのメソッド固有識別子を取得
    pub fn method_specific_id(&self) -> &str {
        &self.id
    }

    /// DIDのメソッドを取得
    pub fn method_name(&self) -> &str {
        &self.method
    }

    /// DIDのフラグメントを設定
    pub fn with_fragment(mut self, fragment: String) -> Self {
        self.fragment = Some(fragment);
        self
    }

    /// DIDのパスを設定
    pub fn with_path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }

    /// DIDのクエリを設定
    pub fn with_query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }
}

impl ToString for DID {
    fn to_string(&self) -> String {
        self.to_string()
    }
}

/// DIDメソッド
pub trait DIDMethod {
    /// メソッド名を取得
    fn name(&self) -> &str;

    /// DIDを生成
    fn generate(&self, options: Option<HashMap<String, String>>) -> Result<DID, Error>;

    /// DIDを検証
    fn validate(&self, did: &DID) -> Result<bool, Error>;

    /// DIDドキュメントを解決
    fn resolve(&self, did: &DID) -> Result<DIDDocument, Error>;

    /// DIDドキュメントを更新
    fn update(&self, did: &DID, document: &DIDDocument) -> Result<(), Error>;

    /// DIDを無効化
    fn deactivate(&self, did: &DID) -> Result<(), Error>;
}

/// DIDリゾルバ
pub trait DIDResolver {
    /// DIDドキュメントを解決
    fn resolve(&self, did: &DID) -> Result<DIDDocument, Error>;

    /// DIDドキュメントを解決（オプション付き）
    fn resolve_with_options(
        &self,
        did: &DID,
        options: HashMap<String, String>,
    ) -> Result<DIDDocument, Error>;

    /// DIDドキュメントのメタデータを解決
    fn resolve_metadata(&self, did: &DID) -> Result<HashMap<String, String>, Error>;

    /// DIDメソッドをサポートしているか確認
    fn supports_method(&self, method: &str) -> bool;

    /// サポートしているDIDメソッドのリストを取得
    fn supported_methods(&self) -> Vec<String>;
}

/// DIDドキュメント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIDDocument {
    /// コンテキスト
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    /// DID
    pub id: String,
    /// コントローラー
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controller: Option<Vec<String>>,
    /// 検証メソッド
    #[serde(rename = "verificationMethod", skip_serializing_if = "Option::is_none")]
    pub verification_method: Option<Vec<DIDVerificationMethod>>,
    /// 認証
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<Vec<DIDAuthentication>>,
    /// アサーション
    #[serde(rename = "assertionMethod", skip_serializing_if = "Option::is_none")]
    pub assertion_method: Option<Vec<String>>,
    /// キー合意
    #[serde(rename = "keyAgreement", skip_serializing_if = "Option::is_none")]
    pub key_agreement: Option<Vec<String>>,
    /// 機能呼び出し
    #[serde(
        rename = "capabilityInvocation",
        skip_serializing_if = "Option::is_none"
    )]
    pub capability_invocation: Option<Vec<String>>,
    /// 機能委任
    #[serde(
        rename = "capabilityDelegation",
        skip_serializing_if = "Option::is_none"
    )]
    pub capability_delegation: Option<Vec<String>>,
    /// サービス
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<Vec<DIDService>>,
    /// 作成日時
    #[serde(rename = "created", skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,
    /// 更新日時
    #[serde(rename = "updated", skip_serializing_if = "Option::is_none")]
    pub updated: Option<DateTime<Utc>>,
    /// 証明
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<Vec<DIDProof>>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

impl DIDDocument {
    /// 新しいDIDドキュメントを作成
    pub fn new(did: &DID) -> Self {
        Self {
            context: vec!["https://www.w3.org/ns/did/v1".to_string()],
            id: did.to_string(),
            controller: None,
            verification_method: None,
            authentication: None,
            assertion_method: None,
            key_agreement: None,
            capability_invocation: None,
            capability_delegation: None,
            service: None,
            created: Some(Utc::now()),
            updated: Some(Utc::now()),
            proof: None,
            additional_properties: HashMap::new(),
        }
    }

    /// 検証メソッドを追加
    pub fn add_verification_method(&mut self, method: DIDVerificationMethod) {
        if self.verification_method.is_none() {
            self.verification_method = Some(Vec::new());
        }

        if let Some(methods) = &mut self.verification_method {
            methods.push(method);
        }
    }

    /// 認証を追加
    pub fn add_authentication(&mut self, auth: DIDAuthentication) {
        if self.authentication.is_none() {
            self.authentication = Some(Vec::new());
        }

        if let Some(auths) = &mut self.authentication {
            auths.push(auth);
        }
    }

    /// サービスを追加
    pub fn add_service(&mut self, service: DIDService) {
        if self.service.is_none() {
            self.service = Some(Vec::new());
        }

        if let Some(services) = &mut self.service {
            services.push(service);
        }
    }

    /// 証明を追加
    pub fn add_proof(&mut self, proof: DIDProof) {
        if self.proof.is_none() {
            self.proof = Some(Vec::new());
        }

        if let Some(proofs) = &mut self.proof {
            proofs.push(proof);
        }
    }

    /// 検証メソッドを取得
    pub fn get_verification_method(&self, id: &str) -> Option<&DIDVerificationMethod> {
        if let Some(methods) = &self.verification_method {
            methods.iter().find(|m| m.id == id)
        } else {
            None
        }
    }

    /// 認証を取得
    pub fn get_authentication(&self, id: &str) -> Option<&DIDAuthentication> {
        if let Some(auths) = &self.authentication {
            auths.iter().find(|a| match a {
                DIDAuthentication::Reference(ref_id) => ref_id == id,
                DIDAuthentication::Embedded(method) => method.id == id,
            })
        } else {
            None
        }
    }

    /// サービスを取得
    pub fn get_service(&self, id: &str) -> Option<&DIDService> {
        if let Some(services) = &self.service {
            services.iter().find(|s| s.id == id)
        } else {
            None
        }
    }

    /// 証明を取得
    pub fn get_proof(&self, type_: &str) -> Option<&DIDProof> {
        if let Some(proofs) = &self.proof {
            proofs.iter().find(|p| p.type_ == type_)
        } else {
            None
        }
    }

    /// 更新日時を更新
    pub fn update_timestamp(&mut self) {
        self.updated = Some(Utc::now());
    }
}

/// DID検証メソッド
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIDVerificationMethod {
    /// ID
    pub id: String,
    /// タイプ
    #[serde(rename = "type")]
    pub type_: String,
    /// コントローラー
    pub controller: String,
    /// 公開鍵（JWK）
    #[serde(rename = "publicKeyJwk", skip_serializing_if = "Option::is_none")]
    pub public_key_jwk: Option<HashMap<String, serde_json::Value>>,
    /// 公開鍵（Base58）
    #[serde(rename = "publicKeyBase58", skip_serializing_if = "Option::is_none")]
    pub public_key_base58: Option<String>,
    /// 公開鍵（Multibase）
    #[serde(rename = "publicKeyMultibase", skip_serializing_if = "Option::is_none")]
    pub public_key_multibase: Option<String>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// DID認証
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DIDAuthentication {
    /// 参照
    Reference(String),
    /// 埋め込み
    Embedded(DIDVerificationMethod),
}

/// DIDサービス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIDService {
    /// ID
    pub id: String,
    /// タイプ
    #[serde(rename = "type")]
    pub type_: String,
    /// サービスエンドポイント
    #[serde(rename = "serviceEndpoint")]
    pub service_endpoint: ServiceEndpoint,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// サービスエンドポイント
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServiceEndpoint {
    /// 単一URI
    Uri(String),
    /// 複数URI
    UriSet(Vec<String>),
    /// マップ
    Map(HashMap<String, serde_json::Value>),
}

/// DID証明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIDProof {
    /// タイプ
    #[serde(rename = "type")]
    pub type_: String,
    /// 作成日時
    pub created: DateTime<Utc>,
    /// 検証メソッド
    #[serde(rename = "verificationMethod")]
    pub verification_method: String,
    /// プロパティ
    #[serde(rename = "proofPurpose")]
    pub proof_purpose: String,
    /// JWS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jws: Option<String>,
    /// 証明値
    #[serde(rename = "proofValue", skip_serializing_if = "Option::is_none")]
    pub proof_value: Option<String>,
    /// ノンス
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    /// ドメイン
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// チャレンジ
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenge: Option<String>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}
