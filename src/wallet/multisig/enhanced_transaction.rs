use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::crypto::{hash, PublicKey, Signature};
use crate::error::Error;
use crate::transaction::{Transaction, TransactionStatus};
use crate::wallet::multisig::threshold::ThresholdPolicy;
use crate::wallet::WalletId;

/// マルチシグトランザクション状態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MultisigTransactionState {
    /// 作成済み
    Created,
    /// 署名中
    PartiallyApproved,
    /// 承認済み
    Approved,
    /// 実行済み
    Executed,
    /// 拒否
    Rejected,
    /// 期限切れ
    Expired,
    /// キャンセル
    Cancelled,
}

/// マルチシグトランザクション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedMultisigTransaction {
    /// トランザクションID
    pub id: String,
    /// ウォレットID
    pub wallet_id: WalletId,
    /// 基本トランザクション
    pub transaction: Transaction,
    /// 閾値ポリシー
    pub policy: ThresholdPolicy,
    /// 署名マップ（公開鍵 -> 署名）
    pub signatures: HashMap<PublicKey, Signature>,
    /// 状態
    pub state: MultisigTransactionState,
    /// 作成時刻
    pub created_at: DateTime<Utc>,
    /// 更新時刻
    pub updated_at: DateTime<Utc>,
    /// 実行時刻
    pub executed_at: Option<DateTime<Utc>>,
    /// 有効期限
    pub expires_at: Option<DateTime<Utc>>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
    /// 拒否理由
    pub rejection_reason: Option<String>,
    /// 実行結果
    pub execution_result: Option<String>,
}

impl EnhancedMultisigTransaction {
    /// 新しいマルチシグトランザクションを作成
    pub fn new(wallet_id: WalletId, transaction: Transaction, policy: ThresholdPolicy) -> Self {
        let now = Utc::now();
        let id = format!(
            "multisig-{}",
            hash(&format!("{}-{}-{}", wallet_id, transaction.id, now))
        );

        // 有効期限を設定（デフォルトは24時間）
        let expires_at = policy
            .expiration
            .or_else(|| Some(now + chrono::Duration::hours(24)));

        Self {
            id,
            wallet_id,
            transaction,
            policy,
            signatures: HashMap::new(),
            state: MultisigTransactionState::Created,
            created_at: now,
            updated_at: now,
            executed_at: None,
            expires_at,
            metadata: HashMap::new(),
            rejection_reason: None,
            execution_result: None,
        }
    }

    /// 署名を追加
    pub fn add_signature(
        &mut self,
        public_key: PublicKey,
        signature: Signature,
    ) -> Result<(), Error> {
        // 状態をチェック
        if self.state != MultisigTransactionState::Created
            && self.state != MultisigTransactionState::PartiallyApproved
        {
            return Err(Error::InvalidState(format!(
                "トランザクションは {} 状態であり、署名を追加できません",
                format!("{:?}", self.state)
            )));
        }

        // 有効期限をチェック
        if let Some(expires_at) = self.expires_at {
            if Utc::now() > expires_at {
                self.state = MultisigTransactionState::Expired;
                return Err(Error::Expired(
                    "トランザクションの有効期限が切れています".to_string(),
                ));
            }
        }

        // 公開鍵が許可されているかどうかをチェック
        if !self.policy.is_allowed(&public_key) {
            return Err(Error::Unauthorized(format!(
                "公開鍵 {} はこのマルチシグウォレットで許可されていません",
                public_key
            )));
        }

        // 署名を追加
        self.signatures.insert(public_key, signature);

        // 状態を更新
        if self.policy.is_threshold_met(&self.signatures) {
            self.state = MultisigTransactionState::Approved;
        } else {
            self.state = MultisigTransactionState::PartiallyApproved;
        }

        self.updated_at = Utc::now();

        Ok(())
    }

    /// トランザクションを実行
    pub fn execute(&mut self) -> Result<(), Error> {
        // 状態をチェック
        if self.state != MultisigTransactionState::Approved {
            return Err(Error::InvalidState(format!(
                "トランザクションは {} 状態であり、実行できません",
                format!("{:?}", self.state)
            )));
        }

        // 有効期限をチェック
        if let Some(expires_at) = self.expires_at {
            if Utc::now() > expires_at {
                self.state = MultisigTransactionState::Expired;
                return Err(Error::Expired(
                    "トランザクションの有効期限が切れています".to_string(),
                ));
            }
        }

        // 閾値を満たしているかどうかを再確認
        if !self.policy.is_threshold_met(&self.signatures) {
            return Err(Error::Unauthorized(
                "必要な署名数を満たしていません".to_string(),
            ));
        }

        // 状態を更新
        self.state = MultisigTransactionState::Executed;
        self.executed_at = Some(Utc::now());
        self.updated_at = Utc::now();
        self.execution_result = Some("トランザクションが正常に実行されました".to_string());

        Ok(())
    }

