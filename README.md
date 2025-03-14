# ShardX（シャードエックス）

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/web/assets/logo.svg" alt="ShardX Logo" width="300"/>
  <h3>次世代高速ブロックチェーンプラットフォーム</h3>
  
  [![GitHub Stars](https://img.shields.io/github/stars/enablerdao/ShardX?style=social)](https://github.com/enablerdao/ShardX/stargazers)
  [![GitHub Forks](https://img.shields.io/github/forks/enablerdao/ShardX?style=social)](https://github.com/enablerdao/ShardX/network/members)
  [![GitHub Issues](https://img.shields.io/github/issues/enablerdao/ShardX)](https://github.com/enablerdao/ShardX/issues)
  [![GitHub License](https://img.shields.io/github/license/enablerdao/ShardX)](https://github.com/enablerdao/ShardX/blob/main/LICENSE)
  [![Twitter Follow](https://img.shields.io/twitter/follow/ShardXOrg?style=social)](https://twitter.com/ShardXOrg)
</div>

## 🚩 ミッション
「分散型テクノロジーで世界中の人々のつながりを深め、誰もが安心して価値を交換できる未来を実現する。」

## 🌌 ビジョン
「まるで呼吸をするように自然で快適に、誰もがストレスなく繋がり、自由に価値をやりとりできる世界を創造する。」

## 🚀 様々なデプロイ方法

### ワンクリックデプロイ

<div align="center">
  <a href="https://heroku.com/deploy?template=https://github.com/enablerdao/ShardX">
    <img src="https://www.herokucdn.com/deploy/button.svg" alt="Deploy to Heroku" />
  </a>
  <a href="https://vercel.com/new/clone?repository-url=https://github.com/enablerdao/ShardX">
    <img src="https://vercel.com/button" alt="Deploy to Vercel" />
  </a>
  <a href="https://app.netlify.com/start/deploy?repository=https://github.com/enablerdao/ShardX">
    <img src="https://www.netlify.com/img/deploy/button.svg" alt="Deploy to Netlify" />
  </a>
  <a href="https://deploy.cloud.run?git_repo=https://github.com/enablerdao/ShardX">
    <img src="https://deploy.cloud.run/button.svg" alt="Run on Google Cloud" />
  </a>
  <a href="https://portal.azure.com/#create/Microsoft.Template/uri/https%3A%2F%2Fraw.githubusercontent.com%2Fenablerdao%2FShardX%2Fmain%2Fazure-pipelines.yml">
    <img src="https://aka.ms/deploytoazurebutton" alt="Deploy to Azure" />
  </a>
</div>

### クラウドプロバイダー別のデプロイ

- **AWS**: [CloudFormation テンプレート](cloudformation.yml)
- **Digital Ocean**: [App Platform 設定](digital_ocean.yml)
- **Fly.io**: [設定ファイル](fly.toml)
- **Railway**: [設定ファイル](railway.json)

### ローカルインストール

```bash
# 完全自動インストール (依存関係も自動的にインストール)
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/auto_install.sh | bash

# 対話なしの簡単インストール (Docker必須)
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/simple_install.sh | bash

# または詳細オプション付きインストール
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/quick_install.sh | bash
```

### 特定のユースケース向けインストール

```bash
# 開発者向けインストール (Rust, Node.js, VSCodeなど)
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/install_scripts/dev_install.sh | bash

# エンタープライズインストール (高可用性構成)
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/install_scripts/enterprise_install.sh | bash

# ミニマルインストール (最小限のリソースで実行)
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/install_scripts/minimal_install.sh | bash
```

詳細なインストールオプションについては、[インストールスクリプト一覧](install_scripts/README.md)を参照してください。

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/web/assets/pof_consensus.svg" alt="ShardX コンセンサスメカニズム" width="80%"/>
</div>

## 📋 概要
ShardXは高速処理、スケーラビリティ、セキュリティを兼ね備えた次世代ブロックチェーンプラットフォームです。まるで呼吸をするように自然で、誰もが簡単に利用できる設計を特徴としています。

初期フェーズでは50,000 TPS（1秒あたりのトランザクション数）を目標とし、フェーズ2では100,000 TPSを目指します。この高いパフォーマンスにより、世界中の人々が安心して価値を交換できる基盤を提供します。

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/web/assets/system_architecture.svg" alt="ShardX システムアーキテクチャ" width="80%"/>
</div>

## ✨ 特徴
- **⚡ Proof of Flow (PoF)**: DAG、PoH、PoSを組み合わせた革新的なコンセンサスアルゴリズム

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/web/assets/pof_consensus.svg" alt="ShardX PoF比較" width="80%"/>
</div>

- **🔄 動的シャーディング**: トラフィックに応じて自動的にシャード数を調整

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/web/assets/dynamic_sharding.svg" alt="ShardX 動的シャーディング" width="80%"/>
</div>

- **🧠 AI駆動型トランザクション管理**: 優先順位付けと予測によるスマートな処理

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/web/assets/ai_transaction.svg" alt="ShardX AI駆動型トランザクション管理" width="80%"/>
</div>

- **🔒 高度なセキュリティ**: AES-256暗号化とマルチシグネチャによる堅牢な保護

## 技術アーキテクチャ

### 1. コンセンサスメカニズム: Proof of Flow (PoF)
PoFは以下の3つの技術を組み合わせた革新的なコンセンサスメカニズムです：

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/web/assets/pof_consensus.svg" alt="ShardX PoFフロー" width="80%"/>
</div>

1. **有向非巡回グラフ (DAG)**
   - ブロックチェーンの代わりにDAG構造を採用
   - トランザクションは過去のトランザクションを参照し、並列処理が可能
   - 実装: `src/transaction.rs`の`DAG`構造体

2. **Proof of History (PoH)**
   - 各トランザクションに暗号学的に検証可能なタイムスタンプを付与
   - 時間の経過を証明し、トランザクションの順序を保証
   - 実装: `src/transaction.rs`の`Transaction`構造体の`timestamp`フィールド

3. **Proof of Stake (PoS)**
   - バリデータがステークを保有し、トランザクションを検証
   - 悪意のある行動に対してはステークが没収される仕組み
   - 実装: `src/consensus.rs`の`Validator`トレイトと`SimpleValidator`構造体

### 2. スケーラビリティ: 動的シャーディング
トラフィック量に応じて自動的にシャード数を調整する仕組みを実装：

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/web/assets/dynamic_sharding.svg" alt="ShardX シャード再分割" width="80%"/>
</div>

1. **シャード割り当て**
   - トランザクションIDのハッシュ値に基づいてシャードを決定
   - 実装: `src/sharding.rs`の`ShardingManager::assign_shard`メソッド

2. **動的調整**
   - 負荷に応じてシャード数を256から最大512まで動的に調整
   - 実装: `src/sharding.rs`の`ShardingManager::adjust_shards`メソッド

3. **クロスシャード通信**
   - 異なるシャード間でのトランザクション転送を効率的に処理
   - 実装: `src/sharding.rs`の`CrossShardManager`構造体

### 3. AI駆動型トランザクション管理
AIを活用してトランザクションの優先順位付けと予測を行います：

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/web/assets/ai_transaction.svg" alt="ShardX AIモデルアーキテクチャ" width="80%"/>
</div>

1. **優先順位付け**
   - トランザクションの特性（サイズ、親数、タイムスタンプなど）に基づいて優先度を計算
   - 実装: `src/ai.rs`の`AIPriorityManager::calculate_priority`メソッド

2. **優先キュー**
   - 優先度の高いトランザクションから処理するためのキュー管理
   - 実装: `src/ai.rs`の`PrioritizedTransaction`構造体と`BinaryHeap`

### 4. ノードアーキテクチャ
各ノードは以下のコンポーネントで構成されています：

1. **コアコンポーネント**
   - DAG: トランザクションの保存と管理
   - コンセンサスエンジン: トランザクションの検証と承認
   - シャーディングマネージャ: シャードの割り当てと調整
   - AI優先度マネージャ: トランザクションの優先順位付け
   - 実装: `src/node.rs`の`Node`構造体

2. **APIサーバー**
   - RESTful APIによるノードとの対話
   - トランザクションの送信と状態確認のエンドポイント
   - 実装: `src/api.rs`の`ApiServer`構造体

3. **Webインターフェース**
   - ノードの状態とパフォーマンスの可視化
   - トランザクションの作成と監視
   - 実装: `web/index.html`と`web/server.js`

## 🚀 クイックスタート

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/web/assets/user_flow.svg" alt="ShardX ユーザーフロー" width="80%"/>
</div>

> **注意**: Dockerビルドに問題がある場合は、以下の手順を試してください：
> 1. Dockerfileが最新のものであることを確認
> 2. `docker build --no-cache -t hyperflux:latest .` コマンドを使用して、キャッシュを使わずにビルド
> 3. それでも問題が解決しない場合は、`docker system prune -a` を実行してDockerシステムをクリーンアップしてから再試行

### 🌐 ワンクリックデプロイ

<div align="center">
  <a href="https://render.com/deploy?repo=https://github.com/enablerdao/ShardX">
    <img src="https://render.com/images/deploy-to-render-button.svg" alt="Deploy to Render" />
  </a>
  <a href="https://gitpod.io/#https://github.com/enablerdao/ShardX">
    <img src="https://gitpod.io/button/open-in-gitpod.svg" alt="Open in Gitpod" />
  </a>
</div>

### 🔥 1コマンドでの起動（すべてのOS対応）

以下の1コマンドで、ShardXのノードとWebインターフェースを起動できます：

```bash
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/quick_install.sh | bash
```

または、より簡単なインストール方法：

```bash
git clone https://github.com/enablerdao/ShardX.git
cd ShardX
./quick_install.sh
```

<details>
<summary>💡 インストールスクリプトの詳細</summary>

インストールスクリプトは以下の機能を提供します：
- 🖥️ OSとアーキテクチャの自動検出（Linux、macOS、Windows、x86_64、ARM64）
- 🐳 Dockerとdocker-composeの自動チェックとインストール補助
- 🔄 インタラクティブモードと非インタラクティブモードの自動切り替え
- 🚦 開発モード、バックグラウンドモード、本番モードの選択肢

</details>

<details>
<summary>🔧 手動インストール</summary>

リポジトリをクローンして手動で実行することもできます：

```bash
# リポジトリのクローン
git clone https://github.com/enablerdao/ShardX.git

# ディレクトリに移動
cd ShardX

# インストールスクリプトを実行
./install.sh
```

</details>

### 🌟 起動後のアクセス

インストールが完了すると、以下のサービスが起動します：

- 🔗 **ノードAPI**: [http://localhost:54868](http://localhost:54868)
- 🖥️ **Webインターフェース**: [http://localhost:54867](http://localhost:54867)

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/web/assets/dashboard.svg" alt="ShardX Dashboard" width="80%"/>
</div>

### 💻 コマンドラインインターフェース (CLI)

ShardXはコマンドラインからの操作も可能です：

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/web/assets/user_flow.svg" alt="ShardX 開発フロー" width="80%"/>
</div>

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/web/assets/cli.svg" alt="ShardX CLI" width="80%"/>
</div>

```bash
# CLIを起動
npm run cli

# または直接実行
node cli.js
```

<details>
<summary>📋 利用可能なCLIコマンド一覧</summary>

| コマンド | 説明 | 例 |
|---------|------|-----|
| `help` | ヘルプメッセージを表示 | `help` |
| `info` | ノード情報を表示 | `info` |
| `accounts` | すべてのアカウントを表示 | `accounts` |
| `create-account NAME` | 新しいアカウントを作成 | `create-account alice` |
| `balance ACCOUNT_ID` | アカウントの残高を表示 | `balance acc_123456` |
| `transfer FROM TO AMOUNT [TOKEN_ID]` | 送金を実行 | `transfer acc_123 acc_456 100 BTC` |
| `pairs` | 取引ペアを表示 | `pairs` |
| `orderbook BASE QUOTE` | オーダーブックを表示 | `orderbook BTC USD` |
| `create-order ACCOUNT_ID BASE QUOTE TYPE PRICE AMOUNT` | 注文を作成 | `create-order acc_123 BTC USD buy 30000 0.5` |
| `exit` | CLIを終了 | `exit` |

</details>

### ⚙️ 開発者向けスクリプト

<details>
<summary>📦 npm スクリプト</summary>

package.jsonに定義されたスクリプトを使用して、簡単に操作できます：

| スクリプト | 説明 | コマンド |
|-----------|------|---------|
| `all` | すべてのサービスを1つのターミナルで起動 | `npm run all` |
| `start` | Rustノードを起動 | `npm start` |
| `web` | Webサーバーを起動 | `npm run web` |
| `cli` | CLIを起動 | `npm run cli` |
| `docker` | Dockerでノードとウェブサーバーを起動（フォアグラウンド） | `npm run docker` |
| `docker-build` | Dockerイメージをビルド | `npm run docker-build` |
| `docker-stop` | Dockerコンテナを停止 | `npm run docker-stop` |
| `docker:dev` | 開発環境でDockerを起動（バックグラウンド） | `npm run docker:dev` |
| `docker:prod` | 本番環境でDockerを起動（バックグラウンド） | `npm run docker:prod` |
| `docker:dev:rebuild` | 開発環境でイメージを再ビルドして起動 | `npm run docker:dev:rebuild` |
| `docker:prod:rebuild` | 本番環境でイメージを再ビルドして起動 | `npm run docker:prod:rebuild` |

</details>

<details>
<summary>🛠️ シェルスクリプト</summary>

直接シェルスクリプトを実行することもできます：

#### すべてのサービスを1つのターミナルで起動
```bash
./start.sh
```

#### Dockerサービスを起動（オプション指定可能）
```bash
./docker-start.sh [オプション]
```

**オプション:**
- `-e, --env ENV` - 環境を指定 (dev または prod) [デフォルト: dev]
- `-r, --rebuild` - イメージを再ビルド
- `-h, --help` - ヘルプメッセージを表示

**例:**
```bash
# 開発環境で起動
./docker-start.sh

# 本番環境で起動
./docker-start.sh -e prod

# 開発環境でイメージを再ビルドして起動
./docker-start.sh -r

# 本番環境でイメージを再ビルドして起動
./docker-start.sh -e prod -r
```

</details>

### 🔍 前提条件

<details>
<summary>必要なソフトウェア</summary>

- **Git**: バージョン管理システム
- **Docker**: コンテナ化プラットフォーム
- **Docker Compose**: マルチコンテナDockerアプリケーションの定義と実行ツール
- **Node.js** (オプション): CLIを使用する場合のみ必要

インストールスクリプトは、これらの依存関係の有無を自動的にチェックし、必要に応じてインストールをサポートします。

</details>

### 🌍 クロスプラットフォームサポート

<div align="center">
  <table>
    <tr>
      <th>プラットフォーム</th>
      <th>サポートされるアーキテクチャ</th>
      <th>テスト済みバージョン</th>
    </tr>
    <tr>
      <td>
        <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/docs/images/linux.png" width="20" alt="Linux" />
        Linux
      </td>
      <td>x86_64, ARM64</td>
      <td>Ubuntu 20.04+, Debian 11+, CentOS 8+, Alpine 3.15+</td>
    </tr>
    <tr>
      <td>
        <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/docs/images/macos.png" width="20" alt="macOS" />
        macOS
      </td>
      <td>Intel Chip, Apple Silicon (M1/M2/M3)</td>
      <td>macOS 11 (Big Sur)+</td>
    </tr>
    <tr>
      <td>
        <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/docs/images/windows.png" width="20" alt="Windows" />
        Windows
      </td>
      <td>x86_64 (WSL2経由)</td>
      <td>Windows 10/11 with WSL2</td>
    </tr>
  </table>
</div>

<details>
<summary>🔧 特定のアーキテクチャ向けにビルドする方法</summary>

Dockerfileはマルチアーキテクチャビルドをサポートしており、ビルド時に自動的に適切なアーキテクチャを検出します。ARM64プラットフォーム（Raspberry Pi、Apple Silicon Macなど）でビルドする場合は、以下のコマンドを使用します：

```bash
# ARM64向けにビルド
TARGETARCH=arm64 docker-compose build
```

または、docker-start.shスクリプトを使用する場合：

```bash
# 環境変数を設定してスクリプトを実行
TARGETARCH=arm64 ./docker-start.sh -r
```

</details>

<details>
<summary>🛠️ クロスプラットフォームビルドの仕組み</summary>

ShardXは、異なるアーキテクチャでのビルドを自動的に処理するための改良されたDockerfileを使用しています：

1. **アーキテクチャの自動検出**：
   - ホストマシンのアーキテクチャを自動的に検出
   - `TARGETARCH`環境変数による手動指定も可能

2. **適切なツールチェーンの設定**：
   - ARM64向けビルド時に必要なクロスコンパイラを自動インストール
   - 環境変数を適切に設定してクロスコンパイルをサポート

3. **マルチアーキテクチャバイナリの生成**：
   - x86_64とARM64の両方のバイナリを生成
   - 実行時に適切なバイナリを自動選択

これにより、Intel/AMD CPUを搭載したマシンでもApple Silicon（M1/M2/M3）やRaspberry Piなどの環境でも同じコマンドで簡単に実行できます。

</details>

## 詳細な実装ガイド

### 各OSでの環境構築

#### Linux (Ubuntu/Debian)

1. **Dockerのインストール**
```bash
# 前提パッケージのインストール
sudo apt-get update
sudo apt-get install -y apt-transport-https ca-certificates curl gnupg lsb-release

# Dockerの公式GPGキーを追加
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg

# リポジトリを設定
echo "deb [arch=amd64 signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

# Dockerをインストール
sudo apt-get update
sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin

# Docker Composeをインストール
sudo curl -L "https://github.com/docker/compose/releases/download/v2.18.1/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# ユーザーをdockerグループに追加（sudo不要でDockerを実行可能に）
sudo usermod -aG docker $USER
```

2. **プロジェクトのクローンと起動**
```bash
git clone https://github.com/enablerdao/ShardX.git
cd ShardX
docker-compose up --build
```

#### macOS

1. **Dockerのインストール**
   - [Docker Desktop for Mac](https://www.docker.com/products/docker-desktop)をダウンロードしてインストール
   - または、Homebrewを使用してインストール:
   ```bash
   brew install --cask docker
   ```

2. **プロジェクトのクローンと起動**
```bash
git clone https://github.com/enablerdao/ShardX.git
cd ShardX
docker-compose up --build
```

#### Windows

1. **Dockerのインストール**
   - [Docker Desktop for Windows](https://www.docker.com/products/docker-desktop)をダウンロードしてインストール
   - WSL2が有効になっていることを確認（Windows 10/11）

2. **プロジェクトのクローンと起動**
```bash
git clone https://github.com/enablerdao/ShardX.git
cd ShardX
docker-compose up --build
```

### Docker不要の開発環境構築

Docker環境を使用せずに開発する場合は、以下の手順で環境を構築できます：

#### 前提条件
- Rust (1.75以上)
- Node.js (v14以上)

#### Rustのインストール
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"  # Linux/macOS
# Windowsの場合は、インストーラーの指示に従ってください
```

#### Node.jsのインストール
```bash
# Ubuntuの場合
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# macOSの場合
brew install node

# Windowsの場合
# https://nodejs.org/からインストーラーをダウンロードしてインストール
```

#### プロジェクトのクローンと構築

```bash
# リポジトリのクローン
git clone https://github.com/enablerdao/ShardX.git
cd ShardX

# Rustの依存関係をインストール
cargo build

# Webサーバーの依存関係をインストール
cd web
npm install
cd ..
```

#### 開発モードでの実行

```bash
# 開発モードでノードとWebサーバーを起動
./run_dev.sh  # Linux/macOS
# Windowsの場合は、run_dev.batを実行
```

### データソースの切り替え

Webインターフェースでは、以下の3つのデータソースを切り替えることができます：

1. **モックデータ**: ランダムに生成されたデータを表示（デフォルト）
2. **テストデータ**: 高負荷テスト用の大きな値を持つデータを表示
3. **実ノード接続**: ローカルで実行中のノードに接続してリアルデータを表示

データソースは、Webインターフェース上部の「データソース」ドロップダウンメニューから切り替えることができます。

### 主要コンポーネントの実装詳細

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/web/assets/system_architecture.svg" alt="ShardX コンポーネント間の相互作用" width="80%"/>
</div>

#### 1. トランザクション処理
トランザクション処理の流れは以下の通りです：

1. クライアントがAPIを通じてトランザクションを送信
2. AIがトランザクションの優先度を計算し、優先キューに追加
3. シャーディングマネージャがトランザクションの所属シャードを決定
4. 所属シャードがローカルの場合、コンセンサスエンジンが処理
5. バリデータがトランザクションを検証し、過半数の承認で確定
6. DAGにトランザクションが追加され、状態が「確認済み」に更新

実装例（トランザクション送信）:
```rust
// トランザクションの作成
let tx = Transaction::new(
    vec!["parent1", "parent2"], // 親トランザクションID
    payload.as_bytes().to_vec(), // ペイロード
    signature.as_bytes().to_vec(), // 署名
);

// ノードにトランザクションを送信
node.submit_transaction(tx).await?;
```

#### 2. シャーディングの実装
シャーディングの実装は以下の通りです：

1. トランザクションIDのハッシュ値を計算
2. ハッシュ値を現在のシャード数で割った余りがシャードID
3. 負荷に応じてシャード数を動的に調整（256〜512）
4. 異なるシャードへのトランザクション転送はメッセージキューを使用

実装例（シャード割り当て）:
```rust
// トランザクションのシャードを決定
let shard_id = sharding_manager.assign_shard(&tx);

// 別のシャードに転送が必要な場合
if shard_id != current_shard_id {
    cross_shard_manager.send_cross_shard(tx, shard_id).await?;
}
```

#### 3. AIによる優先順位付け
AIによる優先順位付けの実装は以下の通りです：

1. トランザクションの特性（サイズ、親数、タイムスタンプ）を分析
2. 特性に基づいて優先スコアを計算
3. 優先スコアに基づいてバイナリヒープでトランザクションを管理
4. 優先度の高いトランザクションから順に処理

実装例（優先度計算）:
```rust
// トランザクションの優先度を計算
let size_score = 1000 - tx.payload.len().min(1000) as u32;
let parent_score = tx.parent_ids.len() as u32 * 100;
let time_score = (current_time - tx.timestamp).min(1000) as u32;

// 総合スコアを計算
let priority = size_score + parent_score + time_score;
```

### テスト方法

#### 単体テスト
```bash
# すべてのテストを実行
cargo test

# 特定のモジュールのテストを実行
cargo test --package hyperflux --lib transaction
```

#### 性能テスト
```bash
# TPSベンチマークを実行
cargo bench --bench tps_benchmark
```

#### 負荷テスト
```bash
# 100ノードでの負荷テスト（Dockerが必要）
./scripts/load_test.sh 100
```

## デプロイガイド

### ローカル環境へのデプロイ
```bash
# 開発モードで実行
./run_dev.sh

# または、Dockerで実行
docker-compose up
```

### 本番環境へのデプロイ
```bash
# 本番用の設定ファイルを使用
cp config/production.toml config/config.toml

# Dockerで本番モードで実行
docker-compose -f docker-compose.prod.yml up -d
```

### Webインターフェースのデプロイ
Webインターフェースは以下の方法でデプロイできます：

1. **Netlify**
   - GitHubリポジトリと連携
   - ビルドディレクトリを`web`に設定
   - 自動デプロイの設定

2. **Vercel**
   - GitHubリポジトリと連携
   - ルートディレクトリを`web`に設定
   - 自動デプロイの設定

3. **AWS S3 + CloudFront**
   - S3バケットにwebディレクトリの内容をアップロード
   - CloudFrontでCDN配信を設定

## 開発ロードマップ

### フェーズ1（現在）
- 基本的なノード構造とP2P通信
- シンプルなトランザクション処理
- Webインターフェースによるシステム監視
- 目標: 50,000 TPS

#### マルチノードテスト結果

最新のマルチノードテストでは、複数のノードを起動し、ノード間でトランザクションを送信して、分散型ブロックチェーンネットワークの機能を検証しました。

##### テスト環境

- **ノード数**: 2
- **ノードID**: node1, node3
- **ポート**: 54868, 54870
- **シャード数**: 256
- **バリデータ数**: 4

##### トランザクションテスト

ノード間で複数のトランザクションを送信し、すべてのトランザクションが正常に処理されたことを確認しました：

```bash
# ノード1へのトランザクション送信
curl -X POST -H "Content-Type: application/json" \
  -d '{"parent_ids":[],"payload":"SGVsbG8sIEh5cGVyRmx1eCE=","signature":"MHgxYTJiM2M0ZDVlNmY="}' \
  http://localhost:54868/transactions

# ノード3への親トランザクション参照付きトランザクション送信
curl -X POST -H "Content-Type: application/json" \
  -d '{"parent_ids":["b078712b-abf0-4405-986c-1285d85a087f"],"payload":"VHJhbnNmZXIgZnJvbSBub2RlMSB0byBub2RlMw==","signature":"MHgxYTJiM2M0ZDVlNmY="}' \
  http://localhost:54870/transactions
```

##### 結論

ShardXのマルチノードテストは成功しました。各ノードは正常に起動し、トランザクションを受け付けることができました。また、2番目のトランザクションでは最初のトランザクションを親として参照することができ、DAG（有向非巡回グラフ）構造の基本的な機能が動作していることを確認しました。

詳細なテスト結果は[こちら](./test_results/transaction_test.md)をご覧ください。

### フェーズ2
- スケーラビリティの向上
- AIモデルの高度化
- セキュリティの強化
- 目標: 100,000 TPS

### フェーズ3
- 本番環境へのデプロイ
- エコシステムの拡大
- サードパーティ開発者向けSDKの提供
- 目標: グローバル規模での採用

## トラブルシューティング

### よくある問題と解決策

1. **ノードが起動しない**
   - ログを確認: `RUST_LOG=debug cargo run`
   - ポートの競合を確認: `lsof -i :54867`
   - 依存関係を更新: `cargo update`

2. **トランザクションが処理されない**
   - バリデータの状態を確認
   - ネットワーク接続を確認
   - シャーディング設定を確認

3. **パフォーマンスが低い**
   - システムリソースを確認
   - ログレベルを下げる: `RUST_LOG=info`
   - シャード数を増やす: 設定ファイルで`initial_shards`を調整

## コントリビューションガイド

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/web/assets/user_flow.svg" alt="ShardX エコシステム" width="80%"/>
</div>

1. このリポジトリをフォーク
2. 機能ブランチを作成: `git checkout -b feature/amazing-feature`
3. 変更をコミット: `git commit -m 'Add amazing feature'`
4. ブランチをプッシュ: `git push origin feature/amazing-feature`
5. プルリクエストを作成

## 📞 お問い合わせ

- **公式サイト**: [https://shardx.org](https://shardx.org)
- **GitHub**: [https://github.com/enablerdao/ShardX](https://github.com/enablerdao/ShardX)
- **Twitter**: [@ShardXOrg](https://twitter.com/ShardXOrg)
- **Discord**: [ShardX Community](https://discord.gg/shardx)
- **メール**: [info@shardx.org](mailto:info@shardx.org)

## ライセンス
MIT