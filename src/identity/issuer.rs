use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::crypto::{PublicKey, PrivateKey, KeyPair, Signature, hash};
use crate::identity::did::{DID, DIDDocument};
use crate::identity::verifiable_credential::{VerifiableCredential, CredentialSubject, CredentialStatus, CredentialSchema, CredentialProof, Issuer as CredentialIssuer};
use crate::identity::schema::{Schema, SchemaValidator};

/// 発行者メタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerMetadata {
    /// 発行者名
    pub name: String,
    /// 発行者説明
    pub description: Option<String>,
    /// 発行者ウェブサイト
    pub website: Option<String>,
    /// 発行者ロゴ
    pub logo: Option<String>,
    /// 発行者メールアドレス
    pub email: Option<String>,
    /// 発行者電話番号
    pub phone: Option<String>,
    /// 発行者住所
    pub address: Option<String>,
    /// 発行者ソーシャルメディア
    pub social_media: Option<HashMap<String, String>>,
    /// 発行者作成日時
    pub created_at: DateTime<Utc>,
    /// 発行者更新日時
    pub updated_at: DateTime<Utc>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// 発行者オプション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerOptions {
    /// 有効期限（秒）
    pub expiry_seconds: Option<u64>,
    /// 自動更新
    pub auto_renewal: Option<bool>,
    /// 自動更新期間（秒）
    pub auto_renewal_seconds: Option<u64>,
    /// 失効可能
    pub revocable: Option<bool>,
    /// 検証可能
    pub verifiable: Option<bool>,
    /// スキーマ検証
    pub schema_validation: Option<bool>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

impl Default for IssuerOptions {
    fn default() -> Self {
        Self {
            expiry_seconds: Some(31536000), // 1年
            auto_renewal: Some(false),
            auto_renewal_seconds: Some(2592000), // 30日
            revocable: Some(true),
            verifiable: Some(true),
            schema_validation: Some(true),
            additional_properties: HashMap::new(),
        }
    }
}

/// 発行者
pub trait Issuer {
    /// 発行者DIDを取得
    fn did(&self) -> &DID;
    
    /// 発行者メタデータを取得
    fn metadata(&self) -> &IssuerMetadata;
    
    /// クレデンシャルを発行
    fn issue_credential(
        &self,
        subject: CredentialSubject,
        types: Vec<String>,
        schema: Option<CredentialSchema>,
        options: Option<IssuerOptions>,
    ) -> Result<VerifiableCredential, Error>;
    
    /// クレデンシャルを検証
    fn verify_credential(&self, credential: &VerifiableCredential) -> Result<bool, Error>;
    
    /// クレデンシャルを失効
    fn revoke_credential(&self, credential_id: &str) -> Result<(), Error>;
    
    /// クレデンシャルの失効状態を確認
    fn check_revocation(&self, credential_id: &str) -> Result<bool, Error>;
    
    /// クレデンシャルを更新
    fn update_credential(
        &self,
        credential: &VerifiableCredential,
        subject: Option<CredentialSubject>,
        options: Option<IssuerOptions>,
    ) -> Result<VerifiableCredential, Error>;
}

/// 基本発行者
pub struct BasicIssuer {
    /// 発行者DID
    did: DID,
    /// 発行者キーペア
    key_pair: KeyPair,
    /// 発行者メタデータ
    metadata: IssuerMetadata,
    /// 発行したクレデンシャル
    credentials: HashMap<String, VerifiableCredential>,
    /// 失効したクレデンシャルID
    revoked_credentials: Vec<String>,
    /// スキーマバリデータ
    schema_validator: SchemaValidator,
}

impl BasicIssuer {
    /// 新しい基本発行者を作成
    pub fn new(did: DID, key_pair: KeyPair, name: String) -> Self {
        let metadata = IssuerMetadata {
            name,
            description: None,
            website: None,
            logo: None,
            email: None,
            phone: None,
            address: None,
            social_media: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            additional_properties: HashMap::new(),
        };
        
        Self {
            did,
            key_pair,
            metadata,
            credentials: HashMap::new(),
            revoked_credentials: Vec::new(),
            schema_validator: SchemaValidator::new(),
        }
    }
    
