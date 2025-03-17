use crate::multisig::{
    MultisigManager, MultisigTransaction, MultisigWallet, Operation, TransactionData,
};
use crate::transaction::TransactionStatus;
use crate::wallet::{Account, WalletManager};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

// テスト用のアカウントを作成
fn create_test_account(id: &str, name: &str) -> Account {
    Account {
        id: id.to_string(),
        public_key: format!("pub_{}", id),
        private_key: format!("priv_{}", id),
        name: name.to_string(),
        balance: 1000.0,
        token_balances: HashMap::new(),
        created_at: chrono::Utc::now(),
    }
}

// テスト環境をセットアップ
fn setup_test_environment() -> (Arc<WalletManager>, Arc<MultisigManager>) {
    // ウォレットマネージャーを作成
    let wallet_manager = Arc::new(WalletManager::new());

    // テスト用のアカウントを追加
    wallet_manager.add_account(create_test_account("owner1", "Owner 1"));
    wallet_manager.add_account(create_test_account("signer2", "Signer 2"));
    wallet_manager.add_account(create_test_account("signer3", "Signer 3"));
    wallet_manager.add_account(create_test_account("recipient", "Recipient"));

    // マルチシグマネージャーを作成
    let multisig_manager = Arc::new(MultisigManager::new(wallet_manager.clone()));

    (wallet_manager, multisig_manager)
}

#[test]
fn test_multisig_wallet_creation() {
    let (_, multisig_manager) = setup_test_environment();

    // マルチシグウォレットを作成
    let wallet = multisig_manager
        .create_wallet(
            "Test Wallet".to_string(),
            "owner1",
            vec![
                "owner1".to_string(),
                "signer2".to_string(),
                "signer3".to_string(),
            ],
            2,
        )
        .unwrap();

    // ウォレットの基本情報を確認
    assert_eq!(wallet.name, "Test Wallet");
    assert_eq!(wallet.owner_id, "owner1");
    assert_eq!(wallet.signers.len(), 3);
    assert_eq!(wallet.required_signatures, 2);
    assert_eq!(wallet.balance, 0.0);
    assert!(wallet.token_balances.is_empty());

    // ウォレットが正しく保存されていることを確認
    let retrieved_wallet = multisig_manager.get_wallet(&wallet.id).unwrap();
    assert_eq!(retrieved_wallet.id, wallet.id);
    assert_eq!(retrieved_wallet.name, wallet.name);
}

#[test]
fn test_multisig_wallet_validation() {
    let (_, multisig_manager) = setup_test_environment();

    // 署名者が空の場合
    let result = multisig_manager.create_wallet("Test Wallet".to_string(), "owner1", vec![], 1);
    assert!(result.is_err());

    // 必要署名数が署名者数を超える場合
    let result = multisig_manager.create_wallet(
        "Test Wallet".to_string(),
        "owner1",
        vec!["owner1".to_string(), "signer2".to_string()],
        3,
    );
    assert!(result.is_err());

    // 必要署名数が0の場合
    let result = multisig_manager.create_wallet(
        "Test Wallet".to_string(),
        "owner1",
        vec!["owner1".to_string(), "signer2".to_string()],
        0,
    );
    assert!(result.is_err());

    // 署名者に重複がある場合
    let result = multisig_manager.create_wallet(
        "Test Wallet".to_string(),
        "owner1",
        vec![
            "owner1".to_string(),
            "signer2".to_string(),
            "signer2".to_string(),
        ],
        2,
    );
    assert!(result.is_err());

    // 所有者が署名者に含まれていない場合
    let result = multisig_manager.create_wallet(
        "Test Wallet".to_string(),
        "owner1",
        vec!["signer2".to_string(), "signer3".to_string()],
        1,
    );
    assert!(result.is_err());

    // 所有者アカウントが存在しない場合
    let result = multisig_manager.create_wallet(
        "Test Wallet".to_string(),
        "nonexistent",
        vec!["nonexistent".to_string(), "signer2".to_string()],
        1,
    );
    assert!(result.is_err());
}

