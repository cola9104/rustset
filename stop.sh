#!/bin/bash

# 停止服务脚本 - rustset项目

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🛑 停止 Rustset 服务...${NC}"
echo ""

# 停止后端
if pgrep -f "target/debug/backend" >/dev/null; then
    BACKEND_PID=$(cat /tmp/rustset-backend.pid 2>/dev/null || echo "unknown")
    echo -e "${YELLOW}停止后端 (PID: $BACKEND_PID)...${NC}"
    pkill -f "target/debug/backend"
    echo -e "${GREEN}✓ 后端已停止${NC}"
else
    echo -e "${YELLOW}⚠️  后端未运行${NC}"
fi

# 停止前端
if pgrep -f "python3 -m http.server.*8080" >/dev/null; then
    FRONTEND_PID=$(cat /tmp/rustset-frontend.pid 2>/dev/null || echo "unknown")
    echo -e "${YELLOW}停止前端 (PID: $FRONTEND_PID)...${NC}"
    pkill -f "python3 -m http.server.*8080"
    echo -e "${GREEN}✓ 前端已停止${NC}"
else
    echo -e "${YELLOW}⚠️  前端未运行${NC}"
fi

# 清理PID文件
rm -f /tmp/rustset-backend.pid /tmp/rustset-frontend.pid

echo ""
echo -e "${GREEN}✓ 所有服务已停止${NC}"
