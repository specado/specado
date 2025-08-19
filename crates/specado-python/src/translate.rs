//! Translation function implementation for Python bindings
//!
//! This module implements the translate function that converts
//! prompts to provider-specific requests.

use pyo3::prelude::*;
use pyo3::exceptions::{PyValueError, PyRuntimeError};
use std::ffi::{CString, CStr};
use std::ptr;
use specado_ffi::SpecadoResult;
use crate::error::{map_ffi_result_to_py_err, get_last_ffi_error};
use crate::types::{PyPromptSpec, PyProviderSpec, PyTranslationResult};
use specado_core::types::TranslationResult;

/// Translate a prompt to a provider-specific request
/// 
/// Args:
///     prompt (PromptSpec): The prompt specification to translate
///     provider_spec (ProviderSpec): The provider specification
///     model_id (str): The model identifier to use
///     mode (str): Translation mode ("standard" or "strict"), defaults to "standard"
/// 
/// Returns:
///     TranslationResult: The translated provider request with lossiness information
/// 
/// Raises:
///     TranslationError: If translation fails
///     ValidationError: If input validation fails
///     ProviderError: If provider or model is not found
#[pyfunction]
#[pyo3(signature = (prompt, provider_spec, model_id, mode="standard"))]
pub fn translate(
    prompt: &PyPromptSpec,
    provider_spec: &PyProviderSpec,
    model_id: &str,
    mode: &str,
) -> PyResult<PyTranslationResult> {
    // Serialize inputs to JSON
    let prompt_json = serde_json::to_string(&prompt.inner)
        .map_err(|e| PyValueError::new_err(format!("Failed to serialize prompt: {}", e)))?;
    
    let provider_spec_json = serde_json::to_string(&provider_spec.inner)
        .map_err(|e| PyValueError::new_err(format!("Failed to serialize provider spec: {}", e)))?;
    
    // Convert strings to C strings
    let prompt_cstr = CString::new(prompt_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid prompt JSON: {}", e)))?;
    let provider_spec_cstr = CString::new(provider_spec_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid provider spec JSON: {}", e)))?;
    let model_id_cstr = CString::new(model_id)
        .map_err(|e| PyValueError::new_err(format!("Invalid model ID: {}", e)))?;
    let mode_cstr = CString::new(mode)
        .map_err(|e| PyValueError::new_err(format!("Invalid mode: {}", e)))?;
    
    // Call FFI function
    let mut result_ptr: *mut std::os::raw::c_char = ptr::null_mut();
    
    let ffi_result = unsafe {
        specado_ffi::specado_translate(
            prompt_cstr.as_ptr(),
            provider_spec_cstr.as_ptr(),
            model_id_cstr.as_ptr(),
            mode_cstr.as_ptr(),
            &mut result_ptr,
        )
    };
    
    // Check for success
    if ffi_result != SpecadoResult::Success {
        let error_msg = get_last_ffi_error();
        return Err(map_ffi_result_to_py_err(ffi_result, error_msg));
    }
    
    // Convert result back to Rust
    if result_ptr.is_null() {
        return Err(PyRuntimeError::new_err("FFI returned null result"));
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
    
    // Parse the result JSON as TranslationResult (not the old wrapper)
    let result: TranslationResult = serde_json::from_str(&result_json)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to parse result JSON: {}", e)))?;
    
    Ok(PyTranslationResult { inner: result })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PyMessage, PyProviderInfo, PyModelSpec};
    use specado_core::types::*;
    use std::collections::HashMap;
    use serde_json::json;
    
    fn create_test_prompt() -> PyPromptSpec {
        let message = PyMessage {
            inner: Message {
                role: MessageRole::User,
                content: "Hello, world!".to_string(),
                name: None,
                metadata: None,
            },
        };

        PyPromptSpec {
            inner: PromptSpec {
                model_class: "Chat".to_string(),
                messages: vec![message.inner.clone()],
                tools: None,
                tool_choice: None,
                response_format: None,
                sampling: None,
                limits: None,
                media: None,
                strict_mode: StrictMode::Warn,
            },
        }
    }
    
    fn create_test_provider() -> PyProviderSpec {
        let provider_info = PyProviderInfo {
            inner: ProviderInfo {
                name: "test".to_string(),
                base_url: "https://api.test.com".to_string(),
                headers: HashMap::new(),
            }
        };
        
        let model_spec = PyModelSpec {
            inner: ModelSpec {
                id: "test-model".to_string(),
                aliases: None,
                family: "test".to_string(),
                endpoints: Endpoints {
                    chat_completion: EndpointConfig {
                        method: "POST".to_string(),
                        path: "/chat/completions".to_string(),
                        protocol: "https".to_string(),
                        query: None,
                        headers: None,
                    },
                    streaming_chat_completion: EndpointConfig {
                        method: "POST".to_string(),
                        path: "/chat/completions".to_string(),
                        protocol: "https".to_string(),
                        query: None,
                        headers: None,
                    },
                },
                input_modes: InputModes {
                    messages: true,
                    single_text: false,
                    images: false,
                },
                tooling: ToolingConfig {
                    tools_supported: false,
                    parallel_tool_calls_default: false,
                    can_disable_parallel_tool_calls: false,
                    disable_switch: None,
                },
                json_output: JsonOutputConfig {
                    native_param: false,
                    strategy: "none".to_string(),
                },
                parameters: json!({}),
                constraints: Constraints {
                    system_prompt_location: "first".to_string(),
                    forbid_unknown_top_level_fields: false,
                    mutually_exclusive: vec![],
                    resolution_preferences: vec![],
                    limits: ConstraintLimits {
                        max_tool_schema_bytes: 1000,
                        max_system_prompt_bytes: 1000,
                    },
                },
                mappings: Mappings {
                    paths: HashMap::new(),
                    flags: HashMap::new(),
                },
                response_normalization: ResponseNormalization {
                    sync: SyncNormalization {
                        content_path: "$.choices[0].message.content".to_string(),
                        finish_reason_path: "$.choices[0].finish_reason".to_string(),
                        finish_reason_map: HashMap::new(),
                    },
                    stream: StreamNormalization {
                        protocol: "sse".to_string(),
                        event_selector: EventSelector {
                            type_path: "$.type".to_string(),
                            routes: vec![],
                        },
                    },
                },
            }
        };
        
        PyProviderSpec {
            inner: ProviderSpec {
                spec_version: "1.0.0".to_string(),
                provider: provider_info.inner.clone(),
                models: vec![model_spec.inner.clone()],
            },
        }
    }
    
    #[test]
    fn test_translate_serialization() {
        let prompt = create_test_prompt();
        let provider = create_test_provider();
        
        // Test that serialization works
        let prompt_json = serde_json::to_string(&prompt.inner).unwrap();
        let provider_json = serde_json::to_string(&provider.inner).unwrap();
        
        assert!(prompt_json.contains("Hello, world!"));
        assert!(provider_json.contains("test-model"));
    }
}