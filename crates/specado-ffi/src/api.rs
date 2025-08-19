//! FFI API function definitions
//!
//! This module contains the extern "C" functions that form
//! the public API of the Specado FFI layer.

use std::os::raw::{c_char, c_int};

use crate::types::{SpecadoResult, SpecadoContext};
use crate::memory::{allocate_string, c_str_to_string, set_last_error, clear_last_error};
use crate::error::validate_ptr;
use crate::ffi_boundary;

/// Initialize a new Specado context
///
/// # Safety
/// The returned context must be freed with `specado_context_free`
#[no_mangle]
pub unsafe extern "C" fn specado_context_new() -> *mut SpecadoContext {
    clear_last_error();
    
    // For now, we don't have any context state
    // In the future, this could hold configuration, caches, etc.
    let context = Box::new(SpecadoContextImpl {
        initialized: true,
    });
    
    Box::into_raw(context) as *mut SpecadoContext
}

/// Free a Specado context
///
/// # Safety
/// The context pointer must have been created by `specado_context_new`
#[no_mangle]
pub unsafe extern "C" fn specado_context_free(context: *mut SpecadoContext) {
    if context.is_null() {
        return;
    }
    
    let _ = Box::from_raw(context as *mut SpecadoContextImpl);
    // Context is freed when Box is dropped
}

/// Translate a prompt to a provider-specific request
///
/// # Parameters
/// - `prompt_json`: JSON string containing the prompt and configuration
/// - `provider_spec_json`: JSON string containing the provider specification
/// - `model_id`: The model identifier to use
/// - `mode`: Translation mode ("standard" or "strict")
/// - `out_json`: Output parameter for the resulting JSON string
///
/// # Returns
/// A SpecadoResult indicating success or failure
///
/// # Safety
/// - All string pointers must be valid null-terminated C strings
/// - The output string must be freed with `specado_string_free`
#[no_mangle]
pub unsafe extern "C" fn specado_translate(
    prompt_json: *const c_char,
    provider_spec_json: *const c_char,
    model_id: *const c_char,
    mode: *const c_char,
    out_json: *mut *mut c_char,
) -> SpecadoResult {
    ffi_boundary!({
        clear_last_error();
        
        // Validate inputs
        validate_ptr(prompt_json, "prompt_json")?;
        validate_ptr(provider_spec_json, "provider_spec_json")?;
        validate_ptr(model_id, "model_id")?;
        validate_ptr(mode, "mode")?;
        validate_ptr(out_json, "out_json")?;
        
        // Convert C strings to Rust strings
        let prompt_str = c_str_to_string(prompt_json)?;
        let provider_spec_str = c_str_to_string(provider_spec_json)?;
        let model_id_str = c_str_to_string(model_id)?;
        let mode_str = c_str_to_string(mode)?;
        
        // Parse JSON inputs
        let prompt: serde_json::Value = serde_json::from_str(&prompt_str)
            .map_err(|e| {
                set_last_error(format!("Invalid prompt JSON: {}", e));
                SpecadoResult::JsonError
            })?;
        
        let provider_spec: specado_core::types::ProviderSpec = 
            serde_json::from_str(&provider_spec_str)
                .map_err(|e| {
                    set_last_error(format!("Invalid provider spec JSON: {}", e));
                    SpecadoResult::JsonError
                })?;
        
        // Perform translation
        let result = translate_internal(prompt, provider_spec, model_id_str, mode_str)?;
        
        // Serialize result to JSON
        let result_json = serde_json::to_string(&result)
            .map_err(|e| {
                set_last_error(format!("Failed to serialize result: {}", e));
                SpecadoResult::JsonError
            })?;
        
        // Allocate output string
        *out_json = allocate_string(&result_json);
        if (*out_json).is_null() {
            return Err(SpecadoResult::MemoryError);
        }
        
        Ok(SpecadoResult::Success)
    })
}

