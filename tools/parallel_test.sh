#!/bin/bash
set -e

# 色の定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}ShardX 並列テスト実行ツール${NC}"
echo "=============================="

# 引数の解析
THREADS=$(nproc)
FILTER=""
VERBOSE=false
NOCAPTURE=false
IGNORED=false
FEATURES=""

print_usage() {
  echo -e "使用方法: $0 [オプション]"
  echo -e "オプション:"
  echo -e "  --threads <数>\t\t並列実行するスレッド数（デフォルト: CPU数）"
  echo -e "  --filter <パターン>\t\t特定のテストのみを実行"
  echo -e "  --verbose\t\t詳細な出力"
  echo -e "  --nocapture\t\t標準出力をキャプチャしない"
  echo -e "  --ignored\t\t#[ignore]属性のテストも実行"
  echo -e "  --features <機能>\t\t特定の機能フラグを有効化"
  echo -e "  --help\t\tこのヘルプメッセージを表示"
}

while [[ $# -gt 0 ]]; do
  case $1 in
    --threads)
      THREADS="$2"
      shift 2
      ;;
    --filter)
      FILTER="$2"
      shift 2
      ;;
    --verbose)
      VERBOSE=true
      shift
      ;;
    --nocapture)
      NOCAPTURE=true
      shift
      ;;
    --ignored)
      IGNORED=true
      shift
      ;;
    --features)
      FEATURES="$2"
      shift 2
      ;;
    --help)
      print_usage
      exit 0
      ;;
    *)
      echo -e "${RED}エラー: 不明なオプション: $1${NC}"
      print_usage
      exit 1
      ;;
  esac
done

# テスト対象のモジュールを取得
echo -e "${BLUE}テスト対象のモジュールを検索中...${NC}"
MODULES=$(find src -name "*.rs" | grep -v "mod.rs" | sed 's|^src/||' | sed 's|\.rs$||' | sed 's|/|::|g')

# テスト対象のモジュールをグループ化
TOTAL_MODULES=$(echo "$MODULES" | wc -l)
MODULES_PER_GROUP=$(( (TOTAL_MODULES + THREADS - 1) / THREADS ))

echo -e "${BLUE}合計 $TOTAL_MODULES モジュールを $THREADS スレッドで実行（1スレッドあたり約 $MODULES_PER_GROUP モジュール）${NC}"

# 一時ディレクトリを作成
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

# モジュールをグループに分割
split -l $MODULES_PER_GROUP -d -a 2 <(echo "$MODULES") "$TEMP_DIR/group_"

# 各グループを並列実行
echo -e "${BLUE}テストを並列実行中...${NC}"

CARGO_ARGS=""
if [ -n "$FEATURES" ]; then
  CARGO_ARGS="$CARGO_ARGS --features=$FEATURES"
fi

if [ "$VERBOSE" = true ]; then
  CARGO_ARGS="$CARGO_ARGS --verbose"
fi

if [ "$NOCAPTURE" = true ]; then
  CARGO_ARGS="$CARGO_ARGS -- --nocapture"
fi

if [ "$IGNORED" = true ]; then
  CARGO_ARGS="$CARGO_ARGS -- --ignored"
fi

START_TIME=$(date +%s)

# 結果を保存するディレクトリ
RESULTS_DIR="$TEMP_DIR/results"
mkdir -p "$RESULTS_DIR"

# 並列実行
for group_file in "$TEMP_DIR"/group_*; do
  group_name=$(basename "$group_file")
  (
    echo -e "${YELLOW}グループ $group_name の実行を開始...${NC}"
    
    # このグループのモジュールに対してテストを実行
    if [ -n "$FILTER" ]; then
      # フィルターが指定されている場合
      cargo test $CARGO_ARGS "$FILTER" $(cat "$group_file" | tr '\n' ' ') > "$RESULTS_DIR/$group_name.log" 2>&1
    else
      # フィルターが指定されていない場合
      cargo test $CARGO_ARGS $(cat "$group_file" | tr '\n' ' ') > "$RESULTS_DIR/$group_name.log" 2>&1
    fi
    
    # 終了ステータスを保存
    echo $? > "$RESULTS_DIR/$group_name.status"
    
    echo -e "${GREEN}グループ $group_name の実行が完了しました${NC}"
  ) &
done

# すべてのジョブが完了するのを待つ
wait

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

# 結果を集計
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
FAILED_MODULES=""

for status_file in "$RESULTS_DIR"/*.status; do
  if [ "$(cat "$status_file")" -ne 0 ]; then
    FAILED_TESTS=$((FAILED_TESTS + 1))
    group_name=$(basename "$status_file" .status)
    FAILED_MODULES="$FAILED_MODULES $group_name"
  else
    PASSED_TESTS=$((PASSED_TESTS + 1))
  fi
  TOTAL_TESTS=$((TOTAL_TESTS + 1))
done

# 結果を表示
echo -e "\n${BLUE}テスト実行結果:${NC}"
echo -e "実行時間: ${DURATION}秒"
echo -e "合計グループ数: $TOTAL_TESTS"
echo -e "成功: $PASSED_TESTS"
echo -e "失敗: $FAILED_TESTS"

if [ "$FAILED_TESTS" -gt 0 ]; then
  echo -e "\n${RED}失敗したグループ:${NC}"
  for group in $FAILED_MODULES; do
    echo -e "${RED}グループ $group のログ:${NC}"
    cat "$RESULTS_DIR/$group.log"
    echo -e "\n"
  done
  exit 1
else
  echo -e "\n${GREEN}すべてのテストが成功しました！${NC}"
fi