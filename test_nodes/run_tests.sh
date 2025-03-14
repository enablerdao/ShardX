#!/bin/bash

# テスト実行スクリプト
cd "$(dirname "$0")"

echo "=== ShardXテスト実行 ==="

# 必要なツールの確認
if ! command -v jq &> /dev/null; then
    echo "jqがインストールされていません。インストールしています..."
    sudo apt-get update && sudo apt-get install -y jq
fi

# ノードのビルド
echo "ShardXをビルドしています..."
cd ..
cargo build
cd test_nodes

# ノードの起動
echo "ノードを起動しています..."
./start_nodes.sh

# ノードの起動を待機
echo "ノードの起動を待機しています (10秒)..."
sleep 10

# トランザクションテストの実行
echo "トランザクションテストを実行しています..."
./test_transactions.sh | tee transaction_test_results.log

# テスト結果の保存
echo "テスト結果を保存しています..."
mkdir -p ../test_results
cp transaction_test_results.log ../test_results/

# ノードのログを保存
echo "ノードのログを保存しています..."
cp node*/node*.log ../test_results/ 2>/dev/null || echo "ノードログはありません"

# ノードの停止
echo "ノードを停止しています..."
./stop_nodes.sh

echo "=== テスト完了 ==="
echo "テスト結果は ../test_results/ ディレクトリに保存されました。"