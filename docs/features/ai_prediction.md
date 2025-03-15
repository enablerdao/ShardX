# AIによる取引予測機能

ShardXのAIによる取引予測機能は、過去のトランザクションデータと市場動向に基づいて将来の価格や取引量を予測する機能を提供します。この機能により、ユーザーはより情報に基づいた取引判断を行うことができます。

## 主な特徴

- **価格予測**: 過去のデータに基づいて将来の価格を予測
- **信頼区間**: 予測の不確実性を表す上限と下限を提供
- **複数の時間枠**: 1時間、1日、1週間、1ヶ月の予測をサポート
- **複数の取引ペア**: 様々な取引ペアに対応
- **精度評価**: 予測の精度を継続的に評価し、モデルを改善
- **特徴重要度**: 予測に影響を与える要因の重要度を表示

## 使用方法

### 予測マネージャーの初期化

```rust
use shardx::ai::trade_prediction::{TradePredictionManager, PredictionPeriod};
use shardx::dex::TradingPair;

// 予測マネージャーを初期化
let prediction_manager = TradePredictionManager::new();

// 取引ペアを作成
let btc_usd = TradingPair {
    base: "BTC".to_string(),
    quote: "USD".to_string(),
};

// 予測モデルを追加
prediction_manager.add_model(btc_usd.clone(), 1000).unwrap();
```

### 取引データの更新

```rust
use shardx::dex::Trade;

// 取引データを更新
let trade = Trade {
    id: uuid::Uuid::new_v4().to_string(),
    buy_order_id: "buy1".to_string(),
    sell_order_id: "sell1".to_string(),
    pair: btc_usd.clone(),
    price: 42000.0,
    amount: 1.0,
    executed_at: chrono::Utc::now(),
};

prediction_manager.update_from_trade(&trade);
```

### 価格予測の実行

```rust
// 1時間後の価格を予測
let hour_prediction = prediction_manager.predict_price(&btc_usd, PredictionPeriod::Hour).unwrap();
println!("1時間後の予測価格: {}", hour_prediction.predicted_price);
println!("予測範囲: {} - {}", hour_prediction.lower_bound, hour_prediction.upper_bound);
println!("信頼度: {}", hour_prediction.confidence);

// 1日後の価格を予測
let day_prediction = prediction_manager.predict_price(&btc_usd, PredictionPeriod::Day).unwrap();
println!("1日後の予測価格: {}", day_prediction.predicted_price);
```

### すべての取引ペアの予測を取得

```rust
// すべての取引ペアの予測を取得
let all_predictions = prediction_manager.get_all_predictions(PredictionPeriod::Hour);

for (pair_str, prediction_result) in all_predictions {
    match prediction_result {
        Ok(prediction) => {
            println!("{}: 予測価格 = {}", pair_str, prediction.predicted_price);
        },
        Err(e) => {
            println!("{}: 予測エラー = {}", pair_str, e);
        }
    }
}
```

### 予測精度の評価

```rust
// 予測精度を評価
if let Some(accuracy) = prediction_manager.evaluate_prediction_accuracy(&btc_usd) {
    println!("予測精度: {}%", accuracy * 100.0);
}
```

## 予測モデルのカスタマイズ

ShardXは、ONNXフォーマットの外部モデルをロードする機能をサポートしています。これにより、独自の機械学習モデルを使用することができます。

```rust
// ONNXモデルをロード
prediction_manager.load_model(&btc_usd, "/path/to/model.onnx").unwrap();
```

## ウェブインターフェース

ShardXは、AIによる取引予測を視覚化するための直感的なウェブインターフェースも提供しています。以下の機能が利用可能です：

- 予測チャートの表示
- 複数の時間枠での予測
- 予測精度の履歴
- 特徴重要度の表示
- カスタム予測の実行

ウェブインターフェースにアクセスするには、ShardXノードを起動し、ブラウザで`http://localhost:PORT/ai_prediction.html`にアクセスしてください。

## 技術的詳細

### 予測モデル

ShardXの予測モデルは以下の特徴を使用します：

1. **価格履歴**: 過去の価格データ
2. **移動平均**: 短期、中期、長期の移動平均
3. **ボラティリティ**: 価格変動の大きさ
4. **取引量**: 取引の頻度と量
5. **時間情報**: 時間帯、曜日、日付などの時間的特徴

### 予測アルゴリズム

ONNXモデルが利用できない場合、ShardXは以下の方法で予測を行います：

1. **過去の価格変化率を計算**
2. **平均変化率と標準偏差を計算**
3. **予測期間に応じて変化率を調整**
4. **現在価格に変化率を適用して予測価格を計算**
5. **標準偏差を使用して信頼区間を計算**

## 注意事項

- 予測は参考情報であり、実際の結果と異なる場合があります
- 市場の急激な変動や予期せぬイベントは予測に反映されない場合があります
- 予測精度は取引ペアや時間枠によって異なります
- 十分なデータがない場合、予測の信頼性は低下します

## APIリファレンス

詳細なAPIリファレンスについては、[TradePredictionManager API](../api/ai_prediction.md)を参照してください。