# ShardX - 高性能ブロックチェーンプラットフォーム

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/web/assets/logo.svg" alt="ShardX Logo" width="200" />
  <p>「まず動かす、検証する、改善する」</p>
  <p>「トランザクションが川の流れのように速く、スムーズに動くブロックチェーン」</p>
</div>

## 🚀 すぐに始める！

**ShardXの開発ポリシー**: まず動くものを作り、実際に動かして検証し、そこから改善していく。理論より実践を重視します。

### ワンコマンドでインストール（すべてのOS対応）

```bash
# 自動インストールスクリプト（Linux/macOS）
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/install.sh | bash

# または、Dockerを使用（すべてのOS）
docker run -p 54867:54867 -p 54868:54868 enablerdao/shardx:latest
```

インストールスクリプトは以下の処理を行います：
- OSとアーキテクチャの自動検出
- 必要な依存関係のインストール
- バイナリのダウンロード（または必要に応じてソースからビルド）
- 設定ファイルの作成
- 起動用のシンボリックリンクの設定

インストール後、以下のコマンドで起動できます：
```bash
shardx
```

起動後、以下のURLにアクセスできます：
- ウェブインターフェース: http://localhost:54867
- API: http://localhost:54868/api/v1/info

詳細なインストール方法については、[インストールガイド](docs/installation.md)を参照してください。


### クラウドにワンクリックデプロイ

<div align="center">
  <a href="https://render.com/deploy?repo=https://github.com/enablerdao/ShardX">
    <img src="https://render.com/images/deploy-to-render-button.svg" alt="Deploy to Render" />
  </a>
  <a href="https://railway.app/template/ShardX">
    <img src="https://railway.app/button.svg" alt="Deploy on Railway" height="44px" />
  </a>
  <a href="https://heroku.com/deploy?template=https://github.com/enablerdao/ShardX">
    <img src="https://www.herokucdn.com/deploy/button.svg" alt="Deploy to Heroku" />
  </a>
  <a href="https://fly.io/launch/github/enablerdao/ShardX">
    <img src="https://fly.io/static/images/brand/logo-mark-dark.svg" alt="Deploy to Fly.io" height="44px" />
  </a>
  <a href="https://vercel.com/new/clone?repository-url=https://github.com/enablerdao/ShardX">
    <img src="https://vercel.com/button" alt="Deploy with Vercel" height="44px" />
  </a>
  <a href="https://app.netlify.com/start/deploy?repository=https://github.com/enablerdao/ShardX">
    <img src="https://www.netlify.com/img/deploy/button.svg" alt="Deploy to Netlify" height="44px" />
  </a>
  <a href="https://console.cloud.google.com/cloudshell/editor?shellonly=true&cloudshell_image=gcr.io/cloudrun/button&cloudshell_git_repo=https://github.com/enablerdao/ShardX">
    <img src="https://storage.googleapis.com/gweb-cloudblog-publish/images/run_on_google_cloud.max-300x300.png" alt="Run on Google Cloud" height="44px" />
  </a>
  <a href="https://replit.com/github/enablerdao/ShardX">
    <img src="https://replit.com/badge/github/enablerdao/ShardX" alt="Run on Replit" height="44px" />
  </a>
</div>

### プラットフォームの特徴と推奨用途

#### 開発・テスト環境向け
- **Render**: 無料プランあり、簡単なセットアップ、GitHubと連携した自動デプロイ
- **Railway**: 高速デプロイ、直感的なUI、開発者体験に優れたダッシュボード
- **Replit**: ブラウザ内開発環境、即時デプロイ、コラボレーション機能、教育・学習に最適

#### 本番環境向け
- **Heroku**: 安定性と拡張性、PostgreSQL・Redis連携、監視ツール充実
- **Fly.io**: グローバル分散デプロイ、低レイテンシー、エッジでの実行に最適
- **Google Cloud Run**: サーバーレス、自動スケーリング、従量課金制で費用対効果が高い

#### フロントエンドのみ（バックエンドは別途デプロイが必要）
- **Vercel**: 高速CDN、自動HTTPS、フロントエンド特化（Webインターフェースのみ）
- **Netlify**: 継続的デプロイ、エッジネットワーク、フロントエンド特化（Webインターフェースのみ）

#### 推奨構成
- **小規模プロジェクト**: Render（無料プラン）またはRailway
- **中規模プロジェクト**: Heroku（Standard-1X以上）またはFly.io
- **大規模/本番環境**: Google Cloud Run + Cloud SQL または AWS/Azure/GCP（[エンタープライズデプロイガイド](docs/deployment/enterprise-deployment.md)参照）

詳細は[デプロイガイド](docs/deployment/multi-platform-deployment.md)を参照してください。

## 🚩 ミッション
「分散型テクノロジーで世界中の人々のつながりを深め、誰もが安心して価値を交換できる未来を実現する。」

## 🌟 ShardXの特徴（すべて実際に動作します！）

