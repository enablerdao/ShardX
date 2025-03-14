#!/bin/bash

# ノードを起動するスクリプト
cd "$(dirname "$0")"

# ノード1を起動
echo "ノード1を起動中..."
cd node1
../target/debug/shardx --node-id node1 --port 54868 --data-dir ./data > node1.log 2>&1 &
NODE1_PID=$!
cd ..

# ノード2を起動
echo "ノード2を起動中..."
cd node2
../target/debug/shardx --node-id node2 --port 54869 --data-dir ./data > node2.log 2>&1 &
NODE2_PID=$!
cd ..

# ノード3を起動
echo "ノード3を起動中..."
cd node3
../target/debug/shardx --node-id node3 --port 54870 --data-dir ./data > node3.log 2>&1 &
NODE3_PID=$!
cd ..

# ノード4を起動
echo "ノード4を起動中..."
cd node4
../target/debug/shardx --node-id node4 --port 54871 --data-dir ./data > node4.log 2>&1 &
NODE4_PID=$!
cd ..

# ノード5を起動
echo "ノード5を起動中..."
cd node5
../target/debug/shardx --node-id node5 --port 54872 --data-dir ./data > node5.log 2>&1 &
NODE5_PID=$!
cd ..

echo "すべてのノードが起動しました。"
echo "ノード1: http://localhost:54868 (PID: $NODE1_PID)"
echo "ノード2: http://localhost:54869 (PID: $NODE2_PID)"
echo "ノード3: http://localhost:54870 (PID: $NODE3_PID)"
echo "ノード4: http://localhost:54871 (PID: $NODE4_PID)"
echo "ノード5: http://localhost:54872 (PID: $NODE5_PID)"

# PIDを保存
echo "$NODE1_PID $NODE2_PID $NODE3_PID $NODE4_PID $NODE5_PID" > node_pids.txt

echo "ノードを停止するには stop_nodes.sh を実行してください。"