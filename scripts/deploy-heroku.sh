#!/bin/bash
set -e

# ShardXをHerokuにデプロイするスクリプト

# 色の定義
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== ShardX Herokuデプロイスクリプト ===${NC}"
echo

# Heroku CLIがインストールされているか確認
if ! command -v heroku &> /dev/null; then
    echo -e "${RED}Heroku CLIがインストールされていません。${NC}"
    echo "インストール方法: https://devcenter.heroku.com/articles/heroku-cli"
    exit 1
fi

# ログイン状態を確認
if ! heroku auth:whoami &> /dev/null; then
    echo -e "${BLUE}Herokuにログインしてください...${NC}"
    heroku login
fi

# アプリ名を取得
read -p "Herokuアプリ名を入力してください (例: shardx-app): " APP_NAME

# アプリが存在するか確認
if heroku apps:info --app "$APP_NAME" &> /dev/null; then
    echo -e "${BLUE}既存のアプリ '$APP_NAME' を使用します${NC}"
else
    echo -e "${BLUE}新しいアプリ '$APP_NAME' を作成します...${NC}"
    heroku apps:create "$APP_NAME"
fi

# アドオンを追加
echo -e "${BLUE}必要なアドオンを追加しています...${NC}"
heroku addons:create heroku-postgresql:hobby-dev --app "$APP_NAME" || echo "PostgreSQLは既に追加されています"
heroku addons:create heroku-redis:hobby-dev --app "$APP_NAME" || echo "Redisは既に追加されています"

# 環境変数を設定
echo -e "${BLUE}環境変数を設定しています...${NC}"
heroku config:set NODE_ID="heroku_node_$(date +%s)" --app "$APP_NAME"
heroku config:set RUST_LOG=info --app "$APP_NAME"
heroku config:set INITIAL_SHARDS=32 --app "$APP_NAME"
heroku config:set DATA_DIR=/app/data --app "$APP_NAME"
heroku config:set REDIS_ENABLED=true --app "$APP_NAME"
heroku config:set WEB_ENABLED=true --app "$APP_NAME"

# スタックを設定
heroku stack:set heroku-22 --app "$APP_NAME"

# ビルドパックを設定
echo -e "${BLUE}ビルドパックを設定しています...${NC}"
heroku buildpacks:clear --app "$APP_NAME" || true
heroku buildpacks:add https://github.com/emk/heroku-buildpack-rust --app "$APP_NAME"
heroku buildpacks:add heroku/nodejs --app "$APP_NAME"

# Herokuリモートを追加
if ! git remote | grep -q heroku; then
    echo -e "${BLUE}Herokuリモートを追加しています...${NC}"
    heroku git:remote --app "$APP_NAME"
fi

# デプロイ
echo -e "${BLUE}Herokuにデプロイしています...${NC}"
echo "現在のブランチからデプロイします"
git push heroku HEAD:main

# スケーリング
echo -e "${BLUE}プロセスをスケーリングしています...${NC}"
heroku ps:scale web=1 worker=1 --app "$APP_NAME"

# 完了
echo -e "${GREEN}デプロイが完了しました！${NC}"
echo -e "アプリURL: ${BLUE}https://$APP_NAME.herokuapp.com${NC}"
echo -e "ダッシュボード: ${BLUE}https://dashboard.heroku.com/apps/$APP_NAME${NC}"
echo
echo -e "${BLUE}ログを確認するには:${NC} heroku logs --tail --app $APP_NAME"