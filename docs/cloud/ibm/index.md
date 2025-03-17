# IBM Cloud へのデプロイガイド

このディレクトリには、ShardXをIBM Cloudにデプロイするために必要なファイルが含まれています。

## 前提条件

- [IBM Cloud アカウント](https://cloud.ibm.com/registration)
- [IBM Cloud CLI](https://cloud.ibm.com/docs/cli?topic=cli-install-ibmcloud-cli)

## デプロイ手順

### 1. IBM Cloud CLIにログイン

```bash
ibmcloud login
```

### 2. 組織とスペースを選択

```bash
ibmcloud target --cf
```

### 3. アプリケーションのデプロイ

```bash
cd /path/to/ShardX
ibmcloud cf push -f ibm-cloud/manifest.yml
```

### 4. デプロイの確認

```bash
ibmcloud cf apps
```

## アクセス方法

デプロイが完了すると、以下のURLでアクセスできます：

- Webインターフェース: https://shardx.mybluemix.net
- API: https://shardx-node.mybluemix.net

## スケーリング

アプリケーションをスケールするには：

```bash
# インスタンス数を増やす
ibmcloud cf scale shardx-node -i 3

# メモリを増やす
ibmcloud cf scale shardx-node -m 2G
```

## ログの確認

```bash
ibmcloud cf logs shardx-node --recent
```

## トラブルシューティング

### アプリケーションが起動しない場合

```bash
ibmcloud cf logs shardx-node --recent
```

### サービスの問題

```bash
ibmcloud cf services
ibmcloud cf service-keys shardx-redis
```

## リソースのクリーンアップ

```bash
ibmcloud cf delete shardx-node -f
ibmcloud cf delete shardx-web -f
ibmcloud cf delete-service shardx-redis -f
```