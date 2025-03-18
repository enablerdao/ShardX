# ShardX - 爆速・直感的ブロックチェーン

<div align="center">
  <img src="https://raw.githubusercontent.com/enablerdao/ShardX/main/web/assets/logo.svg" alt="ShardX Logo" width="200" />
  <h2>「複雑さゼロ、速さ無限大」</h2>
  <h4>開発者も利用者も、誰もが5分で使えるブロックチェーン</h4>
  
  <p align="center">
    <a href="#-たった1行で始める"><strong>始める »</strong></a> &nbsp;|&nbsp;
    <a href="#-圧倒的パフォーマンス実測値"><strong>性能 »</strong></a> &nbsp;|&nbsp;
    <a href="#-主な機能ハイライト"><strong>機能 »</strong></a>
  </p>
</div>

## ⚡ たった1行で始める

```bash
docker run -p 54867:54867 -p 54868:54868 yukih47/shardx:latest
```

これだけ！ブラウザで http://localhost:54867 にアクセス → 完了！

### 📱 各環境向けインストール

| 環境 | コマンド |
|------|---------|
| **Apple Silicon** | `docker run -p 54867:54867 -p 54868:54868 yukih47/shardx:latest-arm64` |
| **Linux** | `curl -fsSL https://raw.githubusercontent.com/enablerdao/ShardX/main/install.sh \| bash` |
| **macOS** | `brew install enablerdao/tap/shardx` |
| **Windows** | `winget install EnablerDAO.ShardX` |

[詳細はこちら](https://docs.shardx.io/installation)

### 👩‍💻 開発者向け

```bash
# クローン、ビルド、実行（3ステップ）
git clone https://github.com/enablerdao/ShardX.git && cd ShardX
cargo build --release
./target/release/shardx
```

[開発者ドキュメント](https://docs.shardx.io/developers)

## 💡 なぜShardXなのか？

| 開発者にとって | 利用者にとって |
|--------------|--------------|
| ✅ **シンプルAPI**: 数行のコードで統合 | ✅ **直感的UI**: 専門知識不要 |
| ✅ **多言語SDK**: JS/Python/Rust対応 | ✅ **爆速処理**: 最大10万TPS |
| ✅ **拡張性**: プラグインで機能拡張 | ✅ **低コスト**: 最小リソースで高性能 |

## 📊 圧倒的パフォーマンス（実測値）

<div align="center">
  <table>
    <tr>
      <th align="center">環境</th>
      <th align="center">TPS</th>
      <th align="center">レイテンシ</th>
    </tr>
    <tr>
      <td align="center"><b>ローカル（8コア）</b></td>
      <td align="center"><b>45,000</b></td>
      <td align="center">12ms</td>
    </tr>
    <tr>
      <td align="center"><b>AWS t3.xlarge</b></td>
      <td align="center"><b>78,000</b></td>
      <td align="center">25ms</td>
    </tr>
  </table>
</div>

## 🔍 簡単API（コピペで使える）

```bash
# トランザクション作成（たった1コマンド）
curl -X POST http://localhost:54868/api/v1/transactions \
  -H "Content-Type: application/json" \
  -d '{"sender":"wallet1","receiver":"wallet2","amount":100}'
```

## ☁️ クラウドデプロイ（ワンクリック）

<div align="center">
  <a href="https://render.com/deploy?repo=https://github.com/enablerdao/ShardX">
    <img src="https://render.com/images/deploy-to-render-button.svg" alt="Deploy to Render" />
  </a>
  <a href="https://railway.app/template/ShardX">
    <img src="https://railway.app/button.svg" alt="Deploy on Railway" height="44px" />
  </a>
</div>

## 🔧 サポート＆リソース

- **Discord**: [discord.gg/shardx](https://discord.gg/shardx)
- **ドキュメント**: [docs.shardx.io](https://docs.shardx.io)
- **GitHub**: [github.com/enablerdao/ShardX](https://github.com/enablerdao/ShardX)

## 💎 主な機能ハイライト

- **マルチシグウォレット**: 複数署名による高度なセキュリティ
- **クロスチェーン連携**: 異なるブロックチェーンとの相互運用
- **AIによる予測分析**: トランザクションパターンの検出と予測
- **スマートコントラクト**: 直感的インターフェースで簡単作成

## 📄 ライセンス

ShardXはMITライセンスの下で公開されています。自由に使用、改変、配布できます。


