#!/bin/bash
# Replit環境でのビルドスクリプト

# 色の定義
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}===============================================${NC}"
echo -e "${BLUE}ShardX - Replitビルドスクリプト${NC}"
echo -e "${BLUE}===============================================${NC}"

# 環境変数の設定
export CARGO_TERM_COLOR=always
export RUST_BACKTRACE=1
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"
export CARGO_NET_RETRY=10
export CARGO_INCREMENTAL=1

# 依存関係のインストール
echo -e "${YELLOW}依存関係を確認しています...${NC}"
if ! command -v lld &> /dev/null; then
  echo -e "${YELLOW}lldがインストールされていないため、代替リンカーを使用します${NC}"
  export RUSTFLAGS=""
fi

# ビルドオプションの設定
CARGO_OPTS="--release --no-default-features --features=snow"

# ビルドの実行
echo -e "${YELLOW}ShardXをビルドしています...${NC}"
cargo build $CARGO_OPTS || {
  echo -e "${RED}標準ビルドに失敗しました。代替方法を試します...${NC}"
  
  # 代替ビルド方法1: 最適化レベルを下げる
  echo -e "${YELLOW}最適化レベルを下げてビルドを試みます...${NC}"
  export RUSTFLAGS="-C opt-level=1"
  cargo build $CARGO_OPTS || {
    
    # 代替ビルド方法2: デバッグビルド
    echo -e "${YELLOW}デバッグビルドを試みます...${NC}"
    cargo build --no-default-features --features=snow || {
      
      # 代替ビルド方法3: 最小限の機能でビルド
      echo -e "${YELLOW}最小限の機能でビルドを試みます...${NC}"
      cargo build --no-default-features || {
        echo -e "${RED}すべてのビルド方法が失敗しました。${NC}"
        exit 1
      }
    }
  }
}

echo -e "${GREEN}ビルドが完了しました！${NC}"
exit 0