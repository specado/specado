//! Fallback strategies for HTTP request failures
//!
//! This module provides automatic recovery mechanisms when provider API calls fail,
//! including retry with jitter, alternative endpoints, and graceful degradation.

use crate::http::{HttpError, ErrorClassification};
use crate::types::{ModelSpec, EndpointConfig};
use serde_json::Value;
use std::time::Duration;
use rand::Rng;
use tracing::{info, debug};

/// Fallback strategy configuration
#[derive(Debug, Clone)]
pub struct FallbackConfig {
    /// Enable automatic fallback attempts
    pub enabled: bool,
    
    /// Maximum number of fallback attempts
    pub max_attempts: u32,
    
    /// Alternative base URLs to try
    pub alternative_urls: Vec<String>,
    
    /// Enable graceful degradation
    pub allow_degradation: bool,
    
    /// Retry configuration with jitter
    pub retry_with_jitter: bool,
    
    /// Maximum jitter in milliseconds
    pub max_jitter_ms: u64,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 3,
            alternative_urls: Vec::new(),
            allow_degradation: true,
            retry_with_jitter: true,
            max_jitter_ms: 1000,
        }
    }
}

/// Represents a fallback attempt result
#[derive(Debug)]
pub struct FallbackAttempt {
    pub strategy: String,
    pub success: bool,
    pub error: Option<String>,
    pub duration: Duration,
}

/// Fallback handler for HTTP requests
pub struct FallbackHandler {
    pub config: FallbackConfig,
    attempts: Vec<FallbackAttempt>,
}

impl FallbackHandler {
    /// Create a new fallback handler
    pub fn new(config: FallbackConfig) -> Self {
        Self {
            config,
            attempts: Vec::new(),
        }
    }
    
    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(FallbackConfig::default())
    }
    
    /// Record a fallback attempt
    pub fn record_attempt(&mut self, attempt: FallbackAttempt) {
        info!(
            "Fallback attempt {}: {} - {}",
            self.attempts.len() + 1,
            attempt.strategy,
            if attempt.success { "success" } else { "failed" }
        );
        self.attempts.push(attempt);
    }
    
    /// Get all fallback attempts
    pub fn attempts(&self) -> &[FallbackAttempt] {
        &self.attempts
    }
    
    /// Calculate retry delay with jitter
    pub fn calculate_retry_delay(&self, attempt: u32) -> Duration {
        let base_delay = Duration::from_millis(100 * 2_u64.pow(attempt));
        
        if self.config.retry_with_jitter {
            let jitter = rand::thread_rng().gen_range(0..=self.config.max_jitter_ms);
            base_delay + Duration::from_millis(jitter)
        } else {
            base_delay
        }
    }
    
    /// Determine if we should retry based on error classification
    pub fn should_retry(&self, error: &HttpError, attempt: u32) -> bool {
        if !self.config.enabled || attempt >= self.config.max_attempts {
            return false;
        }
        
        match error.classification() {
            ErrorClassification::NetworkError => true,
            ErrorClassification::ServerError => true,
            ErrorClassification::RateLimitError => true,
            ErrorClassification::ClientError => false, // Don't retry bad requests
            ErrorClassification::AuthenticationError => false, // Don't retry auth errors
            ErrorClassification::Unknown => false, // Don't retry unknown errors by default
        }
    }
    
    /// Try alternative base URL
    pub fn get_alternative_url(&self, attempt: usize) -> Option<&String> {
        if attempt < self.config.alternative_urls.len() {
            Some(&self.config.alternative_urls[attempt])
        } else {
            None
        }
    }
    
    /// Apply graceful degradation to request
    pub fn apply_degradation(&self, mut request: Value, degradation_level: u32) -> Value {
        if !self.config.allow_degradation {
            return request;
        }
        
        match degradation_level {
            1 => {
                // Reduce token limits
                if let Some(max_tokens) = request.get_mut("max_tokens") {
                    if let Some(tokens) = max_tokens.as_u64() {
                        *max_tokens = Value::from(tokens / 2);
                        info!("Degradation: Reduced max_tokens to {}", tokens / 2);
                    }
                }
            }
            2 => {
                // Disable streaming if enabled
                if request.get("stream").and_then(|v| v.as_bool()).unwrap_or(false) {
                    request["stream"] = Value::Bool(false);
                    info!("Degradation: Disabled streaming");
                }
            }
            3 => {
                // Remove optional features like tools
                if request.get("tools").is_some() {
                    request.as_object_mut().unwrap().remove("tools");
                    request.as_object_mut().unwrap().remove("tool_choice");
                    info!("Degradation: Removed tool usage");
                }
            }
            _ => {}
        }
        
        request
    }
    
    /// Create a fallback endpoint configuration
    pub fn create_fallback_endpoint(
        &self,
        original: &EndpointConfig,
        alternative_url: Option<&str>,
    ) -> EndpointConfig {
        let mut fallback = original.clone();
        
        // Update headers if we're using an alternative URL
        if let Some(alt_url) = alternative_url {
            debug!("Using alternative URL: {}", alt_url);
            // The alternative URL will be handled at the client level
        }
        
        fallback
    }
}

/// Trait for types that can provide fallback strategies
pub trait FallbackProvider {
    /// Get fallback configuration for a model
    fn fallback_config(&self, model: &ModelSpec) -> FallbackConfig;
    
    /// Get alternative models to try
    fn alternative_models(&self, model: &ModelSpec) -> Vec<String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_retry_delay_with_jitter() {
        let handler = FallbackHandler::default();
        
        // Test that delay increases exponentially
        let delay1 = handler.calculate_retry_delay(0);
        let delay2 = handler.calculate_retry_delay(1);
        let delay3 = handler.calculate_retry_delay(2);
        
        assert!(delay1 < delay2);
        assert!(delay2 < delay3);
        
        // Test that jitter is applied
        let delay_a = handler.calculate_retry_delay(1);
        let delay_b = handler.calculate_retry_delay(1);
        // With jitter, same attempt should give different delays
        // (This might occasionally fail due to random chance, but very unlikely)
        assert_ne!(delay_a, delay_b);
    }
    
    #[test]
    fn test_should_retry() {
        let handler = FallbackHandler::default();
        
        // Network errors should be retried
        let network_error = HttpError {
            status_code: None,
            classification: ErrorClassification::NetworkError,
            provider_code: None,
            message: "Connection failed".to_string(),
            details: None,
            retry_after: None,
        };
        assert!(handler.should_retry(&network_error, 0));
        
        // Client errors should not be retried
        let client_error = HttpError {
            status_code: Some(400),
            classification: ErrorClassification::ClientError,
            provider_code: None,
            message: "Invalid request".to_string(),
            details: None,
            retry_after: None,
        };
        assert!(!handler.should_retry(&client_error, 0));
        
        // Should not retry after max attempts
        assert!(!handler.should_retry(&network_error, 5));
    }
    
    #[test]
    fn test_apply_degradation() {
        let handler = FallbackHandler::default();
        
        let mut request = serde_json::json!({
            "model": "gpt-4",
            "messages": [{"role": "user", "content": "Hello"}],
            "max_tokens": 1000,
            "stream": true,
            "tools": [{"name": "test_tool"}]
        });
        
        // Level 1: Reduce tokens
        request = handler.apply_degradation(request, 1);
        assert_eq!(request["max_tokens"], 500);
        
        // Level 2: Disable streaming
        request = handler.apply_degradation(request, 2);
        assert_eq!(request["stream"], false);
        
        // Level 3: Remove tools
        request = handler.apply_degradation(request, 3);
        assert!(request.get("tools").is_none());
    }
}