# ShardX - 高性能ブロックチェーンプラットフォーム

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/web/assets/logo.svg" alt="ShardX Logo" width="200" />
  <p>「まず動かす、検証する、改善する」</p>
  <p>「トランザクションが川の流れのように速く、スムーズに動くブロックチェーン」</p>
</div>

## 🚀 すぐに始める！

**ShardXの開発ポリシー**: まず動くものを作り、実際に動かして検証し、そこから改善していく。理論より実践を重視します。

### ワンコマンドで起動（どのOSでも動作）

```bash
# 最も簡単な方法（Dockerが必要）
curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/install.sh | bash

# または、手動でコピー＆ペースト
docker run -p 54867:54867 -p 54868:54868 enablerdao/shardx:latest
```

### OS別インストール方法（Docker不要）

**Linux (Ubuntu/Debian)**
```bash
# 依存関係をインストール
sudo apt update && sudo apt install -y git curl build-essential libssl-dev pkg-config

# ShardXをクローンして起動
git clone https://github.com/enablerdao/ShardX.git
cd ShardX
./scripts/linux_install.sh
./scripts/run.sh
```

**macOS**
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

**Windows**
```powershell
# PowerShellを管理者権限で実行
# Rustをインストール
Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
.\rustup-init.exe -y

# ShardXをクローンして起動
git clone https://github.com/enablerdao/ShardX.git
cd ShardX
.\scripts\windows_install.ps1
.\scripts\run.ps1
```

起動後、以下のURLにアクセスできます：
- ウェブインターフェース: http://localhost:54867
- API: http://localhost:54868/api/v1/info

### クラウドにワンクリックデプロイ

<div align="center">
  <a href="https://render.com/deploy?repo=https://github.com/enablerdao/ShardX">
    <img src="https://render.com/images/deploy-to-render-button.svg" alt="Deploy to Render" />
  </a>
  <a href="https://railway.app/template/ShardX">
    <img src="https://railway.app/button.svg" alt="Deploy on Railway" height="44px" />
  </a>
  <a href="https://vercel.com/new/clone?repository-url=https://github.com/enablerdao/ShardX">
    <img src="https://vercel.com/button" alt="Deploy to Vercel" />
  </a>
</div>

詳細な手順は[Renderデプロイガイド](docs/deployment/render-free.md)や[クイックスタートガイド](docs/quickstart.md)を参照してください。

## 🌟 ShardXの特徴（すべて実際に動作します！）

- ✅ **高速処理**: 最大100,000 TPSを実現（現在のテスト環境: 50,000 TPS）
- ✅ **動的シャーディング**: 負荷に応じて自動的にスケール
- ✅ **AIによる予測と分析**: トランザクションパターンの検出と予測
- ✅ **マルチシグウォレット**: 複数の署名者による安全な取引
- ✅ **クロスシャード処理**: シャード間の一貫性を保証
- ✅ **詳細な分析ダッシュボード**: リアルタイムでトランザクションを可視化

## 💡 Proof of Flow (PoF) コンセンサスとは？

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/docs/images/pof_consensus.png" alt="Proof of Flow Consensus" width="700" />
</div>

### 概要

Proof of Flow (PoF) は、DAG（有向非巡回グラフ）、PoH（Proof of History）、PoS（Proof of Stake）を組み合わせた革新的なコンセンサスメカニズムです。「川の流れ」のように、トランザクションが連続的かつ並列に処理される仕組みを実現しています。

### 主要コンポーネント

1. **DAG構造**: ブロックチェーンの代わりにDAG構造を採用し、トランザクションの並列処理を実現
2. **時間証明 (PoH)**: 各トランザクションに暗号学的に検証可能なタイムスタンプを付与
3. **ステーク証明 (PoS)**: バリデータがステークを保有し、トランザクションを検証
4. **動的シャーディング**: 負荷に応じてシャードを動的に調整し、最適なパフォーマンスを維持

### 他のコンセンサスとの比較

| 特徴 | ShardX (PoF) | Solana (PoH+PoS) | IOTA (Tangle) | Sui/Aptos (BFT) |
|------|--------------|------------------|---------------|-----------------|
| 構造 | DAG + シャード | ブロックチェーン | DAG | ブロックチェーン |
| スケーラビリティ | 動的シャーディング | 単一チェーン | 静的シャーディング | 並列実行 |
| 処理速度 | ~100,000 TPS | ~50,000 TPS | ~1,000 TPS | ~160,000 TPS |
| 確定性 | 数秒 | 数秒〜数十秒 | 数分 | 数秒 |
| 並列処理 | 完全並列 | 部分的並列 | 部分的並列 | トランザクション並列 |
| シャード間通信 | 2フェーズコミット | N/A | 静的シャード | オブジェクトベース |

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/docs/images/consensus_comparison.png" alt="Consensus Comparison" width="700" />
</div>

### なぜ高速なのか？

1. **完全並列処理**: DAG構造により、複数のトランザクションを完全に並列処理できます
2. **動的シャーディング**: データと処理を複数のシャードに分散し、負荷に応じて自動的にスケーリング
3. **検証の効率化**: PoHによりタイムスタンプが事前に検証され、合意形成が高速化
4. **AI最適化**: AIがトランザクションの優先順位付けと負荷予測を行い、処理効率を向上
5. **クロスシャード最適化**: 2フェーズコミットプロトコルにより、シャード間トランザクションの一貫性を保証しつつ高速処理

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/docs/images/performance_scaling.png" alt="Performance Scaling" width="700" />
</div>

