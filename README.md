# RustSet

RustSet 是 Rust 后端与 Vben Admin 5 前端组成的管理平台。基础后台按 Yudao 的模块化思想实现，动漫工厂按 Toonflow 的业务流程实现；后端统一使用 Rust，前端统一使用 Vue 3、Vben 和 Ant Design Vue，前端工程由 bun 管理。

## 功能组成

- System：认证、用户、角色、权限、菜单、租户及后台管理能力。
- Infra：配置、文件、任务、日志、数据源等基础设施能力。
- AI：统一模型管理、聊天/SSE、工具调用、知识库、图片、Midjourney、音乐、语音、Embedding 和写作。
- Toonflow：项目、小说、剧本、事件、资产、分镜、音频、视频、Agent、提示词、Skill 和任务中心。
- Media：素材上传及媒体基础能力。

## 环境要求

- Rust stable（项目使用 Rust 2024 edition）
- PostgreSQL 18
- bun `1.4+`（前端包管理器；node 22/24 可选，供部分工具链使用）
- Docker 及 Docker Compose（推荐用于本地基础设施）

## 五分钟本地启动

### 1. 启动基础设施

```bash
docker compose -f script/docker/docker-compose.yml up -d
```

启动 PostgreSQL、Redis、NATS 和 MinIO。容器只创建空数据库 `rustset`，
数据库结构统一由 Rust 网关的 SQLx Migrator 自动管理。

### 2. 启动 Rust 网关

```bash
export DATABASE_URL='postgres://rustset:rustset@127.0.0.1:5432/rustset'
export REDIS_URL='redis://127.0.0.1:6379'
export JWT_SECRET='replace-with-at-least-32-random-bytes'
cargo run -p rustset-gateway
```

网关启动时自动执行 SQLx 迁移。`0001_initial.sql` 是已合并的当前完整表结构
和基准数据，因此部署时不需要
`sql/bootstrap/current.sql`。`current.sql` 仅作为人工核对用的快照，不会被应用加载。

后续修改数据库时，必须从 `0002` 开始新增更高版本的迁移文件，并在干净数据库
完成全量迁移后重新导出 `current.sql` 参考快照。合并后的 `0001` 一旦发布就不能再修改。

可选环境变量：
- `DATABASE_MIN_CONNECTIONS`（默认 1）
- `DATABASE_MAX_CONNECTIONS`（默认 20）
- `DATABASE_ACQUIRE_TIMEOUT_SECONDS`（默认 5）
- `GATEWAY_HOST`（默认 `0.0.0.0`）
- `GATEWAY_PORT`（默认 `8080`）
- `RUST_LOG`（推荐 `info`）

### 3. 启动 Vben 前端

```bash
cd apps/web
bun install
bun run dev:antd
```

访问 `http://127.0.0.1:5666`，默认后端为 `http://127.0.0.1:8080`（开发代理将 `/api/*` 转发到网关并去掉前缀）。

基线种子账号：`admin` / `admin123`（来自迁移基线数据，正式部署后请立即修改密码）。

> 生产环境必须替换示例密码和 JWT 密钥。

## 验证

```bash
cargo test --workspace
bash script/test-database-migrations.sh
bash script/test-ai-e2e.sh
bun run --cwd apps/web check:type
bun run --cwd apps/web build:antd
```

## 文档

- [AI 启动交接指南](AGENTS.md)
- [技术架构](docs/technical-solution.md)
- [配置与模型接入](docs/configuration.md)
- [启动、部署与运维](docs/deployment.md)
- [功能范围与验收口径](docs/parity-roadmap.md)
