#!/bin/bash
set -e

# 色の定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}ShardX 依存関係更新ツール${NC}"
echo "=============================="

# 引数の解析
UPDATE_ALL=false
CHECK_ONLY=false
SECURITY_ONLY=false
SPECIFIC_PACKAGE=""

print_usage() {
  echo -e "使用方法: $0 [オプション]"
  echo -e "オプション:"
  echo -e "  --all\t\t全ての依存関係を更新"
  echo -e "  --check\t\t更新可能な依存関係を確認するだけで更新はしない"
  echo -e "  --security\t\tセキュリティ関連の更新のみを適用"
  echo -e "  --package <名前>\t特定のパッケージのみを更新"
  echo -e "  --help\t\tこのヘルプメッセージを表示"
}

while [[ $# -gt 0 ]]; do
  case $1 in
    --all)
      UPDATE_ALL=true
      shift
      ;;
    --check)
      CHECK_ONLY=true
      shift
      ;;
    --security)
      SECURITY_ONLY=true
      shift
      ;;
    --package)
      SPECIFIC_PACKAGE="$2"
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

# 必要なツールがインストールされているか確認
check_tool() {
  if ! command -v $1 &> /dev/null; then
    echo -e "${RED}エラー: $1 がインストールされていません${NC}"
    echo "インストール方法: $2"
    exit 1
  fi
}

check_tool cargo "curl https://sh.rustup.rs -sSf | sh"
check_tool npm "curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash - && sudo apt-get install -y nodejs"

# Rust の依存関係を更新
update_rust_dependencies() {
  echo -e "\n${BLUE}Rust の依存関係を確認中...${NC}"
  
  # cargo-edit がインストールされているか確認
  if ! cargo install --list | grep -q "cargo-edit"; then
    echo -e "${YELLOW}cargo-edit をインストールしています...${NC}"
    cargo install cargo-edit
  fi

  # cargo-audit がインストールされているか確認
  if ! cargo install --list | grep -q "cargo-audit"; then
    echo -e "${YELLOW}cargo-audit をインストールしています...${NC}"
    cargo install cargo-audit
  fi

  # セキュリティ監査を実行
  echo -e "\n${BLUE}セキュリティ監査を実行中...${NC}"
  AUDIT_RESULT=$(cargo audit --json || true)
  VULNERABILITIES=$(echo $AUDIT_RESULT | jq -r '.vulnerabilities.count')
  
  if [ "$VULNERABILITIES" -gt 0 ]; then
    echo -e "${RED}セキュリティの脆弱性が $VULNERABILITIES 件見つかりました${NC}"
    echo $AUDIT_RESULT | jq -r '.vulnerabilities.list[] | "- \(.package.name) \(.package.version): \(.advisory.title)"'
    
    if [ "$SECURITY_ONLY" = true ] || [ "$UPDATE_ALL" = true ]; then
      echo -e "\n${YELLOW}脆弱性のある依存関係を更新しています...${NC}"
      for pkg in $(echo $AUDIT_RESULT | jq -r '.vulnerabilities.list[].package.name'); do
        echo -e "${YELLOW}$pkg を更新しています...${NC}"
        cargo upgrade $pkg --incompatible
      done
    fi
  else
    echo -e "${GREEN}セキュリティの脆弱性は見つかりませんでした${NC}"
  fi

  if [ "$CHECK_ONLY" = true ]; then
    echo -e "\n${BLUE}更新可能な依存関係を確認中...${NC}"
    cargo outdated
    return
  fi

  if [ "$UPDATE_ALL" = true ]; then
    echo -e "\n${BLUE}すべての依存関係を更新中...${NC}"
    cargo upgrade --incompatible
  elif [ -n "$SPECIFIC_PACKAGE" ]; then
    echo -e "\n${BLUE}$SPECIFIC_PACKAGE を更新中...${NC}"
    cargo upgrade $SPECIFIC_PACKAGE --incompatible
  fi
}

# Node.js の依存関係を更新
update_node_dependencies() {
  # JavaScript プロジェクトのディレクトリを検索
  JS_DIRS=$(find . -name "package.json" -not -path "*/node_modules/*" -not -path "*/dist/*" -exec dirname {} \;)
  
  if [ -z "$JS_DIRS" ]; then
    echo -e "\n${YELLOW}JavaScript プロジェクトが見つかりませんでした${NC}"
    return
  fi

  echo -e "\n${BLUE}JavaScript の依存関係を確認中...${NC}"
  
  for dir in $JS_DIRS; do
    echo -e "\n${BLUE}$dir の依存関係を確認中...${NC}"
    
    cd $dir
    
    # npm-check-updates がインストールされているか確認
    if ! npm list -g npm-check-updates &> /dev/null; then
      echo -e "${YELLOW}npm-check-updates をインストールしています...${NC}"
      npm install -g npm-check-updates
    fi

    # セキュリティ監査を実行
    echo -e "\n${BLUE}セキュリティ監査を実行中...${NC}"
    npm audit --json > audit_result.json || true
    VULNERABILITIES=$(cat audit_result.json | jq -r '.metadata.vulnerabilities.total' 2>/dev/null || echo "0")
    
    if [ "$VULNERABILITIES" -gt 0 ]; then
      echo -e "${RED}セキュリティの脆弱性が $VULNERABILITIES 件見つかりました${NC}"
      cat audit_result.json | jq -r '.advisories | to_entries[] | .value | "- \(.module_name) \(.findings[0].version): \(.title)"' 2>/dev/null
      
      if [ "$SECURITY_ONLY" = true ] || [ "$UPDATE_ALL" = true ]; then
        echo -e "\n${YELLOW}脆弱性のある依存関係を更新しています...${NC}"
        npm audit fix --force
      fi
    else
      echo -e "${GREEN}セキュリティの脆弱性は見つかりませんでした${NC}"
    fi
    
    rm -f audit_result.json
    
    if [ "$CHECK_ONLY" = true ]; then
      echo -e "\n${BLUE}更新可能な依存関係を確認中...${NC}"
      ncu
    elif [ "$UPDATE_ALL" = true ]; then
      echo -e "\n${BLUE}すべての依存関係を更新中...${NC}"
      ncu -u
      npm install
    elif [ -n "$SPECIFIC_PACKAGE" ]; then
      echo -e "\n${BLUE}$SPECIFIC_PACKAGE を更新中...${NC}"
      ncu -u $SPECIFIC_PACKAGE
      npm install
    fi
    
    cd - > /dev/null
  done
}

# メイン処理
update_rust_dependencies
update_node_dependencies

echo -e "\n${GREEN}依存関係の確認・更新が完了しました${NC}"

if [ "$CHECK_ONLY" = true ]; then
  echo -e "${YELLOW}注意: 確認モードのため、実際の更新は行われていません${NC}"
fi

echo -e "${BLUE}更新後は 'cargo test' を実行して、すべてのテストが通ることを確認してください${NC}"