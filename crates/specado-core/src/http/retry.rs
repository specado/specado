//! Retry logic with exponential backoff for HTTP requests
//!
//! Implements intelligent retry strategies for transient failures

use std::time::Duration;
use backoff::{ExponentialBackoff, backoff::Backoff};
use crate::http::error::{HttpError, ErrorClassification};

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base delay for exponential backoff (in seconds)
    pub base_delay_secs: u64,
    /// Maximum delay between retries (in seconds)
    pub max_delay_secs: u64,
    /// Whether to add jitter to prevent thundering herd
    pub jitter: bool,
    /// Multiplier for exponential backoff
    pub multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_secs: 1,
            max_delay_secs: 30,
            jitter: true,
            multiplier: 2.0,
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy with custom settings
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            ..Default::default()
        }
    }
    
    /// Set the base delay
    pub fn with_base_delay(mut self, seconds: u64) -> Self {
        self.base_delay_secs = seconds;
        self
    }
    
    /// Set the maximum delay
    pub fn with_max_delay(mut self, seconds: u64) -> Self {
        self.max_delay_secs = seconds;
        self
    }
    
    /// Enable or disable jitter
    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }
    
    /// Create an exponential backoff instance
    pub fn create_backoff(&self) -> ExponentialBackoff {
        let mut backoff = ExponentialBackoff {
            initial_interval: Duration::from_secs(self.base_delay_secs),
            max_interval: Duration::from_secs(self.max_delay_secs),
            multiplier: self.multiplier,
            max_elapsed_time: None, // We handle max attempts separately
            ..Default::default()
        };
        
        if !self.jitter {
            backoff.randomization_factor = 0.0;
        }
        
        backoff
    }
}

/// Decision on whether to retry a request
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryDecision {
    /// Retry the request after the specified delay
    Retry { delay: Duration },
    /// Do not retry the request
    NoRetry,
}

/// Retry handler for HTTP requests
#[derive(Debug)]
pub struct RetryHandler {
    policy: RetryPolicy,
    attempts: u32,
    backoff: ExponentialBackoff,
}

impl RetryHandler {
    /// Create a new retry handler with the given policy
    pub fn new(policy: RetryPolicy) -> Self {
        let backoff = policy.create_backoff();
        Self {
            policy,
            attempts: 0,
            backoff,
        }
    }
    
    /// Create with default policy
    pub fn default() -> Self {
        Self::new(RetryPolicy::default())
    }
    
    /// Determine if a request should be retried based on the error
    pub fn should_retry(&mut self, error: &HttpError) -> RetryDecision {
        // Check if we've exceeded max attempts
        if self.attempts >= self.policy.max_attempts {
            return RetryDecision::NoRetry;
        }
        
        // Check if the error is retryable
        if !error.should_retry() {
            return RetryDecision::NoRetry;
        }
        
        // Increment attempt counter
        self.attempts += 1;
        
        // Calculate delay
        let delay = self.calculate_delay(error);
        
        RetryDecision::Retry { delay }
    }
    
    /// Calculate the delay before the next retry
    fn calculate_delay(&mut self, error: &HttpError) -> Duration {
        // If the error has a Retry-After header, use that
        if let Some(retry_after_secs) = error.get_retry_delay() {
            return Duration::from_secs(retry_after_secs);
        }
        
        // Otherwise, use exponential backoff
        self.backoff.next_backoff()
            .unwrap_or(Duration::from_secs(self.policy.max_delay_secs))
    }
    
    /// Reset the retry handler for a new request
    pub fn reset(&mut self) {
        self.attempts = 0;
        self.backoff.reset();
    }
    
    /// Get the number of attempts made so far
    pub fn attempts(&self) -> u32 {
        self.attempts
    }
}

/// Determine if an error classification should be retried
pub fn should_retry_classification(classification: ErrorClassification) -> bool {
    classification.is_retryable()
}

