# シンプルなビルドステージ
FROM rust:latest as builder

WORKDIR /app

# ビルドに必要なパッケージをインストール
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    build-essential \
    git \
    && rm -rf /var/lib/apt/lists/*

# Rustとcargoのバージョンを確認
RUN rustc --version && cargo --version

# すべてのソースコードをコピー
COPY . .

# Cargo.lockを削除して新しく生成
RUN rm -f Cargo.lock && cargo update

# まずデバッグビルドを試す
RUN cargo build -v

# リリースビルドを実行
RUN cargo build --release -v || (echo "Release build failed, checking errors" && cat target/release/build/*/*.log)

# ランタイムステージ - 超軽量なベースイメージを使用
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# 必要な依存関係のみをインストール
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# データディレクトリを作成
RUN mkdir -p /app/data /app/web && chmod 777 /app/data

# ビルダーステージからバイナリとウェブファイルをコピー（存在する場合）
COPY --from=builder /app/target/release/shardx* /app/ || true
COPY --from=builder /app/target/debug/shardx* /app/ || true
COPY --from=builder /app/web /app/web || true

# デバッグ用のダミーバイナリを作成（ビルドが失敗した場合）
RUN if [ ! -f /app/shardx ]; then \
    echo '#!/bin/sh' > /app/shardx && \
    echo 'echo "ShardX binary not available. Build process failed."' >> /app/shardx; \
    fi

# バイナリが実行可能であることを確認
RUN chmod +x /app/shardx

# 非rootユーザーを作成
RUN groupadd -r shardx && useradd -r -g shardx shardx
RUN chown -R shardx:shardx /app

# 非rootユーザーに切り替え
USER shardx

# APIポートを公開
EXPOSE ${PORT:-54868} ${P2P_PORT:-54867}

# ヘルスチェック設定
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:${PORT:-54868}/info || exit 1

# 環境変数の設定
ENV RUST_LOG=info
ENV DATA_DIR=/app/data
ENV WEB_DIR=/app/web
ENV PORT=54868
ENV P2P_PORT=54867
ENV NODE_ID=node1
ENV INITIAL_SHARDS=256
ENV RUST_BACKTRACE=1

# アプリケーションを実行
CMD ["/app/shardx"]

# 最終的な最適化イメージ
FROM runtime AS optimized

# メタデータを追加
LABEL org.opencontainers.image.title="ShardX"
LABEL org.opencontainers.image.description="高性能ブロックチェーンプラットフォーム"
LABEL org.opencontainers.image.vendor="EnablerDAO"
LABEL org.opencontainers.image.source="https://github.com/enablerdao/ShardX"
LABEL org.opencontainers.image.licenses="MIT"