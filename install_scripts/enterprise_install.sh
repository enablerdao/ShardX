#!/bin/bash

# ShardX エンタープライズインストールスクリプト
# このスクリプトは、ShardXを高可用性構成でインストールします

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
echo -e "${GREEN}ShardX エンタープライズインストーラー${NC}"
echo "========================================"

# 設定
NODE_COUNT=3
REDIS_CLUSTER=true
MONITORING=true
BACKUP=true
DATA_DIR="/opt/shardx/data"
LOG_DIR="/opt/shardx/logs"
CONFIG_DIR="/opt/shardx/config"

# 引数の解析
while [[ $# -gt 0 ]]; do
  case $1 in
    --node-count=*)
      NODE_COUNT="${1#*=}"
      shift
      ;;
    --redis-cluster=*)
      REDIS_CLUSTER="${1#*=}"
      shift
      ;;
    --monitoring=*)
      MONITORING="${1#*=}"
      shift
      ;;
    --backup=*)
      BACKUP="${1#*=}"
      shift
      ;;
    --data-dir=*)
      DATA_DIR="${1#*=}"
      shift
      ;;
    --log-dir=*)
      LOG_DIR="${1#*=}"
      shift
      ;;
    --config-dir=*)
      CONFIG_DIR="${1#*=}"
      shift
      ;;
    *)
      echo -e "${RED}Unknown parameter: $1${NC}"
      exit 1
      ;;
  esac
done

# 必要なディレクトリの作成
echo -e "${BLUE}必要なディレクトリを作成中...${NC}"
sudo mkdir -p $DATA_DIR $LOG_DIR $CONFIG_DIR
sudo chmod -R 755 $DATA_DIR $LOG_DIR $CONFIG_DIR

# 依存関係のインストール
echo -e "${BLUE}依存関係をインストール中...${NC}"
sudo apt-get update
sudo apt-get install -y apt-transport-https ca-certificates curl gnupg lsb-release git

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

# Docker Composeのインストール
if ! command -v docker-compose &> /dev/null; then
  echo -e "${BLUE}Docker Composeをインストール中...${NC}"
  sudo curl -L "https://github.com/docker/compose/releases/download/v2.18.1/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
  sudo chmod +x /usr/local/bin/docker-compose
  echo -e "${GREEN}✓ Docker Composeがインストールされました${NC}"
else
  echo -e "${GREEN}✓ Docker Composeはすでにインストールされています${NC}"
fi

# リポジトリのクローン
echo -e "${BLUE}ShardXリポジトリをクローン中...${NC}"
if [ -d "/opt/shardx/repo" ]; then
  echo -e "${YELLOW}リポジトリディレクトリが既に存在します。更新します...${NC}"
  cd /opt/shardx/repo
  git pull
else
  sudo mkdir -p /opt/shardx/repo
  sudo chown $USER:$USER /opt/shardx/repo
  git clone https://github.com/enablerdao/ShardX.git /opt/shardx/repo
  cd /opt/shardx/repo
fi

# 高可用性構成用のdocker-compose.ymlを作成
echo -e "${BLUE}高可用性構成用のdocker-compose.ymlを作成中...${NC}"
cat > /opt/shardx/repo/docker-compose.ha.yml << EOF
version: '3.8'

services:
EOF

# ノードの設定
for (( i=1; i<=$NODE_COUNT; i++ )); do
  cat >> /opt/shardx/repo/docker-compose.ha.yml << EOF
  node${i}:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "5486${i}:54868"
    environment:
      - PORT=54868
      - NODE_ID=node${i}
      - LOG_LEVEL=info
      - INITIAL_SHARDS=256
      - DATA_DIR=/app/data
      - CLUSTER_NODES=node1:54868,node2:54868,node3:54868
    volumes:
      - ${DATA_DIR}/node${i}:/app/data
      - ${LOG_DIR}/node${i}:/app/logs
    networks:
      - shardx-network
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:54868/info"]
      interval: 10s
      timeout: 5s
      retries: 3
      start_period: 5s
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G

