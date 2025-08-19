//! Rate limiting implementation using token bucket algorithm
//!
//! Provides configurable rate limiting to prevent overwhelming APIs
//! and handle 429 responses with Retry-After headers

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

/// Rate limiting configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum number of requests per time window
    pub max_requests: u32,
    /// Time window for the rate limit
    pub time_window: Duration,
    /// Maximum number of tokens that can be stored (burst capacity)
    pub burst_size: u32,
    /// Rate of token refill per second
    pub refill_rate: f64,
    /// Whether to enable per-provider rate limiting
    pub per_provider: bool,
    /// Custom rate limits per provider
    pub provider_limits: HashMap<String, ProviderRateLimit>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 60,                    // 60 requests
            time_window: Duration::from_secs(60), // per minute
            burst_size: 10,                      // Allow bursts of 10
            refill_rate: 1.0,                    // 1 token per second
            per_provider: true,
            provider_limits: HashMap::new(),
        }
    }
}

/// Provider-specific rate limit configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderRateLimit {
    /// Maximum requests per time window for this provider
    pub max_requests: u32,
    /// Time window for this provider
    pub time_window: Duration,
    /// Burst size for this provider
    pub burst_size: u32,
    /// Refill rate for this provider
    pub refill_rate: f64,
}

impl ProviderRateLimit {
    /// Create a new provider rate limit
    pub fn new(max_requests: u32, time_window: Duration, burst_size: u32, refill_rate: f64) -> Self {
        Self {
            max_requests,
            time_window,
            burst_size,
            refill_rate,
        }
    }
}

/// Token bucket for rate limiting
#[derive(Debug)]
struct TokenBucket {
    /// Current number of tokens
    tokens: f64,
    /// Maximum number of tokens (burst capacity)
    capacity: f64,
    /// Rate of token refill per second
    refill_rate: f64,
    /// Last refill time
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket
    fn new(capacity: u32, refill_rate: f64) -> Self {
        Self {
            tokens: capacity as f64,
            capacity: capacity as f64,
            refill_rate,
            last_refill: Instant::now(),
        }
    }
    
    /// Try to consume tokens from the bucket
    fn try_consume(&mut self, tokens: u32) -> bool {
        self.refill();
        
        let tokens_f64 = tokens as f64;
        if self.tokens >= tokens_f64 {
            self.tokens -= tokens_f64;
            true
        } else {
            false
        }
    }
    
    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let new_tokens = elapsed * self.refill_rate;
        
        self.tokens = (self.tokens + new_tokens).min(self.capacity);
        self.last_refill = now;
    }
    
    /// Get time until next token is available
    fn time_until_available(&mut self, tokens: u32) -> Duration {
        self.refill();
        
        let tokens_f64 = tokens as f64;
        if self.tokens >= tokens_f64 {
            Duration::from_secs(0)
        } else {
            let tokens_needed = tokens_f64 - self.tokens;
            let time_needed = tokens_needed / self.refill_rate;
            Duration::from_secs_f64(time_needed)
        }
    }
}

/// Rate limiter implementation
#[derive(Debug)]
pub struct RateLimiter {
    /// Configuration
    config: RateLimitConfig,
    /// Token buckets per provider (if per_provider is enabled)
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    /// Global token bucket (if per_provider is disabled)
    global_bucket: Arc<Mutex<TokenBucket>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        let global_bucket = Arc::new(Mutex::new(TokenBucket::new(
            config.burst_size,
            config.refill_rate,
        )));
        
