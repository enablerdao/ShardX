# クロスシャードトランザクション

ShardXのクロスシャードトランザクション機能は、複数のシャードにまたがるトランザクションを一貫性を保ちながら処理する機能を提供します。これにより、シャード間の境界を意識することなく、シームレスなトランザクション処理が可能になります。

## 主な特徴

- **2フェーズコミット**: 複数シャード間のトランザクション一貫性を保証
- **原子性**: トランザクションは全シャードで成功するか、全シャードで失敗するかのいずれか
- **コーディネーター**: 各トランザクションにコーディネーターシャードが割り当てられ、全体の調整を担当
- **障害耐性**: 一部のシャードに障害が発生しても、システム全体の整合性を維持
- **スケーラビリティ**: シャード数が増えても効率的に動作

## 動作原理

クロスシャードトランザクションは、以下の2フェーズコミットプロトコルで処理されます：

1. **準備フェーズ**:
   - コーディネーターシャードが各参加シャードに準備リクエストを送信
   - 各シャードはトランザクションを検証し、準備完了または拒否の応答を返す
   - すべてのシャードが準備完了を返した場合のみ、コミットフェーズに進む

2. **コミットフェーズ**:
   - コーディネーターシャードが各参加シャードにコミットリクエストを送信
   - 各シャードはトランザクションをコミットし、完了応答を返す
   - すべてのシャードがコミット完了を返すと、トランザクションは完了

3. **アボートフェーズ** (必要な場合):
   - いずれかのシャードが準備フェーズで拒否した場合、コーディネーターはアボートリクエストを送信
   - 各シャードはトランザクションをアボートし、変更をロールバック

## 使用方法

### クロスシャードコーディネーターの初期化

```rust
use shardx::cross_shard::CrossShardCoordinator;
use shardx::sharding::ShardManager;
use std::sync::Arc;
use tokio::sync::mpsc;

// シャードマネージャーを初期化
let shard_manager = Arc::new(ShardManager::new(10));

// メッセージングチャネルを作成
let (tx, rx) = mpsc::channel(100);

// クロスシャードコーディネーターを初期化
let coordinator = Arc::new(CrossShardCoordinator::new(
    0, // 現在のシャードID
    shard_manager.clone(),
    tx,
    rx,
));

// メッセージ処理ループを開始
coordinator.start_message_processing().await.unwrap();
```

### クロスシャードトランザクションの開始

```rust
use shardx::transaction::Transaction;

// トランザクションを作成
let transaction = Transaction {
    id: uuid::Uuid::new_v4().to_string(),
    parent_ids: vec![],
    timestamp: 12345,
    payload: vec![1, 2, 3], // 複数のシャードに影響するペイロード
    signature: vec![4, 5, 6],
    status: TransactionStatus::Pending,
    created_at: chrono::Utc::now(),
};

// クロスシャードトランザクションを開始
let tx_id = coordinator.start_transaction(transaction).await.unwrap();
println!("トランザクションID: {}", tx_id);
```

### トランザクション状態の確認

```rust
// トランザクションの状態を確認
let status = coordinator.get_transaction_status(&tx_id).unwrap();
println!("トランザクション状態: {:?}", status);

// トランザクションの詳細を取得
let details = coordinator.get_transaction_details(&tx_id).unwrap();
println!("参加シャード数: {}", details.participant_shards.len());
println!("準備完了シャード数: {}", details.prepared_shards.values().filter(|&prepared| *prepared).count());
println!("コミット完了シャード数: {}", details.committed_shards.values().filter(|&committed| *committed).count());
```

## 実装上の注意点

### 1. シャード識別

トランザクションが影響するシャードを正確に特定することが重要です。ShardXでは以下の方法でシャードを特定します：

```rust
fn identify_affected_shards(&self, transaction: &Transaction) -> Result<Vec<ShardId>, Error> {
    // トランザクションの内容に基づいて影響するシャードを特定
    let mut affected_shards = HashSet::new();
    
    // 現在のシャードを追加
    affected_shards.insert(self.current_shard);
    
    // ペイロードの内容に基づいてシャードを決定
    // 実際の実装ではより複雑なロジックを使用
    
    Ok(affected_shards.into_iter().collect())
}
```

### 2. 障害処理

シャードの障害に対処するためのタイムアウトと再試行メカニズムを実装することが重要です：

```rust
// タイムアウト付きで準備完了を待機
async fn wait_for_preparation(&self, tx_id: &str, timeout: Duration) -> Result<bool, Error> {
    let start = Instant::now();
    
    while start.elapsed() < timeout {
        let all_prepared = {
            let transactions = self.transactions.read().unwrap();
            let tx = transactions.get(tx_id).ok_or_else(|| Error::TransactionNotFound(tx_id.to_string()))?;
            tx.all_prepared()
        };
        
        if all_prepared {
            return Ok(true);
        }
        
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // タイムアウト - トランザクションをアボート
    self.start_abort_phase(tx_id).await?;
    Ok(false)
}
```

## パフォーマンスの考慮事項

- クロスシャードトランザクションは単一シャードトランザクションよりもオーバーヘッドが大きい
- 可能な限り、トランザクションが影響するシャード数を最小限に抑えることが望ましい
- シャード数が多い場合、コーディネーターの負荷が高くなる可能性がある
- 大規模なクロスシャードトランザクションは小さなトランザクションに分割することを検討

## APIリファレンス

詳細なAPIリファレンスについては、[CrossShardCoordinator API](../api/cross_shard.md)を参照してください。