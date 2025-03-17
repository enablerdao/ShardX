#!/bin/bash

# ShardX静的解析ツール実行スクリプト
# このスクリプトは、ShardXプロジェクトに対して静的解析ツールを実行します。

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

# 設定ファイルのパス
CLIPPY_CONFIG="$PROJECT_ROOT/tools/static_analysis/clippy_config.toml"
RUSTFMT_CONFIG="$PROJECT_ROOT/tools/static_analysis/rustfmt.toml"

# 出力ディレクトリ
OUTPUT_DIR="$PROJECT_ROOT/target/static_analysis"
mkdir -p "$OUTPUT_DIR"

# ヘルプメッセージを表示
function show_help {
    echo -e "${BLUE}ShardX静的解析ツール${NC}"
    echo ""
    echo "使用方法: $0 [オプション]"
    echo ""
    echo "オプション:"
    echo "  -h, --help       このヘルプメッセージを表示"
    echo "  -c, --clippy     Clippyを実行"
    echo "  -f, --fmt        rustfmtを実行"
    echo "  -a, --all        すべての静的解析ツールを実行"
    echo "  -v, --verbose    詳細な出力を表示"
    echo "  -x, --fix        可能な問題を自動修正"
    echo ""
    echo "例:"
    echo "  $0 --all         すべての静的解析ツールを実行"
    echo "  $0 --clippy      Clippyのみを実行"
    echo "  $0 --fmt --fix   rustfmtを実行し、問題を自動修正"
}

# Clippyを実行
function run_clippy {
    echo -e "${BLUE}Clippyを実行中...${NC}"
    
    local clippy_args=("--all-targets" "--all-features")
    
    if [ -f "$CLIPPY_CONFIG" ]; then
        clippy_args+=("--config-path" "$CLIPPY_CONFIG")
    fi
    
    if [ "$VERBOSE" = true ]; then
        clippy_args+=("--verbose")
    fi
    
    if [ "$FIX" = true ]; then
        clippy_args+=("--fix" "--allow-dirty" "--allow-staged")
    fi
    
    local clippy_output="$OUTPUT_DIR/clippy_output.txt"
    
    if cargo clippy "${clippy_args[@]}" 2>&1 | tee "$clippy_output"; then
        echo -e "${GREEN}Clippy: 問題は検出されませんでした${NC}"
        return 0
    else
        local warning_count=$(grep -c "warning:" "$clippy_output" || true)
        local error_count=$(grep -c "error:" "$clippy_output" || true)
        
        echo -e "${YELLOW}Clippy: $warning_count 件の警告と $error_count 件のエラーが検出されました${NC}"
        
        if [ "$error_count" -gt 0 ]; then
            return 1
        else
            return 0
        fi
    fi
}

# rustfmtを実行
function run_rustfmt {
    echo -e "${BLUE}rustfmtを実行中...${NC}"
    
    local rustfmt_args=("--all")
    
    if [ -f "$RUSTFMT_CONFIG" ]; then
        rustfmt_args+=("--config-path" "$RUSTFMT_CONFIG")
    fi
    
    if [ "$VERBOSE" = true ]; then
        rustfmt_args+=("--verbose")
    fi
    
    if [ "$FIX" = true ]; then
        # 実際に修正を適用
        if cargo fmt "${rustfmt_args[@]}"; then
            echo -e "${GREEN}rustfmt: すべてのファイルがフォーマットされました${NC}"
            return 0
        else
            echo -e "${RED}rustfmt: フォーマット中にエラーが発生しました${NC}"
            return 1
        fi
    else
        # チェックのみ
        local rustfmt_output="$OUTPUT_DIR/rustfmt_output.txt"
        
        if cargo fmt --check "${rustfmt_args[@]}" 2>&1 | tee "$rustfmt_output"; then
            echo -e "${GREEN}rustfmt: すべてのファイルが正しくフォーマットされています${NC}"
            return 0
        else
            local unformatted_count=$(grep -c "Diff in" "$rustfmt_output" || true)
            echo -e "${YELLOW}rustfmt: $unformatted_count 件のファイルが正しくフォーマットされていません${NC}"
            return 1
        fi
    fi
}

