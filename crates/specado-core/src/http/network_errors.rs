//! Network error handling with circuit breaker pattern
//!
//! Provides comprehensive network error handling including automatic retry
//! for transient errors and circuit breaker to prevent cascading failures

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::http::error::{HttpError, ErrorClassification};

/// Circuit breaker configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    pub failure_threshold: u32,
    /// Time window for counting failures
    pub failure_window: Duration,
    /// Time to wait before attempting to close the circuit
    pub recovery_timeout: Duration,
    /// Number of successful requests needed to close the circuit
    pub success_threshold: u32,
    /// Enable per-endpoint circuit breaking
    pub per_endpoint: bool,
    /// Minimum request rate to activate circuit breaker (requests per minute)
    pub min_request_rate: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,                         // 5 failures
            failure_window: Duration::from_secs(60),      // in 1 minute
            recovery_timeout: Duration::from_secs(30),    // wait 30 seconds
            success_threshold: 3,                         // 3 successes to close
            per_endpoint: true,
            min_request_rate: 10, // 10 requests per minute minimum
        }
    }
}

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, allowing all requests
    Closed,
    /// Circuit is open, rejecting all requests
    Open,
    /// Circuit is half-open, allowing limited requests to test recovery
    HalfOpen,
}

/// Circuit breaker implementation
#[derive(Debug)]
struct CircuitBreaker {
    /// Current state
    state: CircuitState,
    /// Number of consecutive failures
    failure_count: u32,
    /// Number of consecutive successes (in half-open state)
    success_count: u32,
    /// Time when the circuit was opened
    opened_at: Option<Instant>,
    /// Configuration
    config: CircuitBreakerConfig,
    /// Request timestamps for rate limiting
    request_times: Vec<Instant>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            opened_at: None,
            config,
            request_times: Vec::new(),
        }
    }
    
    /// Check if a request should be allowed
    fn can_request(&mut self) -> Result<(), NetworkError> {
        self.cleanup_old_requests();
        
        match self.state {
            CircuitState::Closed => {
                // Always allow requests when closed
                self.record_request();
                Ok(())
            }
            CircuitState::Open => {
                // Check if we should transition to half-open
                if let Some(opened_at) = self.opened_at {
                    if opened_at.elapsed() >= self.config.recovery_timeout {
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                        self.record_request();
                        Ok(())
                    } else {
                        Err(NetworkError::CircuitBreakerOpen {
                            retry_after: self.config.recovery_timeout - opened_at.elapsed(),
                        })
                    }
                } else {
                    // This shouldn't happen, but handle gracefully
                    self.state = CircuitState::HalfOpen;
                    self.record_request();
                    Ok(())
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests to test recovery
                self.record_request();
                Ok(())
            }
        }
    }
    
    /// Record a successful request
    fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                // Reset failure count
                self.failure_count = 0;
            }
            CircuitState::Open => {
                // Should not receive success in open state
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    // Close the circuit
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                    self.opened_at = None;
                }
            }
        }
    }
    
    /// Record a failed request
    fn record_failure(&mut self) {
        // Only count failures if we have sufficient request rate
        if !self.has_sufficient_request_rate() {
            return;
        }
        
        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.config.failure_threshold {
                    // Open the circuit
                    self.state = CircuitState::Open;
                    self.opened_at = Some(Instant::now());
                }
            }
            CircuitState::Open => {
                // Already open, no action needed
            }
            CircuitState::HalfOpen => {
                // Failure during half-open, return to open
                self.state = CircuitState::Open;
                self.failure_count += 1;
                self.success_count = 0;
                self.opened_at = Some(Instant::now());
            }
        }
    }
    
    /// Check if there's sufficient request rate to activate circuit breaker
    fn has_sufficient_request_rate(&self) -> bool {
        let min_requests = self.config.min_request_rate;
        let requests_in_window = self.request_times.len() as u32;
        requests_in_window >= min_requests
    }
    
    /// Record a request timestamp
    fn record_request(&mut self) {
        self.request_times.push(Instant::now());
    }
    
    /// Clean up old request timestamps
    fn cleanup_old_requests(&mut self) {
        let cutoff = Instant::now() - Duration::from_secs(60); // Keep 1 minute of history
        self.request_times.retain(|&time| time > cutoff);
    }
    
    /// Get current circuit state
    #[allow(dead_code)]
    fn get_state(&self) -> CircuitState {
        self.state
    }
    
    /// Get circuit breaker statistics
    fn get_stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            state: self.state,
            failure_count: self.failure_count,
            success_count: self.success_count,
            request_count: self.request_times.len() as u32,
            opened_at: self.opened_at,
        }
    }
}

/// Network error handler with circuit breaker
#[derive(Debug)]
pub struct NetworkErrorHandler {
    /// Configuration
    config: CircuitBreakerConfig,
    /// Circuit breakers per endpoint
    circuits: Arc<Mutex<HashMap<String, CircuitBreaker>>>,
    /// Global circuit breaker (when not per-endpoint)
    global_circuit: Arc<Mutex<CircuitBreaker>>,
}

