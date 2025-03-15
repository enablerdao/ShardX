# AIによる取引予測 API リファレンス

このドキュメントでは、ShardXのAIによる取引予測機能に関するAPIエンドポイントについて説明します。

## 基本情報

- **ベースURL**: `http://localhost:54868/api/v1/ai`
- **認証**: APIキーによる認証（ヘッダー: `X-API-Key`）
- **レスポンス形式**: JSON

## エンドポイント一覧

### 価格予測

#### 価格予測の取得

```
GET /predictions/{pair}
```

**パスパラメータ**:
- `pair`: 取引ペア（例: `BTC/USD`）

**クエリパラメータ**:
- `period` (オプション): 予測期間（`hour`, `day`, `week`, `month`）（デフォルト: `hour`）

**レスポンス例**:
```json
{
  "prediction": {
    "pair": {
      "base": "BTC",
      "quote": "USD"
    },
    "period": "hour",
    "current_price": 42000.0,
    "predicted_price": 42850.25,
    "lower_bound": 41200.0,
    "upper_bound": 44500.0,
    "change_percent": 2.02,
    "confidence": 0.85,
    "predicted_at": "2023-01-01T12:00:00Z",
    "target_time": "2023-01-01T13:00:00Z"
  }
}
```

#### 複数取引ペアの予測取得

```
GET /predictions
```

**クエリパラメータ**:
- `period` (オプション): 予測期間（`hour`, `day`, `week`, `month`）（デフォルト: `hour`）
- `pairs` (オプション): カンマ区切りの取引ペアリスト（例: `BTC/USD,ETH/USD`）（指定しない場合はすべての取引ペア）

**レスポンス例**:
```json
{
  "predictions": {
    "BTC/USD": {
      "pair": {
        "base": "BTC",
        "quote": "USD"
      },
      "period": "hour",
      "current_price": 42000.0,
      "predicted_price": 42850.25,
      "lower_bound": 41200.0,
      "upper_bound": 44500.0,
      "change_percent": 2.02,
      "confidence": 0.85,
      "predicted_at": "2023-01-01T12:00:00Z",
      "target_time": "2023-01-01T13:00:00Z"
    },
    "ETH/USD": {
      "pair": {
        "base": "ETH",
        "quote": "USD"
      },
      "period": "hour",
      "current_price": 2800.0,
      "predicted_price": 2780.5,
      "lower_bound": 2650.0,
      "upper_bound": 2900.0,
      "change_percent": -0.7,
      "confidence": 0.72,
      "predicted_at": "2023-01-01T12:00:00Z",
      "target_time": "2023-01-01T13:00:00Z"
    }
  }
}
```

#### カスタム予測の実行

```
POST /predictions/custom
```

**リクエストボディ**:
```json
{
  "pair": "BTC/USD",
  "period": "day",
  "model": "advanced",
  "confidence_threshold": 0.7,
  "additional_features": {
    "market_sentiment": 0.8,
    "trading_volume": 1.2,
    "technical_indicators": {
      "rsi": 65,
      "macd": 0.5
    }
  }
}
```

**レスポンス例**:
```json
{
  "prediction": {
    "pair": {
      "base": "BTC",
      "quote": "USD"
    },
    "period": "day",
    "current_price": 42000.0,
    "predicted_price": 43500.75,
    "lower_bound": 41800.0,
    "upper_bound": 45200.0,
    "change_percent": 3.57,
    "confidence": 0.88,
    "predicted_at": "2023-01-01T12:00:00Z",
    "target_time": "2023-01-02T12:00:00Z",
    "model": "advanced",
    "feature_importance": {
      "price_history": 0.45,
      "volume": 0.25,
      "market_sentiment": 0.15,
      "technical_indicators": 0.15
    }
  }
}
```

### 予測精度

#### 予測精度の取得

```
GET /accuracy/{pair}
```

**パスパラメータ**:
- `pair`: 取引ペア（例: `BTC/USD`）

**クエリパラメータ**:
- `period` (オプション): 予測期間（`hour`, `day`, `week`, `month`）（デフォルト: すべての期間）
- `start_date` (オプション): 開始日（ISO 8601形式）
- `end_date` (オプション): 終了日（ISO 8601形式）

**レスポンス例**:
```json
{
  "accuracy": {
    "pair": "BTC/USD",
    "overall": 0.82,
    "by_period": {
      "hour": 0.85,
      "day": 0.78,
      "week": 0.72,
      "month": 0.65
    },
    "by_date": [
      {
        "date": "2023-01-01",
        "accuracy": 0.83
      },
      {
        "date": "2023-01-02",
        "accuracy": 0.81
      }
    ],
    "sample_size": 1250,
    "last_updated": "2023-01-03T12:00:00Z"
  }
}
```

#### 全体の予測精度の取得

```
GET /accuracy
```

**クエリパラメータ**:
- `period` (オプション): 予測期間（`hour`, `day`, `week`, `month`）（デフォルト: すべての期間）
- `start_date` (オプション): 開始日（ISO 8601形式）
- `end_date` (オプション): 終了日（ISO 8601形式）

