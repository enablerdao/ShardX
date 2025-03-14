#!/bin/bash

# ShardX 開発者向けインストールスクリプト
# このスクリプトは、ShardXの開発環境をセットアップします

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
echo -e "${GREEN}ShardX 開発者向けインストーラー${NC}"
echo "========================================"

# 設定
INSTALL_RUST=true
INSTALL_NODE=true
INSTALL_VSCODE=false
INSTALL_DEV_TOOLS=true
SETUP_GIT=true

# 引数の解析
while [[ $# -gt 0 ]]; do
  case $1 in
    --install-rust=*)
      INSTALL_RUST="${1#*=}"
      shift
      ;;
    --install-node=*)
      INSTALL_NODE="${1#*=}"
      shift
      ;;
    --install-vscode=*)
      INSTALL_VSCODE="${1#*=}"
      shift
      ;;
    --install-dev-tools=*)
      INSTALL_DEV_TOOLS="${1#*=}"
      shift
      ;;
    --setup-git=*)
      SETUP_GIT="${1#*=}"
      shift
      ;;
    *)
      echo -e "${RED}Unknown parameter: $1${NC}"
      exit 1
      ;;
  esac
done

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

echo -e "${GREEN}検出されたシステム:${NC} $OS $DISTRO"

# 基本的な開発ツールのインストール
if [ "$INSTALL_DEV_TOOLS" = "true" ]; then
    echo -e "${BLUE}基本的な開発ツールをインストール中...${NC}"
    if [[ "$OS" == "Linux" ]]; then
        if [[ "$DISTRO" == *"Ubuntu"* ]] || [[ "$DISTRO" == *"Debian"* ]]; then
            sudo apt-get update
            sudo apt-get install -y build-essential git curl wget pkg-config libssl-dev
        elif [[ "$DISTRO" == *"Fedora"* ]] || [[ "$DISTRO" == *"CentOS"* ]] || [[ "$DISTRO" == *"RHEL"* ]]; then
            sudo dnf groupinstall -y "Development Tools"
            sudo dnf install -y git curl wget openssl-devel
        elif [[ "$DISTRO" == *"Arch"* ]]; then
            sudo pacman -Sy --noconfirm base-devel git curl wget openssl
        fi
    elif [[ "$OS" == "macOS" ]]; then
        # Homebrewのインストール（存在しない場合）
        if ! command -v brew &> /dev/null; then
            echo -e "${BLUE}Homebrewをインストール中...${NC}"
            /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        fi
        brew install git curl wget openssl
    fi
fi

# Rustのインストール
if [ "$INSTALL_RUST" = "true" ]; then
    echo -e "${BLUE}Rustをインストール中...${NC}"
    if ! command -v rustc &> /dev/null; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source $HOME/.cargo/env
        
        # 開発に便利なツールをインストール
        rustup component add rustfmt
        rustup component add clippy
        cargo install cargo-watch
        cargo install cargo-edit
        
        echo -e "${GREEN}✓ Rustがインストールされました${NC}"
    else
        echo -e "${GREEN}✓ Rustはすでにインストールされています${NC}"
        # Rustを最新バージョンに更新
        rustup update
    fi
fi

# Node.jsのインストール
if [ "$INSTALL_NODE" = "true" ]; then
    echo -e "${BLUE}Node.jsをインストール中...${NC}"
    if ! command -v node &> /dev/null; then
        if [[ "$OS" == "Linux" ]]; then
            curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
            sudo apt-get install -y nodejs
        elif [[ "$OS" == "macOS" ]]; then
            brew install node@18
        fi
        echo -e "${GREEN}✓ Node.jsがインストールされました${NC}"
    else
        echo -e "${GREEN}✓ Node.jsはすでにインストールされています${NC}"
    fi
fi

