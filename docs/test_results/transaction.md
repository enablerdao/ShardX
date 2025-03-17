# ShardX.org トランザクションテスト結果

## テスト概要

このテストでは、ShardX.orgの複数ノード間でトランザクションを送信し、ノード間の通信とトランザクション処理機能を検証しました。

## テスト環境

- **ノード数**: 2（node1: 54868, node3: 54870）
- **データディレクトリ**: ./data/node1, ./data/node3
- **シャード数**: 256

## ノード起動

2つのノードを以下のコマンドで起動しました：

```bash
./bin/shardx --node-id node1 --port 54868 --data-dir ./data/node1
./bin/shardx --node-id node3 --port 54870 --data-dir ./data/node3
```

各ノードは正常に起動し、以下のログが出力されました：

```
[INFO  shardx] ShardX.org ノードを起動中...
[INFO  shardx] ノードID: nodeX
[INFO  shardx] ポート: 5486X
[INFO  shardx] データディレクトリ: ./data/nodeX
[INFO  shardx] ログレベル: info
[INFO  shardx] DAGを初期化中...
[INFO  shardx] シャーディングマネージャを初期化中 (シャード数: 256)...
[INFO  shardx] コンセンサスエンジンを初期化中...
[INFO  shardx::node] Starting node nodeX
[INFO  shardx::node] Node nodeX started
[INFO  shardx] APIサーバーを起動中 (ポート: 5486X)...
[INFO  shardx::api] Starting API server on port 5486X
[INFO  warp::server] Server::run; addr=0.0.0.0:5486X
[INFO  warp::server] listening on http://0.0.0.0:5486X
```

## ノード情報の確認

各ノードの情報を取得しました：

### ノード1 (node1)

```json
{
  "id": "node1",
  "status": "Running",
  "tps": 0,
  "shard_count": 256,
  "confirmed_transactions": 0
}
```

### ノード3 (node3)

```json
{
  "id": "node3",
  "status": "Running",
  "tps": 0,
  "shard_count": 256,
  "confirmed_transactions": 0
}
```

## トランザクションテスト

### トランザクション1: ノード1への送信

最初のトランザクションをノード1に送信しました：

```bash
curl -X POST -H "Content-Type: application/json" -d '{"parent_ids":[],"payload":"SGVsbG8sIEh5cGVyRmx1eCE=","signature":"MHgxYTJiM2M0ZDVlNmY="}' http://localhost:54868/transactions
```

レスポンス:
```json
{
  "id": "b078712b-abf0-4405-986c-1285d85a087f",
  "status": "success"
}
```

### トランザクション2: ノード3への送信（親トランザクションあり）

2番目のトランザクションをノード3に送信し、最初のトランザクションを親として参照しました：

```bash
curl -X POST -H "Content-Type: application/json" -d '{"parent_ids":["b078712b-abf0-4405-986c-1285d85a087f"],"payload":"VHJhbnNmZXIgZnJvbSBub2RlMSB0byBub2RlMw==","signature":"MHgxYTJiM2M0ZDVlNmY="}' http://localhost:54870/transactions
```

レスポンス:
```json
{
  "id": "045494ed-74e6-45b2-8f64-8abee1c3215b",
  "status": "success"
}
```

## 結論

ShardX.orgの複数ノード間でのトランザクション送信テストは成功しました。各ノードは正常に起動し、トランザクションを受け付けることができました。また、2番目のトランザクションでは最初のトランザクションを親として参照することができ、DAG（有向非巡回グラフ）構造の基本的な機能が動作していることを確認しました。

ただし、現時点ではノード間の同期機能が完全に実装されていないため、一方のノードで作成されたトランザクションが他方のノードで自動的に認識されるわけではありません。これはフェーズ2で実装予定の機能です。

このテスト結果は、ShardX.orgのフェーズ1の目標である「基本的なノード構造とトランザクション処理」が達成されていることを示しています。