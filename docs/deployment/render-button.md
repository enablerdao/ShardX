# Renderでのデプロイ

ShardXは[Render](https://render.com)の無料プランでデプロイできるように最適化されています。

## ワンクリックデプロイ

以下のボタンをクリックして、ShardXをRenderにデプロイできます：

[![Deploy to Render](https://render.com/images/deploy-to-render-button.svg)](https://render.com/deploy?repo=https://github.com/enablerdao/ShardX)

## 無料プランでの制限事項

Renderの無料プランでは以下の制限があります：

1. **リソース制限**: CPUとメモリが制限されています
2. **ディスク容量**: 1GBまで
3. **アイドル時停止**: 15分間アクセスがないとサービスが停止します
4. **月間稼働時間**: 750時間/月（単一サービスの場合は常時稼働可能）

これらの制限に対応するため、ShardXは以下の最適化を行っています：

- シャード数を10に削減（デフォルトは256）
- データベースとRedisを使用せず、ファイルベースのストレージを使用
- 軽量なDockerイメージを使用

## 手動デプロイ手順

1. [Render](https://render.com)にアカウント登録
2. 「New +」ボタンをクリック
3. 「Web Service」を選択
4. GitHubリポジトリ `https://github.com/enablerdao/ShardX` を接続
5. 以下の設定を行う：
   - Name: `shardx-node`
   - Environment: `Docker`
   - Dockerfile Path: `Dockerfile.simple`
   - Plan: `Free`
   - Advanced Settings:
     - Environment Variables:
       - `NODE_ID`: `render_node`
       - `PORT`: `54868`
       - `RUST_LOG`: `info`
       - `INITIAL_SHARDS`: `10`
6. 「Create Web Service」をクリック

## カスタマイズ

より高いパフォーマンスが必要な場合は、有料プランにアップグレードし、`render.yaml`の以下の設定を変更してください：

```yaml
services:
  - type: web
    name: shardx-node
    plan: starter # 無料プランから有料プランに変更
    envVars:
      - key: INITIAL_SHARDS
        value: "256" # シャード数を増やす
    disk:
      sizeGB: 10 # ディスク容量を増やす
```