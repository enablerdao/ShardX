#!/bin/bash
set -e

# ShardX Dockerイメージビルドスクリプト

# バージョン情報
VERSION=$(grep '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
DOCKER_TAG="latest"
PLATFORM="linux/amd64,linux/arm64"
PUSH=false
MULTI_ARCH=false

# 引数の処理
while [[ $# -gt 0 ]]; do
  case $1 in
    --tag|-t)
      DOCKER_TAG="$2"
      shift 2
      ;;
    --version|-v)
      VERSION="$2"
      shift 2
      ;;
    --push|-p)
      PUSH=true
      shift
      ;;
    --platform)
      PLATFORM="$2"
      MULTI_ARCH=true
      shift 2
      ;;
    --multi-arch|-m)
      MULTI_ARCH=true
      shift
      ;;
    --help|-h)
      echo "使用方法: $0 [オプション]"
      echo "オプション:"
      echo "  --tag, -t TAG       Dockerイメージのタグを指定 (デフォルト: latest)"
      echo "  --version, -v VER   バージョン番号を指定 (デフォルト: Cargo.tomlから取得)"
      echo "  --push, -p          ビルド後にDockerイメージをプッシュ"
      echo "  --platform PLATFORM ビルドするプラットフォームを指定 (デフォルト: linux/amd64,linux/arm64)"
      echo "  --multi-arch, -m    マルチアーキテクチャビルドを有効化"
      echo "  --help, -h          このヘルプメッセージを表示"
      exit 0
      ;;
    *)
      echo "不明なオプション: $1"
      exit 1
      ;;
  esac
done

echo "ShardX Dockerイメージをビルドします (バージョン: $VERSION, タグ: $DOCKER_TAG)"

# Dockerがインストールされているか確認
if ! command -v docker &> /dev/null; then
  echo "エラー: Dockerがインストールされていません"
  exit 1
fi

# マルチアーキテクチャビルドの場合
if [ "$MULTI_ARCH" = true ]; then
  echo "マルチアーキテクチャビルドを実行します (プラットフォーム: $PLATFORM)"
  
  # Docker Buildxが有効か確認
  if ! docker buildx version &> /dev/null; then
    echo "Docker Buildxを設定中..."
    docker buildx create --name shardx-builder --use
  fi
  
  # ビルドコマンドの構築
  BUILD_CMD="docker buildx build --platform $PLATFORM -t enablerdao/shardx:$DOCKER_TAG"
  
  if [ "$PUSH" = true ]; then
    BUILD_CMD="$BUILD_CMD --push"
  else
    # ローカルにロードできるのは単一アーキテクチャのみ
    echo "注意: マルチアーキテクチャビルドをローカルに保存するには --push オプションが必要です"
    echo "単一アーキテクチャ ($(uname -m)) のイメージのみをロードします"
    BUILD_CMD="$BUILD_CMD --load"
  fi
  
  BUILD_CMD="$BUILD_CMD ."
  
  # ビルドの実行
  echo "実行コマンド: $BUILD_CMD"
  eval $BUILD_CMD
  
  # バージョンタグも追加
  if [ "$DOCKER_TAG" != "$VERSION" ] && [ "$PUSH" = true ]; then
    echo "バージョンタグ enablerdao/shardx:$VERSION を追加してプッシュします"
    docker buildx build --platform $PLATFORM -t enablerdao/shardx:$VERSION --push .
  fi
else
  # 通常のDockerビルド
  echo "通常のDockerビルドを実行します"
  docker build -t enablerdao/shardx:$DOCKER_TAG .
  
  # バージョンタグも追加
  if [ "$DOCKER_TAG" != "$VERSION" ]; then
    docker tag enablerdao/shardx:$DOCKER_TAG enablerdao/shardx:$VERSION
    echo "バージョンタグ enablerdao/shardx:$VERSION を追加しました"
  fi
  
  echo "ビルド完了: enablerdao/shardx:$DOCKER_TAG"
  
  # イメージをプッシュ
  if [ "$PUSH" = true ]; then
    echo "Dockerイメージをプッシュしています..."
    docker push enablerdao/shardx:$DOCKER_TAG
    
    if [ "$DOCKER_TAG" != "$VERSION" ]; then
      docker push enablerdao/shardx:$VERSION
    fi
    
    echo "プッシュ完了"
  fi
fi

echo "完了しました！"