/// Run a translated request against a provider
///
/// # Parameters
/// - `provider_request_json`: JSON string containing the provider request
/// - `timeout_seconds`: Timeout in seconds (0 for default)
/// - `out_response_json`: Output parameter for the response JSON
///
/// # Returns
/// A SpecadoResult indicating success or failure
///
/// # Safety
/// - All string pointers must be valid null-terminated C strings
/// - The output string must be freed with `specado_string_free`
#[no_mangle]
pub unsafe extern "C" fn specado_run(
    provider_request_json: *const c_char,
    timeout_seconds: c_int,
    out_response_json: *mut *mut c_char,
) -> SpecadoResult {
    ffi_boundary!({
        clear_last_error();
        
        // Validate inputs
        validate_ptr(provider_request_json, "provider_request_json")?;
        validate_ptr(out_response_json, "out_response_json")?;
        
        // Convert C string to Rust string
        let request_str = c_str_to_string(provider_request_json)?;
        
        // Parse request JSON
        let request: serde_json::Value = serde_json::from_str(&request_str)
            .map_err(|e| {
                set_last_error(format!("Invalid request JSON: {}", e));
                SpecadoResult::JsonError
            })?;
        
        // Execute the request
        let timeout = if timeout_seconds > 0 {
            Some(std::time::Duration::from_secs(timeout_seconds as u64))
        } else {
            None
        };
        
        let response = run_internal(request, timeout)?;
        
        // Serialize response to JSON
        let response_json = serde_json::to_string(&response)
            .map_err(|e| {
                set_last_error(format!("Failed to serialize response: {}", e));
                SpecadoResult::JsonError
            })?;
        
        // Allocate output string
        *out_response_json = allocate_string(&response_json);
        if (*out_response_json).is_null() {
            return Err(SpecadoResult::MemoryError);
        }
        
        Ok(SpecadoResult::Success)
    })
}

/// Validate a specification against its schema
///
/// # Parameters
/// - `spec_json`: JSON string containing the specification to validate
/// - `spec_type`: Type of specification ("prompt_spec" or "provider_spec")
/// - `mode`: Validation mode ("basic", "partial", or "strict")
/// - `out_result_json`: Output parameter for the validation result JSON
///
/// # Returns
/// A SpecadoResult indicating success or failure
///
/// # Safety
/// - All string pointers must be valid null-terminated C strings
/// - The output string must be freed with `specado_string_free`
#[no_mangle]
pub unsafe extern "C" fn specado_validate(
    spec_json: *const c_char,
    spec_type: *const c_char,
    mode: *const c_char,
    out_result_json: *mut *mut c_char,
) -> SpecadoResult {
    ffi_boundary!({
        clear_last_error();
        
        // Validate inputs
        validate_ptr(spec_json, "spec_json")?;
        validate_ptr(spec_type, "spec_type")?;
        validate_ptr(mode, "mode")?;
        validate_ptr(out_result_json, "out_result_json")?;
        
        // Convert C strings to Rust strings
        let spec_str = c_str_to_string(spec_json)?;
        let type_str = c_str_to_string(spec_type)?;
        let mode_str = c_str_to_string(mode)?;
        
        // Perform validation
        let result = crate::validate::validate_json(&spec_str, &type_str, &mode_str)?;
        
        // Allocate output string
        *out_result_json = allocate_string(&result);
        if (*out_result_json).is_null() {
            return Err(SpecadoResult::MemoryError);
        }
        
        Ok(SpecadoResult::Success)
    })
}

/// Get version information
///
/// # Returns
/// A static string containing version information
///
/// # Safety
/// The returned string should NOT be freed
#[no_mangle]
pub unsafe extern "C" fn specado_version() -> *const c_char {
    concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

// Internal implementation structures
struct SpecadoContextImpl {
    #[allow(dead_code)]
    initialized: bool,
}

// Internal implementation delegates to modules
fn translate_internal(
    prompt: serde_json::Value,
    provider_spec: specado_core::types::ProviderSpec,
    model_id: String,
    mode: String,
) -> Result<serde_json::Value, SpecadoResult> {
    // Convert to string for the translate module
    let prompt_json = serde_json::to_string(&prompt)
        .map_err(|e| {
            set_last_error(format!("Failed to serialize prompt: {}", e));
            SpecadoResult::JsonError
        })?;
    
    // Call the translate function
    let result_json = crate::translate::translate(&prompt_json, &provider_spec, &model_id, &mode)?;
    
    // Parse back to Value
    serde_json::from_str(&result_json)
        .map_err(|e| {
            set_last_error(format!("Failed to parse translation result: {}", e));
            SpecadoResult::JsonError
        })
}

fn run_internal(
    request: serde_json::Value,
    timeout: Option<std::time::Duration>,
) -> Result<serde_json::Value, SpecadoResult> {
    // Build the run input
    let run_input = serde_json::json!({
        "request": request,
        // Additional fields would be added here based on context
    });
    
    let request_json = serde_json::to_string(&run_input)
        .map_err(|e| {
            set_last_error(format!("Failed to serialize request: {}", e));
            SpecadoResult::JsonError
        })?;
    
    let timeout_seconds = timeout.map(|d| d.as_secs() as u32);
    
    // Call the run function
    let result_json = crate::run::run_sync(&request_json, timeout_seconds)?;
    
    // Parse back to Value
    serde_json::from_str(&result_json)
        .map_err(|e| {
            set_last_error(format!("Failed to parse run result: {}", e));
            SpecadoResult::JsonError
        })
}