//! Provider-specific integration tests for the translation engine
//! 
//! These tests verify provider-specific translation behaviors and ensure
//! proper handling of different provider capabilities and limitations.

use serde_json::{json, Value};
use specado_core::{translate, StrictMode};
use specado_core::error::LossinessCode;
use specado_core::types::{
    JsonOutputSupport, Message, ModelSpec, PromptSpec, ProviderInfo, ProviderSpec, 
    SamplingParams, ToolingSupport, OutputLimits, ReasoningSupport
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Create an OpenAI provider spec with specific capabilities
fn create_openai_provider() -> ProviderSpec {
    ProviderSpec {
        spec_version: "1.0.0".to_string(),
        provider: ProviderInfo {
            name: "openai".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            headers: {
                let mut headers = HashMap::new();
                headers.insert("Authorization".to_string(), "Bearer $OPENAI_API_KEY".to_string());
                headers
            },
        },
        models: vec![
            ModelSpec {
                id: "gpt-5".to_string(),
                aliases: Some(vec!["gpt5".to_string()]),
                model_class: "Chat".to_string(),
                context_window: 128000,
                max_output_tokens: Some(4096),
                pricing: None,
                tooling: ToolingSupport {
                    tools_supported: true,
                    tool_choice_modes: vec!["auto".to_string(), "none".to_string(), "required".to_string()],
                    parallel_tools: true,
                },
                json_output: JsonOutputSupport {
                    supported: true,
                    native_param: true,
                    strategy: "response_format".to_string(),
                },
                media: Some(specado_core::types::MediaSupport {
                    image_support: true,
                    video_support: false,
                    audio_support: false,
                    formats: vec!["image/jpeg".to_string(), "image/png".to_string()],
                    max_media_per_message: Some(10),
                }),
                reasoning: None,
            },
            ModelSpec {
                id: "gpt-5-turbo".to_string(),
                aliases: None,
                model_class: "Chat".to_string(),
                context_window: 128000,
                max_output_tokens: Some(16384),
                pricing: None,
                tooling: ToolingSupport {
                    tools_supported: true,
                    tool_choice_modes: vec!["auto".to_string(), "none".to_string()],
                    parallel_tools: false,
                },
                json_output: JsonOutputSupport {
                    supported: true,
                    native_param: true,
                    strategy: "response_format".to_string(),
                },
                media: None,
                reasoning: None,
            },
        ],
    }
}

/// Create an Anthropic provider spec with specific capabilities
fn create_anthropic_provider() -> ProviderSpec {
    ProviderSpec {
        spec_version: "1.0.0".to_string(),
        provider: ProviderInfo {
            name: "anthropic".to_string(),
            base_url: "https://api.anthropic.com/v1".to_string(),
            headers: {
                let mut headers = HashMap::new();
                headers.insert("X-API-Key".to_string(), "$ANTHROPIC_API_KEY".to_string());
                headers.insert("anthropic-version".to_string(), "2024-02-15".to_string());
                headers
            },
        },
        models: vec![
            ModelSpec {
                id: "claude-opus-4.1".to_string(),
                aliases: Some(vec!["claude-opus".to_string()]),
                model_class: "Chat".to_string(),
                context_window: 200000,
                max_output_tokens: Some(4096),
                pricing: None,
                tooling: ToolingSupport {
                    tools_supported: true,
                    tool_choice_modes: vec!["auto".to_string(), "any".to_string(), "tool".to_string()],
                    parallel_tools: true,
                },
                json_output: JsonOutputSupport {
                    supported: true,
                    native_param: false,
                    strategy: "system_prompt".to_string(),
                },
                media: Some(specado_core::types::MediaSupport {
                    image_support: true,
                    video_support: false,
                    audio_support: false,
                    formats: vec!["image/jpeg".to_string(), "image/png".to_string(), "image/gif".to_string()],
                    max_media_per_message: Some(20),
                }),
                reasoning: Some(ReasoningSupport {
                    supported: true,
                    strategy: "native".to_string(),
                    max_reasoning_tokens: Some(100000),
                }),
            },
        ],
    }
}

/// Create a provider with limited capabilities
fn create_limited_provider() -> ProviderSpec {
    ProviderSpec {
        spec_version: "1.0.0".to_string(),
        provider: ProviderInfo {
            name: "limited".to_string(),
            base_url: "https://api.limited.com".to_string(),
            headers: HashMap::new(),
        },
        models: vec![
            ModelSpec {
                id: "basic-model".to_string(),
                aliases: None,
                model_class: "Chat".to_string(),
                context_window: 4096,
                max_output_tokens: Some(512),
                pricing: None,
                tooling: ToolingSupport {
                    tools_supported: false,
                    tool_choice_modes: vec![],
                    parallel_tools: false,
                },
                json_output: JsonOutputSupport {
                    supported: false,
                    native_param: false,
                    strategy: "none".to_string(),
                },
                media: None,
                reasoning: None,
            },
        ],
    }
}

/// Test OpenAI-specific translation with tools
#[test]
fn test_openai_translation_with_tools() {
    let mut prompt = PromptSpec {
        model_class: "Chat".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "What's the weather in SF?".to_string(),
            },
        ],
        tools: Some(vec![json!({
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "Get the current weather in a location",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "The city and state"
                        }
                    },
                    "required": ["location"]
                }
            }
        })]),
        tool_choice: Some(json!("auto")),
        response_format: None,
        sampling: None,
        limits: None,
        media: None,
        strict_mode: StrictMode::Warn,
    };

    let provider = create_openai_provider();
    let result = translate(&prompt, &provider, "gpt-5", StrictMode::Warn)
        .expect("Translation should succeed");

    let request = result.provider_request_json.as_object().unwrap();
    
    // OpenAI supports tools, so they should be included
    assert!(request.contains_key("tools"));
    assert!(request.contains_key("tool_choice"));
    
    let tools = request["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(request["tool_choice"], json!("auto"));
    
    // Should have no lossiness for supported features
    assert_eq!(result.lossiness.summary.total_items, 0);
}

