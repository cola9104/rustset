#!/bin/bash

set -euo pipefail

BACKEND_PORT="${BACKEND_PORT:-3003}"
FRONTEND_PORT="${FRONTEND_PORT:-8080}"
BACKEND_PID_FILE="/tmp/rustset-backend.pid"
FRONTEND_PID_FILE="/tmp/rustset-frontend.pid"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
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

stop_service() {
    local name="$1"
    local pid_file="$2"
    local port="$3"
    local pid

    pid="$(pid_from_file "$pid_file")"
    if is_running "$pid"; then
        echo -e "${YELLOW}停止${name} (PID: ${pid})...${NC}"
        kill "$pid"
        sleep 1
    else
        pid="$(port_pid "$port")"
        if [[ -n "$pid" ]]; then
            echo -e "${YELLOW}停止${name}端口占用进程 (PID: ${pid})...${NC}"
            kill "$pid"
            sleep 1
        else
            echo -e "${YELLOW}${name}未运行${NC}"
        fi
    fi

    rm -f "$pid_file"
}

echo -e "${YELLOW}停止 RustSet 服务...${NC}"

stop_service "后端" "$BACKEND_PID_FILE" "$BACKEND_PORT"
stop_service "前端" "$FRONTEND_PID_FILE" "$FRONTEND_PORT"

echo -e "${GREEN}✓ 已完成${NC}"
