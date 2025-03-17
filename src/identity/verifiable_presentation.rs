use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::Error;
use crate::identity::did::DID;
use crate::identity::verifiable_credential::VerifiableCredential;

/// 検証可能プレゼンテーション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiablePresentation {
    /// コンテキスト
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    /// ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// タイプ
    #[serde(rename = "type")]
    pub type_: Vec<String>,
    /// 検証可能クレデンシャル
    #[serde(
        rename = "verifiableCredential",
        skip_serializing_if = "Option::is_none"
    )]
    pub verifiable_credential: Option<Vec<VerifiableCredential>>,
    /// ホルダー
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holder: Option<String>,
    /// 証明
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<PresentationProof>,
    /// 追加プロパティ
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

impl VerifiablePresentation {
    /// 新しい検証可能プレゼンテーションを作成
    pub fn new() -> Self {
        Self {
            context: vec!["https://www.w3.org/2018/credentials/v1".to_string()],
            id: None,
            type_: vec!["VerifiablePresentation".to_string()],
            verifiable_credential: None,
            holder: None,
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

    /// タイプを追加
    pub fn add_type(mut self, type_: String) -> Self {
        self.type_.push(type_);
        self
    }

    /// 検証可能クレデンシャルを追加
    pub fn add_credential(mut self, credential: VerifiableCredential) -> Self {
        if self.verifiable_credential.is_none() {
            self.verifiable_credential = Some(Vec::new());
        }

        if let Some(credentials) = &mut self.verifiable_credential {
            credentials.push(credential);
        }

        self
    }

    /// ホルダーを設定
    pub fn with_holder(mut self, holder: String) -> Self {
        self.holder = Some(holder);
        self
    }

    /// 証明を設定
    pub fn with_proof(mut self, proof: PresentationProof) -> Self {
        self.proof = Some(proof);
        self
    }

    /// 検証可能プレゼンテーションを検証
    pub fn verify(&self) -> Result<bool, Error> {
        // 実際の実装では、プレゼンテーションの検証ロジックを実装する
        // ここでは簡易的な実装を提供

        // 必須フィールドをチェック
        if self.context.is_empty() {
            return Err(Error::InvalidInput("Context is required".to_string()));
        }

        if self.type_.is_empty() {
            return Err(Error::InvalidInput("Type is required".to_string()));
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

    /// クレデンシャルを検証
    pub fn verify_credentials(&self) -> Result<bool, Error> {
        // 実際の実装では、すべてのクレデンシャルの検証ロジックを実装する
        // ここでは簡易的な実装を提供

        if let Some(credentials) = &self.verifiable_credential {
            for credential in credentials {
                if !credential.verify()? {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }
}

/// プレゼンテーション証明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationProof {
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
