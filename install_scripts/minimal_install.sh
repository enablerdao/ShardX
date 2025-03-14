#!/bin/bash

# ShardX ミニマルインストールスクリプト
# このスクリプトは、最小限のリソースでShardXをインストールします

set -e

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
echo -e "${GREEN}ShardX ミニマルインストーラー${NC}"
echo "========================================"

# 設定
DATA_DIR="/var/lib/shardx"
LOG_LEVEL="info"
INITIAL_SHARDS=64
DISABLE_WEB=false
DISABLE_METRICS=true

# 引数の解析
while [[ $# -gt 0 ]]; do
  case $1 in
    --data-dir=*)
      DATA_DIR="${1#*=}"
      shift
      ;;
    --log-level=*)
      LOG_LEVEL="${1#*=}"
      shift
      ;;
    --initial-shards=*)
      INITIAL_SHARDS="${1#*=}"
      shift
      ;;
    --disable-web=*)
      DISABLE_WEB="${1#*=}"
      shift
      ;;
    --disable-metrics=*)
      DISABLE_METRICS="${1#*=}"
      shift
      ;;
    *)
      echo -e "${RED}Unknown parameter: $1${NC}"
      exit 1
      ;;
  esac
done

# システム情報の確認
echo -e "${BLUE}システム情報を確認中...${NC}"
MEM_TOTAL=$(free -m | awk '/^Mem:/{print $2}')
CPU_COUNT=$(nproc)

echo -e "${GREEN}利用可能なメモリ:${NC} ${MEM_TOTAL}MB"
echo -e "${GREEN}利用可能なCPU:${NC} ${CPU_COUNT}コア"

# 最小要件のチェック
if [ $MEM_TOTAL -lt 512 ]; then
    echo -e "${YELLOW}警告: メモリが少なすぎます（最小512MB推奨）${NC}"
    echo -e "${YELLOW}シャード数を減らします...${NC}"
    INITIAL_SHARDS=32
fi

if [ $CPU_COUNT -lt 2 ]; then
    echo -e "${YELLOW}警告: CPUコア数が少なすぎます（最小2コア推奨）${NC}"
fi

# 必要なディレクトリの作成
echo -e "${BLUE}必要なディレクトリを作成中...${NC}"
sudo mkdir -p $DATA_DIR
sudo chmod 755 $DATA_DIR

# 依存関係のインストール
echo -e "${BLUE}最小限の依存関係をインストール中...${NC}"
sudo apt-get update
sudo apt-get install -y --no-install-recommends ca-certificates curl git

# Dockerのインストール
if ! command -v docker &> /dev/null; then
    echo -e "${BLUE}Dockerをインストール中...${NC}"
    curl -fsSL https://get.docker.com -o get-docker.sh
    sudo sh get-docker.sh
    sudo usermod -aG docker $USER
    echo -e "${GREEN}✓ Dockerがインストールされました${NC}"
else
    echo -e "${GREEN}✓ Dockerはすでにインストールされています${NC}"
fi

# リポジトリのクローン
echo -e "${BLUE}ShardXリポジトリをクローン中...${NC}"
if [ -d "/opt/shardx" ]; then
    echo -e "${YELLOW}ShardXディレクトリが既に存在します。更新します...${NC}"
    cd /opt/shardx
    sudo git pull
else
    sudo mkdir -p /opt/shardx
    sudo git clone https://github.com/enablerdao/ShardX.git /opt/shardx
    cd /opt/shardx
fi

# ミニマル構成用のdocker-compose.ymlを作成
echo -e "${BLUE}ミニマル構成用のdocker-compose.ymlを作成中...${NC}"
cat > /tmp/docker-compose.minimal.yml << EOF
version: '3'

services:
  node:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "54868:54868"
    environment:
      - PORT=54868
      - NODE_ID=minimal_node
      - LOG_LEVEL=${LOG_LEVEL}
      - INITIAL_SHARDS=${INITIAL_SHARDS}
      - DATA_DIR=/app/data
      - DISABLE_METRICS=${DISABLE_METRICS}
    volumes:
      - ${DATA_DIR}:/app/data
    restart: unless-stopped
    mem_limit: 512m
    cpus: 1
EOF

# Webインターフェースが無効でない場合は追加
if [ "$DISABLE_WEB" != "true" ]; then
    cat >> /tmp/docker-compose.minimal.yml << EOF
  web:
    image: nginx:alpine
    ports:
      - "54867:80"
    volumes:
      - ./web/dist:/usr/share/nginx/html
    depends_on:
      - node
    restart: unless-stopped
    mem_limit: 128m
    cpus: 0.5
EOF
fi

sudo mv /tmp/docker-compose.minimal.yml /opt/shardx/docker-compose.minimal.yml

# システムサービスの作成
echo -e "${BLUE}システムサービスを作成中...${NC}"
cat > /tmp/shardx-minimal.service << EOF
[Unit]
Description=ShardX Minimal Service
After=docker.service
Requires=docker.service

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=/opt/shardx
ExecStart=/usr/bin/docker-compose -f docker-compose.minimal.yml up -d
ExecStop=/usr/bin/docker-compose -f docker-compose.minimal.yml down
TimeoutStartSec=0

[Install]
WantedBy=multi-user.target
EOF

sudo mv /tmp/shardx-minimal.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable shardx-minimal.service

# サービスの起動
echo -e "${BLUE}ShardXサービスを起動中...${NC}"
sudo systemctl start shardx-minimal.service

echo -e "${GREEN}=======================================${NC}"
echo -e "${GREEN}ShardXのミニマルインストールが完了しました！${NC}"
echo -e "${GREEN}=======================================${NC}"
echo -e "${BLUE}アクセス方法:${NC}"
if [ "$DISABLE_WEB" != "true" ]; then
    echo "- Webインターフェース: http://localhost:54867"
fi
echo "- API: http://localhost:54868"
echo ""
echo -e "${YELLOW}サービスの管理:${NC}"
echo "- 起動: sudo systemctl start shardx-minimal"
echo "- 停止: sudo systemctl stop shardx-minimal"
echo "- 再起動: sudo systemctl restart shardx-minimal"
echo "- ステータス確認: sudo systemctl status shardx-minimal"
echo ""
echo -e "${YELLOW}注意:${NC} このインストールは最小限のリソースで動作するように最適化されています。"
echo "高負荷環境では、より多くのリソースを持つサーバーでの実行をお勧めします。"