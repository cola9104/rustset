# 多数据库配置说明

后端现在支持以下数据库类型：
- **SQLite** (默认，嵌入式数据库)
- **PostgreSQL** (生产环境推荐)
- **MySQL / MariaDB**

## 配置方式

### 方式一：使用 DATABASE_URL 环境变量 (推荐)

设置 `DATABASE_URL` 环境变量，程序会自动检测数据库类型：

```bash
# SQLite (默认)
export DATABASE_URL="sqlite://data.db"

# PostgreSQL
export DATABASE_URL="postgres://username:password@localhost:5432/database_name"

# MySQL
export DATABASE_URL="mysql://username:password@localhost:3306/database_name"

# MariaDB (与 MySQL 相同)
export DATABASE_URL="mariadb://username:password@localhost:3306/database_name"
```

### 方式二：使用独立的环境变量

```bash
# 设置数据库类型
export DB_TYPE=sqlite  # 可选: sqlite, postgres, mysql

# SQLite 配置
export DB_PATH=data.db

# PostgreSQL / MySQL 配置
export DB_HOST=localhost
export DB_PORT=5432      # PostgreSQL 默认 5432，MySQL 默认 3306
export DB_NAME=rustset
export DB_USER=postgres  # 或 root (MySQL)
export DB_PASSWORD=your_password
```

## Docker Compose 示例

### PostgreSQL

```yaml
services:
  backend:
    build: ./backend
    environment:
      - DATABASE_URL=postgres://postgres:password@db:5432/rustset
    depends_on:
      - db

  db:
    image: postgres:15
    environment:
      - POSTGRES_DB=rustset
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

### MySQL

```yaml
services:
  backend:
    build: ./backend
    environment:
      - DATABASE_URL=mysql://root:password@db:3306/rustset
    depends_on:
      - db

  db:
    image: mysql:8
    environment:
      - MYSQL_DATABASE=rustset
      - MYSQL_ROOT_PASSWORD=password
    volumes:
      - mysql_data:/var/lib/mysql

volumes:
  mysql_data:
```

## 数据库特性对比

| 特性 | SQLite | PostgreSQL | MySQL/MariaDB |
|------|--------|------------|---------------|
| 适用场景 | 开发/测试 | 生产环境 | 生产环境 |
| 并发性能 | 低 | 高 | 高 |
| 数据类型 | TEXT | TIMESTAMP | VARCHAR |
| 自增主键 | AUTOINCREMENT | SERIAL | AUTO_INCREMENT |

## 迁移数据

从 SQLite 迁移到 PostgreSQL/MySQL：

1. 启动目标数据库
2. 设置 `DATABASE_URL` 指向新数据库
3. 首次启动时，表会自动创建
4. 使用数据库迁移工具导出/导入数据：

```bash
# SQLite 导出
sqlite3 data.db .dump > dump.sql

# 导入到 PostgreSQL
psql -U postgres -d rustset < dump.sql
```

## 注意事项

- 首次运行时，表会自动创建
- SQLite 文件默认位于项目根目录的 `data.db`
- 生产环境建议使用 PostgreSQL 或 MySQL
- 确保 .env 文件不要提交到版本控制系统
