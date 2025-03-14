#!/bin/bash

# ShardX Mac用インストールスクリプト
# このスクリプトは、MacOS環境でShardXを簡単に起動するためのものです

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
echo -e "${GREEN}ShardX Mac用インストーラー${NC}"
echo "========================================"

# Dockerのチェック
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Dockerがインストールされていません。${NC}"
    echo -e "${YELLOW}Docker Desktopをインストールしてください: https://www.docker.com/products/docker-desktop${NC}"
    exit 1
fi

# Docker起動確認
if ! docker info &> /dev/null; then
    echo -e "${YELLOW}Dockerが起動していません。Docker Desktopを起動してください。${NC}"
    open -a Docker
    echo -e "${BLUE}Dockerの起動を待っています...${NC}"
    
    # Dockerが起動するまで待機
    for i in {1..30}; do
        if docker info &> /dev/null; then
            echo -e "${GREEN}Dockerが起動しました。${NC}"
            break
        fi
        
        if [ $i -eq 30 ]; then
            echo -e "${RED}Dockerの起動がタイムアウトしました。Docker Desktopを手動で起動してから再試行してください。${NC}"
            exit 1
        fi
        
        echo -n "."
        sleep 2
    done
fi

# 現在のディレクトリを確認
CURRENT_DIR=$(pwd)
echo -e "${BLUE}現在のディレクトリ: ${CURRENT_DIR}${NC}"

# データディレクトリの作成
mkdir -p data

# Docker Composeファイルの作成
echo -e "${BLUE}Docker Compose設定を作成中...${NC}"
cat > docker-compose.mac.yml << EOF
name: shardx-mac

services:
  # ShardXノード
  node:
    image: enablerdao/shardx:latest
    ports:
      - "54868:54868"
    environment:
      - PORT=54868
      - NODE_ID=mac_node
      - LOG_LEVEL=info
      - RUST_LOG=info
      - INITIAL_SHARDS=256
      - DATA_DIR=/app/data
    volumes:
      - ./data:/app/data
    restart: unless-stopped

  # Webインターフェース
  web:
    image: nginx:alpine
    ports:
      - "54867:80"
    volumes:
      - ./web/dist:/usr/share/nginx/html
      - ./web/nginx.conf:/etc/nginx/conf.d/default.conf
    depends_on:
      - node
    restart: unless-stopped
EOF

# Nginx設定ファイルの作成
mkdir -p web
echo -e "${BLUE}Nginx設定を作成中...${NC}"
cat > web/nginx.conf << EOF
server {
    listen 80;
    server_name localhost;
    
    # CORS設定
    add_header 'Access-Control-Allow-Origin' '*';
    add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS';
    add_header 'Access-Control-Allow-Headers' 'DNT,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range';
    
    # iframeでの埋め込みを許可
    add_header 'X-Frame-Options' 'ALLOWALL';
    
    # Webインターフェースのルート
    location / {
        root /usr/share/nginx/html;
        index index.html;
        try_files \$uri \$uri/ /index.html;
    }
    
    # APIプロキシ設定
    location /api/ {
        proxy_pass http://node:54868/;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
    
    # WebSocketサポート
    location /ws {
        proxy_pass http://node:54868/ws;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host \$host;
    }
}
EOF

# Webインターフェースのダウンロード
echo -e "${BLUE}Webインターフェースをダウンロード中...${NC}"
mkdir -p web/dist
curl -L -o web/dist.zip https://github.com/enablerdao/ShardX/releases/download/v0.1.0/web-dist.zip || {
    echo -e "${YELLOW}リリースからのダウンロードに失敗しました。デモHTMLを作成します...${NC}"
    
    # デモHTMLの作成
    cat > web/dist/index.html << EOF
<!DOCTYPE html>
<html>
<head>
    <title>ShardX - 高性能ブロックチェーンプラットフォーム</title>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        body {
            font-family: Arial, sans-serif;
            margin: 0;
            padding: 0;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            background-color: #f5f5f5;
        }
        .container {
            text-align: center;
            padding: 2rem;
            background-color: white;
            border-radius: 10px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
            max-width: 800px;
            width: 90%;
        }
        h1 {
            color: #3f51b5;
            margin-bottom: 0.5rem;
        }
        h2 {
            color: #666;
            font-weight: normal;
            margin-top: 0;
            margin-bottom: 2rem;
        }
        .status {
            margin: 2rem 0;
            padding: 1rem;
            background-color: #f0f0f0;
            border-radius: 5px;
            text-align: left;
        }
        .btn {
            display: inline-block;
            background-color: #3f51b5;
            color: white;
            padding: 10px 20px;
            border-radius: 5px;
            text-decoration: none;
            margin-top: 1rem;
            transition: background-color 0.3s;
        }
        .btn:hover {
            background-color: #303f9f;
        }
        pre {
            background-color: #f0f0f0;
            padding: 1rem;
            border-radius: 5px;
            overflow-x: auto;
            text-align: left;
        }
        #status {
            font-family: monospace;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>ShardX</h1>
        <h2>高性能ブロックチェーンプラットフォーム</h2>
        
        <p>ShardXは高速処理、スケーラビリティ、セキュリティを兼ね備えた次世代ブロックチェーンプラットフォームです。</p>
        
        <div class="status">
            <h3>ノードステータス</h3>
            <pre id="status">ノード情報を取得中...</pre>
        </div>
        
        <button class="btn" onclick="checkStatus()">ステータス更新</button>
        <a href="/api/info" class="btn">API情報</a>
    </div>

    <script>
        // ノードステータスを取得する関数
        async function checkStatus() {
            const statusElement = document.getElementById('status');
            statusElement.textContent = 'ノード情報を取得中...';
            
            try {
                const response = await fetch('/api/info');
                const data = await response.json();
                statusElement.textContent = JSON.stringify(data, null, 2);
            } catch (error) {
                statusElement.textContent = 'エラー: ノード情報を取得できませんでした。\n' + error.message;
            }
        }
        
        // ページ読み込み時にステータスを取得
        window.addEventListener('load', checkStatus);
    </script>
</body>
</html>
EOF
}

# Docker Composeで起動
echo -e "${BLUE}Docker Composeで起動中...${NC}"
docker-compose -f docker-compose.mac.yml pull
docker-compose -f docker-compose.mac.yml up -d

# 起動確認
echo -e "${BLUE}ShardXの起動を確認中...${NC}"
sleep 5

# ヘルスチェック
HEALTH=$(docker-compose -f docker-compose.mac.yml ps | grep node | grep -o "Up")

if [ "$HEALTH" == "Up" ]; then
    echo -e "${GREEN}=======================================${NC}"
    echo -e "${GREEN}ShardXが正常に起動しました！${NC}"
    echo -e "${GREEN}=======================================${NC}"
    echo -e "${BLUE}アクセス方法:${NC}"
    echo "- Webインターフェース: http://localhost:54867"
    echo "- API: http://localhost:54868"
    echo ""
    echo -e "${YELLOW}ShardXを停止するには:${NC}"
    echo "docker-compose -f docker-compose.mac.yml down"
else
    echo -e "${RED}ShardXの起動に失敗しました。${NC}"
    echo -e "${YELLOW}ログを確認してください:${NC}"
    echo "docker-compose -f docker-compose.mac.yml logs"
fi