    /// メタデータを更新
    pub fn update_metadata(&mut self, metadata: IssuerMetadata) {
        self.metadata = metadata;
        self.metadata.updated_at = Utc::now();
    }
    
    /// スキーマを追加
    pub fn add_schema(&mut self, schema: Schema) {
        self.schema_validator.add_schema(schema);
    }
    
    /// クレデンシャルIDを生成
    fn generate_credential_id(&self) -> String {
        let now = Utc::now();
        let random = rand::random::<u64>();
        format!("urn:uuid:{}-{}-{}", self.did.to_string(), now.timestamp(), random)
    }
    
    /// クレデンシャルに署名
    fn sign_credential(&self, credential: &mut VerifiableCredential) -> Result<(), Error> {
        // クレデンシャルをシリアライズ
        let credential_json = serde_json::to_string(credential)
            .map_err(|e| Error::SerializationError(format!("Failed to serialize credential: {}", e)))?;
        
        // 署名を作成
        let signature = self.key_pair.sign(credential_json.as_bytes())
            .map_err(|e| Error::CryptoError(format!("Failed to sign credential: {}", e)))?;
        
        // 証明を作成
        let proof = CredentialProof {
            type_: "Ed25519Signature2018".to_string(),
            created: Utc::now(),
            verification_method: format!("{}#keys-1", self.did.to_string()),
            proof_purpose: "assertionMethod".to_string(),
            jws: None,
            proof_value: Some(base64::encode(signature.as_bytes())),
            nonce: None,
            domain: None,
            challenge: None,
            additional_properties: HashMap::new(),
        };
        
        // 証明を設定
        credential.proof = Some(proof);
        
        Ok(())
    }
}

impl Issuer for BasicIssuer {
    fn did(&self) -> &DID {
        &self.did
    }
    
    fn metadata(&self) -> &IssuerMetadata {
        &self.metadata
    }
    
    fn issue_credential(
        &self,
        subject: CredentialSubject,
        types: Vec<String>,
        schema: Option<CredentialSchema>,
        options: Option<IssuerOptions>,
    ) -> Result<VerifiableCredential, Error> {
        let options = options.unwrap_or_default();
        
        // スキーマ検証
        if options.schema_validation.unwrap_or(true) {
            if let Some(schema) = &schema {
                let schema_id = &schema.id;
                let subject_json = serde_json::to_value(&subject)
                    .map_err(|e| Error::SerializationError(format!("Failed to serialize subject: {}", e)))?;
                
                if !self.schema_validator.validate(schema_id, &subject_json)? {
                    return Err(Error::ValidationError("Subject does not match schema".to_string()));
                }
            }
        }
        
        // クレデンシャルIDを生成
        let credential_id = self.generate_credential_id();
        
        // 発行者を作成
        let issuer = CredentialIssuer::Uri(self.did.to_string());
        
        // クレデンシャルを作成
        let mut credential = VerifiableCredential::new(
            types,
            issuer,
            subject,
        ).with_id(credential_id.clone());
        
        // スキーマを設定
        if let Some(schema) = schema {
            credential = credential.with_credential_schema(schema);
        }
        
        // 有効期限を設定
        if let Some(expiry_seconds) = options.expiry_seconds {
            let expiry_date = Utc::now() + Duration::seconds(expiry_seconds as i64);
            credential = credential.with_expiration_date(expiry_date);
        }
        
        // クレデンシャルに署名
        self.sign_credential(&mut credential)?;
        
        // クレデンシャルを保存
        let mut issuer = self.clone();
        issuer.credentials.insert(credential_id, credential.clone());
        
        Ok(credential)
    }
    
