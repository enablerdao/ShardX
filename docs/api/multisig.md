# マルチシグウォレット API リファレンス

このドキュメントでは、ShardXのマルチシグウォレット機能に関するAPIエンドポイントについて説明します。

## 基本情報

- **ベースURL**: `http://localhost:54868/api/v1/multisig`
- **認証**: APIキーによる認証（ヘッダー: `X-API-Key`）
- **レスポンス形式**: JSON

## エンドポイント一覧

### ウォレット操作

#### ウォレット一覧の取得

```
GET /wallets
```

**クエリパラメータ**:
- `account_id` (オプション): 特定のアカウントに関連するウォレットのみを取得

**レスポンス例**:
```json
{
  "wallets": [
    {
      "id": "wallet_123456",
      "name": "組織資金ウォレット",
      "owner_id": "account_123",
      "signers": ["account_123", "account_456", "account_789"],
      "required_signatures": 2,
      "balance": 1000.0,
      "token_balances": {
        "TOKEN_A": 500.0,
        "TOKEN_B": 200.0
      },
      "created_at": "2023-01-01T12:00:00Z"
    }
  ]
}
```

#### ウォレットの作成

```
POST /wallets
```

**リクエストボディ**:
```json
{
  "name": "組織資金ウォレット",
  "owner_id": "account_123",
  "signers": ["account_123", "account_456", "account_789"],
  "required_signatures": 2
}
```

**レスポンス例**:
```json
{
  "wallet": {
    "id": "wallet_123456",
    "name": "組織資金ウォレット",
    "owner_id": "account_123",
    "signers": ["account_123", "account_456", "account_789"],
    "required_signatures": 2,
    "balance": 0.0,
    "token_balances": {},
    "created_at": "2023-01-01T12:00:00Z"
  }
}
```

#### ウォレット詳細の取得

```
GET /wallets/{wallet_id}
```

**パスパラメータ**:
- `wallet_id`: ウォレットID

**レスポンス例**:
```json
{
  "wallet": {
    "id": "wallet_123456",
    "name": "組織資金ウォレット",
    "owner_id": "account_123",
    "signers": ["account_123", "account_456", "account_789"],
    "required_signatures": 2,
    "balance": 1000.0,
    "token_balances": {
      "TOKEN_A": 500.0,
      "TOKEN_B": 200.0
    },
    "created_at": "2023-01-01T12:00:00Z"
  }
}
```

### トランザクション操作

#### トランザクション一覧の取得

```
GET /wallets/{wallet_id}/transactions
```

**パスパラメータ**:
- `wallet_id`: ウォレットID

**クエリパラメータ**:
- `status` (オプション): トランザクションのステータスでフィルタリング（`pending`, `confirmed`, `rejected`）
- `limit` (オプション): 取得するトランザクションの最大数（デフォルト: 20）
- `offset` (オプション): ページネーション用のオフセット（デフォルト: 0）

**レスポンス例**:
```json
{
  "transactions": [
    {
      "id": "tx_123456",
      "wallet_id": "wallet_123456",
      "creator_id": "account_123",
      "required_signatures": 2,
      "signatures": {
        "account_123": {
          "signer_id": "account_123",
          "status": "signed",
          "signed_at": "2023-01-01T12:05:00Z"
        },
        "account_456": {
          "signer_id": "account_456",
          "status": "pending",
          "signed_at": null
        }
      },
      "transaction_data": {
        "operation": "transfer",
        "to": "account_xyz",
        "amount": 100.0,
        "token_id": null,
        "memo": "開発費用"
      },
      "status": "pending",
      "created_at": "2023-01-01T12:05:00Z",
      "executed_at": null
    }
  ],
  "total": 1
}
```

#### トランザクションの作成

```
POST /wallets/{wallet_id}/transactions
```

**パスパラメータ**:
- `wallet_id`: ウォレットID

**リクエストボディ**:
```json
{
  "creator_id": "account_123",
  "transaction_data": {
    "operation": "transfer",
    "to": "account_xyz",
    "amount": 100.0,
    "token_id": null,
    "memo": "開発費用"
  }
}
```

