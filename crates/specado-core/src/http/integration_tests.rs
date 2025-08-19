//! Integration tests demonstrating all 4 new HTTP client features
//!
//! This module contains examples of how to use:
//! 1. Timeout Configuration
//! 2. TLS/HTTPS Support
//! 3. Rate Limiting
//! 4. Network Error Handling

#[cfg(test)]
mod tests {
    use crate::http::{
        HttpClientConfig, TimeoutConfig, TlsConfig, RateLimitConfig, 
        CircuitBreakerConfig, RequestTimeout, ProviderRateLimit, TlsVersion
    };
    use std::time::Duration;
    use std::collections::HashMap;

    #[test]
    fn test_comprehensive_http_client_configuration() {
        // 1. Timeout Configuration
        let timeout_config = TimeoutConfig::new(
            Duration::from_secs(5),  // connect timeout
            Duration::from_secs(30), // request timeout
            Some(Duration::from_secs(25)), // read timeout
            Some(Duration::from_secs(90)), // keepalive timeout
        );
        
        assert!(timeout_config.validate().is_ok());

        // 2. TLS Configuration with development settings
        let tls_config = TlsConfig::development()
            .with_min_tls_version(TlsVersion::TLS1_2)
            .with_ca_cert_pem("-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----".to_string());
        
        assert!(tls_config.validate().is_ok());
        assert!(!tls_config.validate_certificates);
        assert!(tls_config.accept_invalid_hostnames);
        assert_eq!(tls_config.min_tls_version, TlsVersion::TLS1_2);

        // 3. Rate Limiting Configuration
        let mut provider_limits = HashMap::new();
        provider_limits.insert(
            "openai".to_string(),
            ProviderRateLimit::new(
                100, // 100 requests
                Duration::from_secs(60), // per minute
                20,  // burst of 20
                1.67 // refill rate
            )
        );
        
        let rate_limit_config = RateLimitConfig::new(60, Duration::from_secs(60))
            .with_provider_limit("openai".to_string(), provider_limits["openai"].clone())
            .with_per_provider(true);
        
        assert!(rate_limit_config.validate().is_ok());
        assert!(rate_limit_config.per_provider);
        assert_eq!(rate_limit_config.max_requests, 60);

        // 4. Circuit Breaker Configuration
        let circuit_breaker_config = CircuitBreakerConfig {
            failure_threshold: 3,                         // 3 failures
            failure_window: Duration::from_secs(30),      // in 30 seconds
            recovery_timeout: Duration::from_secs(15),    // wait 15 seconds
            success_threshold: 2,                         // 2 successes to close
            per_endpoint: true,
            min_request_rate: 5, // 5 requests per minute minimum
        };

        // Create comprehensive HTTP client configuration
        let config = HttpClientConfig {
            retry_policy: crate::http::RetryPolicy::default(),
            timeout_secs: 30, // Backward compatibility
            validate_tls: false, // Backward compatibility
            fallback_config: crate::http::FallbackConfig::default(),
            timeout_config,
            tls_config,
            rate_limit_config: Some(rate_limit_config),
            circuit_breaker_config,
        };

        // Validate the complete configuration
        assert!(config.timeout_config.validate().is_ok());
        assert!(config.tls_config.validate().is_ok());
        assert!(config.rate_limit_config.as_ref().unwrap().validate().is_ok());
        
        // Test timeout override functionality
        let override_timeout = RequestTimeout::request_only(Duration::from_secs(60));
        let modified_config = override_timeout.apply_to(&config.timeout_config);
        assert_eq!(modified_config.request_timeout, Duration::from_secs(60));
        assert_eq!(modified_config.connect_timeout, Duration::from_secs(5)); // unchanged
    }

    #[test]
    fn test_timeout_configurations() {
        // Test fast timeout preset
        let fast_config = TimeoutConfig::fast();
        assert_eq!(fast_config.connect_timeout, Duration::from_secs(5));
        assert_eq!(fast_config.request_timeout, Duration::from_secs(15));
        
        // Test slow timeout preset
        let slow_config = TimeoutConfig::slow();
        assert_eq!(slow_config.connect_timeout, Duration::from_secs(30));
        assert_eq!(slow_config.request_timeout, Duration::from_secs(300));
        
        // Test timeout override
        let base_config = TimeoutConfig::default();
        let override_config = base_config.with_request_timeout(Duration::from_secs(120));
        assert_eq!(override_config.request_timeout, Duration::from_secs(120));
    }

