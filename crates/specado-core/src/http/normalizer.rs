//! Response normalization for provider API responses
//!
//! This module normalizes provider-specific response formats into the
//! standard UniformResponse structure using JSONPath rules from ProviderSpec.

use serde_json::Value;
use crate::types::{
    UniformResponse, FinishReason, ToolCall,
    ModelSpec, SyncNormalization,
};
use crate::translation::jsonpath::JSONPath;
use crate::Result;

/// Response normalizer that converts provider responses to UniformResponse
#[derive(Debug)]
pub struct ResponseNormalizer {
    /// Model specification containing normalization rules
    model_spec: ModelSpec,
}

impl ResponseNormalizer {
    /// Create a new normalizer for a specific model
    pub fn new(model_spec: ModelSpec) -> Self {
        Self { model_spec }
    }
    
    /// Normalize a provider response to UniformResponse format
    pub fn normalize_response(
        &self,
        provider_response: &Value,
        model_id: &str,
    ) -> Result<UniformResponse> {
        let norm_config = &self.model_spec.response_normalization.sync;
        
        // Extract content using JSONPath
        let content = self.extract_content(provider_response, norm_config)?;
        
        // Extract and map finish reason
        let finish_reason = self.extract_finish_reason(provider_response, norm_config)?;
        
        // Extract tool calls if present
        let tool_calls = self.extract_tool_calls(provider_response, norm_config)?;
        
        Ok(UniformResponse {
            model: model_id.to_string(),
            content,
            finish_reason,
            tool_calls,
            raw_metadata: provider_response.clone(),
        })
    }
    