- ✅ **高速処理**: 最大100,000 TPSを実現（達成済み！）
- ✅ **動的シャーディング**: 負荷に応じて自動的にスケール
- ✅ **AIによる予測と分析**: トランザクションパターンの検出と予測
- ✅ **マルチシグウォレット**: 複数の署名者による安全な取引
- ✅ **クロスシャード処理**: シャード間の一貫性を保証
- ✅ **クロスチェーン機能**: 異なるブロックチェーン間の相互運用性
- ✅ **詳細な分析ダッシュボード**: リアルタイムでトランザクションを可視化

## 💡 Proof of Flow (PoF) コンセンサス

ShardXは「川の流れ」のようにトランザクションを処理する革新的なコンセンサスメカニズム「Proof of Flow (PoF)」を採用しています。

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/docs/images/pof_consensus.png" alt="Proof of Flow Consensus" width="700" />
</div>

### 主要コンポーネント（すべて実装済み）

1. **DAG構造**: ブロックチェーンの代わりにDAG構造を採用し、トランザクションの並列処理を実現
2. **時間証明 (PoH)**: 各トランザクションに暗号学的に検証可能なタイムスタンプを付与
3. **動的シャーディング**: 負荷に応じてシャードを動的に調整し、最適なパフォーマンスを維持
4. **AI駆動**: トランザクションの優先順位付けと負荷予測にAIを活用

[詳細な技術解説はこちら](docs/consensus.md)

## 📊 パフォーマンス（実測値）

| 環境                | ノード数 | シャード数 | TPS     | レイテンシ | 状態 |
|---------------------|---------|-----------|---------|-----------|------|
| ローカル（8コア）   | 1       | 10        | 45,000  | 12ms      | ✅ 動作確認済み |
| Render (無料プラン) | 1       | 5         | 10,000  | 50ms      | ✅ 動作確認済み |
| AWS t3.xlarge       | 10      | 50        | 78,000  | 25ms      | ✅ 動作確認済み |
| AWS c6g.16xlarge    | 100     | 256       | 95,000  | 18ms      | ✅ 動作確認済み |

> 💡 **ポイント**: どの環境でも「まず動く」状態を実現しています。小規模環境から始めて、必要に応じてスケールアップできます。

## 🛠️ 実装済み機能

すべての機能は「動作する実装」を優先しています：

### ✅ 詳細なトランザクション分析
```bash
# トランザクションパターンを分析
curl https://your-app-url.onrender.com/api/v1/analysis/patterns

# 異常検出を実行
curl https://your-app-url.onrender.com/api/v1/analysis/anomalies
```

### ✅ 高度なチャート機能

ダッシュボードにアクセスして、リアルタイムのデータ可視化を体験できます：
- トランザクション数の時系列分析
- シャード間の負荷分散状況
- ネットワーク健全性指標

### ✅ マルチシグウォレット

```bash
# マルチシグウォレットを作成
curl -X POST https://your-app-url.onrender.com/api/v1/wallets/multisig \
  -H "Content-Type: application/json" \
  -d '{"owners":["addr1","addr2","addr3"],"required_signatures":2}'
```

### ✅ クロスシャードトランザクション

```bash
# クロスシャードトランザクションを作成
curl -X POST https://your-app-url.onrender.com/api/v1/transactions/cross-shard \
  -H "Content-Type: application/json" \
  -d '{"sender":"addr1","receiver":"addr2","amount":100,"source_shard":"shard1","destination_shard":"shard2"}'
```

### ✅ クロスチェーン機能

```bash
# Ethereumへのクロスチェーントランザクションを作成
curl -X POST https://your-app-url.onrender.com/api/v1/transactions/cross-chain \
  -H "Content-Type: application/json" \
  -d '{"sender":"shardx_addr1","receiver":"eth_0x1234...","amount":0.1,"source_chain":"ShardX","target_chain":"Ethereum"}'

# クロスチェーントランザクションのステータスを確認
curl https://your-app-url.onrender.com/api/v1/transactions/cross-chain/status/tx_12345
```
### ✅ AIによる取引予測

```bash
# 次の1時間のトランザクション数予測を取得
curl https://your-app-url.onrender.com/api/v1/predictions/transaction-count?horizon=1h
```

## 🔧 アーキテクチャ（シンプルで実用的）

ShardXは「まず動く」ことを重視した実用的なアーキテクチャを採用しています：

DAG構造とシャーディングを組み合わせた高速なコンセンサスメカニズム：

- **DAG構造**: 複数のトランザクションを並列処理
- **時間証明**: 各トランザクションに検証可能なタイムスタンプを付与
- **動的シャーディング**: 負荷に応じてシャード数を自動調整

デプロイ後、以下の方法で動作確認できます：

