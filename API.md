# RustSet API 文档

## 基础信息

- **Base URL**: `http://localhost:3003/api`
- **认证方式**: Bearer Token (Header: `Authorization: <token>`)
- **内容类型**: `application/json`

## 认证 API

### 登录
```
POST /api/login
```

**请求体**:
```json
{
  "username": "admin",
  "password": "admin"
}
```

**响应**:
```json
{
  "token": "admin",
  "user": {
    "id": "uuid",
    "username": "admin",
    "role": "SysAdmin",
    "permissions": {...}
  }
}
```

### 登出
```
POST /api/logout
Authorization: <token>
```

### 刷新 Token
```
POST /api/refresh-token
Authorization: <token>
```

---

## 资产管理 API

### 获取资产列表
```
GET /api/assets
Authorization: <token>
```

### 创建资产
```
POST /api/assets
Authorization: <token>

{
  "name": "Server 1",
  "ip": "192.168.1.100",
  "zone": "Intranet",
  "ports": [],
  "weight": 80,
  "labels": []
}
```

### 更新资产
```
PUT /api/assets/{id}
Authorization: <token>
```

### 删除资产
```
DELETE /api/assets/{id}
Authorization: <token>
```

---

## IP 区域 API

### 获取所有区域
```
GET /api/ip-zones
Authorization: <token>
```

### 查找 IP 所属区域
```
GET /api/ip-zones/find?ip=192.168.1.100
Authorization: <token>
```

**响应**:
```json
{
  "ip": "192.168.1.100",
  "zone": "Intranet",
  "zone_type": "Intranet",
  "matched_cidr": "192.168.0.0/16"
}
```

### 创建区域
```
POST /api/ip-zones
Authorization: <token>

{
  "name": "CustomZone",
  "cidr": "172.16.0.0/12",
  "priority": 5
}
```

### 删除区域
```
DELETE /api/ip-zones/{id}
Authorization: <token>
```

---

## 端口详情 API

### 获取端口详情列表
```
GET /api/port-details
Authorization: <token>
```

### 创建端口详情
```
POST /api/port-details
Authorization: <token>

{
  "port": 8080,
  "protocol": "tcp"
}
```

**自动服务识别**:
- 端口 22 → SSH
- 端口 80 → HTTP
- 端口 443 → HTTPS
- 端口 3306 → MySQL
- 端口 5432 → PostgreSQL
- 端口 6379 → Redis
- 端口 27017 → MongoDB

### 批量绑定端口到资产
```
POST /api/port-details/batch-bind
Authorization: <token>

{
  "asset_id": 1,
  "ports": [
    {"port": 22, "protocol": "tcp"},
    {"port": 80, "protocol": "tcp"}
  ]
}
```

---

## 扫描器 API

### 单 IP 扫描
```
POST /api/scan-ip
Authorization: <token>

{
  "target": "192.168.1.1",
  "ports": [22, 80, 443]  // 可选，默认扫描常用端口
}
```

**响应**:
```json
{
  "id": "scan-uuid",
  "target": "192.168.1.1",
  "status": "completed",
  "start_time": "2026-03-14T...",
  "end_time": "2026-03-14T...",
  "ports": [
    {"port": 22, "is_open": false, "service": "SSH"},
    {"port": 80, "is_open": false, "service": "HTTP"}
  ]
}
```

### 批量扫描
```
POST /api/batch-scan-ips
Authorization: <token>

{
  "targets": ["192.168.1.1", "192.168.1.2"],
  "ports": [22, 80]
}
```

### 获取扫描结果
```
GET /api/scan-results
Authorization: <token>
```

查询参数:
- `target`: 过滤目标 IP
- `status`: 过滤状态 (pending, running, completed, failed)
- `limit`: 限制结果数量

---

## 用户管理 API

### 获取用户列表
```
GET /api/users
Authorization: <token>
```

**权限要求**: SysAdmin

### 创建用户
```
POST /api/users
Authorization: <token>

{
  "username": "newuser",
  "password": "password123",
  "role": "SecAdmin"
}
```

### 更新用户权限
```
PUT /api/users/{id}/permissions
Authorization: <token>
```

### 修改密码
```
PUT /api/users/{id}/password
Authorization: <token>

{
  "old_password": "old",
  "new_password": "new"
}
```

---

## 任务管理 API

### 获取任务列表
```
GET /api/tasks
Authorization: <token>
```

### 创建扫描任务
```
POST /api/tasks
Authorization: <token>

{
  "name": "Weekly Scan",
  "target": "192.168.1.0/24",
  "scan_type": "PortScan"
}
```

### 触发任务执行
```
POST /api/tasks/{id}/trigger
Authorization: <token>
```

---

## 风险管理 API

### 获取风险列表
```
GET /api/risks
Authorization: <token>
```

### 更新风险状态
```
PUT /api/risks/{id}/status
Authorization: <token>

{
  "status": "resolved"
}
```

---

## 审计日志 API

### 获取审计日志
```
GET /api/audit-logs
Authorization: <token>
```

查询参数:
- `user_id`: 过滤用户
- `action`: 过滤操作类型
- `start_time`: 开始时间
- `end_time`: 结束时间

---

## 服务商管理 API

### 获取服务商列表
```
GET /api/service-providers
Authorization: <token>
```

### 创建服务商
```
POST /api/service-providers
Authorization: <token>

{
  "provider_name": "阿里云",
  "provider_code": "aliyun",
  "short_name": "阿里云",
  "contact_person": "客服",
  "contact_phone": "400-xxx-xxxx",
  "contact_email": "support@aliyun.com",
  "headquarters": "杭州",
  "service_area": "全国",
  "business_license": "xxx"
}
```

---

## 错误响应

所有 API 在出错时返回统一格式:

```json
{
  "error": "error_code",
  "message": "Error description",
  "status": 400
}
```

常见错误码:
- `400` - 请求参数错误
- `401` - 未授权
- `403` - 权限不足
- `404` - 资源不存在
- `409` - 资源冲突
- `429` - 请求过于频繁
- `500` - 服务器内部错误

---

## Rate Limiting

默认限制: 60 请求/分钟

超过限制时返回:
```json
{
  "error": "rate_limit_exceeded",
  "message": "Rate limit exceeded. Try again in 30 seconds.",
  "status": 429
}
```

---

_更新时间: 2026-03-14_
