//! 统一错误处理中间件
//!
//! 提供统一的错误类型、错误响应格式和错误处理中间件
#![allow(dead_code)]

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;

/// 统一的 API 错误类型
#[derive(Debug, Clone)]
pub enum ApiError {
    /// 未授权 (401)
    Unauthorized(String),

    /// 禁止访问 (403)
    Forbidden(String),

    /// 资源未找到 (404)
    NotFound(String),

    /// 请求错误 (400)
    BadRequest(String),

    /// 内部服务器错误 (500)
    InternalError(String),

    /// 冲突 (409)
    Conflict(String),

    /// 请求体过大 (413)
    PayloadTooLarge(String),

    /// 不支持的媒体类型 (415)
    UnsupportedMediaType(String),

    /// 服务不可用 (503)
    ServiceUnavailable(String),

    /// 数据库错误
    DatabaseError(String),

    /// 验证错误
    ValidationError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ApiError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            ApiError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            ApiError::InternalError(msg) => write!(f, "Internal Error: {}", msg),
            ApiError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            ApiError::PayloadTooLarge(msg) => write!(f, "Payload Too Large: {}", msg),
            ApiError::UnsupportedMediaType(msg) => write!(f, "Unsupported Media Type: {}", msg),
            ApiError::ServiceUnavailable(msg) => write!(f, "Service Unavailable: {}", msg),
            ApiError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            ApiError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

impl ApiError {
    /// 获取对应的 HTTP 状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            ApiError::UnsupportedMediaType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ApiError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
        }
    }

    /// 获取错误类型字符串
    pub fn error_type(&self) -> &'static str {
        match self {
            ApiError::Unauthorized(_) => "unauthorized",
            ApiError::Forbidden(_) => "forbidden",
            ApiError::NotFound(_) => "not_found",
            ApiError::BadRequest(_) => "bad_request",
            ApiError::InternalError(_) => "internal_error",
            ApiError::Conflict(_) => "conflict",
            ApiError::PayloadTooLarge(_) => "payload_too_large",
            ApiError::UnsupportedMediaType(_) => "unsupported_media_type",
            ApiError::ServiceUnavailable(_) => "service_unavailable",
            ApiError::DatabaseError(_) => "database_error",
            ApiError::ValidationError(_) => "validation_error",
        }
    }
}

/// 实现 IntoResponse trait，自动转换为 HTTP 响应
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_type = self.error_type();
        let message = self.to_string();

        let body = json!({
            "error": error_type,
            "message": message,
            "status": status.as_u16(),
        });

        tracing::error!("API Error: {} - {}", error_type, message);

        (status, Json(body)).into_response()
    }
}

/// 错误响应的结构化格式
#[derive(serde::Serialize)]
pub struct ErrorResponse {
    /// 错误类型
    pub error: &'static str,

    /// 错误消息
    pub message: String,

    /// HTTP 状态码
    pub status: u16,

    /// 可选的详细错误信息（用于调试）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    /// 创建新的错误响应
    pub fn new(error: &'static str, message: String, status: u16) -> Self {
        Self {
            error,
            message,
            status,
            details: None,
        }
    }

    /// 添加详细错误信息
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// 便捷构造函数
impl ApiError {
    /// 未授权
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        ApiError::Unauthorized(msg.into())
    }

    /// 禁止访问
    pub fn forbidden(msg: impl Into<String>) -> Self {
        ApiError::Forbidden(msg.into())
    }

    /// 资源未找到
    pub fn not_found(msg: impl Into<String>) -> Self {
        ApiError::NotFound(msg.into())
    }

    /// 请求错误
    pub fn bad_request(msg: impl Into<String>) -> Self {
        ApiError::BadRequest(msg.into())
    }

    /// 内部服务器错误
    pub fn internal(msg: impl Into<String>) -> Self {
        ApiError::InternalError(msg.into())
    }

    /// 冲突
    pub fn conflict(msg: impl Into<String>) -> Self {
        ApiError::Conflict(msg.into())
    }

    /// 请求体过大
    pub fn payload_too_large(msg: impl Into<String>) -> Self {
        ApiError::PayloadTooLarge(msg.into())
    }

    /// 不支持的媒体类型
    pub fn unsupported_media_type(msg: impl Into<String>) -> Self {
        ApiError::UnsupportedMediaType(msg.into())
    }

    /// 服务不可用
    pub fn service_unavailable(msg: impl Into<String>) -> Self {
        ApiError::ServiceUnavailable(msg.into())
    }

    /// 数据库错误
    pub fn database(msg: impl Into<String>) -> Self {
        ApiError::DatabaseError(msg.into())
    }

    /// 验证错误
    pub fn validation(msg: impl Into<String>) -> Self {
        ApiError::ValidationError(msg.into())
    }
}

/// 从其他错误类型转换为 ApiError 的辅助 trait
pub trait IntoApiError {
    fn into_api_error(self) -> ApiError;
}

impl IntoApiError for sea_orm::DbErr {
    fn into_api_error(self) -> ApiError {
        tracing::error!("Database error: {}", self);
        ApiError::database(format!("Database operation failed: {}", self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_status_code() {
        assert_eq!(
            ApiError::unauthorized("test").status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ApiError::forbidden("test").status_code(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            ApiError::not_found("test").status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ApiError::bad_request("test").status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ApiError::internal("test").status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_api_error_display() {
        let err = ApiError::unauthorized("Invalid token");
        assert_eq!(format!("{}", err), "Unauthorized: Invalid token");
    }

    #[test]
    fn test_api_error_type() {
        assert_eq!(ApiError::unauthorized("test").error_type(), "unauthorized");
        assert_eq!(ApiError::not_found("test").error_type(), "not_found");
    }
}