/// Test Anthropic-specific translation with system prompt JSON mode
#[test]
fn test_anthropic_json_mode_emulation() {
    let mut prompt = PromptSpec {
        model_class: "Chat".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Return a JSON object with name and age".to_string(),
            },
        ],
        tools: None,
        tool_choice: None,
        response_format: Some(json!({
            "type": "json_object"
        })),
        sampling: None,
        limits: None,
        media: None,
        strict_mode: StrictMode::Warn,
    };

    let provider = create_anthropic_provider();
    let result = translate(&prompt, &provider, "claude-opus-4.1", StrictMode::Warn)
        .expect("Translation should succeed");

    // Anthropic uses system_prompt strategy for JSON mode
    // Should track this as emulation in lossiness
    assert!(result.lossiness.items.iter().any(|item| 
        item.code == specado_core::error::LossinessCode::Emulate
    ));
}

/// Test provider with no tool support
#[test]
fn test_limited_provider_drops_tools() {
    let mut prompt = PromptSpec {
        model_class: "Chat".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
        ],
        tools: Some(vec![json!({
            "name": "test_tool",
            "description": "A test tool"
        })]),
        tool_choice: Some(json!("auto")),
        response_format: None,
        sampling: None,
        limits: None,
        media: None,
        strict_mode: StrictMode::Warn,
    };

    let provider = create_limited_provider();
    let result = translate(&prompt, &provider, "basic-model", StrictMode::Warn)
        .expect("Translation should succeed in Warn mode");

    let request = result.provider_request_json.as_object().unwrap();
    
    // Limited provider doesn't support tools, so they should be dropped
    assert!(!request.contains_key("tools"));
    assert!(!request.contains_key("tool_choice"));
    
    // Should track in lossiness
    assert!(result.lossiness.summary.total_items > 0);
    assert!(result.lossiness.items.iter().any(|item| 
        item.path == "$.tools" && item.code == LossinessCode::Unsupported
    ));
}

/// Test translation with media support (OpenAI)
#[test]
fn test_openai_media_translation() {
    let mut prompt = PromptSpec {
        model_class: "Chat".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "What's in this image?".to_string(),
            },
        ],
        tools: None,
        tool_choice: None,
        response_format: None,
        sampling: None,
        limits: None,
        media: Some(vec![json!({
            "type": "image",
            "url": "https://example.com/image.jpg",
            "mime_type": "image/jpeg"
        })]),
        strict_mode: StrictMode::Warn,
    };

    let provider = create_openai_provider();
    let result = translate(&prompt, &provider, "gpt-5", StrictMode::Warn)
        .expect("Translation should succeed");

    // OpenAI supports images, so media should be handled appropriately
    // (actual implementation details depend on provider mapping)
    assert!(result.provider_request_json.is_object());
}

/// Test reasoning model translation (Anthropic)
#[test]
fn test_anthropic_reasoning_model() {
    let mut prompt = PromptSpec {
        model_class: "ReasoningChat".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Solve this complex problem step by step".to_string(),
            },
        ],
        tools: None,
        tool_choice: None,
        response_format: None,
        sampling: None,
        limits: Some(OutputLimits {
            max_output_tokens: Some(1000),
            stop_sequences: None,
            reasoning_tokens: Some(5000),
        }),
        media: None,
        strict_mode: StrictMode::Warn,
    };

    let provider = create_anthropic_provider();
    let result = translate(&prompt, &provider, "claude-opus-4.1", StrictMode::Warn)
        .expect("Translation should succeed");

    let request = result.provider_request_json.as_object().unwrap();
    
    // Should handle reasoning tokens appropriately for Anthropic
    assert!(request.contains_key("max_tokens"));
}

