# マルチプラットフォームデプロイガイド

このガイドでは、ShardXを以下の主要なクラウドプラットフォームにデプロイする方法を説明します：

- [Render](#render)
- [Railway](#railway)
- [Heroku](#heroku)

各プラットフォームには、それぞれの特徴と利点があります。プロジェクトの要件に最も適したプラットフォームを選択してください。

## Render

[Render](https://render.com/)は、静的サイト、Webサービス、データベースなどを簡単にデプロイできるクラウドプラットフォームです。無料プランが利用可能で、ShardXの開発やテスト環境に最適です。

### デプロイ手順

1. [Renderアカウント](https://dashboard.render.com/register)を作成します
2. 以下のボタンをクリックしてデプロイを開始します：

[![Deploy to Render](https://render.com/images/deploy-to-render-button.svg)](https://render.com/deploy?repo=https://github.com/enablerdao/ShardX)

3. デプロイが完了すると、以下のサービスが自動的に作成されます：
   - `shardx-node`: ShardXのメインノード
   - `shardx-web`: Webインターフェース
   - `redis`: キャッシュとメッセージングに使用
   - `shardx-worker`: バックグラウンド処理用ワーカー

### 設定オプション

Renderでは、`render.yaml`ファイルを通じて以下の設定が可能です：

- シャード数の調整（`INITIAL_SHARDS`環境変数）
- ログレベルの設定（`RUST_LOG`環境変数）
- データ保存先の指定（`DATA_DIR`環境変数）

詳細な設定については、[Renderのドキュメント](https://render.com/docs)を参照してください。

## Railway

[Railway](https://railway.app/)は、アプリケーションのデプロイと管理を簡素化するプラットフォームです。GitHubリポジトリと連携して、自動デプロイが可能です。

### デプロイ手順

1. [Railwayアカウント](https://railway.app/login)を作成します
2. 以下のボタンをクリックしてデプロイを開始します：

[![Deploy on Railway](https://railway.app/button.svg)](https://railway.app/template/ShardX)

3. デプロイが完了すると、以下のサービスが自動的に作成されます：
   - ShardXノード
   - Webインターフェース
   - Redisインスタンス

### 設定オプション

Railwayでは、`railway.json`ファイルを通じて以下の設定が可能です：

- サービスの定義と環境変数の設定
- Redisなどのプラグインの追加
- デプロイ設定のカスタマイズ

詳細な設定については、[Railwayのドキュメント](https://docs.railway.app/)を参照してください。

## Heroku

[Heroku](https://www.heroku.com/)は、クラウドアプリケーションプラットフォームとして長い歴史を持ち、多くの開発者に利用されています。

### デプロイ手順

1. [Herokuアカウント](https://signup.heroku.com/)を作成します
2. Heroku CLIをインストールします：
   ```bash
   curl https://cli-assets.heroku.com/install.sh | sh
   ```

3. ログインしてアプリを作成します：
   ```bash
   heroku login
   heroku create shardx-app
   ```

4. GitリポジトリをHerokuにプッシュします：
   ```bash
   git push heroku main
   ```

または、以下のボタンをクリックして直接デプロイすることもできます：

[![Deploy to Heroku](https://www.herokucdn.com/deploy/button.svg)](https://heroku.com/deploy?template=https://github.com/enablerdao/ShardX)

### 設定オプション

Herokuでは、以下のファイルを通じて設定が可能です：

- `app.json`: アプリケーションの基本設定
- `Procfile`: 実行するプロセスの定義
- `heroku.yml`: Dockerコンテナを使用する場合の設定

詳細な設定については、[Herokuのドキュメント](https://devcenter.heroku.com/categories/reference)を参照してください。

## 共通の設定項目

どのプラットフォームでも、以下の共通設定が可能です：

| 環境変数 | 説明 | デフォルト値 |
|---------|------|------------|
| `NODE_ID` | ノードの一意の識別子 | プラットフォーム名_node |
| `RUST_LOG` | ログレベル | info |
| `INITIAL_SHARDS` | 初期シャード数 | 32 |
| `DATA_DIR` | データ保存ディレクトリ | /app/data |
| `REDIS_ENABLED` | Redisを使用するかどうか | true |
| `WEB_ENABLED` | Webインターフェースを有効にするかどうか | true |

## トラブルシューティング

### Render

- **問題**: デプロイ後にサービスが起動しない
  **解決策**: ログを確認し、必要なリソースが不足していないか確認してください。無料プランではリソースに制限があります。

### Railway

- **問題**: サービス間の接続エラー
  **解決策**: 環境変数が正しく設定されているか確認してください。特に`REDIS_URL`などの接続文字列を確認します。

### Heroku

- **問題**: ビルドエラー
  **解決策**: `heroku logs --tail`コマンドでログを確認し、ビルドプロセスのエラーを特定してください。

## パフォーマンスの最適化

各プラットフォームでのパフォーマンスを最適化するためのヒント：

1. **シャード数の調整**: 無料プランでは、シャード数を10-32程度に設定することをお勧めします
2. **ログレベル**: 本番環境では`info`または`warn`に設定してください
3. **永続ストレージ**: 重要なデータには、プラットフォームが提供する永続ストレージを使用してください

## 次のステップ

デプロイが完了したら、以下の手順でShardXの機能を確認できます：

1. Webインターフェースにアクセスして、ダッシュボードを確認
2. APIエンドポイントを使用して、トランザクションを作成
3. パフォーマンスモニタリングを設定して、システムの動作を監視

詳細については、[ShardXドキュメント](../README.md)を参照してください。