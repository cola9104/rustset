//! 中间件模块
//!
//! 包含所有中间件，如错误处理、认证、日志等

pub mod error_handler;
pub mod rate_limit;

#[allow(unused_imports)]
pub use error_handler::{ApiError, ErrorResponse, IntoApiError};
pub use rate_limit::{RateLimiter, RateLimitConfig, init_rate_limiter, check_rate_limit};
