# 统一错误处理使用指南

## 概述

RustSet 现在使用统一的错误处理系统（`backend/src/middleware/error_handler.rs`），提供一致的错误响应格式和更好的开发体验。

## 快速开始

### 1. 在 Handler 中使用 `ApiError`

```rust
use axum::{
    extract::{State, Path},
    response::Json,
};
use crate::middleware::ApiError;

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<User>, ApiError> {
    // 使用便捷构造函数
    let user = state.users.read()
        .map_err(|e| ApiError::internal(format!("Lock error: {}", e)))?
        .iter()
        .find(|u| u.id == Some(id))
        .ok_or_else(|| ApiError::not_found("User not found"))?;

    Ok(Json(user.clone()))
}
```

### 2. 错误响应格式

所有 API 错误现在返回统一的 JSON 格式：

```json
{
  "error": "not_found",
  "message": "User not found",
  "status": 404
}
```

### 3. 可用的错误类型

| 错误类型 | HTTP 状态码 | 构造函数 | 说明 |
|---------|------------|---------|------|
| Unauthorized | 401 | `ApiError::unauthorized(msg)` | 未授权，需要登录 |
| Forbidden | 403 | `ApiError::forbidden(msg)` | 禁止访问，权限不足 |
| NotFound | 404 | `ApiError::not_found(msg)` | 资源未找到 |
| BadRequest | 400 | `ApiError::bad_request(msg)` | 请求错误 |
| InternalError | 500 | `ApiError::internal(msg)` | 内部服务器错误 |
| Conflict | 409 | `ApiError::conflict(msg)` | 资源冲突（如重复创建） |
| PayloadTooLarge | 413 | `ApiError::payload_too_large(msg)` | 请求体过大 |
| UnsupportedMediaType | 415 | `ApiError::unsupported_media_type(msg)` | 不支持的媒体类型 |
| ServiceUnavailable | 503 | `ApiError::service_unavailable(msg)` | 服务不可用 |
| DatabaseError | 500 | `ApiError::database(msg)` | 数据库错误 |
| ValidationError | 400 | `ApiError::validation(msg)` | 验证错误 |

## 迁移指南

### 从旧错误处理迁移

#### 旧方式（不推荐）
```rust
pub async fn get_asset(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Asset>, (StatusCode, String)> {
    let assets = state.assets.read().unwrap();
    let asset = assets.iter()
        .find(|a| a.id == Some(id))
        .ok_or((StatusCode::NOT_FOUND, "Asset not found".to_string()))?;

    Ok(Json(asset.clone()))
}
```

#### 新方式（推荐）
```rust
pub async fn get_asset(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Asset>, ApiError> {
    let assets = state.assets.read()
        .map_err(|e| ApiError::internal(format!("Lock error: {}", e)))?;

    let asset = assets.iter()
        .find(|a| a.id == Some(id))
        .ok_or_else(|| ApiError::not_found("Asset not found"))?;

    Ok(Json(asset.clone()))
}
```

## 最佳实践

### 1. 提供有意义的错误消息

```rust
// ✅ 好
Err(ApiError::not_found("User with ID 123 not found"))

// ❌ 差
Err(ApiError::not_found("Not found"))
```

### 2. 使用适当的错误类型

```rust
// ✅ 好 - 使用具体的错误类型
Err(ApiError::validation("Password must be at least 8 characters"))

// ❌ 差 - 统一使用 bad_request
Err(ApiError::bad_request("Password too short"))
```

### 3. 记录错误日志

`ApiError` 会自动记录错误日志（使用 `tracing::error!`），所以不需要手动记录。

### 4. 处理数据库错误

```rust
use crate::middleware::IntoApiError;

pub async fn get_assets() -> Result<Json<Vec<Asset>>, ApiError> {
    let assets = crate::database::get_assets()
        .await
        .map_err(|e| e.into_api_error())?;

    Ok(Json(assets))
}
```

## 示例：完整的 Handler

```rust
use axum::{
    extract::{State, Path, Json},
    response::Json,
};
use crate::state::AppState;
use crate::middleware::ApiError;
use shared::User;

pub async fn create_user(
    State(state): State<AppState>,
    Json(mut user): Json<User>,
) -> Result<Json<User>, ApiError> {
    // 1. 验证输入
    if user.username.is_empty() {
        return Err(ApiError::validation("Username cannot be empty"));
    }

    if user.username.len() < 3 {
        return Err(ApiError::bad_request(
            "Username must be at least 3 characters long"
        ));
    }

    // 2. 检查冲突
    {
        let users = state.users.read()
            .map_err(|e| ApiError::internal(format!("Lock error: {}", e)))?;

        if users.iter().any(|u| u.username == user.username) {
            return Err(ApiError::conflict(format!(
                "User '{}' already exists",
                user.username
            )));
        }
    }

    // 3. 创建用户
    user.id = Some(users_count(&state) + 1);

    // 4. 返回结果
    Ok(Json(user))
}

fn users_count(state: &AppState) -> i32 {
    state.users.read().unwrap().len() as i32
}
```

## 测试

### 单元测试示例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_construction() {
        let err = ApiError::not_found("User not found");
        assert_eq!(err.error_type(), "not_found");
        assert_eq!(err.status_code(), 404);
    }

    #[test]
    fn test_error_display() {
        let err = ApiError::bad_request("Invalid input");
        assert!(format!("{}", err).contains("Invalid input"));
    }

    #[test]
    fn test_error_into_response() {
        let err = ApiError::unauthorized("Invalid token");
        let response = err.into_response();

        // response 可以进一步测试
    }
}
```

## 常见问题

### Q: 如何添加自定义错误类型？

A: 在 `ApiError` 枚举中添加新的变体：

```rust
pub enum ApiError {
    // ... 现有变体

    // 新增：自定义错误
    CustomError(String),
}
```

然后更新 `status_code()` 和 `error_type()` 方法。

### Q: 如何添加更多错误详情？

A: 使用 `ErrorResponse` 结构：

```rust
pub fn with_details(mut self, details: serde_json::Value) -> Self {
    self.details = Some(details);
    self
}
```

### Q: 如何在全局错误处理中间件中使用？

A: Axum 会自动处理 `ApiError`，因为它实现了 `IntoResponse` trait。不需要额外的中间件。

## 迁移检查清单

- [ ] 将所有 handler 的返回类型从 `Result<Json<T>, (StatusCode, String)>` 改为 `Result<Json<T>, ApiError>`
- [ ] 将错误消息从 `(StatusCode::XXX, "message".to_string())` 改为 `ApiError::xxx("message")`
- [ ] 更新错误消息，使其更具描述性
- [ ] 添加单元测试（如果尚未存在）
- [ ] 测试所有 API endpoint，确保错误响应正确

## 相关文件

- `backend/src/middleware/error_handler.rs` - 错误处理实现
- `backend/src/handlers/example.rs` - 使用示例
- `backend/docs/ERROR_HANDLING_GUIDE.md` - 本文档

---

_最后更新：2026-03-12_
