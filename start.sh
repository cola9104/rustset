#!/bin/bash
cd "$(dirname "$0")"

echo "=== RustSet ==="

# Docker PostgreSQL is already at localhost:5432
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/rustset"
export WEB_PERMISSIVE_CORS="true"

# Start backend
echo ">> 启动后端 :8080"
cargo run -p rustset-gateway &
BACKEND_PID=$!
sleep 2

# Start Dioxus frontend if requested
if [ "$1" = "-f" ] || [ "$1" = "--with-frontend" ]; then
    echo ">> 启动前端 :3000"
    cd apps/web-dioxus && dx serve --port 3000 &
    FRONTEND_PID=$!
    echo ""
    echo "  后端 API : http://localhost:8080"
    echo "  前端 UI  : http://localhost:3000"
    wait $BACKEND_PID $FRONTEND_PID
else
    echo "  后端 API : http://localhost:8080"
    wait $BACKEND_PID
fi
