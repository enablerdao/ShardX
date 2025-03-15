#!/bin/bash
set -e

# ShardXをFly.ioにデプロイするスクリプト（修正版）

# 色の定義
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== ShardX Fly.ioデプロイスクリプト（修正版） ===${NC}"
echo

# Fly CLIがインストールされているか確認
if ! command -v flyctl &> /dev/null; then
    echo -e "${RED}Fly CLIがインストールされていません。${NC}"
    echo "インストール方法: curl -L https://fly.io/install.sh | sh"
    
    # インストールを試みる
    read -p "Fly CLIをインストールしますか？ (y/n): " INSTALL_CLI
    if [[ "$INSTALL_CLI" == "y" || "$INSTALL_CLI" == "Y" ]]; then
        echo -e "${BLUE}Fly CLIをインストールしています...${NC}"
        curl -L https://fly.io/install.sh | sh
        
        # PATHを更新
        export PATH=$HOME/.fly/bin:$PATH
    else
        echo -e "${RED}Fly CLIが必要です。インストール後に再実行してください。${NC}"
        exit 1
    fi
fi

# ログイン状態を確認
if ! flyctl auth whoami &> /dev/null; then
    echo -e "${BLUE}Fly.ioにログインしてください...${NC}"
    flyctl auth login
fi

# アプリ名を取得
read -p "Fly.ioアプリ名を入力してください (例: shardx-app): " APP_NAME

# リージョンを選択
echo -e "${BLUE}デプロイするリージョンを選択してください:${NC}"
echo "1) Tokyo (nrt)"
echo "2) Singapore (sin)"
echo "3) Los Angeles (lax)"
echo "4) New York (ewr)"
echo "5) London (lhr)"
read -p "リージョン番号を選択してください (1-5): " REGION_NUM

case $REGION_NUM in
    1) REGION="nrt" ;;
    2) REGION="sin" ;;
    3) REGION="lax" ;;
    4) REGION="ewr" ;;
    5) REGION="lhr" ;;
    *) REGION="nrt" ;;
esac

# 一時的な設定ファイルを作成
echo -e "${BLUE}設定ファイルを作成しています...${NC}"
cat > fly.toml.new << EOL
# fly.toml - Fly.io設定ファイル
app = "${APP_NAME}"
primary_region = "${REGION}"

[build]
  dockerfile = "Dockerfile"
  builder = "dockerfile"

[env]
  NODE_ID = "fly_node"
  RUST_LOG = "info"
  INITIAL_SHARDS = "64"
  DATA_DIR = "/app/data"
  REDIS_ENABLED = "true"
  WEB_ENABLED = "true"
  PORT = "54868"
  P2P_PORT = "54867"

[http_service]
  internal_port = 54868
  force_https = true
  auto_stop_machines = false
  auto_start_machines = true
  min_machines_running = 1
  processes = ["app"]

[[services]]
  protocol = "tcp"
  internal_port = 54868
  processes = ["app"]

  [[services.ports]]
    port = 80
    handlers = ["http"]
    force_https = true

  [[services.ports]]
    port = 443
    handlers = ["tls", "http"]

  [services.concurrency]
    type = "connections"
    hard_limit = 1000
    soft_limit = 500

[[services]]
  protocol = "tcp"
  internal_port = 54867
  processes = ["app"]

  [[services.ports]]
    port = 54867
    handlers = ["tcp"]

[mounts]
  source = "shardx_data"
  destination = "/app/data"

[metrics]
  port = 9091
  path = "/metrics"

[[vm]]
  memory = "1gb"
  cpu_kind = "shared"
  cpus = 1
EOL

# 設定ファイルを置き換え
mv fly.toml.new fly.toml

# アプリを作成
echo -e "${BLUE}Fly.ioアプリを作成しています...${NC}"
flyctl apps create "$APP_NAME" --json || echo "アプリは既に存在します"

# ボリュームを作成
echo -e "${BLUE}永続ボリュームを作成しています...${NC}"
flyctl volumes create shardx_data --region "$REGION" --size 1 --app "$APP_NAME" || echo "ボリュームは既に存在します"

# デプロイ
echo -e "${BLUE}Fly.ioにデプロイしています...${NC}"
flyctl deploy --app "$APP_NAME" --remote-only

# 完了
echo -e "${GREEN}デプロイが完了しました！${NC}"
echo -e "アプリURL: ${BLUE}https://$APP_NAME.fly.dev${NC}"
echo
echo -e "${BLUE}ログを確認するには:${NC} flyctl logs --app $APP_NAME"