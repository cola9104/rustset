#!/bin/bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_PORT="${BACKEND_PORT:-3003}"
FRONTEND_PORT="${FRONTEND_PORT:-8080}"
BACKEND_URL="http://127.0.0.1:${BACKEND_PORT}/api/health"
FRONTEND_URL="http://127.0.0.1:${FRONTEND_PORT}"
BACKEND_PID_FILE="/tmp/rustset-backend.pid"
FRONTEND_PID_FILE="/tmp/rustset-frontend.pid"
BACKEND_LOG="/tmp/rustset-backend.log"
FRONTEND_LOG="/tmp/rustset-frontend.log"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

pid_from_file() {
    local pid_file="$1"
    if [[ -f "$pid_file" ]]; then
        cat "$pid_file"
    fi
}

is_running() {
    local pid="$1"
    [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null
}

port_pid() {
    local port="$1"
    lsof -n -P -iTCP:"$port" -sTCP:LISTEN 2>/dev/null | awk 'NR==2 { print $2; exit }' || true
}

wait_for_http() {
    local url="$1"
    local name="$2"
    local pid_file="$3"
    local retries="${4:-30}"

    for _ in $(seq 1 "$retries"); do
        if curl -fsS "$url" >/dev/null 2>&1; then
            echo -e "${GREEN}✓ ${name} 已就绪${NC}"
            return 0
        fi

        local pid
        pid="$(pid_from_file "$pid_file")"
        if [[ -n "$pid" ]] && ! is_running "$pid"; then
            echo -e "${RED}✗ ${name} 进程已退出，请查看日志${NC}"
            return 1
        fi
        sleep 1
    done

    echo -e "${RED}✗ ${name} 启动超时，请查看日志${NC}"
    return 1
}

start_backend() {
    local pid
    pid="$(pid_from_file "$BACKEND_PID_FILE")"
    if is_running "$pid"; then
        echo -e "${YELLOW}后端已在运行 (PID: ${pid})${NC}"
        return 0
    fi

    local existing_pid
    existing_pid="$(port_pid "$BACKEND_PORT")"
    if [[ -n "$existing_pid" ]]; then
        echo -e "${YELLOW}后端端口 ${BACKEND_PORT} 已被占用 (PID: ${existing_pid})${NC}"
        return 0
    fi

    echo "启动后端..."
    nohup bash -lc "cd '$ROOT_DIR' && if [ -f backend/.env ]; then set -a; source backend/.env; set +a; fi; cargo run -p backend" \
        >"$BACKEND_LOG" 2>&1 &
    echo $! > "$BACKEND_PID_FILE"

    wait_for_http "$BACKEND_URL" "后端" "$BACKEND_PID_FILE" 45
}

start_frontend() {
    local pid
    pid="$(pid_from_file "$FRONTEND_PID_FILE")"
    if is_running "$pid"; then
        echo -e "${YELLOW}前端已在运行 (PID: ${pid})${NC}"
        return 0
    fi

    local existing_pid
    existing_pid="$(port_pid "$FRONTEND_PORT")"
    if [[ -n "$existing_pid" ]]; then
        echo -e "${YELLOW}前端端口 ${FRONTEND_PORT} 已被占用 (PID: ${existing_pid})${NC}"
        return 0
    fi

    echo "启动前端..."
    nohup bash -lc "cd '$ROOT_DIR/frontend' && VITE_API_BASE='http://127.0.0.1:${BACKEND_PORT}/api' dx serve --port ${FRONTEND_PORT}" \
        >"$FRONTEND_LOG" 2>&1 &
    echo $! > "$FRONTEND_PID_FILE"

    wait_for_http "$FRONTEND_URL" "前端" "$FRONTEND_PID_FILE" 45
}

show_status() {
    local backend_pid frontend_pid
    backend_pid="$(pid_from_file "$BACKEND_PID_FILE")"
    frontend_pid="$(pid_from_file "$FRONTEND_PID_FILE")"

    echo "RustSet 服务状态"

    if is_running "$backend_pid"; then
        echo -e "后端: ${GREEN}运行中${NC} PID=${backend_pid} URL=http://127.0.0.1:${BACKEND_PORT}"
    else
        local port_backend_pid
        port_backend_pid="$(port_pid "$BACKEND_PORT")"
        if [[ -n "$port_backend_pid" ]]; then
            echo -e "后端: ${YELLOW}端口占用${NC} PID=${port_backend_pid} URL=http://127.0.0.1:${BACKEND_PORT}"
        else
            echo -e "后端: ${RED}未运行${NC}"
        fi
    fi

    if is_running "$frontend_pid"; then
        echo -e "前端: ${GREEN}运行中${NC} PID=${frontend_pid} URL=http://127.0.0.1:${FRONTEND_PORT}"
    else
        local port_frontend_pid
        port_frontend_pid="$(port_pid "$FRONTEND_PORT")"
        if [[ -n "$port_frontend_pid" ]]; then
            echo -e "前端: ${YELLOW}端口占用${NC} PID=${port_frontend_pid} URL=http://127.0.0.1:${FRONTEND_PORT}"
        else
            echo -e "前端: ${RED}未运行${NC}"
        fi
    fi

    echo "后端日志: $BACKEND_LOG"
    echo "前端日志: $FRONTEND_LOG"
}

build_all() {
    echo "构建后端..."
    (cd "$ROOT_DIR" && cargo build --release -p backend)
    echo "构建前端..."
    (cd "$ROOT_DIR/frontend" && dx build --platform web --release)
}

case "${1:-start}" in
    start)
        start_backend
        start_frontend
        show_status
        ;;
    backend)
        start_backend
        show_status
        ;;
    frontend)
        start_frontend
        show_status
        ;;
    stop)
        "$ROOT_DIR/stop.sh"
        ;;
    restart)
        "$ROOT_DIR/stop.sh"
        start_backend
        start_frontend
        show_status
        ;;
    status)
        show_status
        ;;
    build)
        build_all
        ;;
    *)
        echo "用法: $0 {start|backend|frontend|stop|restart|status|build}"
        exit 1
        ;;
esac
