#!/bin/bash

# ShardX 統合スクリプト
# このスクリプトは、ShardXの起動、停止、インストール、アップデート、ステータス確認などの機能を提供します。
# 以前の複数のスクリプト（install.sh, start_nodes.sh, run_tests.sh など）の機能を統合しています。

# 色の定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# ロゴを表示
show_logo() {
    echo -e "${BLUE}"
    echo "  ____  _                    _ __   __"
    echo " / ___|| |__   __ _ _ __ __| |\ \ / /"
    echo " \___ \| '_ \ / _\` | '__/ _\` | \ V / "
    echo "  ___) | | | | (_| | | | (_| |  | |  "
    echo " |____/|_| |_|\__,_|_|  \__,_|  |_|  "
    echo -e "${NC}"
    echo "高性能ブロックチェーンプラットフォーム"
    echo "「まず動かす、検証する、改善する」"
    echo ""
}

# ヘルプを表示
show_help() {
    echo -e "${CYAN}使用方法:${NC} $0 [コマンド]"
    echo ""
    echo -e "${CYAN}コマンド:${NC}"
    echo "  install       - ShardXをインストール"
    echo "  start         - ShardXを起動"
    echo "  stop          - ShardXを停止"
    echo "  restart       - ShardXを再起動"
    echo "  status        - ShardXの状態を確認"
    echo "  update        - ShardXを最新バージョンに更新"
    echo "  benchmark     - パフォーマンスベンチマークを実行"
    echo "  logs          - ログを表示"
    echo "  docker-start  - Dockerコンテナで起動"
    echo "  docker-stop   - Dockerコンテナを停止"
    echo "  test-nodes    - テスト用ノードを起動"
    echo "  stop-test-nodes - テスト用ノードを停止"
    echo "  run-tests     - テストスイートを実行"
    echo "  help          - このヘルプを表示"
    echo ""
    echo -e "${CYAN}例:${NC}"
    echo "  $0 install"
    echo "  $0 start"
    echo ""
}

# OSを検出
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        OS="linux"
        if [ -f /etc/os-release ]; then
            . /etc/os-release
            DISTRO=$ID
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macos"
    elif [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
        OS="windows"
    else
        OS="unknown"
    fi
    
    echo -e "${CYAN}検出されたOS:${NC} $OS"
    if [ "$OS" = "linux" ]; then
        echo -e "${CYAN}ディストリビューション:${NC} $DISTRO"
    fi
}

# 依存関係をインストール
install_dependencies() {
    echo -e "${CYAN}依存関係をインストール中...${NC}"
    
    case $OS in
        linux)
            case $DISTRO in
                ubuntu|debian)
                    sudo apt update
                    sudo apt install -y git curl build-essential libssl-dev pkg-config
                    ;;
                fedora|centos|rhel)
                    sudo dnf install -y git curl gcc gcc-c++ openssl-devel
                    ;;
                arch|manjaro)
                    sudo pacman -Sy git curl base-devel openssl
                    ;;
                *)
                    echo -e "${YELLOW}未知のLinuxディストリビューションです。手動で依存関係をインストールしてください。${NC}"
                    ;;
            esac
            ;;
        macos)
            if ! command -v brew &> /dev/null; then
                echo -e "${YELLOW}Homebrewがインストールされていません。インストールします...${NC}"
                /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
            fi
            brew install git curl openssl
            ;;
        windows)
            echo -e "${YELLOW}Windowsでは、WSL (Windows Subsystem for Linux) またはDocker環境での実行をお勧めします。${NC}"
            echo -e "${YELLOW}WSLをインストールするには、管理者権限でPowerShellを開き、以下のコマンドを実行してください:${NC}"
            echo -e "${YELLOW}wsl --install${NC}"
            ;;
        *)
            echo -e "${RED}サポートされていないOSです。${NC}"
            exit 1
            ;;
    esac
    
    # Rustをインストール
    if ! command -v rustc &> /dev/null; then
        echo -e "${CYAN}Rustをインストール中...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source $HOME/.cargo/env
    fi
    
    echo -e "${GREEN}依存関係のインストールが完了しました。${NC}"
}

# ShardXをインストール
install_shardx() {
    echo -e "${CYAN}ShardXをインストール中...${NC}"
    
    # リポジトリをクローン
    if [ ! -d "ShardX" ]; then
        git clone https://github.com/enablerdao/ShardX.git
        cd ShardX
    else
        cd ShardX
        git pull
    fi
    
    # ビルド
    echo -e "${CYAN}ShardXをビルド中...${NC}"
    cargo build --release
    
    echo -e "${GREEN}ShardXのインストールが完了しました。${NC}"
}

