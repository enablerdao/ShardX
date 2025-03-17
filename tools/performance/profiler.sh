#!/bin/bash

# ShardX パフォーマンスプロファイリングスクリプト
# このスクリプトは、ShardXのパフォーマンスをプロファイリングするためのツールです。

set -e

# 色の定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# プロジェクトルートディレクトリ
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$PROJECT_ROOT"

# 出力ディレクトリ
OUTPUT_DIR="$PROJECT_ROOT/target/profile"
mkdir -p "$OUTPUT_DIR"

# 現在の日時
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# ヘルプメッセージを表示
function show_help {
    echo -e "${BLUE}ShardXパフォーマンスプロファイリングツール${NC}"
    echo ""
    echo "使用方法: $0 [オプション] [バイナリ名]"
    echo ""
    echo "オプション:"
    echo "  -h, --help           このヘルプメッセージを表示"
    echo "  -c, --cpu            CPUプロファイリングを実行"
    echo "  -m, --memory         メモリプロファイリングを実行"
    echo "  -a, --all            すべてのプロファイリングを実行"
    echo "  -f, --flamegraph     フレームグラフを生成"
    echo "  -t, --time <seconds> プロファイリング時間を指定（デフォルト: 30秒）"
    echo "  -o, --output <dir>   出力ディレクトリを指定（デフォルト: target/profile）"
    echo ""
    echo "例:"
    echo "  $0 --cpu shardx      ShardXバイナリのCPUプロファイリングを実行"
    echo "  $0 --memory --time 60 shardx  60秒間のメモリプロファイリングを実行"
    echo "  $0 --all --flamegraph shardx  すべてのプロファイリングとフレームグラフを生成"
}

# 必要なツールがインストールされているか確認
function check_dependencies {
    echo -e "${BLUE}依存関係を確認中...${NC}"
    
    local missing_deps=()
    
    # perf
    if ! command -v perf &> /dev/null; then
        missing_deps+=("perf (linux-tools-common)")
    fi
    
    # valgrind
    if ! command -v valgrind &> /dev/null; then
        missing_deps+=("valgrind")
    fi
    
    # flamegraph
    if ! command -v flamegraph &> /dev/null && [ "$GENERATE_FLAMEGRAPH" = true ]; then
        missing_deps+=("flamegraph (cargo install flamegraph)")
    fi
    
    # inferno (rust flamegraph)
    if ! command -v inferno-flamegraph &> /dev/null && [ "$GENERATE_FLAMEGRAPH" = true ]; then
        missing_deps+=("inferno (cargo install inferno)")
    fi
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        echo -e "${YELLOW}以下の依存関係が見つかりません:${NC}"
        for dep in "${missing_deps[@]}"; do
            echo "  - $dep"
        done
        
        echo -e "${YELLOW}インストール方法:${NC}"
        echo "  sudo apt-get install linux-tools-common linux-tools-generic valgrind"
        echo "  cargo install flamegraph"
        echo "  cargo install inferno"
        
        read -p "続行しますか？ (y/n) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    else
        echo -e "${GREEN}すべての依存関係が見つかりました${NC}"
    fi
}

# CPUプロファイリングを実行
function run_cpu_profiling {
    echo -e "${BLUE}CPUプロファイリングを実行中...${NC}"
    
    local binary="$1"
    local output_file="$OUTPUT_DIR/cpu_profile_${binary}_${TIMESTAMP}.data"
    
    echo "バイナリ: $binary"
    echo "プロファイリング時間: ${PROFILE_TIME}秒"
    echo "出力ファイル: $output_file"
    
    # perfを使用してCPUプロファイリングを実行
    perf record -F 99 -g -o "$output_file" -- "$PROJECT_ROOT/target/release/$binary" &
    local pid=$!
    
    echo "プロファイリング中... (PID: $pid)"
    sleep "$PROFILE_TIME"
    
    # プロセスを終了
    kill -INT $pid 2>/dev/null || true
    wait $pid 2>/dev/null || true
    
    # レポートを生成
    local report_file="$OUTPUT_DIR/cpu_report_${binary}_${TIMESTAMP}.txt"
    perf report -i "$output_file" > "$report_file"
    
    echo -e "${GREEN}CPUプロファイリングが完了しました${NC}"
    echo "レポート: $report_file"
    
    # フレームグラフを生成
    if [ "$GENERATE_FLAMEGRAPH" = true ]; then
        local flamegraph_file="$OUTPUT_DIR/cpu_flamegraph_${binary}_${TIMESTAMP}.svg"
        echo -e "${BLUE}フレームグラフを生成中...${NC}"
        
        perf script -i "$output_file" | inferno-flamegraph > "$flamegraph_file"
        
        echo -e "${GREEN}フレームグラフが生成されました: $flamegraph_file${NC}"
    fi
}

