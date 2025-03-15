# ShardX クロスチェーン機能

ShardXは、異なるブロックチェーン間での相互運用性を実現するためのクロスチェーン機能を提供します。この機能により、ShardXと他のブロックチェーン（Ethereum、Solana、Polkadotなど）との間でのトークンの移動やメッセージの交換が可能になります。

## 主な機能

### 1. クロスチェーンブリッジ

異なるブロックチェーン間での資産移動を可能にするブリッジ機能を提供します。

```rust
// ブリッジの初期化
let bridge_config = BridgeConfig {
    id: "shardx-ethereum-bridge",
    source_chain: ChainType::ShardX,
    target_chain: ChainType::Ethereum,
    // 他の設定...
};

let bridge = CrossChainBridge::new(bridge_config, message_tx, message_rx);
bridge.initialize().await?;

// トランザクションの送信
let tx_id = bridge.start_transaction(transaction).await?;
```

### 2. クロスチェーンメッセージング

異なるブロックチェーン間でのメッセージ交換プロトコルを提供します。

```rust
// メッセージの作成と送信
let message = CrossChainMessage::new(
    transaction_id,
    ChainType::ShardX,
    ChainType::Ethereum,
    MessageType::TransactionRequest,
    Some(data),
);

message_sender.send(message).await?;
```

### 3. クロスチェーントランザクション

複数のブロックチェーンにまたがる取引の実行と検証を可能にします。

```rust
// クロスチェーントランザクションの作成
let cross_tx = CrossChainTransaction::new(
    original_transaction,
    ChainType::ShardX,
    ChainType::Ethereum,
);

// トランザクションの状態を確認
if cross_tx.is_completed() && cross_tx.is_successful() {
    // 成功時の処理
}
```

## サポートされているブロックチェーン

現在、以下のブロックチェーンとの相互運用性をサポートしています：

- **ShardX**: 内部チェーン
- **Ethereum**: EVM互換チェーン
- **Solana**: 高性能チェーン
- **Polkadot**: クロスチェーン特化チェーン
- **Cosmos**: インターブロックチェーン通信プロトコル

## 使用例

### Ethereumとの資産移動

```rust
// ShardXからEthereumへのトークン送信
let transaction = Transaction {
    from: "shardx_address",
    to: "ethereum_address",
    amount: "1.0",
    // 他のフィールド...
};

let bridge = get_ethereum_bridge();
let tx_id = bridge.start_transaction(transaction).await?;

// トランザクションの状態を確認
let status = bridge.get_transaction_status(&tx_id)?;
```

### Solanaとのメッセージ交換

```rust
// ShardXからSolanaへのメッセージ送信
let message = CrossChainMessage::new(
    transaction_id,
    ChainType::ShardX,
    ChainType::Solana,
    MessageType::StatusRequest,
    None,
);

let bridge = get_solana_bridge();
bridge.send_message(message).await?;
```

## セキュリティ

クロスチェーン機能は、以下のセキュリティ機能を備えています：

1. **トランザクション証明**: 各トランザクションには暗号学的証明が付与され、改ざんを防止します。
2. **マルチシグ検証**: 重要な操作には複数の署名が必要です。
3. **タイムアウト機構**: 長時間応答のないトランザクションは自動的にタイムアウトします。
4. **リトライメカニズム**: 一時的な障害に対して自動的にリトライします。

## 設定

ブリッジの設定は、`BridgeConfig`構造体で指定します：

```rust
let bridge_config = BridgeConfig {
    id: "shardx-ethereum-bridge",
    name: "ShardX-Ethereum Bridge",
    source_chain: ChainType::ShardX,
    target_chain: ChainType::Ethereum,
    source_endpoint: "http://localhost:8545",
    target_endpoint: "https://mainnet.infura.io/v3/your-project-id",
    source_contract: None,
    target_contract: Some("0x1234567890123456789012345678901234567890"),
    max_transaction_size: 1024 * 1024, // 1MB
    max_message_size: 1024 * 1024, // 1MB
    confirmation_blocks: 12,
    timeout_sec: 60,
    retry_count: 3,
    retry_interval_sec: 10,
    fee_settings: FeeSetting {
        base_fee: 0.001,
        fee_per_byte: 0.0001,
        fee_currency: "ETH",
        min_fee: 0.001,
        max_fee: Some(0.1),
    },
};
```

## デモ

クロスチェーン機能のデモは、以下のコマンドで実行できます：

```bash
cargo run --bin cross_chain_bridge_demo
```

## 今後の開発予定

1. **追加チェーンのサポート**: Bitcoin、Cardano、Avalancheなどのサポート追加
2. **クロスチェーンスマートコントラクト**: 複数チェーンにまたがるスマートコントラクトの実行
3. **分散型オラクル統合**: チェーン外データの安全な取得
4. **クロスチェーンDEX**: 異なるチェーン間での分散型取引所機能

## 関連ドキュメント

- [API リファレンス](../api/cross_chain.md)
- [開発者ガイド](../developers/cross_chain.md)
- [セキュリティモデル](../security/cross_chain.md)