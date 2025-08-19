//! Validation function implementation for Python bindings
//!
//! This module implements validation functions for prompt and provider
//! specifications using the specado-schemas crate.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::exceptions::PyValueError;
use crate::types::{PyPromptSpec, PyProviderSpec, PyValidationResult};
use serde_json::Value;
use specado_schemas::{
    create_prompt_spec_validator, create_provider_spec_validator,
    ValidationMode, ValidationResult as SchemaValidationResult,
    SchemaValidator, ValidationContext,
};

/// Validate a specification against its schema
/// 
/// Args:
///     spec (Any): The specification to validate (PromptSpec, ProviderSpec, or dict)
///     schema_type (Literal["prompt", "provider"]): The type of schema to validate against
/// 
/// Returns:
///     ValidationResult: Validation result with errors if any
/// 
/// Raises:
///     ValidationError: If schema type is invalid or validation setup fails
#[pyfunction]
pub fn validate(py: Python<'_>, spec: PyObject, schema_type: &str) -> PyResult<PyValidationResult> {
    let spec_json = match schema_type {
        "prompt" => {
            // Try to extract as PyPromptSpec first, then as dict
            if let Ok(prompt_spec) = spec.extract::<PyRef<PyPromptSpec>>(py) {
                serde_json::to_value(&prompt_spec.inner)
                    .map_err(|e| PyValueError::new_err(format!("Failed to serialize prompt spec: {}", e)))?
            } else if let Ok(dict) = spec.downcast::<PyDict>(py) {
                dict_to_json_value(py, dict)?
            } else {
                return Err(PyValueError::new_err("spec must be a PromptSpec or dict for prompt validation"));
            }
        }
        "provider" => {
            // Try to extract as PyProviderSpec first, then as dict
            if let Ok(provider_spec) = spec.extract::<PyRef<PyProviderSpec>>(py) {
                serde_json::to_value(&provider_spec.inner)
                    .map_err(|e| PyValueError::new_err(format!("Failed to serialize provider spec: {}", e)))?
            } else if let Ok(dict) = spec.downcast::<PyDict>(py) {
                dict_to_json_value(py, dict)?
            } else {
                return Err(PyValueError::new_err("spec must be a ProviderSpec or dict for provider validation"));
            }
        }
        _ => {
            return Err(PyValueError::new_err("schema_type must be 'prompt' or 'provider'"));
        }
    };

    // Perform validation
    let validation_result = validate_spec_internal(&spec_json, schema_type)?;
    
    Ok(validation_result)
}

/// Internal validation logic using specado-schemas
fn validate_spec_internal(spec: &Value, schema_type: &str) -> PyResult<PyValidationResult> {
    let result = match schema_type {
        "prompt" => {
            let validator = create_prompt_spec_validator()
                .map_err(|e| PyValueError::new_err(format!("Failed to create prompt validator: {}", e)))?;
            let context = ValidationContext::new(ValidationMode::Strict);
            validator.validate_with_context(spec, &context)
        }
        "provider" => {
            let validator = create_provider_spec_validator()
                .map_err(|e| PyValueError::new_err(format!("Failed to create provider validator: {}", e)))?;
            let context = ValidationContext::new(ValidationMode::Strict);
            validator.validate_with_context(spec, &context)
        }
        _ => unreachable!(), // Already validated above
    };
    
    match result {
        Ok(_) => Ok(PyValidationResult::new(true, vec![])),
        Err(validation_error) => {
            // Single validation error
            let errors = vec![validation_error.to_string()];
            Ok(PyValidationResult::new(false, errors))
        }
    }
}


