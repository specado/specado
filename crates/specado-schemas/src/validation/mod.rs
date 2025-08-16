//! Validation module for PromptSpec and ProviderSpec schemas
//!
//! This module provides comprehensive JSON Schema validation with custom business logic
//! rules for both PromptSpec and ProviderSpec schemas. It supports multiple validation
//! modes for different use cases:
//!
//! - **Basic**: JSON Schema validation only
//! - **Partial**: Schema + selected custom rules (development mode)
//! - **Strict**: Schema + all custom rules (production mode)
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

pub mod base;
pub mod error;
pub mod prompt_spec;
pub mod provider_spec;

// Re-export commonly used types
pub use base::{SchemaValidator, ValidationContext, ValidationMode, ValidationHelpers};
pub use error::{ValidationError, ValidationErrors, ValidationResult, Violation};
pub use prompt_spec::PromptSpecValidator;
pub use provider_spec::ProviderSpecValidator;

/// Convenience function to create a PromptSpec validator
///
/// # Examples
///
/// ```rust
/// use specado_schemas::validation::{create_prompt_spec_validator, SchemaValidator};
/// use serde_json::json;
///
/// let validator = create_prompt_spec_validator().unwrap();
/// let spec = json!({
///     "spec_version": "1.0",
///     "id": "test-123",
///     "model_class": "Chat",
///     "messages": [{"role": "user", "content": "Hello"}],
///     "strict_mode": false
/// });
///
/// assert!(validator.validate(&spec).is_ok());
/// ```
pub fn create_prompt_spec_validator() -> Result<PromptSpecValidator, Box<dyn std::error::Error>> {
    PromptSpecValidator::new()
}

/// Convenience function to create a ProviderSpec validator
///
/// # Examples
///
/// ```rust
/// use specado_schemas::validation::{create_provider_spec_validator, SchemaValidator};
/// use serde_json::json;
///
/// let validator = create_provider_spec_validator().unwrap();
/// let spec = json!({
///     "spec_version": "1.0",
///     "provider": {
///         "name": "test-provider",
///         "base_url": "https://api.test.com",
///         "headers": {},
///         "auth": {
///             "type": "api_key",
///             "header_name": "X-API-Key"
///         }
///     },
///     "models": []
/// });
///
/// assert!(validator.validate(&spec).is_ok());
/// ```
pub fn create_provider_spec_validator() -> Result<ProviderSpecValidator, Box<dyn std::error::Error>> {
    ProviderSpecValidator::new()
}

/// Validation configuration for batch operations
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Validation mode to use
    pub mode: ValidationMode,
    /// Whether to stop on first error or collect all errors
    pub fail_fast: bool,
    /// Maximum number of errors to collect (0 = unlimited)
    pub max_errors: usize,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            mode: ValidationMode::Strict,
            fail_fast: false,
            max_errors: 0,
        }
    }
}

impl ValidationConfig {
    /// Create a configuration for strict validation
    pub fn strict() -> Self {
        Self {
            mode: ValidationMode::Strict,
            fail_fast: false,
            max_errors: 0,
        }
    }

    /// Create a configuration for partial validation (development mode)
    pub fn partial() -> Self {
        Self {
            mode: ValidationMode::Partial,
            fail_fast: false,
            max_errors: 0,
        }
    }

    /// Create a configuration for basic validation
    pub fn basic() -> Self {
        Self {
            mode: ValidationMode::Basic,
            fail_fast: false,
            max_errors: 0,
        }
    }

    /// Enable fail-fast mode
    pub fn with_fail_fast(mut self) -> Self {
        self.fail_fast = true;
        self
    }

    /// Set maximum number of errors to collect
    pub fn with_max_errors(mut self, max_errors: usize) -> Self {
        self.max_errors = max_errors;
        self
    }
}

