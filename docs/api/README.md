# HyperFlux.io API ドキュメント

このドキュメントでは、HyperFlux.io APIの使用方法について説明します。

## 基本情報

- **ベースURL**: `http://localhost:54868` (ローカル開発環境)
- **本番URL**: `https://api.hyperflux.io` (本番環境)
- **認証**: APIキーベース (ヘッダー: `X-API-Key`)
- **レスポンス形式**: JSON
- **エラーコード**: 標準HTTPステータスコード + 詳細なエラーメッセージ

## エンドポイント一覧

### ノード情報

#### GET /info

ノードの基本情報を取得します。

**リクエスト例**:
```bash
curl -X GET http://localhost:54868/info
```

**レスポンス例**:
```json
{
  "node_id": "node_01a2b3c4",
  "version": "1.0.0",
  "uptime": 3600,
  "peers": 5,
  "current_tps": 42156,
  "shard_count": 256,
  "confirmed_transactions": 1284301
}
```

### トランザクション

#### POST /tx/create

新しいトランザクションを作成します。

**リクエスト例**:
```bash
curl -X POST http://localhost:54868/tx/create \
  -H "Content-Type: application/json" \
  -d '{
    "parent_ids": ["tx_123456", "tx_789012"],
    "payload": "Hello, HyperFlux!",
    "signature": "0x1a2b3c4d5e6f..."
  }'
```

**レスポンス例**:
```json
{
  "tx_id": "tx_abcdef123456",
  "status": "pending",
  "timestamp": 1647123456789
}
```

#### GET /tx/{tx_id}

トランザクションの詳細を取得します。

**リクエスト例**:
```bash
curl -X GET http://localhost:54868/tx/tx_abcdef123456
```

**レスポンス例**:
```json
{
  "tx_id": "tx_abcdef123456",
  "parent_ids": ["tx_123456", "tx_789012"],
  "payload": "Hello, HyperFlux!",
  "signature": "0x1a2b3c4d5e6f...",
  "status": "confirmed",
  "timestamp": 1647123456789,
  "confirmation_time": 1647123457012,
  "shard_id": 42
}
```

#### GET /tx/list

最近のトランザクションのリストを取得します。

**パラメータ**:
- `limit` (オプション): 取得するトランザクション数 (デフォルト: 10, 最大: 100)
- `offset` (オプション): ページネーションオフセット (デフォルト: 0)
- `status` (オプション): フィルタリングするステータス (pending, confirmed, all)

**リクエスト例**:
```bash
curl -X GET "http://localhost:54868/tx/list?limit=5&status=confirmed"
```

**レスポンス例**:
```json
{
  "transactions": [
    {
      "tx_id": "tx_abcdef123456",
      "payload": "Hello, HyperFlux!",
      "status": "confirmed",
      "timestamp": 1647123456789
    },
    {
      "tx_id": "tx_fedcba654321",
      "payload": "Another transaction",
      "status": "confirmed",
      "timestamp": 1647123456123
    }
    // ... 他のトランザクション
  ],
  "total": 1284301,
  "limit": 5,
  "offset": 0
}
```

### ウォレット

#### POST /wallet/create

新しいウォレットを作成します。

**リクエスト例**:
```bash
curl -X POST http://localhost:54868/wallet/create \
  -H "Content-Type: application/json" \
  -d '{
    "password": "secure_password_123"
  }'
```

**レスポンス例**:
```json
{
  "wallet_id": "wallet_123abc",
  "address": "0x1a2b3c4d5e6f...",
  "public_key": "0x7g8h9i0j...",
  "created_at": 1647123456789
}
```

#### GET /wallet/{wallet_id}/balance

ウォレットの残高を取得します。

**リクエスト例**:
```bash
curl -X GET http://localhost:54868/wallet/wallet_123abc/balance
```

**レスポンス例**:
```json
{
  "wallet_id": "wallet_123abc",
  "address": "0x1a2b3c4d5e6f...",
  "balance": 100.5,
  "pending_balance": 10.25,
  "last_updated": 1647123456789
}
```

### シャーディング

#### GET /shards/info

シャーディング情報を取得します。

**リクエスト例**:
```bash
curl -X GET http://localhost:54868/shards/info
```

**レスポンス例**:
```json
{
  "total_shards": 256,
  "active_shards": 256,
  "shard_distribution": {
    "shard_0": 5012,
    "shard_1": 4987,
    // ... 他のシャード
  },
  "scaling_status": "stable",
  "last_scaling_event": 1647123000000
}
```

## エラーレスポンス

エラーが発生した場合、APIは適切なHTTPステータスコードと詳細なエラーメッセージを返します。

**エラーレスポンス例**:
```json
{
  "error": {
    "code": "tx_not_found",
    "message": "Transaction with ID tx_invalid123 not found",
    "status": 404
  }
}
```

## レート制限

APIにはレート制限があります。制限を超えると、429 Too Many Requestsステータスコードが返されます。

- 認証なし: 60リクエスト/分
- 認証あり: 1000リクエスト/分

## SDKとサンプルコード

各言語向けのSDKとサンプルコードは以下のリポジトリで提供されています：

- [JavaScript SDK](https://github.com/enablerdao/hyperflux-js)
- [Python SDK](https://github.com/enablerdao/hyperflux-py)
- [Rust SDK](https://github.com/enablerdao/hyperflux-rs)

## ウェブソケットAPI

リアルタイム更新を受信するためのウェブソケットAPIも提供しています。

**接続URL**: `ws://localhost:54869` (ローカル開発環境)

**イベントタイプ**:
- `tx_created`: 新しいトランザクションが作成されたとき
- `tx_confirmed`: トランザクションが確認されたとき
- `shard_scaled`: シャード数が変更されたとき

**サブスクリプション例**:
```javascript
const socket = new WebSocket('ws://localhost:54869');

socket.onopen = () => {
  socket.send(JSON.stringify({
    action: 'subscribe',
    channels: ['tx_confirmed', 'shard_scaled']
  }));
};

socket.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Received event:', data);
};
```