    /// トランザクションを拒否
    pub fn reject(&mut self, reason: Option<String>) -> Result<(), Error> {
        // 状態をチェック
        if self.state != MultisigTransactionState::Created
            && self.state != MultisigTransactionState::PartiallyApproved
            && self.state != MultisigTransactionState::Approved
        {
            return Err(Error::InvalidState(format!(
                "トランザクションは {} 状態であり、拒否できません",
                format!("{:?}", self.state)
            )));
        }

        // 状態を更新
        self.state = MultisigTransactionState::Rejected;
        self.updated_at = Utc::now();
        self.rejection_reason = reason;

        Ok(())
    }

    /// トランザクションをキャンセル
    pub fn cancel(&mut self) -> Result<(), Error> {
        // 状態をチェック
        if self.state != MultisigTransactionState::Created
            && self.state != MultisigTransactionState::PartiallyApproved
            && self.state != MultisigTransactionState::Approved
        {
            return Err(Error::InvalidState(format!(
                "トランザクションは {} 状態であり、キャンセルできません",
                format!("{:?}", self.state)
            )));
        }

        // 状態を更新
        self.state = MultisigTransactionState::Cancelled;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// 署名数を取得
    pub fn signature_count(&self) -> usize {
        self.signatures.len()
    }

    /// 残りの必要署名数を取得
    pub fn remaining_signatures(&self) -> usize {
        self.policy.remaining_signatures(&self.signatures)
    }

    /// 有効期限までの残り時間（秒）を取得
    pub fn time_remaining(&self) -> Option<i64> {
        self.expires_at.map(|expires_at| {
            let now = Utc::now();
            if now >= expires_at {
                return 0;
            }

            (expires_at - now).num_seconds()
        })
    }

    /// メタデータを設定
    pub fn set_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
        self.updated_at = Utc::now();
    }

    /// メタデータを取得
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// 有効期限を延長
    pub fn extend_expiration(&mut self, duration_seconds: i64) -> Result<(), Error> {
        // 状態をチェック
        if self.state != MultisigTransactionState::Created
            && self.state != MultisigTransactionState::PartiallyApproved
            && self.state != MultisigTransactionState::Approved
        {
            return Err(Error::InvalidState(format!(
                "トランザクションは {} 状態であり、有効期限を延長できません",
                format!("{:?}", self.state)
            )));
        }

        // 有効期限を延長
        if let Some(expires_at) = self.expires_at {
            self.expires_at = Some(expires_at + chrono::Duration::seconds(duration_seconds));
            self.updated_at = Utc::now();
        } else {
            // 有効期限が設定されていない場合は、現在時刻から設定
            self.expires_at = Some(Utc::now() + chrono::Duration::seconds(duration_seconds));
            self.updated_at = Utc::now();
        }

        Ok(())
    }

    /// トランザクションが期限切れかどうかをチェック
    pub fn check_expiration(&mut self) -> bool {
        if let Some(expires_at) = self.expires_at {
            if Utc::now() > expires_at && self.state != MultisigTransactionState::Expired {
                self.state = MultisigTransactionState::Expired;
                self.updated_at = Utc::now();
                return true;
            }
        }

        false
    }

