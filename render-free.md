# Renderの無料プランでのデプロイ方法

Renderの無料プランでShardXをデプロイする方法を説明します。

## 無料プランの制限事項

Renderの無料プランには以下の制限があります：

1. **永続ディスクなし**: 無料プランではディスク機能が使用できません
2. **リソース制限**: CPUとメモリが制限されています
3. **アイドル時停止**: 15分間アクセスがないとサービスが停止します
4. **月間稼働時間**: 750時間/月（単一サービスの場合は常時稼働可能）

## 対応策

ShardXは以下の対応を行っています：

1. **一時ディレクトリの使用**: データを `/tmp/shardx-data` に保存
2. **軽量設定**: シャード数を10に削減（デフォルトは256）
3. **最小限のリソース使用**: メモリ使用量を最適化

## デプロイ手順

1. 以下のボタンをクリックしてRenderにデプロイします：

   [![Deploy to Render](https://render.com/images/deploy-to-render-button.svg)](https://render.com/deploy?repo=https://github.com/enablerdao/ShardX)
   
   > **注意**: Renderの仕様変更により、デプロイボタンがうまく機能しない場合は、以下の手順で手動デプロイしてください。

### 手動デプロイ手順

1. [Render](https://render.com)にアカウント登録
2. ダッシュボードから「New +」→「Blueprint」を選択
3. GitHubリポジトリ `https://github.com/enablerdao/ShardX` を接続
4. 「Apply」をクリック

デプロイが完了したら、以下のURLでアクセスできます：
   - Webインターフェース: `https://shardx-web.onrender.com`
   - API: `https://shardx-node.onrender.com`

## 注意事項

無料プランでは以下の点に注意してください：

1. **データの永続性なし**: サービスが再起動するとデータは失われます
2. **テスト用途のみ**: 本番環境での使用は推奨されません
3. **パフォーマンス制限**: 高負荷のテストには適していません

## 有料プランへのアップグレード

より高いパフォーマンスと永続ディスクが必要な場合は、有料プランにアップグレードしてください。有料プランでは以下の設定が可能です：

```yaml
services:
  - type: web
    name: shardx-node
    plan: starter # 無料プランから有料プランに変更
    disk:
      name: shardx-data
      mountPath: /app/data
      sizeGB: 10 # ディスク容量を設定
    envVars:
      - key: DATA_DIR
        value: /app/data # 永続ディスクを使用
      - key: INITIAL_SHARDS
        value: "256" # シャード数を増やす
```