impl NetworkErrorHandler {
    /// Create a new network error handler
    pub fn new(config: CircuitBreakerConfig) -> Self {
        let global_circuit = Arc::new(Mutex::new(CircuitBreaker::new(config.clone())));
        
        Self {
            config,
            circuits: Arc::new(Mutex::new(HashMap::new())),
            global_circuit,
        }
    }
    
    /// Check if a request should be allowed
    pub fn can_request(&self, endpoint: &str) -> Result<(), NetworkError> {
        if self.config.per_endpoint {
            let mut circuits = self.circuits.lock().unwrap();
            let circuit = circuits
                .entry(endpoint.to_string())
                .or_insert_with(|| CircuitBreaker::new(self.config.clone()));
            circuit.can_request()
        } else {
            let mut circuit = self.global_circuit.lock().unwrap();
            circuit.can_request()
        }
    }
    
    /// Record a successful request
    pub fn record_success(&self, endpoint: &str) {
        if self.config.per_endpoint {
            let mut circuits = self.circuits.lock().unwrap();
            if let Some(circuit) = circuits.get_mut(endpoint) {
                circuit.record_success();
            }
        } else {
            let mut circuit = self.global_circuit.lock().unwrap();
            circuit.record_success();
        }
    }
    
    /// Record a failed request
    pub fn record_failure(&self, endpoint: &str, error: &HttpError) {
        // Only count certain types of failures for circuit breaker
        if !self.should_count_failure(error) {
            return;
        }
        
        if self.config.per_endpoint {
            let mut circuits = self.circuits.lock().unwrap();
            let circuit = circuits
                .entry(endpoint.to_string())
                .or_insert_with(|| CircuitBreaker::new(self.config.clone()));
            circuit.record_failure();
        } else {
            let mut circuit = self.global_circuit.lock().unwrap();
            circuit.record_failure();
        }
    }
    
    /// Check if an error should count towards circuit breaker failure threshold
    fn should_count_failure(&self, error: &HttpError) -> bool {
        match error.classification {
            // Count server errors and network errors
            ErrorClassification::ServerError |
            ErrorClassification::NetworkError |
            ErrorClassification::TimeoutError |
            ErrorClassification::ConnectionError |
            ErrorClassification::DnsError => true,
            
            // Don't count client errors or auth errors
            ErrorClassification::ClientError |
            ErrorClassification::AuthenticationError |
            ErrorClassification::RateLimitError => false,
            
            // Don't count TLS errors (might be temporary)
            ErrorClassification::TlsError => false,
            
            // Count unknown errors to be safe
            ErrorClassification::Unknown => true,
            
            // Don't count circuit breaker errors
            ErrorClassification::CircuitBreakerOpen => false,
        }
    }
    
    /// Get retry delay for a network error
    pub fn get_retry_delay(&self, error: &HttpError) -> Option<Duration> {
        match error.classification {
            ErrorClassification::TimeoutError => Some(Duration::from_secs(2)),
            ErrorClassification::ConnectionError => Some(Duration::from_secs(5)),
            ErrorClassification::DnsError => Some(Duration::from_secs(10)),
            ErrorClassification::NetworkError => Some(Duration::from_secs(3)),
            ErrorClassification::TlsError => Some(Duration::from_secs(30)),
            ErrorClassification::ServerError => Some(Duration::from_secs(5)),
            ErrorClassification::RateLimitError => Some(Duration::from_secs(60)),
            _ => None,
        }
    }
    
    /// Check if an error is retryable
    pub fn is_retryable(&self, error: &HttpError, attempt: u32) -> bool {
        // Don't retry if circuit breaker is open
        if error.classification == ErrorClassification::CircuitBreakerOpen {
            return false;
        }
        
        // Don't retry too many times
        if attempt >= 5 {
            return false;
        }
        
        // Check if error type is retryable
        error.classification.is_retryable()
    }
    
    /// Get circuit breaker statistics
    pub fn get_circuit_stats(&self) -> HashMap<String, CircuitBreakerStats> {
        let mut stats = HashMap::new();
        
        if self.config.per_endpoint {
            let circuits = self.circuits.lock().unwrap();
            for (endpoint, circuit) in circuits.iter() {
                stats.insert(endpoint.clone(), circuit.get_stats());
            }
        } else {
            let circuit = self.global_circuit.lock().unwrap();
            stats.insert("global".to_string(), circuit.get_stats());
        }
        
        stats
    }
}

/// Network error types
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Circuit breaker is open, retry after {retry_after:?}")]
    CircuitBreakerOpen { retry_after: Duration },
    
    #[error("Network operation failed: {message}")]
    OperationFailed { message: String },
    
    #[error("Timeout occurred after {timeout:?}")]
    Timeout { timeout: Duration },
    
    #[error("Connection failed: {message}")]
    ConnectionFailed { message: String },
    
    #[error("DNS resolution failed: {message}")]
    DnsResolutionFailed { message: String },
    
    #[error("TLS handshake failed: {message}")]
    TlsHandshakeFailed { message: String },
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    /// Current state
    pub state: CircuitState,
    /// Current failure count
    pub failure_count: u32,
    /// Current success count (in half-open state)
    pub success_count: u32,
    /// Number of requests in the current window
    pub request_count: u32,
    /// When the circuit was opened (if applicable)
    pub opened_at: Option<Instant>,
}

