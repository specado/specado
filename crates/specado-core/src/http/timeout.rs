//! Timeout configuration and management for HTTP requests
//!
//! Provides configurable request and connection timeouts with per-request overrides

use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Timeout configuration for HTTP requests
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Connection timeout - time to establish a connection
    pub connect_timeout: Duration,
    /// Request timeout - total time for the entire request
    pub request_timeout: Duration,
    /// Read timeout - time to wait for response data
    pub read_timeout: Option<Duration>,
    /// Keep-alive timeout for connection reuse
    pub keepalive_timeout: Option<Duration>,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            read_timeout: Some(Duration::from_secs(30)),
            keepalive_timeout: Some(Duration::from_secs(90)),
        }
    }
}

impl TimeoutConfig {
    /// Create a new timeout configuration
    pub fn new(
        connect_timeout: Duration,
        request_timeout: Duration,
        read_timeout: Option<Duration>,
        keepalive_timeout: Option<Duration>,
    ) -> Self {
        Self {
            connect_timeout,
            request_timeout,
            read_timeout,
            keepalive_timeout,
        }
    }
    
    /// Create a fast timeout configuration (for testing/development)
    pub fn fast() -> Self {
        Self {
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(15),
            read_timeout: Some(Duration::from_secs(15)),
            keepalive_timeout: Some(Duration::from_secs(30)),
        }
    }
    
    /// Create a slow timeout configuration (for large requests)
    pub fn slow() -> Self {
        Self {
            connect_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(300), // 5 minutes
            read_timeout: Some(Duration::from_secs(300)),
            keepalive_timeout: Some(Duration::from_secs(300)),
        }
    }
    
    /// Override the request timeout for a specific request
    pub fn with_request_timeout(&self, timeout: Duration) -> Self {
        let mut config = self.clone();
        config.request_timeout = timeout;
        config
    }
    
    /// Validate timeout configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.connect_timeout.is_zero() {
            return Err("Connect timeout cannot be zero".to_string());
        }
        
        if self.request_timeout.is_zero() {
            return Err("Request timeout cannot be zero".to_string());
        }
        
        // Request timeout should be >= connect timeout
        if self.request_timeout < self.connect_timeout {
            return Err("Request timeout should be >= connect timeout".to_string());
        }
        
        // If read timeout is set, it should be <= request timeout
        if let Some(read_timeout) = self.read_timeout {
            if read_timeout > self.request_timeout {
                return Err("Read timeout should be <= request timeout".to_string());
            }
        }
        
        Ok(())
    }
}

/// Per-request timeout override
#[derive(Debug, Clone)]
pub struct RequestTimeout {
    /// Override for the entire request
    pub request_timeout: Option<Duration>,
    /// Override for connection establishment
    pub connect_timeout: Option<Duration>,
}

impl RequestTimeout {
    /// Create a new request timeout override
    pub fn new(request_timeout: Option<Duration>, connect_timeout: Option<Duration>) -> Self {
        Self {
            request_timeout,
            connect_timeout,
        }
    }
    
    /// Create request timeout override with just request timeout
    pub fn request_only(timeout: Duration) -> Self {
        Self {
            request_timeout: Some(timeout),
            connect_timeout: None,
        }
    }
    
    /// Apply override to a timeout configuration
    pub fn apply_to(&self, config: &TimeoutConfig) -> TimeoutConfig {
        let mut new_config = config.clone();
        
        if let Some(request_timeout) = self.request_timeout {
            new_config.request_timeout = request_timeout;
        }
        
        if let Some(connect_timeout) = self.connect_timeout {
            new_config.connect_timeout = connect_timeout;
        }
        
        new_config
    }
}

/// Timeout wrapper using tokio::time::timeout
pub async fn with_timeout<F, T>(
    future: F,
    timeout_config: &TimeoutConfig,
    request_override: Option<&RequestTimeout>,
) -> Result<T, tokio::time::error::Elapsed>
where
    F: std::future::Future<Output = T>,
{
    let timeout_duration = if let Some(override_config) = request_override {
        override_config
            .request_timeout
            .unwrap_or(timeout_config.request_timeout)
    } else {
        timeout_config.request_timeout
    };
    
    tokio::time::timeout(timeout_duration, future).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_timeout_config_default() {
        let config = TimeoutConfig::default();
        assert_eq!(config.connect_timeout, Duration::from_secs(10));
        assert_eq!(config.request_timeout, Duration::from_secs(30));
        assert_eq!(config.read_timeout, Some(Duration::from_secs(30)));
        assert_eq!(config.keepalive_timeout, Some(Duration::from_secs(90)));
    }
    
    #[test]
    fn test_timeout_config_presets() {
        let fast = TimeoutConfig::fast();
        assert_eq!(fast.connect_timeout, Duration::from_secs(5));
        assert_eq!(fast.request_timeout, Duration::from_secs(15));
        
        let slow = TimeoutConfig::slow();
        assert_eq!(slow.connect_timeout, Duration::from_secs(30));
        assert_eq!(slow.request_timeout, Duration::from_secs(300));
    }
    
    #[test]
    fn test_timeout_config_validation() {
        let mut config = TimeoutConfig::default();
        assert!(config.validate().is_ok());
        
        // Zero connect timeout should fail
        config.connect_timeout = Duration::from_secs(0);
        assert!(config.validate().is_err());
        
        // Zero request timeout should fail
        config.connect_timeout = Duration::from_secs(10);
        config.request_timeout = Duration::from_secs(0);
        assert!(config.validate().is_err());
        
        // Request timeout < connect timeout should fail
        config.request_timeout = Duration::from_secs(5);
        config.connect_timeout = Duration::from_secs(10);
        assert!(config.validate().is_err());
        
        // Read timeout > request timeout should fail
        config.request_timeout = Duration::from_secs(30);
        config.connect_timeout = Duration::from_secs(10);
        config.read_timeout = Some(Duration::from_secs(60));
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_request_timeout_override() {
        let base_config = TimeoutConfig::default();
        let override_timeout = RequestTimeout::request_only(Duration::from_secs(60));
        
        let new_config = override_timeout.apply_to(&base_config);
        assert_eq!(new_config.request_timeout, Duration::from_secs(60));
        assert_eq!(new_config.connect_timeout, base_config.connect_timeout);
    }
    
    #[tokio::test]
    async fn test_timeout_wrapper() {
        let config = TimeoutConfig {
            connect_timeout: Duration::from_millis(100),
            request_timeout: Duration::from_millis(100),
            read_timeout: None,
            keepalive_timeout: None,
        };
        
        // Fast operation should succeed
        let result = with_timeout(
            async { tokio::time::sleep(Duration::from_millis(50)).await },
            &config,
            None,
        ).await;
        assert!(result.is_ok());
        
        // Slow operation should timeout
        let result = with_timeout(
            async { tokio::time::sleep(Duration::from_millis(200)).await },
            &config,
            None,
        ).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_timeout_override_wrapper() {
        let config = TimeoutConfig {
            connect_timeout: Duration::from_millis(50),
            request_timeout: Duration::from_millis(50),
            read_timeout: None,
            keepalive_timeout: None,
        };
        
        let override_timeout = RequestTimeout::request_only(Duration::from_millis(200));
        
        // Operation should succeed with override
        let result = with_timeout(
            async { tokio::time::sleep(Duration::from_millis(100)).await },
            &config,
            Some(&override_timeout),
        ).await;
        assert!(result.is_ok());
    }
}