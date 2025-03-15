#!/bin/bash
set -e

echo "=== ShardX macOS インストールスクリプト ==="
echo "このスクリプトはShardXをmacOS環境にインストールします"
echo

# 必要な依存関係をチェック
echo "依存関係をチェックしています..."
if ! command -v cargo &> /dev/null; then
    echo "Rustがインストールされていません。インストールします..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

if ! command -v git &> /dev/null; then
    echo "Gitがインストールされていません。インストールしてください:"
    echo "brew install git"
    exit 1
fi

if ! command -v brew &> /dev/null; then
    echo "Homebrewがインストールされていません。インストールしてください:"
    echo '/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"'
    exit 1
fi

# macOS特有の依存関係をチェック
if ! brew list openssl &> /dev/null; then
    echo "OpenSSLをインストールしています..."
    brew install openssl
fi

# プロジェクトをビルド
echo "ShardXをビルドしています..."
export OPENSSL_DIR=$(brew --prefix openssl)
cargo build --release

echo
echo "=== インストールが完了しました！ ==="
echo "ShardXを起動するには次のコマンドを実行してください:"
echo "./scripts/run.sh"
echo
echo "ブラウザで以下のURLにアクセスできます:"
echo "- ウェブインターフェース: http://localhost:54867"
echo "- API: http://localhost:54868/api/v1/info"