#!/bin/bash
set -e

# ShardXをRenderにデプロイするスクリプト（修正版）

# 色の定義
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== ShardX Renderデプロイスクリプト（修正版） ===${NC}"
echo

# 必要なツールの確認
if ! command -v curl &> /dev/null; then
    echo -e "${RED}curlがインストールされていません。${NC}"
    exit 1
fi

# Renderのデプロイボタンを使用するためのURL
REPO_URL="https://github.com/enablerdao/ShardX"
RENDER_DEPLOY_URL="https://render.com/deploy?repo=$REPO_URL"

echo -e "${BLUE}Renderへのデプロイを開始します...${NC}"
echo -e "以下のURLをブラウザで開いてください:"
echo -e "${GREEN}$RENDER_DEPLOY_URL${NC}"
echo
echo -e "${BLUE}ブラウザでRenderにログインし、デプロイボタンをクリックしてください。${NC}"
echo -e "デプロイが完了すると、以下のサービスが自動的に作成されます:"
echo "- shardx-node: ShardXのメインノード"
echo "- shardx-web: Webインターフェース"
echo "- redis: キャッシュとメッセージングに使用"
echo "- shardx-worker: バックグラウンド処理用ワーカー"
echo
echo -e "${BLUE}注意: もしheadersエラーが発生した場合は、以下の手順で手動デプロイしてください:${NC}"
echo "1. Renderダッシュボード(https://dashboard.render.com)にログイン"
echo "2. 「New +」→「Web Service」をクリック"
echo "3. GitHubリポジトリ「enablerdao/ShardX」を選択"
echo "4. 以下の設定を行います:"
echo "   - Name: shardx-node"
echo "   - Environment: Docker"
echo "   - Branch: main"
echo "   - Dockerfile Path: Dockerfile.simple"
echo "   - Region: お好みのリージョン"
echo "   - Plan: Free"
echo "5. 「Advanced」をクリックして環境変数を設定:"
echo "   - PORT: 54868"
echo "   - NODE_ID: render_node"
echo "   - RUST_LOG: info"
echo "   - INITIAL_SHARDS: 10"
echo "   - DATA_DIR: /tmp/shardx-data"
echo "6. 「Create Web Service」をクリック"
echo
echo -e "${BLUE}ブラウザでURLを開きますか？ (y/n):${NC}"
read -r OPEN_BROWSER

if [[ "$OPEN_BROWSER" == "y" || "$OPEN_BROWSER" == "Y" ]]; then
    if command -v xdg-open &> /dev/null; then
        xdg-open "$RENDER_DEPLOY_URL"
    elif command -v open &> /dev/null; then
        open "$RENDER_DEPLOY_URL"
    elif command -v start &> /dev/null; then
        start "$RENDER_DEPLOY_URL"
    else
        echo -e "${RED}ブラウザを自動で開けませんでした。上記のURLを手動でブラウザにコピーしてください。${NC}"
    fi
fi

echo
echo -e "${BLUE}デプロイが完了したら、Renderダッシュボードでサービスの状態を確認してください。${NC}"
echo -e "Renderダッシュボード: ${GREEN}https://dashboard.render.com/${NC}"