    /// トランザクションの概要を取得
    pub fn get_summary(&self) -> String {
        let status = match self.state {
            MultisigTransactionState::Created => "作成済み",
            MultisigTransactionState::PartiallyApproved => "署名中",
            MultisigTransactionState::Approved => "承認済み",
            MultisigTransactionState::Executed => "実行済み",
            MultisigTransactionState::Rejected => "拒否",
            MultisigTransactionState::Expired => "期限切れ",
            MultisigTransactionState::Cancelled => "キャンセル",
        };

        let signatures = format!(
            "{}/{}",
            self.signature_count(),
            self.policy.required_signatures
        );

        let expiration = if let Some(expires_at) = self.expires_at {
            format!("{}", expires_at.format("%Y-%m-%d %H:%M:%S"))
        } else {
            "なし".to_string()
        };

        format!(
            "ID: {}\n状態: {}\n署名: {}\n有効期限: {}\n作成日時: {}\n更新日時: {}",
            self.id,
            status,
            signatures,
            expiration,
            self.created_at.format("%Y-%m-%d %H:%M:%S"),
            self.updated_at.format("%Y-%m-%d %H:%M:%S")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_keypair;

    fn create_test_transaction() -> Transaction {
        Transaction {
            id: "tx1".to_string(),
            sender: "sender".to_string(),
            receiver: "receiver".to_string(),
            amount: 100,
            fee: 10,
            timestamp: Utc::now().timestamp(),
            signature: None,
            status: TransactionStatus::Pending,
            data: None,
        }
    }

    #[test]
    fn test_multisig_transaction() {
        // キーペアを生成
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let keypair3 = generate_keypair();

        // 2-of-3ポリシーを作成
        let policy = ThresholdPolicy::new(
            2,
            vec![
                keypair1.public.clone(),
                keypair2.public.clone(),
                keypair3.public.clone(),
            ],
        );

        // トランザクションを作成
        let transaction = create_test_transaction();

        // マルチシグトランザクションを作成
        let mut multisig_tx =
            EnhancedMultisigTransaction::new("wallet1".to_string(), transaction, policy);

        // 初期状態を確認
        assert_eq!(multisig_tx.state, MultisigTransactionState::Created);
        assert_eq!(multisig_tx.signature_count(), 0);
        assert_eq!(multisig_tx.remaining_signatures(), 2);

        // 署名を追加
        multisig_tx
            .add_signature(keypair1.public.clone(), "sig1".to_string())
            .unwrap();

        // 状態を確認
        assert_eq!(
            multisig_tx.state,
            MultisigTransactionState::PartiallyApproved
        );
        assert_eq!(multisig_tx.signature_count(), 1);
        assert_eq!(multisig_tx.remaining_signatures(), 1);

        // 署名を追加
        multisig_tx
            .add_signature(keypair2.public.clone(), "sig2".to_string())
            .unwrap();

        // 状態を確認
        assert_eq!(multisig_tx.state, MultisigTransactionState::Approved);
        assert_eq!(multisig_tx.signature_count(), 2);
        assert_eq!(multisig_tx.remaining_signatures(), 0);

        // トランザクションを実行
        multisig_tx.execute().unwrap();

        // 状態を確認
        assert_eq!(multisig_tx.state, MultisigTransactionState::Executed);
        assert!(multisig_tx.executed_at.is_some());
        assert!(multisig_tx.execution_result.is_some());
    }

    #[test]
    fn test_transaction_expiration() {
        // キーペアを生成
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();

        // 有効期限付きポリシーを作成（過去の日時）
        let past = Utc::now() - chrono::Duration::hours(1);
        let policy = ThresholdPolicy::with_expiration(
            1,
            vec![keypair1.public.clone(), keypair2.public.clone()],
            past,
        );

        // トランザクションを作成
        let transaction = create_test_transaction();

        // マルチシグトランザクションを作成
        let mut multisig_tx =
            EnhancedMultisigTransaction::new("wallet1".to_string(), transaction, policy);

        // 有効期限をチェック
        assert!(multisig_tx.check_expiration());
        assert_eq!(multisig_tx.state, MultisigTransactionState::Expired);

        // 署名を追加（失敗するはず）
        let result = multisig_tx.add_signature(keypair1.public.clone(), "sig1".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_transaction_rejection() {
        // キーペアを生成
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();

        // ポリシーを作成
        let policy =
            ThresholdPolicy::new(2, vec![keypair1.public.clone(), keypair2.public.clone()]);

        // トランザクションを作成
        let transaction = create_test_transaction();

        // マルチシグトランザクションを作成
        let mut multisig_tx =
            EnhancedMultisigTransaction::new("wallet1".to_string(), transaction, policy);

        // トランザクションを拒否
        let reason = Some("不審な取引先".to_string());
        multisig_tx.reject(reason.clone()).unwrap();

        // 状態を確認
        assert_eq!(multisig_tx.state, MultisigTransactionState::Rejected);
        assert_eq!(multisig_tx.rejection_reason, reason);

        // 署名を追加（失敗するはず）
        let result = multisig_tx.add_signature(keypair1.public.clone(), "sig1".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_metadata() {
        // キーペアを生成
        let keypair1 = generate_keypair();

        // ポリシーを作成
        let policy = ThresholdPolicy::new(1, vec![keypair1.public.clone()]);

        // トランザクションを作成
        let transaction = create_test_transaction();

        // マルチシグトランザクションを作成
        let mut multisig_tx =
            EnhancedMultisigTransaction::new("wallet1".to_string(), transaction, policy);

        // メタデータを設定
        multisig_tx.set_metadata("purpose", "給与支払い");
        multisig_tx.set_metadata("department", "経理部");

        // メタデータを取得
        assert_eq!(
            multisig_tx.get_metadata("purpose"),
            Some(&"給与支払い".to_string())
        );
        assert_eq!(
            multisig_tx.get_metadata("department"),
            Some(&"経理部".to_string())
        );
        assert_eq!(multisig_tx.get_metadata("nonexistent"), None);
    }
}
