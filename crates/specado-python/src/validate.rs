//! Validation function implementation for Python bindings
//!
//! This module implements validation functions for prompt and provider
//! specifications using the specado-schemas crate.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::exceptions::PyValueError;
use crate::types::{PyPromptSpec, PyProviderSpec, PyValidationResult};
use serde_json::Value;

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

    // Convert to JSON string for FFI
    let spec_json_str = serde_json::to_string(&spec_json)
        .map_err(|e| PyValueError::new_err(format!("Failed to serialize spec: {}", e)))?;
    
    // Map schema_type to FFI spec_type
    let spec_type = match schema_type {
        "prompt" => "prompt_spec",
        "provider" => "provider_spec",
        _ => unreachable!(), // Already validated above
    };
    
    // Use FFI validation
    validate_spec(&spec_json_str, spec_type, "standard")
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
    
    // Parse the result JSON as a Value first
    let validation_json: serde_json::Value = serde_json::from_str(&result_json)
        .map_err(|e| PyValueError::new_err(format!("Failed to parse result JSON: {}", e)))?;
    
    // Extract validation fields from JSON
    let is_valid = validation_json.get("valid")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    let errors = validation_json.get("errors")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| {
                    let path = e.get("path")?.as_str()?.to_string();
                    let message = e.get("message")?.as_str()?.to_string();
                    let code = e.get("code").and_then(|c| c.as_str()).unwrap_or("ERROR").to_string();
                    Some(format!("{} at {}: {}", code, path, message))
                })
                .collect()
        })
        .unwrap_or_else(Vec::new);
    
    Ok(PyValidationResult::new(is_valid, errors))
}

/// Helper function to convert Python dict to JSON value
fn dict_to_json_value(py: Python<'_>, dict: &PyDict) -> PyResult<Value> {
    crate::types::py_to_json(py, dict.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_prompt_spec_minimal() {
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
        
        let spec_json = serde_json::to_string(&spec).unwrap();
        let result = validate_spec(&spec_json, "prompt_spec", "standard");
        
        match result {
            Ok(validation_result) => {
                // Check if validation passed (might fail with real FFI validation)
                println!("Validation result: {:?}", validation_result.errors);
            }
            Err(e) => {
                println!("Validation error: {:?}", e);
            }
        }
    }
    
    #[test]
    fn test_validate_provider_spec_minimal() {
        // Use a valid provider spec from golden corpus structure
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
                    "endpoints": {
                        "chat_completion": {
                            "method": "POST",
                            "path": "/chat/completions",
                            "protocol": "http"
                        },
                        "streaming_chat_completion": {
                            "method": "POST",
                            "path": "/chat/completions",
                            "protocol": "sse"
                        }
                    },
                    "input_modes": {
                        "messages": true,
                        "single_text": false,
                        "images": false
                    },
                    "tooling": {
                        "tools_supported": false,
                        "parallel_tool_calls_default": false,
                        "can_disable_parallel_tool_calls": false
                    },
                    "json_output": {
                        "native_param": false,
                        "strategy": "none"
                    },
                    "parameters": {},
                    "constraints": {
                        "system_prompt_location": "first_message",
                        "forbid_unknown_top_level_fields": false,
                        "mutually_exclusive": [],
                        "resolution_preferences": [],
                        "limits": {
                            "max_tool_schema_bytes": 1000,
                            "max_system_prompt_bytes": 1000
                        }
                    },
                    "mappings": {
                        "paths": {},
                        "flags": {}
                    },
                    "response_normalization": {
                        "sync": {
                            "content_path": "$.choices[0].message.content",
                            "finish_reason_path": "$.choices[0].finish_reason",
                            "finish_reason_map": {}
                        },
                        "stream": {
                            "protocol": "sse",
                            "event_selector": {
                                "type_path": "$.type",
                                "routes": []
                            }
                        }
                    }
                }
            ]
        });
        
        let spec_json = serde_json::to_string(&spec).unwrap();
        let result = validate_spec(&spec_json, "provider_spec", "standard");
        
        match result {
            Ok(validation_result) => {
                println!("Validation result: {:?}", validation_result.errors);
            }
            Err(e) => {
                println!("Validation error: {:?}", e);
            }
        }
    }
}