# メモリプロファイリングを実行
function run_memory_profiling {
    echo -e "${BLUE}メモリプロファイリングを実行中...${NC}"
    
    local binary="$1"
    local output_file="$OUTPUT_DIR/memory_profile_${binary}_${TIMESTAMP}.txt"
    
    echo "バイナリ: $binary"
    echo "プロファイリング時間: ${PROFILE_TIME}秒"
    echo "出力ファイル: $output_file"
    
    # valgrindを使用してメモリプロファイリングを実行
    valgrind --tool=massif --time-unit=ms --massif-out-file="$OUTPUT_DIR/massif_${binary}_${TIMESTAMP}.out" \
        "$PROJECT_ROOT/target/release/$binary" &
    local pid=$!
    
    echo "プロファイリング中... (PID: $pid)"
    sleep "$PROFILE_TIME"
    
    # プロセスを終了
    kill -INT $pid 2>/dev/null || true
    wait $pid 2>/dev/null || true
    
    # レポートを生成
    ms_print "$OUTPUT_DIR/massif_${binary}_${TIMESTAMP}.out" > "$output_file"
    
    echo -e "${GREEN}メモリプロファイリングが完了しました${NC}"
    echo "レポート: $output_file"
}

# プロファイリング結果の概要レポートを生成
function generate_report {
    local binary="$1"
    local report_file="$OUTPUT_DIR/profile_report_${binary}_${TIMESTAMP}.md"
    
    echo "# ShardXパフォーマンスプロファイリングレポート" > "$report_file"
    echo "" >> "$report_file"
    echo "実行日時: $(date)" >> "$report_file"
    echo "バイナリ: $binary" >> "$report_file"
    echo "プロファイリング時間: ${PROFILE_TIME}秒" >> "$report_file"
    echo "" >> "$report_file"
    
    # システム情報
    echo "## システム情報" >> "$report_file"
    echo "```" >> "$report_file"
    uname -a >> "$report_file"
    echo "" >> "$report_file"
    if [ -f /proc/cpuinfo ]; then
        grep "model name" /proc/cpuinfo | head -1 >> "$report_file"
        grep "cpu cores" /proc/cpuinfo | head -1 >> "$report_file"
    fi
    echo "" >> "$report_file"
    if [ -f /proc/meminfo ]; then
        grep "MemTotal" /proc/meminfo >> "$report_file"
    fi
    echo "```" >> "$report_file"
    echo "" >> "$report_file"
    
    # CPUプロファイリング結果
    if [ "$RUN_CPU_PROFILING" = true ]; then
        echo "## CPUプロファイリング結果" >> "$report_file"
        echo "" >> "$report_file"
        
        local cpu_report="$OUTPUT_DIR/cpu_report_${binary}_${TIMESTAMP}.txt"
        if [ -f "$cpu_report" ]; then
            echo "### ホットスポット（上位10件）" >> "$report_file"
            echo "```" >> "$report_file"
            grep -A 10 "Overhead" "$cpu_report" >> "$report_file"
            echo "```" >> "$report_file"
            echo "" >> "$report_file"
        fi
        
        if [ "$GENERATE_FLAMEGRAPH" = true ]; then
            local flamegraph_file="cpu_flamegraph_${binary}_${TIMESTAMP}.svg"
            echo "### フレームグラフ" >> "$report_file"
            echo "![CPUフレームグラフ]($flamegraph_file)" >> "$report_file"
            echo "" >> "$report_file"
        fi
    fi
    
    # メモリプロファイリング結果
    if [ "$RUN_MEMORY_PROFILING" = true ]; then
        echo "## メモリプロファイリング結果" >> "$report_file"
        echo "" >> "$report_file"
        
        local memory_report="$OUTPUT_DIR/memory_profile_${binary}_${TIMESTAMP}.txt"
        if [ -f "$memory_report" ]; then
            echo "### メモリ使用量の概要" >> "$report_file"
            echo "```" >> "$report_file"
            grep -A 5 "Heap Summary" "$memory_report" >> "$report_file"
            echo "```" >> "$report_file"
            echo "" >> "$report_file"
            
            echo "### メモリ割り当て（上位10件）" >> "$report_file"
            echo "```" >> "$report_file"
            grep -A 10 "detailed snapshot" "$memory_report" | grep -v "n/a" >> "$report_file"
            echo "```" >> "$report_file"
            echo "" >> "$report_file"
        fi
    fi
    
    # 推奨事項
    echo "## 推奨事項" >> "$report_file"
    echo "" >> "$report_file"
    echo "以下の点について検討してください：" >> "$report_file"
    echo "" >> "$report_file"
    echo "1. **CPUホットスポットの最適化**: プロファイリング結果で特定された高負荷の関数を最適化する" >> "$report_file"
    echo "2. **メモリ割り当ての削減**: 頻繁なメモリ割り当てを行っている箇所を特定し、オブジェクトプールやアリーナアロケータの使用を検討する" >> "$report_file"
    echo "3. **並列処理の改善**: 並列処理の効率を向上させるため、ロックの競合を減らし、タスク分割を最適化する" >> "$report_file"
    echo "4. **I/O操作の最適化**: ディスクI/OやネットワークI/Oのボトルネックを特定し、非同期処理やバッファリングを改善する" >> "$report_file"
    echo "" >> "$report_file"
    
    echo "レポートが生成されました: $report_file"
}

