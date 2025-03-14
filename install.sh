#!/bin/bash

# ShardX インストールスクリプト
# このスクリプトは、ShardXノードとWebインターフェースをインストールして起動します。

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
echo -e "${GREEN}ShardX インストーラー${NC}"
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

# 依存関係のチェック
echo -e "${BLUE}依存関係をチェック中...${NC}"

# Dockerのチェック
if command -v docker &> /dev/null; then
    echo -e "${GREEN}✓ Docker がインストールされています${NC}"
    DOCKER_VERSION=$(docker --version)
    echo "  $DOCKER_VERSION"
else
    echo -e "${YELLOW}! Docker がインストールされていません${NC}"
    echo -e "  Docker のインストール方法: https://docs.docker.com/get-docker/"
    
    # Dockerのインストールを提案
    if [[ "$OS" == "Linux" ]]; then
        echo -e "${BLUE}Dockerをインストールしますか？ (y/n)${NC}"
        read -r install_docker
        if [[ "$install_docker" == "y" ]]; then
            echo -e "${BLUE}Dockerをインストール中...${NC}"
            curl -fsSL https://get.docker.com -o get-docker.sh
            sudo sh get-docker.sh
            sudo usermod -aG docker $USER
            echo -e "${GREEN}✓ Docker がインストールされました${NC}"
            echo "  新しいグループメンバーシップを有効にするには、ログアウトして再度ログインしてください。"
        fi
    fi
fi

# Docker Composeのチェック
if command -v docker-compose &> /dev/null; then
    echo -e "${GREEN}✓ Docker Compose がインストールされています${NC}"
    COMPOSE_VERSION=$(docker-compose --version)
    echo "  $COMPOSE_VERSION"
elif command -v docker &> /dev/null && docker compose version &> /dev/null; then
    echo -e "${GREEN}✓ Docker Compose プラグインがインストールされています${NC}"
    COMPOSE_VERSION=$(docker compose version)
    echo "  $COMPOSE_VERSION"
else
    echo -e "${YELLOW}! Docker Compose がインストールされていません${NC}"
    echo -e "  Docker Compose のインストール方法: https://docs.docker.com/compose/install/"
    
    # Docker Composeのインストールを提案
    if [[ "$OS" == "Linux" ]]; then
        echo -e "${BLUE}Docker Composeをインストールしますか？ (y/n)${NC}"
        read -r install_compose
        if [[ "$install_compose" == "y" ]]; then
            echo -e "${BLUE}Docker Composeをインストール中...${NC}"
            sudo curl -L "https://github.com/docker/compose/releases/download/v2.18.1/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
            sudo chmod +x /usr/local/bin/docker-compose
            echo -e "${GREEN}✓ Docker Compose がインストールされました${NC}"
        fi
    fi
fi

# Gitのチェック
if command -v git &> /dev/null; then
    echo -e "${GREEN}✓ Git がインストールされています${NC}"
    GIT_VERSION=$(git --version)
    echo "  $GIT_VERSION"
else
    echo -e "${YELLOW}! Git がインストールされていません${NC}"
    echo -e "  Git のインストール方法: https://git-scm.com/downloads"
    
    # Gitのインストールを提案
    if [[ "$OS" == "Linux" ]]; then
        echo -e "${BLUE}Gitをインストールしますか？ (y/n)${NC}"
        read -r install_git
        if [[ "$install_git" == "y" ]]; then
            echo -e "${BLUE}Gitをインストール中...${NC}"
            if [[ "$DISTRO" == *"Ubuntu"* ]] || [[ "$DISTRO" == *"Debian"* ]]; then
                sudo apt-get update
                sudo apt-get install -y git
            elif [[ "$DISTRO" == *"Fedora"* ]] || [[ "$DISTRO" == *"CentOS"* ]] || [[ "$DISTRO" == *"RHEL"* ]]; then
                sudo dnf install -y git
            fi
            echo -e "${GREEN}✓ Git がインストールされました${NC}"
        fi
    fi
fi

# インストールモードの選択
echo -e "${BLUE}インストールモードを選択してください:${NC}"
echo "1) 開発モード (ソースコードからビルド)"
echo "2) バックグラウンドモード (Dockerで実行)"
echo "3) 本番モード (システムサービスとして実行)"
read -r install_mode

# リポジトリのクローン
echo -e "${BLUE}ShardXリポジトリをクローン中...${NC}"
if [ -d "ShardX" ]; then
    echo -e "${YELLOW}ShardXディレクトリが既に存在します。更新しますか？ (y/n)${NC}"
    read -r update_repo
    if [[ "$update_repo" == "y" ]]; then
        cd ShardX
        git pull
        cd ..
    fi
else
    git clone https://github.com/enablerdao/ShardX.git
fi

cd ShardX

# インストールモードに応じた処理
case $install_mode in
    1)
        echo -e "${BLUE}開発モードでインストール中...${NC}"
        if command -v cargo &> /dev/null; then
            echo -e "${GREEN}✓ Rust/Cargo がインストールされています${NC}"
            cargo build
            echo -e "${GREEN}✓ ShardXがビルドされました${NC}"
            echo -e "${BLUE}ShardXを起動しますか？ (y/n)${NC}"
            read -r start_shardx
            if [[ "$start_shardx" == "y" ]]; then
                cargo run
            fi
        else
            echo -e "${YELLOW}! Rust/Cargo がインストールされていません${NC}"
            echo -e "  Rust のインストール方法: https://www.rust-lang.org/tools/install"
            echo -e "${BLUE}Rustをインストールしますか？ (y/n)${NC}"
            read -r install_rust
            if [[ "$install_rust" == "y" ]]; then
                curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
                source $HOME/.cargo/env
                cargo build
                echo -e "${GREEN}✓ ShardXがビルドされました${NC}"
                echo -e "${BLUE}ShardXを起動しますか？ (y/n)${NC}"
                read -r start_shardx
                if [[ "$start_shardx" == "y" ]]; then
                    cargo run
                fi
            fi
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
    3)
        echo -e "${BLUE}本番モードでインストール中...${NC}"
        if [[ "$OS" == "Linux" ]]; then
            echo -e "${BLUE}システムサービスを作成中...${NC}"
            sudo cp ./scripts/shardx.service /etc/systemd/system/
            sudo systemctl daemon-reload
            sudo systemctl enable shardx
            sudo systemctl start shardx
            echo -e "${GREEN}✓ ShardXがシステムサービスとして起動されました${NC}"
        else
            echo -e "${RED}✗ システムサービスは現在Linuxのみサポートしています${NC}"
            exit 1
        fi
        ;;
    *)
        echo -e "${RED}✗ 無効な選択です${NC}"
        exit 1
        ;;
esac

# インストール完了メッセージ
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