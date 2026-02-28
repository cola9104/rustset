# RustSet - 网络安全资产管理平台

RustSet 是一个全栈 Rust 应用，用于管理网络资产、执行端口扫描、监控安全合规，以及混合云多云资产管理。
**技术栈**: Dioxus（前端/WASM） + Axum（后端 API） + SeaORM（数据库） + Tokio（异步运行时） + Tonic（gRPC） + Tower（中间件）
## 🚀 快速启动

### 使用启动脚本（推荐）

```bash
# 启动所有服务
./start.sh

# 查看服务状态
./start.sh status

# 停止所有服务
./stop.sh
```

详细说明请查看 [脚本使用文档](SCRIPTS.md)

### 手动启动

```bash
# 后端
cargo run -p backend

# 前端（需要单独终端窗口）
cd frontend
dx serve
```

## 系统架构

- **frontend**: Dioxus (Rust + WASM) 前端应用
- **backend**: Axum (Rust) REST API 服务器
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

## 快速启动

### 1. 启动后端服务
```bash
cargo run -p backend
```
服务将在 `http://127.0.0.1:3003` 启动

默认登录账号:
- 用户名: `admin`
- 密码: `admin`

### 2. 启动前端服务
```bash
cd frontend
dx serve
```
应用将在浏览器中自动打开 `http://127.0.0.1:8080`

### 3. 生产环境构建
```bash
# 前端构建
cd frontend
dx build --release

# 后端构建
cargo build --release -p backend
```

## 技术架构

- **前端**: Dioxus 通过 `gloo-net` 调用后端 API，使用 `use_signal` 管理状态
- **后端**: Axum 处理 HTTP 请求，使用 SeaORM 进行数据库操作
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
