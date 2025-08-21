//! Shared test support utilities for integration tests

use serde_json::json;
use specado_core::types::{
    ConstraintLimits, Constraints, EndpointConfig, Endpoints, EventSelector,
    InputModes, JsonOutputConfig, Limits, Mappings, Message, MessageRole, ModelSpec, 
    PromptSpec, ProviderInfo, ProviderSpec, ResponseFormat, ResponseNormalization, 
    SamplingParams, StreamNormalization, SyncNormalization, Tool, ToolChoice, ToolingConfig,
};
use specado_core::StrictMode;

/// Create a minimal prompt spec for testing
pub fn minimal_prompt() -> PromptSpec {
    PromptSpec {
        model_class: "Chat".to_string(),
        messages: vec![Message {
            role: MessageRole::User,
            content: "Test message".to_string(),
            name: None,
            metadata: None,
        }],
        tools: None,
        tool_choice: None,
        response_format: None,
        sampling: None,
        limits: None,
        media: None,
        strict_mode: StrictMode::Warn,
    }
}

/// Create a prompt with system and user messages
pub fn chat_prompt(system: &str, user: &str) -> PromptSpec {
    PromptSpec {
        model_class: "Chat".to_string(),
        messages: vec![
            Message {
                role: MessageRole::System,
                content: system.to_string(),
                name: None,
                metadata: None,
            },
            Message {
                role: MessageRole::User,
                content: user.to_string(),
                name: None,
                metadata: None,
            },
        ],
        tools: None,
        tool_choice: None,
        response_format: None,
        sampling: None,
        limits: None,
        media: None,
        strict_mode: StrictMode::Warn,
    }
}

/// Create a prompt with tools
pub fn prompt_with_tools() -> PromptSpec {
    let mut prompt = minimal_prompt();
    prompt.tools = Some(vec![Tool {
        name: "get_weather".to_string(),
        description: Some("Get the current weather".to_string()),
        json_schema: json!({
            "type": "object",
            "properties": {
                "location": {"type": "string"}
            }
        }),
    }]);
    prompt.tool_choice = Some(ToolChoice::Auto);
    prompt
}

/// Create a prompt with sampling parameters
pub fn prompt_with_sampling() -> PromptSpec {
    let mut prompt = minimal_prompt();
    prompt.sampling = Some(SamplingParams {
        temperature: Some(0.7),
        top_p: Some(0.9),
        top_k: Some(40),
        frequency_penalty: Some(0.5),
        presence_penalty: Some(0.2),
    });
    prompt
}

/// Create a prompt with output limits
pub fn prompt_with_limits() -> PromptSpec {
    let mut prompt = minimal_prompt();
    prompt.limits = Some(Limits {
        max_output_tokens: Some(1000),
        reasoning_tokens: None,
        max_prompt_tokens: None,
    });
    prompt
}

/// Create a minimal OpenAI provider spec
pub fn openai_provider() -> ProviderSpec {
    ProviderSpec {
        spec_version: "1.0.0".to_string(),
        provider: ProviderInfo {
            name: "openai".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            headers: {
                let mut headers = std::collections::HashMap::new();
                headers.insert(
                    "Authorization".to_string(),
                    "Bearer $OPENAI_API_KEY".to_string(),
                );
                headers
            },
        },
        models: vec![openai_gpt5_model()],
    }
}

