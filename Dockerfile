# マルチステージビルド - ビルダーステージ
FROM rust:1.75-slim-bookworm as builder

WORKDIR /app

# ビルドに必要なパッケージをインストール
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    build-essential \
    git \
    && rm -rf /var/lib/apt/lists/*

# Cargoを最新バージョンに更新
RUN rustup update

# キャッシュ最適化のためにまずCargo.tomlとCargo.lockをコピー
COPY Cargo.toml Cargo.lock* ./

# Cargo.lockが存在しない場合は新しく生成
RUN if [ ! -f Cargo.lock ]; then touch Cargo.lock; fi

# 依存関係のみをビルドするためのダミーソースを作成
RUN mkdir -p src && \
    echo "fn main() {println!(\"Dummy build\");}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 実際のソースコードをコピー
COPY src ./src
COPY web ./web

# 最適化されたリリースビルドを実行
RUN RUSTFLAGS="-C target-cpu=native -C opt-level=3" cargo build --release

# バイナリを最適化
RUN strip target/release/shardx

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

# ビルダーステージからバイナリとウェブファイルをコピー
COPY --from=builder /app/target/release/shardx /app/shardx
COPY --from=builder /app/web /app/web

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