/// Batch validation for multiple PromptSpec documents
pub fn validate_prompt_specs_batch(
    specs: &[serde_json::Value],
    config: &ValidationConfig,
) -> Result<(), ValidationErrors> {
    let validator = create_prompt_spec_validator()
        .map_err(|e| ValidationErrors::from(ValidationError::new("$", format!("Failed to create validator: {}", e))))?;

    let mut errors = ValidationErrors::new();
    let context = ValidationContext::new(config.mode);

    for (i, spec) in specs.iter().enumerate() {
        let spec_context = context.child_index(i);
        match validator.validate_with_context(spec, &spec_context) {
            Ok(_) => continue,
            Err(error) => {
                errors.add(error);
                
                if config.fail_fast {
                    break;
                }
                
                if config.max_errors > 0 && errors.len() >= config.max_errors {
                    break;
                }
            }
        }
    }

    errors.into_result()
}

/// Batch validation for multiple ProviderSpec documents
pub fn validate_provider_specs_batch(
    specs: &[serde_json::Value],
    config: &ValidationConfig,
) -> Result<(), ValidationErrors> {
    let validator = create_provider_spec_validator()
        .map_err(|e| ValidationErrors::from(ValidationError::new("$", format!("Failed to create validator: {}", e))))?;

    let mut errors = ValidationErrors::new();
    let context = ValidationContext::new(config.mode);

    for (i, spec) in specs.iter().enumerate() {
        let spec_context = context.child_index(i);
        match validator.validate_with_context(spec, &spec_context) {
            Ok(_) => continue,
            Err(error) => {
                errors.add(error);
                
                if config.fail_fast {
                    break;
                }
                
                if config.max_errors > 0 && errors.len() >= config.max_errors {
                    break;
                }
            }
        }
    }

    errors.into_result()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_validators() {
        assert!(create_prompt_spec_validator().is_ok());
        assert!(create_provider_spec_validator().is_ok());
    }

    #[test]
    fn test_validation_config_defaults() {
        let config = ValidationConfig::default();
        assert_eq!(config.mode, ValidationMode::Strict);
        assert!(!config.fail_fast);
        assert_eq!(config.max_errors, 0);
    }

    #[test]
    fn test_validation_config_builders() {
        let strict = ValidationConfig::strict().with_fail_fast().with_max_errors(5);
        assert_eq!(strict.mode, ValidationMode::Strict);
        assert!(strict.fail_fast);
        assert_eq!(strict.max_errors, 5);

        let partial = ValidationConfig::partial();
        assert_eq!(partial.mode, ValidationMode::Partial);

        let basic = ValidationConfig::basic();
        assert_eq!(basic.mode, ValidationMode::Basic);
    }

    #[test]
    fn test_batch_validation_prompt_specs() {
        let specs = vec![
            json!({
                "spec_version": "1.0",
                "id": "test-1",
                "model_class": "Chat",
                "messages": [{"role": "user", "content": "Hello"}]
            }),
            json!({
                "spec_version": "1.0",
                "id": "test-2",
                "model_class": "Chat",
                "messages": []  // Invalid: empty messages
            }),
        ];

        let config = ValidationConfig::strict();
        let result = validate_prompt_specs_batch(&specs, &config);
        assert!(result.is_err());

        let config = ValidationConfig::strict().with_fail_fast();
        let result = validate_prompt_specs_batch(&specs, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_batch_validation_provider_specs() {
        let specs = vec![
            json!({
                "spec_version": "1.0",
                "provider": {
                    "id": "test-1",
                    "name": "Test Provider 1",
                    "organization": "Test Org"
                },
                "models": []
            }),
            json!({
                "spec_version": "1.0",
                // Missing required provider field
                "models": []
            }),
        ];

        let config = ValidationConfig::strict();
        let result = validate_provider_specs_batch(&specs, &config);
        assert!(result.is_err());

        // For basic validation, only the second spec should fail (missing required field)
        let config = ValidationConfig::basic();
        let result = validate_provider_specs_batch(&specs, &config);
        assert!(result.is_err()); // Should fail because second spec is missing provider field
    }
}