# Visual Studio Codeのインストール
if [ "$INSTALL_VSCODE" = "true" ]; then
    echo -e "${BLUE}Visual Studio Codeをインストール中...${NC}"
    if ! command -v code &> /dev/null; then
        if [[ "$OS" == "Linux" ]]; then
            if [[ "$DISTRO" == *"Ubuntu"* ]] || [[ "$DISTRO" == *"Debian"* ]]; then
                wget -qO- https://packages.microsoft.com/keys/microsoft.asc | gpg --dearmor > packages.microsoft.gpg
                sudo install -o root -g root -m 644 packages.microsoft.gpg /etc/apt/trusted.gpg.d/
                sudo sh -c 'echo "deb [arch=amd64,arm64,armhf signed-by=/etc/apt/trusted.gpg.d/packages.microsoft.gpg] https://packages.microsoft.com/repos/code stable main" > /etc/apt/sources.list.d/vscode.list'
                rm -f packages.microsoft.gpg
                sudo apt-get update
                sudo apt-get install -y code
            elif [[ "$DISTRO" == *"Fedora"* ]] || [[ "$DISTRO" == *"CentOS"* ]] || [[ "$DISTRO" == *"RHEL"* ]]; then
                sudo rpm --import https://packages.microsoft.com/keys/microsoft.asc
                sudo sh -c 'echo -e "[code]\nname=Visual Studio Code\nbaseurl=https://packages.microsoft.com/yumrepos/vscode\nenabled=1\ngpgcheck=1\ngpgkey=https://packages.microsoft.com/keys/microsoft.asc" > /etc/yum.repos.d/vscode.repo'
                sudo dnf install -y code
            elif [[ "$DISTRO" == *"Arch"* ]]; then
                sudo pacman -Sy --noconfirm code
            fi
        elif [[ "$OS" == "macOS" ]]; then
            brew install --cask visual-studio-code
        fi
        echo -e "${GREEN}✓ Visual Studio Codeがインストールされました${NC}"
        
        # 推奨拡張機能のインストール
        code --install-extension rust-lang.rust-analyzer
        code --install-extension serayuzgur.crates
        code --install-extension vadimcn.vscode-lldb
        code --install-extension ms-azuretools.vscode-docker
    else
        echo -e "${GREEN}✓ Visual Studio Codeはすでにインストールされています${NC}"
    fi
fi

# Gitの設定
if [ "$SETUP_GIT" = "true" ]; then
    echo -e "${BLUE}Gitを設定中...${NC}"
    read -p "Gitユーザー名を入力してください [デフォルト: ShardX Developer]: " GIT_USER
    GIT_USER=${GIT_USER:-"ShardX Developer"}
    
    read -p "Gitメールアドレスを入力してください [デフォルト: dev@shardx.org]: " GIT_EMAIL
    GIT_EMAIL=${GIT_EMAIL:-"dev@shardx.org"}
    
    git config --global user.name "$GIT_USER"
    git config --global user.email "$GIT_EMAIL"
    git config --global init.defaultBranch main
    
    echo -e "${GREEN}✓ Gitが設定されました${NC}"
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

# フロントエンドの依存関係をインストール
if [ "$INSTALL_NODE" = "true" ]; then
    echo -e "${BLUE}フロントエンドの依存関係をインストール中...${NC}"
    if [ -d "web" ]; then
        cd web
        npm install
        cd ..
    fi
fi

# 開発用の設定ファイルを作成
echo -e "${BLUE}開発用の設定ファイルを作成中...${NC}"
mkdir -p .vscode
cat > .vscode/settings.json << EOF
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "editor.formatOnSave": true,
    "rust-analyzer.cargo.allFeatures": true,
    "rust-analyzer.procMacro.enable": true,
    "rust-analyzer.cargo.loadOutDirsFromCheck": true
}
EOF

cat > .vscode/launch.json << EOF
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug ShardX",
            "cargo": {
                "args": [
                    "build",
                    "--bin=shardx",
                    "--package=shardx"
                ],
                "filter": {
                    "name": "shardx",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}",
            "env": {
                "RUST_LOG": "debug"
            }
        }
    ]
}
EOF

# 開発用のスクリプトを作成
echo -e "${BLUE}開発用のスクリプトを作成中...${NC}"
cat > dev.sh << EOF
#!/bin/bash

# ShardX 開発用スクリプト

case "\$1" in
    build)
        cargo build
        ;;
    run)
        RUST_LOG=debug cargo run
        ;;
    test)
        cargo test
        ;;
    watch)
        cargo watch -x 'run'
        ;;
    web)
        cd web && npm run dev
        ;;
    clean)
        cargo clean
        ;;
    update)
        cargo update
        ;;
    *)
        echo "使用方法: ./dev.sh [コマンド]"
        echo "コマンド:"
        echo "  build   - プロジェクトをビルド"
        echo "  run     - プロジェクトを実行"
        echo "  test    - テストを実行"
        echo "  watch   - ファイル変更を監視して自動的に再ビルド・再実行"
        echo "  web     - フロントエンド開発サーバーを起動"
        echo "  clean   - ビルドキャッシュをクリア"
        echo "  update  - 依存関係を更新"
        ;;
esac
EOF

chmod +x dev.sh

echo -e "${GREEN}=======================================${NC}"
echo -e "${GREEN}ShardXの開発環境セットアップが完了しました！${NC}"
echo -e "${GREEN}=======================================${NC}"
echo -e "${BLUE}開発を始めるには:${NC}"
echo "1. ./dev.sh build - プロジェクトをビルド"
echo "2. ./dev.sh run - プロジェクトを実行"
echo "3. ./dev.sh web - フロントエンド開発サーバーを起動"
echo ""
echo -e "${YELLOW}Visual Studio Codeでプロジェクトを開く:${NC}"
echo "code ."