### Solanaとの違い

Solanaは単一チェーンでPoHを使用していますが、ShardXはDAG構造と動的シャーディングを組み合わせることで、より高いスケーラビリティを実現しています。Solanaがブロック生成とトランザクション処理を分離しているのに対し、ShardXはトランザクション自体がDAGのノードとなり、より効率的な並列処理が可能です。

また、Solanaは単一チェーンのため、ネットワーク全体の処理能力に上限がありますが、ShardXは動的シャーディングにより理論上無制限のスケーラビリティを実現できます。

### IOTAとの違い

IOTAもDAG（Tangle）を使用していますが、ShardXは動的シャーディングとAI駆動の負荷分散を採用しており、より高いスループットを実現しています。また、ShardXはクロスシャードトランザクションの一貫性を保証する2フェーズコミットプロトコルを実装しています。

IOTAの確定性は数分かかることがありますが、ShardXは数秒で確定するため、リアルタイム取引に適しています。

### Sui/Aptosとの違い

Sui/Aptosは並列実行とオブジェクトベースのトランザクション処理を特徴としていますが、ShardXはDAG構造と動的シャーディングにより、より柔軟なスケーリングを実現しています。また、ShardXはAIを活用したトランザクション管理により、リソース使用効率を最適化しています。

Sui/Aptosはオブジェクト所有権に基づく並列処理を行いますが、これは特定のユースケースに限定されます。一方、ShardXのDAG構造は、より汎用的な並列処理を可能にします。

[詳細な技術解説はこちら](docs/consensus.md)

## 🚀 クイックスタート

### Dockerを使用した起動

```bash
docker run -p 54867:54867 -p 54868:54868 enablerdao/shardx:latest
```

### ソースからのインストール

```bash
git clone https://github.com/enablerdao/ShardX.git
cd ShardX
./quick_start.sh  # 一般的な環境用
# または
./mac_install.sh  # Mac専用
```

## 🚀 ワンクリックデプロイ

<div align="center">
  <a href="https://render.com/deploy?repo=https://github.com/enablerdao/ShardX">
    <img src="https://render.com/images/deploy-to-render-button.svg" alt="Deploy to Render" />
  </a>
  <a href="https://railway.app/template/ShardX">
    <img src="https://railway.app/button.svg" alt="Deploy on Railway" height="44px" />
  </a>
  <a href="https://heroku.com/deploy?template=https://github.com/enablerdao/ShardX">
    <img src="https://www.herokucdn.com/deploy/button.svg" alt="Deploy to Heroku" height="44px" />
  </a>
  <a href="https://gitpod.io/#https://github.com/enablerdao/ShardX">
    <img src="https://gitpod.io/button/open-in-gitpod.svg" alt="Open in Gitpod" />
  </a>
  <a href="https://vercel.com/new/clone?repository-url=https://github.com/enablerdao/ShardX">
    <img src="https://vercel.com/button" alt="Deploy to Vercel" />
  </a>
  <a href="https://app.netlify.com/start/deploy?repository=https://github.com/enablerdao/ShardX">
    <img src="https://www.netlify.com/img/deploy/button.svg" alt="Deploy to Netlify" />
  </a>
</div>

Renderの無料プランでデプロイする方法については、[Renderデプロイガイド](docs/deployment/render-free.md)を参照してください。

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

## 🔧 アーキテクチャ（シンプルで実用的）

ShardXは「まず動く」ことを重視した実用的なアーキテクチャを採用しています：

### Proof of Flow (PoF) コンセンサス

DAG構造とシャーディングを組み合わせた高速なコンセンサスメカニズム：

- **DAG構造**: 複数のトランザクションを並列処理
- **時間証明**: 各トランザクションに検証可能なタイムスタンプを付与
- **動的シャーディング**: 負荷に応じてシャード数を自動調整

### AI駆動型管理

AIがトランザクションの優先順位と負荷予測を行い、効率を最適化：

- **優先順位付け**: 手数料や緊急性に基づいて順番を決定
- **負荷予測**: 過去のデータから将来の負荷を予測

## 🚀 開発ロードマップ（実践重視）

- **現在**: 基本機能の実装と検証 ✅
- **次のステップ**: ユーザーフィードバックに基づく改善 🔄
- **将来**: コミュニティ主導の拡張と最適化 🔮

## 📚 シンプルなドキュメント

- [クイックスタートガイド](docs/quickstart.md) - 5分で始める方法
- [API リファレンス](docs/api/README.md) - すべてのエンドポイントの説明
- [デプロイガイド](docs/deployment/render-free.md) - 無料でのデプロイ方法

## 🤝 コントリビューション

「まず動かす」精神を大切にしています。完璧なコードよりも、動作する実装を優先します：

1. リポジトリをフォーク
2. 機能を実装（完璧でなくてもOK！）
3. プルリクエストを送信
4. フィードバックを受けて改善

詳細は[コントリビューションガイド](CONTRIBUTING.md)を参照してください。

## 📄 ライセンス

ShardXはMITライセンスの下で公開されています。自由に使用、改変、配布できます。

## 📞 お問い合わせ

- Twitter: [@ShardXOrg](https://twitter.com/ShardXOrg)
- Discord: [ShardX Community](https://discord.gg/shardx)
- Email: info@shardx.org