//! Core types and error definitions for TranslationResult builder
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

/// Builder state tracking the completeness of the result
#[derive(Debug, Clone, PartialEq)]
pub enum BuilderState {
    /// Builder is incomplete, missing required fields
    Incomplete,
    /// Builder has all required fields and is ready to build
    Ready,
    /// Builder has been consumed to create a TranslationResult
    Built,
}

/// Enhanced error type for builder operations
#[derive(Debug, Clone)]
pub enum BuilderError {
    /// Attempted to use a builder that was already built
    AlreadyBuilt,
    /// Missing required fields for building
    MissingRequired(Vec<String>),
    /// Invalid merge operation
    InvalidMerge(String),
    /// Validation failed
    ValidationFailed(String),
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuilderError::AlreadyBuilt => write!(f, "Builder has already been built and cannot be reused"),
            BuilderError::MissingRequired(fields) => write!(f, "Missing required fields: {}", fields.join(", ")),
            BuilderError::InvalidMerge(msg) => write!(f, "Invalid merge operation: {}", msg),
            BuilderError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl std::error::Error for BuilderError {}