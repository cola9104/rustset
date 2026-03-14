#!/bin/bash
# RustSet 部署脚本

set -e

echo "=== RustSet 部署脚本 ==="

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 检查命令是否存在
check_command() {
    if ! command -v $1 &> /dev/null; then
        echo -e "${RED}Error: $1 is not installed${NC}"
        exit 1
    fi
}

# 检查依赖
echo -e "${YELLOW}Checking dependencies...${NC}"
check_command cargo
check_command docker
check_command psql

# 运行测试
echo -e "${YELLOW}Running tests...${NC}"
cargo test --all
if [ $? -ne 0 ]; then
    echo -e "${RED}Tests failed!${NC}"
    exit 1
fi
echo -e "${GREEN}Tests passed!${NC}"

# 构建发布版本
echo -e "${YELLOW}Building release...${NC}"
cargo build --release
echo -e "${GREEN}Build complete!${NC}"

# 构建 Docker 镜像
echo -e "${YELLOW}Building Docker image...${NC}"
docker build -t rustset:latest .
echo -e "${GREEN}Docker image built!${NC}"

# 停止旧容器
echo -e "${YELLOW}Stopping old containers...${NC}"
docker-compose down 2>/dev/null || true

# 启动新容器
echo -e "${YELLOW}Starting new containers...${NC}"
docker-compose up -d

# 等待服务启动
echo -e "${YELLOW}Waiting for services to start...${NC}"
sleep 10

# 健康检查
echo -e "${YELLOW}Checking health...${NC}"
HEALTH=$(curl -s http://localhost:3003/api/health | grep -o '"status":"[^"]*"')
echo -e "${GREEN}Health: $HEALTH${NC}"

echo -e "${GREEN}Deployment complete!${NC}"
echo "API: http://localhost:3003"
echo "Metrics: http://localhost:3003/api/metrics"
