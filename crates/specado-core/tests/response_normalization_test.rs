//! Integration tests for response normalization
//!
//! Tests the complete response normalization pipeline with various provider formats

use specado_core::types::*;
use specado_core::http::normalize_response;
use serde_json::json;
use std::collections::HashMap;

/// Create a test OpenAI model spec with normalization rules
fn create_openai_model_spec() -> ModelSpec {
    ModelSpec {
        id: "gpt-5".to_string(),
        aliases: Some(vec!["gpt-5-chat".to_string()]),
        family: "gpt-5".to_string(),
        endpoints: Endpoints {
            chat_completion: EndpointConfig {
                method: "POST".to_string(),
                path: "/v1/chat/completions".to_string(),
                protocol: "http".to_string(),
                query: None,
                headers: None,
            },
            streaming_chat_completion: EndpointConfig {
                method: "POST".to_string(),
                path: "/v1/chat/completions".to_string(),
                protocol: "sse".to_string(),
                query: Some(HashMap::from([("stream".to_string(), "true".to_string())])),
                headers: None,
            },
        },
        input_modes: InputModes {
            messages: true,
            single_text: false,
            images: true,
        },
        tooling: ToolingConfig {
            tools_supported: true,
            parallel_tool_calls_default: true,
            can_disable_parallel_tool_calls: true,
            disable_switch: Some(json!({"path": "$.parallel_tool_calls", "value": false})),
        },
        json_output: JsonOutputConfig {
            native_param: true,
            strategy: "json_schema".to_string(),
        },
        parameters: json!({}),
        constraints: Constraints {
            system_prompt_location: "message_role".to_string(),
            forbid_unknown_top_level_fields: false,
            mutually_exclusive: vec![],
            resolution_preferences: vec![],
            limits: ConstraintLimits {
                max_tool_schema_bytes: 200000,
                max_system_prompt_bytes: 32000,
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
                    ("function_call".to_string(), "tool_call".to_string()),
                ]),
            },
            stream: StreamNormalization {
                protocol: "sse".to_string(),
                event_selector: EventSelector {
                    type_path: "$.object".to_string(),
                    routes: vec![],
                },
            },
        },
    }
}

/// Create a test Anthropic model spec with normalization rules
fn create_anthropic_model_spec() -> ModelSpec {
    ModelSpec {
        id: "claude-opus-4-1".to_string(),
        aliases: Some(vec!["opus-4.1".to_string()]),
        family: "opus-4.1".to_string(),
        endpoints: Endpoints {
            chat_completion: EndpointConfig {
                method: "POST".to_string(),
                path: "/v1/messages".to_string(),
                protocol: "http".to_string(),
                query: None,
                headers: None,
            },
            streaming_chat_completion: EndpointConfig {
                method: "POST".to_string(),
                path: "/v1/messages".to_string(),
                protocol: "sse".to_string(),
                query: Some(HashMap::from([("stream".to_string(), "true".to_string())])),
                headers: None,
            },
        },
        input_modes: InputModes {
            messages: true,
            single_text: false,
            images: true,
        },
        tooling: ToolingConfig {
            tools_supported: true,
            parallel_tool_calls_default: true,
            can_disable_parallel_tool_calls: true,
            disable_switch: Some(json!({"path": "$.tool_choice.disable_parallel_tool_use", "value": true})),
        },
        json_output: JsonOutputConfig {
            native_param: false,
            strategy: "tools".to_string(),
        },
        parameters: json!({}),
        constraints: Constraints {
            system_prompt_location: "top_level".to_string(),
            forbid_unknown_top_level_fields: true,
            mutually_exclusive: vec![
                vec!["sampling.temperature".to_string(), "sampling.top_p".to_string()],
            ],
            resolution_preferences: vec!["sampling.temperature".to_string()],
            limits: ConstraintLimits {
                max_tool_schema_bytes: 180000,
                max_system_prompt_bytes: 30000,
            },
        },
        mappings: Mappings {
            paths: HashMap::new(),
            flags: HashMap::new(),
        },
        response_normalization: ResponseNormalization {
            sync: SyncNormalization {
                content_path: "$.content[-1].text".to_string(),
                finish_reason_path: "$.stop_reason".to_string(),
                finish_reason_map: HashMap::from([
                    ("end_turn".to_string(), "stop".to_string()),
                    ("max_tokens".to_string(), "length".to_string()),
                    ("tool_use".to_string(), "tool_call".to_string()),
                    ("stop_sequence".to_string(), "stop".to_string()),
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
fn test_openai_simple_response() {
    let model_spec = create_openai_model_spec();
    
    let response = json!({
        "id": "chatcmpl-abc123",
        "object": "chat.completion",
        "created": 1677652288,
        "model": "gpt-5",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "The capital of France is Paris."
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 7,
            "total_tokens": 17
        }
    });
    
    let result = normalize_response(&response, &model_spec, "gpt-5").unwrap();
    
    assert_eq!(result.model, "gpt-5");
    assert_eq!(result.content, "The capital of France is Paris.");
    assert_eq!(result.finish_reason, FinishReason::Stop);
    assert!(result.tool_calls.is_none());
}

#[test]
fn test_anthropic_simple_response() {
    let model_spec = create_anthropic_model_spec();
    
    let response = json!({
        "id": "msg_01XFDUDYJgAACzvnptvVoYEL",
        "type": "message",
        "role": "assistant",
        "content": [{
            "type": "text",
            "text": "The capital of France is Paris."
        }],
        "model": "claude-opus-4-1",
        "stop_reason": "end_turn",
        "stop_sequence": null,
        "usage": {
            "input_tokens": 12,
            "output_tokens": 7
        }
    });
    
    let result = normalize_response(&response, &model_spec, "claude-opus-4-1").unwrap();
    
    assert_eq!(result.model, "claude-opus-4-1");
    assert_eq!(result.content, "The capital of France is Paris.");
    assert_eq!(result.finish_reason, FinishReason::Stop);
    assert!(result.tool_calls.is_none());
}

#[test]
fn test_openai_with_tools() {
    let model_spec = create_openai_model_spec();
    
    let response = json!({
        "id": "chatcmpl-abc123",
        "object": "chat.completion",
        "created": 1677652288,
        "model": "gpt-5",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": "call_abc123",
                    "type": "function",
                    "function": {
                        "name": "get_weather",
                        "arguments": "{\"location\": \"San Francisco\", \"unit\": \"celsius\"}"
                    }
                }]
            },
            "finish_reason": "tool_calls"
        }],
        "usage": {
            "prompt_tokens": 82,
            "completion_tokens": 17,
            "total_tokens": 99
        }
    });
    
    let result = normalize_response(&response, &model_spec, "gpt-5").unwrap();
    
    assert_eq!(result.model, "gpt-5");
    assert_eq!(result.content, ""); // No content when using tools
    assert_eq!(result.finish_reason, FinishReason::ToolCall);
    
    assert!(result.tool_calls.is_some());
    let tools = result.tool_calls.unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "get_weather");
    assert_eq!(tools[0].id, Some("call_abc123".to_string()));
}

