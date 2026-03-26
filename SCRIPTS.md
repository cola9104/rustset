# 快速启动脚本使用说明

## 脚本说明

项目提供两个脚本:

- `start.sh`: 启动、重启、查看状态和构建
- `stop.sh`: 停止前后端服务

默认端口:

- 后端: `3003`
- 前端: `8080`

默认日志:

- 后端: `/tmp/rustset-backend.log`
- 前端: `/tmp/rustset-frontend.log`

## 使用方法

### 启动前后端

```bash
./start.sh
# 或
./start.sh start
```

脚本会:

- 启动后端 `cargo run -p backend`
- 启动前端 `dx serve --port 8080`
- 将前端 API 基址注入为 `http://127.0.0.1:3003/api`
- 等待健康检查通过后输出状态

### 仅启动后端

```bash
./start.sh backend
```

### 仅启动前端

```bash
./start.sh frontend
```

### 查看状态

```bash
./start.sh status
```

### 停止服务

```bash
./stop.sh
# 或
./start.sh stop
```

### 重启服务

```bash
./start.sh restart
```

### 构建发布包

```bash
./start.sh build
```

## 探活地址

- 后端健康检查: `http://127.0.0.1:3003/api/health`
- 前端首页: `http://127.0.0.1:8080`

## 查看日志

```bash
tail -f /tmp/rustset-backend.log
tail -f /tmp/rustset-frontend.log
```

## 环境变量

- 后端会优先读取当前 shell 环境，再读取项目根目录 `.env` 和 `backend/.env`
- `HOST` / `PORT`: 后端监听地址
- `FRONTEND_URL`: CORS 白名单，多个来源用逗号分隔
- `BOOTSTRAP_DEFAULT_USERS`: 空数据库时是否自动创建默认开发账号

## 默认账号

当 `BOOTSTRAP_DEFAULT_USERS=true` 且数据库为空时，会创建以下开发账号:

- `admin / admin`
- `sec / sec`
- `audit / audit`

首次登录后应立即修改密码。

## 故障排查

### 端口已被占用

先执行:

```bash
./stop.sh
```

### 服务未成功启动

按顺序检查:

1. `tail -f /tmp/rustset-backend.log`
2. `tail -f /tmp/rustset-frontend.log`
3. `./start.sh build`

### 缺少依赖

确保已经安装:

- Rust / Cargo
- `dioxus-cli`
- 数据库驱动对应的本地环境
