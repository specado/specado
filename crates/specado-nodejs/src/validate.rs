//! Validation function implementation
//!
//! This module implements validation functions for prompt and provider specifications.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::Value;

use crate::error::SpecadoError;
use crate::types::{ValidationResult, ValidationError, ValidationWarning};

/// Schema types for validation
#[napi]
pub enum SchemaType {
    /// Prompt specification schema
    Prompt,
    /// Provider specification schema
    Provider,
}

/// Validate a specification against its schema
///
/// # Arguments
/// * `spec` - The specification to validate (as JSON object)
/// * `schema_type` - The type of schema to validate against
///
/// # Returns
/// A `ValidationResult` containing validation status, errors, and warnings
///
/// # Example
/// ```typescript
/// import { validate, SchemaType } from '@specado/nodejs';
/// 
/// const prompt = {
///   model_class: "Chat",
///   messages: [
///     { role: "user", content: "Hello!" }
///   ],
///   strict_mode: "standard"
/// };
/// 
/// const result = validate(prompt, SchemaType.Prompt);
/// if (result.valid) {
///   console.log("Prompt is valid!");
/// } else {
///   console.error("Validation errors:", result.errors);
/// }
/// ```
#[napi]
pub fn validate(spec: Value, schema_type: SchemaType) -> Result<ValidationResult> {
    // Convert schema type to string
    let schema_type_str = match schema_type {
        SchemaType::Prompt => "prompt",
        SchemaType::Provider => "provider",
    };

    // Serialize the spec to JSON string
    let spec_json = serde_json::to_string(&spec)
        .map_err(|e| Error::new(
            Status::InvalidArg,
            format!("Failed to serialize spec: {}", e)
        ))?;

    // Call the validation function
    let result = validate_internal(&spec_json, schema_type_str)?;

    Ok(result)
}

/// Internal validation function that calls the FFI validation logic
fn validate_internal(spec_json: &str, schema_type: &str) -> Result<ValidationResult> {
    // Map schema type to FFI spec type parameter
    let spec_type = match schema_type {
        "prompt" => "prompt_spec",
        "provider" => "provider_spec",
        _ => {
            return Err(Error::new(
                Status::InvalidArg,
                format!("Unknown schema type: {}", schema_type)
            ));
        }
    };
    
    // Call FFI validation function
    let result_json = unsafe {
        let mut out_ptr: *mut std::os::raw::c_char = std::ptr::null_mut();
        
        let spec_cstr = std::ffi::CString::new(spec_json)
            .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid spec JSON: {}", e)))?;
        let type_cstr = std::ffi::CString::new(spec_type)
            .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid spec type: {}", e)))?;
        let mode_cstr = std::ffi::CString::new("standard")
            .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid mode: {}", e)))?;
        
        let result = specado_ffi::specado_validate(
            spec_cstr.as_ptr(),
            type_cstr.as_ptr(),
            mode_cstr.as_ptr(),
            &mut out_ptr,
        );
        
        if result != specado_ffi::SpecadoResult::Success {
            // Get error message from FFI
            let error_msg = specado_ffi::specado_get_last_error();
            let error_str = if error_msg.is_null() {
                "Validation failed".to_string()
            } else {
                std::ffi::CStr::from_ptr(error_msg)
                    .to_string_lossy()
                    .to_string()
            };
            
            return Err(Error::new(Status::GenericFailure, error_str));
        }
        
        if out_ptr.is_null() {
            return Err(Error::new(Status::GenericFailure, "Validation returned null result"));
        }
        
        let result_cstr = std::ffi::CStr::from_ptr(out_ptr);
        let result_string = result_cstr.to_string_lossy().to_string();
        
        // Free the allocated string
        specado_ffi::specado_string_free(out_ptr);
        
        result_string
    };
    
    // Parse the result JSON from FFI
    let validation_response: serde_json::Value = serde_json::from_str(&result_json)
        .map_err(|e| Error::new(
            Status::GenericFailure,
            format!("Failed to parse validation result: {}", e)
        ))?;
    
    // Convert FFI validation result to our Node.js ValidationResult type
    let valid = validation_response.get("valid")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    let errors = validation_response.get("errors")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| {
                    Some(ValidationError {
                        path: e.get("path")?.as_str()?.to_string(),
                        message: e.get("message")?.as_str()?.to_string(),
                        code: e.get("code")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_else(Vec::new);
    
    let warnings = validation_response.get("warnings")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|w| {
                    Some(ValidationWarning {
                        path: w.get("path")?.as_str()?.to_string(),
                        message: w.get("message")?.as_str()?.to_string(),
                        code: w.get("code")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_else(Vec::new);
    
    let schema_version = validation_response.get("schema_version")
        .and_then(|v| v.as_str())
        .unwrap_or("1.0")
        .to_string();
    
    Ok(ValidationResult {
        valid,
        errors,
        warnings,
        schema_version,
    })
}

// Custom validation functions removed - now using FFI validation