# デフォルト値
RUN_CPU_PROFILING=false
RUN_MEMORY_PROFILING=false
GENERATE_FLAMEGRAPH=false
PROFILE_TIME=30

# 引数がない場合はヘルプを表示
if [ $# -eq 0 ]; then
    show_help
    exit 0
fi

# 引数の解析
while [ $# -gt 0 ]; do
    case "$1" in
        -h|--help)
            show_help
            exit 0
            ;;
        -c|--cpu)
            RUN_CPU_PROFILING=true
            shift
            ;;
        -m|--memory)
            RUN_MEMORY_PROFILING=true
            shift
            ;;
        -a|--all)
            RUN_CPU_PROFILING=true
            RUN_MEMORY_PROFILING=true
            shift
            ;;
        -f|--flamegraph)
            GENERATE_FLAMEGRAPH=true
            shift
            ;;
        -t|--time)
            PROFILE_TIME="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            mkdir -p "$OUTPUT_DIR"
            shift 2
            ;;
        *)
            BINARY="$1"
            shift
            ;;
    esac
done

# バイナリが指定されていない場合はエラー
if [ -z "$BINARY" ]; then
    echo -e "${RED}エラー: プロファイリングするバイナリを指定してください${NC}"
    show_help
    exit 1
fi

# バイナリが存在するか確認
if [ ! -f "$PROJECT_ROOT/target/release/$BINARY" ]; then
    echo -e "${RED}エラー: バイナリが見つかりません: $PROJECT_ROOT/target/release/$BINARY${NC}"
    echo "リリースビルドを実行してください: cargo build --release"
    exit 1
fi

# 依存関係を確認
check_dependencies

# プロファイリングを実行
if [ "$RUN_CPU_PROFILING" = true ]; then
    run_cpu_profiling "$BINARY"
fi

if [ "$RUN_MEMORY_PROFILING" = true ]; then
    run_memory_profiling "$BINARY"
fi

# レポートを生成
generate_report "$BINARY"

echo -e "${GREEN}プロファイリングが完了しました。結果は $OUTPUT_DIR ディレクトリに保存されています。${NC}"
exit 0