# デプロイのトラブルシューティング

このドキュメントでは、各クラウドプラットフォームでのデプロイ時に発生する可能性のある問題と、その解決方法を説明します。

## 目次

- [Render](#render)
- [Railway](#railway)
- [Heroku](#heroku)
- [Fly.io](#flyio)
- [一般的な問題](#一般的な問題)

## Render

### 問題: `headers only supported for static web services`

**症状**: Renderへのデプロイ時に、以下のエラーが表示される：
```
A render.yaml file was found, but there was an issue.
services[1].headers
headers only supported for static web services
```

**解決策**:
1. `render.yaml`ファイルから`headers`セクションを削除します
2. 代わりに環境変数`CORS_ENABLED`を追加してCORS設定を有効化します
3. 修正済みの`deploy-render-fix.sh`スクリプトを使用してデプロイします

### 問題: ビルドが失敗する

**症状**: Renderでのビルドプロセスが失敗する

**解決策**:
1. Renderダッシュボードでビルドログを確認します
2. 依存関係の問題がある場合は、`Dockerfile.simple`を使用してデプロイします
3. メモリ不足の場合は、有料プランにアップグレードするか、`INITIAL_SHARDS`の値を小さくします

## Railway

### 問題: デプロイが進まない

**症状**: Railwayでのデプロイが進まず、ステータスが「Building」のままになる

**解決策**:
1. Railwayダッシュボードでビルドログを確認します
2. プロジェクトを削除して、再度デプロイを試みます
3. `railway.json`ファイルの設定を確認し、必要に応じて修正します

### 問題: サービス間の接続エラー

**症状**: サービスが起動するが、Redis接続などでエラーが発生する

**解決策**:
1. 環境変数が正しく設定されているか確認します
2. Railwayダッシュボードで各サービスのステータスを確認します
3. 必要に応じて、サービスを再起動します

## Heroku

### 問題: `launch manifest was created for a app, but this is a app`

**症状**: Herokuへのデプロイ時に、以下のエラーが表示される：
```
Generate requirements for build
Scanning source code
Detected a Dockerfile app
Error: launch manifest was created for a app, but this is a app
```

**解決策**:
1. `heroku.yml`ファイルを簡略化します：
```yaml
build:
  docker:
    web: Dockerfile
```
2. `Dockerfile.heroku`を使用してデプロイします
3. 修正済みの`deploy-heroku-fix.sh`スクリプトを使用してデプロイします

### 問題: ビルドが失敗する

**症状**: Herokuでのビルドプロセスが失敗する

**解決策**:
1. `heroku logs --tail --app YOUR_APP_NAME`でログを確認します
2. Dockerfileが正しいか確認します
3. スタックが`container`に設定されているか確認します：
```bash
heroku stack:set container --app YOUR_APP_NAME
```

## Fly.io

### 問題: デプロイエラー

**症状**: Fly.ioへのデプロイ時にエラーが発生する

**解決策**:
1. `fly.toml`ファイルの構造を最新のFly.io仕様に合わせて修正します
2. `[services]`セクションを`[[services]]`に変更します
3. 修正済みの`deploy-fly-fix.sh`スクリプトを使用してデプロイします

### 問題: ボリュームマウントエラー

**症状**: ボリュームのマウントに関するエラーが発生する

**解決策**:
1. ボリュームが正しく作成されているか確認します：
```bash
flyctl volumes list
```
2. 必要に応じて、新しいボリュームを作成します：
```bash
flyctl volumes create shardx_data --size 1 --region YOUR_REGION
```

## 一般的な問題

### 問題: 環境変数が正しく設定されていない

**症状**: アプリケーションが起動するが、正常に動作しない

**解決策**:
1. 各プラットフォームのダッシュボードで環境変数を確認します
2. 必要な環境変数がすべて設定されているか確認します
3. 特に以下の環境変数が重要です：
   - `PORT`: APIが使用するポート
   - `NODE_ID`: ノードの一意の識別子
   - `RUST_LOG`: ログレベル
   - `INITIAL_SHARDS`: 初期シャード数
   - `DATA_DIR`: データ保存ディレクトリ
   - `REDIS_ENABLED`: Redisを使用するかどうか

### 問題: メモリ不足

**症状**: アプリケーションが起動後にクラッシュする、またはパフォーマンスが低下する

**解決策**:
1. `INITIAL_SHARDS`の値を小さくします（例：10）
2. より多くのメモリを持つプランにアップグレードします
3. 不要なサービスを無効化します

### 問題: ネットワーク接続の問題

**症状**: サービス間の通信ができない

**解決策**:
1. 各サービスのURLが正しく設定されているか確認します
2. CORSが正しく設定されているか確認します
3. ファイアウォールやネットワーク設定を確認します

---

問題が解決しない場合は、[GitHub Issues](https://github.com/enablerdao/ShardX/issues)で報告してください。