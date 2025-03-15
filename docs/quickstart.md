# ShardX クイックスタートガイド

このガイドでは、ShardXを5分以内に起動する方法を説明します。

## 1. 最速の方法（すべてのOS対応）

以下のコマンドを実行するだけでShardXを自動的にインストールできます：

```bash
# 自動インストールスクリプト（Linux/macOS）
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/install.sh | bash
```

または、Dockerを使用する場合：

```bash
# Dockerを使用（すべてのOS）
docker run -d -p 54867:54867 -p 54868:54868 --name shardx enablerdao/shardx:latest
```

詳細なインストール方法については、[インストールガイド](../installation.md)を参照してください。

## 2. クラウドにデプロイ

以下のボタンをクリックするだけで、ShardXをクラウドにデプロイできます：

- [Renderにデプロイ](https://render.com/deploy?repo=https://github.com/enablerdao/ShardX)
- [Railwayにデプロイ](https://railway.app/template/ShardX)
- [Vercelにデプロイ](https://vercel.com/new/clone?repository-url=https://github.com/enablerdao/ShardX)

詳細な手順は[デプロイガイド](deployment/multi-platform-deployment.md)を参照してください。

## 3. 動作確認

ShardXが起動したら、以下のURLにアクセスできます：

- ウェブインターフェース: http://localhost:54867
- API: http://localhost:54868/api/v1/info

APIを使用して基本的な操作を行うことができます：

```bash
# システム情報を取得
curl http://localhost:54868/api/v1/info

# トランザクションを作成
curl -X POST http://localhost:54868/api/v1/transactions \
  -H "Content-Type: application/json" \
  -d '{"sender":"test1","receiver":"test2","amount":100}'

# トランザクション一覧を取得
curl http://localhost:54868/api/v1/transactions

# シャード情報を取得
curl http://localhost:54868/api/v1/shards
```

## 4. 次のステップ

- [API リファレンス](api/README.md)を参照して、ShardX APIの使用方法を学びます
- [開発者ガイド](developers/README.md)を参照して、ShardXの開発方法を学びます
- [デプロイガイド](deployment/multi-platform-deployment.md)を参照して、ShardXの本番環境へのデプロイ方法を学びます