#[test]
fn test_anthropic_with_tools() {
    let model_spec = create_anthropic_model_spec();
    
    let response = json!({
        "id": "msg_01XFDUDYJgAACzvnptvVoYEL",
        "type": "message",
        "role": "assistant",
        "content": [
            {
                "type": "text",
                "text": "I'll check the weather for you."
            },
            {
                "type": "tool_use",
                "id": "toolu_01A09q90qw90lq917835lq9",
                "name": "get_weather",
                "input": {
                    "location": "San Francisco",
                    "unit": "celsius"
                }
            }
        ],
        "model": "claude-opus-4-1",
        "stop_reason": "tool_use",
        "stop_sequence": null,
        "usage": {
            "input_tokens": 300,
            "output_tokens": 50
        }
    });
    
    let result = normalize_response(&response, &model_spec, "claude-opus-4-1").unwrap();
    
    assert_eq!(result.model, "claude-opus-4-1");
    assert_eq!(result.content, "I'll check the weather for you.");
    assert_eq!(result.finish_reason, FinishReason::ToolCall);
    
    assert!(result.tool_calls.is_some());
    let tools = result.tool_calls.unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "get_weather");
    assert_eq!(tools[0].id, Some("toolu_01A09q90qw90lq917835lq9".to_string()));
}

#[test]
fn test_length_limit_finish_reason() {
    let model_spec = create_openai_model_spec();
    
    let response = json!({
        "id": "chatcmpl-abc123",
        "object": "chat.completion",
        "created": 1677652288,
        "model": "gpt-5",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "This is a partial response that was cut off due to"
            },
            "finish_reason": "length"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 100,
            "total_tokens": 110
        }
    });
    
    let result = normalize_response(&response, &model_spec, "gpt-5").unwrap();
    
    assert_eq!(result.finish_reason, FinishReason::Length);
    assert_eq!(result.content, "This is a partial response that was cut off due to");
}

#[test]
fn test_missing_fields_graceful_handling() {
    let model_spec = create_openai_model_spec();
    
    // Minimal response with missing optional fields
    let response = json!({
        "choices": [{
            "message": {
                "role": "assistant"
                // No content field
            }
            // No finish_reason field
        }]
    });
    
    let result = normalize_response(&response, &model_spec, "gpt-5").unwrap();
    
    assert_eq!(result.model, "gpt-5");
    assert_eq!(result.content, ""); // Empty content when missing
    assert_eq!(result.finish_reason, FinishReason::Stop); // Default to Stop
    assert!(result.tool_calls.is_none());
}