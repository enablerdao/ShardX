# マルチシグウォレット機能

ShardXのマルチシグウォレット機能は、複数の署名者による承認が必要なセキュアなウォレット機能を提供します。企業の資金管理、共同プロジェクトの資金管理、高額資産の保護などに最適です。

## 主な特徴

- **複数署名者**: 複数のアカウントによる署名が必要なトランザクションを作成
- **カスタマイズ可能な閾値**: 必要な署名数をカスタマイズ可能（例：3人中2人の署名が必要）
- **階層的権限**: 所有者と署名者の区別による階層的な権限管理
- **トランザクション履歴**: すべてのトランザクションと署名の履歴を追跡
- **セキュリティ強化**: 単一障害点を排除し、セキュリティを強化

## 使用方法

### マルチシグウォレットの作成

```rust
use shardx::multisig::{MultisigManager, MultisigWallet};
use shardx::wallet::WalletManager;
use std::sync::Arc;

// ウォレットマネージャーを初期化
let wallet_manager = Arc::new(WalletManager::new());

// マルチシグマネージャーを初期化
let multisig_manager = Arc::new(MultisigManager::new(wallet_manager.clone()));

// マルチシグウォレットを作成
let wallet = multisig_manager.create_wallet(
    "組織資金ウォレット".to_string(),
    "owner1",  // 所有者アカウントID
    vec!["owner1".to_string(), "signer2".to_string(), "signer3".to_string()],  // 署名者リスト
    2,  // 必要署名数
).unwrap();

println!("ウォレットID: {}", wallet.id);
```

### トランザクションの作成

```rust
use shardx::multisig::{Operation, TransactionData};

// トランザクションデータを作成
let tx_data = TransactionData {
    operation: Operation::Transfer {
        to: "recipient".to_string(),
        amount: 100.0,
        token_id: None,  // ネイティブトークンの場合はNone
    },
    memo: Some("開発費用".to_string()),
    timestamp: chrono::Utc::now().timestamp(),
};

let tx_data_bytes = serde_json::to_vec(&tx_data).unwrap();

// マルチシグトランザクションを作成
let tx = multisig_manager.create_transaction(
    &wallet.id,
    "owner1",  // 作成者アカウントID
    tx_data_bytes,
).unwrap();

println!("トランザクションID: {}", tx.id);
```

### トランザクションへの署名

```rust
// 署名を作成（実際の実装では暗号署名を使用）
let signature = vec![1, 2, 3];  // ダミー署名

// トランザクションに署名
let signed_tx = multisig_manager.sign_transaction(
    &tx.id,
    "owner1",  // 署名者アカウントID
    signature,
).unwrap();

// 署名状態を確認
println!("署名数: {}/{}", 
    signed_tx.signatures.values().filter(|sig| sig.status == SignatureStatus::Signed).count(),
    signed_tx.required_signatures
);
```

### トランザクションの拒否

```rust
// トランザクションを拒否
let rejected_tx = multisig_manager.reject_transaction(
    &tx.id,
    "signer2",  // 拒否者アカウントID
).unwrap();

println!("トランザクション状態: {:?}", rejected_tx.status);
```

## ウェブインターフェース

ShardXは、マルチシグウォレットを管理するための直感的なウェブインターフェースも提供しています。以下の機能が利用可能です：

- ウォレットの作成と管理
- トランザクションの作成と署名
- 署名状態の追跡
- トランザクション履歴の表示

ウェブインターフェースにアクセスするには、ShardXノードを起動し、ブラウザで`http://localhost:PORT/multisig_wallet.html`にアクセスしてください。

## セキュリティ上の注意点

- 所有者アカウントの秘密鍵は安全に保管してください
- 必要署名数は署名者数の半分以上に設定することをお勧めします
- 重要なトランザクションには複数の署名者による確認を徹底してください
- 定期的に署名者リストを見直し、必要に応じて更新してください

## APIリファレンス

詳細なAPIリファレンスについては、[MultisigManager API](../api/multisig.md)を参照してください。