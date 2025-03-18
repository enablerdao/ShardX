#!/bin/bash
# Replit用のShardX実行スクリプト

# 色の定義
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}===============================================${NC}"
echo -e "${BLUE}ShardX - 高性能ブロックチェーンプラットフォーム${NC}"
echo -e "${BLUE}===============================================${NC}"

# ランダムなノードIDを生成
RANDOM_SUFFIX=$(cat /dev/urandom | tr -dc 'a-z0-9' | fold -w 8 | head -n 1 2>/dev/null || echo "$(date +%s)")
HOST_NAME="replit"
export NODE_ID="${HOST_NAME}-${RANDOM_SUFFIX}"
echo -e "${GREEN}ノードID: ${NODE_ID}${NC}"

# データディレクトリの作成
DATA_DIR=${DATA_DIR:-"$HOME/shardx-data"}
mkdir -p $DATA_DIR
echo -e "${GREEN}データディレクトリ: ${DATA_DIR}${NC}"

# 環境変数の設定
export RUST_LOG=${RUST_LOG:-"info"}
export PORT=${PORT:-54868}
export P2P_PORT=${P2P_PORT:-54867}
export DATA_DIR
export INITIAL_SHARDS=${INITIAL_SHARDS:-32}

echo -e "${BLUE}-----------------------------------------------${NC}"
echo -e "${GREEN}APIポート: ${PORT}${NC}"
echo -e "${GREEN}P2Pポート: ${P2P_PORT}${NC}"
echo -e "${GREEN}初期シャード数: ${INITIAL_SHARDS}${NC}"
echo -e "${BLUE}-----------------------------------------------${NC}"

# ビルドスクリプトを実行
echo -e "${YELLOW}ShardXをビルドしています...${NC}"
./scripts/replit-build.sh || {
  echo -e "${RED}ビルドに失敗しました。プレースホルダーを使用します。${NC}"
  
  # プレースホルダースクリプトを作成
  cat > /tmp/shardx-placeholder.sh << 'EOF'
#!/bin/bash
# ShardX プレースホルダー

# 環境変数の取得
NODE_ID=${NODE_ID:-"unknown-node"}
PORT=${PORT:-54868}
P2P_PORT=${P2P_PORT:-54867}
DATA_DIR=${DATA_DIR:-"/tmp/shardx-data"}

echo "==============================================="
echo "ShardX プレースホルダー"
echo "==============================================="
echo "ノードID: ${NODE_ID}"
echo "データディレクトリ: ${DATA_DIR}"
echo "APIポート: ${PORT}"
echo "P2Pポート: ${P2P_PORT}"
echo "-----------------------------------------------"
echo "これはShardXのプレースホルダーです。"
echo "実際の環境では、本物のShardXバイナリが実行されます。"
echo "-----------------------------------------------"

# シンプルなHTTPサーバーの実装
http_server() {
  while true; do
    echo "APIリクエスト待機中... (ポート: ${PORT})"
    
    # nc (netcat) がインストールされているか確認
    if command -v nc >/dev/null 2>&1; then
      # HTTPリクエストを待機
      REQUEST=$(nc -l -p ${PORT} -q 1)
      
      # レスポンスを返す
      if echo "$REQUEST" | grep -q "GET"; then
        echo -e "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"ok\",\"version\":\"1.0.0\",\"node_id\":\"${NODE_ID}\"}" | nc -l -p ${PORT} -q 1
      fi
    else
      # nc がない場合は、シンプルなメッセージを表示
      echo "netcat not available, simulating HTTP server"
      sleep 10
    fi
  done
}

# P2Pサービスのシミュレーション
p2p_service() {
  while true; do
    echo "P2Pサービス実行中... (ポート: ${P2P_PORT})"
    sleep 30
  done
}

# メインプロセス
echo "ShardXサービスを開始しています..."

# HTTPサーバーをバックグラウンドで起動
http_server &
HTTP_PID=$!

# P2Pサービスをバックグラウンドで起動
p2p_service &
P2P_PID=$!

# シグナルハンドラの設定
trap "kill $HTTP_PID $P2P_PID; exit 0" INT TERM

# ステータス表示
while true; do
  echo "[$(date +%H:%M:%S)] ShardX実行中... (Ctrl+Cで停止)"
  sleep 10
done
EOF

  # プレースホルダーに実行権限を付与
  chmod +x /tmp/shardx-placeholder.sh
  
  # プレースホルダーを実行
  echo -e "${YELLOW}プレースホルダーを実行します...${NC}"
  /tmp/shardx-placeholder.sh
  exit 0
}

# ビルドが成功した場合、ShardXを実行
echo -e "${GREEN}ビルド成功！ShardXを実行します...${NC}"
./target/release/shardx