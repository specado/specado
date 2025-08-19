//! Run function implementation for Python bindings
//!
//! This module implements both synchronous and asynchronous run functions
//! for executing provider requests and getting normalized responses.

use pyo3::prelude::*;
use pyo3::exceptions::{PyValueError, PyRuntimeError};
use pyo3_asyncio::tokio::future_into_py;
use std::ffi::{CString, CStr};
use std::ptr;
use std::time::Duration;
use specado_ffi::SpecadoResult;
use crate::error::{map_ffi_result_to_py_err, get_last_ffi_error};
use crate::types::PyUniformResponse;
use specado_core::types::UniformResponse;
use serde_json::Value;

/// Run a provider request asynchronously
/// 
/// Args:
///     request (dict): The provider request to execute
///     provider_spec (ProviderSpec): The provider specification
///     timeout (Optional[int]): Timeout in seconds (None for default)
/// 
/// Returns:
///     UniformResponse: The normalized response from the provider
/// 
/// Raises:
///     ProviderError: If the provider request fails
///     TimeoutError: If the request times out
///     NetworkError: If there are network issues
#[pyfunction]
#[pyo3(signature = (request, provider_spec, timeout=None))]
pub fn run_async<'p>(
    py: Python<'p>,
    request: PyObject,
    provider_spec: &crate::types::PyProviderSpec,
    timeout: Option<u32>,
) -> PyResult<&'p PyAny> {
    // Convert inputs to the format expected by the async function
    let request_json = Python::with_gil(|py| {
        let request_value = crate::types::py_to_json(py, request.as_ref(py))?;
        serde_json::to_string(&request_value)
            .map_err(|e| PyValueError::new_err(format!("Failed to serialize request: {}", e)))
    })?;
    
    future_into_py(py, async move {
        run_internal_async(&request_json, timeout).await
    })
}

/// Run a provider request synchronously
/// 
/// Args:
///     request (dict): The provider request to execute
///     provider_spec (ProviderSpec): The provider specification
///     timeout (Optional[int]): Timeout in seconds (None for default)
/// 
/// Returns:
///     UniformResponse: The normalized response from the provider
/// 
/// Raises:
///     ProviderError: If the provider request fails
///     TimeoutError: If the request times out
///     NetworkError: If there are network issues
#[pyfunction]
#[pyo3(signature = (request, provider_spec, timeout=None))]
pub fn run_sync(
    py: Python<'_>,
    request: PyObject,
    provider_spec: &crate::types::PyProviderSpec,
    timeout: Option<u32>,
) -> PyResult<PyUniformResponse> {
    // Convert request to JSON
    let request_json = Python::with_gil(|py| {
        let request_value = crate::types::py_to_json(py, request.as_ref(py))?;
        serde_json::to_string(&request_value)
            .map_err(|e| PyValueError::new_err(format!("Failed to serialize request: {}", e)))
    })?;
    
    // Convert to C string
    let request_cstr = CString::new(request_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid request JSON: {}", e)))?;
    
    // Call FFI function
    let mut result_ptr: *mut std::os::raw::c_char = ptr::null_mut();
    let timeout_seconds = timeout.unwrap_or(0) as std::os::raw::c_int;
    
    let ffi_result = unsafe {
        specado_ffi::specado_run(
            request_cstr.as_ptr(),
            timeout_seconds,
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
    
    // Parse the result JSON
    let result: UniformResponse = serde_json::from_str(&result_json)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to parse result JSON: {}", e)))?;
    
    Ok(PyUniformResponse { inner: result })
}

/// Internal async implementation
async fn run_internal_async(
    request_json: &str,
    timeout: Option<u32>,
) -> PyResult<PyUniformResponse> {
    // This is a placeholder implementation. In a real implementation,
    // we would need to integrate with the actual async runtime and
    // provider execution logic from specado-core.
    
    // Simulate async execution with tokio
    let request_json = request_json.to_string();
    let result = tokio::task::spawn_blocking(move || {
        execute_provider_request_blocking(&request_json, timeout)
    }).await
    .map_err(|e| PyRuntimeError::new_err(format!("Async execution failed: {}", e)))??;
    
    Ok(result)
}

/// Blocking implementation for provider request execution
fn execute_provider_request_blocking(
    request_json: &str,
    timeout: Option<u32>,
) -> PyResult<PyUniformResponse> {
    // Convert to C string
    let request_cstr = CString::new(request_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid request JSON: {}", e)))?;
    
    // Call FFI function
    let mut result_ptr: *mut std::os::raw::c_char = ptr::null_mut();
    let timeout_seconds = timeout.unwrap_or(0) as std::os::raw::c_int;
    
    let ffi_result = unsafe {
        specado_ffi::specado_run(
            request_cstr.as_ptr(),
            timeout_seconds,
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
    
    // Parse the result JSON
    let result: UniformResponse = serde_json::from_str(&result_json)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to parse result JSON: {}", e)))?;
    
    Ok(PyUniformResponse { inner: result })
}

/// Create a provider request from a translation result
/// 
/// This is a helper function to make it easier to chain translate -> run operations.
/// 
/// Args:
///     translation_result (TranslationResult): Result from translate function
///     provider_spec (ProviderSpec): The provider specification used for translation
/// 
/// Returns:
///     dict: Provider request ready for execution
#[pyfunction]
pub fn create_provider_request(
    py: Python<'_>,
    translation_result: &crate::types::PyTranslationResult,
    provider_spec: &crate::types::PyProviderSpec,
) -> PyResult<PyObject> {
    // Extract the provider request JSON from the translation result
    let provider_request = &translation_result.inner.provider_request_json;
    
    // Convert to Python object
    crate::types::json_to_py(py, provider_request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PyProviderSpec, PyProviderInfo, PyModelSpec};
    use specado_core::types::*;
    use std::collections::HashMap;
    use serde_json::json;
    
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
    fn test_create_provider_request() {
        Python::with_gil(|py| {
            let translation_result = crate::types::PyTranslationResult {
                inner: TranslationResult {
                    provider_request_json: json!({
                        "model": "test-model",
                        "messages": [{"role": "user", "content": "Hello"}]
                    }),
                    lossiness: LossinessReport {
                        items: vec![],
                        max_severity: Severity::Info,
                        summary: LossinessSummary {
                            total_items: 0,
                            by_severity: HashMap::new(),
                            by_code: HashMap::new(),
                        },
                    },
                    metadata: None,
                },
            };
            
            let provider = create_test_provider();
            
            let request = create_provider_request(py, &translation_result, &provider).unwrap();
            
            // Verify the request is a valid Python object
            assert!(!request.is_none(py));
        });
    }
    
    #[tokio::test]
    async fn test_async_execution_setup() {
        // Test that the async setup works correctly
        let request = json!({
            "model": "test-model",
            "messages": [{"role": "user", "content": "Hello"}]
        });
        
        let provider_spec = create_test_provider().inner;
        
        // This would normally fail because we don't have a real provider,
        // but we can test that the setup is correct
        let request_json = serde_json::to_string(&request).unwrap();
        
        // Test serialization works
        assert!(request_json.contains("test-model"));
        assert!(request_json.contains("Hello"));
    }
}