# ビルドステージ - 依存関係のインストール
FROM node:18-alpine AS builder

WORKDIR /app

# パッケージ依存関係をコピー
COPY package*.json ./

# 依存関係をインストール
RUN npm ci --no-fund --no-audit --loglevel=error

# ランタイムステージ - 軽量なイメージを使用
FROM node:18-alpine

# セキュリティ強化: rootユーザーではなくnodeユーザーを使用
USER node

WORKDIR /app

# 依存関係をビルドステージからコピー
COPY --from=builder --chown=node:node /app/node_modules /app/node_modules

# アプリケーションのソースをコピー
COPY --chown=node:node . .

# ポートを公開
EXPOSE ${PORT:-54867}

# ヘルスチェック設定
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD wget -q -O - http://localhost:${PORT:-54867}/ || exit 1

# 環境変数の設定
ENV NODE_ENV=production
ENV NODE_URL=http://node:54867
ENV PORT=10000

# アプリケーションを起動
CMD ["node", "server.js"]