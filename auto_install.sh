#!/bin/bash

# ShardX 完全自動インストールスクリプト
# このスクリプトは、ユーザー入力を必要とせずにShardXをインストールします

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
echo -e "${GREEN}ShardX 完全自動インストーラー${NC}"
echo "========================================"

# OSの検出
echo -e "${BLUE}システム情報を確認中...${NC}"
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="Linux"
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        DISTRO=$NAME
    fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macOS"
    DISTRO="macOS"
elif [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    OS="Windows"
    DISTRO="Windows"
else
    OS="Unknown"
    DISTRO="Unknown"
fi

# アーキテクチャの検出
ARCH=$(uname -m)
if [[ "$ARCH" == "x86_64" ]]; then
    ARCH="amd64"
elif [[ "$ARCH" == "aarch64" ]] || [[ "$ARCH" == "arm64" ]]; then
    ARCH="arm64"
fi

echo -e "${GREEN}検出されたシステム:${NC} $OS $DISTRO ($ARCH)"

# インストールモードの自動選択
echo -e "${BLUE}インストールモードを自動選択中...${NC}"
if command -v docker &> /dev/null; then
    echo -e "${GREEN}Dockerが検出されました。バックグラウンドモードを選択します。${NC}"
    INSTALL_MODE=2
elif command -v cargo &> /dev/null; then
    echo -e "${GREEN}Rustが検出されました。開発モードを選択します。${NC}"
    INSTALL_MODE=1
else
    echo -e "${YELLOW}Dockerが見つかりません。Dockerをインストールしてバックグラウンドモードを使用します。${NC}"
    INSTALL_MODE=2
    
    # Dockerのインストール
    if [[ "$OS" == "Linux" ]]; then
        echo -e "${BLUE}Dockerをインストール中...${NC}"
        curl -fsSL https://get.docker.com -o get-docker.sh
        sudo sh get-docker.sh
        sudo usermod -aG docker $USER
        echo -e "${GREEN}✓ Dockerがインストールされました${NC}"
        
        # Docker Composeのインストール
        echo -e "${BLUE}Docker Composeをインストール中...${NC}"
        sudo curl -L "https://github.com/docker/compose/releases/download/v2.18.1/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
        sudo chmod +x /usr/local/bin/docker-compose
        echo -e "${GREEN}✓ Docker Composeがインストールされました${NC}"
    elif [[ "$OS" == "macOS" ]]; then
        echo -e "${YELLOW}macOSの場合は、Docker Desktopを手動でインストールしてください: https://www.docker.com/products/docker-desktop${NC}"
        echo -e "${YELLOW}インストール後に再度このスクリプトを実行してください。${NC}"
        exit 1
    else
        echo -e "${RED}サポートされていないOSです。${NC}"
        exit 1
    fi
fi

# リポジトリのクローン
echo -e "${BLUE}ShardXリポジトリをクローン中...${NC}"
if [ -d "ShardX" ]; then
    echo -e "${YELLOW}ShardXディレクトリが既に存在します。更新します...${NC}"
    cd ShardX
    git pull
else
    git clone https://github.com/enablerdao/ShardX.git
    cd ShardX
fi

# インストールモードに応じた処理
case $INSTALL_MODE in
    1)
        echo -e "${BLUE}開発モードでインストール中...${NC}"
        if command -v cargo &> /dev/null; then
            echo -e "${GREEN}✓ Rust/Cargo がインストールされています${NC}"
            cargo build
            echo -e "${GREEN}✓ ShardXがビルドされました${NC}"
            echo -e "${BLUE}ShardXを起動中...${NC}"
            RUST_LOG=info cargo run &
            echo -e "${GREEN}✓ ShardXが起動されました${NC}"
        else
            echo -e "${BLUE}Rustをインストール中...${NC}"
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            source $HOME/.cargo/env
            cargo build
            echo -e "${GREEN}✓ ShardXがビルドされました${NC}"
            echo -e "${BLUE}ShardXを起動中...${NC}"
            RUST_LOG=info cargo run &
            echo -e "${GREEN}✓ ShardXが起動されました${NC}"
        fi
        ;;
    2)
        echo -e "${BLUE}バックグラウンドモードでインストール中...${NC}"
        if command -v docker-compose &> /dev/null; then
            docker-compose up -d
            echo -e "${GREEN}✓ ShardXがバックグラウンドで起動されました${NC}"
        elif command -v docker &> /dev/null && docker compose &> /dev/null; then
            docker compose up -d
            echo -e "${GREEN}✓ ShardXがバックグラウンドで起動されました${NC}"
        else
            echo -e "${RED}✗ Docker Composeが見つかりません${NC}"
            exit 1
        fi
        ;;
    *)
        echo -e "${RED}✗ 無効なインストールモードです${NC}"
        exit 1
        ;;
esac

echo -e "${GREEN}=======================================${NC}"
echo -e "${GREEN}ShardXのインストールが完了しました！${NC}"
echo -e "${GREEN}=======================================${NC}"
echo ""
echo -e "${BLUE}アクセス方法:${NC}"
echo "- Webインターフェース: http://localhost:54867"
echo "- API: http://localhost:54868"
echo ""
echo -e "${BLUE}詳細なドキュメント:${NC}"
echo "- https://github.com/enablerdao/ShardX/blob/main/README.md"
echo "- https://shardx.org/docs"
echo ""
echo -e "${YELLOW}問題が発生した場合は、GitHubのIssueを作成してください:${NC}"
echo "- https://github.com/enablerdao/ShardX/issues"
echo ""
echo -e "${GREEN}ShardXをご利用いただきありがとうございます！${NC}"