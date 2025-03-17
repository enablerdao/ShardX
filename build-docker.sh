#!/bin/bash
set -e

# ShardX Dockerイメージビルドスクリプト

# バージョン情報
VERSION=$(grep '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
DOCKER_TAG="latest"

# 引数の処理
while [[ $# -gt 0 ]]; do
  case $1 in
    --tag)
      DOCKER_TAG="$2"
      shift 2
      ;;
    --version)
      VERSION="$2"
      shift 2
      ;;
    --push)
      PUSH=true
      shift
      ;;
    --help)
      echo "使用方法: $0 [オプション]"
      echo "オプション:"
      echo "  --tag TAG       Dockerイメージのタグを指定 (デフォルト: latest)"
      echo "  --version VER   バージョン番号を指定 (デフォルト: Cargo.tomlから取得)"
      echo "  --push          ビルド後にDockerイメージをプッシュ"
      echo "  --help          このヘルプメッセージを表示"
      exit 0
      ;;
    *)
      echo "不明なオプション: $1"
      exit 1
      ;;
  esac
done

echo "ShardX Dockerイメージをビルドします (バージョン: $VERSION, タグ: $DOCKER_TAG)"

# Dockerイメージをビルド
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

echo "完了しました！"