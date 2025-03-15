#!/bin/bash
set -e

echo "=== ShardX 起動スクリプト ==="
echo "ShardXを起動しています..."
echo

# ビルドされたバイナリが存在するか確認
if [ ! -f "./target/release/shardx" ]; then
    echo "ShardXがビルドされていません。インストールスクリプトを実行してください:"
    echo "  Linux: ./scripts/linux_install.sh"
    echo "  macOS: ./scripts/mac_install.sh"
    exit 1
fi

# 設定ファイルが存在するか確認
if [ ! -f "./config/default.toml" ]; then
    echo "設定ファイルが見つかりません。デフォルト設定を作成します..."
    mkdir -p ./config
    cat > ./config/default.toml << EOF
[server]
host = "0.0.0.0"
port = 54868
web_port = 54867

[node]
id = "local_node"
initial_shards = 10

[storage]
data_dir = "./data"
EOF
fi

# データディレクトリを作成
mkdir -p ./data

# ShardXを起動
echo "ShardXを起動しています..."
echo "ログはターミナルに表示されます。Ctrl+Cで終了できます。"
echo
echo "ブラウザで以下のURLにアクセスできます:"
echo "- ウェブインターフェース: http://localhost:54867"
echo "- API: http://localhost:54868/api/v1/info"
echo

# バイナリを実行
./target/release/shardx