#!/bin/bash
set -e

echo "=== ShardX 簡易インストールスクリプト ==="
echo "このスクリプトはDockerを使用してShardXを起動します"
echo

# Dockerがインストールされているか確認
if ! command -v docker &> /dev/null; then
    echo "Dockerがインストールされていません。インストールしてください:"
    echo "Linux: https://docs.docker.com/engine/install/"
    echo "macOS: https://docs.docker.com/desktop/mac/install/"
    echo "Windows: https://docs.docker.com/desktop/windows/install/"
    exit 1
fi

echo "ShardXイメージをダウンロードしています..."
docker pull enablerdao/shardx:latest

echo "ShardXを起動しています..."
docker run -d -p 54867:54867 -p 54868:54868 --name shardx enablerdao/shardx:latest

echo
echo "=== ShardXが正常に起動しました！ ==="
echo "ブラウザで以下のURLにアクセスできます:"
echo "- ウェブインターフェース: http://localhost:54867"
echo "- API: http://localhost:54868/api/v1/info"
echo
echo "コンテナを停止するには次のコマンドを実行してください:"
echo "docker stop shardx"
echo
echo "コンテナを再起動するには次のコマンドを実行してください:"
echo "docker start shardx"