/// Create a GPT-5 model spec
pub fn openai_gpt5_model() -> ModelSpec {
    ModelSpec {
        id: "gpt-5".to_string(),
        aliases: Some(vec!["gpt5".to_string()]),
        family: "gpt".to_string(),
        endpoints: Endpoints {
            chat_completion: EndpointConfig {
                method: "POST".to_string(),
                path: "/chat/completions".to_string(),
                protocol: "http".to_string(),
                query: None,
                headers: None,
            },
            streaming_chat_completion: EndpointConfig {
                method: "POST".to_string(),
                path: "/chat/completions".to_string(),
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
            tools_supported: true,
            parallel_tool_calls_default: true,
            can_disable_parallel_tool_calls: true,
            disable_switch: Some(json!({"parallel_tool_calls": false})),
        },
        json_output: JsonOutputConfig {
            native_param: true,
            strategy: "response_format".to_string(),
        },
        capabilities: None,
        parameters: json!({
            "temperature": {
                "type": "number",
                "minimum": 0.0,
                "maximum": 2.0,
                "default": 1.0
            },
            "top_p": {
                "type": "number",
                "minimum": 0.0,
                "maximum": 1.0,
                "default": 1.0
            },
            "top_k": {
                "type": "integer",
                "minimum": 1,
                "maximum": 100
            },
            "frequency_penalty": {
                "type": "number",
                "minimum": -2.0,
                "maximum": 2.0,
                "default": 0.0
            },
            "presence_penalty": {
                "type": "number",
                "minimum": -2.0,
                "maximum": 2.0,
                "default": 0.0
            }
        }),
        constraints: Constraints {
            system_prompt_location: "first_message".to_string(),
            forbid_unknown_top_level_fields: true,
            mutually_exclusive: vec![],
            resolution_preferences: vec![],
            limits: ConstraintLimits {
                max_tool_schema_bytes: 16384,
                max_system_prompt_bytes: 32768,
            },
        },
        mappings: Mappings {
            paths: {
                let mut paths = std::collections::HashMap::new();
                paths.insert(
                    "$.limits.max_output_tokens".to_string(),
                    "$.max_tokens".to_string(),
                );
                paths.insert(
                    "$.sampling.temperature".to_string(),
                    "$.temperature".to_string(),
                );
                paths.insert(
                    "$.sampling.top_p".to_string(),
                    "$.top_p".to_string(),
                );
                paths.insert(
                    "$.sampling.top_k".to_string(),
                    "$.top_k".to_string(),
                );
                paths.insert(
                    "$.sampling.frequency_penalty".to_string(),
                    "$.frequency_penalty".to_string(),
                );
                paths.insert(
                    "$.sampling.presence_penalty".to_string(),
                    "$.presence_penalty".to_string(),
                );
                paths.insert(
                    "$.response_format".to_string(),
                    "$.response_format".to_string(),
                );
                paths
            },
            flags: std::collections::HashMap::new(),
        },
        response_normalization: ResponseNormalization {
            sync: SyncNormalization {
                content_path: "$.choices[0].message.content".to_string(),
                finish_reason_path: "$.choices[0].finish_reason".to_string(),
                finish_reason_map: std::collections::HashMap::new(),
            },
            stream: StreamNormalization {
                protocol: "sse".to_string(),
                event_selector: EventSelector {
                    type_path: "$.choices[0].delta".to_string(),
                    routes: vec![],
                },
            },
        },
    }
}

/// Create an Anthropic provider spec
pub fn anthropic_provider() -> ProviderSpec {
    ProviderSpec {
        spec_version: "1.0.0".to_string(),
        provider: ProviderInfo {
            name: "anthropic".to_string(),
            base_url: "https://api.anthropic.com/v1".to_string(),
            headers: {
                let mut headers = std::collections::HashMap::new();
                headers.insert("X-API-Key".to_string(), "$ANTHROPIC_API_KEY".to_string());
                headers.insert(
                    "anthropic-version".to_string(),
                    "2024-02-15".to_string(),
                );
                headers
            },
        },
        models: vec![anthropic_claude_model()],
    }
}

/// Create a Claude Opus model spec
pub fn anthropic_claude_model() -> ModelSpec {
    ModelSpec {
        id: "claude-opus-4.1".to_string(),
        aliases: Some(vec!["claude-opus".to_string()]),
        family: "claude".to_string(),
        endpoints: Endpoints {
            chat_completion: EndpointConfig {
                method: "POST".to_string(),
                path: "/messages".to_string(),
                protocol: "http".to_string(),
                query: None,
                headers: None,
            },
            streaming_chat_completion: EndpointConfig {
                method: "POST".to_string(),
                path: "/messages".to_string(),
                protocol: "sse".to_string(),
                query: None,
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
            can_disable_parallel_tool_calls: false,
            disable_switch: None,
        },
        json_output: JsonOutputConfig {
            native_param: false,
            strategy: "system_prompt".to_string(),
        },
        capabilities: None,
        parameters: json!({
            "temperature": {
                "type": "number",
                "minimum": 0.0,
                "maximum": 1.0,
                "default": 1.0
            }
        }),
        constraints: Constraints {
            system_prompt_location: "first_message".to_string(),
            forbid_unknown_top_level_fields: true,
            mutually_exclusive: vec![],
            resolution_preferences: vec![],
            limits: ConstraintLimits {
                max_tool_schema_bytes: 16384,
                max_system_prompt_bytes: 100000,
            },
        },
        mappings: Mappings {
            paths: {
                let mut paths = std::collections::HashMap::new();
                paths.insert(
                    "$.limits.max_output_tokens".to_string(),
                    "$.max_tokens".to_string(),
                );
                paths
            },
            flags: std::collections::HashMap::new(),
        },
        response_normalization: ResponseNormalization {
            sync: SyncNormalization {
                content_path: "$.content[0].text".to_string(),
                finish_reason_path: "$.stop_reason".to_string(),
                finish_reason_map: std::collections::HashMap::new(),
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

/// Create a provider with limited capabilities
pub fn limited_provider() -> ProviderSpec {
    ProviderSpec {
        spec_version: "1.0.0".to_string(),
        provider: ProviderInfo {
            name: "limited".to_string(),
            base_url: "https://api.limited.com".to_string(),
            headers: std::collections::HashMap::new(),
        },
        models: vec![ModelSpec {
            id: "basic-model".to_string(),
            aliases: None,
            family: "basic".to_string(),
            endpoints: Endpoints {
                chat_completion: EndpointConfig {
                    method: "POST".to_string(),
                    path: "/chat".to_string(),
                    protocol: "http".to_string(),
                    query: None,
                    headers: None,
                },
                streaming_chat_completion: EndpointConfig {
                    method: "POST".to_string(),
                    path: "/chat".to_string(),
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
            capabilities: None,
            parameters: json!({}),
            constraints: Constraints {
                system_prompt_location: "first_message".to_string(),
                forbid_unknown_top_level_fields: false,
                mutually_exclusive: vec![],
                resolution_preferences: vec![],
                limits: ConstraintLimits {
                    max_tool_schema_bytes: 8192,
                    max_system_prompt_bytes: 16384,
                },
            },
            mappings: Mappings {
                paths: std::collections::HashMap::new(),
                flags: std::collections::HashMap::new(),
            },
            response_normalization: ResponseNormalization {
                sync: SyncNormalization {
                    content_path: "$.content".to_string(),
                    finish_reason_path: "$.finish_reason".to_string(),
                    finish_reason_map: std::collections::HashMap::new(),
                },
                stream: StreamNormalization {
                    protocol: "sse".to_string(),
                    event_selector: EventSelector {
                        type_path: "$.type".to_string(),
                        routes: vec![],
                    },
                },
            },
        }],
    }
}

/// Create a multi-turn conversation prompt
pub fn multi_turn_prompt() -> PromptSpec {
    PromptSpec {
        model_class: "Chat".to_string(),
        messages: vec![
            Message {
                role: MessageRole::System,
                content: "You are a helpful assistant.".to_string(),
                name: None,
                metadata: None,
            },
            Message {
                role: MessageRole::User,
                content: "What is the capital of France?".to_string(),
                name: None,
                metadata: None,
            },
            Message {
                role: MessageRole::Assistant,
                content: "The capital of France is Paris.".to_string(),
                name: None,
                metadata: None,
            },
            Message {
                role: MessageRole::User,
                content: "What is its population?".to_string(),
                name: None,
                metadata: None,
            },
        ],
        tools: None,
        tool_choice: None,
        response_format: None,
        sampling: None,
        limits: None,
        media: None,
        strict_mode: StrictMode::Warn,
    }
}

/// Create a prompt with JSON response format
pub fn prompt_with_json_response() -> PromptSpec {
    let mut prompt = minimal_prompt();
    prompt.response_format = Some(ResponseFormat::JsonObject);
    prompt
}

/// Assert that translation succeeds
pub fn assert_translation_succeeds(
    prompt: &PromptSpec,
    provider: &ProviderSpec,
    model: &str,
) -> specado_core::TranslationResult {
    match specado_core::translate(prompt, provider, model, StrictMode::Warn) {
        Ok(result) => result,
        Err(e) => panic!("Translation failed: {}", e),
    }
}

/// Assert that translation fails
#[allow(dead_code)]
pub fn assert_translation_fails(prompt: &PromptSpec, provider: &ProviderSpec, model: &str) {
    if specado_core::translate(prompt, provider, model, StrictMode::Strict).is_ok() {
        panic!("Translation should have failed but succeeded");
    }
}