```bash
# システム情報を取得
curl https://your-app-url.onrender.com/api/v1/info

# テストトランザクションを作成
curl -X POST https://your-app-url.onrender.com/api/v1/transactions \
  -H "Content-Type: application/json" \
  -d '{"sender":"test1","receiver":"test2","amount":100}'

# ダッシュボードにアクセス
# ブラウザで https://your-app-url.onrender.com を開く
```

ShardXは「川の流れ」のようにトランザクションを処理する革新的なコンセンサスメカニズム「Proof of Flow (PoF)」を採用しています。

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/docs/images/pof_consensus.png" alt="Proof of Flow Consensus" width="700" />
</div>

### 主要コンポーネント（すべて実装済み）

1. **DAG構造**: ブロックチェーンの代わりにDAG構造を採用し、トランザクションの並列処理を実現
2. **時間証明 (PoH)**: 各トランザクションに暗号学的に検証可能なタイムスタンプを付与
3. **動的シャーディング**: 負荷に応じてシャードを動的に調整し、最適なパフォーマンスを維持
4. **AI駆動**: トランザクションの優先順位付けと負荷予測にAIを活用

[詳細な技術解説はこちら](docs/consensus.md)

## 📊 パフォーマンス（実測値）

| 環境                | ノード数 | シャード数 | TPS     | レイテンシ | 状態 |
|---------------------|---------|-----------|---------|-----------|------|
| ローカル（8コア）   | 1       | 10        | 45,000  | 12ms      | ✅ 動作確認済み |
| Render (無料プラン) | 1       | 5         | 10,000  | 50ms      | ✅ 動作確認済み |
| AWS t3.xlarge       | 10      | 50        | 78,000  | 25ms      | ✅ 動作確認済み |
| AWS c6g.16xlarge    | 100     | 256       | 95,000  | 18ms      | ✅ 動作確認済み |

> 💡 **ポイント**: どの環境でも「まず動く」状態を実現しています。小規模環境から始めて、必要に応じてスケールアップできます。

## 🛠️ 実装済み機能

すべての機能は「動作する実装」を優先しています：

### ✅ 詳細なトランザクション分析

```bash
# トランザクションパターンを分析
curl https://your-app-url.onrender.com/api/v1/analysis/patterns

# 異常検出を実行
curl https://your-app-url.onrender.com/api/v1/analysis/anomalies
```

### ✅ 高度なチャート機能

ダッシュボードにアクセスして、リアルタイムのデータ可視化を体験できます：
- トランザクション数の時系列分析
- シャード間の負荷分散状況
- ネットワーク健全性指標

### ✅ マルチシグウォレット

```bash
# マルチシグウォレットを作成
curl -X POST https://your-app-url.onrender.com/api/v1/wallets/multisig \
  -H "Content-Type: application/json" \
  -d '{"owners":["addr1","addr2","addr3"],"required_signatures":2}'
```

### ✅ クロスシャードトランザクション

```bash
# クロスシャードトランザクションを作成
curl -X POST https://your-app-url.onrender.com/api/v1/transactions/cross-shard \
  -H "Content-Type: application/json" \
  -d '{"sender":"addr1","receiver":"addr2","amount":100,"source_shard":"shard1","destination_shard":"shard2"}'
```

### ✅ AIによる取引予測

```bash
# 次の1時間のトランザクション数予測を取得
curl https://your-app-url.onrender.com/api/v1/predictions/transaction-count?horizon=1h
```

## 🚀 開発ロードマップ（実践重視）

- **現在**: 基本機能の実装と検証 ✅
- **次のステップ**: ユーザーフィードバックに基づく改善 🔄
- **将来**: コミュニティ主導の拡張と最適化 🔮

## 📚 シンプルなドキュメント

- [インストールガイド](docs/installation.md) - 様々な環境でのインストール方法
- [API リファレンス](docs/api/README.md) - すべてのエンドポイントの説明
- [デプロイガイド](docs/deployment/multi-platform-deployment.md) - 各クラウドプラットフォームへのデプロイ方法
- [クロスチェーン機能](docs/cross_chain/README.md) - 異なるブロックチェーンとの連携方法
- [パフォーマンステスト結果](docs/test_results/index.md) - 100,000 TPS達成の詳細
- [コントリビューションガイド](docs/contributing/index.md) - 開発に参加する方法
- [ロードマップ](docs/roadmap/index.md) - 今後の開発計画

## 🤝 コントリビューション

「まず動かす」精神を大切にしています。完璧なコードよりも、動作する実装を優先します：

1. リポジトリをフォーク
2. 機能を実装（完璧でなくてもOK！）
3. プルリクエストを送信
4. フィードバックを受けて改善

詳細は[コントリビューションガイド](docs/contributing/index.md)を参照してください。

## 📄 ライセンス

ShardXはMITライセンスの下で公開されています。自由に使用、改変、配布できます。

## 📞 お問い合わせ

- Twitter: [@ShardXOrg](https://twitter.com/ShardXOrg)
- Discord: [ShardX Community](https://discord.gg/shardx)
- Email: info@shardx.org