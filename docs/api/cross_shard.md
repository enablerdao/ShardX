# クロスシャードトランザクション API リファレンス

このドキュメントでは、ShardXのクロスシャードトランザクション機能に関するAPIエンドポイントについて説明します。

## 基本情報

- **ベースURL**: `http://localhost:54868/api/v1/cross-shard`
- **認証**: APIキーによる認証（ヘッダー: `X-API-Key`）
- **レスポンス形式**: JSON

## エンドポイント一覧

### トランザクション操作

#### クロスシャードトランザクションの作成

```
POST /transactions
```

**リクエストボディ**:
```json
{
  "transaction": {
    "parent_ids": ["tx_123", "tx_456"],
    "payload": "base64_encoded_payload",
    "signature": "base64_encoded_signature"
  }
}
```

**レスポンス例**:
```json
{
  "transaction_id": "tx_789",
  "status": "pending",
  "coordinator_shard": 3,
  "participant_shards": [3, 5, 7],
  "created_at": "2023-01-01T12:00:00Z"
}
```

#### トランザクション状態の取得

```
GET /transactions/{tx_id}
```

**パスパラメータ**:
- `tx_id`: トランザクションID

**レスポンス例**:
```json
{
  "transaction_id": "tx_789",
  "status": "committed",
  "coordinator_shard": 3,
  "participant_shards": [3, 5, 7],
  "prepared_shards": [3, 5, 7],
  "committed_shards": [3, 5, 7],
  "created_at": "2023-01-01T12:00:00Z",
  "completed_at": "2023-01-01T12:00:05Z"
}
```

#### トランザクション履歴の取得

```
GET /transactions
```

**クエリパラメータ**:
- `status` (オプション): トランザクションのステータスでフィルタリング（`pending`, `preparing`, `prepared`, `committing`, `committed`, `aborting`, `aborted`）
- `shard_id` (オプション): 特定のシャードに関連するトランザクションのみを取得
- `limit` (オプション): 取得するトランザクションの最大数（デフォルト: 20）
- `offset` (オプション): ページネーション用のオフセット（デフォルト: 0）

**レスポンス例**:
```json
{
  "transactions": [
    {
      "transaction_id": "tx_789",
      "status": "committed",
      "coordinator_shard": 3,
      "participant_shards": [3, 5, 7],
      "created_at": "2023-01-01T12:00:00Z",
      "completed_at": "2023-01-01T12:00:05Z"
    },
    {
      "transaction_id": "tx_790",
      "status": "pending",
      "coordinator_shard": 2,
      "participant_shards": [2, 4],
      "created_at": "2023-01-01T12:01:00Z",
      "completed_at": null
    }
  ],
  "total": 2
}
```

### シャード情報

#### シャード一覧の取得

```
GET /shards
```

**レスポンス例**:
```json
{
  "shards": [
    {
      "id": 0,
      "status": "active",
      "transaction_count": 1250,
      "load": 0.45
    },
    {
      "id": 1,
      "status": "active",
      "transaction_count": 980,
      "load": 0.32
    }
  ],
  "total_shards": 256,
  "active_shards": 256
}
```

#### シャード詳細の取得

```
GET /shards/{shard_id}
```

**パスパラメータ**:
- `shard_id`: シャードID

**レスポンス例**:
```json
{
  "shard": {
    "id": 5,
    "status": "active",
    "transaction_count": 1250,
    "pending_transactions": 15,
    "load": 0.45,
    "connected_nodes": 8,
    "created_at": "2023-01-01T00:00:00Z",
    "last_activity": "2023-01-01T12:05:00Z"
  }
}
```

### メッセージング

#### メッセージの送信

```
POST /messages
```

**リクエストボディ**:
```json
{
  "message": {
    "transaction_id": "tx_789",
    "from_shard": 3,
    "to_shard": 5,
    "message_type": "prepare_request",
    "transaction_data": "base64_encoded_data"
  }
}
```

**レスポンス例**:
```json
{
  "message_id": "msg_123",
  "status": "sent",
  "sent_at": "2023-01-01T12:00:01Z"
}
```

#### メッセージ状態の取得

```
GET /messages/{message_id}
```

**パスパラメータ**:
- `message_id`: メッセージID

**レスポンス例**:
```json
{
  "message": {
    "id": "msg_123",
    "transaction_id": "tx_789",
    "from_shard": 3,
    "to_shard": 5,
    "message_type": "prepare_request",
    "status": "delivered",
    "sent_at": "2023-01-01T12:00:01Z",
    "delivered_at": "2023-01-01T12:00:02Z"
  }
}
```

## トランザクションステータス

クロスシャードトランザクションは、以下のステータスを持ちます：

| ステータス | 説明 |
|------------|------|
| `pending` | トランザクションが作成され、処理待ち |
| `preparing` | 準備フェーズが進行中 |
| `prepared` | すべてのシャードが準備完了 |
| `committing` | コミットフェーズが進行中 |
| `committed` | すべてのシャードがコミット完了 |
| `aborting` | アボートフェーズが進行中 |
| `aborted` | トランザクションがアボートされた |

## メッセージタイプ

クロスシャードメッセージは、以下のタイプを持ちます：

| メッセージタイプ | 説明 |
|------------------|------|
| `prepare_request` | 準備フェーズのリクエスト |
| `prepare_response` | 準備フェーズのレスポンス |
| `commit_request` | コミットフェーズのリクエスト |
| `commit_response` | コミットフェーズのレスポンス |
| `abort_request` | アボートフェーズのリクエスト |
| `abort_response` | アボートフェーズのレスポンス |

## エラーレスポンス

エラーが発生した場合、以下の形式でレスポンスが返されます：

```json
{
  "error": {
    "code": "invalid_request",
    "message": "エラーメッセージ"
  }
}
```

### エラーコード一覧

| コード | 説明 |
|--------|------|
| `invalid_request` | リクエストの形式が不正 |
| `unauthorized` | 認証エラー |
| `forbidden` | 権限エラー |
| `not_found` | リソースが見つからない |
| `validation_error` | バリデーションエラー |
| `timeout` | タイムアウトエラー |
| `duplicate_transaction` | 重複トランザクションエラー |
| `internal_error` | サーバー内部エラー |

## 実装上の注意点

### 1. シャード識別

トランザクションが影響するシャードを正確に特定することが重要です。ShardXでは以下の方法でシャードを特定します：

```rust
fn identify_affected_shards(transaction: &Transaction) -> Vec<ShardId> {
    // トランザクションの内容に基づいて影響するシャードを特定
    // 実際の実装ではより複雑なロジックを使用
}
```

### 2. 障害処理

シャードの障害に対処するためのタイムアウトと再試行メカニズムを実装することが重要です：

```rust
async fn wait_for_preparation(tx_id: &str, timeout: Duration) -> Result<bool, Error> {
    // タイムアウト付きで準備完了を待機
    // タイムアウトした場合はトランザクションをアボート
}
```

### 3. パフォーマンスの考慮事項

- クロスシャードトランザクションは単一シャードトランザクションよりもオーバーヘッドが大きい
- 可能な限り、トランザクションが影響するシャード数を最小限に抑えることが望ましい
- シャード数が多い場合、コーディネーターの負荷が高くなる可能性がある
- 大規模なクロスシャードトランザクションは小さなトランザクションに分割することを検討