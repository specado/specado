//! Run function implementation
//!
//! This module implements the async run function that executes
//! provider requests and returns uniform responses.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::Value;
use std::collections::HashMap;

use crate::types::{ProviderRequest, ProviderSpec, RunOptions, UniformResponse, UsageStats, ResponseMetadata};

/// Execute a provider request asynchronously
///
/// # Arguments
/// * `request` - The provider request to execute
/// * `provider_spec` - The provider specification
/// * `options` - Optional execution options
///
/// # Returns
/// A Promise that resolves to a `UniformResponse`
///
/// # Example
/// ```typescript
/// import { run } from '@specado/nodejs';
/// 
/// const request = {
///   provider: providerSpec,
///   request: translatedRequest,
///   credentials: { "api_key": "your-key" }
/// };
/// 
/// const response = await run(request, providerSpec, { timeout_seconds: 30 });
/// console.log(response.content);
/// ```
#[napi]
pub async fn run(
    request: ProviderRequest,
    provider_spec: ProviderSpec,
    options: Option<RunOptions>,
) -> Result<UniformResponse> {
    // Set default options
    let opts = options.unwrap_or(RunOptions {
        timeout_seconds: Some(30),
        max_retries: Some(3),
        follow_redirects: Some(true),
        user_agent: Some("Specado Node.js Binding/1.0".to_string()),
    });

    // Build the run request
    let run_request = build_run_request(&request, &provider_spec, &opts)?;

    // Serialize request to JSON for FFI layer
    let request_json = serde_json::to_string(&run_request)
        .map_err(|e| Error::new(
            Status::InvalidArg,
            format!("Failed to serialize request: {}", e)
        ))?;

    // Execute the request asynchronously
    let response_json = run_async_internal(request_json, opts.timeout_seconds.unwrap_or(30)).await?;

    // Parse the response
    let response_value: Value = serde_json::from_str(&response_json)
        .map_err(|e| Error::new(
            Status::GenericFailure,
            format!("Failed to parse response: {}", e)
        ))?;

    // Convert to UniformResponse
    let uniform_response = parse_uniform_response(response_value, &provider_spec)?;

    Ok(uniform_response)
}

/// Build the internal run request structure
fn build_run_request(
    request: &ProviderRequest,
    provider_spec: &ProviderSpec,
    options: &RunOptions,
) -> Result<Value> {
    let mut run_request = serde_json::json!({
        "provider": provider_spec,
        "request": request.request,
        "timeout_seconds": options.timeout_seconds.unwrap_or(30),
        "max_retries": options.max_retries.unwrap_or(3),
        "follow_redirects": options.follow_redirects.unwrap_or(true)
    });

    // Add credentials if provided
    if let Some(ref creds) = request.credentials {
        run_request["credentials"] = serde_json::to_value(creds)
            .map_err(|e| Error::new(
                Status::InvalidArg,
                format!("Failed to serialize credentials: {}", e)
            ))?;
    }

    // Add headers if provided
    if let Some(ref headers) = request.headers {
        run_request["headers"] = serde_json::to_value(headers)
            .map_err(|e| Error::new(
                Status::InvalidArg,
                format!("Failed to serialize headers: {}", e)
            ))?;
    }

    // Add user agent
    if let Some(ref user_agent) = options.user_agent {
        run_request["user_agent"] = Value::String(user_agent.clone());
    }

    Ok(run_request)
}

/// Execute the request asynchronously using Tokio
async fn run_async_internal(request_json: String, timeout_seconds: u32) -> Result<String> {
    // Use tokio::task::spawn_blocking to call the FFI function
    // since FFI calls are blocking
    let result = tokio::task::spawn_blocking(move || {
        run_sync_internal(request_json, timeout_seconds)
    }).await;

    match result {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(e)) => Err(e),
        Err(join_error) => Err(Error::new(
            Status::GenericFailure,
            format!("Task execution failed: {}", join_error)
        )),
    }
}

