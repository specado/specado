//! Error types and handling for the CLI
//!
//! This module provides error types and utilities for handling
//! various failure modes in the CLI application.

use std::io;
use std::path::PathBuf;

/// Result type alias for CLI operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for CLI operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// IO error (file operations, etc.)
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Error from specado-core library
    #[error("Core error: {0}")]
    Core(#[from] specado_core::Error),

    /// File not found
    #[error("File not found: {}", path.display())]
    FileNotFound { path: PathBuf },

    /// Invalid file format
    #[error("Invalid file format for {}: expected {} format", path.display(), expected)]
    InvalidFormat { path: PathBuf, expected: String },

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Invalid argument combination
    #[error("Invalid arguments: {0}")]
    #[allow(dead_code)]
    InvalidArgs(String),

    /// Provider not found
    #[error("Provider '{}' not found", name)]
    ProviderNotFound { name: String },

    /// Model not found
    #[error("Model '{}' not found for provider '{}'", model, provider)]
    #[allow(dead_code)]
    ModelNotFound { provider: String, model: String },

    /// API key missing
    #[error("API key required for provider '{}'. Set via --api-key or SPECADO_API_KEY", provider)]
    #[allow(dead_code)]
    ApiKeyMissing { provider: String },

    /// Network error
    #[error("Network error: {0}")]
    #[allow(dead_code)]
    Network(String),

    /// Timeout error
    #[error("Operation timed out after {} seconds", seconds)]
    #[allow(dead_code)]
    Timeout { seconds: u64 },

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// YAML serialization/deserialization error
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// Generic error with context
    #[error("{message}")]
    Other { message: String },
}

impl Error {
    /// Create a configuration error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    /// Create an invalid arguments error
    #[allow(dead_code)]
    pub fn invalid_args(message: impl Into<String>) -> Self {
        Self::InvalidArgs(message.into())
    }

    /// Create a network error
    #[allow(dead_code)]
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network(message.into())
    }

    /// Create a generic error with message
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other {
            message: message.into(),
        }
    }

    /// Get the exit code for this error
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Io(_) => 1,
            Self::Core(_) => 2,
            Self::FileNotFound { .. } => 3,
            Self::InvalidFormat { .. } => 4,
            Self::Config(_) => 5,
            Self::InvalidArgs(_) => 6,
            Self::ProviderNotFound { .. } => 7,
            Self::ModelNotFound { .. } => 8,
            Self::ApiKeyMissing { .. } => 9,
            Self::Network(_) => 10,
            Self::Timeout { .. } => 11,
            Self::Json(_) => 12,
            Self::Yaml(_) => 13,
            Self::Other { .. } => 99,
        }
    }

    /// Check if this error should display usage help
    pub fn should_show_help(&self) -> bool {
        matches!(self, Self::InvalidArgs(_))
    }
}

/// Extension trait for displaying errors with context
#[allow(dead_code)]
pub trait ErrorContext<T> {
    /// Add context to an error
    fn context(self, msg: &str) -> Result<T>;
    
    /// Add context with a closure (only evaluated on error)
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: Into<Error>,
{
    fn context(self, msg: &str) -> Result<T> {
        self.map_err(|e| {
            let inner = e.into();
            Error::Other {
                message: format!("{}: {}", msg, inner),
            }
        })
    }

    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let inner = e.into();
            Error::Other {
                message: format!("{}: {}", f(), inner),
            }
        })
    }
}

/// Format an error for display to the user
pub fn format_error(error: &Error, use_color: bool) -> String {
    // Check if this is a Core error with enhanced diagnostics
    if let Error::Core(core_error) = error {
        // The HttpWithDiagnostics variant has its own formatting
        // that will be displayed via the Display trait
        return format!("{}", core_error);
    }
    
    if use_color {
        use colored::Colorize;
        format!("{} {}", "Error:".red().bold(), error)
    } else {
        format!("Error: {}", error)
    }
}

/// Format a chain of errors (with causes) for display
#[allow(dead_code)]
pub fn format_error_chain(error: &Error, use_color: bool) -> String {
    
    
    // If we had error sources, we'd iterate through them here
    // For now, just return the main error
    format_error(error, use_color)
}