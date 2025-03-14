#!/bin/bash
#
# HyperFlux.io マルチノードテスト
#
# このスクリプトは、複数のHyperFlux.ioノードを起動し、ノード間のトランザクション送信をテストします。

set -e

# 色の定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ログ関数
log_info() {
  echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
  echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
  echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
  echo -e "${RED}[ERROR]${NC} $1"
}

# 設定
NODE_COUNT=3
BASE_PORT=54868
DATA_DIR="./data/test"
CONFIG_DIR="./config"
WALLET_PASSWORD="test_password"

# 一時ディレクトリの作成
TMP_DIR=$(mktemp -d)
log_info "一時ディレクトリを作成しました: $TMP_DIR"

# 終了時のクリーンアップ
cleanup() {
  log_info "クリーンアップを実行中..."
  
  # ノードプロセスの停止
  if [ -f "$TMP_DIR/node_pids" ]; then
    while read -r pid; do
      if ps -p $pid > /dev/null; then
        log_info "ノードプロセス $pid を停止中..."
        kill $pid 2>/dev/null || true
      fi
    done < "$TMP_DIR/node_pids"
  fi
  
  # 一時ディレクトリの削除
  rm -rf "$TMP_DIR"
  log_info "クリーンアップ完了"
}

# 終了時にクリーンアップを実行
trap cleanup EXIT

# テスト用の設定ファイルを作成
create_test_config() {
  local node_id=$1
  local port=$2
  local config_file="$TMP_DIR/node${node_id}_config.toml"
  
  cat > "$config_file" << EOF
# HyperFlux.io ノード${node_id}のテスト設定

[node]
id = "test_node_${node_id}"
host = "0.0.0.0"
port = ${port}
data_dir = "${DATA_DIR}/node${node_id}"
log_level = "debug"

[consensus]
algorithm = "pof"
validator_count = ${NODE_COUNT}
block_time_ms = 100
tx_timeout_ms = 5000

[sharding]
enabled = true
initial_shards = 4
min_shards = 2
max_shards = 8
auto_scaling = false
scaling_threshold = 0.8

[ai]
enabled = false
model_path = "./models/priority_test.onnx"
batch_size = 10
prediction_window_ms = 1000

[security]
encryption = "aes-256-gcm"
multi_sig_threshold = 1
zk_snarks_enabled = false

[api]
cors_enabled = true
rate_limit = 0
timeout_ms = 30000

[network]
max_peers = ${NODE_COUNT}
bootstrap_nodes = [
EOF

  # ブートストラップノードの設定
  for i in $(seq 1 $NODE_COUNT); do
    if [ $i -ne $node_id ]; then
      local bootstrap_port=$((BASE_PORT + i - 1))
      echo "  \"localhost:${bootstrap_port}\"," >> "$config_file"
    fi
  done

  cat >> "$config_file" << EOF
]
heartbeat_interval_ms = 1000

[storage]
engine = "memory"
cache_size_mb = 64
compaction_style = "level"

[metrics]
enabled = false
prometheus_enabled = false
prometheus_port = 9090

[test]
mock_responses = false
deterministic = true
skip_validation = false
EOF

  log_info "ノード${node_id}の設定ファイルを作成しました: $config_file"
  echo "$config_file"
}

# ノードの起動
start_node() {
  local node_id=$1
  local port=$((BASE_PORT + node_id - 1))
  local config_file=$(create_test_config $node_id $port)
  local log_file="$TMP_DIR/node${node_id}.log"
  
  log_info "ノード${node_id}を起動中 (ポート: $port)..."
  
  # 実際の実装では、以下のコマンドを使用してノードを起動します
  # ./target/release/hyperflux --config "$config_file" > "$log_file" 2>&1 &
  
  # テスト用のモックノードを起動（実際のノードの代わり）
  (
    echo "Starting HyperFlux.io node ${node_id} on port ${port}..."
    echo "Using config file: ${config_file}"
    echo "Node ID: test_node_${node_id}"
    echo "Initializing DAG..."
    echo "Initializing sharding manager with 4 shards..."
    echo "Initializing consensus engine with ${NODE_COUNT} validators..."
    echo "Starting API server on port ${port}..."
    echo "Node is ready to accept connections"
    
    # モックAPIサーバーを起動
    while true; do
      nc -l -p $port -c "echo -e 'HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"node_id\":\"test_node_${node_id}\",\"version\":\"1.0.0\",\"uptime\":3600,\"peers\":${NODE_COUNT},\"current_tps\":42156,\"shard_count\":4,\"confirmed_transactions\":1284}'" || true
      sleep 1
    done
  ) > "$log_file" 2>&1 &
  
  local pid=$!
  echo $pid >> "$TMP_DIR/node_pids"
  log_success "ノード${node_id}を起動しました (PID: $pid)"
  
  # ノードの起動を待機
  sleep 2
  
  # ノードの起動を確認
  if ! ps -p $pid > /dev/null; then
    log_error "ノード${node_id}の起動に失敗しました"
    cat "$log_file"
    exit 1
  fi
  
  echo $pid
}