    fn verify_credential(&self, credential: &VerifiableCredential) -> Result<bool, Error> {
        // 発行者をチェック
        let issuer_did = match &credential.issuer {
            CredentialIssuer::Uri(uri) => {
                DID::parse(uri)?
            },
            CredentialIssuer::Object(obj) => {
                DID::parse(&obj.id)?
            },
        };
        
        if issuer_did != self.did {
            return Ok(false);
        }
        
        // 有効期限をチェック
        if let Some(expiration_date) = credential.expirationDate {
            if expiration_date < Utc::now() {
                return Ok(false);
            }
        }
        
        // 失効状態をチェック
        if let Some(id) = &credential.id {
            if self.revoked_credentials.contains(id) {
                return Ok(false);
            }
        }
        
        // 証明をチェック
        if let Some(proof) = &credential.proof {
            // 証明値を取得
            let proof_value = proof.proof_value.as_ref()
                .ok_or_else(|| Error::ValidationError("Proof value is missing".to_string()))?;
            
            // 署名を復元
            let signature_bytes = base64::decode(proof_value)
                .map_err(|e| Error::ValidationError(format!("Invalid proof value: {}", e)))?;
            
            let signature = Signature::from_bytes(&signature_bytes)
                .map_err(|e| Error::ValidationError(format!("Invalid signature: {}", e)))?;
            
            // 証明を一時的に削除
            let mut credential_without_proof = credential.clone();
            credential_without_proof.proof = None;
            
            // クレデンシャルをシリアライズ
            let credential_json = serde_json::to_string(&credential_without_proof)
                .map_err(|e| Error::SerializationError(format!("Failed to serialize credential: {}", e)))?;
            
            // 署名を検証
            let valid = self.key_pair.public_key().verify(credential_json.as_bytes(), &signature)
                .map_err(|e| Error::CryptoError(format!("Failed to verify signature: {}", e)))?;
            
            return Ok(valid);
        }
        
        Ok(false)
    }
    
    fn revoke_credential(&self, credential_id: &str) -> Result<(), Error> {
        // クレデンシャルの存在を確認
        if !self.credentials.contains_key(credential_id) {
            return Err(Error::NotFound(format!("Credential not found: {}", credential_id)));
        }
        
        // クレデンシャルを失効
        let mut issuer = self.clone();
        issuer.revoked_credentials.push(credential_id.to_string());
        
        Ok(())
    }
    
    fn check_revocation(&self, credential_id: &str) -> Result<bool, Error> {
        Ok(self.revoked_credentials.contains(&credential_id.to_string()))
    }
    
    fn update_credential(
        &self,
        credential: &VerifiableCredential,
        subject: Option<CredentialSubject>,
        options: Option<IssuerOptions>,
    ) -> Result<VerifiableCredential, Error> {
        let options = options.unwrap_or_default();
        
        // クレデンシャルIDを取得
        let credential_id = credential.id.as_ref()
            .ok_or_else(|| Error::InvalidInput("Credential ID is missing".to_string()))?;
        
        // クレデンシャルの存在を確認
        if !self.credentials.contains_key(credential_id) {
            return Err(Error::NotFound(format!("Credential not found: {}", credential_id)));
        }
        
        // クレデンシャルの失効状態を確認
        if self.revoked_credentials.contains(credential_id) {
            return Err(Error::InvalidState(format!("Credential is revoked: {}", credential_id)));
        }
        
        // 新しいクレデンシャルを作成
        let mut new_credential = credential.clone();
        
        // サブジェクトを更新
        if let Some(subject) = subject {
            new_credential.credentialSubject = subject;
        }
        
        // 有効期限を更新
        if let Some(expiry_seconds) = options.expiry_seconds {
            let expiry_date = Utc::now() + Duration::seconds(expiry_seconds as i64);
            new_credential.expirationDate = Some(expiry_date);
        }
        
        // クレデンシャルに署名
        self.sign_credential(&mut new_credential)?;
        
        // クレデンシャルを保存
        let mut issuer = self.clone();
        issuer.credentials.insert(credential_id.clone(), new_credential.clone());
        
        Ok(new_credential)
    }
}