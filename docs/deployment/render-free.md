# Renderでの無料デプロイガイド

このガイドでは、ShardXをRenderの無料プランを使用してデプロイする方法を説明します。

## 前提条件

- GitHubアカウント
- Renderアカウント（無料）

## 手順

### 1. Renderアカウントの作成

1. [Render](https://render.com/)にアクセスし、「Sign Up」をクリックします
2. GitHubアカウントでサインアップすることをお勧めします（連携が簡単になります）

### 2. ワンクリックデプロイ

最も簡単な方法は、以下のボタンをクリックすることです：

[![Deploy to Render](https://render.com/images/deploy-to-render-button.svg)](https://render.com/deploy?repo=https://github.com/enablerdao/ShardX)

### 3. 手動デプロイ（ワンクリックデプロイがうまくいかない場合）

1. Renderダッシュボードで「New +」→「Web Service」をクリックします
2. GitHubリポジトリを連携し、「enablerdao/ShardX」を選択します
3. 以下の設定を入力します：
   - **Name**: shardx-node
   - **Environment**: Docker
   - **Branch**: main
   - **Plan**: Free

4. 「Create Web Service」をクリックします

5. 次に、「New +」→「Web Service」をクリックして、ウェブインターフェースをデプロイします
6. 同じリポジトリを選択し、以下の設定を入力します：
   - **Name**: shardx-web
   - **Environment**: Node
   - **Branch**: main
   - **Build Command**: `cd web && npm install && npm run build`
   - **Start Command**: `cd web && npm start`
   - **Plan**: Free
   - **Advanced** → **Environment Variables**:
     - `PORT`: 52153
     - `API_URL`: https://shardx-node.onrender.com

7. 「Create Web Service」をクリックします

### 4. デプロイの確認

1. デプロイが完了するまで数分待ちます（Renderダッシュボードで進行状況を確認できます）
2. デプロイが完了したら、以下のURLにアクセスできます：
   - ウェブインターフェース: https://shardx-web.onrender.com
   - API: https://shardx-node.onrender.com/api/v1/info

## 無料プランの制限事項

Renderの無料プランには以下の制限があります：

- 毎月750時間の実行時間（1つのサービスを常時実行可能）
- 512MB RAM
- 共有CPU
- 15分間のアイドル後にスリープ状態になる（次のリクエストで自動的に起動）

これらの制限は開発やテスト目的には十分ですが、本番環境では有料プランへのアップグレードを検討してください。

## トラブルシューティング

### デプロイに失敗する場合

1. Renderダッシュボードでログを確認します
2. 一般的な問題：
   - メモリ不足: 無料プランでは512MBのRAMしか使用できません。設定で`INITIAL_SHARDS`を減らしてみてください。
   - ビルドタイムアウト: 無料プランではビルド時間が制限されています。Dockerfileを最適化してみてください。

### サービスがスリープから復帰しない場合

1. Renderダッシュボードでサービスを手動で再起動します
2. 無料プランでは15分間のアイドル後にスリープ状態になります。定期的なpingを設定して、サービスをアクティブに保つことができます。

## 次のステップ

- [API リファレンス](../api/README.md)を参照して、ShardX APIの使用方法を学びます
- [開発者ガイド](../developers/README.md)を参照して、ShardXの開発方法を学びます