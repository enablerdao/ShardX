#!/bin/sh
# ShardX プレースホルダースクリプト
# このスクリプトは、ShardXバイナリが利用できない場合に使用される機能するプレースホルダーです

# ランダムなノードIDを生成
RANDOM_SUFFIX=$(cat /dev/urandom | tr -dc 'a-z0-9' | fold -w 8 | head -n 1 2>/dev/null || echo "$(date +%s)")
HOST_NAME=$(hostname | tr '[:upper:]' '[:lower:]' 2>/dev/null || echo "node")
NODE_ID="${NODE_ID:-${HOST_NAME}-${RANDOM_SUFFIX}}"

# 環境変数の設定
PORT=${PORT:-54868}
P2P_PORT=${P2P_PORT:-54867}
DATA_DIR=${DATA_DIR:-/tmp/shardx-data}
VERSION="1.0.0"

# データディレクトリの作成
mkdir -p $DATA_DIR

# 起動情報の表示
echo "==============================================="
echo "ShardX v${VERSION} Placeholder"
echo "==============================================="
echo "Node ID: ${NODE_ID}"
echo "Data directory: ${DATA_DIR}"
echo "API port: ${PORT}"
echo "P2P port: ${P2P_PORT}"
echo "-----------------------------------------------"
echo "This is a functional placeholder for ShardX."
echo "In production, this would be replaced with the actual ShardX binary."
echo "-----------------------------------------------"

# アクセス可能なURLを表示
echo ""
echo "=== ShardX サービスが起動しました ==="
echo "API エンドポイント: http://localhost:${PORT}/"
echo "P2P サービス: http://localhost:${P2P_PORT}/"
echo "====================================="
echo ""

# シンプルなHTTPサーバーの実装
http_server() {
  while true; do
    # nc (netcat) がインストールされているか確認
    if command -v nc >/dev/null 2>&1; then
      echo "Starting HTTP server on port ${PORT}..."
      while true; do
        echo -e "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"ok\",\"version\":\"${VERSION}\",\"node_id\":\"${NODE_ID}\"}" | nc -l -p ${PORT} -q 1
      done
    else
      # nc がない場合は、シンプルなメッセージを表示
      echo "netcat not available, HTTP server not started"
      sleep 60
    fi
  done
}

# P2Pサーバーの実装（シミュレーション）
p2p_server() {
  while true; do
    echo "[$(date +%H:%M:%S)] ShardX P2P service running on port ${P2P_PORT}..."
    sleep 30
  done
}

# メインプロセス
main() {
  echo "Starting ShardX services..."
  
  # HTTPサーバーをバックグラウンドで起動
  http_server &
  HTTP_PID=$!
  
  # P2Pサーバーをバックグラウンドで起動
  p2p_server &
  P2P_PID=$!
  
  # シグナルハンドラの設定
  trap "kill $HTTP_PID $P2P_PID; exit 0" INT TERM
  
  # ステータス表示
  while true; do
    echo "[$(date +%H:%M:%S)] ShardX is running... (press Ctrl+C to stop)"
    sleep 10
  done
}

# スクリプトの実行
main