        Self {
            config,
            buckets: Arc::new(Mutex::new(HashMap::new())),
            global_bucket,
        }
    }
    
    /// Wait for rate limit clearance for a request
    pub async fn wait_for_permit(&self, provider: &str) -> Result<(), RateLimitError> {
        self.wait_for_permits(provider, 1).await
    }
    
    /// Wait for rate limit clearance for multiple requests
    pub async fn wait_for_permits(&self, provider: &str, count: u32) -> Result<(), RateLimitError> {
        if count == 0 {
            return Ok(());
        }
        
        loop {
            let wait_time = if self.config.per_provider {
                self.check_provider_limit(provider, count)
            } else {
                self.check_global_limit(count)
            };
            
            match wait_time {
                Ok(()) => return Ok(()),
                Err(RateLimitError::WaitRequired(duration)) => {
                    if duration > Duration::from_secs(300) {
                        // Don't wait more than 5 minutes
                        return Err(RateLimitError::ExcessiveDelay(duration));
                    }
                    sleep(duration).await;
                }
                Err(e) => return Err(e),
            }
        }
    }
    
    /// Check provider-specific rate limit
    fn check_provider_limit(&self, provider: &str, tokens: u32) -> Result<(), RateLimitError> {
        let mut buckets = self.buckets.lock().unwrap();
        
        // Get or create bucket for this provider
        let bucket = buckets.entry(provider.to_string()).or_insert_with(|| {
            // Use provider-specific config if available, otherwise use default
            let provider_config = self.config.provider_limits.get(provider);
            match provider_config {
                Some(config) => TokenBucket::new(config.burst_size, config.refill_rate),
                None => TokenBucket::new(self.config.burst_size, self.config.refill_rate),
            }
        });
        
        if bucket.try_consume(tokens) {
            Ok(())
        } else {
            let wait_time = bucket.time_until_available(tokens);
            Err(RateLimitError::WaitRequired(wait_time))
        }
    }
    
    /// Check global rate limit
    fn check_global_limit(&self, tokens: u32) -> Result<(), RateLimitError> {
        let mut bucket = self.global_bucket.lock().unwrap();
        
        if bucket.try_consume(tokens) {
            Ok(())
        } else {
            let wait_time = bucket.time_until_available(tokens);
            Err(RateLimitError::WaitRequired(wait_time))
        }
    }
    
    /// Handle 429 response with Retry-After header
    pub async fn handle_429_response(
        &self,
        retry_after_secs: Option<u64>,
        provider: &str,
    ) -> Result<(), RateLimitError> {
        let wait_time = match retry_after_secs {
            Some(secs) => Duration::from_secs(secs),
            None => {
                // Use provider-specific or default backoff
                let provider_config = self.config.provider_limits.get(provider);
                match provider_config {
                    Some(config) => config.time_window,
                    None => self.config.time_window,
                }
            }
        };
        
        if wait_time > Duration::from_secs(300) {
            // Don't wait more than 5 minutes for 429
            return Err(RateLimitError::ExcessiveDelay(wait_time));
        }
        
        // Clear tokens for this provider to enforce the wait
        if self.config.per_provider {
            let mut buckets = self.buckets.lock().unwrap();
            if let Some(bucket) = buckets.get_mut(provider) {
                bucket.tokens = 0.0; // Force wait
            }
        } else {
            let mut bucket = self.global_bucket.lock().unwrap();
            bucket.tokens = 0.0; // Force wait
        }
        
        sleep(wait_time).await;
        Ok(())
    }
    
    /// Get current token counts for debugging
    pub fn get_token_status(&self) -> TokenStatus {
        let provider_tokens = if self.config.per_provider {
            let buckets = self.buckets.lock().unwrap();
            buckets
                .iter()
                .map(|(k, v)| (k.clone(), v.tokens))
                .collect()
        } else {
            HashMap::new()
        };
        
        let global_tokens = if !self.config.per_provider {
            Some(self.global_bucket.lock().unwrap().tokens)
        } else {
            None
        };
        
        TokenStatus {
            provider_tokens,
            global_tokens,
        }
    }
}

/// Rate limit error types
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded, wait required: {0:?}")]
    WaitRequired(Duration),
    
    #[error("Rate limit delay too long: {0:?}")]
    ExcessiveDelay(Duration),
    
    #[error("Rate limit configuration error: {0}")]
    ConfigError(String),
}

/// Token status for debugging
#[derive(Debug)]
pub struct TokenStatus {
    /// Tokens available per provider
    pub provider_tokens: HashMap<String, f64>,
    /// Global tokens available (if not using per-provider)
    pub global_tokens: Option<f64>,
}

impl RateLimitConfig {
    /// Create a new rate limit configuration
    pub fn new(max_requests: u32, time_window: Duration) -> Self {
        let refill_rate = max_requests as f64 / time_window.as_secs_f64();
        
        Self {
            max_requests,
            time_window,
            burst_size: max_requests.min(10), // Default burst is smaller
            refill_rate,
            per_provider: true,
            provider_limits: HashMap::new(),
        }
    }
    
    /// Create configuration for high-volume usage
    pub fn high_volume() -> Self {
        Self::new(1000, Duration::from_secs(60)) // 1000 requests per minute
    }
    
    /// Create configuration for low-volume usage
    pub fn low_volume() -> Self {
        Self::new(60, Duration::from_secs(60)) // 60 requests per minute
    }
    
    /// Add provider-specific rate limit
    pub fn with_provider_limit(mut self, provider: String, limit: ProviderRateLimit) -> Self {
        self.provider_limits.insert(provider, limit);
        self
    }
    
