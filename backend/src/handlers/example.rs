//! 示例 Handler - 演示如何使用统一的错误处理
//!
//! 这是一个示例文件，展示如何使用新的 ApiError 类型进行错误处理

use axum::{
    extract::{State, Path},
    response::Json,
};
use crate::state::AppState;
use crate::middleware::ApiError;
use shared::User;

/// 示例：获取用户信息（使用 ApiError）
///
/// 这个例子展示了如何使用 `Result<Json<T>, ApiError>`
pub async fn get_user_example(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<User>, ApiError> {
    // 查找用户
    let users = state.users.read().map_err(|e| {
        ApiError::internal(format!("Failed to acquire lock: {}", e))
    })?;

    let user = users
        .iter()
        .find(|u| u.username == id)
        .ok_or_else(|| ApiError::not_found(format!("User '{}' not found", id)))?;

    Ok(Json(user.clone()))
}

/// 示例：创建用户（使用 ApiError 的便捷构造函数）
pub async fn create_user_example(
    State(state): State<AppState>,
    Json(mut new_user): Json<User>,
) -> Result<Json<User>, ApiError> {
    // 验证用户名
    if new_user.username.is_empty() {
        return Err(ApiError::validation("Username cannot be empty"));
    }

    // 检查用户名长度
    if new_user.username.len() < 3 {
        return Err(ApiError::bad_request(
            "Username must be at least 3 characters long",
        ));
    }

    // 检查用户是否已存在
    {
        let users = state.users.read().map_err(|e| {
            ApiError::internal(format!("Failed to acquire lock: {}", e))
        })?;

        if users.iter().any(|u| u.username == new_user.username) {
            return Err(ApiError::conflict(format!(
                "User '{}' already exists",
                new_user.username
            )));
        }
    }

    // 创建用户（示例代码，实际逻辑会更复杂）
    new_user.id = Some(users_count(&state) + 1);

    Ok(Json(new_user))
}

/// 示例：处理数据库错误
pub async fn database_operation_example(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // 这里展示如何将数据库错误转换为 ApiError
    // 注意：需要实现 IntoApiError trait 或手动转换

    // 示例：查询数据库
    // let result = crate::database::get_assets()
    //     .await
    //     .map_err(|e| ApiError::database(format!("Failed to query assets: {}", e)))?;

    Ok(Json(serde_json::json!({
        "message": "Database operation successful",
        // "data": result,
    })))
}

/// 辅助函数：获取用户数量
fn users_count(state: &AppState) -> i32 {
    state.users.read().unwrap().len() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_construction() {
        let err = ApiError::not_found("User not found");
        assert_eq!(err.error_type(), "not_found");
        assert_eq!(err.status_code(), 404);
    }

    #[test]
    fn test_api_error_message() {
        let err = ApiError::bad_request("Invalid input");
        assert!(format!("{}", err).contains("Invalid input"));
    }
}