/// Helper function to create network error from HTTP error
pub fn create_network_error(http_error: &HttpError) -> NetworkError {
    match http_error.classification {
        ErrorClassification::TimeoutError => NetworkError::Timeout {
            timeout: Duration::from_secs(30), // Default timeout
        },
        ErrorClassification::ConnectionError => NetworkError::ConnectionFailed {
            message: http_error.message.clone(),
        },
        ErrorClassification::DnsError => NetworkError::DnsResolutionFailed {
            message: http_error.message.clone(),
        },
        ErrorClassification::TlsError => NetworkError::TlsHandshakeFailed {
            message: http_error.message.clone(),
        },
        _ => NetworkError::OperationFailed {
            message: http_error.message.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_circuit_breaker_config_default() {
        let config = CircuitBreakerConfig::default();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.failure_window, Duration::from_secs(60));
        assert_eq!(config.recovery_timeout, Duration::from_secs(30));
        assert_eq!(config.success_threshold, 3);
        assert!(config.per_endpoint);
    }
    
    #[test]
    fn test_circuit_breaker_closed_state() {
        let config = CircuitBreakerConfig::default();
        let mut cb = CircuitBreaker::new(config);
        
        // Should allow requests in closed state
        assert!(cb.can_request().is_ok());
        assert_eq!(cb.get_state(), CircuitState::Closed);
    }
    
    #[test]
    fn test_circuit_breaker_opening() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            min_request_rate: 1, // Allow with minimal requests
            ..CircuitBreakerConfig::default()
        };
        
        let mut cb = CircuitBreaker::new(config);
        
        // Add some requests to meet minimum rate
        cb.record_request();
        cb.record_request();
        cb.record_request();
        
        // Record failures
        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Closed);
        
        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Open);
        
        // Should reject requests in open state
        assert!(cb.can_request().is_err());
    }
    
    #[test]
    fn test_circuit_breaker_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 2,
            recovery_timeout: Duration::from_millis(100),
            min_request_rate: 1,
            ..CircuitBreakerConfig::default()
        };
        
        let mut cb = CircuitBreaker::new(config);
        
        // Force minimum request rate
        cb.record_request();
        cb.record_request();
        
        // Open the circuit
        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Open);
        
        // Wait for recovery timeout
        thread::sleep(Duration::from_millis(150));
        
        // Should transition to half-open
        assert!(cb.can_request().is_ok());
        assert_eq!(cb.get_state(), CircuitState::HalfOpen);
        
        // Record successes to close
        cb.record_success();
        cb.record_success();
        assert_eq!(cb.get_state(), CircuitState::Closed);
    }
    
    #[test]
    fn test_network_error_handler() {
        let config = CircuitBreakerConfig::default();
        let handler = NetworkErrorHandler::new(config);
        
        // Should allow initial requests
        assert!(handler.can_request("test-endpoint").is_ok());
        
        // Record success
        handler.record_success("test-endpoint");
        
        // Create a test error
        let error = HttpError {
            status_code: Some(500),
            classification: ErrorClassification::ServerError,
            provider_code: None,
            message: "Internal server error".to_string(),
            details: None,
            retry_after: None,
        };
        
        // Should count server errors
        assert!(handler.should_count_failure(&error));
        
        // Should not count client errors
        let client_error = HttpError {
            status_code: Some(400),
            classification: ErrorClassification::ClientError,
            provider_code: None,
            message: "Bad request".to_string(),
            details: None,
            retry_after: None,
        };
        assert!(!handler.should_count_failure(&client_error));
    }
    
    #[test]
    fn test_retry_logic() {
        let config = CircuitBreakerConfig::default();
        let handler = NetworkErrorHandler::new(config);
        
        let timeout_error = HttpError {
            status_code: None,
            classification: ErrorClassification::TimeoutError,
            provider_code: None,
            message: "Request timeout".to_string(),
            details: None,
            retry_after: None,
        };
        
        // Should be retryable for early attempts
        assert!(handler.is_retryable(&timeout_error, 1));
        assert!(handler.is_retryable(&timeout_error, 3));
        
        // Should not be retryable after too many attempts
        assert!(!handler.is_retryable(&timeout_error, 6));
        
        // Should get retry delay
        let delay = handler.get_retry_delay(&timeout_error);
        assert!(delay.is_some());
        assert_eq!(delay.unwrap(), Duration::from_secs(2));
    }
    
    #[test]
    fn test_minimum_request_rate() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            min_request_rate: 5,
            ..CircuitBreakerConfig::default()
        };
        
        let mut cb = CircuitBreaker::new(config);
        
        // Record failure without enough requests
        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Closed); // Should stay closed
        
        // Add enough requests
        for _ in 0..5 {
            cb.record_request();
        }
        
        // Now failure should open circuit
        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Open);
    }
}