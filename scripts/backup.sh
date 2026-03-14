#!/bin/bash
# RustSet 数据库备份脚本

set -e

# 配置
DB_NAME="rustset"
DB_USER="rustset"
BACKUP_DIR="./backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/rustset_${TIMESTAMP}.sql"

# 创建备份目录
mkdir -p $BACKUP_DIR

# 备份数据库
echo "Backing up database..."
pg_dump -U $DB_USER $DB_NAME > $BACKUP_FILE

# 压缩
gzip $BACKUP_FILE

echo "Backup created: ${BACKUP_FILE}.gz"

# 清理旧备份（保留最近7个）
ls -t ${BACKUP_DIR}/rustset_*.sql.gz | tail -n +8 | xargs rm -f 2>/dev/null || true

echo "Old backups cleaned"
