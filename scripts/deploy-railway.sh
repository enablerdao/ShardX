#!/bin/bash
set -e

# ShardXをRailwayにデプロイするスクリプト

# 色の定義
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== ShardX Railwayデプロイスクリプト ===${NC}"
echo

# Railway CLIがインストールされているか確認
if ! command -v railway &> /dev/null; then
    echo -e "${RED}Railway CLIがインストールされていません。${NC}"
    echo "インストール方法: npm i -g @railway/cli"
    echo "または: curl -fsSL https://railway.app/install.sh | sh"
    
    # インストールを試みる
    read -p "Railway CLIをインストールしますか？ (y/n): " INSTALL_CLI
    if [[ "$INSTALL_CLI" == "y" || "$INSTALL_CLI" == "Y" ]]; then
        echo -e "${BLUE}Railway CLIをインストールしています...${NC}"
        curl -fsSL https://railway.app/install.sh | sh
        
        # PATHを更新
        export PATH=$HOME/.railway/bin:$PATH
    else
        echo -e "${RED}Railway CLIが必要です。インストール後に再実行してください。${NC}"
        exit 1
    fi
fi

# ログイン状態を確認
if ! railway whoami &> /dev/null; then
    echo -e "${BLUE}Railwayにログインしてください...${NC}"
    railway login
fi

# プロジェクト名を取得
read -p "Railwayプロジェクト名を入力してください (例: shardx-app): " PROJECT_NAME

# 新しいプロジェクトを作成するか確認
read -p "新しいプロジェクトを作成しますか？ (y/n): " CREATE_PROJECT
if [[ "$CREATE_PROJECT" == "y" || "$CREATE_PROJECT" == "Y" ]]; then
    echo -e "${BLUE}新しいプロジェクト '$PROJECT_NAME' を作成します...${NC}"
    railway project create "$PROJECT_NAME"
fi

# プロジェクトを選択
echo -e "${BLUE}プロジェクトを選択してください...${NC}"
railway link

# 環境変数を設定
echo -e "${BLUE}環境変数を設定しています...${NC}"
railway variables set NODE_ID="railway_node_$(date +%s)"
railway variables set RUST_LOG=info
railway variables set INITIAL_SHARDS=64
railway variables set DATA_DIR=/app/data
railway variables set REDIS_ENABLED=true
railway variables set WEB_ENABLED=true

# デプロイ
echo -e "${BLUE}Railwayにデプロイしています...${NC}"
railway up

# 完了
echo -e "${GREEN}デプロイが完了しました！${NC}"
echo -e "プロジェクトURL: ${BLUE}https://railway.app/project/$(railway project)${NC}"
echo
echo -e "${BLUE}ログを確認するには:${NC} railway logs"