    #[test]
    fn test_tls_configurations() {
        // Test secure preset
        let secure = TlsConfig::secure();
        assert!(secure.validate_certificates);
        assert!(!secure.accept_invalid_hostnames);
        assert_eq!(secure.min_tls_version, TlsVersion::TLS1_2);
        
        // Test development preset
        let dev = TlsConfig::development();
        assert!(!dev.validate_certificates);
        assert!(dev.accept_invalid_hostnames);
        
        // Test testing preset
        let testing = TlsConfig::testing();
        assert!(!testing.validate_certificates);
        assert!(testing.accept_invalid_hostnames);
        assert_eq!(testing.min_tls_version, TlsVersion::TLS1_0);
        
        // Test builder pattern
        let custom_tls = TlsConfig::secure()
            .with_min_tls_version(TlsVersion::TLS1_3)
            .with_sni_hostname("api.example.com".to_string());
        
        assert_eq!(custom_tls.min_tls_version, TlsVersion::TLS1_3);
        assert_eq!(custom_tls.sni_hostname, Some("api.example.com".to_string()));
    }

    #[test]
    fn test_rate_limiting_configurations() {
        // Test high volume preset
        let high_volume = RateLimitConfig::high_volume();
        assert_eq!(high_volume.max_requests, 1000);
        assert_eq!(high_volume.time_window, Duration::from_secs(60));
        
        // Test low volume preset
        let low_volume = RateLimitConfig::low_volume();
        assert_eq!(low_volume.max_requests, 60);
        
        // Test per-provider configuration
        let provider_config = RateLimitConfig::new(100, Duration::from_secs(60))
            .with_per_provider(true)
            .with_provider_limit(
                "anthropic".to_string(),
                ProviderRateLimit::new(50, Duration::from_secs(60), 10, 0.83)
            );
        
        assert!(provider_config.per_provider);
        assert!(provider_config.provider_limits.contains_key("anthropic"));
        assert_eq!(provider_config.provider_limits["anthropic"].max_requests, 50);
    }

    #[test]
    fn test_circuit_breaker_configurations() {
        let config = CircuitBreakerConfig::default();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.failure_window, Duration::from_secs(60));
        assert_eq!(config.recovery_timeout, Duration::from_secs(30));
        assert_eq!(config.success_threshold, 3);
        assert!(config.per_endpoint);
        assert_eq!(config.min_request_rate, 10);
        
        // Test custom configuration
        let custom_config = CircuitBreakerConfig {
            failure_threshold: 2,
            failure_window: Duration::from_secs(10),
            recovery_timeout: Duration::from_secs(5),
            success_threshold: 1,
            per_endpoint: false,
            min_request_rate: 3,
        };
        
        assert_eq!(custom_config.failure_threshold, 2);
        assert!(!custom_config.per_endpoint);
    }

    #[tokio::test]
    async fn test_timeout_wrapper() {
        use crate::http::timeout::with_timeout;
        
        let config = TimeoutConfig {
            connect_timeout: Duration::from_millis(100),
            request_timeout: Duration::from_millis(50),
            read_timeout: None,
            keepalive_timeout: None,
        };
        
        // Fast operation should succeed
        let result = with_timeout(
            async { 
                tokio::time::sleep(Duration::from_millis(10)).await;
                42
            },
            &config,
            None,
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        
        // Slow operation should timeout
        let result = with_timeout(
            async { 
                tokio::time::sleep(Duration::from_millis(100)).await;
                42
            },
            &config,
            None,
        ).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter_integration() {
        use crate::http::rate_limit::RateLimiter;
        
        let config = RateLimitConfig::new(2, Duration::from_secs(1))
            .with_per_provider(true);
        let limiter = RateLimiter::new(config);
        
        // First two requests should succeed quickly
        let start = std::time::Instant::now();
        assert!(limiter.wait_for_permit("test-provider").await.is_ok());
        assert!(limiter.wait_for_permit("test-provider").await.is_ok());
        let elapsed = start.elapsed();
        
        // Should be very fast (< 100ms)
        assert!(elapsed < Duration::from_millis(100));
        
        // Third request should require waiting
        let start = std::time::Instant::now();
        assert!(limiter.wait_for_permit("test-provider").await.is_ok());
        let elapsed = start.elapsed();
        
        // Should have waited at least 400ms (allowing for test timing variance)
        assert!(elapsed >= Duration::from_millis(400));
    }

    #[test]
    fn test_network_error_handler_integration() {
        use crate::http::network_errors::NetworkErrorHandler;
        use crate::http::{HttpError, ErrorClassification};
        
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            failure_window: Duration::from_secs(10),
            recovery_timeout: Duration::from_secs(1),
            success_threshold: 1,
            per_endpoint: true,
            min_request_rate: 1,
        };
        
        let handler = NetworkErrorHandler::new(config);
        
        // Should allow initial requests
        assert!(handler.can_request("test-endpoint").is_ok());
        
        // Record some failures
        let error = HttpError {
            status_code: Some(500),
            classification: ErrorClassification::ServerError,
            provider_code: None,
            message: "Internal server error".to_string(),
            details: None,
            retry_after: None,
        };
        
        // Should be retryable
        assert!(handler.is_retryable(&error, 1));
        
        // Should get retry delay
        let delay = handler.get_retry_delay(&error);
        assert!(delay.is_some());
    }
}