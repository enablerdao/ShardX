# Fly.ioへのShardXデプロイガイド

Fly.ioは、グローバルに分散したエッジサーバーでアプリケーションを実行できるプラットフォームです。ShardXをFly.ioにデプロイすることで、低レイテンシーと高可用性を実現できます。

## 特徴

- **グローバル分散デプロイ**: 世界中の複数のリージョンにアプリケーションをデプロイ
- **低レイテンシー**: ユーザーに最も近いリージョンからサービスを提供
- **自動スケーリング**: 負荷に応じて自動的にスケールアップ/ダウン
- **永続ストレージ**: ボリュームを使用してデータを永続化
- **プライベートネットワーク**: アプリケーション間の安全な通信

## 前提条件

- [Fly.io](https://fly.io)アカウント
- [Flyctl](https://fly.io/docs/hands-on/install-flyctl/)（Fly.ioのCLIツール）

## デプロイ手順

### 1. Fly.io CLIのインストール

```bash
# macOS
brew install flyctl

# Linux
curl -L https://fly.io/install.sh | sh

# Windows (PowerShell)
iwr https://fly.io/install.ps1 -useb | iex
```

### 2. Fly.ioにログイン

```bash
fly auth login
```

### 3. アプリケーションの初期化

ShardXリポジトリのルートディレクトリに移動し、以下のコマンドを実行します：

```bash
cd ShardX
fly launch
```

対話式のセットアップが始まります：

- アプリ名を入力（または自動生成を受け入れる）
- リージョンを選択（複数選択可能）
- PostgreSQLやRedisなどのサービスを追加するか選択
- デプロイを確認

### 4. fly.tomlの設定

`fly launch`コマンドによって`fly.toml`ファイルが生成されます。必要に応じて以下のように編集します：

```toml
# fly.toml
[env]
  PORT = "54868"
  P2P_PORT = "54867"
  NODE_ID = "fly-node-1"
  LOG_LEVEL = "info"

[mounts]
  source = "shardx_data"
  destination = "/app/data"

[[services]]
  internal_port = 54868
  protocol = "tcp"

  [[services.ports]]
    port = 80
    handlers = ["http"]

  [[services.ports]]
    port = 443
    handlers = ["tls", "http"]

[[services]]
  internal_port = 54867
  protocol = "tcp"

  [[services.ports]]
    port = 54867
```

### 5. アプリケーションのデプロイ

```bash
fly deploy
```

### 6. デプロイの確認

```bash
# アプリケーションのステータスを確認
fly status

# ログを表示
fly logs

# シェルにアクセス
fly ssh console
```

### 7. スケーリング（オプション）

```bash
# インスタンス数を増やす
fly scale count 3

# マシンサイズを変更
fly scale vm shared-cpu-1x
```

## 複数ノードのクラスタ構成

ShardXを複数ノードのクラスタとして実行するには、以下の手順に従います：

### 1. 最初のノードをデプロイ

上記の手順に従って最初のノードをデプロイします。

### 2. 追加ノードの設定

追加ノードごとに新しいアプリケーションを作成し、環境変数で最初のノードをブートストラップノードとして指定します：

```bash
# 新しいアプリケーションを作成
fly launch --name shardx-node-2

# fly.tomlを編集して最初のノードをブートストラップとして指定
```

`fly.toml`に以下の環境変数を追加します：

```toml
[env]
  PORT = "54868"
  P2P_PORT = "54867"
  NODE_ID = "fly-node-2"
  BOOTSTRAP = "fly-node-1.internal:54867"
  LOG_LEVEL = "info"
```

### 3. 追加ノードをデプロイ

```bash
fly deploy
```

## トラブルシューティング

### ネットワーク接続の問題

ノード間の通信に問題がある場合は、以下を確認してください：

1. 各ノードの`fly.toml`ファイルでP2Pポート（54867）が正しく公開されているか
2. 内部ネットワーク名（`.internal`）が正しく設定されているか

```bash
# ノード間の接続をテスト
fly ssh console -C "curl fly-node-1.internal:54868/api/v1/info"
```

### ストレージの問題

データの永続化に問題がある場合：

```bash
# ボリュームの状態を確認
fly volumes list

# 新しいボリュームを作成（必要な場合）
fly volumes create shardx_data --size 10
```

## 本番環境のベストプラクティス

1. **複数リージョンへのデプロイ**: グローバルな可用性を確保するため、複数のリージョンにデプロイ
2. **監視の設定**: Prometheusメトリクスを有効にし、Grafanaダッシュボードを設定
3. **自動バックアップ**: 定期的なデータバックアップを設定
4. **スケーリングポリシー**: 負荷に応じた自動スケーリングを設定
5. **セキュリティ設定**: 適切なファイアウォールルールとアクセス制限を設定

## 参考リンク

- [Fly.io公式ドキュメント](https://fly.io/docs/)
- [Fly.ioのボリューム管理](https://fly.io/docs/reference/volumes/)
- [Fly.ioのスケーリング](https://fly.io/docs/reference/scaling/)
- [Fly.ioのネットワーキング](https://fly.io/docs/reference/networking/)