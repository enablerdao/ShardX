#!/bin/bash

# ShardX バックアップスクリプト
# このスクリプトは、ShardXのデータをバックアップします

set -e

# 設定
BACKUP_DIR="/var/backups/shardx"
DATA_DIR="/var/lib/shardx/data"
RETENTION_DAYS=7
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/shardx_backup_${DATE}.tar.gz"

# カラー設定
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ロゴ表示
echo -e "${BLUE}"
echo "  ____  _                    _ __   __"
echo " / ___|| |__   __ _ _ __ __| |\ \ / /"
echo " \___ \| '_ \ / _\` | '__/ _\` | \ V / "
echo "  ___) | | | | (_| | | | (_| |  | |  "
echo " |____/|_| |_|\__,_|_|  \__,_|  |_|  "
echo -e "${NC}"
echo -e "${GREEN}ShardX バックアップツール${NC}"
echo "========================================"

# 引数の解析
while [[ $# -gt 0 ]]; do
  case $1 in
    --data-dir=*)
      DATA_DIR="${1#*=}"
      shift
      ;;
    --backup-dir=*)
      BACKUP_DIR="${1#*=}"
      shift
      ;;
    --retention-days=*)
      RETENTION_DAYS="${1#*=}"
      shift
      ;;
    *)
      echo -e "${RED}Unknown parameter: $1${NC}"
      exit 1
      ;;
  esac
done

# バックアップディレクトリの作成
mkdir -p $BACKUP_DIR

# データディレクトリの確認
if [ ! -d "$DATA_DIR" ]; then
    echo -e "${RED}エラー: データディレクトリが存在しません: $DATA_DIR${NC}"
    exit 1
fi

# Dockerが実行中かどうかを確認
DOCKER_RUNNING=false
if command -v docker &> /dev/null; then
    if docker ps | grep -q shardx; then
        DOCKER_RUNNING=true
        echo -e "${YELLOW}Dockerコンテナが実行中です。一時停止します...${NC}"
        docker-compose -f $(dirname $(dirname $(realpath $0)))/docker-compose.yml pause || docker compose -f $(dirname $(dirname $(realpath $0)))/docker-compose.yml pause
    fi
fi

# バックアップの作成
echo -e "${BLUE}バックアップを作成中: $BACKUP_FILE${NC}"
tar -czf $BACKUP_FILE -C $(dirname $DATA_DIR) $(basename $DATA_DIR)

# Dockerコンテナの再開
if [ "$DOCKER_RUNNING" = true ]; then
    echo -e "${YELLOW}Dockerコンテナを再開します...${NC}"
    docker-compose -f $(dirname $(dirname $(realpath $0)))/docker-compose.yml unpause || docker compose -f $(dirname $(dirname $(realpath $0)))/docker-compose.yml unpause
fi

# 古いバックアップの削除
echo -e "${BLUE}古いバックアップを削除中...${NC}"
find $BACKUP_DIR -name "shardx_backup_*.tar.gz" -type f -mtime +$RETENTION_DAYS -delete

# バックアップサイズの計算
BACKUP_SIZE=$(du -h $BACKUP_FILE | cut -f1)

echo -e "${GREEN}=======================================${NC}"
echo -e "${GREEN}バックアップが完了しました！${NC}"
echo -e "${GREEN}=======================================${NC}"
echo ""
echo -e "${BLUE}バックアップファイル:${NC} $BACKUP_FILE"
echo -e "${BLUE}バックアップサイズ:${NC} $BACKUP_SIZE"
echo -e "${BLUE}保存期間:${NC} $RETENTION_DAYS 日"
echo ""
echo -e "${YELLOW}リストア方法:${NC}"
echo "tar -xzf $BACKUP_FILE -C /path/to/restore"