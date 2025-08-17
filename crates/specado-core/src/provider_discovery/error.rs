//! Error types for provider discovery

use std::fmt;

/// Error types for provider discovery operations
#[derive(Debug)]
pub enum SpecadoError {
    /// Provider not found for the given model
    ProviderNotFound {
        model: String,
        available: Vec<String>,
    },
    /// IO error when reading provider specs
    IoError {
        path: String,
        operation: String,
        details: String,
    },
    /// JSON parsing error
    ParseError {
        path: String,
        line: usize,
        column: usize,
        message: String,
    },
}

impl fmt::Display for SpecadoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProviderNotFound { model, available } => {
                write!(f, "Provider not found for model '{}'. Available models: {}", 
                    model, available.join(", "))
            }
            Self::IoError { path, operation, details } => {
                write!(f, "IO error at {}: {} - {}", path, operation, details)
            }
            Self::ParseError { path, line, column, message } => {
                write!(f, "Parse error in {} at line {}, column {}: {}", 
                    path, line, column, message)
            }
        }
    }
}

impl std::error::Error for SpecadoError {}

/// Result type for provider discovery operations
pub type SpecadoResult<T> = std::result::Result<T, SpecadoError>;