# ShardX クイックスタートガイド

このガイドでは、ShardXを5分以内に起動する方法を説明します。

## 1. 最速の方法（Docker）

Dockerがインストールされている場合、以下のコマンドを実行するだけでShardXを起動できます：

```bash
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/install.sh | bash
```

または、手動でDockerコマンドを実行することもできます：

```bash
docker run -d -p 54867:54867 -p 54868:54868 --name shardx enablerdao/shardx:latest
```

## 2. OS別インストール方法

### Linux (Ubuntu/Debian)

```bash
# 依存関係をインストール
sudo apt update && sudo apt install -y git curl build-essential libssl-dev pkg-config

# ShardXをクローンして起動
git clone https://github.com/enablerdao/ShardX.git
cd ShardX
./scripts/linux_install.sh
./scripts/run.sh
```

### macOS

```bash
# Homebrewがない場合はインストール
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 依存関係をインストール
brew install git curl rust

# ShardXをクローンして起動
git clone https://github.com/enablerdao/ShardX.git
cd ShardX
./scripts/mac_install.sh
./scripts/run.sh
```

### Windows

PowerShellを管理者権限で実行し、以下のコマンドを実行します：

```powershell
# Rustをインストール
Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
.\rustup-init.exe -y

# ShardXをクローンして起動
git clone https://github.com/enablerdao/ShardX.git
cd ShardX
.\scripts\windows_install.ps1
.\scripts\run.ps1
```

## 3. クラウドにデプロイ

以下のボタンをクリックするだけで、ShardXをクラウドにデプロイできます：

- [Renderにデプロイ](https://render.com/deploy?repo=https://github.com/enablerdao/ShardX)
- [Railwayにデプロイ](https://railway.app/template/ShardX)
- [Vercelにデプロイ](https://vercel.com/new/clone?repository-url=https://github.com/enablerdao/ShardX)

詳細な手順は[Renderデプロイガイド](deployment/render-free.md)を参照してください。

## 4. 動作確認

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

## 5. 次のステップ

- [API リファレンス](api/README.md)を参照して、ShardX APIの使用方法を学びます
- [開発者ガイド](developers/README.md)を参照して、ShardXの開発方法を学びます
- [デプロイガイド](deployment/README.md)を参照して、ShardXの本番環境へのデプロイ方法を学びます