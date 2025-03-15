# トランザクション分析機能

ShardXのトランザクション分析機能は、ブロックチェーン上のトランザクションを詳細に分析し、パターンや関連性を発見するための機能を提供します。この機能により、ユーザーはトランザクションの流れを理解し、より効果的な意思決定を行うことができます。

## 主な特徴

- **詳細な統計情報**: トランザクション数、確認時間、ステータス分布などの基本統計
- **時間帯別分析**: 時間帯ごとのトランザクション量の分析
- **パターン検出**: 繰り返し発生するトランザクションパターンの自動検出
- **関連性分析**: トランザクション間の親子関係や関連性の分析
- **グラフメトリクス**: トランザクションのグラフ構造に関する指標
- **視覚化**: 高度なチャートによるデータの視覚化

## 使用方法

### トランザクションアナライザーの初期化

```rust
use shardx::transaction::DAG;
use shardx::transaction_analysis::{TransactionAnalyzer, AnalysisPeriod};
use std::sync::Arc;

// DAGを取得
let dag = Arc::new(DAG::new());

// トランザクションアナライザーを初期化
let analyzer = TransactionAnalyzer::new(dag.clone());
```

### 基本的な分析の実行

```rust
// 過去24時間のトランザクションを分析
let analysis = analyzer.analyze(AnalysisPeriod::Last24Hours);

// 基本統計情報を表示
println!("総トランザクション数: {}", analysis.total_transactions);
println!("確認済みトランザクション: {}", analysis.confirmed_transactions);
println!("拒否されたトランザクション: {}", analysis.rejected_transactions);
println!("保留中のトランザクション: {}", analysis.pending_transactions);
println!("平均確認時間: {:.2}秒", analysis.avg_confirmation_time);

// 時間帯別ボリュームを表示
for (hour, count) in &analysis.volume_by_hour {
    println!("{}時: {}件", hour, count);
}

// トランザクションタイプの分布を表示
for (tx_type, count) in &analysis.transaction_types {
    println!("{}: {}件", tx_type, count);
}

// 最もアクティブなアドレスを表示
for (address, count) in &analysis.top_active_addresses {
    println!("{}: {}件", address, count);
}

// グラフメトリクスを表示
println!("平均次数: {:.2}", analysis.graph_metrics.avg_degree);
println!("最大次数: {}", analysis.graph_metrics.max_degree);
println!("クラスタリング係数: {:.3}", analysis.graph_metrics.clustering_coefficient);
println!("最長パス長: {}", analysis.graph_metrics.longest_path);
```

### 異なる期間での分析

```rust
// 過去1週間の分析
let week_analysis = analyzer.analyze(AnalysisPeriod::LastWeek);

// 過去1ヶ月の分析
let month_analysis = analyzer.analyze(AnalysisPeriod::LastMonth);

// カスタム期間の分析
use chrono::{Duration, Utc};

let custom_analysis = analyzer.analyze(AnalysisPeriod::Custom {
    start: Utc::now() - Duration::days(3),
    end: Utc::now(),
});
```

### パターン検出

```rust
// トランザクションパターンを検出
let patterns = analyzer.detect_patterns(AnalysisPeriod::Last24Hours);

// 検出されたパターンを表示
for pattern in &patterns {
    println!("パターン: {}", pattern.name);
    println!("説明: {}", pattern.description);
    println!("検出回数: {}", pattern.occurrences);
    println!("関連トランザクション: {:?}", pattern.related_transactions);
    println!("---");
}
```

### トランザクションの関連性分析

```rust
// トランザクションの関連性を分析
let tx_id = "tx123"; // 分析対象のトランザクションID
if let Some(relationships) = analyzer.analyze_transaction_relationships(tx_id) {
    println!("トランザクションID: {}", relationships.transaction.id);
    
    // 親トランザクション
    println!("親トランザクション: {} 件", relationships.parents.len());
    for parent in &relationships.parents {
        println!("  - {}", parent.id);
    }
    
    // 子トランザクション
    println!("子トランザクション: {} 件", relationships.children.len());
    for child in &relationships.children {
        println!("  - {}", child.id);
    }
    
    // 兄弟トランザクション
    println!("兄弟トランザクション: {} 件", relationships.siblings.len());
    for sibling in &relationships.siblings {
        println!("  - {}", sibling.id);
    }
}
```

## ウェブインターフェース

ShardXは、トランザクション分析を視覚化するための直感的なウェブインターフェースも提供しています。以下の機能が利用可能です：

- トランザクション検索と詳細表示
- 時間帯別ボリュームのチャート
- トランザクションタイプ分布の円グラフ
- パターン検出結果の表示
- トランザクショングラフの視覚化
- アクティブアドレスのランキング

ウェブインターフェースにアクセスするには、ShardXノードを起動し、ブラウザで`http://localhost:PORT/transaction_analysis.html`にアクセスしてください。

## 高度なチャート機能

ShardXは、トランザクションデータを視覚化するための高度なチャート機能も提供しています：

- **パフォーマンスメトリクス**: TPS、レイテンシ、確認済みトランザクション数の時系列チャート
- **ボリュームチャート**: 時間帯別のトランザクション量
- **シャード負荷分布**: 各シャードの負荷状況
- **テクニカル分析**: 移動平均、ボリンジャーバンド、RSIなどのテクニカル指標
- **ネットワーク健全性**: アップタイム、レスポンスタイム、エラー率の推移

高度なチャート機能にアクセスするには、ShardXノードを起動し、ブラウザで`http://localhost:PORT/advanced_charts.html`にアクセスしてください。

## 実装上の注意点

### 大規模データの処理

トランザクション数が多い場合、メモリ使用量と処理時間に注意が必要です：

```rust
// 分析対象のトランザクション数を制限
let max_transactions = 10000;
let limited_analysis = analyzer.analyze_with_limit(AnalysisPeriod::LastMonth, max_transactions);
```

### リアルタイム分析

リアルタイム分析を行う場合は、増分更新を使用することで効率的に処理できます：

```rust
// 新しいトランザクションが追加されたときに分析を更新
dag.on_transaction_added(|tx| {
    analyzer.update_analysis(tx);
});
```

## APIリファレンス

詳細なAPIリファレンスについては、[TransactionAnalyzer API](../api/transaction_analysis.md)を参照してください。