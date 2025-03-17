# ShardX パフォーマンス最適化ツール

このディレクトリには、ShardXのパフォーマンスを測定、分析、最適化するためのツールが含まれています。

## 概要

パフォーマンス最適化は、ShardXの重要な目標の一つです。このツールセットは、以下の目的で使用されます：

1. **パフォーマンスのベンチマーク**: 様々な条件下でのシステムのパフォーマンスを測定
2. **ボトルネックの特定**: システム内のパフォーマンスボトルネックを特定
3. **最適化の効果測定**: 最適化の前後でのパフォーマンスの変化を測定
4. **スケーラビリティのテスト**: 負荷の増加に対するシステムの応答を評価

## ツール一覧

### ベンチマークツール

- **benchmark.sh**: 包括的なベンチマークスイートを実行するスクリプト
- **benchmarks/**: 各コンポーネント向けのベンチマークコード
  - **transaction_benchmark.rs**: トランザクション処理のベンチマーク
  - **sharding_benchmark.rs**: シャーディング機能のベンチマーク
  - **storage_benchmark.rs**: ストレージ操作のベンチマーク

### プロファイリングツール

- **profiler.sh**: CPUとメモリのプロファイリングを実行するスクリプト

## 使用方法

### ベンチマークの実行

```bash
# すべてのベンチマークを実行
./tools/performance/benchmark.sh --all

# 特定のベンチマークのみを実行
./tools/performance/benchmark.sh --transaction
./tools/performance/benchmark.sh --sharding
./tools/performance/benchmark.sh --network

# レポートを生成
./tools/performance/benchmark.sh --report
```

### プロファイリングの実行

```bash
# CPUプロファイリングを実行
./tools/performance/profiler.sh --cpu shardx

# メモリプロファイリングを実行
./tools/performance/profiler.sh --memory shardx

# すべてのプロファイリングを実行し、フレームグラフを生成
./tools/performance/profiler.sh --all --flamegraph shardx

# プロファイリング時間を指定
./tools/performance/profiler.sh --all --time 60 shardx
```

## ベンチマーク結果の解釈

ベンチマーク結果は `target/benchmark` ディレクトリに保存されます。結果には以下の情報が含まれます：

- **スループット**: 1秒あたりに処理できるトランザクション数
- **レイテンシ**: トランザクション処理にかかる時間
- **リソース使用量**: CPU、メモリ、ディスクI/Oの使用量
- **スケーラビリティ**: 負荷の増加に対するパフォーマンスの変化

## プロファイリング結果の解釈

プロファイリング結果は `target/profile` ディレクトリに保存されます。結果には以下の情報が含まれます：

- **CPUホットスポット**: CPU時間を最も消費している関数やコードパス
- **メモリ使用量**: ヒープメモリの割り当てと解放のパターン
- **フレームグラフ**: コールスタックとCPU使用率の視覚的な表現

## パフォーマンス最適化のベストプラクティス

1. **測定してから最適化**: 最適化の前に必ずプロファイリングを行い、実際のボトルネックを特定する
2. **ベースラインを確立**: 最適化の前後で比較できるようにベースラインのパフォーマンスを測定する
3. **一度に一つの変更**: 複数の最適化を同時に行うと、どの変更が効果的だったかを判断するのが難しくなる
4. **実際の使用パターンをシミュレート**: 実際の使用状況に近いベンチマークを設計する
5. **スケーラビリティをテスト**: 様々な負荷レベルでテストし、システムのスケーラビリティを評価する

## 依存関係

- **Criterion.rs**: Rustのベンチマークフレームワーク
- **perf**: Linuxのパフォーマンスプロファイリングツール
- **Valgrind**: メモリプロファイリングツール
- **Flamegraph**: コールスタックの視覚化ツール

## トラブルシューティング

### よくある問題

1. **「command not found」エラー**:
   スクリプトに実行権限があることを確認してください：
   ```bash
   chmod +x tools/performance/benchmark.sh
   chmod +x tools/performance/profiler.sh
   ```

2. **プロファイリングツールのエラー**:
   必要な依存関係がインストールされていることを確認してください：
   ```bash
   sudo apt-get install linux-tools-common linux-tools-generic valgrind
   cargo install flamegraph
   cargo install inferno
   ```

3. **ベンチマークの実行が遅い**:
   ベンチマークはリリースモードで実行してください：
   ```bash
   cargo build --release
   ```

### サポート

パフォーマンス最適化ツールに関する問題やフィードバックがある場合は、GitHubのIssueを作成してください。