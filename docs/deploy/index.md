# ShardX デプロイガイド

このガイドでは、ShardXをさまざまな環境にデプロイする方法を説明します。

## 必要条件

- Docker と Docker Compose (v1.29.0以上)
- 最小システム要件:
  - メインノード: 4コアCPU、8GB RAM、100GB SSD
  - シャードノード: 2コアCPU、4GB RAM、50GB SSD
  - モニタリング: 2コアCPU、4GB RAM、20GB SSD

## クイックスタート

### 1. リポジトリをクローン

```bash
git clone https://github.com/enablerdao/ShardX.git
cd ShardX
```

### 2. 環境設定

`.env`ファイルを作成して環境変数を設定します：

```bash
cp .env.example .env
```

必要に応じて`.env`ファイルを編集します。

### 3. Docker Composeでデプロイ

```bash
docker-compose up -d
```

これにより、メインノード、5つのシャードノード、Webインターフェース、モニタリングツールがデプロイされます。

### 4. 動作確認

- Web UI: http://localhost:80
- メインノードAPI: http://localhost:8080
- Grafanaダッシュボード: http://localhost:3000 (ユーザー名: admin, パスワード: shardx)
- Prometheusモニタリング: http://localhost:9090

## 高度なデプロイオプション

### マルチサーバーデプロイ

複数のサーバーにShardXをデプロイする場合は、以下の手順に従います：

1. 各サーバーにリポジトリをクローン
2. `docker-compose.yml`ファイルを編集して、適切なIPアドレスとポートを設定
3. メインノードを最初に起動
4. シャードノードを起動し、メインノードのアドレスを指定

例：

```bash
# メインノードサーバー
docker-compose up -d main-node

# シャードノードサーバー
MAIN_NODE=192.168.1.100:9000 docker-compose up -d shard-node-1
```

### Kubernetesデプロイ

Kubernetesクラスターにデプロイする場合は、`deploy/kubernetes`ディレクトリのマニフェストファイルを使用します：

```bash
kubectl apply -f deploy/kubernetes/namespace.yaml
kubectl apply -f deploy/kubernetes/configmap.yaml
kubectl apply -f deploy/kubernetes/secrets.yaml
kubectl apply -f deploy/kubernetes/main-node.yaml
kubectl apply -f deploy/kubernetes/shard-nodes.yaml
kubectl apply -f deploy/kubernetes/monitoring.yaml
kubectl apply -f deploy/kubernetes/web.yaml
```

## パフォーマンスチューニング

### メモリ最適化

メモリ使用量を最適化するには、以下の環境変数を設定します：

```
MEMORY_LIMIT=2G                # メモリ制限
GC_INTERVAL=60                 # GC間隔（秒）
WARNING_THRESHOLD=70           # 警告閾値（%）
CRITICAL_THRESHOLD=85          # 危険閾値（%）
AUTO_OPTIMIZE=true             # 自動最適化
```

### ストレージ最適化

ストレージを最適化するには、以下の環境変数を設定します：

```
COMPACTION_THRESHOLD=100M      # 圧縮閾値
COMPACTION_INTERVAL=3600       # 圧縮間隔（秒）
MAX_FILE_SIZE=1G               # 最大ファイルサイズ
AUTO_COMPACT=true              # 自動圧縮
COMPRESSION_LEVEL=6            # 圧縮レベル（1-9）
```

### クロスシャード最適化

クロスシャードトランザクションを最適化するには、以下の環境変数を設定します：

```
BATCH_SIZE=50                  # バッチサイズ
BATCH_INTERVAL_MS=100          # バッチ間隔（ミリ秒）
ROUTING_UPDATE_INTERVAL=60     # ルーティング更新間隔（秒）
MAX_PARALLEL_EXECUTIONS=10     # 最大並列実行数
```

## モニタリングとメンテナンス

### メトリクスの確認

Grafanaダッシュボードにアクセスして、以下のメトリクスを確認できます：

- トランザクション処理速度（TPS）
- メモリ使用量
- ストレージ使用量
- クロスシャードトランザクション数
- レイテンシ

### ログの確認

各コンテナのログを確認するには：

```bash
docker-compose logs -f main-node
docker-compose logs -f shard-node-1
```

### バックアップと復元

データのバックアップを作成するには：

```bash
docker-compose exec main-node /app/scripts/backup.sh
```

バックアップからデータを復元するには：

```bash
docker-compose exec main-node /app/scripts/restore.sh /app/backups/backup-20230101.tar.gz
```

## トラブルシューティング

### よくある問題と解決策

1. **ノードが起動しない**
   - ログを確認: `docker-compose logs -f main-node`
   - メモリ制限を確認: `docker stats`

2. **シャードノードがメインノードに接続できない**
   - ネットワーク設定を確認
   - ファイアウォール設定を確認

3. **パフォーマンスが低い**
   - リソース使用量を確認: `docker stats`
   - 設定パラメータを調整
   - ハードウェアスペックを確認

## サポートとフィードバック

問題やフィードバックがある場合は、GitHubリポジトリのIssueセクションに投稿してください：
https://github.com/enablerdao/ShardX/issues