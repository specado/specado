//! Validation error types for PromptSpec and ProviderSpec schemas
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// A validation violation with detailed context
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Violation {
    /// The validation rule that was violated
    pub rule: String,
    /// What was expected
    pub expected: String,
    /// What was actually found
    pub actual: String,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Rule '{}' violated: expected {}, but found {}",
            self.rule, self.expected, self.actual
        )
    }
}

/// Schema validation error with path context and detailed violations
#[derive(Debug, Error, Serialize, Deserialize)]
pub struct ValidationError {
    /// JSON path where the error occurred
    pub path: String,
    /// Human-readable error message
    pub message: String,
    /// Detailed schema violations
    pub schema_violations: Vec<Violation>,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error at '{}': {}", self.path, self.message)?;
        
        if !self.schema_violations.is_empty() {
            write!(f, "\nViolations:")?;
            for violation in &self.schema_violations {
                write!(f, "\n  - {}", violation)?;
            }
        }
        
        Ok(())
    }
}

impl ValidationError {
    /// Create a new validation error
    pub fn new<P, M>(path: P, message: M) -> Self
    where
        P: Into<String>,
        M: Into<String>,
    {
        Self {
            path: path.into(),
            message: message.into(),
            schema_violations: Vec::new(),
        }
    }

    /// Create a validation error with violations
    pub fn with_violations<P, M>(path: P, message: M, violations: Vec<Violation>) -> Self
    where
        P: Into<String>,
        M: Into<String>,
    {
        Self {
            path: path.into(),
            message: message.into(),
            schema_violations: violations,
        }
    }

    /// Add a violation to this error
    pub fn add_violation(&mut self, violation: Violation) {
        self.schema_violations.push(violation);
    }

    /// Create a violation for a specific rule
    pub fn create_violation<R, E, A>(rule: R, expected: E, actual: A) -> Violation
    where
        R: Into<String>,
        E: Into<String>,
        A: Into<String>,
    {
        Violation {
            rule: rule.into(),
            expected: expected.into(),
            actual: actual.into(),
        }
    }
}

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Multiple validation errors that can occur during batch validation
#[derive(Debug, Error, Serialize, Deserialize)]
pub struct ValidationErrors {
    /// List of validation errors
    pub errors: Vec<ValidationError>,
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Multiple validation errors occurred:")?;
        for (i, error) in self.errors.iter().enumerate() {
            write!(f, "\n{}. {}", i + 1, error)?;
        }
        Ok(())
    }
}

impl ValidationErrors {
    /// Create a new validation errors collection
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    /// Add an error to the collection
    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Check if there are any errors
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get the number of errors
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Convert to result - Ok if no errors, Err if any errors exist
    pub fn into_result(self) -> Result<(), Self> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl Default for ValidationErrors {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ValidationError> for ValidationErrors {
    fn from(error: ValidationError) -> Self {
        let mut errors = Self::new();
        errors.add(error);
        errors
    }
}

impl From<Vec<ValidationError>> for ValidationErrors {
    fn from(errors: Vec<ValidationError>) -> Self {
        Self { errors }
    }
}