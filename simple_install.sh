#!/bin/bash

# ShardX シンプルインストールスクリプト
# このスクリプトは、ShardXを素早くインストールして起動します
# 対話的な入力を必要としません

set -e

# カラー設定
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ロゴ表示
echo -e "${BLUE}"
echo "  ____  _                    _ __   __"
echo " / ___|| |__   __ _ _ __ __| |\ \ / /"
echo " \___ \| '_ \ / _\` | '__/ _\` | \ V / "
echo "  ___) | | | | (_| | | | (_| |  | |  "
echo " |____/|_| |_|\__,_|_|  \__,_|  |_|  "
echo -e "${NC}"
echo -e "${GREEN}ShardX シンプルインストーラー${NC}"
echo "========================================"

# 現在のディレクトリを確認
CURRENT_DIR=$(pwd)
echo -e "${BLUE}現在のディレクトリ: ${CURRENT_DIR}${NC}"

# リポジトリのクローン
echo -e "${BLUE}ShardXリポジトリをクローン中...${NC}"
if [ -d "ShardX" ]; then
    echo -e "${YELLOW}ShardXディレクトリが既に存在します。${NC}"
    cd ShardX
    echo -e "${BLUE}リポジトリを更新中...${NC}"
    git pull
else
    git clone https://github.com/enablerdao/ShardX.git
    cd ShardX
fi

# インストール方法の選択
echo -e "${BLUE}インストール方法を選択: Docker${NC}"

# Dockerがインストールされているか確認
if command -v docker &> /dev/null; then
    echo -e "${GREEN}✓ Docker がインストールされています${NC}"
else
    echo -e "${RED}✗ Docker がインストールされていません${NC}"
    echo -e "${YELLOW}Dockerをインストールしてから再度実行してください。${NC}"
    exit 1
fi

# Docker Composeがインストールされているか確認
if command -v docker-compose &> /dev/null || (command -v docker &> /dev/null && docker compose version &> /dev/null); then
    echo -e "${GREEN}✓ Docker Compose がインストールされています${NC}"
else
    echo -e "${RED}✗ Docker Compose がインストールされていません${NC}"
    echo -e "${YELLOW}Docker Composeをインストールしてから再度実行してください。${NC}"
    exit 1
fi

# Docker Composeでビルドと起動
echo -e "${BLUE}Docker Composeでビルドと起動中...${NC}"
docker-compose up -d

echo -e "${GREEN}=======================================${NC}"
echo -e "${GREEN}ShardXのインストールが完了しました！${NC}"
echo -e "${GREEN}=======================================${NC}"
echo -e "${BLUE}アクセス方法:${NC}"
echo "- Webインターフェース: http://localhost:54867"
echo "- API: http://localhost:54868"
echo ""
echo -e "${YELLOW}ShardXを停止するには:${NC}"
echo "docker-compose down"