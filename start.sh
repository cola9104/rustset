#!/bin/bash

# 快速启动脚本 - rustset项目
# 用于同时启动后端和前端服务

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 项目根目录
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_ROOT"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Rustset 项目快速启动脚本${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# 检查端口占用
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1 ; then
        echo -e "${YELLOW}⚠️  端口 $port 已被占用${NC}"
        return 1
    fi
    return 0
}

# 停止现有服务
stop_services() {
    echo -e "${YELLOW}🛑 停止现有服务...${NC}"
    pkill -f "target/debug/backend" || true
    pkill -f "python3 -m http.server.*8080" || true
    sleep 2
    echo -e "${GREEN}✓ 服务已停止${NC}"
    echo ""
}

# 启动后端
start_backend() {
    echo -e "${BLUE}🚀 启动后端服务...${NC}"
    cd "$PROJECT_ROOT"

    # 编译后端（如果需要）
    if [ ! -f "target/debug/backend" ] || [ "backend/src/main.rs" -nt "target/debug/backend" ]; then
        echo -e "${YELLOW}📦 编译后端...${NC}"
        cargo build --bin backend
    fi

    # 启动后端 (使用 PostgreSQL)
    DATABASE_URL="postgres://rustset:rustset123@localhost:5432/rustset" nohup ./target/debug/backend > /tmp/rustset-backend.log 2>&1 &
    BACKEND_PID=$!
    echo $BACKEND_PID > /tmp/rustset-backend.pid

    # 等待后端启动
    echo -e "${YELLOW}⏳ 等待后端启动...${NC}"
    for i in {1..30}; do
        if curl -s http://localhost:3003/health >/dev/null 2>&1; then
            echo -e "${GREEN}✓ 后端启动成功 (PID: $BACKEND_PID)${NC}"
            echo -e "${GREEN}  地址: http://localhost:3003${NC}"
            break
        fi
        sleep 1
        echo -n "."
    done
    echo ""
}

# 启动前端
start_frontend() {
    echo -e "${BLUE}🎨 启动前端服务...${NC}"
    cd "$PROJECT_ROOT/frontend"

    # 检查是否需要构建
    if [ ! -d "dist" ] || [ "src/lib.rs" -nt "dist/index.html" ]; then
        echo -e "${YELLOW}📦 构建前端...${NC}"
        trunk build --public-url /
    fi

    # 启动前端服务
    cd "$PROJECT_ROOT"
    nohup python3 -m http.server 8080 --directory /home/cola/tools/rustset/frontend/dist > /tmp/rustset-frontend.log 2>&1 &
    FRONTEND_PID=$!
    echo $FRONTEND_PID > /tmp/rustset-frontend.pid

    # 等待前端启动
    echo -e "${YELLOW}⏳ 等待前端启动...${NC}"
    sleep 3
    if curl -s http://localhost:8080 >/dev/null 2>&1; then
        echo -e "${GREEN}✓ 前端启动成功 (PID: $FRONTEND_PID)${NC}"
        echo -e "${GREEN}  地址: http://localhost:8080${NC}"
    else
        echo -e "${RED}✗ 前端启动失败${NC}"
        cat /tmp/rustset-frontend.log
    fi
    echo ""
}

# 显示服务状态
show_status() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  服务状态${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""

    # 后端状态
    if pgrep -f "target/debug/backend" >/dev/null; then
        BACKEND_PID=$(cat /tmp/rustset-backend.pid 2>/dev/null || echo "unknown")
        echo -e "${GREEN}✓ 后端运行中${NC} (PID: $BACKEND_PID)"
        echo -e "  地址: ${BLUE}http://localhost:3003${NC}"
    else
        echo -e "${RED}✗ 后端未运行${NC}"
    fi
    echo ""

    # 前端状态
    if pgrep -f "python3 -m http.server.*8080" >/dev/null; then
        FRONTEND_PID=$(cat /tmp/rustset-frontend.pid 2>/dev/null || echo "unknown")
        echo -e "${GREEN}✓ 前端运行中${NC} (PID: $FRONTEND_PID)"
        echo -e "  地址: ${BLUE}http://localhost:8080${NC}"
    else
        echo -e "${RED}✗ 前端未运行${NC}"
    fi
    echo ""

    echo -e "${YELLOW}💡 提示:${NC}"
    echo -e "  - 查看后端日志: ${BLUE}tail -f /tmp/rustset-backend.log${NC}"
    echo -e "  - 查看前端日志: ${BLUE}tail -f /tmp/rustset-frontend.log${NC}"
    echo -e "  - 停止服务: ${BLUE}./stop.sh${NC}"
    echo ""
}

# 主函数
main() {
    # 解析命令行参数
    case "${1:-start}" in
        start)
            stop_services
            start_backend
            start_frontend
            show_status
            ;;
        stop)
            stop_services
            echo -e "${GREEN}✓ 所有服务已停止${NC}"
            ;;
        restart)
            stop_services
            start_backend
            start_frontend
            show_status
            ;;
        status)
            show_status
            ;;
        backend)
            start_backend
            ;;
        frontend)
            start_frontend
            ;;
        build)
            echo -e "${YELLOW}📦 构建项目...${NC}"
            echo -e "${BLUE}编译后端...${NC}"
            cargo build --bin backend
            echo -e "${BLUE}构建前端...${NC}"
            cd frontend
            trunk build --public-url /
            echo -e "${GREEN}✓ 构建完成${NC}"
            ;;
        *)
            echo "用法: $0 {start|stop|restart|status|backend|frontend|build}"
            echo ""
            echo "命令说明:"
            echo "  start    - 启动所有服务（默认）"
            echo "  stop     - 停止所有服务"
            echo "  restart  - 重启所有服务"
            echo "  status   - 查看服务状态"
            echo "  backend  - 仅启动后端"
            echo "  frontend - 仅启动前端"
            echo "  build    - 构建项目"
            exit 1
            ;;
    esac
}

# 运行主函数
main "$@"
