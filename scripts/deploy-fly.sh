#!/bin/bash
set -e

# ShardXをFly.ioにデプロイするスクリプト

# 色の定義
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== ShardX Fly.ioデプロイスクリプト ===${NC}"
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

# fly.tomlを更新
echo -e "${BLUE}設定ファイルを更新しています...${NC}"
sed -i "s/app = \"shardx\"/app = \"$APP_NAME\"/" fly.toml
sed -i "s/primary_region = \"nrt\"/primary_region = \"$REGION\"/" fly.toml

# アプリを作成
echo -e "${BLUE}Fly.ioアプリを作成しています...${NC}"
flyctl apps create "$APP_NAME" --generate-name

# ボリュームを作成
echo -e "${BLUE}永続ボリュームを作成しています...${NC}"
flyctl volumes create shardx_data --region "$REGION" --size 1 --app "$APP_NAME"

# Redisを追加
echo -e "${BLUE}Redisを追加しています...${NC}"
flyctl redis create --name "${APP_NAME}-redis" --region "$REGION" --vm-size shared-cpu-1x --initial-cluster-size 1

# Redisの接続情報を取得
REDIS_URL=$(flyctl redis status "${APP_NAME}-redis" --json | jq -r '.url')
flyctl secrets set REDIS_URL="$REDIS_URL" --app "$APP_NAME"

# デプロイ
echo -e "${BLUE}Fly.ioにデプロイしています...${NC}"
flyctl deploy --app "$APP_NAME"

# 完了
echo -e "${GREEN}デプロイが完了しました！${NC}"
echo -e "アプリURL: ${BLUE}https://$APP_NAME.fly.dev${NC}"
echo
echo -e "${BLUE}ログを確認するには:${NC} flyctl logs --app $APP_NAME"