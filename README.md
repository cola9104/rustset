# RustSet - 网络安全资产管理平台

RustSet 是一个全栈 Rust 应用，用于管理网络资产、执行端口扫描、监控安全合规，以及混合云多云资产管理。
**技术栈**: Dioxus（前端/WASM） + Axum（后端 API） + SeaORM（数据库） + Redis（Session / 事件流） + Tokio（异步运行时） + Tonic（gRPC） + Tower（中间件）
## 🚀 快速启动

### 使用启动脚本（推荐）

```bash
# 启动前后端
./start.sh start

# 查看状态
./start.sh status

# 停止
./stop.sh
```

详细说明请查看 [脚本使用文档](SCRIPTS.md)

### 手动启动

```bash
# 可选：先启动 PostgreSQL / Redis
docker compose up -d db redis

# 后端
cargo run -p backend

# 前端（单独终端）
cd frontend
VITE_API_BASE=http://127.0.0.1:3003/api dx serve --port 8080
```

## 系统架构

- **frontend**: Dioxus (Rust + WASM) 前端应用
- **backend**: Axum (Rust) REST API 服务器
- **redis**: 分布式 Session、限流、热点缓存与业务事件流（Redis Stream）
- **shared**: 前后端共享的 Rust 类型定义

## 核心功能

### 资产管理
- 资产列表视图（名称、IP、网络区域）
- 手动添加资产
- 资产编辑和删除
- 网络区域分类（内网、DMZ、互联网）

### 混合云多云管理
- 支持多云厂商（阿里云、腾讯云、华为云等）
- 资产信息展示：
  - 资产名称、云厂商、区域
  - CPU/内存/系统盘规格
  - IP 地址、计费模式、到期时间
  - 虚拟机状态、操作系统
  - 部门、项目、负责人
  - 虚拟机创建时间、规格详情
  - 镜像 ID、云盘数据总量、云盘数量
  - 快照信息
- Excel 风格表格（支持列宽拖拽调整）
- 操作列固定显示
- 资产详情查看、云控制台跳转
- SSH 连接、实例重启/启动/停止/释放

### 安全扫描
- **手动扫描**: 对特定资产触发端口扫描
- **定期扫描**: 后台每 30 秒自动扫描资产
- **安全告警**:
  - 检测开放端口
  - 未绑定端口警告（标记为 ⚠️）

### 用户权限与审计
- 角色管理（安全管理员、审计员、普通用户）
- 审计日志记录
- 用户与 Gitee 账号绑定

### 中英文切换
- 支持中文/英文界面切换

## 环境要求

- Rust (cargo)
- `dioxus-cli` (前端构建工具): `cargo install dioxus-cli`
- 建议先复制 `.env.example` 或 `backend/.env.example` 为本地配置文件
- 后端会自动读取项目根目录 `.env` 和 `backend/.env`

## 快速启动

### 1. 启动后端服务
```bash
cargo run -p backend
```
服务将在 `http://127.0.0.1:3003` 启动

健康检查:
- `http://127.0.0.1:3003/api/health`

### 2. 启动前端服务
```bash
cd frontend
VITE_API_BASE=http://127.0.0.1:3003/api dx serve --port 8080
```
应用将在 `http://127.0.0.1:8080` 可访问

### 3. 生产环境构建
```bash
# 前端构建
cd frontend
dx build --platform web --release

# 后端构建
cargo build --release -p backend
```

### 4. 默认开发账号

当数据库为空且 `BOOTSTRAP_DEFAULT_USERS=true` 时，会自动创建:

- `admin / admin`
- `sec / sec`
- `audit / audit`

生产环境建议关闭该开关，并通过正式初始化流程创建账号。

### 5. Session / Cookie 行为

- Session Cookie 名称默认是 `rustset.sid`
- 默认 `HttpOnly`
- 默认 `SameSite=Lax`
- 默认空闲超时与 `JWT_EXPIRATION_HOURS` 对齐，默认 24 小时
- 可通过 `SESSION_COOKIE_*` 和 `SESSION_IDLE_TIMEOUT_HOURS` 覆盖
- 启用 Redis 后，Session 会从内存切换为 Redis 持久化存储

### 6. Redis 集成

- `REDIS_ENABLED=true` 时，后端会在启动时连接 Redis；连接失败会直接终止启动，避免看起来“已启用”但实际上没生效。
- `REDIS_SESSION_STORE_ENABLED=true` 时，登录 Session 使用 Redis 存储，适合多实例部署。
- `REDIS_EVENT_STREAM_ENABLED=true` 时，审计/业务动作会写入 Redis Stream，默认流名为 `rustset:events:audit`。
- `REDIS_RATE_LIMIT_ENABLED=true` 时，请求限流会切到 Redis 计数器，支持多实例共享限流状态。
- `REDIS_CACHE_ENABLED=true` 时，仪表盘汇总和审计日志列表会走 Redis 短 TTL 缓存。
- Redis 健康状态会出现在 `GET /api/health`、`GET /api/ready` 和 `GET /api/metrics` 里。

常用环境变量：

```bash
REDIS_ENABLED=true
REDIS_URL=redis://127.0.0.1:6379/0
REDIS_POOL_SIZE=8
REDIS_CONNECTION_TIMEOUT_MS=3000
REDIS_SESSION_STORE_ENABLED=true
REDIS_EVENT_STREAM_ENABLED=true
REDIS_EVENT_STREAM_NAME=rustset:events:audit
REDIS_EVENT_STREAM_MAX_LEN=10000
REDIS_RATE_LIMIT_ENABLED=true
REDIS_RATE_LIMIT_PREFIX=rustset:rate_limit
REDIS_CACHE_ENABLED=true
REDIS_DASHBOARD_CACHE_TTL_SECS=15
REDIS_AUDIT_LOGS_CACHE_TTL_SECS=10
```

## 技术架构

- **前端**: Dioxus 通过 `gloo-net` 调用后端 API，使用 `use_signal` 管理状态
- **后端**: Axum 处理 HTTP 请求，使用 SeaORM 进行数据库操作
- **Redis**: 提供分布式 Session、分布式限流、热点缓存和 Redis Stream 事件队列入口
- **扫描器**: Tokio 后台任务定期运行，模拟网络扫描

## 更新到 Gitee

### 提交代码到本地 Git
```bash
git add .
git commit -m "描述你的更改"
```

### 推送到 Gitee
```bash
git push origin main
```

### 如果需要添加 Gitee 远程仓库
```bash
git remote add origin https://gitee.com/你的用户名/rustset.git
```

### 快捷推送命令（推荐配置 Git alias）
```bash
# 在 ~/.gitconfig 中添加别名
git config --global alias.pg 'push origin main'

# 之后可以使用快捷命令
git pg
```

## 常用 Git 命令

```bash
# 查看状态
git status

# 查看远程仓库
git remote -v

# 查看当前分支
git branch

# 拉取 Gitee 更新
git pull origin main

# 强制推送（慎用）
git push origin main --force
```

## 项目结构

```
rustset/
├── backend/          # 后端服务
│   └── src/
│       ├── handlers/ # API 处理器
│       └── main.rs   # 入口文件
├── frontend/         # 前端应用
│   ├── src/
│   │   └── lib.rs    # Dioxus 组件
│   ├── index.html    # HTML 入口
│   └── Cargo.toml    # 前端依赖
├── shared/           # 共享类型
│   └── src/lib.rs    # Asset, CloudAsset 等
└── Cargo.toml        # 工作空间配置
```

## 许可证

MIT License
