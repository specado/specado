//! Base validation trait and common utilities
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::validation::error::{ValidationError, ValidationErrors, ValidationResult};
use serde_json::Value;
use std::collections::HashMap;

/// Validation mode for different use cases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationMode {
    /// Full validation with all rules
    Strict,
    /// Partial validation for development
    Partial,
    /// Basic schema validation only
    Basic,
}

/// Validation context for passing additional information
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Current JSON path
    pub path: String,
    /// Validation mode
    pub mode: ValidationMode,
    /// Additional context data
    pub context: HashMap<String, Value>,
}

impl ValidationContext {
    /// Create a new validation context
    pub fn new(mode: ValidationMode) -> Self {
        Self {
            path: "$".to_string(),
            mode,
            context: HashMap::new(),
        }
    }

    /// Create a child context with updated path
    pub fn child<P: AsRef<str>>(&self, path_segment: P) -> Self {
        let new_path = if self.path == "$" {
            format!("$.{}", path_segment.as_ref())
        } else {
            format!("{}.{}", self.path, path_segment.as_ref())
        };

        Self {
            path: new_path,
            mode: self.mode,
            context: self.context.clone(),
        }
    }

    /// Create a child context for array index
    pub fn child_index(&self, index: usize) -> Self {
        Self {
            path: format!("{}[{}]", self.path, index),
            mode: self.mode,
            context: self.context.clone(),
        }
    }

    /// Add context data
    pub fn with_context<K: Into<String>>(mut self, key: K, value: Value) -> Self {
        self.context.insert(key.into(), value);
        self
    }

    /// Get context data
    pub fn get_context(&self, key: &str) -> Option<&Value> {
        self.context.get(key)
    }
}

/// Base trait for schema validators
pub trait SchemaValidator {
    /// The type being validated
    type Input;

    /// Validate a complete schema with all rules
    fn validate(&self, input: &Self::Input) -> ValidationResult<()> {
        let context = ValidationContext::new(ValidationMode::Strict);
        self.validate_with_context(input, &context)
    }

    /// Validate with specific context and mode
    fn validate_with_context(
        &self,
        input: &Self::Input,
        context: &ValidationContext,
    ) -> ValidationResult<()>;

    /// Validate in partial mode (for development)
    fn validate_partial(&self, input: &Self::Input) -> ValidationResult<()> {
        let context = ValidationContext::new(ValidationMode::Partial);
        self.validate_with_context(input, &context)
    }

    /// Validate basic schema compliance only
    fn validate_basic(&self, input: &Self::Input) -> ValidationResult<()> {
        let context = ValidationContext::new(ValidationMode::Basic);
        self.validate_with_context(input, &context)
    }

    /// Collect all validation errors (non-failing)
    fn collect_errors(&self, input: &Self::Input) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        if let Err(error) = self.validate(input) {
            errors.add(error);
        }
        errors
    }
}

/// Helper functions for common validation patterns
pub struct ValidationHelpers;

impl ValidationHelpers {
    /// Validate that a JSONPath expression is syntactically correct
    pub fn validate_jsonpath(path: &str, context: &ValidationContext) -> ValidationResult<()> {
        // Basic JSONPath syntax validation - check for common patterns
        if path.is_empty() {
            return Err(ValidationError::with_violations(
                &context.path,
                "JSONPath expression cannot be empty".to_string(),
                vec![ValidationError::create_violation(
                    "jsonpath_syntax",
                    "non-empty JSONPath expression",
                    "empty string".to_string(),
                )],
            ));
        }

        // Must start with $ for root
        if !path.starts_with('$') {
            return Err(ValidationError::with_violations(
                &context.path,
                format!("Invalid JSONPath expression: {}", path),
                vec![ValidationError::create_violation(
                    "jsonpath_syntax",
                    "JSONPath expression starting with '$'",
                    format!("expression starts with '{}'", path.chars().next().unwrap_or(' ')),
                )],
            ));
        }

        // For now, accept any path starting with $ as valid
        // In a real implementation, you'd use a proper JSONPath parser
        Ok(())
    }