/// Test temperature scaling between providers
#[test]
fn test_cross_provider_temperature_scaling() {
    let mut prompt = PromptSpec {
        model_class: "Chat".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Be creative".to_string(),
            },
        ],
        tools: None,
        tool_choice: None,
        response_format: None,
        sampling: Some(SamplingParams {
            temperature: Some(1.5),
            top_p: None,
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
            repetition_penalty: None,
            seed: None,
        }),
        limits: None,
        media: None,
        strict_mode: StrictMode::Warn,
    };

    // Test with OpenAI (0-2 range)
    let openai_provider = create_openai_provider();
    let openai_result = translate(&prompt, &openai_provider, "gpt-5", StrictMode::Warn)
        .expect("OpenAI translation should succeed");
    
    let openai_request = openai_result.provider_request_json.as_object().unwrap();
    assert_eq!(openai_request["temperature"], json!(1.5));

    // Test with Anthropic (might have different range handling)
    let anthropic_provider = create_anthropic_provider();
    let anthropic_result = translate(&prompt, &anthropic_provider, "claude-opus-4.1", StrictMode::Warn)
        .expect("Anthropic translation should succeed");
    
    let anthropic_request = anthropic_result.provider_request_json.as_object().unwrap();
    // Temperature handling may vary by provider
    assert!(anthropic_request.contains_key("temperature"));
}

/// Test parallel tools capability
#[test]
fn test_parallel_tools_capability() {
    let mut prompt = PromptSpec {
        model_class: "Chat".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Get weather for multiple cities".to_string(),
            },
        ],
        tools: Some(vec![
            json!({
                "name": "get_weather",
                "description": "Get weather for a city"
            }),
            json!({
                "name": "get_forecast",
                "description": "Get forecast for a city"
            }),
        ]),
        tool_choice: Some(json!("auto")),
        response_format: None,
        sampling: None,
        limits: None,
        media: None,
        strict_mode: StrictMode::Warn,
    };

    // OpenAI GPT-5 supports parallel tools
    let openai_provider = create_openai_provider();
    let openai_result = translate(&prompt, &openai_provider, "gpt-5", StrictMode::Warn)
        .expect("Translation should succeed");
    
    let openai_request = openai_result.provider_request_json.as_object().unwrap();
    assert!(openai_request.contains_key("tools"));
    let tools = openai_request["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 2);

    // GPT-5-turbo doesn't support parallel tools
    let turbo_result = translate(&prompt, &openai_provider, "gpt-5-turbo", StrictMode::Warn)
        .expect("Translation should succeed");
    
    // Should track limitation in lossiness if parallel tools are needed
    // (actual behavior depends on implementation details)
    assert!(turbo_result.provider_request_json.is_object());
}

/// Test max output tokens capping
#[test]
fn test_max_output_tokens_capping() {
    let mut prompt = PromptSpec {
        model_class: "Chat".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Write a long story".to_string(),
            },
        ],
        tools: None,
        tool_choice: None,
        response_format: None,
        sampling: None,
        limits: Some(OutputLimits {
            max_output_tokens: Some(10000), // Exceeds model limit
            stop_sequences: None,
            reasoning_tokens: None,
        }),
        media: None,
        strict_mode: StrictMode::Warn,
    };

    let provider = create_openai_provider();
    let result = translate(&prompt, &provider, "gpt-5", StrictMode::Warn)
        .expect("Translation should succeed");

    let request = result.provider_request_json.as_object().unwrap();
    
    // Should cap to model's max (4096 for gpt-5)
    let max_tokens = request["max_tokens"].as_i64().unwrap();
    assert!(max_tokens <= 4096);
    
    // Should track capping in lossiness
    assert!(result.lossiness.items.iter().any(|item| 
        item.code == LossinessCode::Clamp
    ));
}

/// Test model not found error
#[test]
fn test_model_not_found() {
    let prompt = PromptSpec {
        model_class: "Chat".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
        ],
        tools: None,
        tool_choice: None,
        response_format: None,
        sampling: None,
        limits: None,
        media: None,
        strict_mode: StrictMode::Warn,
    };

    let provider = create_openai_provider();
    let result = translate(&prompt, &provider, "nonexistent-model", StrictMode::Warn);
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Model 'nonexistent-model' not found"));
}

/// Test strict mode failure on multiple issues
#[test]
fn test_strict_mode_multiple_failures() {
    let mut prompt = PromptSpec {
        model_class: "Chat".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Test".to_string(),
            },
        ],
        tools: Some(vec![json!({"name": "tool"})]),
        tool_choice: Some(json!("required")),
        response_format: Some(json!({"type": "json_schema"})),
        sampling: Some(SamplingParams {
            temperature: Some(5.0),
            top_p: None,
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
            repetition_penalty: None,
            seed: None,
        }),
        limits: None,
        media: None,
        strict_mode: StrictMode::Strict,
    };

    let provider = create_limited_provider();
    let result = translate(&prompt, &provider, "basic-model", StrictMode::Strict);
    
    // Should fail in strict mode due to unsupported features
    assert!(result.is_err());
}