/// Synchronous internal run function that calls FFI
fn run_sync_internal(request_json: String, timeout_seconds: u32) -> Result<String> {
    unsafe {
        let mut out_ptr: *mut std::os::raw::c_char = std::ptr::null_mut();
        
        let request_cstr = std::ffi::CString::new(request_json)
            .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid request JSON: {}", e)))?;

        let result = specado_ffi::specado_run(
            request_cstr.as_ptr(),
            timeout_seconds as std::os::raw::c_int,
            &mut out_ptr,
        );

        if result != specado_ffi::SpecadoResult::Success {
            // Get error message from FFI
            let error_msg = specado_ffi::specado_get_last_error();
            let error_str = if error_msg.is_null() {
                "Request execution failed".to_string()
            } else {
                std::ffi::CStr::from_ptr(error_msg)
                    .to_string_lossy()
                    .to_string()
            };
            
            return Err(Error::new(Status::GenericFailure, error_str));
        }

        if out_ptr.is_null() {
            return Err(Error::new(Status::GenericFailure, "Request returned null result"));
        }

        let result_cstr = std::ffi::CStr::from_ptr(out_ptr);
        let result_string = result_cstr.to_string_lossy().to_string();
        
        // Free the allocated string
        specado_ffi::specado_string_free(out_ptr);
        
        Ok(result_string)
    }
}

/// Parse the raw response into a UniformResponse
fn parse_uniform_response(response: Value, provider_spec: &ProviderSpec) -> Result<UniformResponse> {
    let response_obj = response.as_object()
        .ok_or_else(|| Error::new(Status::GenericFailure, "Response must be an object"))?;

    // Extract content
    let content = response_obj.get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Extract usage stats
    let usage = response_obj.get("usage")
        .and_then(|v| parse_usage_stats(v).ok());

    // Extract finish reason
    let finish_reason = response_obj.get("finish_reason")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Extract tool calls
    let tool_calls = response_obj.get("tool_calls")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|tool_call| parse_tool_call(tool_call).ok())
                .collect()
        });

    // Build metadata
    let metadata = ResponseMetadata {
        provider: provider_spec.name.clone(),
        model: response_obj.get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        request_id: response_obj.get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        response_time_ms: response_obj.get("response_time_ms")
            .and_then(|v| v.as_u64())
            .map(|v| v as f64)
            .unwrap_or(0.0),
    };

    Ok(UniformResponse {
        content,
        usage,
        metadata,
        tool_calls,
        finish_reason,
    })
}

/// Parse usage statistics from response
fn parse_usage_stats(usage_value: &Value) -> Result<UsageStats> {
    let usage_obj = usage_value.as_object()
        .ok_or_else(|| Error::new(Status::GenericFailure, "Usage must be an object"))?;

    let input_tokens = usage_obj.get("prompt_tokens")
        .or_else(|| usage_obj.get("input_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let output_tokens = usage_obj.get("completion_tokens")
        .or_else(|| usage_obj.get("output_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let total_tokens = usage_obj.get("total_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or((input_tokens + output_tokens) as u64) as u32;

    let estimated_cost = usage_obj.get("estimated_cost")
        .and_then(|v| v.as_f64());

    Ok(UsageStats {
        input_tokens,
        output_tokens,
        total_tokens,
        estimated_cost,
    })
}

/// Parse tool call from response
fn parse_tool_call(tool_call_value: &Value) -> Result<crate::types::ToolCall> {
    let tool_call_obj = tool_call_value.as_object()
        .ok_or_else(|| Error::new(Status::GenericFailure, "Tool call must be an object"))?;

    let id = tool_call_obj.get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::new(Status::GenericFailure, "Tool call missing id"))?
        .to_string();

    let name = tool_call_obj.get("function")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .or_else(|| tool_call_obj.get("name").and_then(|v| v.as_str()))
        .ok_or_else(|| Error::new(Status::GenericFailure, "Tool call missing name"))?
        .to_string();

    let arguments = tool_call_obj.get("function")
        .and_then(|v| v.get("arguments"))
        .or_else(|| tool_call_obj.get("arguments"))
        .cloned()
        .unwrap_or(Value::Object(serde_json::Map::new()));

    Ok(crate::types::ToolCall {
        id,
        name,
        arguments,
    })
}