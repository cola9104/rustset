#!/bin/bash
# 快速启动脚本 - Rustset Backend with PostgreSQL

set -e

echo "🚀 启动 Rustset Backend..."
echo ""

# 检查 PostgreSQL 是否运行
if ! brew services list | grep postgresql@16 | grep -q "started"; then
    echo "📦 启动 PostgreSQL 服务..."
    brew services start postgresql@16
    sleep 2
fi

echo "✅ PostgreSQL 服务运行中"
echo ""

# 进入 backend 目录
cd backend

# 检查 .env 文件
if [ ! -f .env ]; then
    echo "⚠️  未找到 .env 文件，使用默认配置"
    export DATABASE_URL="postgres://rustset:rustset_password@localhost:5432/rustset"
else
    echo "✅ 找到 .env 配置文件"
    export DATABASE_URL=$(grep DATABASE_URL .env | cut -d'=' -f2)
fi

echo "🔗 数据库连接: ${DATABASE_URL:0:50}..."
echo ""

# 测试数据库连接
echo "🔍 测试数据库连接..."
if psql -U rustset -d rustset -c "SELECT 1;" > /dev/null 2>&1; then
    echo "✅ 数据库连接成功"
else
    echo "❌ 数据库连接失败，请检查配置"
    exit 1
fi

echo ""
echo "🎯 启动后端服务..."
echo "   访问地址: http://localhost:3003"
echo "   按 Ctrl+C 停止服务"
echo ""

# 启动后端
cargo run
