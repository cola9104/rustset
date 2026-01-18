# 快速启动脚本使用说明

## 脚本说明

项目提供了两个快捷脚本来管理服务的启动和停止:

- `start.sh` - 启动服务脚本
- `stop.sh` - 停止服务脚本

## 使用方法

### 1. 启动所有服务

```bash
./start.sh
# 或
./start.sh start
```

这将:
- 停止现有服务
- 启动后端服务 (端口 3003)
- 启动前端服务 (端口 8080)
- 显示服务状态

### 2. 停止所有服务

```bash
./stop.sh
```

### 3. 重启所有服务

```bash
./start.sh restart
```

### 4. 查看服务状态

```bash
./start.sh status
```

### 5. 仅启动后端

```bash
./start.sh backend
```

### 6. 仅启动前端

```bash
./start.sh frontend
```

### 7. 构建项目

```bash
./start.sh build
```

这将编译后端和构建前端,但不会启动服务。

## 服务信息

### 后端服务
- **端口**: 3003
- **日志**: `/tmp/rustset-backend.log`
- **健康检查**: http://localhost:3003/health

### 前端服务
- **端口**: 8080
- **日志**: `/tmp/rustset-frontend.log`
- **访问地址**: http://localhost:8080

## 查看日志

### 实时查看后端日志
```bash
tail -f /tmp/rustset-backend.log
```

### 实时查看前端日志
```bash
tail -f /tmp/rustset-frontend.log
```

## 故障排查

### 端口被占用
如果提示端口被占用,请先停止服务:
```bash
./stop.sh
```

### 服务启动失败
1. 检查日志文件查看错误信息
2. 确保所有依赖已安装
3. 尝试重新构建: `./start.sh build`

### 权限问题
如果提示权限不足,请确保脚本有执行权限:
```bash
chmod +x start.sh stop.sh
```

## 默认登录信息

- **用户名**: admin
- **密码**: admin123

首次登录后请及时修改密码!
