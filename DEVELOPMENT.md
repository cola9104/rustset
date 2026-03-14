# RustSet 开发指南

## 快速开始

### 环境要求
- Rust 1.75+
- PostgreSQL 15+
- Node.js 18+ (可选，用于前端开发)

### 安装依赖

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 Dioxus CLI
cargo install dioxus-cli

# 安装 cargo-watch (开发时热重载)
cargo install cargo-watch
```

### 数据库设置

```bash
# 创建数据库
createdb rustset

# 或使用脚本
psql -U postgres -f scripts/init-db.sql
```

### 运行项目

```bash
# 运行后端
make run
# 或
cargo run -p backend

# 运行前端
make frontend
# 或
cd frontend && dx serve --port 8080
```

### 开发模式

```bash
# 后端热重载
make dev

# 或
cargo watch -x "run -p backend"
```

## 项目结构

```
rustset/
├── backend/           # 后端代码
│   ├── src/
│   │   ├── handlers/  # API 处理器
│   │   ├── middleware/# 中间件
│   │   ├── entities/  # 数据库实体
│   │   └── main.rs    # 入口
│   └── tests/         # 测试
├── frontend/          # 前端代码
│   └── src/
│       └── components/
├── shared/            # 共享类型
└── scripts/           # 脚本
```

## API 测试

```bash
# 登录
curl -X POST http://localhost:3003/api/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin"}'

# 获取资产
curl http://localhost:3003/api/assets \
  -H "Authorization: admin"

# IP 查找
curl "http://localhost:3003/api/ip-zones/find?ip=192.168.1.100" \
  -H "Authorization: admin"
```

## 常用命令

```bash
# 构建发布版本
make build

# 运行测试
make test

# 代码格式化
make fmt

# 代码检查
make check

# Docker 部署
make docker-up
```

## 故障排查

### 端口被占用
```bash
lsof -ti:3003 | xargs kill -9
```

### 数据库连接失败
```bash
# 检查 PostgreSQL 是否运行
pg_isready

# 检查连接字符串
echo $DATABASE_URL
```

### 前端编译错误
```bash
# 清理并重新构建
cargo clean -p frontend
cargo build -p frontend
```