#[test]
fn test_multisig_transaction_creation_and_signing() {
    let (wallet_manager, multisig_manager) = setup_test_environment();

    // マルチシグウォレットを作成
    let wallet = multisig_manager
        .create_wallet(
            "Test Wallet".to_string(),
            "owner1",
            vec![
                "owner1".to_string(),
                "signer2".to_string(),
                "signer3".to_string(),
            ],
            2,
        )
        .unwrap();

    // トランザクションデータを作成
    let tx_data = TransactionData {
        operation: Operation::Transfer {
            to: "recipient".to_string(),
            amount: 100.0,
            token_id: None,
        },
        memo: Some("Test transfer".to_string()),
        timestamp: chrono::Utc::now().timestamp(),
    };

    let tx_data_bytes = serde_json::to_vec(&tx_data).unwrap();

    // マルチシグトランザクションを作成
    let tx = multisig_manager
        .create_transaction(&wallet.id, "owner1", tx_data_bytes)
        .unwrap();

    // トランザクションの基本情報を確認
    assert_eq!(tx.wallet_id, wallet.id);
    assert_eq!(tx.creator_id, "owner1");
    assert_eq!(tx.required_signatures, 2);
    assert_eq!(tx.status, TransactionStatus::Pending);
    assert!(tx.signatures.contains_key("owner1"));

    // 署名を追加
    let signature = vec![1, 2, 3]; // ダミー署名
    let signed_tx = multisig_manager
        .sign_transaction(&tx.id, "owner1", signature.clone())
        .unwrap();

    // 署名が正しく追加されたことを確認
    assert!(signed_tx.signatures.contains_key("owner1"));
    assert_eq!(signed_tx.signatures["owner1"].signature, signature);

    // まだ署名が不足しているため、トランザクションはPending状態
    assert_eq!(signed_tx.status, TransactionStatus::Pending);

    // 2人目の署名を追加
    let signature2 = vec![4, 5, 6]; // ダミー署名
    let signed_tx2 = multisig_manager
        .sign_transaction(&tx.id, "signer2", signature2.clone())
        .unwrap();

    // 署名が十分なため、トランザクションはConfirmed状態
    assert_eq!(signed_tx2.status, TransactionStatus::Confirmed);
    assert!(signed_tx2.executed_at.is_some());

    // 残高が正しく更新されたことを確認
    let recipient = wallet_manager.get_account("recipient").unwrap();
    assert_eq!(recipient.balance, 1100.0); // 初期残高1000 + 送金額100
}

#[test]
fn test_multisig_transaction_rejection() {
    let (_, multisig_manager) = setup_test_environment();

    // マルチシグウォレットを作成
    let wallet = multisig_manager
        .create_wallet(
            "Test Wallet".to_string(),
            "owner1",
            vec![
                "owner1".to_string(),
                "signer2".to_string(),
                "signer3".to_string(),
            ],
            2,
        )
        .unwrap();

    // トランザクションデータを作成
    let tx_data = TransactionData {
        operation: Operation::Transfer {
            to: "recipient".to_string(),
            amount: 100.0,
            token_id: None,
        },
        memo: Some("Test transfer".to_string()),
        timestamp: chrono::Utc::now().timestamp(),
    };

    let tx_data_bytes = serde_json::to_vec(&tx_data).unwrap();

    // マルチシグトランザクションを作成
    let tx = multisig_manager
        .create_transaction(&wallet.id, "owner1", tx_data_bytes)
        .unwrap();

    // トランザクションを拒否
    let rejected_tx = multisig_manager
        .reject_transaction(&tx.id, "signer2")
        .unwrap();

    // トランザクションがRejected状態になったことを確認
    assert_eq!(rejected_tx.status, TransactionStatus::Rejected);

    // 拒否されたトランザクションに署名しようとすると失敗する
    let signature = vec![1, 2, 3]; // ダミー署名
    let result = multisig_manager.sign_transaction(&tx.id, "owner1", signature);
    assert!(result.is_err());
}