**レスポンス例**:
```json
{
  "accuracy": {
    "overall": 0.79,
    "by_pair": {
      "BTC/USD": 0.82,
      "ETH/USD": 0.76,
      "BTC/ETH": 0.68
    },
    "by_period": {
      "hour": 0.83,
      "day": 0.77,
      "week": 0.70,
      "month": 0.62
    },
    "sample_size": 5280,
    "last_updated": "2023-01-03T12:00:00Z"
  }
}
```

### モデル管理

#### 予測モデル一覧の取得

```
GET /models
```

**レスポンス例**:
```json
{
  "models": [
    {
      "id": "standard",
      "name": "標準モデル",
      "description": "基本的な時系列予測モデル",
      "supported_pairs": ["BTC/USD", "ETH/USD", "BTC/ETH"],
      "accuracy": 0.75,
      "last_trained": "2023-01-01T00:00:00Z"
    },
    {
      "id": "advanced",
      "name": "高度モデル",
      "description": "外部データソースを含む高度な予測モデル",
      "supported_pairs": ["BTC/USD", "ETH/USD"],
      "accuracy": 0.82,
      "last_trained": "2023-01-01T00:00:00Z"
    }
  ]
}
```

#### モデルの再トレーニング

```
POST /models/{model_id}/train
```

**パスパラメータ**:
- `model_id`: モデルID

**リクエストボディ**:
```json
{
  "pairs": ["BTC/USD", "ETH/USD"],
  "training_window": "30d",
  "hyperparameters": {
    "learning_rate": 0.01,
    "epochs": 100
  }
}
```

**レスポンス例**:
```json
{
  "training_job": {
    "id": "job_123",
    "model_id": "advanced",
    "status": "in_progress",
    "progress": 0,
    "started_at": "2023-01-03T12:00:00Z",
    "estimated_completion": "2023-01-03T12:30:00Z"
  }
}
```

#### トレーニングジョブの状態取得

```
GET /training-jobs/{job_id}
```

**パスパラメータ**:
- `job_id`: トレーニングジョブID

**レスポンス例**:
```json
{
  "training_job": {
    "id": "job_123",
    "model_id": "advanced",
    "status": "completed",
    "progress": 100,
    "started_at": "2023-01-03T12:00:00Z",
    "completed_at": "2023-01-03T12:25:00Z",
    "metrics": {
      "training_loss": 0.05,
      "validation_loss": 0.08,
      "accuracy_improvement": 0.03
    }
  }
}
```

### 市場分析

#### 市場概況の取得

```
GET /market-analysis
```

**レスポンス例**:
```json
{
  "market_analysis": {
    "sentiment": "bullish",
    "confidence": 0.75,
    "key_factors": [
      {
        "factor": "buying_pressure",
        "impact": 0.6,
        "description": "買い圧力が売り圧力を上回っている"
      },
      {
        "factor": "technical_indicators",
        "impact": 0.4,
        "description": "テクニカル指標は上昇トレンドを示している"
      }
    ],
    "risk_assessment": {
      "volatility_risk": {
        "level": "medium",
        "score": 0.65
      },
      "liquidity_risk": {
        "level": "low",
        "score": 0.25
      },
      "trend_reversal_risk": {
        "level": "high",
        "score": 0.8
      }
    },
    "trading_recommendations": [
      {
        "pair": "BTC/USD",
        "action": "buy",
        "target_price": 45000.0,
        "stop_loss": 40500.0,
        "confidence": 0.85
      },
      {
        "pair": "ETH/USD",
        "action": "sell",
        "target_price": 2600.0,
        "stop_loss": 2900.0,
        "confidence": 0.65
      }
    ],
    "analyzed_at": "2023-01-03T12:00:00Z"
  }
}
```

## 予測期間

予測期間は以下の値を取ります：

| 期間 | 説明 |
|------|------|
| `hour` | 1時間後の予測 |
| `day` | 24時間後の予測 |
| `week` | 1週間後の予測 |
| `month` | 1ヶ月後の予測 |

## 予測モデル

以下の予測モデルが利用可能です：

| モデルID | 説明 |
|----------|------|
| `standard` | 基本的な時系列予測モデル |
| `advanced` | 外部データソースを含む高度な予測モデル |
| `ensemble` | 複数モデルの結果を組み合わせたアンサンブルモデル |

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
| `model_error` | モデル関連のエラー |
| `insufficient_data` | 予測に必要なデータが不足 |
| `internal_error` | サーバー内部エラー |

## 実装上の注意点

### 1. 予測の不確実性

予測は参考情報であり、実際の結果と異なる場合があります。常に予測の不確実性を考慮し、信頼区間と信頼度を提供することが重要です。

### 2. データの鮮度

予測の精度はデータの鮮度に大きく依存します。定期的にモデルを再トレーニングし、最新のデータを反映させることが重要です。

### 3. モデルの選択

異なる市場状況や取引ペアに対して、異なるモデルが最適な場合があります。複数のモデルを用意し、状況に応じて最適なモデルを選択することを検討してください。

### 4. リソース使用量

AIモデルの推論は計算リソースを消費します。高負荷時にはリソース使用量を監視し、必要に応じてスケーリングすることが重要です。