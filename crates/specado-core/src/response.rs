//! Unified response interface for all providers
//! 
//! This module extends the UniformResponse type with convenience methods
//! to provide a consistent interface regardless of the underlying provider.

use crate::types::{UniformResponse, FinishReason};
use serde_json::Value;

/// Extension trait for UniformResponse to provide unified access methods
pub trait ResponseExt {
    /// Get the main text content of the response
    fn text(&self) -> &str;
    
    /// Get token usage information
    fn usage(&self) -> TokenUsage;
    
    /// Check if the response was truncated due to length
    fn is_truncated(&self) -> bool;
    
    /// Get the raw response for advanced use cases
    fn raw(&self) -> &Value;
    
    /// Get any tool calls in the response
    fn tool_calls(&self) -> Vec<ToolCallInfo>;
}

impl ResponseExt for UniformResponse {
    fn text(&self) -> &str {
        &self.content
    }
    
    fn usage(&self) -> TokenUsage {
        // Extract usage from raw_metadata
        let usage_data = self.raw_metadata.get("usage");
        
        TokenUsage {
            input_tokens: usage_data
                .and_then(|u| u.get("input_tokens"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            output_tokens: usage_data
                .and_then(|u| u.get("output_tokens"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            total_tokens: usage_data
                .and_then(|u| u.get("total_tokens"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            reasoning_tokens: usage_data
                .and_then(|u| u.get("output_tokens_details"))
                .and_then(|d| d.get("reasoning_tokens"))
                .and_then(|v| v.as_u64()),
            cache_tokens: usage_data
                .and_then(|u| u.get("cache_creation_input_tokens"))
                .and_then(|v| v.as_u64()),
        }
    }
    
    fn is_truncated(&self) -> bool {
        matches!(self.finish_reason, FinishReason::Length)
    }
    
    fn raw(&self) -> &Value {
        &self.raw_metadata
    }
    
    fn tool_calls(&self) -> Vec<ToolCallInfo> {
        self.tool_calls
            .as_ref()
            .map(|calls| {
                calls.iter().map(|call| ToolCallInfo {
                    name: call.name.clone(),
                    arguments: call.arguments.clone(),
                }).collect()
            })
            .unwrap_or_default()
    }
}

/// Token usage information
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub reasoning_tokens: Option<u64>,  // GPT-5 specific
    pub cache_tokens: Option<u64>,      // Claude specific
}

/// Simplified tool call information
#[derive(Debug, Clone)]
pub struct ToolCallInfo {
    pub name: String,
    pub arguments: Value,
}

/// Helper to extract content from different provider response formats
pub fn extract_content(response: &Value, provider_type: &str) -> Option<String> {
    match provider_type {
        "gpt-5" | "gpt-5-mini" => {
            // GPT-5 Responses API format
            response
                .get("output")
                .and_then(|o| o.as_array())
                .and_then(|arr| arr.get(1))
                .and_then(|msg| msg.get("content"))
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.get(0))
                .and_then(|item| item.get("text"))
                .and_then(|t| t.as_str())
                .map(|s| s.to_string())
        }
        "claude" | "anthropic" => {
            // Claude Messages API format
            response
                .get("content")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.get(0))
                .and_then(|item| item.get("text"))
                .and_then(|t| t.as_str())
                .map(|s| s.to_string())
        }
        "openai" => {
            // OpenAI Chat Completions format
            response
                .get("choices")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.get(0))
                .and_then(|choice| choice.get("message"))
                .and_then(|msg| msg.get("content"))
                .and_then(|c| c.as_str())
                .map(|s| s.to_string())
        }
        _ => None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_extract_gpt5_content() {
        let response = json!({
            "output": [
                {"type": "reasoning"},
                {
                    "content": [
                        {"text": "Hello, world!"}
                    ]
                }
            ]
        });
        
        let content = extract_content(&response, "gpt-5");
        assert_eq!(content, Some("Hello, world!".to_string()));
    }
    
    #[test]
    fn test_extract_claude_content() {
        let response = json!({
            "content": [
                {"text": "Hello from Claude!"}
            ]
        });
        
        let content = extract_content(&response, "claude");
        assert_eq!(content, Some("Hello from Claude!".to_string()));
    }
    
    #[test]
    fn test_token_usage() {
        let response = UniformResponse {
            model: "test".to_string(),
            content: "test".to_string(),
            finish_reason: FinishReason::Stop,
            tool_calls: None,
            raw_metadata: json!({
                "usage": {
                    "input_tokens": 10,
                    "output_tokens": 20,
                    "total_tokens": 30
                }
            }),
        };
        
        let usage = response.usage();
        assert_eq!(usage.input_tokens, 10);
        assert_eq!(usage.output_tokens, 20);
        assert_eq!(usage.total_tokens, 30);
    }
}