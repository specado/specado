//! Core types and data structures for the Specado translation engine
//!
//! This module defines the fundamental data structures used throughout
//! the library for representing prompts, providers, and translation results.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// Re-export error types for convenience
pub use crate::error::{LossinessCode, Severity, StrictMode};

/// Represents a uniform prompt specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSpec {
    /// Model class (e.g., "Chat", "ReasoningChat")
    pub model_class: String,
    
    /// Conversation messages
    pub messages: Vec<Message>,
    
    /// Available tools for the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    
    /// Tool selection strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    
    /// Expected response format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    
    /// Sampling parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<SamplingParams>,
    
    /// Token and output limits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<Limits>,
    
    /// Media inputs/outputs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<MediaConfig>,
    
    /// Strictness mode for translation
    pub strict_mode: StrictMode,
}

/// Represents a provider specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSpec {
    /// Specification version (semver)
    pub spec_version: String,
    
    /// Provider information
    pub provider: ProviderInfo,
    
    /// Supported models
    pub models: Vec<ModelSpec>,
}

/// Provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Provider name (e.g., "openai", "anthropic")
    pub name: String,
    
    /// Base URL for API requests
    pub base_url: String,
    
    /// Default headers for requests
    pub headers: HashMap<String, String>,
}

/// Model specification within a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpec {
    /// Model identifier
    pub id: String,
    
    /// Alternative names/aliases
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliases: Option<Vec<String>>,
    
    /// Model family
    pub family: String,
    
    /// API endpoints
    pub endpoints: Endpoints,
    
    /// Input mode capabilities
    pub input_modes: InputModes,
    
    /// Tool support configuration
    pub tooling: ToolingConfig,
    
    /// JSON output configuration
    pub json_output: JsonOutputConfig,
    
    /// Parameter mappings and constraints
    pub parameters: Value, // Flexible for provider-specific params
    
    /// Constraints and limits
    pub constraints: Constraints,
    
    /// Field mappings
    pub mappings: Mappings,
    
    /// Response normalization rules
    pub response_normalization: ResponseNormalization,
}

/// Message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message sender
    pub role: MessageRole,
    
    /// Content of the message
    pub content: String,
    
    /// Optional name for the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// Message role enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name
    pub name: String,
    
    /// Tool description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// JSON Schema for tool parameters
    pub json_schema: Value,
}

/// Tool choice strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// Automatic tool selection
    Auto,
    /// Required tool use
    Required,
    /// Specific tool selection
    Specific { name: String },
}

/// Response format specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseFormat {
    /// Plain text response
    Text,
    /// JSON object response
    JsonObject,
    /// Structured JSON with schema
    JsonSchema {
        json_schema: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },
}

/// Sampling parameters for generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
}

/// Token and output limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limits {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_prompt_tokens: Option<u32>,
}

/// Media configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_images: Option<Vec<Value>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_audio: Option<Value>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_audio: Option<Value>,
}

/// API endpoints configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoints {
    pub chat_completion: EndpointConfig,
    pub streaming_chat_completion: EndpointConfig,
}

/// Single endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub method: String,
    pub path: String,
    pub protocol: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<HashMap<String, String>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

/// Input mode capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputModes {
    pub messages: bool,
    pub single_text: bool,
    pub images: bool,
}

/// Tool support configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolingConfig {
    pub tools_supported: bool,
    pub parallel_tool_calls_default: bool,
    pub can_disable_parallel_tool_calls: bool,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_switch: Option<Value>,
}

/// JSON output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonOutputConfig {
    pub native_param: bool,
    pub strategy: String,
}

/// Provider constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    pub system_prompt_location: String,
    pub forbid_unknown_top_level_fields: bool,
    pub mutually_exclusive: Vec<Vec<String>>,
    pub resolution_preferences: Vec<String>,
    pub limits: ConstraintLimits,
}

/// Constraint limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintLimits {
    pub max_tool_schema_bytes: u32,
    pub max_system_prompt_bytes: u32,
}

/// Field mappings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mappings {
    pub paths: HashMap<String, String>,
    pub flags: HashMap<String, Value>,
}

/// Response normalization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseNormalization {
    pub sync: SyncNormalization,
    pub stream: StreamNormalization,
}

/// Synchronous response normalization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncNormalization {
    pub content_path: String,
    pub finish_reason_path: String,
    pub finish_reason_map: HashMap<String, String>,
}

/// Stream response normalization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamNormalization {
    pub protocol: String,
    pub event_selector: EventSelector,
}

/// Event selection rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSelector {
    pub type_path: String,
    pub routes: Vec<EventRoute>,
}

/// Single event route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRoute {
    pub when: String,
    pub emit: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_path: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_path: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args_path: Option<String>,
}

/// Result of a translation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    /// The provider-specific request JSON
    pub provider_request_json: Value,
    
    /// Lossiness report detailing any deviations
    pub lossiness: LossinessReport,
    
    /// Metadata about the translation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<TranslationMetadata>,
}

impl TranslationResult {
    /// Check if the translation has any lossiness
    pub fn has_lossiness(&self) -> bool {
        !self.lossiness.items.is_empty()
    }
}

/// Lossiness report containing all deviations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LossinessReport {
    /// List of lossiness items
    pub items: Vec<LossinessItem>,
    
    /// Overall severity of the report
    pub max_severity: Severity,
    
    /// Summary statistics
    pub summary: LossinessSummary,
}

/// Individual lossiness item
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LossinessItem {
    pub code: LossinessCode,
    pub path: String,
    pub message: String,
    pub severity: Severity,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<Value>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<Value>,
}

/// Summary of lossiness statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LossinessSummary {
    pub total_items: usize,
    pub by_severity: HashMap<String, usize>,
    pub by_code: HashMap<String, usize>,
}

/// Metadata about a translation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranslationMetadata {
    pub provider: String,
    pub model: String,
    pub timestamp: String,
    pub duration_ms: Option<u64>,
    pub strict_mode: StrictMode,
}

/// Normalized response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniformResponse {
    pub model: String,
    pub content: String,
    pub finish_reason: FinishReason,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    
    pub raw_metadata: Value,
}

/// Finish reason for a response
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ToolCall,
    EndConversation,
    Other,
}

/// Tool call in a response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Value,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Stream handle for streaming operations
pub struct StreamHandle {
    // Implementation details will be added when implementing streaming
    pub(crate) _private: (),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_role_serialization() {
        let role = MessageRole::System;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"system\"");
        
        let deserialized: MessageRole = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, MessageRole::System);
    }

    #[test]
    fn test_prompt_spec_minimal() {
        let spec = PromptSpec {
            model_class: "Chat".to_string(),
            messages: vec![
                Message {
                    role: MessageRole::User,
                    content: "Hello".to_string(),
                    name: None,
                    metadata: None,
                }
            ],
            tools: None,
            tool_choice: None,
            response_format: None,
            sampling: None,
            limits: None,
            media: None,
            strict_mode: StrictMode::Warn,
        };
        
        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("\"model_class\":\"Chat\""));
        assert!(json.contains("\"strict_mode\":\"Warn\""));
    }

    #[test]
    fn test_finish_reason_serialization() {
        let reason = FinishReason::Stop;
        let json = serde_json::to_string(&reason).unwrap();
        assert_eq!(json, "\"stop\"");
    }
}