#!/bin/bash
# 数据库连接测试脚本

echo "🔍 测试 PostgreSQL 连接..."
echo ""

# 测试 1: 检查数据库连接
echo "1️⃣ 测试数据库连接..."
psql -U rustset -d rustset -c "SELECT version();" > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "   ✅ 数据库连接成功"
else
    echo "   ❌ 数据库连接失败"
    exit 1
fi

# 测试 2: 检查表是否创建
echo ""
echo "2️⃣ 检查表是否创建..."
TABLE_COUNT=$(psql -U rustset -d rustset -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';")
echo "   📊 已创建 $TABLE_COUNT 个表"

# 测试 3: 显示所有表
echo ""
echo "3️⃣ 数据库表列表："
psql -U rustset -d rustset -c "\dt" | grep -E "cloud_|users|assets|tasks"

# 测试 4: 检查 migrations
echo ""
echo "4️⃣ Migrations 历史："
MIGRATION_COUNT=$(psql -U rustset -d rustset -t -c "SELECT COUNT(*) FROM seaql_migrations;")
echo "   📝 已运行 $MIGRATION_COUNT 个 migrations"

# 测试 5: 测试插入数据
echo ""
echo "5️⃣ 测试插入数据..."
psql -U rustset -d rustset -c "INSERT INTO cloud_zones (zone_name, zone_code, description, created_at) VALUES ('测试区域', 'TEST001', '这是一个测试区域', datetime('now')) ON CONFLICT (zone_code) DO NOTHING;" > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "   ✅ 数据插入测试成功"
else
    echo "   ⚠️  数据插入测试失败（可能已存在）"
fi

echo ""
echo "🎉 数据库配置完成！"
echo ""
echo "📋 连接信息："
echo "   主机: localhost:5432"
echo "   数据库: rustset"
echo "   用户: rustset"
echo "   密码: rustset_password"
echo ""
echo "🔧 使用方法："
echo "   psql -U rustset -d rustset"
