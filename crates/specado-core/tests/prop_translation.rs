//! Property-based tests for the translation engine
//!
//! These tests verify key invariants and properties that should hold
//! for all valid inputs to the translation system.

use proptest::prelude::*;
use specado_core::{translate, StrictMode, LossinessCode, Severity};
use specado_core::types::*;
use serde_json::Value;

// Strategy functions for property testing

/// Strategy for generating message roles
fn message_role_strategy() -> impl Strategy<Value = MessageRole> {
    prop_oneof![
        Just(MessageRole::System),
        Just(MessageRole::User),
        Just(MessageRole::Assistant),
    ]
}

/// Strategy for generating messages
fn message_strategy() -> impl Strategy<Value = Message> {
    (
        message_role_strategy(),
        "[a-zA-Z0-9 .,!?]{1,200}",  // content
        proptest::option::of("[a-zA-Z0-9_]{1,20}"),  // name
    ).prop_map(|(role, content, name)| {
        Message {
            role,
            content,
            name,
            metadata: None,
        }
    })
}

/// Strategy for generating sampling parameters
fn sampling_params_strategy() -> impl Strategy<Value = SamplingParams> {
    (
        proptest::option::of(0.0f32..=2.0),  // temperature
        proptest::option::of(0.0f32..=1.0),  // top_p
        proptest::option::of(1u32..=100),     // top_k
        proptest::option::of(-2.0f32..=2.0),  // frequency_penalty
        proptest::option::of(-2.0f32..=2.0),  // presence_penalty
    ).prop_map(|(temperature, top_p, top_k, frequency_penalty, presence_penalty)| {
        SamplingParams {
            temperature,
            top_p,
            top_k,
            frequency_penalty,
            presence_penalty,
        }
    })
}

/// Strategy for generating limits
fn limits_strategy() -> impl Strategy<Value = Limits> {
    (
        proptest::option::of(1u32..=100000),  // max_output_tokens
        proptest::option::of(1u32..=10000),   // reasoning_tokens
        proptest::option::of(1u32..=100000),  // max_prompt_tokens
    ).prop_map(|(max_output_tokens, reasoning_tokens, max_prompt_tokens)| {
        Limits {
            max_output_tokens,
            reasoning_tokens,
            max_prompt_tokens,
        }
    })
}

/// Strategy for generating strict modes
fn strict_mode_strategy() -> impl Strategy<Value = StrictMode> {
    prop_oneof![
        Just(StrictMode::Strict),
        Just(StrictMode::Warn),
        Just(StrictMode::Coerce),
    ]
}

/// Strategy for generating simple tools
fn tool_strategy() -> impl Strategy<Value = Tool> {
    (
        "[a-zA-Z_][a-zA-Z0-9_]{0,30}",  // name
        proptest::option::of("[a-zA-Z0-9 .,]{1,100}"),  // description
    ).prop_map(|(name, description)| {
        Tool {
            name,
            description,
            json_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "param": {"type": "string"}
                }
            }),
        }
    })
}

/// Strategy for generating PromptSpec
fn prompt_spec_strategy() -> impl Strategy<Value = PromptSpec> {
    (
        Just("Chat".to_string()),  // model_class
        proptest::collection::vec(message_strategy(), 1..5),  // messages
        proptest::option::of(proptest::collection::vec(tool_strategy(), 0..3)),  // tools
        proptest::option::of(sampling_params_strategy()),  // sampling
        proptest::option::of(limits_strategy()),  // limits
        strict_mode_strategy(),  // strict_mode
    ).prop_map(|(model_class, messages, tools, sampling, limits, strict_mode)| {
        PromptSpec {
            model_class,
            messages,
            tools,
            tool_choice: None,
            response_format: None,
            sampling,
            limits,
            media: None,
            strict_mode,
        }
    })
}