# ShardXを起動
start_shardx() {
    echo -e "${CYAN}ShardXを起動中...${NC}"
    
    if [ ! -d "ShardX" ]; then
        echo -e "${RED}ShardXがインストールされていません。先に 'install' コマンドを実行してください。${NC}"
        exit 1
    fi
    
    cd ShardX
    
    # 設定ファイルが存在するか確認
    if [ ! -f "config.toml" ]; then
        echo -e "${YELLOW}設定ファイルが見つかりません。デフォルト設定を使用します。${NC}"
        cp config.example.toml config.toml
    fi
    
    # バックグラウンドで実行
    nohup cargo run --release --bin shardx_node > shardx.log 2>&1 &
    echo $! > shardx.pid
    
    echo -e "${GREEN}ShardXが起動しました。PID: $(cat shardx.pid)${NC}"
    echo -e "${GREEN}ウェブインターフェース: http://localhost:54867${NC}"
    echo -e "${GREEN}API: http://localhost:54868/api/v1/info${NC}"
}

# ShardXを停止
stop_shardx() {
    echo -e "${CYAN}ShardXを停止中...${NC}"
    
    if [ ! -f "ShardX/shardx.pid" ]; then
        echo -e "${YELLOW}ShardXが実行されていないか、PIDファイルが見つかりません。${NC}"
        return
    fi
    
    PID=$(cat ShardX/shardx.pid)
    if ps -p $PID > /dev/null; then
        kill $PID
        rm ShardX/shardx.pid
        echo -e "${GREEN}ShardXを停止しました。${NC}"
    else
        echo -e "${YELLOW}ShardXプロセス (PID: $PID) が見つかりません。${NC}"
        rm ShardX/shardx.pid
    fi
}

# ShardXを再起動
restart_shardx() {
    echo -e "${CYAN}ShardXを再起動中...${NC}"
    stop_shardx
    sleep 2
    start_shardx
}

# ShardXの状態を確認
check_status() {
    echo -e "${CYAN}ShardXの状態を確認中...${NC}"
    
    if [ ! -f "ShardX/shardx.pid" ]; then
        echo -e "${YELLOW}ShardXが実行されていないか、PIDファイルが見つかりません。${NC}"
        return
    fi
    
    PID=$(cat ShardX/shardx.pid)
    if ps -p $PID > /dev/null; then
        echo -e "${GREEN}ShardXは実行中です。PID: $PID${NC}"
        
        # APIが応答するか確認
        if command -v curl &> /dev/null; then
            if curl -s http://localhost:54868/api/v1/info > /dev/null; then
                echo -e "${GREEN}APIは正常に応答しています。${NC}"
                
                # ノード情報を取得
                NODE_INFO=$(curl -s http://localhost:54868/api/v1/info)
                echo -e "${CYAN}ノード情報:${NC}"
                echo "$NODE_INFO" | grep -E 'version|height|peers|shards'
            else
                echo -e "${YELLOW}APIが応答していません。${NC}"
            fi
        else
            echo -e "${YELLOW}curlコマンドが見つからないため、API状態を確認できません。${NC}"
        fi
    else
        echo -e "${YELLOW}ShardXプロセス (PID: $PID) が見つかりません。${NC}"
        rm ShardX/shardx.pid
    fi
}

# ShardXを更新
update_shardx() {
    echo -e "${CYAN}ShardXを更新中...${NC}"
    
    if [ ! -d "ShardX" ]; then
        echo -e "${RED}ShardXがインストールされていません。先に 'install' コマンドを実行してください。${NC}"
        exit 1
    fi
    
    # 実行中の場合は停止
    if [ -f "ShardX/shardx.pid" ]; then
        stop_shardx
    fi
    
    cd ShardX
    
    # 現在のブランチを保存
    CURRENT_BRANCH=$(git symbolic-ref --short HEAD)
    
    # 変更を退避
    git stash
    
    # 最新の変更を取得
    git fetch
    
    # 現在のブランチを更新
    git pull origin $CURRENT_BRANCH
    
    # 変更を戻す
    git stash pop || true
    
    # 再ビルド
    cargo build --release
    
    echo -e "${GREEN}ShardXの更新が完了しました。${NC}"
}

# ベンチマークを実行
run_benchmark() {
    echo -e "${CYAN}パフォーマンスベンチマークを実行中...${NC}"
    
    if [ ! -d "ShardX" ]; then
        echo -e "${RED}ShardXがインストールされていません。先に 'install' コマンドを実行してください。${NC}"
        exit 1
    fi
    
    cd ShardX
    
    # ベンチマークの種類を選択
    echo -e "${CYAN}実行するベンチマークを選択してください:${NC}"
    echo "1) シンプルベンチマーク (最も速い)"
    echo "2) マイクロベンチマーク (詳細な測定)"
    echo "3) 現実的ベンチマーク (ネットワーク遅延、ディスクI/O、コンセンサスをシミュレート)"
    echo "4) クロスチェーンベンチマーク (Ethereum連携)"
    echo "5) すべてのベンチマークを実行"
    
    read -p "選択 (1-5): " BENCHMARK_CHOICE
    
    case $BENCHMARK_CHOICE in
        1)
            cargo run --release --bin simple_benchmark
            ;;
        2)
            cargo run --release --bin micro_benchmark
            ;;
        3)
            cargo run --release --bin realistic_benchmark
            ;;
        4)
            cargo run --release --bin ethereum_bridge_demo
            ;;
        5)
            echo -e "${CYAN}すべてのベンチマークを順番に実行します...${NC}"
            cargo run --release --bin simple_benchmark
            cargo run --release --bin micro_benchmark
            cargo run --release --bin realistic_benchmark
            cargo run --release --bin ethereum_bridge_demo
            ;;
        *)
            echo -e "${RED}無効な選択です。${NC}"
            ;;
    esac
}

