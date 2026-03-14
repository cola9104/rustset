//! Rate Limiting Middleware
//! 
//! Simple in-memory rate limiting for API endpoints

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Rate limiter configuration
#[derive(Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub block_duration_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            block_duration_seconds: 60,
        }
    }
}

/// Client rate limit state
struct ClientState {
    request_count: u32,
    window_start: Instant,
    blocked_until: Option<Instant>,
}

/// Rate limiter
pub struct RateLimiter {
    clients: RwLock<HashMap<String, ClientState>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Check if a client is allowed to make a request
    pub fn check(&self, client_id: &str) -> Result<(), String> {
        let mut clients = self.clients.write().unwrap();
        let now = Instant::now();

        // Clean up old entries periodically
        if clients.len() > 1000 {
            clients.retain(|_, state| {
                now.duration_since(state.window_start) < Duration::from_secs(120)
            });
        }

        let state = clients.entry(client_id.to_string()).or_insert(ClientState {
            request_count: 0,
            window_start: now,
            blocked_until: None,
        });

        // Check if blocked
        if let Some(blocked_until) = state.blocked_until {
            if now < blocked_until {
                let remaining = (blocked_until - now).as_secs();
                return Err(format!("Rate limit exceeded. Try again in {} seconds.", remaining));
            } else {
                // Unblock
                state.blocked_until = None;
                state.request_count = 0;
                state.window_start = now;
            }
        }

        // Check window
        let window_duration = Duration::from_secs(60);
        if now.duration_since(state.window_start) > window_duration {
            // Reset window
            state.request_count = 0;
            state.window_start = now;
        }

        // Check rate
        state.request_count += 1;
        if state.request_count > self.config.requests_per_minute {
            // Block client
            state.blocked_until = Some(now + Duration::from_secs(self.config.block_duration_seconds));
            return Err(format!(
                "Rate limit exceeded ({} requests/minute). Blocked for {} seconds.",
                self.config.requests_per_minute,
                self.config.block_duration_seconds
            ));
        }

        Ok(())
    }
}

/// Global rate limiter instance
pub static RATE_LIMITER: std::sync::OnceLock<Arc<RateLimiter>> = std::sync::OnceLock::new();

/// Initialize rate limiter
pub fn init_rate_limiter(config: RateLimitConfig) {
    RATE_LIMITER.set(Arc::new(RateLimiter::new(config))).ok();
}

/// Check rate limit for a client
pub fn check_rate_limit(client_id: &str) -> Result<(), String> {
    RATE_LIMITER
        .get()
        .ok_or("Rate limiter not initialized")?
        .check(client_id)
}