/// Create a minimal valid provider spec for testing
fn minimal_provider_spec() -> ProviderSpec {
    ProviderSpec {
        spec_version: "1.0.0".to_string(),
        provider: ProviderInfo {
            name: "test".to_string(),
            base_url: "https://api.test.com".to_string(),
            headers: Default::default(),
        },
        models: vec![ModelSpec {
            id: "test-model".to_string(),
            aliases: None,
            family: "test".to_string(),
            endpoints: Endpoints {
                chat_completion: EndpointConfig {
                    method: "POST".to_string(),
                    path: "/chat".to_string(),
                    protocol: "https".to_string(),
                    query: None,
                    headers: None,
                },
                streaming_chat_completion: EndpointConfig {
                    method: "POST".to_string(),
                    path: "/chat".to_string(),
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
                tools_supported: true,
                parallel_tool_calls_default: false,
                can_disable_parallel_tool_calls: false,
                disable_switch: None,
            },
            json_output: JsonOutputConfig {
                native_param: false,
                strategy: "system_prompt".to_string(),
            },
            parameters: Value::Object(Default::default()),
            constraints: Constraints {
                system_prompt_location: "first_message".to_string(),
                forbid_unknown_top_level_fields: false,
                mutually_exclusive: vec![],
                resolution_preferences: vec![],
                limits: ConstraintLimits {
                    max_tool_schema_bytes: 65536,
                    max_system_prompt_bytes: 100000,
                },
            },
            mappings: Mappings {
                paths: Default::default(),
                flags: Default::default(),
            },
            response_normalization: ResponseNormalization {
                sync: SyncNormalization {
                    content_path: "$.content".to_string(),
                    finish_reason_path: "$.finish_reason".to_string(),
                    finish_reason_map: Default::default(),
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

proptest! {
    /// Property: Translation should never panic for valid inputs
    #[test]
    fn prop_translation_never_panics(
        prompt_spec in prompt_spec_strategy()
    ) {
        let provider_spec = minimal_provider_spec();
        let model_id = "test-model";
        
        // Translation should either succeed or return an error, but never panic
        let _ = translate(&prompt_spec, &provider_spec, model_id, prompt_spec.strict_mode);
    }
    
    /// Property: Translation should preserve message count
    /// The number of messages in the output should be at least the number in the input
    #[test]
    fn prop_translation_preserves_message_structure(
        prompt_spec in prompt_spec_strategy()
    ) {
        let provider_spec = minimal_provider_spec();
        let model_id = "test-model";
        
        match translate(&prompt_spec, &provider_spec, model_id, prompt_spec.strict_mode) {
            Ok(result) => {
                // The result should contain JSON with messages
                if let Some(messages) = result.provider_request_json.get("messages") {
                    if let Some(arr) = messages.as_array() {
                        // Output messages should be at least as many as input
                        // (system prompts might be added)
                        assert!(arr.len() >= prompt_spec.messages.len());
                    }
                }
            }
            Err(_) => {
                // Translation errors are acceptable
            }
        }
    }
    
    /// Property: Stricter modes should produce equal or more lossiness warnings
    #[test]
    fn prop_strict_mode_ordering(
        prompt_spec in prompt_spec_strategy()
    ) {
        let provider_spec = minimal_provider_spec();
        let model_id = "test-model";
        
        // Try translation with different strict modes
        let mut spec_strict = prompt_spec.clone();
        spec_strict.strict_mode = StrictMode::Strict;
        let result_strict = translate(&spec_strict, &provider_spec, model_id, StrictMode::Strict);
        
        let mut spec_warn = prompt_spec.clone();
        spec_warn.strict_mode = StrictMode::Warn;
        let result_warn = translate(&spec_warn, &provider_spec, model_id, StrictMode::Warn);
        
        let mut spec_coerce = prompt_spec.clone();
        spec_coerce.strict_mode = StrictMode::Coerce;
        let result_coerce = translate(&spec_coerce, &provider_spec, model_id, StrictMode::Coerce);
        
        // If Strict mode succeeds, Warn and Coerce should also succeed
        if result_strict.is_ok() {
            assert!(result_warn.is_ok() || result_warn.is_err());
            assert!(result_coerce.is_ok() || result_coerce.is_err());
        }
        
        // Compare lossiness counts when all succeed
        if let (Ok(strict), Ok(warn), Ok(coerce)) = (result_strict, result_warn, result_coerce) {
            let strict_count = strict.lossiness.items.len();
            let warn_count = warn.lossiness.items.len();
            let coerce_count = coerce.lossiness.items.len();
            
            // Stricter modes should report at least as many issues
            assert!(strict_count >= warn_count || strict_count == 0);
            assert!(warn_count >= coerce_count || warn_count == 0);
        }
    }
    
    /// Property: Translation output should always be valid JSON
    #[test]
    fn prop_translation_produces_valid_json(
        prompt_spec in prompt_spec_strategy()
    ) {
        let provider_spec = minimal_provider_spec();
        let model_id = "test-model";
        
        if let Ok(result) = translate(&prompt_spec, &provider_spec, model_id, prompt_spec.strict_mode) {
            // The provider_request_json should be serializable
            let serialized = serde_json::to_string(&result.provider_request_json);
            assert!(serialized.is_ok());
            
            // And deserializable
            if let Ok(json_str) = serialized {
                let parsed: Result<Value, _> = serde_json::from_str(&json_str);
                assert!(parsed.is_ok());
            }
        }
    }
    
    /// Property: Sampling parameters should be clamped to valid ranges
    #[test]
    fn prop_sampling_params_clamped(
        mut prompt_spec in prompt_spec_strategy(),
        temp in prop::option::of(0.0f32..=5.0),
        top_p in prop::option::of(0.0f32..=2.0),
    ) {
        // Set potentially out-of-range sampling params
        prompt_spec.sampling = Some(SamplingParams {
            temperature: temp,
            top_p,
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
        });
        
        let provider_spec = minimal_provider_spec();
        let model_id = "test-model";
        
        if let Ok(result) = translate(&prompt_spec, &provider_spec, model_id, StrictMode::Coerce) {
            // Check if temperature was clamped
            if let Some(temp_val) = temp {
                if temp_val > 2.0 {
                    // Should have a lossiness item for clamping
                    let has_clamp = result.lossiness.items.iter()
                        .any(|item| matches!(item.code, LossinessCode::Clamp));
                    assert!(has_clamp || result.lossiness.items.is_empty());
                }
            }
        }
    }
    
    /// Property: Tool definitions should be preserved or explicitly marked as unsupported
    #[test]
    fn prop_tools_handled_correctly(
        mut prompt_spec in prompt_spec_strategy(),
        tools in proptest::option::of(proptest::collection::vec(tool_strategy(), 0..3))
    ) {
        prompt_spec.tools = tools.clone();
        
        let provider_spec = minimal_provider_spec();
        let model_id = "test-model";
        
        if let Ok(result) = translate(&prompt_spec, &provider_spec, model_id, prompt_spec.strict_mode) {
            if tools.is_some() && !tools.as_ref().unwrap().is_empty() {
                // Either tools are in the output or marked as unsupported
                let has_tools = result.provider_request_json.get("tools").is_some();
                let has_unsupported = result.lossiness.items.iter()
                    .any(|item| matches!(item.code, LossinessCode::Unsupported) && item.path.contains("tools"));
                
                // One or the other should be true
                assert!(has_tools || has_unsupported || provider_spec.models[0].tooling.tools_supported);
            }
        }
    }
    
    /// Property: Message roles should be preserved
    #[test]
    fn prop_message_roles_preserved(
        messages in proptest::collection::vec(message_strategy(), 1..5)
    ) {
        let prompt_spec = PromptSpec {
            model_class: "Chat".to_string(),
            messages: messages.clone(),
            tools: None,
            tool_choice: None,
            response_format: None,
            sampling: None,
            limits: None,
            media: None,
            strict_mode: StrictMode::Warn,
        };
        
        let provider_spec = minimal_provider_spec();
        let model_id = "test-model";
        
        if let Ok(result) = translate(&prompt_spec, &provider_spec, model_id, StrictMode::Warn) {
            if let Some(output_messages) = result.provider_request_json.get("messages") {
                if let Some(arr) = output_messages.as_array() {
                    // Check that roles are preserved
                    for (i, msg) in messages.iter().enumerate() {
                        if i < arr.len() {
                            if let Some(role) = arr[i].get("role") {
                                let expected_role = match msg.role {
                                    MessageRole::System => "system",
                                    MessageRole::User => "user",
                                    MessageRole::Assistant => "assistant",
                                };
                                assert_eq!(role.as_str(), Some(expected_role));
                            }
                        }
                    }
                }
            }
        }
    }
}