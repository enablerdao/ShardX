#!/bin/bash

# ShardX クイックスタートスクリプト
# このスクリプトは、ShardXを素早く起動するためのものです

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
echo -e "${GREEN}ShardX クイックスタート${NC}"
echo "========================================"

# Dockerのチェック
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Dockerがインストールされていません。${NC}"
    echo -e "${YELLOW}Dockerをインストールしてください: https://docs.docker.com/get-docker/${NC}"
    exit 1
fi

# Docker Composeのチェック
if ! command -v docker-compose &> /dev/null && ! (command -v docker &> /dev/null && docker compose version &> /dev/null); then
    echo -e "${RED}Docker Composeがインストールされていません。${NC}"
    echo -e "${YELLOW}Docker Composeをインストールしてください: https://docs.docker.com/compose/install/${NC}"
    exit 1
fi

# データディレクトリの作成
mkdir -p data

# シンプルなDocker Composeを使用
echo -e "${BLUE}ShardXを起動中...${NC}"
if command -v docker-compose &> /dev/null; then
    docker-compose -f docker-compose.simple.yml up -d
elif command -v docker &> /dev/null && docker compose &> /dev/null; then
    docker compose -f docker-compose.simple.yml up -d
else
    echo -e "${RED}Docker Composeが見つかりません${NC}"
    exit 1
fi

# 起動確認
echo -e "${BLUE}ShardXの起動を確認中...${NC}"
sleep 5

# ヘルスチェック
if command -v docker-compose &> /dev/null; then
    HEALTH=$(docker-compose -f docker-compose.simple.yml ps | grep shardx | grep -o "Up")
elif command -v docker &> /dev/null && docker compose &> /dev/null; then
    HEALTH=$(docker compose -f docker-compose.simple.yml ps | grep shardx | grep -o "Up")
fi

if [ "$HEALTH" == "Up" ]; then
    echo -e "${GREEN}=======================================${NC}"
    echo -e "${GREEN}ShardXが正常に起動しました！${NC}"
    echo -e "${GREEN}=======================================${NC}"
    echo -e "${BLUE}アクセス方法:${NC}"
    echo "- Webインターフェース: http://localhost:54867"
    echo "- API: http://localhost:54868"
    echo ""
    echo -e "${YELLOW}ShardXを停止するには:${NC}"
    if command -v docker-compose &> /dev/null; then
        echo "docker-compose -f docker-compose.simple.yml down"
    elif command -v docker &> /dev/null && docker compose &> /dev/null; then
        echo "docker compose -f docker-compose.simple.yml down"
    fi
else
    echo -e "${RED}ShardXの起動に失敗しました。${NC}"
    echo -e "${YELLOW}ログを確認してください:${NC}"
    if command -v docker-compose &> /dev/null; then
        echo "docker-compose -f docker-compose.simple.yml logs"
    elif command -v docker &> /dev/null && docker compose &> /dev/null; then
        echo "docker compose -f docker-compose.simple.yml logs"
    fi
fi