    /// Enable or disable per-provider rate limiting
    pub fn with_per_provider(mut self, per_provider: bool) -> Self {
        self.per_provider = per_provider;
        self
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_requests == 0 {
            return Err("max_requests cannot be zero".to_string());
        }
        
        if self.time_window.is_zero() {
            return Err("time_window cannot be zero".to_string());
        }
        
        if self.burst_size == 0 {
            return Err("burst_size cannot be zero".to_string());
        }
        
        if self.refill_rate <= 0.0 {
            return Err("refill_rate must be positive".to_string());
        }
        
        // Validate provider limits
        for (provider, limit) in &self.provider_limits {
            if limit.max_requests == 0 {
                return Err(format!("Provider {} max_requests cannot be zero", provider));
            }
            if limit.time_window.is_zero() {
                return Err(format!("Provider {} time_window cannot be zero", provider));
            }
            if limit.refill_rate <= 0.0 {
                return Err(format!("Provider {} refill_rate must be positive", provider));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.max_requests, 60);
        assert_eq!(config.time_window, Duration::from_secs(60));
        assert_eq!(config.burst_size, 10);
        assert!(config.per_provider);
    }
    
    #[test]
    fn test_rate_limit_config_presets() {
        let high = RateLimitConfig::high_volume();
        assert_eq!(high.max_requests, 1000);
        
        let low = RateLimitConfig::low_volume();
        assert_eq!(low.max_requests, 60);
    }
    
    #[test]
    fn test_token_bucket() {
        let mut bucket = TokenBucket::new(10, 1.0);
        
        // Should be able to consume initial tokens
        assert!(bucket.try_consume(5));
        assert!(bucket.try_consume(5));
        
        // Should not be able to consume more
        assert!(!bucket.try_consume(1));
        
        // Wait time should be calculated correctly
        let wait_time = bucket.time_until_available(1);
        assert!(wait_time > Duration::from_secs(0));
        assert!(wait_time <= Duration::from_secs(1));
    }
    
    #[test]
    fn test_rate_limit_config_validation() {
        let mut config = RateLimitConfig::default();
        assert!(config.validate().is_ok());
        
        // Zero max_requests should fail
        config.max_requests = 0;
        assert!(config.validate().is_err());
        
        // Zero time_window should fail
        config.max_requests = 60;
        config.time_window = Duration::from_secs(0);
        assert!(config.validate().is_err());
        
        // Zero burst_size should fail
        config.time_window = Duration::from_secs(60);
        config.burst_size = 0;
        assert!(config.validate().is_err());
        
        // Negative refill_rate should fail
        config.burst_size = 10;
        config.refill_rate = -1.0;
        assert!(config.validate().is_err());
    }
    
    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let config = RateLimitConfig::new(2, Duration::from_secs(2));
        let limiter = RateLimiter::new(config);
        
        // First two requests should succeed immediately
        assert!(limiter.wait_for_permit("test-provider").await.is_ok());
        assert!(limiter.wait_for_permit("test-provider").await.is_ok());
        
        // Third request should require waiting
        let start = Instant::now();
        assert!(limiter.wait_for_permit("test-provider").await.is_ok());
        let elapsed = start.elapsed();
        
        // Should have waited some time (allowing for test timing variance)
        assert!(elapsed >= Duration::from_millis(500));
    }
    
    #[tokio::test]
    async fn test_rate_limiter_per_provider() {
        let config = RateLimitConfig::new(1, Duration::from_secs(1))
            .with_per_provider(true);
        let limiter = RateLimiter::new(config);
        
        // Different providers should have separate limits
        assert!(limiter.wait_for_permit("provider1").await.is_ok());
        assert!(limiter.wait_for_permit("provider2").await.is_ok());
        
        // Same provider should be rate limited
        let start = Instant::now();
        assert!(limiter.wait_for_permit("provider1").await.is_ok());
        let elapsed = start.elapsed();
        
        assert!(elapsed >= Duration::from_millis(500));
    }
    
    #[tokio::test]
    async fn test_handle_429_response() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);
        
        // Should handle 429 with retry-after
        let result = limiter.handle_429_response(Some(1), "test-provider").await;
        assert!(result.is_ok());
        
        // Should reject excessive delays
        let result = limiter.handle_429_response(Some(400), "test-provider").await;
        assert!(matches!(result, Err(RateLimitError::ExcessiveDelay(_))));
    }
    
    #[test]
    fn test_provider_rate_limit() {
        let limit = ProviderRateLimit::new(100, Duration::from_secs(60), 20, 1.67);
        assert_eq!(limit.max_requests, 100);
        assert_eq!(limit.time_window, Duration::from_secs(60));
        assert_eq!(limit.burst_size, 20);
        assert_eq!(limit.refill_rate, 1.67);
    }
}