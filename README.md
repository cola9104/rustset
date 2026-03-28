

# RustSet - 网络安全资产管理平台

RustSet 是一个全栈 Rust 应用，用于管理网络资产、执行端口扫描、监控安全合规，以及混合云多云资产管理。

## 技术栈

| 组件 | 技术 |
|------|------|
| 前端 | Dioxus (Rust + WASM) |
| 后端 | Axum (Rust REST API) |
| 数据库 | SeaORM + PostgreSQL/SQLite |
| 缓存/Session | Redis |
| 异步运行时 | Tokio |
| gRPC | Tonic |

## 核心功能

### 资产管理
- 资产列表视图（名称、IP、网络区域）
- 手动添加、编辑和删除资产
- 网络区域分类（内网、DMZ、互联网）

### 混合云多云管理
- 支持阿里云、腾讯云、华为云等云厂商
- 云资产信息展示（规格、IP、计费模式、状态等）
- SSH 连接、实例启停操作

### 安全扫描
- 手动扫描与定期自动扫描
- 端口检测与安全告警

### 用户权限与审计
- 角色管理（安全管理员、审计员、普通用户）
- 审计日志记录

### 中英文切换
- 支持中文/英文界面

## 快速启动

### 使用启动脚本（推荐）

```bash
# 启动前后端
./start.sh start

# 查看状态
./start.sh status

# 停止
./stop.sh
```

### 手动启动

```bash
# 启动 PostgreSQL / Redis
docker compose up -d db redis

# 后端
cargo run -p backend

# 前端
cd frontend
VITE_API_BASE=http://127.0.0.1:3003/api dx serve --port 8080
```

### 环境要求

- Rust (cargo)
- Dioxus CLI: `cargo install dioxus-cli`
- PostgreSQL / Redis（可选）

复制 `.env.example` 为 `.env` 进行配置。

### 默认开发账号

当数据库为空且 `BOOTSTRAP_DEFAULT_USERS=true` 时自动创建：
- `admin / admin`
- `sec / sec`
- `audit / audit`

## 项目结构

```
rustset/
├── backend/          # Axum 后端服务
│   └── src/
│       ├── handlers/ # API 处理器
│       ├── entities/ # 数据库实体
│       └── main.rs   # 入口
├── frontend/         # Dioxus 前端
├── shared/           # 共享类型定义
└── Cargo.toml        # 工作空间配置
```

## 许可证

MIT License