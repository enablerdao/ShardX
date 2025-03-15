#!/bin/bash
set -e

# ShardXインストールスクリプト
# このスクリプトは、ShardXをダウンロードしてインストールします

# 色の定義
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== ShardX インストールスクリプト ===${NC}"
echo

# OSを検出
OS="$(uname)"
case $OS in
  'Linux')
    OS='linux'
    ;;
  'Darwin')
    OS='macos'
    ;;
  *)
    echo -e "${RED}サポートされていないOS: $OS${NC}"
    echo "現在、LinuxとmacOSのみサポートしています。"
    exit 1
    ;;
esac

# アーキテクチャを検出
ARCH="$(uname -m)"
case $ARCH in
  'x86_64')
    ARCH='amd64'
    ;;
  'arm64' | 'aarch64')
    ARCH='arm64'
    ;;
  *)
    echo -e "${RED}サポートされていないアーキテクチャ: $ARCH${NC}"
    echo "現在、x86_64とarm64のみサポートしています。"
    exit 1
    ;;
esac

echo -e "${BLUE}OS: $OS, アーキテクチャ: $ARCH を検出しました${NC}"

# インストールディレクトリを設定
INSTALL_DIR="$HOME/.shardx"
BIN_DIR="$INSTALL_DIR/bin"
DATA_DIR="$INSTALL_DIR/data"
CONFIG_DIR="$INSTALL_DIR/config"
WEB_DIR="$INSTALL_DIR/web"

# 必要なツールの確認
command -v curl >/dev/null 2>&1 || { echo -e "${RED}curlが必要です${NC}"; exit 1; }
command -v tar >/dev/null 2>&1 || { echo -e "${RED}tarが必要です${NC}"; exit 1; }

# バージョンを設定
VERSION="v0.1.0"
if [ -n "$1" ]; then
  VERSION="$1"
fi

echo -e "${BLUE}ShardX $VERSION をインストールします...${NC}"

# インストールディレクトリを作成
mkdir -p "$BIN_DIR" "$DATA_DIR" "$CONFIG_DIR" "$WEB_DIR"

# GitHubリリースからバイナリをダウンロード
BINARY_URL="https://github.com/enablerdao/ShardX/releases/download/$VERSION/shardx-$OS-$ARCH"
echo -e "${BLUE}バイナリをダウンロード中: $BINARY_URL${NC}"

if ! curl -L -o "$BIN_DIR/shardx" "$BINARY_URL"; then
  echo -e "${YELLOW}リリースバイナリのダウンロードに失敗しました。ソースからビルドします...${NC}"
  
  # 必要なツールの確認
  command -v git >/dev/null 2>&1 || { echo -e "${RED}gitが必要です${NC}"; exit 1; }
  command -v cargo >/dev/null 2>&1 || { 
    echo -e "${YELLOW}Rustがインストールされていません。Rustをインストールします...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
  }
  
  # 一時ディレクトリを作成
  TMP_DIR=$(mktemp -d)
  cd "$TMP_DIR"
  
  # リポジトリをクローン
  git clone https://github.com/enablerdao/ShardX.git
  cd ShardX
  
  # 特定のバージョンをチェックアウト（タグがある場合）
  if [ "$VERSION" != "latest" ]; then
    git checkout "$VERSION" || echo -e "${YELLOW}指定されたバージョンが見つかりません。mainブランチを使用します。${NC}"
  fi
  
  # ビルド
  echo -e "${BLUE}ShardXをビルドしています...${NC}"
  cargo build --release
  
  # バイナリをコピー
  cp target/release/shardx "$BIN_DIR/shardx"
  
  # Webインターフェースをビルド
  if [ -d "web" ]; then
    echo -e "${BLUE}Webインターフェースをビルドしています...${NC}"
    cd web
    
    # Node.jsの確認
    if command -v npm >/dev/null 2>&1; then
      npm install
      npm run build
      cp -r dist/* "$WEB_DIR/"
    else
      echo -e "${YELLOW}Node.jsがインストールされていないため、Webインターフェースはビルドされません${NC}"
    fi
  fi
  
  # 一時ディレクトリを削除
  cd
  rm -rf "$TMP_DIR"
else
  chmod +x "$BIN_DIR/shardx"
  
  # Webインターフェースをダウンロード
  WEB_URL="https://github.com/enablerdao/ShardX/releases/download/$VERSION/web-dist.tar.gz"
  echo -e "${BLUE}Webインターフェースをダウンロード中: $WEB_URL${NC}"
  
  if curl -L -o /tmp/web.tar.gz "$WEB_URL"; then
    tar -xzf /tmp/web.tar.gz -C "$WEB_DIR"
    rm /tmp/web.tar.gz
  else
    echo -e "${YELLOW}Webインターフェースのダウンロードに失敗しました${NC}"
  fi
fi

# 実行ファイルへのシンボリックリンクを作成
SYMLINK_DIR="$HOME/.local/bin"
mkdir -p "$SYMLINK_DIR"

if [ -f "$SYMLINK_DIR/shardx" ]; then
  rm "$SYMLINK_DIR/shardx"
fi

ln -s "$BIN_DIR/shardx" "$SYMLINK_DIR/shardx"

# PATHを確認
if [[ ":$PATH:" != *":$SYMLINK_DIR:"* ]]; then
  echo -e "${YELLOW}$SYMLINK_DIR がPATHに含まれていません。${NC}"
  echo -e "以下をシェルの設定ファイル（~/.bashrc、~/.zshrc など）に追加してください:"
  echo -e "${GREEN}export PATH=\"\$PATH:$SYMLINK_DIR\"${NC}"
fi

# 設定ファイルを作成
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
  cat > "$CONFIG_DIR/config.toml" << EOL
# ShardX設定ファイル

# ノード設定
[node]
id = "node1"
initial_shards = 32
data_dir = "$DATA_DIR"

# APIサーバー設定
[api]
port = 54868
cors_enabled = true

# Webインターフェース設定
[web]
enabled = true
dir = "$WEB_DIR"
port = 54867

# ログ設定
[log]
level = "info"
EOL
fi

# 完了メッセージ
echo
echo -e "${GREEN}ShardX $VERSION のインストールが完了しました！${NC}"
echo
echo -e "バイナリの場所: ${BLUE}$BIN_DIR/shardx${NC}"
echo -e "設定ファイル: ${BLUE}$CONFIG_DIR/config.toml${NC}"
echo -e "データディレクトリ: ${BLUE}$DATA_DIR${NC}"
echo
echo -e "ShardXを起動するには: ${GREEN}shardx${NC}"
echo -e "設定ファイルを指定して起動するには: ${GREEN}shardx --config $CONFIG_DIR/config.toml${NC}"
echo
echo -e "詳細なドキュメントは https://github.com/enablerdao/ShardX/docs を参照してください"