/// Execute a request with retry logic
pub async fn execute_with_retry<F, Fut, T>(
    mut request_fn: F,
    policy: RetryPolicy,
) -> Result<T, HttpError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, HttpError>>,
{
    let mut handler = RetryHandler::new(policy);
    
    loop {
        match request_fn().await {
            Ok(response) => return Ok(response),
            Err(error) => {
                match handler.should_retry(&error) {
                    RetryDecision::Retry { delay } => {
                        log::warn!(
                            "Request failed (attempt {}), retrying after {:?}: {}",
                            handler.attempts(),
                            delay,
                            error
                        );
                        tokio::time::sleep(delay).await;
                    }
                    RetryDecision::NoRetry => {
                        log::error!(
                            "Request failed after {} attempts, not retrying: {}",
                            handler.attempts(),
                            error
                        );
                        return Err(error);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::error::ErrorClassification;
    
    #[test]
    fn test_default_retry_policy() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.base_delay_secs, 1);
        assert_eq!(policy.max_delay_secs, 30);
        assert!(policy.jitter);
    }
    
    #[test]
    fn test_retry_handler_max_attempts() {
        let policy = RetryPolicy::new(2);
        let mut handler = RetryHandler::new(policy);
        
        let error = HttpError {
            status_code: Some(500),
            classification: ErrorClassification::ServerError,
            provider_code: None,
            message: "Server error".to_string(),
            details: None,
            retry_after: None,
        };
        
        // First retry should be allowed
        assert!(matches!(
            handler.should_retry(&error),
            RetryDecision::Retry { .. }
        ));
        assert_eq!(handler.attempts(), 1);
        
        // Second retry should be allowed
        assert!(matches!(
            handler.should_retry(&error),
            RetryDecision::Retry { .. }
        ));
        assert_eq!(handler.attempts(), 2);
        
        // Third retry should not be allowed (max attempts reached)
        assert_eq!(handler.should_retry(&error), RetryDecision::NoRetry);
    }
    
    #[test]
    fn test_non_retryable_errors() {
        let mut handler = RetryHandler::default();
        
        // Client error should not be retried
        let client_error = HttpError {
            status_code: Some(400),
            classification: ErrorClassification::ClientError,
            provider_code: None,
            message: "Bad request".to_string(),
            details: None,
            retry_after: None,
        };
        
        assert_eq!(handler.should_retry(&client_error), RetryDecision::NoRetry);
        
        // Authentication error should not be retried
        let auth_error = HttpError {
            status_code: Some(401),
            classification: ErrorClassification::AuthenticationError,
            provider_code: None,
            message: "Unauthorized".to_string(),
            details: None,
            retry_after: None,
        };
        
        assert_eq!(handler.should_retry(&auth_error), RetryDecision::NoRetry);
    }
    
    #[test]
    fn test_retry_after_header() {
        let mut handler = RetryHandler::default();
        
        let error = HttpError {
            status_code: Some(429),
            classification: ErrorClassification::RateLimitError,
            provider_code: None,
            message: "Rate limited".to_string(),
            details: None,
            retry_after: Some(10), // 10 seconds
        };
        
        if let RetryDecision::Retry { delay } = handler.should_retry(&error) {
            assert_eq!(delay.as_secs(), 10);
        } else {
            panic!("Expected retry decision");
        }
    }
    
    #[test]
    fn test_exponential_backoff() {
        let policy = RetryPolicy::default().with_jitter(false);
        let mut handler = RetryHandler::new(policy);
        
        let error = HttpError {
            status_code: Some(500),
            classification: ErrorClassification::ServerError,
            provider_code: None,
            message: "Server error".to_string(),
            details: None,
            retry_after: None,
        };
        
        // First retry should have base delay
        if let RetryDecision::Retry { delay } = handler.should_retry(&error) {
            assert!(delay.as_secs() >= 1);
        }
        
        // Second retry should have increased delay
        if let RetryDecision::Retry { delay } = handler.should_retry(&error) {
            assert!(delay.as_secs() >= 2);
        }
    }
}