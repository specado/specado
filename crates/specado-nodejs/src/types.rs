//! Type definitions for Node.js bindings
//!
//! This module provides TypeScript-compatible type definitions for all
//! Specado data structures and function parameters.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Prompt specification for LLM requests
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSpec {
    /// Model class (e.g., "Chat", "ReasoningChat")
    pub model_class: String,
    /// Conversation messages
    pub messages: Vec<Message>,
    /// Available tools for the model
    pub tools: Option<Vec<Tool>>,
    /// Tool selection strategy
    pub tool_choice: Option<ToolChoice>,
    /// Expected response format
    pub response_format: Option<ResponseFormat>,
    /// Sampling parameters
    pub sampling: Option<SamplingParams>,
    /// Token and output limits
    pub limits: Option<Limits>,
    /// Media inputs/outputs
    pub media: Option<MediaConfig>,
    /// Strictness mode for translation
    pub strict_mode: String,
}

/// Message in a conversation
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message sender
    pub role: String,
    /// Text content of the message
    pub content: String,
    /// Optional tool calls
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Optional tool results
    pub tool_results: Option<Vec<ToolResult>>,
    /// Optional metadata
    pub metadata: Option<HashMap<String, Value>>,
}

/// Tool definition
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name/identifier
    pub name: String,
    /// Tool description
    pub description: String,
    /// JSON schema for parameters
    pub parameters: Value,
    /// Whether the tool is required
    pub required: Option<bool>,
}

/// Tool choice strategy
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChoice {
    /// Choice type ("auto", "none", "required", "specific")
    pub choice_type: String,
    /// Specific tool name if choice_type is "specific"
    pub tool_name: Option<String>,
}

/// Response format specification
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    /// Format type ("text", "json_object", "json_schema")
    pub format_type: String,
    /// JSON schema if format_type is "json_schema"
    pub schema: Option<Value>,
}

/// Sampling parameters for generation
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingParams {
    /// Temperature for randomness (0.0 to 2.0)
    pub temperature: Option<f64>,
    /// Nucleus sampling threshold
    pub top_p: Option<f64>,
    /// Top-k sampling limit
    pub top_k: Option<u32>,
    /// Frequency penalty
    pub frequency_penalty: Option<f64>,
    /// Presence penalty
    pub presence_penalty: Option<f64>,
    /// Stop sequences
    pub stop: Option<Vec<String>>,
    /// Random seed
    pub seed: Option<u64>,
}

/// Token and output limits
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limits {
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Maximum input tokens
    pub max_input_tokens: Option<u32>,
    /// Maximum total tokens
    pub max_total_tokens: Option<u32>,
}

/// Media configuration
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaConfig {
    /// Supported input media types
    pub input_types: Option<Vec<String>>,
    /// Supported output media types
    pub output_types: Option<Vec<String>>,
    /// Maximum file size in bytes
    pub max_file_size: Option<u64>,
}

/// Tool call in a message
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool call ID
    pub id: String,
    /// Tool name
    pub name: String,
    /// Tool arguments as JSON
    pub arguments: Value,
}

/// Tool execution result
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Tool call ID this result corresponds to
    pub call_id: String,
    /// Result content
    pub content: String,
    /// Whether the tool call was successful
    pub success: bool,
    /// Error message if unsuccessful
    pub error: Option<String>,
}

/// Provider specification
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSpec {
    /// Provider name
    pub name: String,
    /// Provider version
    pub version: String,
    /// Base URL for API
    pub base_url: String,
    /// Authentication configuration
    pub auth: AuthConfig,
    /// Supported models
    pub models: Vec<ModelSpec>,
    /// Rate limiting configuration
    pub rate_limits: Option<RateLimitConfig>,
    /// Request/response mappings
    pub mappings: ProviderMappings,
}

