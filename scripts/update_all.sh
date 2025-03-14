#!/bin/bash

# ShardX 一括アップデートスクリプト
# このスクリプトは、ShardXの全コンポーネントを最新バージョンに更新します

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
echo -e "${GREEN}ShardX アップデートツール${NC}"
echo "========================================"

# 現在のディレクトリを確認
CURRENT_DIR=$(pwd)
SHARDX_DIR=$(dirname $(dirname $(realpath $0)))

echo -e "${BLUE}ShardXディレクトリ: ${SHARDX_DIR}${NC}"
cd $SHARDX_DIR

# リポジトリの更新
echo -e "${BLUE}リポジトリを更新中...${NC}"
git fetch
LOCAL=$(git rev-parse HEAD)
REMOTE=$(git rev-parse @{u})

if [ "$LOCAL" != "$REMOTE" ]; then
    echo -e "${YELLOW}新しいバージョンが利用可能です。更新しています...${NC}"
    git pull
    echo -e "${GREEN}✓ リポジトリが更新されました${NC}"
else
    echo -e "${GREEN}✓ リポジトリは最新です${NC}"
fi

# 実行環境の検出
if [ -f "docker-compose.yml" ]; then
    echo -e "${BLUE}Docker環境を検出しました。コンテナを更新中...${NC}"
    
    # Dockerコンテナの更新
    if command -v docker-compose &> /dev/null; then
        docker-compose pull
        docker-compose build --no-cache
        docker-compose up -d
        echo -e "${GREEN}✓ Dockerコンテナが更新されました${NC}"
    elif command -v docker &> /dev/null && docker compose &> /dev/null; then
        docker compose pull
        docker compose build --no-cache
        docker compose up -d
        echo -e "${GREEN}✓ Dockerコンテナが更新されました${NC}"
    else
        echo -e "${RED}✗ Docker Composeが見つかりません${NC}"
        exit 1
    fi
elif [ -f "Cargo.toml" ]; then
    echo -e "${BLUE}Rust環境を検出しました。依存関係を更新中...${NC}"
    
    # Rust依存関係の更新
    if command -v cargo &> /dev/null; then
        cargo update
        cargo build --release
        echo -e "${GREEN}✓ Rust依存関係が更新されました${NC}"
        
        # サービスの再起動
        if [ -f "/etc/systemd/system/shardx.service" ]; then
            echo -e "${BLUE}システムサービスを再起動中...${NC}"
            sudo systemctl restart shardx
            echo -e "${GREEN}✓ サービスが再起動されました${NC}"
        else
            echo -e "${YELLOW}システムサービスが見つかりません。手動で再起動してください。${NC}"
        fi
    else
        echo -e "${RED}✗ Cargoが見つかりません${NC}"
        exit 1
    fi
else
    echo -e "${RED}✗ サポートされている実行環境が見つかりません${NC}"
    exit 1
fi

# フロントエンドの更新
if [ -d "web" ]; then
    echo -e "${BLUE}Webフロントエンドを更新中...${NC}"
    cd web
    
    if [ -f "package.json" ]; then
        if command -v npm &> /dev/null; then
            npm install
            npm run build
            echo -e "${GREEN}✓ Webフロントエンドが更新されました${NC}"
        else
            echo -e "${YELLOW}npmが見つかりません。Webフロントエンドは更新されませんでした。${NC}"
        fi
    fi
    
    cd $SHARDX_DIR
fi

# データベースのマイグレーション
if [ -d "migrations" ]; then
    echo -e "${BLUE}データベースマイグレーションを実行中...${NC}"
    
    if command -v diesel &> /dev/null; then
        diesel migration run
        echo -e "${GREEN}✓ データベースマイグレーションが完了しました${NC}"
    else
        echo -e "${YELLOW}dieselが見つかりません。データベースマイグレーションはスキップされました。${NC}"
    fi
fi

echo -e "${GREEN}=======================================${NC}"
echo -e "${GREEN}ShardXの更新が完了しました！${NC}"
echo -e "${GREEN}=======================================${NC}"
echo ""
echo -e "${BLUE}現在のバージョン:${NC}"
git describe --tags --abbrev=0 2>/dev/null || echo "開発バージョン"
echo ""
echo -e "${YELLOW}問題が発生した場合は、GitHubのIssueを作成してください:${NC}"
echo "- https://github.com/enablerdao/ShardX/issues"