    /// Validate environment variable reference format
    pub fn validate_env_var_reference(
        reference: &str,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        // Check for ${ENV:VARIABLE_NAME} pattern
        if reference.starts_with("${ENV:") && reference.ends_with('}') {
            let var_name = &reference[6..reference.len()-1]; // Extract VARIABLE_NAME
            if var_name.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_') {
                Ok(())
            } else {
                Err(ValidationError::with_violations(
                    &context.path,
                    format!("Invalid environment variable name: {}", var_name),
                    vec![ValidationError::create_violation(
                        "env_var_format",
                        "uppercase letters, digits, and underscores only",
                        reference.to_string(),
                    )],
                ))
            }
        } else {
            Err(ValidationError::with_violations(
                &context.path,
                format!("Invalid environment variable reference: {}", reference),
                vec![ValidationError::create_violation(
                    "env_var_format",
                    "${{ENV:VARIABLE_NAME}} format",
                    reference.to_string(),
                )],
            ))
        }
    }

    /// Validate URL scheme matches expected protocols
    pub fn validate_url_scheme(
        url: &str,
        allowed_schemes: &[&str],
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        // Extract scheme from URL (everything before the first colon)
        if let Some(colon_pos) = url.find(':') {
            let scheme = &url[..colon_pos].to_lowercase();
            
            if allowed_schemes.iter().any(|&s| s == scheme) {
                Ok(())
            } else {
                Err(ValidationError::with_violations(
                    &context.path,
                    format!("Invalid URL scheme: {}", scheme),
                    vec![ValidationError::create_violation(
                        "url_scheme",
                        format!("one of: {}", allowed_schemes.join(", ")),
                        scheme.to_string(),
                    )],
                ))
            }
        } else {
            Err(ValidationError::with_violations(
                &context.path,
                format!("Invalid URL format: {}", url),
                vec![ValidationError::create_violation(
                    "url_format",
                    "valid URL with scheme",
                    url.to_string(),
                )],
            ))
        }
    }

    /// Validate that a field exists when a condition is met
    pub fn validate_conditional_field<T>(
        condition: bool,
        field_value: Option<&T>,
        field_name: &str,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        if condition && field_value.is_none() {
            Err(ValidationError::with_violations(
                &context.path,
                format!("Field {} is required when condition is met", field_name),
                vec![ValidationError::create_violation(
                    "conditional_field",
                    format!("{} to be present", field_name),
                    "field is missing".to_string(),
                )],
            ))
        } else {
            Ok(())
        }
    }

    /// Validate that a field is absent when a condition is met
    pub fn validate_conditional_absence<T>(
        condition: bool,
        field_value: Option<&T>,
        field_name: &str,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        if condition && field_value.is_some() {
            Err(ValidationError::with_violations(
                &context.path,
                format!("Field {} is not allowed when condition is met", field_name),
                vec![ValidationError::create_violation(
                    "conditional_absence",
                    format!("{} to be absent", field_name),
                    "field is present".to_string(),
                )],
            ))
        } else {
            Ok(())
        }
    }

    /// Validate array is non-empty when required
    pub fn validate_non_empty_array<T>(
        array: Option<&Vec<T>>,
        field_name: &str,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        match array {
            Some(arr) if arr.is_empty() => Err(ValidationError::with_violations(
                &context.path,
                format!("Array {} cannot be empty", field_name),
                vec![ValidationError::create_violation(
                    "non_empty_array",
                    "non-empty array",
                    "empty array".to_string(),
                )],
            )),
            None => Err(ValidationError::with_violations(
                &context.path,
                format!("Array {} is required", field_name),
                vec![ValidationError::create_violation(
                    "required_array",
                    format!("{} array to be present", field_name),
                    "array is missing".to_string(),
                )],
            )),
            _ => Ok(()),
        }
    }

    /// Validate that a string is one of allowed values
    pub fn validate_enum_value(
        value: &str,
        allowed_values: &[&str],
        field_name: &str,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        if allowed_values.contains(&value) {
            Ok(())
        } else {
            Err(ValidationError::with_violations(
                &context.path,
                format!("Invalid value for {}: {}", field_name, value),
                vec![ValidationError::create_violation(
                    "enum_value",
                    format!("one of: {}", allowed_values.join(", ")),
                    value.to_string(),
                )],
            ))
        }
    }

    /// Validate compatibility between multiple fields
    pub fn validate_field_compatibility(
        field1_name: &str,
        field1_value: &str,
        field2_name: &str,
        field2_value: &str,
        compatible_combinations: &[(&str, &str)],
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        let combination = (field1_value, field2_value);
        
        if compatible_combinations.contains(&combination) {
            Ok(())
        } else {
            Err(ValidationError::with_violations(
                &context.path,
                format!(
                    "Incompatible combination: {}='{}' with {}='{}'",
                    field1_name, field1_value, field2_name, field2_value
                ),
                vec![ValidationError::create_violation(
                    "field_compatibility",
                    format!("compatible combination for {} and {}", field1_name, field2_name),
                    format!("{}='{}', {}='{}'", field1_name, field1_value, field2_name, field2_value),
                )],
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_context_child() {
        let context = ValidationContext::new(ValidationMode::Strict);
        let child = context.child("test");
        assert_eq!(child.path, "$.test");
        
        let grandchild = child.child("nested");
        assert_eq!(grandchild.path, "$.test.nested");
    }

    #[test]
    fn test_validation_context_child_index() {
        let context = ValidationContext::new(ValidationMode::Strict).child("array");
        let indexed = context.child_index(0);
        assert_eq!(indexed.path, "$.array[0]");
    }

    #[test]
    fn test_validate_jsonpath_valid() {
        let context = ValidationContext::new(ValidationMode::Strict);
        assert!(ValidationHelpers::validate_jsonpath("$.path.to.field", &context).is_ok());
        assert!(ValidationHelpers::validate_jsonpath("$[0].items", &context).is_ok());
    }

    #[test]
    fn test_validate_env_var_reference() {
        let context = ValidationContext::new(ValidationMode::Strict);
        assert!(ValidationHelpers::validate_env_var_reference("${ENV:API_KEY}", &context).is_ok());
        assert!(ValidationHelpers::validate_env_var_reference("${ENV:DATABASE_URL}", &context).is_ok());
        assert!(ValidationHelpers::validate_env_var_reference("invalid", &context).is_err());
        assert!(ValidationHelpers::validate_env_var_reference("${env:lowercase}", &context).is_err());
    }

    #[test]
    fn test_validate_url_scheme() {
        let context = ValidationContext::new(ValidationMode::Strict);
        let allowed = &["http", "https"];
        
        assert!(ValidationHelpers::validate_url_scheme("https://api.example.com", allowed, &context).is_ok());
        assert!(ValidationHelpers::validate_url_scheme("http://localhost:8080", allowed, &context).is_ok());
        assert!(ValidationHelpers::validate_url_scheme("ftp://example.com", allowed, &context).is_err());
        assert!(ValidationHelpers::validate_url_scheme("invalid-url", allowed, &context).is_err());
    }
}