EOF
done

# Webサーバーの設定
cat >> /opt/shardx/repo/docker-compose.ha.yml << EOF
  web:
    image: nginx:alpine
    ports:
      - "54867:80"
    volumes:
      - ./web/dist:/usr/share/nginx/html
      - ${CONFIG_DIR}/nginx.conf:/etc/nginx/conf.d/default.conf
    depends_on:
EOF

for (( i=1; i<=$NODE_COUNT; i++ )); do
  cat >> /opt/shardx/repo/docker-compose.ha.yml << EOF
      - node${i}
EOF
done

cat >> /opt/shardx/repo/docker-compose.ha.yml << EOF
    networks:
      - shardx-network
    restart: unless-stopped
    deploy:
      resources:
        limits:
          cpus: '1'
          memory: 1G

EOF

# Redisクラスターの設定（オプション）
if [ "$REDIS_CLUSTER" = "true" ]; then
  cat >> /opt/shardx/repo/docker-compose.ha.yml << EOF
  redis1:
    image: redis:alpine
    command: redis-server --port 6379 --cluster-enabled yes --cluster-config-file nodes.conf --cluster-node-timeout 5000
    ports:
      - "6379:6379"
    volumes:
      - ${DATA_DIR}/redis1:/data
    networks:
      - shardx-network
    restart: unless-stopped

  redis2:
    image: redis:alpine
    command: redis-server --port 6379 --cluster-enabled yes --cluster-config-file nodes.conf --cluster-node-timeout 5000
    ports:
      - "6380:6379"
    volumes:
      - ${DATA_DIR}/redis2:/data
    networks:
      - shardx-network
    restart: unless-stopped

  redis3:
    image: redis:alpine
    command: redis-server --port 6379 --cluster-enabled yes --cluster-config-file nodes.conf --cluster-node-timeout 5000
    ports:
      - "6381:6379"
    volumes:
      - ${DATA_DIR}/redis3:/data
    networks:
      - shardx-network
    restart: unless-stopped

  redis-init:
    image: redis:alpine
    depends_on:
      - redis1
      - redis2
      - redis3
    command: >
      sh -c "sleep 5 && redis-cli --cluster create redis1:6379 redis2:6379 redis3:6379 --cluster-replicas 0 --cluster-yes"
    networks:
      - shardx-network

EOF
fi

# モニタリングの設定（オプション）
if [ "$MONITORING" = "true" ]; then
  cat >> /opt/shardx/repo/docker-compose.ha.yml << EOF
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ${CONFIG_DIR}/prometheus.yml:/etc/prometheus/prometheus.yml
      - ${DATA_DIR}/prometheus:/prometheus
    networks:
      - shardx-network
    restart: unless-stopped

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    volumes:
      - ${DATA_DIR}/grafana:/var/lib/grafana
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    depends_on:
      - prometheus
    networks:
      - shardx-network
    restart: unless-stopped

EOF
fi

# バックアップの設定（オプション）
if [ "$BACKUP" = "true" ]; then
  cat >> /opt/shardx/repo/docker-compose.ha.yml << EOF
  backup:
    image: alpine:latest
    volumes:
      - ${DATA_DIR}:/data
      - ${CONFIG_DIR}/backup.sh:/backup.sh
    command: sh -c "chmod +x /backup.sh && crond -f"
    restart: unless-stopped
    networks:
      - shardx-network

EOF
fi

# ネットワークの設定
cat >> /opt/shardx/repo/docker-compose.ha.yml << EOF
networks:
  shardx-network:
    driver: bridge
EOF

