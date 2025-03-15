#!/bin/bash

# ShardX複数ノード起動スクリプト
# 複数のShardXノードを異なる設定で起動するスクリプト

# Create data directories
mkdir -p data1 data2 data3

# Start node 1
echo "Starting node 1..."
cargo run --bin shardx -- --node-id node1 --port 54868 --data-dir ./data1 --log-level debug --shard-count 256 > node1.log 2>&1 &
NODE1_PID=$!
echo "Started node 1 with PID $NODE1_PID"

# Wait a bit for the first node to initialize
sleep 5

# Start node 2
echo "Starting node 2..."
cargo run --bin shardx -- --node-id node2 --port 54869 --data-dir ./data2 --log-level debug --shard-count 256 > node2.log 2>&1 &
NODE2_PID=$!
echo "Started node 2 with PID $NODE2_PID"

# Wait a bit for the second node to initialize
sleep 5

# Start node 3
echo "Starting node 3..."
cargo run --bin shardx -- --node-id node3 --port 54870 --data-dir ./data3 --log-level debug --shard-count 256 > node3.log 2>&1 &
NODE3_PID=$!
echo "Started node 3 with PID $NODE3_PID"

echo "All nodes started. Press Ctrl+C to stop all nodes."
echo "Node 1: http://localhost:54868"
echo "Node 2: http://localhost:54869"
echo "Node 3: http://localhost:54870"
echo "Check node1.log, node2.log, and node3.log for output"

# Wait for Ctrl+C
trap "kill $NODE1_PID $NODE2_PID $NODE3_PID; echo 'All nodes stopped.'; exit 0" INT
wait
