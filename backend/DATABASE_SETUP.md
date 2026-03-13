# PostgreSQL 数据库设置完成 ✅

## 📊 数据库信息

- **主机**: localhost:5432
- **数据库名**: rustset
- **用户**: rustset
- **密码**: rustset_password
- **类型**: PostgreSQL 16

## 📋 已创建的表 (23个)

### 核心表
- `users` - 用户表
- `assets` - 资产表
- `tasks` - 任务表
- `risks` - 风险表
- `audit_logs` - 审计日志表
- `network_zones` - 网络区域表
- `custom_roles` - 自定义角色表

### 云服务表
- `cloud_zones` - 云区域表
- `cloud_platforms` - 云平台表
- `cloud_services` - 云服务表
- `cloud_provider_configs` - 云厂商配置表
- `cloud_virtual_machines` - 云虚拟机表
- `cloud_platform_configs` - 云平台配置表

### 基础设施表
- `physical_machines` - 物理机表
- `machine_rooms` - 机房表
- `service_providers` - 服务商表
- `security_products` - 安全产品表
- `regions` - 区域表

### 业务表
- `business_resources` - 业务资源表
- `resource_ticket` - 资源工单表

### 扫描表
- `advanced_scan_tasks` - 高级扫描任务表
- `quick_scan_results` - 快速扫描结果表

### 系统表
- `seaql_migrations` - 迁移历史表

## 🔧 使用方法

### 1. 连接数据库

```bash
# 使用 psql 命令行工具
psql -U rustset -d rustset

# 或使用完整连接字符串
psql "postgres://rustset:rustset_password@localhost:5432/rustset"
```

### 2. 启动后端应用

```bash
cd backend

# 方式 1: 使用 .env 文件
cargo run

# 方式 2: 直接指定环境变量
DATABASE_URL="postgres://rustset:rustset_password@localhost:5432/rustset" cargo run

# 方式 3: 后台运行
DATABASE_URL="postgres://rustset:rustset_password@localhost:5432/rustset" cargo run &
```

### 3. 常用数据库命令

```bash
# 查看所有表
psql -U rustset -d rustset -c "\dt"

# 查看表结构
psql -U rustset -d rustset -c "\d users"

# 查询用户
psql -U rustset -d rustset -c "SELECT * FROM users;"

# 查看迁移历史
psql -U rustset -d rustset -c "SELECT * FROM seaql_migrations;"

# 统计表记录数
psql -U rustset -d rustset -c "SELECT COUNT(*) FROM users;"
```

## 🔐 默认用户账号

后端应用启动时会自动创建以下用户：

| 用户名 | 密码 | 角色 | 说明 |
|--------|------|------|------|
| admin | admin | SysAdmin | 系统管理员 |
| sec | sec | SecAdmin | 安全管理员 |
| audit | audit | Auditor | 审计员 |

⚠️ **安全提示**: 首次登录后请立即修改默认密码！

## 🚀 启动 PostgreSQL 服务

```bash
# 启动服务
brew services start postgresql@16

# 停止服务
brew services stop postgresql@16

# 重启服务
brew services restart postgresql@16

# 查看状态
brew services list
```

## 📝 配置文件

### .env 文件
```
DATABASE_URL=postgres://rustset:rustset_password@localhost:5432/rustset
```

### 其他数据库选项

```bash
# SQLite (开发环境)
DATABASE_URL=sqlite://data.db

# MySQL / MariaDB
DATABASE_URL=mysql://rustset:changeme@localhost:3306/rustset
```

## 🧪 测试数据库连接

```bash
# 运行测试脚本
./backend/test_db.sh
```

## 📚 SeaORM CLI 命令

```bash
# 安装 SeaORM CLI (已安装)
cargo install sea-orm-cli

# 生成新迁移 (需要正确的目录结构)
sea-orm-cli migrate generate create_new_table

# 运行迁移 (通过应用启动时自动执行)
cargo run
```

## 🔄 数据库迁移

 migrations 在应用启动时自动运行。迁移历史记录在 `seaql_migrations` 表中。

已运行的 migrations:
1. m20250101_000001_create_tables
2. m20250203_000001_split_resource_tables
3. m20250204_000001_add_provider_vendor
4. m20250204_000002_rename_cloud_platforms_to_cloud_services
5. m20250204_000003_add_regions
6. m20250301_000001_add_business_resource_fields
7. m20250306_000001_create_resource_tickets
8. m20250307_000001_add_security_products
9. m20250308_000001_create_infrastructure_tables

## 💡 开发提示

1. **开发环境**: 使用 SQLite 更简单（无需额外服务）
2. **生产环境**: 推荐使用 PostgreSQL（更好的性能和并发）
3. **备份数据库**:
   ```bash
   pg_dump -U rustset rustset > backup.sql
   ```
4. **恢复数据库**:
   ```bash
   psql -U rustset rustset < backup.sql
   ```

## 🎉 完成！

你的 PostgreSQL 数据库已经设置完成，可以开始使用了！