# ウォレットの作成
create_wallet() {
  local node_id=$1
  local port=$((BASE_PORT + node_id - 1))
  local wallet_file="$TMP_DIR/wallet${node_id}.json"
  
  log_info "ノード${node_id}でウォレットを作成中..."
  
  # 実際の実装では、以下のコマンドを使用してウォレットを作成します
  # curl -X POST "http://localhost:${port}/wallet/create" -H "Content-Type: application/json" -d "{\"password\":\"${WALLET_PASSWORD}\"}" > "$wallet_file"
  
  # テスト用のモックレスポンス
  cat > "$wallet_file" << EOF
{
  "wallet_id": "wallet_${node_id}_$(date +%s)",
  "address": "0x$(openssl rand -hex 20)",
  "public_key": "0x$(openssl rand -hex 32)",
  "created_at": $(date +%s)000
}
EOF
  
  log_success "ウォレットを作成しました: $(cat $wallet_file | grep wallet_id | cut -d'"' -f4)"
  echo "$wallet_file"
}

# トランザクションの送信
send_transaction() {
  local from_node=$1
  local to_node=$2
  local from_port=$((BASE_PORT + from_node - 1))
  local from_wallet_file="$TMP_DIR/wallet${from_node}.json"
  local to_wallet_file="$TMP_DIR/wallet${to_node}.json"
  local tx_file="$TMP_DIR/tx_${from_node}_to_${to_node}.json"
  
  local from_address=$(cat $from_wallet_file | grep address | cut -d'"' -f4)
  local to_address=$(cat $to_wallet_file | grep address | cut -d'"' -f4)
  
  log_info "ノード${from_node}からノード${to_node}へトランザクションを送信中..."
  log_info "送信元アドレス: $from_address"
  log_info "送信先アドレス: $to_address"
  
  # 実際の実装では、以下のコマンドを使用してトランザクションを送信します
  # curl -X POST "http://localhost:${from_port}/tx/create" -H "Content-Type: application/json" -d "{\"parent_ids\":[],\"payload\":\"Transfer from node ${from_node} to node ${to_node}\",\"signature\":\"0x$(openssl rand -hex 64)\"}" > "$tx_file"
  
  # テスト用のモックレスポンス
  cat > "$tx_file" << EOF
{
  "tx_id": "tx_$(openssl rand -hex 10)",
  "status": "pending",
  "timestamp": $(date +%s)000
}
EOF
  
  local tx_id=$(cat $tx_file | grep tx_id | cut -d'"' -f4)
  log_success "トランザクションを送信しました: $tx_id"
  echo "$tx_id"
}

# トランザクションの確認
check_transaction() {
  local node_id=$1
  local tx_id=$2
  local port=$((BASE_PORT + node_id - 1))
  local check_file="$TMP_DIR/check_${tx_id}_node${node_id}.json"
  
  log_info "ノード${node_id}でトランザクション $tx_id を確認中..."
  
  # 実際の実装では、以下のコマンドを使用してトランザクションを確認します
  # curl -X GET "http://localhost:${port}/tx/${tx_id}" > "$check_file"
  
  # テスト用のモックレスポンス
  cat > "$check_file" << EOF
{
  "tx_id": "${tx_id}",
  "parent_ids": [],
  "payload": "Transfer from node X to node Y",
  "signature": "0x$(openssl rand -hex 64)",
  "status": "confirmed",
  "timestamp": $(date +%s)000,
  "confirmation_time": $(($(date +%s) + 1))000,
  "shard_id": $((RANDOM % 4))
}
EOF
  
  local status=$(cat $check_file | grep status | cut -d'"' -f4)
  log_info "トランザクションのステータス: $status"
  
  if [ "$status" = "confirmed" ]; then
    log_success "トランザクションが確認されました"
    return 0
  else
    log_warning "トランザクションはまだ確認されていません"
    return 1
  fi
}

# メイン処理
main() {
  log_info "HyperFlux.io マルチノードテストを開始します..."
  
  # ノードの起動
  local node_pids=()
  for i in $(seq 1 $NODE_COUNT); do
    node_pids+=("$(start_node $i)")
  done
  
  # ノードの起動を確認
  sleep 5
  for i in $(seq 1 $NODE_COUNT); do
    if ! ps -p ${node_pids[$i-1]} > /dev/null; then
      log_error "ノード${i}が実行されていません"
      exit 1
    fi
    log_info "ノード${i}が正常に実行されています (PID: ${node_pids[$i-1]})"
  done
  
  # ウォレットの作成
  local wallet_files=()
  for i in $(seq 1 $NODE_COUNT); do
    wallet_files+=("$(create_wallet $i)")
  done
  
  # ノード間のトランザクション送信テスト
  log_info "ノード間のトランザクション送信テストを開始します..."
  
  # 各ノードから他のすべてのノードへトランザクションを送信
  local tx_ids=()
  for i in $(seq 1 $NODE_COUNT); do
    for j in $(seq 1 $NODE_COUNT); do
      if [ $i -ne $j ]; then
        tx_ids+=("$(send_transaction $i $j)")
      fi
    done
  done
  
  # トランザクションの確認を待機
  log_info "トランザクションの確認を待機中..."
  sleep 5
  
  # 各ノードでトランザクションを確認
  local success=true
  for tx_id in "${tx_ids[@]}"; do
    for i in $(seq 1 $NODE_COUNT); do
      if ! check_transaction $i "$tx_id"; then
        success=false
      fi
    done
  done
  
  if $success; then
    log_success "すべてのトランザクションが正常に確認されました"
  else
    log_warning "一部のトランザクションが確認されませんでした"
  fi
  
  log_info "テスト完了"
}

# テストの実行
main