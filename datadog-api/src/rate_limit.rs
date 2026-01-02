//! Client-side rate limiting for Datadog API requests
//!
//! Implements a token bucket algorithm to prevent hitting Datadog's rate limits.
//! The limiter tracks requests per second and delays when limits are approached.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::debug;

/// Default requests per second limit (conservative estimate for Datadog API)
pub const DEFAULT_REQUESTS_PER_SECOND: u32 = 10;

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per second
    pub requests_per_second: u32,
    /// Whether rate limiting is enabled
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: DEFAULT_REQUESTS_PER_SECOND,
            enabled: true,
        }
    }
}

impl RateLimitConfig {
    /// Create a new rate limit config with custom requests per second
    pub fn new(requests_per_second: u32) -> Self {
        Self {
            requests_per_second,
            enabled: true,
        }
    }

    /// Disable rate limiting
    pub fn disabled() -> Self {
        Self {
            requests_per_second: DEFAULT_REQUESTS_PER_SECOND,
            enabled: false,
        }
    }
}

/// Token bucket rate limiter
///
/// Allows bursts up to the bucket size, then rate limits to the configured RPS.
#[derive(Debug)]
pub struct RateLimiter {
    config: RateLimitConfig,
    state: Arc<Mutex<RateLimiterState>>,
}

#[derive(Debug)]
struct RateLimiterState {
    /// Available tokens (fractional to allow smooth rate limiting)
    tokens: f64,
    /// Last time tokens were updated
    last_update: Instant,
    /// Maximum tokens (bucket size = 2x RPS for burst allowance)
    max_tokens: f64,
    /// Token refill rate per millisecond
    refill_rate: f64,
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration
    pub fn new(config: RateLimitConfig) -> Self {
        let max_tokens = (config.requests_per_second * 2) as f64; // Allow 2-second burst
        let refill_rate = config.requests_per_second as f64 / 1000.0; // tokens per ms

        Self {
            config,
            state: Arc::new(Mutex::new(RateLimiterState {
                tokens: max_tokens, // Start with full bucket
                last_update: Instant::now(),
                max_tokens,
                refill_rate,
            })),
        }
    }

    /// Acquire a token, waiting if necessary
    ///
    /// Returns immediately if a token is available, otherwise waits
    /// until a token becomes available.
    pub async fn acquire(&self) {
        if !self.config.enabled {
            return;
        }

        loop {
            let wait_time = {
                let mut state = self.state.lock().await;

                // Refill tokens based on elapsed time
                let now = Instant::now();
                let elapsed_ms = now.duration_since(state.last_update).as_millis() as f64;
                state.tokens = (state.tokens + elapsed_ms * state.refill_rate).min(state.max_tokens);
                state.last_update = now;

                if state.tokens >= 1.0 {
                    // Token available, consume it
                    state.tokens -= 1.0;
                    debug!(
                        "Rate limiter: acquired token, {} remaining",
                        state.tokens as u32
                    );
                    return;
                }

                // Calculate wait time for next token
                let tokens_needed = 1.0 - state.tokens;
                let wait_ms = (tokens_needed / state.refill_rate).ceil() as u64;
                Duration::from_millis(wait_ms.max(1))
            };

            debug!("Rate limiter: waiting {:?} for token", wait_time);
            tokio::time::sleep(wait_time).await;
        }
    }

    /// Try to acquire a token without waiting
    ///
    /// Returns true if a token was acquired, false if rate limited.
    pub async fn try_acquire(&self) -> bool {
        if !self.config.enabled {
            return true;
        }

        let mut state = self.state.lock().await;

        // Refill tokens
        let now = Instant::now();
        let elapsed_ms = now.duration_since(state.last_update).as_millis() as f64;
        state.tokens = (state.tokens + elapsed_ms * state.refill_rate).min(state.max_tokens);
        state.last_update = now;

        if state.tokens >= 1.0 {
            state.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Get current available tokens (for monitoring)
    pub async fn available_tokens(&self) -> u32 {
        let state = self.state.lock().await;
        state.tokens as u32
    }
}

impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            state: Arc::clone(&self.state),
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(RateLimitConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_acquire() {
        let limiter = RateLimiter::new(RateLimitConfig::new(100));

        // Should be able to acquire immediately (bucket starts full)
        for _ in 0..10 {
            limiter.acquire().await;
        }

        // Check we consumed tokens
        let available = limiter.available_tokens().await;
        assert!(available < 200); // Started with 200 (2x RPS)
    }

    #[tokio::test]
    async fn test_rate_limiter_try_acquire() {
        let limiter = RateLimiter::new(RateLimitConfig::new(10));

        // Should succeed for burst
        for _ in 0..20 {
            assert!(limiter.try_acquire().await);
        }

        // Should fail when bucket is empty
        assert!(!limiter.try_acquire().await);
    }

    #[tokio::test]
    async fn test_rate_limiter_disabled() {
        let limiter = RateLimiter::new(RateLimitConfig::disabled());

        // Should always succeed when disabled
        for _ in 0..100 {
            assert!(limiter.try_acquire().await);
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_refill() {
        let limiter = RateLimiter::new(RateLimitConfig::new(1000)); // 1000 RPS = 1 per ms

        // Drain the bucket
        for _ in 0..2000 {
            limiter.try_acquire().await;
        }

        // Wait for refill
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Should have some tokens now
        assert!(limiter.try_acquire().await);
    }

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_second, DEFAULT_REQUESTS_PER_SECOND);
        assert!(config.enabled);
    }

    #[test]
    fn test_rate_limit_config_disabled() {
        let config = RateLimitConfig::disabled();
        assert!(!config.enabled);
    }

    #[tokio::test]
    async fn test_rate_limiter_clone_shares_state() {
        let limiter1 = RateLimiter::new(RateLimitConfig::new(10));
        let limiter2 = limiter1.clone();

        // Drain through limiter1
        for _ in 0..20 {
            limiter1.try_acquire().await;
        }

        // limiter2 should see the same empty bucket
        assert!(!limiter2.try_acquire().await);
    }
}
