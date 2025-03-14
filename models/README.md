# HyperFlux.io AI モデル

このディレクトリには、HyperFlux.ioで使用されるAIモデルが格納されています。

## モデルの概要

### priority.onnx

トランザクションの優先順位付けに使用されるONNXモデル。以下の特徴量に基づいて優先スコアを計算します：

- トランザクションサイズ
- 親トランザクション数
- タイムスタンプ
- 手数料
- 緊急性フラグ

### 環境別モデル

- `priority_dev.onnx`: 開発環境用の軽量モデル
- `priority_test.onnx`: テスト環境用の決定論的モデル
- `priority_prod.onnx`: 本番環境用の最適化モデル

## モデルのトレーニング

モデルは以下の手順でトレーニングされています：

1. 過去のトランザクションデータを収集
2. 特徴量エンジニアリングを実施
3. PyTorchでモデルをトレーニング
4. ONNXフォーマットにエクスポート
5. 推論パフォーマンスを最適化

## 使用方法

```rust
use tract_onnx::prelude::*;

fn prioritize_tx(tx_data: Vec<f32>) -> f32 {
    let model = tract_onnx::onnx()
        .model_for_path("models/priority.onnx")
        .unwrap()
        .into_optimized()
        .unwrap()
        .into_runnable()
        .unwrap();
    
    let input = tvec!(tx_data.into());
    let result = model.run(input).unwrap();
    result[0].to_scalar::<f32>().unwrap() // 優先スコア
}
```

## モデルの更新

モデルは定期的に再トレーニングされ、パフォーマンスが向上します。更新されたモデルは自動的にノードにデプロイされます。

## ライセンス

モデルは MIT ライセンスの下で提供されています。