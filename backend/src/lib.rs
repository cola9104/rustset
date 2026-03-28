//! RustSet Backend Library
//!
//! 这个库包含了 backend 的所有模块，供测试使用。

pub mod auth;
pub mod config;
pub mod database;
pub mod entities;
pub mod handlers;
pub mod middleware;
pub mod migration;
pub mod password;
pub mod redis;
pub mod scanners;
pub mod state;
pub mod utils;

pub use state::AppState;
