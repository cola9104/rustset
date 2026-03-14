//! 中间件模块
//!
//! 包含所有中间件，如错误处理、认证、日志等

pub mod error_handler;

#[allow(unused_imports)]
pub use error_handler::{ApiError, ErrorResponse, IntoApiError};
