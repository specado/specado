//! Error types for the Specado core library
//!
//! This module defines the comprehensive error handling system for Specado,
//! using thiserror for ergonomic error definitions and anyhow for flexible error contexts.

use std::fmt;
use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Main error type for Specado operations
#[derive(Error, Debug)]
pub enum Error {
    /// Schema validation errors
    #[error("Schema validation failed: {message}")]
    SchemaValidation {
        message: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    /// Translation errors during prompt compilation
    #[error("Translation failed: {message}")]
    Translation {
        message: String,
        context: Option<String>,
    },

    /// Provider-related errors
    #[error("Provider error: {provider} - {message}")]
    Provider {
        provider: String,
        message: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    /// Strictness policy violations
    #[error("Strictness violation: {message} (mode: {mode:?})")]
    StrictnessViolation {
        message: String,
        mode: StrictMode,
        severity: Severity,
    },

    /// JSON parsing and serialization errors
    #[error("JSON error: {message}")]
    Json {
        message: String,
        #[source]
        source: serde_json::Error,
    },

    /// HTTP/Network related errors
    #[error("HTTP error: {message}")]
    Http {
        message: String,
        status_code: Option<u16>,
        #[source]
        source: Option<anyhow::Error>,
    },
    
    /// HTTP error with enhanced diagnostics
    #[error("{}", diagnostics.format_display(true))]
    HttpWithDiagnostics {
        error: crate::http::HttpError,
        diagnostics: crate::http::ErrorDiagnostics,
    },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        #[source]
        source: Option<anyhow::Error>,
    },
    
    /// HTTP request building errors
    #[error("HTTP request error: {message}")]
    HttpRequest {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Validation errors for inputs
    #[error("Validation error: {field} - {message}")]
    Validation {
        field: String,
        message: String,
        expected: Option<String>,
    },

    /// Lossiness-related errors
    #[error("Lossiness error: {code:?} at {path} - {message}")]
    Lossiness {
        code: LossinessCode,
        path: String,
        message: String,
    },

    /// IO errors
    #[error("IO error: {message}")]
    Io {
        message: String,
        #[source]
        source: std::io::Error,
    },

    /// Unsupported feature or operation
    #[error("Unsupported operation: {message}")]
    Unsupported {
        message: String,
        feature: Option<String>,
    },

    /// Timeout errors
    #[error("Operation timed out: {message} (after {timeout_duration:?})")]
    Timeout {
        message: String,
        timeout_duration: std::time::Duration,
    },
    
    /// Rate limiting errors
    #[error("Rate limit exceeded: {message}")]
    RateLimit {
        message: String,
        retry_after: Option<u64>,
    },
    
    /// Circuit breaker errors
    #[error("Circuit breaker is open: {message}")]
    CircuitBreakerOpen {
        message: String,
        retry_after: Option<u64>,
    },
    
    /// TLS/SSL errors
    #[error("TLS error: {message}")]
    Tls {
        message: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    /// Generic internal error with context
    #[error("Internal error: {message}")]
    Internal {
        message: String,
        #[source]
        source: anyhow::Error,
    },
}

/// Convenience type alias for Results using our Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Strictness modes for translation operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrictMode {
    /// Fail on any lossiness
    Strict,
    /// Proceed with warnings
    Warn,
    /// Auto-adjust values to fit constraints
    Coerce,
}

/// Severity levels for errors and warnings
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational, no action required
    Info,
    /// Warning, should be reviewed
    Warning,
    /// Error, operation may fail
    Error,
    /// Critical, operation will fail
    Critical,
}

/// Lossiness codes for translation deviations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LossinessCode {
    /// Value clamped into supported range
    Clamp,
    /// Unsupported field removed
    Drop,
    /// Behavior achieved via non-native mechanism
    Emulate,
    /// Mutually exclusive fields resolved
    Conflict,
    /// Field moved to different location
    Relocate,
    /// Requested capability not available
    Unsupported,
    /// Alternate mapping used
    MapFallback,
    /// Likely quality/latency risk
    PerformanceImpact,
}

impl fmt::Display for StrictMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrictMode::Strict => write!(f, "Strict"),
            StrictMode::Warn => write!(f, "Warn"),
            StrictMode::Coerce => write!(f, "Coerce"),
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
            Severity::Critical => write!(f, "critical"),
        }
    }
}

impl fmt::Display for LossinessCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LossinessCode::Clamp => write!(f, "Clamp"),
            LossinessCode::Drop => write!(f, "Drop"),
            LossinessCode::Emulate => write!(f, "Emulate"),
            LossinessCode::Conflict => write!(f, "Conflict"),
            LossinessCode::Relocate => write!(f, "Relocate"),
            LossinessCode::Unsupported => write!(f, "Unsupported"),
            LossinessCode::MapFallback => write!(f, "MapFallback"),
            LossinessCode::PerformanceImpact => write!(f, "PerformanceImpact"),
        }
    }
}

// Conversion implementations
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json {
            message: err.to_string(),
            source: err,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io {
            message: err.to_string(),
            source: err,
        }
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Internal {
            message: err.to_string(),
            source: err,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::SchemaValidation {
            message: "Invalid schema".to_string(),
            source: None,
        };
        assert_eq!(err.to_string(), "Schema validation failed: Invalid schema");
        
        let timeout_err = Error::Timeout {
            message: "Request took too long".to_string(),
            timeout_duration: std::time::Duration::from_secs(30),
        };
        assert!(timeout_err.to_string().contains("Operation timed out"));
        assert!(timeout_err.to_string().contains("30s"));
        
        let rate_limit_err = Error::RateLimit {
            message: "Too many requests".to_string(),
            retry_after: Some(60),
        };
        assert_eq!(rate_limit_err.to_string(), "Rate limit exceeded: Too many requests");
    }

    #[test]
    fn test_strict_mode_display() {
        assert_eq!(StrictMode::Strict.to_string(), "Strict");
        assert_eq!(StrictMode::Warn.to_string(), "Warn");
        assert_eq!(StrictMode::Coerce.to_string(), "Coerce");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Error < Severity::Critical);
    }
}