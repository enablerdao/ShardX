#!/bin/bash

# ShardX ワンクリックインストールスクリプト
# このスクリプトは、ShardXを素早くインストールして起動します
# 対話的な入力を必要とせず、自動的にすべての依存関係をインストールします

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
echo -e "${GREEN}ShardX ワンクリックインストーラー${NC}"
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

# 依存関係のインストール
echo -e "${BLUE}必要な依存関係をインストール中...${NC}"

# Gitのインストール
if ! command -v git &> /dev/null; then
    echo -e "${YELLOW}Gitをインストール中...${NC}"
    if [[ "$OS" == "Linux" ]]; then
        if [[ "$DISTRO" == *"Ubuntu"* ]] || [[ "$DISTRO" == *"Debian"* ]]; then
            sudo apt-get update
            sudo apt-get install -y git
        elif [[ "$DISTRO" == *"Fedora"* ]] || [[ "$DISTRO" == *"CentOS"* ]] || [[ "$DISTRO" == *"RHEL"* ]]; then
            sudo dnf install -y git
        elif [[ "$DISTRO" == *"Arch"* ]]; then
            sudo pacman -Sy --noconfirm git
        fi
    elif [[ "$OS" == "macOS" ]]; then
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        brew install git
    fi
fi

# Dockerのインストール
if ! command -v docker &> /dev/null; then
    echo -e "${YELLOW}Dockerをインストール中...${NC}"
    if [[ "$OS" == "Linux" ]]; then
        curl -fsSL https://get.docker.com -o get-docker.sh
        sudo sh get-docker.sh
        sudo usermod -aG docker $USER
        echo -e "${YELLOW}注意: Dockerグループの変更を適用するには、ログアウトして再度ログインしてください。${NC}"
    elif [[ "$OS" == "macOS" ]]; then
        echo -e "${YELLOW}macOSの場合は、Docker Desktopを手動でインストールしてください: https://www.docker.com/products/docker-desktop${NC}"
    fi
fi

# Docker Composeのインストール
if ! command -v docker-compose &> /dev/null && ! (command -v docker &> /dev/null && docker compose version &> /dev/null); then
    echo -e "${YELLOW}Docker Composeをインストール中...${NC}"
    if [[ "$OS" == "Linux" ]]; then
        sudo curl -L "https://github.com/docker/compose/releases/download/v2.18.1/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
        sudo chmod +x /usr/local/bin/docker-compose
    elif [[ "$OS" == "macOS" ]]; then
        echo -e "${YELLOW}macOSの場合は、Docker Desktopに含まれるDocker Composeを使用してください。${NC}"
    fi
fi

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

# Docker Composeでビルドと起動
echo -e "${BLUE}Docker Composeでビルドと起動中...${NC}"
if command -v docker-compose &> /dev/null; then
    docker-compose up -d
elif command -v docker &> /dev/null && docker compose &> /dev/null; then
    docker compose up -d
else
    echo -e "${RED}✗ Docker Composeが見つかりません${NC}"
    exit 1
fi

echo -e "${GREEN}=======================================${NC}"
echo -e "${GREEN}ShardXのインストールが完了しました！${NC}"
echo -e "${GREEN}=======================================${NC}"
echo -e "${BLUE}アクセス方法:${NC}"
echo "- Webインターフェース: http://localhost:54867"
echo "- API: http://localhost:54868"
echo ""
echo -e "${YELLOW}ShardXを停止するには:${NC}"
echo "cd ShardX && docker-compose down"