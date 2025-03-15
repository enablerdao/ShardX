#!/bin/bash
set -e

# 環境変数の設定
export RUST_LOG=${LOG_LEVEL:-info}
export RUST_BACKTRACE=${RUST_BACKTRACE:-1}

# データディレクトリの確認
if [ ! -d "${DATA_DIR}" ]; then
  echo "Creating data directory: ${DATA_DIR}"
  mkdir -p "${DATA_DIR}"
fi

# メモリ最適化設定
if [ ! -z "${MEMORY_LIMIT}" ]; then
  echo "Setting memory limit: ${MEMORY_LIMIT}"
  export MEMORY_LIMIT
fi

if [ ! -z "${GC_INTERVAL}" ]; then
  echo "Setting GC interval: ${GC_INTERVAL}"
  export GC_INTERVAL
fi

# ストレージ最適化設定
if [ ! -z "${COMPACTION_THRESHOLD}" ]; then
  echo "Setting compaction threshold: ${COMPACTION_THRESHOLD}"
  export COMPACTION_THRESHOLD
fi

if [ ! -z "${COMPACTION_INTERVAL}" ]; then
  echo "Setting compaction interval: ${COMPACTION_INTERVAL}"
  export COMPACTION_INTERVAL
fi

# ノードタイプに応じた設定
if [ "${NODE_TYPE}" = "main" ]; then
  echo "Starting as main node: ${NODE_ID}"
  ARGS="--main --node-id=${NODE_ID} --api-port=${API_PORT} --p2p-port=${P2P_PORT} --rpc-port=${RPC_PORT} --data-dir=${DATA_DIR}"
  
  if [ ! -z "${SHARD_COUNT}" ]; then
    ARGS="${ARGS} --shard-count=${SHARD_COUNT}"
  fi
elif [ "${NODE_TYPE}" = "shard" ]; then
  echo "Starting as shard node: ${NODE_ID} (${SHARD_ID})"
  ARGS="--shard --node-id=${NODE_ID} --shard-id=${SHARD_ID} --api-port=${API_PORT} --p2p-port=${P2P_PORT} --rpc-port=${RPC_PORT} --data-dir=${DATA_DIR}"
  
  if [ ! -z "${MAIN_NODE}" ]; then
    ARGS="${ARGS} --main-node=${MAIN_NODE}"
  fi
else
  echo "Invalid NODE_TYPE: ${NODE_TYPE}. Must be 'main' or 'shard'"
  exit 1
fi

# 追加の引数を追加
if [ $# -gt 0 ]; then
  exec "$@" ${ARGS}
else
  exec /app/shardx ${ARGS}
fi