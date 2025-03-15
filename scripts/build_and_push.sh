#!/bin/bash
set -e

# イメージタグ
TAG=${1:-latest}
REGISTRY=${2:-enablerdao}
IMAGE_NAME=${3:-shardx}

# ビルドとプッシュ
echo "Building Docker image: ${REGISTRY}/${IMAGE_NAME}:${TAG}"
docker build -t ${REGISTRY}/${IMAGE_NAME}:${TAG} .

echo "Pushing Docker image: ${REGISTRY}/${IMAGE_NAME}:${TAG}"
docker push ${REGISTRY}/${IMAGE_NAME}:${TAG}

echo "Image built and pushed successfully!"