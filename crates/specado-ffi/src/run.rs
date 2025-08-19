//! Run FFI implementation
//!
//! This module implements the run function that executes
//! provider requests and returns normalized responses.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use specado_core::{
    http::HttpClient,
    types::ProviderSpec,
};

use crate::types::SpecadoResult;
use crate::memory::set_last_error;
use crate::error::map_core_error;

/// Input structure for run operation
#[derive(Debug, Deserialize)]
pub struct RunInput {
    /// Provider specification
    pub provider_spec: ProviderSpec,
    /// Model ID
    pub model_id: String,
    /// The request to execute
    pub request: Value,
    /// Optional configuration
    #[allow(dead_code)]
    pub config: Option<RunConfig>,
}

/// Run configuration
#[derive(Debug, Deserialize)]
pub struct RunConfig {
    /// Timeout in seconds
    #[allow(dead_code)]
    pub timeout_seconds: Option<u32>,
    /// Retry attempts
    #[allow(dead_code)]
    pub max_retries: Option<u32>,
    /// Enable fallback strategies
    #[allow(dead_code)]
    pub enable_fallback: Option<bool>,
}

/// Run result
#[derive(Debug, Serialize)]
pub struct RunResult {
    /// Success status
    pub success: bool,
    /// Response from provider
    pub response: Option<UniformResponse>,
    /// Error information
    pub error: Option<ErrorInfo>,
    /// Timing information
    pub timing: Option<TimingInfo>,
}

/// Uniform response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct UniformResponse {
    /// Content from the response
    pub content: Option<String>,
    /// Role (usually "assistant")
    pub role: String,
    /// Finish reason
    pub finish_reason: Option<String>,
    /// Usage information
    pub usage: Option<Usage>,
    /// Model used
    pub model: String,
    /// Response ID
    pub id: Option<String>,
}

/// Usage information
#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    /// Prompt tokens
    pub prompt_tokens: Option<u32>,
    /// Completion tokens
    pub completion_tokens: Option<u32>,
    /// Total tokens
    pub total_tokens: Option<u32>,
}

/// Error information
#[derive(Debug, Serialize)]
pub struct ErrorInfo {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Provider error code if available
    pub provider_code: Option<String>,
    /// Retry after seconds
    pub retry_after: Option<u32>,
}

/// Timing information
#[derive(Debug, Serialize)]
pub struct TimingInfo {
    /// Request start time
    pub start_time: String,
    /// Request end time
    pub end_time: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Execute a provider request
pub async fn run(
    request_json: &str,
    _timeout_seconds: Option<u32>,
) -> Result<String, SpecadoResult> {
    let start_time = std::time::Instant::now();
    let start_timestamp = chrono::Utc::now();
    
    // Parse the run input
    let input: RunInput = serde_json::from_str(request_json)
        .map_err(|e| {
            set_last_error(format!("Failed to parse request JSON: {}", e));
            SpecadoResult::JsonError
        })?;
    
    // Find the model
    let model = input.provider_spec
        .models
        .iter()
        .find(|m| m.id == input.model_id || 
              m.aliases.as_ref().is_some_and(|a| a.contains(&input.model_id)))
        .ok_or_else(|| {
            set_last_error(format!("Model '{}' not found", input.model_id));
            SpecadoResult::ModelNotFound
        })?;
    
    // Create HTTP client with default config
    // TODO: Add config support when HttpClientConfig is exposed
    let client = HttpClient::with_default_config(input.provider_spec.clone())
        .map_err(|e| {
            set_last_error(format!("Failed to create HTTP client: {}", e));
            map_core_error(e)
        })?;
    
    // Execute the request
    let response = client
        .execute_chat_completion(model, input.request.clone())
        .await
        .map_err(|e| {
            let _error_info = ErrorInfo {
                code: "REQUEST_FAILED".to_string(),
                message: e.to_string(),
                provider_code: None,
                retry_after: None,
            };
            
            set_last_error(format!("Request failed: {}", e));
            
            // Store error info for later if needed
            map_core_error(e)
        })?;
    
    // Normalize the response
    let normalized = normalize_response(response, &input.model_id)?;
    
    let end_timestamp = chrono::Utc::now();
    let duration = start_time.elapsed();
    
    // Build result
    let result = RunResult {
        success: true,
        response: Some(normalized),
        error: None,
        timing: Some(TimingInfo {
            start_time: start_timestamp.to_rfc3339(),
            end_time: end_timestamp.to_rfc3339(),
            duration_ms: duration.as_millis() as u64,
        }),
    };
    
    // Serialize result
    serde_json::to_string(&result)
        .map_err(|e| {
            set_last_error(format!("Failed to serialize result: {}", e));
            SpecadoResult::JsonError
        })
}

/// Normalize provider response to uniform format
fn normalize_response(response: Value, model_id: &str) -> Result<UniformResponse, SpecadoResult> {
    // Try to extract common fields from different provider formats
    
    // OpenAI format
    if let Some(choices) = response["choices"].as_array() {
        if let Some(first_choice) = choices.first() {
            let content = first_choice["message"]["content"]
                .as_str()
                .map(|s| s.to_string());
            
            let role = first_choice["message"]["role"]
                .as_str()
                .unwrap_or("assistant")
                .to_string();
            
            let finish_reason = first_choice["finish_reason"]
                .as_str()
                .map(|s| s.to_string());
            
            let usage = response["usage"].as_object().map(|u| Usage {
                prompt_tokens: u["prompt_tokens"].as_u64().map(|v| v as u32),
                completion_tokens: u["completion_tokens"].as_u64().map(|v| v as u32),
                total_tokens: u["total_tokens"].as_u64().map(|v| v as u32),
            });
            
            return Ok(UniformResponse {
                content,
                role,
                finish_reason,
                usage,
                model: response["model"].as_str().unwrap_or(model_id).to_string(),
                id: response["id"].as_str().map(|s| s.to_string()),
            });
        }
    }
    
    // Anthropic format
    if let Some(content) = response["content"].as_array() {
        if let Some(first_content) = content.first() {
            let text = first_content["text"]
                .as_str()
                .map(|s| s.to_string());
            
            let usage = response["usage"].as_object().map(|u| Usage {
                prompt_tokens: u["input_tokens"].as_u64().map(|v| v as u32),
                completion_tokens: u["output_tokens"].as_u64().map(|v| v as u32),
                total_tokens: None,
            });
            
            return Ok(UniformResponse {
                content: text,
                role: "assistant".to_string(),
                finish_reason: response["stop_reason"].as_str().map(|s| s.to_string()),
                usage,
                model: response["model"].as_str().unwrap_or(model_id).to_string(),
                id: response["id"].as_str().map(|s| s.to_string()),
            });
        }
    }
    
    // Fallback: return raw response as content
    Ok(UniformResponse {
        content: Some(response.to_string()),
        role: "assistant".to_string(),
        finish_reason: None,
        usage: None,
        model: model_id.to_string(),
        id: None,
    })
}

/// Synchronous wrapper for the async run function
pub fn run_sync(request_json: &str, timeout_seconds: Option<u32>) -> Result<String, SpecadoResult> {
    // Create a runtime for the async operation
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| {
            set_last_error(format!("Failed to create async runtime: {}", e));
            SpecadoResult::InternalError
        })?;
    
    // Execute the async function
    runtime.block_on(run(request_json, timeout_seconds))
}