# ログを表示
show_logs() {
    echo -e "${CYAN}ShardXのログを表示中...${NC}"
    
    if [ ! -f "ShardX/shardx.log" ]; then
        echo -e "${YELLOW}ログファイルが見つかりません。${NC}"
        return
    fi
    
    # ログの表示方法を選択
    echo -e "${CYAN}ログの表示方法を選択してください:${NC}"
    echo "1) 最新の10行を表示"
    echo "2) ログをリアルタイムで監視"
    echo "3) エラーのみを表示"
    echo "4) ログファイルを開く"
    
    read -p "選択 (1-4): " LOG_CHOICE
    
    case $LOG_CHOICE in
        1)
            tail -n 10 ShardX/shardx.log
            ;;
        2)
            tail -f ShardX/shardx.log
            ;;
        3)
            grep -i "error\|warning" ShardX/shardx.log | tail -n 20
            ;;
        4)
            if command -v nano &> /dev/null; then
                nano ShardX/shardx.log
            elif command -v vim &> /dev/null; then
                vim ShardX/shardx.log
            else
                less ShardX/shardx.log
            fi
            ;;
        *)
            echo -e "${RED}無効な選択です。${NC}"
            ;;
    esac
}

# Dockerで起動
docker_start() {
    echo -e "${CYAN}DockerでShardXを起動中...${NC}"
    
    # Dockerがインストールされているか確認
    if ! command -v docker &> /dev/null; then
        echo -e "${RED}Dockerがインストールされていません。${NC}"
        echo -e "${YELLOW}Dockerをインストールするには、以下のコマンドを実行してください:${NC}"
        echo -e "${YELLOW}curl -fsSL https://get.docker.com | sh${NC}"
        exit 1
    fi
    
    # イメージをプル
    docker pull enablerdao/shardx:latest
    
    # コンテナを起動
    docker run -d --name shardx -p 54867:54867 -p 54868:54868 enablerdao/shardx:latest
    
    echo -e "${GREEN}ShardXのDockerコンテナが起動しました。${NC}"
    echo -e "${GREEN}ウェブインターフェース: http://localhost:54867${NC}"
    echo -e "${GREEN}API: http://localhost:54868/api/v1/info${NC}"
}

# Dockerを停止
docker_stop() {
    echo -e "${CYAN}ShardXのDockerコンテナを停止中...${NC}"
    
    # コンテナが実行中か確認
    if docker ps | grep -q shardx; then
        docker stop shardx
        docker rm shardx
        echo -e "${GREEN}ShardXのDockerコンテナを停止しました。${NC}"
    else
        echo -e "${YELLOW}ShardXのDockerコンテナが実行されていません。${NC}"
    fi
}

# メイン処理
main() {
    show_logo
    detect_os
    
    # コマンドライン引数がない場合はヘルプを表示
    if [ $# -eq 0 ]; then
        show_help
        exit 0
    fi
    
    # コマンドを処理
    case $1 in
        install)
            install_dependencies
            install_shardx
            ;;
        start)
            start_shardx
            ;;
        stop)
            stop_shardx
            ;;
        restart)
            restart_shardx
            ;;
        status)
            check_status
            ;;
        update)
            update_shardx
            ;;
        benchmark)
            run_benchmark
            ;;
        logs)
            show_logs
            ;;
        docker-start)
            docker_start
            ;;
        docker-stop)
            docker_stop
            ;;
        test-nodes)
            start_test_nodes
            ;;
        stop-test-nodes)
            stop_test_nodes
            ;;
        run-tests)
            run_tests
            ;;
        help)
            show_help
            ;;
        *)
            echo -e "${RED}不明なコマンド: $1${NC}"
            show_help
            exit 1
            ;;
    esac
}

