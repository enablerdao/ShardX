#!/bin/bash
set -e

# ShardXをFly.ioにデプロイするスクリプト（Webインターフェース用）

# 色の定義
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== ShardX Fly.io Webデプロイスクリプト ===${NC}"
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
read -p "Fly.ioアプリ名を入力してください (例: shardx-web): " APP_NAME

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

# 一時ディレクトリを作成
TEMP_DIR=$(mktemp -d)
echo -e "${BLUE}一時ディレクトリを作成しました: ${TEMP_DIR}${NC}"

# Webディレクトリをコピー
cp -r web/* $TEMP_DIR/
cd $TEMP_DIR

# Dockerfileを作成
cat > Dockerfile << EOL
FROM node:18-alpine

WORKDIR /app

COPY package*.json ./
RUN npm install

COPY . .

EXPOSE 52153

ENV PORT=52153
ENV NODE_ENV=production

CMD ["npm", "start"]
EOL

# fly.tomlを作成
cat > fly.toml << EOL
# fly.toml - Web Interface
app = "${APP_NAME}"
primary_region = "${REGION}"

[build]
  dockerfile = "Dockerfile"

[env]
  PORT = "52153"
  NODE_ENV = "production"
  API_URL = "https://shardx.fly.dev"

[http]
  internal_port = 52153
  force_https = true

[[services]]
  protocol = "tcp"
  internal_port = 52153
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

[[vm]]
  memory = "512mb"
  cpu_kind = "shared"
  cpus = 1
EOL

# アプリを作成
echo -e "${BLUE}Fly.ioアプリを作成しています...${NC}"
flyctl apps create "$APP_NAME" --json || echo "アプリは既に存在します"

# デプロイ
echo -e "${BLUE}Fly.ioにデプロイしています...${NC}"
flyctl deploy

# 完了
echo -e "${GREEN}デプロイが完了しました！${NC}"
echo -e "アプリURL: ${BLUE}https://$APP_NAME.fly.dev${NC}"
echo
echo -e "${BLUE}ログを確認するには:${NC} flyctl logs --app $APP_NAME"

# 一時ディレクトリを削除
cd -
rm -rf $TEMP_DIR