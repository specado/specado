//! Validation FFI implementation
//!
//! This module implements validation functions using the specado-schemas crate
//! for proper JSON Schema and business rule validation.

use serde_json::Value;
use specado_schemas::{
    create_prompt_spec_validator, create_provider_spec_validator,
    ValidationMode, ValidationResult as SchemaValidationResult,
    SchemaValidator, ValidationContext,
};
use crate::types::SpecadoResult;
use crate::memory::set_last_error;

/// Validation result for FFI
#[derive(Debug, serde::Serialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Validation mode used
    pub mode: String,
}

/// Validate a PromptSpec JSON
pub fn validate_prompt_spec(
    prompt_json: &str,
    mode: ValidationMode,
) -> Result<String, SpecadoResult> {
    // Parse JSON
    let prompt_value: Value = serde_json::from_str(prompt_json)
        .map_err(|e| {
            set_last_error(format!("Failed to parse prompt JSON: {}", e));
            SpecadoResult::JsonError
        })?;
    
    // Create validator
    let validator = create_prompt_spec_validator()
        .map_err(|e| {
            set_last_error(format!("Failed to create validator: {}", e));
            SpecadoResult::InternalError
        })?;
    
    // Create validation context
    let context = ValidationContext::new(mode);
    
    // Perform validation
    let validation_result = validator.validate_with_context(&prompt_value, &context);
    
    let result = match validation_result {
        Ok(_) => ValidationResult {
            is_valid: true,
            errors: vec![],
            warnings: vec![],
            mode: format!("{:?}", mode),
        },
        Err(validation_error) => {
            // Single validation error
            ValidationResult {
                is_valid: false,
                errors: vec![validation_error.to_string()],
                warnings: vec![], // Schema validation doesn't separate warnings currently
                mode: format!("{:?}", mode),
            }
        }
    };
    
    // Serialize result
    serde_json::to_string(&result)
        .map_err(|e| {
            set_last_error(format!("Failed to serialize validation result: {}", e));
            SpecadoResult::JsonError
        })
}

/// Validate a ProviderSpec JSON
pub fn validate_provider_spec(
    provider_spec_json: &str,
    mode: ValidationMode,
) -> Result<String, SpecadoResult> {
    // Parse JSON
    let provider_value: Value = serde_json::from_str(provider_spec_json)
        .map_err(|e| {
            set_last_error(format!("Failed to parse provider spec JSON: {}", e));
            SpecadoResult::JsonError
        })?;
    
    // Create validator
    let validator = create_provider_spec_validator()
        .map_err(|e| {
            set_last_error(format!("Failed to create validator: {}", e));
            SpecadoResult::InternalError
        })?;
    
    // Create validation context
    let context = ValidationContext::new(mode);
    
    // Perform validation
    let validation_result = validator.validate_with_context(&provider_value, &context);
    
    let result = match validation_result {
        Ok(_) => ValidationResult {
            is_valid: true,
            errors: vec![],
            warnings: vec![],
            mode: format!("{:?}", mode),
        },
        Err(validation_error) => {
            // Single validation error  
            ValidationResult {
                is_valid: false,
                errors: vec![validation_error.to_string()],
                warnings: vec![],
                mode: format!("{:?}", mode),
            }
        }
    };
    
    // Serialize result
    serde_json::to_string(&result)
        .map_err(|e| {
            set_last_error(format!("Failed to serialize validation result: {}", e));
            SpecadoResult::JsonError
        })
}

/// Simple validation function that takes a JSON string and validation mode string
pub fn validate_json(
    json_str: &str,
    spec_type: &str,
    mode_str: &str,
) -> Result<String, SpecadoResult> {
    // Parse validation mode
    let mode = match mode_str {
        "basic" => ValidationMode::Basic,
        "partial" => ValidationMode::Partial,
        "strict" => ValidationMode::Strict,
        _ => {
            set_last_error(format!("Invalid validation mode: {}", mode_str));
            return Err(SpecadoResult::InvalidInput);
        }
    };
    
    // Route to appropriate validator
    match spec_type {
        "prompt_spec" => validate_prompt_spec(json_str, mode),
        "provider_spec" => validate_provider_spec(json_str, mode),
        _ => {
            set_last_error(format!("Invalid spec type: {}", spec_type));
            Err(SpecadoResult::InvalidInput)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_validate_prompt_spec_valid() {
        let prompt = json!({
            "spec_version": "1.0",
            "id": "test-prompt",
            "model_class": "Chat",
            "messages": [
                {"role": "user", "content": "Hello, world!"}
            ]
        });
        
        let result = validate_prompt_spec(&prompt.to_string(), ValidationMode::Basic);
        assert!(result.is_ok());
        
        let validation_result: ValidationResult = serde_json::from_str(&result.unwrap()).unwrap();
        assert!(validation_result.is_valid);
        assert!(validation_result.errors.is_empty());
    }
    
    #[test]
    fn test_validate_prompt_spec_invalid() {
        let prompt = json!({
            "model_class": "Chat"
            // Missing required fields
        });
        
        let result = validate_prompt_spec(&prompt.to_string(), ValidationMode::Strict);
        assert!(result.is_ok());
        
        let validation_result: ValidationResult = serde_json::from_str(&result.unwrap()).unwrap();
        assert!(!validation_result.is_valid);
        assert!(!validation_result.errors.is_empty());
    }
    
    #[test]
    fn test_validate_json_wrapper() {
        let prompt = json!({
            "spec_version": "1.0",
            "id": "test-prompt",
            "model_class": "Chat",
            "messages": [
                {"role": "user", "content": "Hello, world!"}
            ]
        });
        
        let result = validate_json(&prompt.to_string(), "prompt_spec", "basic");
        assert!(result.is_ok());
        
        let validation_result: ValidationResult = serde_json::from_str(&result.unwrap()).unwrap();
        assert!(validation_result.is_valid);
    }
}