**レスポンス例**:
```json
{
  "transaction": {
    "id": "tx_123456",
    "wallet_id": "wallet_123456",
    "creator_id": "account_123",
    "required_signatures": 2,
    "signatures": {
      "account_123": {
        "signer_id": "account_123",
        "status": "pending",
        "signed_at": null
      }
    },
    "transaction_data": {
      "operation": "transfer",
      "to": "account_xyz",
      "amount": 100.0,
      "token_id": null,
      "memo": "開発費用"
    },
    "status": "pending",
    "created_at": "2023-01-01T12:05:00Z",
    "executed_at": null
  }
}
```

#### トランザクション詳細の取得

```
GET /transactions/{tx_id}
```

**パスパラメータ**:
- `tx_id`: トランザクションID

**レスポンス例**:
```json
{
  "transaction": {
    "id": "tx_123456",
    "wallet_id": "wallet_123456",
    "creator_id": "account_123",
    "required_signatures": 2,
    "signatures": {
      "account_123": {
        "signer_id": "account_123",
        "status": "signed",
        "signed_at": "2023-01-01T12:05:00Z"
      },
      "account_456": {
        "signer_id": "account_456",
        "status": "pending",
        "signed_at": null
      }
    },
    "transaction_data": {
      "operation": "transfer",
      "to": "account_xyz",
      "amount": 100.0,
      "token_id": null,
      "memo": "開発費用"
    },
    "status": "pending",
    "created_at": "2023-01-01T12:05:00Z",
    "executed_at": null
  }
}
```

#### トランザクションへの署名

```
POST /transactions/{tx_id}/sign
```

**パスパラメータ**:
- `tx_id`: トランザクションID

**リクエストボディ**:
```json
{
  "signer_id": "account_456",
  "signature": "base64_encoded_signature_data"
}
```

**レスポンス例**:
```json
{
  "transaction": {
    "id": "tx_123456",
    "wallet_id": "wallet_123456",
    "creator_id": "account_123",
    "required_signatures": 2,
    "signatures": {
      "account_123": {
        "signer_id": "account_123",
        "status": "signed",
        "signed_at": "2023-01-01T12:05:00Z"
      },
      "account_456": {
        "signer_id": "account_456",
        "status": "signed",
        "signed_at": "2023-01-01T12:10:00Z"
      }
    },
    "transaction_data": {
      "operation": "transfer",
      "to": "account_xyz",
      "amount": 100.0,
      "token_id": null,
      "memo": "開発費用"
    },
    "status": "confirmed",
    "created_at": "2023-01-01T12:05:00Z",
    "executed_at": "2023-01-01T12:10:00Z"
  }
}
```

#### トランザクションの拒否

```
POST /transactions/{tx_id}/reject
```

**パスパラメータ**:
- `tx_id`: トランザクションID

**リクエストボディ**:
```json
{
  "signer_id": "account_456"
}
```

**レスポンス例**:
```json
{
  "transaction": {
    "id": "tx_123456",
    "wallet_id": "wallet_123456",
    "creator_id": "account_123",
    "required_signatures": 2,
    "signatures": {
      "account_123": {
        "signer_id": "account_123",
        "status": "signed",
        "signed_at": "2023-01-01T12:05:00Z"
      },
      "account_456": {
        "signer_id": "account_456",
        "status": "rejected",
        "signed_at": "2023-01-01T12:10:00Z"
      }
    },
    "transaction_data": {
      "operation": "transfer",
      "to": "account_xyz",
      "amount": 100.0,
      "token_id": null,
      "memo": "開発費用"
    },
    "status": "rejected",
    "created_at": "2023-01-01T12:05:00Z",
    "executed_at": null
  }
}
```

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
| `internal_error` | サーバー内部エラー |

## 操作タイプ

マルチシグトランザクションでは、以下の操作タイプがサポートされています：

| 操作タイプ | 説明 | 必須パラメータ |
|------------|------|----------------|
| `transfer` | 資金の送金 | `to`, `amount`, `token_id` (オプション) |
| `add_signer` | 署名者の追加 | `signer_id` |
| `remove_signer` | 署名者の削除 | `signer_id` |
| `change_required` | 必要署名数の変更 | `required` |

## セキュリティに関する注意事項

- すべてのAPIリクエストはHTTPS経由で行ってください
- APIキーは安全に保管し、定期的に更新してください
- 署名の検証は必ずサーバーサイドで行ってください
- 重要なトランザクションには複数の署名者による確認を徹底してください