/// Authentication configuration
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Authentication type ("api_key", "bearer", "oauth", etc.)
    pub auth_type: String,
    /// Header name for authentication
    pub header: Option<String>,
    /// Environment variable for credentials
    pub env_var: Option<String>,
}

/// Model specification
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpec {
    /// Model ID
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Model capabilities
    pub capabilities: Vec<String>,
    /// Context window size
    pub context_size: Option<u32>,
    /// Maximum output tokens
    pub max_output: Option<u32>,
    /// Cost per input token
    pub cost_per_input_token: Option<f64>,
    /// Cost per output token
    pub cost_per_output_token: Option<f64>,
}

/// Rate limiting configuration
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per minute
    pub requests_per_minute: Option<u32>,
    /// Tokens per minute
    pub tokens_per_minute: Option<u32>,
    /// Concurrent requests limit
    pub concurrent_requests: Option<u32>,
}

/// Provider request/response mappings
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMappings {
    /// Request transformation mappings
    pub request: Value,
    /// Response transformation mappings
    pub response: Value,
    /// Error handling mappings
    pub errors: Option<Value>,
}

/// Translation options
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateOptions {
    /// Translation mode ("standard" or "strict")
    pub mode: Option<String>,
    /// Whether to include metadata
    pub include_metadata: Option<bool>,
    /// Custom transformation rules
    pub custom_rules: Option<Value>,
}

/// Translation result
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateResult {
    /// Translated provider request
    pub request: Value,
    /// Translation metadata
    pub metadata: TranslationMetadata,
    /// Any warnings during translation
    pub warnings: Vec<String>,
}

/// Translation metadata
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationMetadata {
    /// Source format version
    pub source_version: String,
    /// Target provider name
    pub target_provider: String,
    /// Target model ID
    pub target_model: String,
    /// Translation timestamp
    pub timestamp: String,
    /// Features used
    pub features_used: Vec<String>,
    /// Features not supported
    pub unsupported_features: Vec<String>,
}

/// Validation result
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the validation passed
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<ValidationError>,
    /// Validation warnings
    pub warnings: Vec<ValidationWarning>,
    /// Schema version used
    pub schema_version: String,
}

/// Validation error
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// JSON path where error occurred
    pub path: String,
    /// Error message
    pub message: String,
    /// Error code
    pub code: String,
}

/// Validation warning
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// JSON path where warning occurred
    pub path: String,
    /// Warning message
    pub message: String,
    /// Warning code
    pub code: String,
}

/// Provider request for execution
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRequest {
    /// Provider specification
    pub provider: ProviderSpec,
    /// The translated request payload
    pub request: Value,
    /// Authentication credentials
    pub credentials: Option<HashMap<String, String>>,
    /// Additional headers
    pub headers: Option<HashMap<String, String>>,
}

/// Run options for request execution
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunOptions {
    /// Request timeout in seconds
    pub timeout_seconds: Option<u32>,
    /// Maximum retries on failure
    pub max_retries: Option<u32>,
    /// Whether to follow redirects
    pub follow_redirects: Option<bool>,
    /// Custom user agent
    pub user_agent: Option<String>,
}

/// Uniform response format
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniformResponse {
    /// Response content
    pub content: String,
    /// Usage statistics
    pub usage: Option<UsageStats>,
    /// Response metadata
    pub metadata: ResponseMetadata,
    /// Any tool calls in the response
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Finish reason
    pub finish_reason: Option<String>,
}

/// Token usage statistics
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    /// Input tokens used
    pub input_tokens: u32,
    /// Output tokens generated
    pub output_tokens: u32,
    /// Total tokens
    pub total_tokens: u32,
    /// Estimated cost
    pub estimated_cost: Option<f64>,
}

/// Response metadata
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// Provider that handled the request
    pub provider: String,
    /// Model that generated the response
    pub model: String,
    /// Response timestamp
    pub timestamp: String,
    /// Request ID for tracking
    pub request_id: Option<String>,
    /// Response time in milliseconds
    pub response_time_ms: u64,
}