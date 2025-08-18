//! HTTP error classification and normalization
//!
//! Normalizes provider-specific error responses into a uniform error format

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// Classification of HTTP errors for retry logic
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorClassification {
    /// Client errors (4xx) - should not retry
    ClientError,
    /// Server errors (5xx) - should retry
    ServerError,
    /// Network errors - should retry
    NetworkError,
    /// Rate limiting - should retry with backoff
    RateLimitError,
    /// Authentication errors - should not retry
    AuthenticationError,
    /// Unknown errors - default to no retry
    Unknown,
}

impl ErrorClassification {
    /// Check if this error type should be retried
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ErrorClassification::ServerError
                | ErrorClassification::NetworkError
                | ErrorClassification::RateLimitError
        )
    }
    
    /// Get recommended retry delay in seconds
    pub fn retry_delay_hint(&self) -> Option<u64> {
        match self {
            ErrorClassification::RateLimitError => Some(60), // 1 minute for rate limits
            ErrorClassification::ServerError => Some(5),     // 5 seconds for server errors
            ErrorClassification::NetworkError => Some(2),    // 2 seconds for network errors
            _ => None,
        }
    }
}

/// Normalized HTTP error representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpError {
    /// HTTP status code if available
    pub status_code: Option<u16>,
    /// Error classification for retry logic
    pub classification: ErrorClassification,
    /// Provider-specific error code
    pub provider_code: Option<String>,
    /// Human-readable error message
    pub message: String,
    /// Additional error details
    pub details: Option<Value>,
    /// Retry-After header value if present
    pub retry_after: Option<u64>,
}

impl HttpError {
    /// Create from a reqwest Response
    pub async fn from_response(response: reqwest::Response) -> Self {
        let status = response.status();
        let status_code = Some(status.as_u16());
        
        // Check for Retry-After header
        let retry_after = response
            .headers()
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
        
        // Try to parse response body
        let body = response.text().await.unwrap_or_default();
        let details = serde_json::from_str::<Value>(&body).ok();
        
        // Extract provider-specific error information
        let (provider_code, message) = Self::extract_provider_error(&details, &body);
        
        // Classify the error
        let classification = Self::classify_status(status);
        
        Self {
            status_code,
            classification,
            provider_code,
            message,
            details,
            retry_after,
        }
    }
    
    /// Create from a network/request error
    pub fn from_request_error(error: reqwest::Error) -> Self {
        let classification = if error.is_timeout() {
            ErrorClassification::NetworkError
        } else if error.is_connect() {
            ErrorClassification::NetworkError
        } else {
            ErrorClassification::Unknown
        };
        
        Self {
            status_code: None,
            classification,
            provider_code: None,
            message: error.to_string(),
            details: None,
            retry_after: None,
        }
    }
    
    /// Classify HTTP status code
    fn classify_status(status: StatusCode) -> ErrorClassification {
        match status.as_u16() {
            401 | 403 => ErrorClassification::AuthenticationError,
            429 => ErrorClassification::RateLimitError,
            400..=499 => ErrorClassification::ClientError,
            500..=599 => ErrorClassification::ServerError,
            _ => ErrorClassification::Unknown,
        }
    }
    
    /// Extract provider-specific error information
    fn extract_provider_error(details: &Option<Value>, body: &str) -> (Option<String>, String) {
        if let Some(json) = details {
            // Try OpenAI error format
            if let Some(error) = json.get("error") {
                let code = error.get("code")
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string());
                let message = error.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or(body)
                    .to_string();
                return (code, message);
            }
            
            // Try Anthropic error format
            if let Some(error_type) = json.get("type") {
                let code = error_type.as_str().map(|s| s.to_string());
                let message = json.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or(body)
                    .to_string();
                return (code, message);
            }
            
            // Generic JSON error format
            if let Some(message) = json.get("message").and_then(|m| m.as_str()) {
                return (None, message.to_string());
            }
        }
        
        // Fallback to raw body
        (None, body.to_string())
    }
    
    /// Check if this error should trigger a retry
    pub fn should_retry(&self) -> bool {
        self.classification.is_retryable()
    }
    
    /// Get the delay before retry (in seconds)
    pub fn get_retry_delay(&self) -> Option<u64> {
        // Prefer Retry-After header if present
        self.retry_after.or_else(|| self.classification.retry_delay_hint())
    }
    
    /// Get the error classification
    pub fn classification(&self) -> ErrorClassification {
        self.classification
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HTTP Error [{}]: {} (classification: {:?})",
            self.status_code
                .map(|c| c.to_string())
                .unwrap_or_else(|| "N/A".to_string()),
            self.message,
            self.classification
        )
    }
}

impl std::error::Error for HttpError {}

/// Convert HttpError to crate Error
impl From<HttpError> for crate::Error {
    fn from(http_error: HttpError) -> Self {
        crate::Error::Http {
            message: http_error.message.clone(),
            status_code: http_error.status_code,
            source: Some(anyhow::anyhow!("{:?}", http_error.details)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_classification() {
        assert!(ErrorClassification::ServerError.is_retryable());
        assert!(ErrorClassification::NetworkError.is_retryable());
        assert!(ErrorClassification::RateLimitError.is_retryable());
        assert!(!ErrorClassification::ClientError.is_retryable());
        assert!(!ErrorClassification::AuthenticationError.is_retryable());
    }
    
    #[test]
    fn test_status_classification() {
        assert_eq!(
            HttpError::classify_status(StatusCode::UNAUTHORIZED),
            ErrorClassification::AuthenticationError
        );
        assert_eq!(
            HttpError::classify_status(StatusCode::TOO_MANY_REQUESTS),
            ErrorClassification::RateLimitError
        );
        assert_eq!(
            HttpError::classify_status(StatusCode::BAD_REQUEST),
            ErrorClassification::ClientError
        );
        assert_eq!(
            HttpError::classify_status(StatusCode::INTERNAL_SERVER_ERROR),
            ErrorClassification::ServerError
        );
    }
    
    #[test]
    fn test_retry_delay_hints() {
        assert_eq!(ErrorClassification::RateLimitError.retry_delay_hint(), Some(60));
        assert_eq!(ErrorClassification::ServerError.retry_delay_hint(), Some(5));
        assert_eq!(ErrorClassification::NetworkError.retry_delay_hint(), Some(2));
        assert_eq!(ErrorClassification::ClientError.retry_delay_hint(), None);
    }
    
    #[test]
    fn test_openai_error_extraction() {
        let json = serde_json::json!({
            "error": {
                "code": "rate_limit_exceeded",
                "message": "You have exceeded your rate limit"
            }
        });
        
        let (code, message) = HttpError::extract_provider_error(&Some(json), "raw body");
        assert_eq!(code, Some("rate_limit_exceeded".to_string()));
        assert_eq!(message, "You have exceeded your rate limit");
    }
    
    #[test]
    fn test_anthropic_error_extraction() {
        let json = serde_json::json!({
            "type": "invalid_request_error",
            "message": "Invalid request format"
        });
        
        let (code, message) = HttpError::extract_provider_error(&Some(json), "raw body");
        assert_eq!(code, Some("invalid_request_error".to_string()));
        assert_eq!(message, "Invalid request format");
    }
}