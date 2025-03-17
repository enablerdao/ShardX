use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::Error;
use crate::identity::did::DID;

/// 検証可能クレデンシャル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiableCredential {
    /// コンテキスト
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    /// ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// タイプ
    #[serde(rename = "type")]
    pub type_: Vec<String>,
    /// 発行者
    pub issuer: Issuer,
    /// 発行日時
    pub issuanceDate: DateTime<Utc>,
    /// 有効期限
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expirationDate: Option<DateTime<Utc>>,
    /// クレデンシャルサブジェクト
    pub credentialSubject: CredentialSubject,
    /// クレデンシャルステータス
    #[serde(rename = "credentialStatus", skip_serializing_if = "Option::is_none")]
    pub credential_status: Option<CredentialStatus>,
    /// クレデンシャルスキーマ
    #[serde(rename = "credentialSchema", skip_serializing_if = "Option::is_none")]
    pub credential_schema: Option<CredentialSchema>,
    /// 更新可能
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refreshService: Option<RefreshService>,
    /// 期限切れ
    #[serde(skip_serializing_if = "Option::is_none")]
    pub termsOfUse: Option<Vec<TermsOfUse>>,
    /// 証拠
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<Vec<CredentialEvidence>>,
    /// 証明
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<CredentialProof>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

impl VerifiableCredential {
    /// 新しい検証可能クレデンシャルを作成
    pub fn new(type_: Vec<String>, issuer: Issuer, subject: CredentialSubject) -> Self {
        Self {
            context: vec!["https://www.w3.org/2018/credentials/v1".to_string()],
            id: None,
            type_: type_,
            issuer,
            issuanceDate: Utc::now(),
            expirationDate: None,
            credentialSubject: subject,
            credential_status: None,
            credential_schema: None,
            refreshService: None,
            termsOfUse: None,
            evidence: None,
            proof: None,
            additional_properties: HashMap::new(),
        }
    }

    /// IDを設定
    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    /// コンテキストを追加
    pub fn add_context(mut self, context: String) -> Self {
        self.context.push(context);
        self
    }

    /// 有効期限を設定
    pub fn with_expiration_date(mut self, expiration_date: DateTime<Utc>) -> Self {
        self.expirationDate = Some(expiration_date);
        self
    }

    /// クレデンシャルステータスを設定
    pub fn with_credential_status(mut self, status: CredentialStatus) -> Self {
        self.credential_status = Some(status);
        self
    }

    /// クレデンシャルスキーマを設定
    pub fn with_credential_schema(mut self, schema: CredentialSchema) -> Self {
        self.credential_schema = Some(schema);
        self
    }

    /// 更新サービスを設定
    pub fn with_refresh_service(mut self, service: RefreshService) -> Self {
        self.refreshService = Some(service);
        self
    }

    /// 利用規約を追加
    pub fn add_terms_of_use(mut self, terms: TermsOfUse) -> Self {
        if self.termsOfUse.is_none() {
            self.termsOfUse = Some(Vec::new());
        }

        if let Some(terms_list) = &mut self.termsOfUse {
            terms_list.push(terms);
        }

        self
    }

    /// 証拠を追加
    pub fn add_evidence(mut self, evidence: CredentialEvidence) -> Self {
        if self.evidence.is_none() {
            self.evidence = Some(Vec::new());
        }

        if let Some(evidence_list) = &mut self.evidence {
            evidence_list.push(evidence);
        }

        self
    }

    /// 証明を設定
    pub fn with_proof(mut self, proof: CredentialProof) -> Self {
        self.proof = Some(proof);
        self
    }

    /// 検証可能クレデンシャルを検証
    pub fn verify(&self) -> Result<bool, Error> {
        // 実際の実装では、クレデンシャルの検証ロジックを実装する
        // ここでは簡易的な実装を提供

        // 必須フィールドをチェック
        if self.context.is_empty() {
            return Err(Error::InvalidInput("Context is required".to_string()));
        }

        if self.type_.is_empty() {
            return Err(Error::InvalidInput("Type is required".to_string()));
        }

        // 有効期限をチェック
        if let Some(expiration_date) = self.expirationDate {
            if expiration_date < Utc::now() {
                return Ok(false);
            }
        }

        // 証明をチェック
        if let Some(proof) = &self.proof {
            // 実際の実装では、証明の検証ロジックを実装する
            // ここでは簡易的に常にtrueを返す
            return Ok(true);
        }

        // 証明がない場合はfalseを返す
        Ok(false)
    }
}

/// 発行者
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Issuer {
    /// URI
    Uri(String),
    /// オブジェクト
    Object(IssuerObject),
}

/// 発行者オブジェクト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerObject {
    /// ID
    pub id: String,
    /// 名前
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// クレデンシャルサブジェクト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialSubject {
    /// ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub claims: HashMap<String, serde_json::Value>,
}

impl CredentialSubject {
    /// 新しいクレデンシャルサブジェクトを作成
    pub fn new() -> Self {
        Self {
            id: None,
            claims: HashMap::new(),
        }
    }

    /// IDを設定
    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    /// クレームを追加
    pub fn add_claim<T: Serialize>(mut self, key: &str, value: T) -> Result<Self, Error> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| Error::SerializationError(format!("Failed to serialize claim: {}", e)))?;

        self.claims.insert(key.to_string(), json_value);

        Ok(self)
    }

    /// クレームを取得
    pub fn get_claim<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>, Error> {
        if let Some(value) = self.claims.get(key) {
            let result = serde_json::from_value(value.clone()).map_err(|e| {
                Error::DeserializationError(format!("Failed to deserialize claim: {}", e))
            })?;

            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}

/// クレデンシャルステータス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialStatus {
    /// ID
    pub id: String,
    /// タイプ
    #[serde(rename = "type")]
    pub type_: String,
    /// ステータスリストインデックス
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statusListIndex: Option<String>,
    /// ステータスリストクレデンシャル
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statusListCredential: Option<String>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// クレデンシャルスキーマ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialSchema {
    /// ID
    pub id: String,
    /// タイプ
    #[serde(rename = "type")]
    pub type_: String,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// 更新サービス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshService {
    /// ID
    pub id: String,
    /// タイプ
    #[serde(rename = "type")]
    pub type_: String,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// 利用規約
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermsOfUse {
    /// タイプ
    #[serde(rename = "type")]
    pub type_: String,
    /// ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// クレデンシャル証拠
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialEvidence {
    /// ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// タイプ
    #[serde(rename = "type")]
    pub type_: Vec<String>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// クレデンシャル証明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialProof {
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
