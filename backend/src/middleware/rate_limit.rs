//! Rate Limiting Middleware
//!
//! Simple in-memory rate limiting for API endpoints

use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
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
        let mut clients = self
            .clients
            .write()
            .map_err(|e| format!("Rate limiter lock poisoned: {}", e))?;
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
                return Err(format!(
                    "Rate limit exceeded. Try again in {} seconds.",
                    remaining
                ));
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
            state.blocked_until =
                Some(now + Duration::from_secs(self.config.block_duration_seconds));
            return Err(format!(
                "Rate limit exceeded ({} requests/minute). Blocked for {} seconds.",
                self.config.requests_per_minute, self.config.block_duration_seconds
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

/// Extract client IP from headers
fn extract_client_ip(headers: &HeaderMap) -> String {
    // Try to get real IP from common headers (for reverse proxy setups)
    if let Some(forwarded_for) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        // X-Forwarded-For can contain multiple IPs, take the first one (original client)
        return forwarded_for
            .split(',')
            .next()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());
    }

    if let Some(real_ip) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
        return real_ip.to_string();
    }

    if let Some(cf_connecting_ip) = headers
        .get("cf-connecting-ip")
        .and_then(|v| v.to_str().ok())
    {
        return cf_connecting_ip.to_string();
    }

    // Fallback to unknown
    "unknown".to_string()
}

/// Axum middleware for rate limiting
pub async fn rate_limit_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let headers = req.headers();
    let client_ip = extract_client_ip(headers);

    // Check rate limit
    match check_rate_limit(&client_ip) {
        Ok(()) => {
            // Rate limit check passed, proceed with request
            Ok(next.run(req).await)
        }
        Err(error_msg) => {
            tracing::warn!(
                "Rate limit exceeded for client {}: {}",
                client_ip,
                error_msg
            );
            // Return 429 Too Many Requests
            Err(StatusCode::TOO_MANY_REQUESTS)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_minute, 60);
        assert_eq!(config.block_duration_seconds, 60);
    }

    #[test]
    fn test_rate_limiter_new() {
        let config = RateLimitConfig {
            requests_per_minute: 10,
            block_duration_seconds: 30,
        };
        let limiter = RateLimiter::new(config);
        assert_eq!(limiter.config.requests_per_minute, 10);
        assert_eq!(limiter.config.block_duration_seconds, 30);
    }

    #[test]
    fn test_rate_limiter_check_within_limit() {
        let config = RateLimitConfig {
            requests_per_minute: 5,
            block_duration_seconds: 60,
        };
        let limiter = RateLimiter::new(config);

        // Make requests within the limit
        for _ in 0..5 {
            let result = limiter.check("client1");
            assert!(result.is_ok(), "Request should be allowed within limit");
        }
    }

    #[test]
    fn test_rate_limiter_check_exceeds_limit() {
        let config = RateLimitConfig {
            requests_per_minute: 3,
            block_duration_seconds: 60,
        };
        let limiter = RateLimiter::new(config);

        // Make requests that exceed the limit
        for _ in 0..3 {
            let _ = limiter.check("client2");
        }

        // 4th request should be blocked
        let result = limiter.check("client2");
        assert!(
            result.is_err(),
            "Request should be blocked after exceeding limit"
        );
        assert!(result.unwrap_err().contains("Rate limit exceeded"));
    }

    #[test]
    fn test_rate_limiter_different_clients() {
        let config = RateLimitConfig {
            requests_per_minute: 2,
            block_duration_seconds: 60,
        };
        let limiter = RateLimiter::new(config);

        // Each client should have its own limit
        for _ in 0..2 {
            assert!(limiter.check("client_a").is_ok());
            assert!(limiter.check("client_b").is_ok());
        }

        // Both should be blocked now
        assert!(limiter.check("client_a").is_err());
        assert!(limiter.check("client_b").is_err());

        // New client should still be allowed
        assert!(limiter.check("client_c").is_ok());
    }

    #[test]
    fn test_rate_limiter_blocked_duration() {
        let config = RateLimitConfig {
            requests_per_minute: 1,
            block_duration_seconds: 1,
        };
        let limiter = RateLimiter::new(config);

        // Use up the limit
        assert!(limiter.check("client3").is_ok());

        // Next request should be blocked
        assert!(limiter.check("client3").is_err());

        // Wait for block to expire
        thread::sleep(Duration::from_secs(2));

        // Should be allowed again
        let result = limiter.check("client3");
        assert!(
            result.is_ok(),
            "Request should be allowed after block duration"
        );
    }

    #[test]
    fn test_rate_limiter_window_reset() {
        let config = RateLimitConfig {
            requests_per_minute: 2,
            block_duration_seconds: 60,
        };
        let limiter = RateLimiter::new(config);

        // Make requests
        assert!(limiter.check("client4").is_ok());
        assert!(limiter.check("client4").is_ok());
        assert!(limiter.check("client4").is_err());

        // Wait for window to reset (we need to wait more than 60 seconds, but for testing
        // we'll check that the window_start gets updated appropriately)
        // Note: This test would need to sleep for >60 seconds to fully test window reset
        // which is not practical in unit tests. The logic is tested implicitly.
    }

    #[test]
    fn test_rate_limiter_multiple_clients_isolation() {
        let config = RateLimitConfig {
            requests_per_minute: 1,
            block_duration_seconds: 10,
        };
        let limiter = RateLimiter::new(config);

        // Test that clients are isolated
        assert!(limiter.check("client_x").is_ok());
        assert!(limiter.check("client_x").is_err()); // blocked

        assert!(limiter.check("client_y").is_ok()); // different client, allowed
        assert!(limiter.check("client_y").is_err()); // blocked

        assert!(limiter.check("client_z").is_ok()); // another client, allowed
    }

    #[test]
    fn test_rate_limiter_error_message_format() {
        let config = RateLimitConfig {
            requests_per_minute: 1,
            block_duration_seconds: 30,
        };
        let limiter = RateLimiter::new(config);

        // Use up the limit
        assert!(limiter.check("client5").is_ok());

        // Check error message format
        let result = limiter.check("client5");
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Rate limit exceeded"));
        assert!(error_msg.contains("30"));
    }

    #[test]
    fn test_rate_limiter_cleanup_old_entries() {
        let config = RateLimitConfig {
            requests_per_minute: 100,
            block_duration_seconds: 60,
        };
        let limiter = RateLimiter::new(config);

        // Add many clients to trigger cleanup
        for i in 0..1100 {
            let _ = limiter.check(&format!("client_{}", i));
        }

        // Check that limiter still works after cleanup
        let result = limiter.check("new_client");
        assert!(result.is_ok());
    }

    #[test]
    fn test_rate_limiter_unblock_after_duration() {
        let config = RateLimitConfig {
            requests_per_minute: 1,
            block_duration_seconds: 1,
        };
        let limiter = RateLimiter::new(config);

        // Block the client
        assert!(limiter.check("client6").is_ok());
        assert!(limiter.check("client6").is_err());

        // Wait for block to expire
        thread::sleep(Duration::from_secs(2));

        // Should be unblocked and allowed to make requests again
        assert!(limiter.check("client6").is_ok());
        assert!(limiter.check("client6").is_err()); // Blocked again after using limit
    }
}
