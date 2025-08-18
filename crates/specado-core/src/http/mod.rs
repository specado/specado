//! HTTP client implementation for provider API communication
//!
//! This module provides a robust HTTP client with:
//! - Request building from ProviderSpec configurations
//! - Authentication handling for multiple providers
//! - Error classification and normalization
//! - Retry logic with exponential backoff
//! - Fallback strategies for resilience
//! - Enhanced error diagnostics

pub mod builder;
pub mod auth;
pub mod error;
pub mod retry;
pub mod client;
pub mod normalizer;
pub mod fallback;
pub mod diagnostics;

pub use builder::RequestBuilder;
pub use auth::{AuthHandler, AuthError};
pub use error::{HttpError, ErrorClassification};
pub use retry::{RetryPolicy, RetryDecision};
pub use client::HttpClient;
pub use normalizer::{ResponseNormalizer, normalize_response};
pub use fallback::{FallbackHandler, FallbackConfig, FallbackAttempt};
pub use diagnostics::{ErrorDiagnostics, DiagnosticsBuilder};

// Re-export commonly used types
pub use reqwest::{Method, StatusCode};