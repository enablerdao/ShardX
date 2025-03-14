#!/bin/bash

# ノードを停止するスクリプト
cd "$(dirname "$0")"

if [ -f node_pids.txt ]; then
    echo "ノードを停止中..."
    
    # PIDを読み込む
    read -r NODE1_PID NODE2_PID NODE3_PID NODE4_PID NODE5_PID < node_pids.txt
    
    # ノードを停止
    kill $NODE1_PID 2>/dev/null || echo "ノード1はすでに停止しています"
    kill $NODE2_PID 2>/dev/null || echo "ノード2はすでに停止しています"
    kill $NODE3_PID 2>/dev/null || echo "ノード3はすでに停止しています"
    kill $NODE4_PID 2>/dev/null || echo "ノード4はすでに停止しています"
    kill $NODE5_PID 2>/dev/null || echo "ノード5はすでに停止しています"
    
    rm node_pids.txt
    
    echo "すべてのノードを停止しました。"
else
    echo "node_pids.txtが見つかりません。ノードが起動していない可能性があります。"
fi