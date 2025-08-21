//! Core validation engine and PreValidator implementation
//!
//! This module contains the main PreValidator struct and orchestrates all
//! validation logic by coordinating field validators and constraint validators.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::{Error, Result, StrictMode};
use super::{ValidationError, ValidationSeverity, ValidationMode};
use super::super::TranslationContext;
use super::field_validators::{
    validate_messages, validate_model_class, validate_sampling_params, 
    validate_limits, validate_tools, validate_media, validate_response_format
};
use super::constraint_validators::{
    validate_provider_constraints, validate_mutually_exclusive_fields
};

/// Pre-validator for checking input compatibility before translation
///
/// The PreValidator performs comprehensive validation checks on the input PromptSpec
/// to ensure it can be successfully translated to the target provider format.
/// This includes checking for:
/// - Required fields based on model class
/// - Field constraints (min/max lengths, allowed values, patterns)
/// - Type constraints match expected schemas
/// - Model compatibility checks
/// - Provider-specific limitations
/// - Token count and size validation
pub struct PreValidator<'a> {
    context: &'a TranslationContext,
    validation_mode: ValidationMode,
}

impl<'a> PreValidator<'a> {
    /// Create a new pre-validator with default validation mode
    pub fn new(context: &'a TranslationContext) -> Self {
        let validation_mode = match context.strict_mode {
            StrictMode::Strict => ValidationMode::Strict,
            StrictMode::Warn | StrictMode::Coerce => ValidationMode::Lenient,
        };
        
        Self {
            context,
            validation_mode,
        }
    }
    
    /// Create a new pre-validator with explicit validation mode
    pub fn with_mode(context: &'a TranslationContext, mode: ValidationMode) -> Self {
        Self {
            context,
            validation_mode: mode,
        }
    }
    
    /// Perform comprehensive pre-validation checks
    ///
    /// This method runs all validation rules and returns detailed validation errors.
    /// Depending on the validation mode, it may return early on the first error
    /// (strict mode) or collect all errors (lenient mode).
    pub fn validate(&self) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        // Run basic structural validation
        errors.extend(self.validate_basic_structure()?);
        
        // Run field-specific validation
        errors.extend(validate_messages(self.context)?);
        errors.extend(validate_model_class(self.context)?);
        errors.extend(validate_sampling_params(self.context)?);
        errors.extend(validate_limits(self.context)?);
        errors.extend(validate_tools(self.context)?);
        errors.extend(validate_media(self.context)?);
        errors.extend(validate_response_format(self.context)?);
        
        // Run provider-specific validation
        errors.extend(validate_provider_constraints(self.context)?);
        
        // Run mutually exclusive field validation
        errors.extend(validate_mutually_exclusive_fields(self.context)?);
        
        // Filter errors based on validation mode
        let filtered_errors = self.filter_errors_by_mode(&errors);
        
        Ok(filtered_errors)
    }
    
    /// Validate only and return the first error if any (legacy compatibility)
    pub fn validate_strict(&self) -> Result<()> {
        let errors = self.validate()?;
        
        // Find the first error-level validation issue
        if let Some(error) = errors.iter().find(|e| e.severity == ValidationSeverity::Error) {
            return Err(Error::Validation {
                field: error.field_path.clone(),
                message: error.message.clone(),
                expected: error.expected.clone(),
            });
        }
        
        // Check for compatibility issues based on strict mode
        if self.context.should_fail_on_error() {
            if let Some(warning) = errors.iter().find(|e| e.severity == ValidationSeverity::Warning) {
                return Err(Error::Validation {
                    field: warning.field_path.clone(),
                    message: warning.message.clone(),
                    expected: warning.expected.clone(),
                });
            }
        }
        
        Ok(())
    }
    
    /// Validate basic structural requirements
    fn validate_basic_structure(&self) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        // Check that required top-level fields are present
        if self.context.prompt_spec.model_class.is_empty() {
            errors.push(ValidationError {
                field_path: "model_class".to_string(),
                message: "Model class is required".to_string(),
                expected: Some("Non-empty string".to_string()),
                actual: Some("empty".to_string()),
                severity: ValidationSeverity::Error,
            });
        }
        
        Ok(errors)
    }
    
    /// Filter validation errors based on the current validation mode
    fn filter_errors_by_mode(&self, errors: &[ValidationError]) -> Vec<ValidationError> {
        match self.validation_mode {
            ValidationMode::Strict => {
                // Return all errors and warnings
                errors.to_vec()
            }
            ValidationMode::Lenient => {
                // Return only errors, filter out warnings and info
                errors.iter()
                    .filter(|e| e.severity == ValidationSeverity::Error)
                    .cloned()
                    .collect()
            }
        }
    }
}