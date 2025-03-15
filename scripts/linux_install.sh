#!/bin/bash
set -e

echo "=== ShardX Linux インストールスクリプト ==="
echo "このスクリプトはShardXをLinux環境にインストールします"
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
    echo "sudo apt install git"
    exit 1
fi

if ! command -v pkg-config &> /dev/null || ! command -v libssl-dev &> /dev/null; then
    echo "必要なライブラリがインストールされていません。インストールしてください:"
    echo "sudo apt install pkg-config libssl-dev"
    exit 1
fi

# プロジェクトをビルド
echo "ShardXをビルドしています..."
cargo build --release

echo
echo "=== インストールが完了しました！ ==="
echo "ShardXを起動するには次のコマンドを実行してください:"
echo "./scripts/run.sh"
echo
echo "ブラウザで以下のURLにアクセスできます:"
echo "- ウェブインターフェース: http://localhost:54867"
echo "- API: http://localhost:54868/api/v1/info"