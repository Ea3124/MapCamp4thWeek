#!/bin/bash

# 이름 정의
IMAGE_NAME=blockchain-server
CONTAINER_NAME=blockchain_server
PORT=3000
DATA_DIR=./data

# 데이터 디렉토리 생성 (없으면)
mkdir -p "$DATA_DIR"

# 1. 이미지 빌드
echo "🔨 Building Docker image..."
docker build -f Dockerfile.server -t $IMAGE_NAME .

# 2. 기존 컨테이너 정리
echo "🧹 Removing existing container (if exists)..."
docker rm -f $CONTAINER_NAME 2>/dev/null || true

# 3. 컨테이너 실행
echo "🚀 Running Docker container..."
docker run -d \
  --name $CONTAINER_NAME \
  -p $PORT:3000 \
  -v "$(pwd)/data:/app/server/data" \
  $IMAGE_NAME

echo "✅ Server is running at http://localhost:$PORT"
