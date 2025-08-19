//! Error handling for Node.js bindings
//!
//! This module provides error mapping between Rust errors and JavaScript exceptions.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::fmt;

/// Specado error types for JavaScript
#[napi]
pub enum SpecadoErrorKind {
    /// Invalid input parameters
    InvalidInput,
    /// JSON parsing error
    JsonError,
    /// Provider not found
    ProviderNotFound,
    /// Model not found
    ModelNotFound,
    /// Network error during operation
    NetworkError,
    /// Authentication failure
    AuthenticationError,
    /// Rate limit exceeded
    RateLimitError,
    /// Timeout occurred
    TimeoutError,
    /// Internal error
    InternalError,
    /// Unknown error
    Unknown,
}

/// Specado error for JavaScript consumption
#[napi(object)]
#[derive(Debug, Clone)]
pub struct SpecadoError {
    /// Error kind/type
    pub kind: SpecadoErrorKind,
    /// Human-readable error message
    pub message: String,
    /// Optional error details
    pub details: Option<String>,
    /// Error code for programmatic handling
    pub code: String,
}

impl SpecadoError {
    /// Create a new SpecadoError
    pub fn new(kind: SpecadoErrorKind, message: impl Into<String>) -> Self {
        let message = message.into();
        let code = match kind {
            SpecadoErrorKind::InvalidInput => "INVALID_INPUT",
            SpecadoErrorKind::JsonError => "JSON_ERROR",
            SpecadoErrorKind::ProviderNotFound => "PROVIDER_NOT_FOUND",
            SpecadoErrorKind::ModelNotFound => "MODEL_NOT_FOUND",
            SpecadoErrorKind::NetworkError => "NETWORK_ERROR",
            SpecadoErrorKind::AuthenticationError => "AUTHENTICATION_ERROR",
            SpecadoErrorKind::RateLimitError => "RATE_LIMIT_ERROR",
            SpecadoErrorKind::TimeoutError => "TIMEOUT_ERROR",
            SpecadoErrorKind::InternalError => "INTERNAL_ERROR",
            SpecadoErrorKind::Unknown => "UNKNOWN_ERROR",
        }.to_string();

        Self {
            kind,
            message,
            details: None,
            code,
        }
    }

    /// Create a new SpecadoError with details
    pub fn with_details(kind: SpecadoErrorKind, message: impl Into<String>, details: impl Into<String>) -> Self {
        let mut error = Self::new(kind, message);
        error.details = Some(details.into());
        error
    }
}

impl fmt::Display for SpecadoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(details) = &self.details {
            write!(f, " - {}", details)?;
        }
        Ok(())
    }
}

impl std::error::Error for SpecadoError {}

/// Convert FFI result codes to JavaScript errors
impl From<specado_ffi::types::SpecadoResult> for SpecadoError {
    fn from(result: specado_ffi::types::SpecadoResult) -> Self {
        use specado_ffi::types::SpecadoResult as FFIResult;
        
        let (kind, message) = match result {
            FFIResult::Success => return Self::new(SpecadoErrorKind::InternalError, "Unexpected success result"),
            FFIResult::InvalidInput => (SpecadoErrorKind::InvalidInput, "Invalid input parameters"),
            FFIResult::JsonError => (SpecadoErrorKind::JsonError, "JSON parsing error"),
            FFIResult::ProviderNotFound => (SpecadoErrorKind::ProviderNotFound, "Provider not found"),
            FFIResult::ModelNotFound => (SpecadoErrorKind::ModelNotFound, "Model not found"),
            FFIResult::NetworkError => (SpecadoErrorKind::NetworkError, "Network error"),
            FFIResult::AuthenticationError => (SpecadoErrorKind::AuthenticationError, "Authentication failed"),
            FFIResult::RateLimitError => (SpecadoErrorKind::RateLimitError, "Rate limit exceeded"),
            FFIResult::TimeoutError => (SpecadoErrorKind::TimeoutError, "Operation timed out"),
            FFIResult::InternalError => (SpecadoErrorKind::InternalError, "Internal error"),
            FFIResult::MemoryError => (SpecadoErrorKind::InternalError, "Memory allocation failed"),
            FFIResult::Utf8Error => (SpecadoErrorKind::InvalidInput, "Invalid UTF-8 string"),
            FFIResult::NullPointer => (SpecadoErrorKind::InternalError, "Null pointer error"),
            FFIResult::Cancelled => (SpecadoErrorKind::InternalError, "Operation cancelled"),
            FFIResult::NotImplemented => (SpecadoErrorKind::InternalError, "Feature not implemented"),
            FFIResult::Unknown => (SpecadoErrorKind::Unknown, "Unknown error"),
        };

        Self::new(kind, message)
    }
}

/// Convert core library errors to JavaScript errors
impl From<specado_core::error::Error> for SpecadoError {
    fn from(error: specado_core::error::Error) -> Self {
        use specado_core::error::Error as CoreError;
        
        match error {
            CoreError::InvalidInput { message } => Self::new(SpecadoErrorKind::InvalidInput, message),
            CoreError::JsonError { message, .. } => Self::new(SpecadoErrorKind::JsonError, message),
            CoreError::ProviderNotFound { provider, .. } => {
                Self::with_details(SpecadoErrorKind::ProviderNotFound, "Provider not found", provider)
            },
            CoreError::ModelNotFound { model, provider, .. } => {
                Self::with_details(
                    SpecadoErrorKind::ModelNotFound, 
                    format!("Model '{}' not found", model),
                    format!("Provider: {}", provider)
                )
            },
            CoreError::NetworkError { message, .. } => Self::new(SpecadoErrorKind::NetworkError, message),
            CoreError::AuthenticationError { message, .. } => Self::new(SpecadoErrorKind::AuthenticationError, message),
            CoreError::RateLimitError { message, .. } => Self::new(SpecadoErrorKind::RateLimitError, message),
            CoreError::TimeoutError { message, .. } => Self::new(SpecadoErrorKind::TimeoutError, message),
            _ => Self::new(SpecadoErrorKind::InternalError, error.to_string()),
        }
    }
}

/// Convert anyhow errors to JavaScript errors
impl From<anyhow::Error> for SpecadoError {
    fn from(error: anyhow::Error) -> Self {
        Self::new(SpecadoErrorKind::InternalError, error.to_string())
    }
}

/// Convert serde_json errors to JavaScript errors
impl From<serde_json::Error> for SpecadoError {
    fn from(error: serde_json::Error) -> Self {
        Self::new(SpecadoErrorKind::JsonError, error.to_string())
    }
}