# ShardX インストールガイド

このドキュメントでは、ShardXをさまざまな方法でインストールする手順を説明します。

## 目次

- [自動インストール](#自動インストール)
- [手動インストール](#手動インストール)
- [Dockerを使用したインストール](#dockerを使用したインストール)
- [ソースからのビルド](#ソースからのビルド)
- [クラウドへのデプロイ](#クラウドへのデプロイ)

## 自動インストール

ShardXは、ワンライナーのインストールスクリプトを提供しています。これは最も簡単なインストール方法です。

```bash
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/install.sh | bash
```

特定のバージョンをインストールする場合は、以下のようにバージョンを指定できます：

```bash
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/install.sh | bash -s v0.1.0
```

このスクリプトは以下の処理を行います：

1. OSとアーキテクチャを検出
2. 必要なディレクトリを作成
3. バイナリをダウンロード（または必要に応じてソースからビルド）
4. Webインターフェースをダウンロード
5. 設定ファイルを作成
6. PATHにシンボリックリンクを追加

インストール後、以下のコマンドでShardXを起動できます：

```bash
shardx
```

## 手動インストール

### 前提条件

- Linux または macOS
- x86_64 または ARM64 アーキテクチャ

### インストール手順

1. [GitHubリリースページ](https://github.com/enablerdao/ShardX/releases)から、お使いのOSとアーキテクチャに合ったバイナリをダウンロードします。

2. バイナリを実行可能にします：

```bash
chmod +x shardx-linux-amd64
```

3. バイナリを適切な場所に移動します：

```bash
mkdir -p ~/.local/bin
mv shardx-linux-amd64 ~/.local/bin/shardx
```

4. PATHが正しく設定されていることを確認します：

```bash
export PATH="$PATH:$HOME/.local/bin"
```

5. ShardXを起動します：

```bash
shardx
```

## Dockerを使用したインストール

ShardXはDockerイメージも提供しています。

### Dockerイメージを使用する

```bash
docker pull enablerdao/shardx:latest
docker run -p 54868:54868 -p 54867:54867 enablerdao/shardx:latest
```

### Docker Composeを使用する

`docker-compose.yml`ファイルを作成します：

```yaml
version: '3'
services:
  shardx:
    image: enablerdao/shardx:latest
    ports:
      - "54868:54868"
      - "54867:54867"
    volumes:
      - ./data:/app/data
    environment:
      - NODE_ID=my_node
      - INITIAL_SHARDS=32
      - RUST_LOG=info
```

そして、以下のコマンドで起動します：

```bash
docker-compose up -d
```

## ソースからのビルド

### 前提条件

- Rust 1.70以上
- Git
- Node.js 18以上（Webインターフェースをビルドする場合）

### ビルド手順

1. リポジトリをクローンします：

```bash
git clone https://github.com/enablerdao/ShardX.git
cd ShardX
```

2. ShardXをビルドします：

```bash
cargo build --release
```

3. Webインターフェースをビルドします（オプション）：

```bash
cd web
npm install
npm run build
```

4. ShardXを起動します：

```bash
./target/release/shardx
```

## クラウドへのデプロイ

ShardXは、さまざまなクラウドプラットフォームにデプロイできます。詳細な手順については、[デプロイガイド](deployment/multi-platform-deployment.md)を参照してください。

### サポートされているプラットフォーム

- [Render](deployment/multi-platform-deployment.md#render)
- [Railway](deployment/multi-platform-deployment.md#railway)
- [Heroku](deployment/multi-platform-deployment.md#heroku)
- [Fly.io](deployment/multi-platform-deployment.md#flyio)

## トラブルシューティング

インストールに問題がある場合は、[トラブルシューティングガイド](deployment/troubleshooting.md)を参照するか、[GitHubのIssue](https://github.com/enablerdao/ShardX/issues)を作成してください。