    /// Extract content from the response
    fn extract_content(
        &self,
        response: &Value,
        config: &SyncNormalization,
    ) -> Result<String> {
        // Special handling for Anthropic content blocks
        if config.content_path.contains("$.content") && config.content_path.contains("text") {
            // Try to extract text from Anthropic-style content array
            if let Some(content_array) = response.get("content").and_then(|c| c.as_array()) {
                // Collect all text blocks
                let text_parts: Vec<String> = content_array.iter()
                    .filter_map(|block| {
                        if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                            block.get("text").and_then(|t| t.as_str()).map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
                
                if !text_parts.is_empty() {
                    return Ok(text_parts.join(""));
                }
            }
        }
        
        // Standard JSONPath extraction
        let path = JSONPath::parse(&config.content_path)?;
        let results = path.execute(response)?;
        
        // Get the first result and convert to string
        if let Some(value) = results.first() {
            match value {
                Value::String(s) => Ok(s.clone()),
                Value::Null => Ok(String::new()), // Handle null content explicitly
                Value::Array(arr) => {
                    // Handle array of content blocks
                    let text_parts: Vec<String> = arr.iter()
                        .filter_map(|v| {
                            if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                                Some(text.to_string())
                            } else {
                                v.as_str().map(|s| s.to_string())
                            }
                        })
                        .collect();
                    Ok(text_parts.join(""))
                }
                _ => Ok(value.to_string()),
            }
        } else {
            // If no content found, return empty string
            Ok(String::new())
        }
    }
    
    /// Extract and map finish reason
    fn extract_finish_reason(
        &self,
        response: &Value,
        config: &SyncNormalization,
    ) -> Result<FinishReason> {
        let path = JSONPath::parse(&config.finish_reason_path)?;
        let results = path.execute(response)?;
        
        if let Some(value) = results.first() {
            if let Some(reason_str) = value.as_str() {
                // Map using the provider's mapping table
                if let Some(mapped) = config.finish_reason_map.get(reason_str) {
                    return self.parse_finish_reason(mapped);
                }
                // Try to parse directly if not in mapping
                return self.parse_finish_reason(reason_str);
            }
        }
        
        // Default to "stop" if no finish reason found
        Ok(FinishReason::Stop)
    }
    
    /// Parse finish reason string to enum
    fn parse_finish_reason(&self, reason: &str) -> Result<FinishReason> {
        Ok(match reason.to_lowercase().as_str() {
            "stop" | "end_turn" => FinishReason::Stop,
            "length" | "max_tokens" => FinishReason::Length,
            "tool_call" | "tool_calls" | "tool_use" => FinishReason::ToolCall,
            "end_conversation" | "end" => FinishReason::EndConversation,
            _ => FinishReason::Other,
        })
    }
    
    /// Extract tool calls from the response
    fn extract_tool_calls(
        &self,
        response: &Value,
        _config: &SyncNormalization,
    ) -> Result<Option<Vec<ToolCall>>> {
        // Try common patterns for tool calls
        let tool_calls = self.try_extract_openai_tools(response)
            .or_else(|| self.try_extract_anthropic_tools(response));
        
        Ok(tool_calls)
    }
    
    /// Try to extract OpenAI-style tool calls
    fn try_extract_openai_tools(&self, response: &Value) -> Option<Vec<ToolCall>> {
        // OpenAI format: choices[0].message.tool_calls
        response.get("choices")
            .and_then(|c| c.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|msg| msg.get("tool_calls"))
            .and_then(|tools| tools.as_array())
            .map(|tools| {
                tools.iter()
                    .filter_map(|tool| {
                        let name = tool.get("function")
                            .and_then(|f| f.get("name"))
                            .and_then(|n| n.as_str())?;
                        
                        let arguments = tool.get("function")
                            .and_then(|f| f.get("arguments"))
                            .cloned()
                            .unwrap_or(Value::Null);
                        
                        let id = tool.get("id")
                            .and_then(|i| i.as_str())
                            .map(|s| s.to_string());
                        
                        Some(ToolCall {
                            name: name.to_string(),
                            arguments,
                            id,
                        })
                    })
                    .collect()
            })
            .filter(|v: &Vec<ToolCall>| !v.is_empty())
    }
    
    /// Try to extract Anthropic-style tool calls
    fn try_extract_anthropic_tools(&self, response: &Value) -> Option<Vec<ToolCall>> {
        // Anthropic format: content array with tool_use blocks
        response.get("content")
            .and_then(|content| content.as_array())
            .map(|blocks| {
                blocks.iter()
                    .filter_map(|block| {
                        if block.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                            let name = block.get("name")
                                .and_then(|n| n.as_str())?;
                            
                            let arguments = block.get("input")
                                .cloned()
                                .unwrap_or(Value::Null);
                            
                            let id = block.get("id")
                                .and_then(|i| i.as_str())
                                .map(|s| s.to_string());
                            
                            Some(ToolCall {
                                name: name.to_string(),
                                arguments,
                                id,
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .filter(|v: &Vec<ToolCall>| !v.is_empty())
    }
}

/// Normalize a response using model-specific rules
pub fn normalize_response(
    provider_response: &Value,
    model_spec: &ModelSpec,
    model_id: &str,
) -> Result<UniformResponse> {
    let normalizer = ResponseNormalizer::new(model_spec.clone());
    normalizer.normalize_response(provider_response, model_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;
    
    fn create_test_model_spec() -> ModelSpec {
        use crate::types::*;
        
        ModelSpec {
            id: "test-model".to_string(),
            aliases: Some(vec!["test".to_string()]),
            family: "test".to_string(),
            endpoints: Endpoints {
                chat_completion: EndpointConfig {
                    method: "POST".to_string(),
                    path: "/v1/chat".to_string(),
                    protocol: "http".to_string(),
                    query: None,
                    headers: None,
                },
                streaming_chat_completion: EndpointConfig {
                    method: "POST".to_string(),
                    path: "/v1/chat".to_string(),
                    protocol: "sse".to_string(),
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
                system_prompt_location: "message_role".to_string(),
                forbid_unknown_top_level_fields: false,
                mutually_exclusive: vec![],
                resolution_preferences: vec![],
                limits: ConstraintLimits {
                    max_tool_schema_bytes: 100000,
                    max_system_prompt_bytes: 10000,
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
                    finish_reason_map: HashMap::from([
                        ("stop".to_string(), "stop".to_string()),
                        ("length".to_string(), "length".to_string()),
                        ("tool_calls".to_string(), "tool_call".to_string()),
                    ]),
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
    }
    
    #[test]
    fn test_normalize_openai_response() {
        let model_spec = create_test_model_spec();
        let normalizer = ResponseNormalizer::new(model_spec);
        
        let openai_response = json!({
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "gpt-5",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello! How can I help you today?"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 8,
                "total_tokens": 18
            }
        });
        
        let result = normalizer.normalize_response(&openai_response, "gpt-5").unwrap();
        
        assert_eq!(result.model, "gpt-5");
        assert_eq!(result.content, "Hello! How can I help you today?");
        assert_eq!(result.finish_reason, FinishReason::Stop);
        assert!(result.tool_calls.is_none());
    }
    
    #[test]
    fn test_normalize_anthropic_response() {
        // Create Anthropic-specific model spec
        let mut model_spec = create_test_model_spec();
        model_spec.response_normalization.sync.content_path = "$.content[-1].text".to_string();
        model_spec.response_normalization.sync.finish_reason_path = "$.stop_reason".to_string();
        model_spec.response_normalization.sync.finish_reason_map = HashMap::from([
            ("end_turn".to_string(), "stop".to_string()),
            ("max_tokens".to_string(), "length".to_string()),
            ("tool_use".to_string(), "tool_call".to_string()),
        ]);
        
        let normalizer = ResponseNormalizer::new(model_spec);
        
        let anthropic_response = json!({
            "id": "msg_123",
            "type": "message",
            "role": "assistant",
            "content": [{
                "type": "text",
                "text": "Hello! I'm Claude, how can I assist you?"
            }],
            "model": "claude-opus-4-1",
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "usage": {
                "input_tokens": 10,
                "output_tokens": 12
            }
        });
        
        let result = normalizer.normalize_response(&anthropic_response, "claude-opus-4-1").unwrap();
        
        assert_eq!(result.model, "claude-opus-4-1");
        assert_eq!(result.content, "Hello! I'm Claude, how can I assist you?");
        assert_eq!(result.finish_reason, FinishReason::Stop);
        assert!(result.tool_calls.is_none());
    }
    
    #[test]
    fn test_extract_openai_tool_calls() {
        let model_spec = create_test_model_spec();
        let normalizer = ResponseNormalizer::new(model_spec);
        
        let response_with_tools = json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": "call_123",
                        "type": "function",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"location\": \"San Francisco\"}"
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }]
        });
        
        let result = normalizer.normalize_response(&response_with_tools, "gpt-5").unwrap();
        
        assert!(result.tool_calls.is_some());
        let tools = result.tool_calls.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "get_weather");
        assert_eq!(tools[0].id, Some("call_123".to_string()));
    }
    
    #[test]
    fn test_missing_content_returns_empty() {
        let model_spec = create_test_model_spec();
        let normalizer = ResponseNormalizer::new(model_spec);
        
        let empty_response = json!({
            "choices": [{
                "message": {
                    "role": "assistant"
                },
                "finish_reason": "stop"
            }]
        });
        
        let result = normalizer.normalize_response(&empty_response, "test").unwrap();
        assert_eq!(result.content, "");
    }
}