# Nginxの設定ファイルを作成
echo -e "${BLUE}Nginxの設定ファイルを作成中...${NC}"
cat > ${CONFIG_DIR}/nginx.conf << EOF
server {
    listen 80;
    server_name localhost;

    location / {
        root /usr/share/nginx/html;
        index index.html;
        try_files \$uri \$uri/ /index.html;
    }

    location /api/ {
        proxy_pass http://backend;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}

upstream backend {
EOF

for (( i=1; i<=$NODE_COUNT; i++ )); do
  cat >> ${CONFIG_DIR}/nginx.conf << EOF
    server node${i}:54868;
EOF
done

cat >> ${CONFIG_DIR}/nginx.conf << EOF
}
EOF

# Prometheusの設定ファイルを作成（モニタリングが有効な場合）
if [ "$MONITORING" = "true" ]; then
  echo -e "${BLUE}Prometheusの設定ファイルを作成中...${NC}"
  cat > ${CONFIG_DIR}/prometheus.yml << EOF
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'shardx'
    static_configs:
EOF

  for (( i=1; i<=$NODE_COUNT; i++ )); do
    cat >> ${CONFIG_DIR}/prometheus.yml << EOF
      - targets: ['node${i}:54868']
        labels:
          instance: 'node${i}'
EOF
  done
fi

# バックアップスクリプトを作成（バックアップが有効な場合）
if [ "$BACKUP" = "true" ]; then
  echo -e "${BLUE}バックアップスクリプトを作成中...${NC}"
  cat > ${CONFIG_DIR}/backup.sh << EOF
#!/bin/sh

# バックアップディレクトリの作成
BACKUP_DIR="/data/backups/\$(date +%Y%m%d_%H%M%S)"
mkdir -p \$BACKUP_DIR

# データのバックアップ
cp -r /data/node* \$BACKUP_DIR/
cp -r /data/redis* \$BACKUP_DIR/

# 古いバックアップの削除（7日以上前のもの）
find /data/backups -type d -mtime +7 -exec rm -rf {} \;

# ログ出力
echo "\$(date): Backup completed to \$BACKUP_DIR" >> /data/backup.log
EOF

  # cronジョブの設定
  echo -e "${BLUE}cronジョブを設定中...${NC}"
  echo "0 2 * * * /backup.sh" > /tmp/crontab
  sudo crontab /tmp/crontab
  rm /tmp/crontab
fi

# システムサービスの作成
echo -e "${BLUE}システムサービスを作成中...${NC}"
cat > /tmp/shardx.service << EOF
[Unit]
Description=ShardX High Availability Service
After=docker.service
Requires=docker.service

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=/opt/shardx/repo
ExecStart=/usr/local/bin/docker-compose -f docker-compose.ha.yml up -d
ExecStop=/usr/local/bin/docker-compose -f docker-compose.ha.yml down
TimeoutStartSec=0

[Install]
WantedBy=multi-user.target
EOF

sudo mv /tmp/shardx.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable shardx.service

# サービスの起動
echo -e "${BLUE}ShardXサービスを起動中...${NC}"
sudo systemctl start shardx.service

echo -e "${GREEN}=======================================${NC}"
echo -e "${GREEN}ShardXのエンタープライズインストールが完了しました！${NC}"
echo -e "${GREEN}=======================================${NC}"
echo -e "${BLUE}アクセス方法:${NC}"
echo "- Webインターフェース: http://localhost:54867"
echo "- API: http://localhost:54861 (node1)"
for (( i=2; i<=$NODE_COUNT; i++ )); do
  echo "       http://localhost:5486${i} (node${i})"
done

if [ "$MONITORING" = "true" ]; then
  echo "- Prometheus: http://localhost:9090"
  echo "- Grafana: http://localhost:3000 (ユーザー名: admin, パスワード: admin)"
fi

echo ""
echo -e "${YELLOW}サービスの管理:${NC}"
echo "- 起動: sudo systemctl start shardx"
echo "- 停止: sudo systemctl stop shardx"
echo "- 再起動: sudo systemctl restart shardx"
echo "- ステータス確認: sudo systemctl status shardx"