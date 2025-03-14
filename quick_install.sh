#!/bin/bash

# ShardX クイックインストールスクリプト
# このスクリプトは、ShardXを素早くインストールして起動します

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
echo -e "${GREEN}ShardX クイックインストーラー${NC}"
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

# Rustがインストールされているか確認
if command -v cargo &> /dev/null; then
    echo -e "${GREEN}✓ Rust/Cargo がインストールされています${NC}"
else
    echo -e "${YELLOW}! Rust/Cargo がインストールされていません${NC}"
    echo -e "${BLUE}Rustをインストール中...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# ShardXをビルド
echo -e "${BLUE}ShardXをビルド中...${NC}"
cargo build

# データディレクトリの作成
echo -e "${BLUE}データディレクトリを作成中...${NC}"
mkdir -p data

# ShardXを起動
echo -e "${GREEN}✓ ShardXがビルドされました${NC}"
echo -e "${BLUE}ShardXを起動中...${NC}"
echo -e "${YELLOW}Ctrl+Cで停止できます${NC}"
cargo run

echo -e "${GREEN}=======================================${NC}"
echo -e "${GREEN}ShardXのインストールが完了しました！${NC}"
echo -e "${GREEN}=======================================${NC}"