# 静的解析の概要レポートを生成
function generate_report {
    local report_file="$OUTPUT_DIR/static_analysis_report.md"
    
    echo "# ShardX静的解析レポート" > "$report_file"
    echo "" >> "$report_file"
    echo "実行日時: $(date)" >> "$report_file"
    echo "" >> "$report_file"
    
    echo "## Clippy結果" >> "$report_file"
    if [ -f "$OUTPUT_DIR/clippy_output.txt" ]; then
        local clippy_warning_count=$(grep -c "warning:" "$OUTPUT_DIR/clippy_output.txt" || true)
        local clippy_error_count=$(grep -c "error:" "$OUTPUT_DIR/clippy_output.txt" || true)
        
        echo "- 警告: $clippy_warning_count" >> "$report_file"
        echo "- エラー: $clippy_error_count" >> "$report_file"
        
        if [ "$clippy_warning_count" -gt 0 ] || [ "$clippy_error_count" -gt 0 ]; then
            echo "" >> "$report_file"
            echo "### 検出された問題" >> "$report_file"
            echo '```' >> "$report_file"
            grep -E "warning:|error:" "$OUTPUT_DIR/clippy_output.txt" | sort | uniq -c | sort -nr >> "$report_file"
            echo '```' >> "$report_file"
        fi
    else
        echo "- Clippy結果が見つかりません" >> "$report_file"
    fi
    
    echo "" >> "$report_file"
    echo "## rustfmt結果" >> "$report_file"
    if [ -f "$OUTPUT_DIR/rustfmt_output.txt" ]; then
        local unformatted_count=$(grep -c "Diff in" "$OUTPUT_DIR/rustfmt_output.txt" || true)
        
        echo "- フォーマットされていないファイル: $unformatted_count" >> "$report_file"
        
        if [ "$unformatted_count" -gt 0 ]; then
            echo "" >> "$report_file"
            echo "### フォーマットされていないファイル" >> "$report_file"
            echo '```' >> "$report_file"
            grep "Diff in" "$OUTPUT_DIR/rustfmt_output.txt" >> "$report_file"
            echo '```' >> "$report_file"
        fi
    else
        echo "- rustfmt結果が見つかりません" >> "$report_file"
    fi
    
    echo "" >> "$report_file"
    echo "## 推奨される対応" >> "$report_file"
    echo "" >> "$report_file"
    
    if [ -f "$OUTPUT_DIR/clippy_output.txt" ] && ([ "$(grep -c "warning:" "$OUTPUT_DIR/clippy_output.txt" || true)" -gt 0 ] || [ "$(grep -c "error:" "$OUTPUT_DIR/clippy_output.txt" || true)" -gt 0 ]); then
        echo "### Clippy警告/エラーの修正" >> "$report_file"
        echo "以下のコマンドを実行して、Clippyの警告とエラーを修正してください：" >> "$report_file"
        echo '```bash' >> "$report_file"
        echo "./tools/static_analysis/run_analysis.sh --clippy --fix" >> "$report_file"
        echo '```' >> "$report_file"
        echo "" >> "$report_file"
    fi
    
    if [ -f "$OUTPUT_DIR/rustfmt_output.txt" ] && [ "$(grep -c "Diff in" "$OUTPUT_DIR/rustfmt_output.txt" || true)" -gt 0 ]; then
        echo "### コードフォーマットの修正" >> "$report_file"
        echo "以下のコマンドを実行して、コードフォーマットの問題を修正してください：" >> "$report_file"
        echo '```bash' >> "$report_file"
        echo "./tools/static_analysis/run_analysis.sh --fmt --fix" >> "$report_file"
        echo '```' >> "$report_file"
        echo "" >> "$report_file"
    fi
    
    echo "レポートが生成されました: $report_file"
}

# デフォルト値
RUN_CLIPPY=false
RUN_FMT=false
VERBOSE=false
FIX=false

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
        -c|--clippy)
            RUN_CLIPPY=true
            shift
            ;;
        -f|--fmt)
            RUN_FMT=true
            shift
            ;;
        -a|--all)
            RUN_CLIPPY=true
            RUN_FMT=true
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -x|--fix)
            FIX=true
            shift
            ;;
        *)
            echo "不明なオプション: $1"
            show_help
            exit 1
            ;;
    esac
done

# 結果の追跡
CLIPPY_RESULT=0
FMT_RESULT=0

# 選択されたツールを実行
if [ "$RUN_CLIPPY" = true ]; then
    run_clippy
    CLIPPY_RESULT=$?
fi

if [ "$RUN_FMT" = true ]; then
    run_rustfmt
    FMT_RESULT=$?
fi

# レポートを生成
generate_report

# 終了コードを決定
if [ "$CLIPPY_RESULT" -ne 0 ] || [ "$FMT_RESULT" -ne 0 ]; then
    echo -e "${YELLOW}静的解析で問題が検出されました。詳細はレポートを確認してください。${NC}"
    exit 1
else
    echo -e "${GREEN}静的解析は正常に完了しました。問題は検出されませんでした。${NC}"
    exit 0
fi