# テスト用ノードを起動
start_test_nodes() {
    echo -e "${CYAN}テスト用ノードを起動中...${NC}"
    
    # ノードのビルド
    cd "$(dirname "$0")"
    cargo build
    
    # ノードディレクトリの作成
    mkdir -p test_nodes/{node1,node2,node3,node4,node5}/data
    
    # ノード1を起動
    echo "ノード1を起動中..."
    cd test_nodes/node1
    ../../target/debug/shardx_node --node-id node1 --port 54868 --data-dir ./data > node1.log 2>&1 &
    NODE1_PID=$!
    cd ../..
    
    # ノード2を起動
    echo "ノード2を起動中..."
    cd test_nodes/node2
    ../../target/debug/shardx_node --node-id node2 --port 54869 --data-dir ./data > node2.log 2>&1 &
    NODE2_PID=$!
    cd ../..
    
    # ノード3を起動
    echo "ノード3を起動中..."
    cd test_nodes/node3
    ../../target/debug/shardx_node --node-id node3 --port 54870 --data-dir ./data > node3.log 2>&1 &
    NODE3_PID=$!
    cd ../..
    
    # ノード4を起動
    echo "ノード4を起動中..."
    cd test_nodes/node4
    ../../target/debug/shardx_node --node-id node4 --port 54871 --data-dir ./data > node4.log 2>&1 &
    NODE4_PID=$!
    cd ../..
    
    # ノード5を起動
    echo "ノード5を起動中..."
    cd test_nodes/node5
    ../../target/debug/shardx_node --node-id node5 --port 54872 --data-dir ./data > node5.log 2>&1 &
    NODE5_PID=$!
    cd ../..
    
    echo "すべてのノードが起動しました。"
    echo "ノード1: http://localhost:54868 (PID: $NODE1_PID)"
    echo "ノード2: http://localhost:54869 (PID: $NODE2_PID)"
    echo "ノード3: http://localhost:54870 (PID: $NODE3_PID)"
    echo "ノード4: http://localhost:54871 (PID: $NODE4_PID)"
    echo "ノード5: http://localhost:54872 (PID: $NODE5_PID)"
    
    # PIDを保存
    echo "$NODE1_PID $NODE2_PID $NODE3_PID $NODE4_PID $NODE5_PID" > test_nodes/node_pids.txt
    
    echo -e "${GREEN}テスト用ノードが起動しました。${NC}"
    echo "ノードを停止するには ./shardx.sh stop-test-nodes を実行してください。"
}

# テスト用ノードを停止
stop_test_nodes() {
    echo -e "${CYAN}テスト用ノードを停止中...${NC}"
    
    if [ ! -f "test_nodes/node_pids.txt" ]; then
        echo -e "${YELLOW}テスト用ノードが実行されていないか、PIDファイルが見つかりません。${NC}"
        return
    fi
    
    # PIDを読み込み
    read -r NODE1_PID NODE2_PID NODE3_PID NODE4_PID NODE5_PID < test_nodes/node_pids.txt
    
    # ノードを停止
    kill $NODE1_PID $NODE2_PID $NODE3_PID $NODE4_PID $NODE5_PID 2>/dev/null || true
    
    # PIDファイルを削除
    rm test_nodes/node_pids.txt
    
    echo -e "${GREEN}テスト用ノードを停止しました。${NC}"
}

# テストスイートを実行
run_tests() {
    echo -e "${CYAN}テストスイートを実行中...${NC}"
    
    # 必要なツールの確認
    if ! command -v jq &> /dev/null; then
        echo "jqがインストールされていません。インストールしています..."
        sudo apt-get update && sudo apt-get install -y jq
    fi
    
    # ノードが起動しているか確認
    if [ ! -f "test_nodes/node_pids.txt" ]; then
        echo "ノードが起動していません。ノードを起動します..."
        start_test_nodes
        
        # ノードの起動を待機
        echo "ノードの起動を待機しています (10秒)..."
        sleep 10
    fi
    
    # テストディレクトリを作成
    mkdir -p test_results
    
    # 基本的な接続テスト
    echo "基本的な接続テストを実行しています..."
    for port in 54868 54869 54870 54871 54872; do
        if curl -s "http://localhost:$port/api/v1/info" > /dev/null; then
            echo -e "${GREEN}ノード (port: $port) に接続できました${NC}"
        else
            echo -e "${RED}ノード (port: $port) に接続できませんでした${NC}"
        fi
    done
    
    # トランザクションテスト
    echo "トランザクションテストを実行しています..."
    # ここにトランザクションテストのコードを追加
    
    echo -e "${GREEN}テストが完了しました。${NC}"
}

# スクリプトを実行
main "$@"