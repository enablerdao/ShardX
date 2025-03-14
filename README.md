# ShardX - 高性能ブロックチェーンプラットフォーム

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/web/assets/logo.png" alt="ShardX Logo" width="200" />
  <p>「トランザクションが川の流れのように速く、スムーズに動くブロックチェーン。」</p>
</div>

## 🚩 ミッション
「分散型テクノロジーで世界中の人々のつながりを深め、誰もが安心して価値を交換できる未来を実現する。」

## 🌟 特徴

- **高速処理**: Proof of Flow (PoF) コンセンサスにより、最大100,000 TPSを実現
- **スケーラビリティ**: 動的シャーディングで負荷に応じて自動的にスケール
- **セキュリティ**: 最新の暗号技術と分散型検証で高いセキュリティを確保
- **AI駆動**: トランザクションの優先順位付けと負荷予測にAIを活用
- **開発者フレンドリー**: 直感的なAPIと充実したドキュメント

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

Renderの無料プランでデプロイする方法については、[Renderデプロイガイド](render-free.md)を参照してください。

## 📊 パフォーマンス

ShardXは以下のパフォーマンスを実現しています：

- **フェーズ1**: 50,000 TPS（テスト環境）
- **フェーズ2**: 100,000 TPS（目標値）

### ベンチマーク結果

| 環境                | ノード数 | シャード数 | TPS     | レイテンシ |
|---------------------|---------|-----------|---------|-----------|
| ローカル（8コア）   | 1       | 10        | 45,000  | 12ms      |
| AWS t3.xlarge       | 10      | 50        | 78,000  | 25ms      |
| AWS c6g.16xlarge    | 100     | 256       | 95,000  | 18ms      |

## 🔧 アーキテクチャ

ShardXは以下の革新的なコンポーネントで構成されています：

### Proof of Flow (PoF) コンセンサス

DAG（有向非巡回グラフ）、PoH（Proof of History）、PoS（Proof of Stake）を組み合わせた革新的なコンセンサスメカニズムです。

- **DAG構造**: ブロックチェーンの代わりにDAG構造を採用し、トランザクションの並列処理を実現
- **PoH**: 各トランザクションに暗号学的に検証可能なタイムスタンプを付与
- **PoS**: バリデータがステークを保有し、トランザクションを検証

### 動的シャーディング

トラフィックに応じて自動的にシャード数を調整し、常に最適なパフォーマンスを維持します。

- **初期**: 256の支流（シャード）に分割
- **動的調整**: トランザクションが増えたら支流を増やし、減ったら減らす
- **クロスシャード通信**: 高速な非同期通信でシャード間のデータ交換を実現

### AI駆動型管理

AIがトランザクションの優先順位を最適化し、ネットワークの効率を向上させます。

- **優先順位付け**: AIが手数料や緊急性を判断して順番を決定
- **負荷予測**: 過去のデータから将来の負荷を予測し、リソースを最適化

## 📝 開発ロードマップ

- **フェーズ1** (2024 Q1-Q2): コアシステムの実装、テストネットの立ち上げ、50,000 TPSの達成
- **フェーズ2** (2024 Q3-Q4): スケーラビリティの向上、クロスチェーン機能の追加、100,000 TPSの達成
- **フェーズ3** (2025 Q1-Q2): スマートコントラクト機能、エコシステムの拡大、エンタープライズ向け機能
- **フェーズ4** (2025 Q3-Q4): グローバル展開、分散型アプリケーション、業界標準の確立

## 📚 ドキュメント

- [API リファレンス](docs/api/README.md)
- [開発者ガイド](docs/developers/README.md)
- [デプロイガイド](docs/deployment/README.md)
- [コントリビューションガイド](CONTRIBUTING.md)

## 🤝 コントリビューション

ShardXはオープンソースプロジェクトであり、コミュニティからの貢献を歓迎します。詳細は[コントリビューションガイド](CONTRIBUTING.md)を参照してください。

## 📄 ライセンス

ShardXはMITライセンスの下で公開されています。詳細は[LICENSE](LICENSE)ファイルを参照してください。

## 📞 お問い合わせ

- Twitter: [@ShardXOrg](https://twitter.com/ShardXOrg)
- Discord: [ShardX Community](https://discord.gg/shardx)
- Email: info@shardx.org