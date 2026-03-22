//! RustSet Backend Library
//!
//! 这个库包含了 backend 的所有模块，供测试使用。

pub mod state;
pub mod utils;
pub mod auth;
pub mod password;
pub mod handlers;
pub mod scanners;
pub mod database;
pub mod config;
pub mod entities;
pub mod migration;
pub mod middleware;

pub use state::AppState;
