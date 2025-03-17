# Oracle Cloud へのデプロイガイド

このディレクトリには、ShardXをOracle Cloud Infrastructure (OCI)にデプロイするためのTerraformスクリプトが含まれています。

## 前提条件

- [Oracle Cloud アカウント](https://www.oracle.com/cloud/free/)
- [Terraform](https://www.terraform.io/downloads.html) (バージョン1.0.0以上)
- [OCI CLI](https://docs.oracle.com/en-us/iaas/Content/API/SDKDocs/cliinstall.htm)

## セットアップ

### 1. OCIの設定

OCIコンソールから以下の情報を取得します：

- テナンシーOCID
- ユーザーOCID
- リージョン
- コンパートメントOCID

APIキーを生成し、フィンガープリントを取得します：

```bash
mkdir -p ~/.oci
openssl genrsa -out ~/.oci/oci_api_key.pem 2048
chmod 600 ~/.oci/oci_api_key.pem
openssl rsa -pubout -in ~/.oci/oci_api_key.pem -out ~/.oci/oci_api_key_public.pem
```

生成された公開鍵をOCIコンソールのユーザー設定にアップロードし、フィンガープリントを記録します。

### 2. terraform.tfvarsファイルの作成

```bash
cd oracle-cloud
cat > terraform.tfvars << EOF
tenancy_ocid     = "your-tenancy-ocid"
user_ocid        = "your-user-ocid"
fingerprint      = "your-fingerprint"
private_key_path = "~/.oci/oci_api_key.pem"
region           = "your-region"
compartment_ocid = "your-compartment-ocid"
EOF
```

## デプロイ

### 1. Terraformの初期化

```bash
terraform init
```

### 2. デプロイプランの確認

```bash
terraform plan
```

### 3. リソースのデプロイ

```bash
terraform apply
```

確認を求められたら「yes」と入力します。

## アクセス方法

デプロイが完了すると、以下のURLが出力されます：

- Webインターフェース: http://<instance_public_ip>:54867
- API: http://<instance_public_ip>:54868

## リソースの管理

### インスタンスの停止/起動

OCIコンソールまたはCLIを使用してインスタンスを管理できます：

```bash
# インスタンスの停止
oci compute instance action --action STOP --instance-id <instance_id>

# インスタンスの起動
oci compute instance action --action START --instance-id <instance_id>
```

### リソースの削除

```bash
terraform destroy
```

## トラブルシューティング

### SSHでインスタンスに接続

```bash
ssh ubuntu@<instance_public_ip>
```

### ログの確認

```bash
ssh ubuntu@<instance_public_ip> "docker-compose -f /opt/ShardX/docker-compose.yml logs"
```

### インスタンスの再起動

```bash
ssh ubuntu@<instance_public_ip> "cd /opt/ShardX && docker-compose restart"
```