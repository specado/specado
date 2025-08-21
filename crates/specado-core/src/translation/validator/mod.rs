//! Comprehensive pre-validation logic for translation operations
//!
//! This module implements comprehensive pre-validation that runs before the main
//! translation process to catch issues early and provide detailed validation errors.
//! The validation system supports both strict and lenient modes and provides
//! provider-specific validation rules.
//!
//! The validator is organized into focused modules:
//! - `types`: Core validation types and enums
//! - `core`: Main PreValidator struct and orchestration logic
//! - `field_validators`: Field-specific validation functions
//! - `constraint_validators`: Provider and constraint validation
//! - `tests`: Comprehensive test suite
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

pub mod types;
pub mod core;
pub mod field_validators;
pub mod constraint_validators;
pub mod tests;

// Re-export public API
pub use types::{ValidationError, ValidationSeverity, ValidationMode};
pub use core::PreValidator;