#[test]
fn test_multisig_wallet_operations() {
    let (_, multisig_manager) = setup_test_environment();

    // マルチシグウォレットを作成
    let wallet = multisig_manager
        .create_wallet(
            "Test Wallet".to_string(),
            "owner1",
            vec!["owner1".to_string(), "signer2".to_string()],
            2,
        )
        .unwrap();

    // 署名者追加のトランザクションを作成
    let tx_data = TransactionData {
        operation: Operation::AddSigner {
            signer_id: "signer3".to_string(),
        },
        memo: Some("Add signer".to_string()),
        timestamp: chrono::Utc::now().timestamp(),
    };

    let tx_data_bytes = serde_json::to_vec(&tx_data).unwrap();

    let tx = multisig_manager
        .create_transaction(&wallet.id, "owner1", tx_data_bytes)
        .unwrap();

    // 署名を追加
    multisig_manager
        .sign_transaction(&tx.id, "owner1", vec![1, 2, 3])
        .unwrap();

    multisig_manager
        .sign_transaction(&tx.id, "signer2", vec![4, 5, 6])
        .unwrap();

    // ウォレットに署名者が追加されたことを確認
    let updated_wallet = multisig_manager.get_wallet(&wallet.id).unwrap();
    assert_eq!(updated_wallet.signers.len(), 3);
    assert!(updated_wallet.signers.contains(&"signer3".to_string()));

    // 必要署名数変更のトランザクションを作成
    let tx_data = TransactionData {
        operation: Operation::ChangeRequiredSignatures { required: 3 },
        memo: Some("Change required signatures".to_string()),
        timestamp: chrono::Utc::now().timestamp(),
    };

    let tx_data_bytes = serde_json::to_vec(&tx_data).unwrap();

    let tx = multisig_manager
        .create_transaction(&wallet.id, "owner1", tx_data_bytes)
        .unwrap();

    // 署名を追加
    multisig_manager
        .sign_transaction(&tx.id, "owner1", vec![1, 2, 3])
        .unwrap();

    multisig_manager
        .sign_transaction(&tx.id, "signer2", vec![4, 5, 6])
        .unwrap();

    // 必要署名数が変更されたことを確認
    let updated_wallet = multisig_manager.get_wallet(&wallet.id).unwrap();
    assert_eq!(updated_wallet.required_signatures, 3);
}

#[test]
fn test_get_wallets_by_account() {
    let (_, multisig_manager) = setup_test_environment();

    // 複数のウォレットを作成
    multisig_manager
        .create_wallet(
            "Wallet 1".to_string(),
            "owner1",
            vec!["owner1".to_string(), "signer2".to_string()],
            2,
        )
        .unwrap();

    multisig_manager
        .create_wallet(
            "Wallet 2".to_string(),
            "signer2",
            vec!["signer2".to_string(), "signer3".to_string()],
            1,
        )
        .unwrap();

    multisig_manager
        .create_wallet(
            "Wallet 3".to_string(),
            "owner1",
            vec!["owner1".to_string(), "signer3".to_string()],
            2,
        )
        .unwrap();

    // アカウントに関連するウォレットを取得
    let owner1_wallets = multisig_manager.get_wallets_by_account("owner1");
    assert_eq!(owner1_wallets.len(), 2);

    let signer2_wallets = multisig_manager.get_wallets_by_account("signer2");
    assert_eq!(signer2_wallets.len(), 2);

    let signer3_wallets = multisig_manager.get_wallets_by_account("signer3");
    assert_eq!(signer3_wallets.len(), 2);

    let nonexistent_wallets = multisig_manager.get_wallets_by_account("nonexistent");
    assert_eq!(nonexistent_wallets.len(), 0);
}