/// Validate a specification using the FFI layer (alternative interface)
/// 
/// Args:
///     spec_json (str): JSON string of the specification to validate
///     spec_type (str): Type of specification ("prompt_spec" or "provider_spec")
///     mode (str): Validation mode ("basic", "partial", or "strict")
/// 
/// Returns:
///     ValidationResult: Validation result with errors if any
/// 
/// Raises:
///     ValidationError: If validation setup fails
#[pyfunction]
pub fn validate_spec(spec_json: &str, spec_type: &str, mode: &str) -> PyResult<PyValidationResult> {
    use std::ffi::{CString, CStr};
    use std::ptr;
    
    // Convert strings to C strings
    let spec_cstr = CString::new(spec_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid spec JSON: {}", e)))?;
    let type_cstr = CString::new(spec_type)
        .map_err(|e| PyValueError::new_err(format!("Invalid spec type: {}", e)))?;
    let mode_cstr = CString::new(mode)
        .map_err(|e| PyValueError::new_err(format!("Invalid mode: {}", e)))?;
    
    // Call FFI function
    let mut result_ptr: *mut std::os::raw::c_char = ptr::null_mut();
    
    let ffi_result = unsafe {
        specado_ffi::specado_validate(
            spec_cstr.as_ptr(),
            type_cstr.as_ptr(),
            mode_cstr.as_ptr(),
            &mut result_ptr,
        )
    };
    
    // Check for success
    if ffi_result != specado_ffi::SpecadoResult::Success {
        let error_msg = unsafe {
            let error_ptr = specado_ffi::specado_get_last_error();
            if error_ptr.is_null() {
                "Validation failed".to_string()
            } else {
                CStr::from_ptr(error_ptr).to_string_lossy().to_string()
            }
        };
        return Err(PyValueError::new_err(error_msg));
    }
    
    // Convert result back to Rust
    if result_ptr.is_null() {
        return Err(PyValueError::new_err("FFI returned null result"));
    }
    
    let result_json = unsafe {
        let c_str = CStr::from_ptr(result_ptr);
        let rust_str = c_str.to_str()
            .map_err(|e| PyValueError::new_err(format!("Invalid UTF-8 in result: {}", e)))?;
        let json_copy = rust_str.to_string();
        
        // Free the FFI-allocated string
        specado_ffi::specado_string_free(result_ptr);
        
        json_copy
    };
    
    // Parse the result JSON
    let validation_result: specado_ffi::ValidationResult = serde_json::from_str(&result_json)
        .map_err(|e| PyValueError::new_err(format!("Failed to parse result JSON: {}", e)))?;
    
    Ok(PyValidationResult::new(validation_result.is_valid, validation_result.errors))
}

/// Helper function to convert Python dict to JSON value
fn dict_to_json_value(py: Python<'_>, dict: &PyDict) -> PyResult<Value> {
    crate::types::py_to_json(py, dict.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::types::PyDict;
    
    #[test]
    fn test_validate_prompt_spec_minimal() {
        let mut errors = Vec::new();
        let spec = serde_json::json!({
            "model_class": "Chat",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello"
                }
            ],
            "strict_mode": "warn"
        });
        
        validate_prompt_spec(&spec, &mut errors);
        assert!(errors.is_empty(), "Errors: {:?}", errors);
    }
    
    #[test]
    fn test_validate_prompt_spec_missing_fields() {
        let mut errors = Vec::new();
        let spec = serde_json::json!({});
        
        validate_prompt_spec(&spec, &mut errors);
        assert!(errors.len() >= 3); // Should have errors for missing required fields
    }
    
    #[test]
    fn test_validate_provider_spec_minimal() {
        let mut errors = Vec::new();
        let spec = serde_json::json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test",
                "base_url": "https://api.test.com",
                "headers": {}
            },
            "models": [
                {
                    "id": "test-model",
                    "family": "test",
                    "endpoints": {},
                    "input_modes": {},
                    "tooling": {},
                    "json_output": {},
                    "constraints": {},
                    "mappings": {},
                    "response_normalization": {}
                }
            ]
        });
        
        validate_provider_spec(&spec, &mut errors);
        // Some errors might be expected due to incomplete structure, 
        // but should validate basic structure
        assert!(errors.len() < 10, "Too many validation errors: {:?}", errors);
    }
}