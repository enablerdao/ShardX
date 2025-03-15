# マルチプラットフォームデプロイガイド

このガイドでは、ShardXを以下の主要なクラウドプラットフォームにデプロイする方法を説明します：

- [Render](#render)
- [Railway](#railway)
- [Heroku](#heroku)
- [Fly.io](#flyio)
- [統合デプロイスクリプト](#統合デプロイスクリプト)
- [トラブルシューティング](#トラブルシューティング)

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

## Fly.io

[Fly.io](https://fly.io/)は、アプリケーションをグローバルに分散デプロイできるプラットフォームです。ユーザーに近い場所でアプリケーションを実行することで、低レイテンシーを実現します。

### 特徴

- グローバル分散デプロイ
- 低レイテンシー
- 高可用性
- 無料枠あり（最大3つの小規模アプリ）

### デプロイ手順

1. [Fly.ioアカウント](https://fly.io/app/sign-up)を作成します
2. Fly CLIをインストールします：
   ```bash
   curl -L https://fly.io/install.sh | sh
   ```

3. ログインします：
   ```bash
   fly auth login
   ```

4. リポジトリに含まれるスクリプトを使用してデプロイします：
   ```bash
   # メインノードをデプロイ
   ./scripts/deploy-fly-fix.sh
   
   # Webインターフェースをデプロイ（オプション）
   ./scripts/deploy-fly-web.sh
   ```

5. スクリプト実行中に、アプリ名とリージョンを選択します

### 手動デプロイ

1. リポジトリをクローンします：
   ```bash
   git clone https://github.com/enablerdao/ShardX.git
   cd ShardX
   ```

2. Fly.ioアプリを作成します：
   ```bash
   fly apps create shardx-app
   ```

3. ボリュームを作成します：
   ```bash
   fly volumes create shardx_data --size 1
   ```

4. デプロイします：
   ```bash
   fly deploy
   ```

5. デプロイが完了したら、アプリにアクセスします：
   ```bash
   fly open
   ```

### 注意事項

Fly.ioでデプロイする際に「launch manifest was created for a app, but this is a app」というエラーが発生する場合は、`fly.toml`ファイルの構造を最新のFly.io仕様に合わせて修正する必要があります。修正済みの`deploy-fly-fix.sh`スクリプトを使用することで、この問題を回避できます。

## 統合デプロイスクリプト

複数のクラウドプラットフォームへのデプロイを簡単に行うために、統合デプロイスクリプト`deploy-all.sh`を用意しています。

### 使用方法

```bash
./scripts/deploy-all.sh
```

このスクリプトを実行すると、デプロイ先を選択するメニューが表示されます。選択したプラットフォームに応じて、適切なデプロイスクリプトが実行されます。

### 機能

- Render、Railway、Heroku、Fly.ioへのデプロイをサポート
- 各プラットフォームの特性に合わせた最適な設定を自動適用
- デプロイ中の問題に対するガイダンスを提供

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

デプロイ時に問題が発生した場合は、[トラブルシューティングガイド](troubleshooting.md)を参照してください。一般的な問題と解決策を以下に示します：

### Render

- **問題**: デプロイ後にサービスが起動しない
  **解決策**: ログを確認し、必要なリソースが不足していないか確認してください。無料プランではリソースに制限があります。

- **問題**: `headers only supported for static web services`エラー
  **解決策**: `render.yaml`ファイルから`headers`セクションを削除し、代わりに環境変数`CORS_ENABLED`を追加してCORS設定を有効化します。

### Railway

- **問題**: サービス間の接続エラー
  **解決策**: 環境変数が正しく設定されているか確認してください。特に`REDIS_URL`などの接続文字列を確認します。

- **問題**: デプロイが進まない
  **解決策**: Railwayダッシュボードでビルドログを確認し、プロジェクトを削除して再度デプロイを試みます。

### Heroku

- **問題**: ビルドエラー
  **解決策**: `heroku logs --tail`コマンドでログを確認し、ビルドプロセスのエラーを特定してください。

- **問題**: `launch manifest was created for a app, but this is a app`エラー
  **解決策**: `heroku.yml`ファイルを簡略化し、`Dockerfile.heroku`を使用してデプロイします。

### Fly.io

- **問題**: デプロイエラー
  **解決策**: `fly.toml`ファイルの構造を最新のFly.io仕様に合わせて修正します。

- **問題**: ボリュームマウントエラー
  **解決策**: ボリュームが正しく作成されているか確認し、必要に応じて新しいボリュームを作成します。

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