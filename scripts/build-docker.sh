#!/bin/bash
set -e

# ShardX Dockerイメージビルドスクリプト
# マルチアーキテクチャ（AMD64/ARM64）対応

# 色の定義
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# デフォルト設定
DOCKER_USERNAME=${DOCKER_USERNAME:-"yukih47"}
IMAGE_NAME="shardx"
TAG="latest"
PUSH=false
PLATFORMS="linux/amd64,linux/arm64"

# 引数の解析
while [[ $# -gt 0 ]]; do
  case $1 in
    --tag|-t)
      TAG="$2"
      shift 2
      ;;
    --push|-p)
      PUSH=true
      shift
      ;;
    --username|-u)
      DOCKER_USERNAME="$2"
      shift 2
      ;;
    --platforms)
      PLATFORMS="$2"
      shift 2
      ;;
    --help|-h)
      echo "使用方法: $0 [オプション]"
      echo "オプション:"
      echo "  --tag, -t TAG       イメージのタグ (デフォルト: latest)"
      echo "  --push, -p          ビルド後にDockerHubにプッシュ"
      echo "  --username, -u USER DockerHubのユーザー名 (デフォルト: yukih47)"
      echo "  --platforms PLAT    ビルドするプラットフォーム (デフォルト: linux/amd64,linux/arm64)"
      echo "  --help, -h          このヘルプを表示"
      exit 0
      ;;
    *)
      echo "不明なオプション: $1"
      exit 1
      ;;
  esac
done

echo -e "${BLUE}=== ShardX Dockerイメージビルド ===${NC}"
echo -e "ユーザー名: ${GREEN}${DOCKER_USERNAME}${NC}"
echo -e "イメージ名: ${GREEN}${IMAGE_NAME}${NC}"
echo -e "タグ: ${GREEN}${TAG}${NC}"
echo -e "プラットフォーム: ${GREEN}${PLATFORMS}${NC}"
echo -e "プッシュ: ${GREEN}${PUSH}${NC}"
echo

# BuildKitを有効化
export DOCKER_BUILDKIT=1

# マルチアーキテクチャビルダーを作成
echo -e "${BLUE}マルチアーキテクチャビルダーを設定中...${NC}"
docker buildx create --name multiarch --use || docker buildx use multiarch

# ビルド
echo -e "${BLUE}Dockerイメージをビルド中...${NC}"
if [ "$PUSH" = true ]; then
  echo -e "${BLUE}DockerHubにログイン...${NC}"
  docker login
  
  # 各アーキテクチャ向けにビルドしてプッシュ
  echo -e "${BLUE}AMD64向けイメージをビルドしてプッシュ中...${NC}"
  docker buildx build \
    --platform linux/amd64 \
    -t ${DOCKER_USERNAME}/${IMAGE_NAME}:${TAG}-amd64 \
    --push \
    -f Dockerfile.simple .
  
  echo -e "${BLUE}ARM64向けイメージをビルドしてプッシュ中...${NC}"
  docker buildx build \
    --platform linux/arm64 \
    -t ${DOCKER_USERNAME}/${IMAGE_NAME}:${TAG}-arm64 \
    --push \
    -f Dockerfile.simple .
  
  # マニフェストリストを作成（改善版）
  echo -e "${BLUE}マニフェストリストを作成中...${NC}"
  
  # 実験的機能を有効化
  export DOCKER_CLI_EXPERIMENTAL=enabled
  
  # 既存のマニフェストを削除（存在する場合）
  docker manifest rm ${DOCKER_USERNAME}/${IMAGE_NAME}:${TAG} 2>/dev/null || true
  
  # マニフェストリストを作成
  docker manifest create --amend ${DOCKER_USERNAME}/${IMAGE_NAME}:${TAG} \
    ${DOCKER_USERNAME}/${IMAGE_NAME}:${TAG}-amd64 \
    ${DOCKER_USERNAME}/${IMAGE_NAME}:${TAG}-arm64
  
  # アーキテクチャ情報を注釈
  echo -e "${BLUE}アーキテクチャ情報を注釈中...${NC}"
  docker manifest annotate ${DOCKER_USERNAME}/${IMAGE_NAME}:${TAG} \
    ${DOCKER_USERNAME}/${IMAGE_NAME}:${TAG}-arm64 --arch arm64 --os linux --variant v8
  docker manifest annotate ${DOCKER_USERNAME}/${IMAGE_NAME}:${TAG} \
    ${DOCKER_USERNAME}/${IMAGE_NAME}:${TAG}-amd64 --arch amd64 --os linux
  
  # マニフェストリストを検査
  echo -e "${BLUE}マニフェストリストを検査中...${NC}"
  docker manifest inspect ${DOCKER_USERNAME}/${IMAGE_NAME}:${TAG}
  
  # マニフェストリストをプッシュ（3回まで再試行）
  echo -e "${BLUE}マニフェストリストをプッシュ中...${NC}"
  for i in {1..3}; do
    if docker manifest push --purge ${DOCKER_USERNAME}/${IMAGE_NAME}:${TAG}; then
      echo -e "${GREEN}マニフェストリストのプッシュに成功しました（試行 $i）${NC}"
      break
    else
      echo -e "${RED}マニフェストリストのプッシュに失敗しました（試行 $i）${NC}"
      if [ $i -eq 3 ]; then
        echo -e "${RED}マニフェストリストのプッシュに3回失敗しました。${NC}"
      else
        echo -e "${YELLOW}5秒後に再試行します...${NC}"
        sleep 5
      fi
    fi
  done
  
  # 検証のためにイメージをプル
  echo -e "${BLUE}検証のためにイメージをプル中...${NC}"
  docker pull --platform=linux/amd64 ${DOCKER_USERNAME}/${IMAGE_NAME}:${TAG} || echo -e "${YELLOW}AMD64イメージのプルに失敗しました${NC}"
  docker pull --platform=linux/arm64 ${DOCKER_USERNAME}/${IMAGE_NAME}:${TAG} || echo -e "${YELLOW}ARM64イメージのプルに失敗しました${NC}"
else
  echo -e "${BLUE}イメージをビルド中（プッシュなし）...${NC}"
  docker buildx build \
    --platform ${PLATFORMS} \
    -t ${DOCKER_USERNAME}/${IMAGE_NAME}:${TAG} \
    --load \
    -f Dockerfile.simple .
fi

echo -e "${GREEN}ビルド完了！${NC}"
if [ "$PUSH" = true ]; then
  echo -e "${GREEN}イメージがDockerHubにプッシュされました: ${DOCKER_USERNAME}/${IMAGE_NAME}:${TAG}${NC}"
  echo -e "以下のコマンドで実行できます:"
  echo -e "${BLUE}docker run -p 54867:54867 -p 54868:54868 ${DOCKER_USERNAME}/${IMAGE_NAME}:${TAG}${NC}"
else
  echo -e "${GREEN}イメージがローカルにビルドされました: ${DOCKER_USERNAME}/${IMAGE_NAME}:${TAG}${NC}"
  echo -e "以下のコマンドでプッシュできます:"
  echo -e "${BLUE}docker push ${DOCKER_USERNAME}/${IMAGE_NAME}:${TAG}${NC}"
  echo -e "または、以下のコマンドで実行できます:"
  echo -e "${BLUE}docker run -p 54867:54867 -p 54868:54868 ${DOCKER_USERNAME}/${IMAGE_NAME}:${TAG}${NC}"
fi