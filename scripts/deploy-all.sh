#!/bin/bash
set -e

# ShardXを各クラウドプラットフォームにデプロイするための統合スクリプト

# 色の定義
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== ShardX 統合デプロイスクリプト ===${NC}"
echo

# デプロイ先を選択
echo -e "${BLUE}デプロイ先を選択してください:${NC}"
echo "1) Render (最も簡単、無料プランあり)"
echo "2) Railway (GitHubと連携、無料枠あり)"
echo "3) Heroku (安定性と拡張性、無料枠なし)"
echo "4) Fly.io (グローバル分散デプロイ、無料枠あり)"
read -p "選択してください (1-4): " PLATFORM_NUM

case $PLATFORM_NUM in
    1)
        echo -e "${BLUE}Renderへのデプロイを開始します...${NC}"
        if [ -f "./scripts/deploy-render-fix.sh" ]; then
            ./scripts/deploy-render-fix.sh
        else
            echo -e "${RED}デプロイスクリプトが見つかりません。${NC}"
            echo -e "${YELLOW}以下のURLをブラウザで開いてデプロイしてください:${NC}"
            echo -e "${GREEN}https://render.com/deploy?repo=https://github.com/enablerdao/ShardX${NC}"
        fi
        ;;
    2)
        echo -e "${BLUE}Railwayへのデプロイを開始します...${NC}"
        if [ -f "./scripts/deploy-railway.sh" ]; then
            ./scripts/deploy-railway.sh
        else
            echo -e "${RED}デプロイスクリプトが見つかりません。${NC}"
            echo -e "${YELLOW}以下のURLをブラウザで開いてデプロイしてください:${NC}"
            echo -e "${GREEN}https://railway.app/template/ShardX${NC}"
        fi
        ;;
    3)
        echo -e "${BLUE}Herokuへのデプロイを開始します...${NC}"
        if [ -f "./scripts/deploy-heroku-fix.sh" ]; then
            ./scripts/deploy-heroku-fix.sh
        else
            echo -e "${RED}デプロイスクリプトが見つかりません。${NC}"
            echo -e "${YELLOW}以下のURLをブラウザで開いてデプロイしてください:${NC}"
            echo -e "${GREEN}https://heroku.com/deploy?template=https://github.com/enablerdao/ShardX${NC}"
        fi
        ;;
    4)
        echo -e "${BLUE}Fly.ioへのデプロイを開始します...${NC}"
        if [ -f "./scripts/deploy-fly-fix.sh" ]; then
            ./scripts/deploy-fly-fix.sh
            
            echo -e "${BLUE}Webインターフェースもデプロイしますか？ (y/n):${NC}"
            read -r DEPLOY_WEB
            if [[ "$DEPLOY_WEB" == "y" || "$DEPLOY_WEB" == "Y" ]]; then
                if [ -f "./scripts/deploy-fly-web.sh" ]; then
                    ./scripts/deploy-fly-web.sh
                else
                    echo -e "${RED}Webデプロイスクリプトが見つかりません。${NC}"
                fi
            fi
        else
            echo -e "${RED}デプロイスクリプトが見つかりません。${NC}"
            echo -e "${YELLOW}以下のURLをブラウザで開いてデプロイしてください:${NC}"
            echo -e "${GREEN}https://fly.io/launch/github/enablerdao/ShardX${NC}"
        fi
        ;;
    *)
        echo -e "${RED}無効な選択です。${NC}"
        exit 1
        ;;
esac

echo
echo -e "${GREEN}デプロイプロセスが完了しました！${NC}"
echo -e "${BLUE}問題が発生した場合は、各プラットフォームのダッシュボードを確認してください。${NC}"
echo -e "${BLUE}詳細なデプロイ手順は docs/deployment/multi-platform-deployment.md を参照してください。${NC}"