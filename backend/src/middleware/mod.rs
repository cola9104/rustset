//! 中间件模块

pub mod error_handler;
pub mod rate_limit;
pub mod cors;
pub mod auth_middleware;
pub mod performance;

#[allow(unused_imports)]
pub use error_handler::{ApiError, ErrorResponse, IntoApiError};
pub use rate_limit::{RateLimitConfig, RateLimiter, init_rate_limiter, check_rate_limit, rate_limit_middleware};
pub use auth_middleware::{auth_middleware, AuthUser};
pub use performance::performance_monitoring;
pub use cors::create_cors_layer;
