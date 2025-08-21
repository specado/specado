//! Core validation types and enums
//!
//! This module contains the fundamental types used throughout the validation system,
//! including error types, severity levels, and validation modes.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

/// Validation error with detailed field path information
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field_path: String,
    pub message: String,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub severity: ValidationSeverity,
}

/// Severity levels for validation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    /// Error that will cause translation to fail
    Error,
    /// Warning that may impact translation quality
    Warning,
    /// Information about potential issues
    Info,
}

/// Validation mode configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationMode {
    /// All validation rules enforced
    Strict,
    /// Relaxed validation, warnings only
    Lenient,
}