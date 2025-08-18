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
    /// Property: Translation is deterministic - same input produces same lossiness
    #[test]
    fn prop_translation_deterministic_lossiness(
        prompt_spec in prompt_spec_strategy()
    ) {
        let provider_spec = minimal_provider_spec();
        let model_id = "test-model";
        let mode = prompt_spec.strict_mode.clone();
        
        // Run translation twice with the same input
        let result1 = translate(&prompt_spec, &provider_spec, model_id, mode);
        let result2 = translate(&prompt_spec, &provider_spec, model_id, mode);
        
        match (result1, result2) {
            (Ok(r1), Ok(r2)) => {
                // Same lossiness codes should be reported
                let codes1: Vec<_> = r1.lossiness.items.iter().map(|i| &i.code).collect();
                let codes2: Vec<_> = r2.lossiness.items.iter().map(|i| &i.code).collect();
                assert_eq!(codes1, codes2, "Lossiness should be deterministic");
                
                // Same severity levels should be reported
                let severities1: Vec<_> = r1.lossiness.items.iter().map(|i| &i.severity).collect();
                let severities2: Vec<_> = r2.lossiness.items.iter().map(|i| &i.severity).collect();
                assert_eq!(severities1, severities2, "Severity should be deterministic");
            }
            (Err(_), Err(_)) => {
                // Both failed - this is deterministic
            }
            _ => {
                panic!("Non-deterministic translation results");
            }
        }
    }
    
    /// Property: Temperature values are always clamped to valid ranges (0.0-2.0)
    #[test]
    fn prop_temperature_always_in_range(
        mut prompt_spec in prompt_spec_strategy(),
        raw_temp in -10.0f32..=10.0f32  // Use a reasonable range for testing
    ) {
        // Set arbitrary temperature value
        prompt_spec.sampling = Some(SamplingParams {
            temperature: Some(raw_temp),
            top_p: None,
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
        });
        
        let provider_spec = minimal_provider_spec();
        let model_id = "test-model";
        
        if let Ok(result) = translate(&prompt_spec, &provider_spec, model_id, StrictMode::Coerce) {
            // Check if temperature is in the output
            if let Some(temp) = result.provider_request_json.get("temperature") {
                if let Some(temp_val) = temp.as_f64() {
                    // Temperature must be within valid range
                    assert!(temp_val >= 0.0 && temp_val <= 2.0, 
                           "Temperature {} should be clamped to [0.0, 2.0]", temp_val);
                }
            }
            
            // If temperature was out of range, there should be a Clamp lossiness
            if raw_temp < 0.0 || raw_temp > 2.0 {
                let has_clamp = result.lossiness.items.iter()
                    .any(|item| matches!(item.code, LossinessCode::Clamp) && 
                                item.path.contains("temperature"));
                // In Coerce mode, we should see the clamp
                assert!(has_clamp || raw_temp.is_nan() || raw_temp.is_infinite());
            }
        }
    }
    
    /// Property: top_p values are always clamped to valid ranges (0.0-1.0)
    #[test]
    fn prop_top_p_always_in_range(
        mut prompt_spec in prompt_spec_strategy(),
        raw_top_p in -1000.0f32..=1000.0f32  // Use a reasonable range for testing
    ) {
        // Set arbitrary top_p value
        prompt_spec.sampling = Some(SamplingParams {
            temperature: None,
            top_p: Some(raw_top_p),
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
        });
        
        let provider_spec = minimal_provider_spec();
        let model_id = "test-model";
        
        if let Ok(result) = translate(&prompt_spec, &provider_spec, model_id, StrictMode::Coerce) {
            // Check if top_p is in the output
            if let Some(top_p) = result.provider_request_json.get("top_p") {
                if let Some(top_p_val) = top_p.as_f64() {
                    // top_p must be within valid range or not be a normal float
                    if top_p_val.is_finite() && !top_p_val.is_nan() {
                        // Only check range for valid float values
                        // The engine might pass through invalid values unchanged
                        if raw_top_p.is_finite() && !raw_top_p.is_nan() && (raw_top_p < 0.0 || raw_top_p > 1.0) {
                            // If input was out of range, we should have clamped or have lossiness
                            let has_clamp = result.lossiness.items.iter()
                                .any(|item| matches!(item.code, LossinessCode::Clamp));
                            assert!(has_clamp || (top_p_val >= 0.0 && top_p_val <= 1.0),
                                   "top_p {} should be clamped to [0.0, 1.0] or marked as clamped", top_p_val);
                        }
                    }
                }
            }
            
            // If top_p was out of range, there should be a Clamp lossiness
            if raw_top_p < 0.0 || raw_top_p > 1.0 {
                let has_clamp = result.lossiness.items.iter()
                    .any(|item| matches!(item.code, LossinessCode::Clamp) && 
                                item.path.contains("top_p"));
                // In Coerce mode, we should see the clamp
                assert!(has_clamp || raw_top_p.is_nan() || raw_top_p.is_infinite());
            }
        }
    }
    
    /// Property: top_k values are always positive integers
    #[test]
    fn prop_top_k_always_positive(
        mut prompt_spec in prompt_spec_strategy(),
        raw_top_k in any::<i32>()
    ) {
        // Set arbitrary top_k value (could be negative)
        let top_k_val = if raw_top_k < 0 { 
            None  // Negative values should be dropped or clamped
        } else {
            Some(raw_top_k as u32)
        };
        
        prompt_spec.sampling = Some(SamplingParams {
            temperature: None,
            top_p: None,
            top_k: top_k_val,
            frequency_penalty: None,
            presence_penalty: None,
        });
        
        let provider_spec = minimal_provider_spec();
        let model_id = "test-model";
        
        if let Ok(result) = translate(&prompt_spec, &provider_spec, model_id, StrictMode::Coerce) {
            // Check if top_k is in the output
            if let Some(top_k) = result.provider_request_json.get("top_k") {
                if let Some(top_k_val) = top_k.as_u64() {
                    // top_k must be positive
                    assert!(top_k_val > 0, "top_k {} should be positive", top_k_val);
                }
            }
        }
    }
    
    /// Property: Frequency and presence penalties are clamped to [-2.0, 2.0]
    #[test]
    fn prop_penalties_in_range(
        mut prompt_spec in prompt_spec_strategy(),
        raw_freq in -10.0f32..=10.0f32,  // Use reasonable ranges
        raw_pres in -10.0f32..=10.0f32
    ) {
        prompt_spec.sampling = Some(SamplingParams {
            temperature: None,
            top_p: None,
            top_k: None,
            frequency_penalty: Some(raw_freq),
            presence_penalty: Some(raw_pres),
        });
        
        let provider_spec = minimal_provider_spec();
        let model_id = "test-model";
        
        if let Ok(result) = translate(&prompt_spec, &provider_spec, model_id, StrictMode::Coerce) {
            // Check frequency penalty
            if let Some(freq) = result.provider_request_json.get("frequency_penalty") {
                if let Some(freq_val) = freq.as_f64() {
                    // Only check range for finite values
                    if freq_val.is_finite() && raw_freq.is_finite() {
                        // If input was out of range, output should be clamped
                        if raw_freq < -2.0 || raw_freq > 2.0 {
                            let has_clamp = result.lossiness.items.iter()
                                .any(|item| matches!(item.code, LossinessCode::Clamp));
                            assert!(has_clamp || (freq_val >= -2.0 && freq_val <= 2.0),
                                   "frequency_penalty {} should be clamped or marked", freq_val);
                        }
                    }
                }
            }
            
            // Check presence penalty
            if let Some(pres) = result.provider_request_json.get("presence_penalty") {
                if let Some(pres_val) = pres.as_f64() {
                    // Only check range for finite values
                    if pres_val.is_finite() && raw_pres.is_finite() {
                        // If input was out of range, output should be clamped
                        if raw_pres < -2.0 || raw_pres > 2.0 {
                            let has_clamp = result.lossiness.items.iter()
                                .any(|item| matches!(item.code, LossinessCode::Clamp));
                            assert!(has_clamp || (pres_val >= -2.0 && pres_val <= 2.0),
                                   "presence_penalty {} should be clamped or marked", pres_val);
                        }
                    }
                }
            }
            
            // Check for clamping lossiness if values were out of range
            if raw_freq < -2.0 || raw_freq > 2.0 {
                let has_clamp = result.lossiness.items.iter()
                    .any(|item| matches!(item.code, LossinessCode::Clamp) && 
                                item.path.contains("frequency_penalty"));
                assert!(has_clamp || raw_freq.is_nan() || raw_freq.is_infinite());
            }
            
            if raw_pres < -2.0 || raw_pres > 2.0 {
                let has_clamp = result.lossiness.items.iter()
                    .any(|item| matches!(item.code, LossinessCode::Clamp) && 
                                item.path.contains("presence_penalty"));
                assert!(has_clamp || raw_pres.is_nan() || raw_pres.is_infinite());
            }
        }
    }
    
    /// Property: Message structure is preserved (order and count)
    #[test]
    fn prop_message_structure_preservation(
        messages in proptest::collection::vec(message_strategy(), 1..10)
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
                    // Original message order should be preserved
                    let mut original_idx = 0;
                    for output_msg in arr {
                        if let Some(content) = output_msg.get("content") {
                            // Find this content in original messages
                            if original_idx < messages.len() {
                                let orig_content = &messages[original_idx].content;
                                if content.as_str() == Some(orig_content) {
                                    original_idx += 1;
                                }
                            }
                        }
                    }
                    // We should have found all original messages in order
                    assert!(original_idx >= messages.len() || 
                           messages.iter().any(|m| m.role == MessageRole::System),
                           "Message order should be preserved");
                }
            }
        }
    }
    
    /// Property: Token limits are always positive and reasonable
    #[test]
    fn prop_token_limits_valid(
        mut prompt_spec in prompt_spec_strategy(),
        max_output in -1000i32..=2000000i32,  // Include negative and very large values
        max_prompt in -1000i32..=2000000i32,
        reasoning in -1000i32..=100000i32
    ) {
        // Set potentially invalid token limits
        prompt_spec.limits = Some(Limits {
            max_output_tokens: if max_output > 0 { Some(max_output as u32) } else { None },
            max_prompt_tokens: if max_prompt > 0 { Some(max_prompt as u32) } else { None },
            reasoning_tokens: if reasoning > 0 { Some(reasoning as u32) } else { None },
        });
        
        let provider_spec = minimal_provider_spec();
        let model_id = "test-model";
        
        if let Ok(result) = translate(&prompt_spec, &provider_spec, model_id, StrictMode::Coerce) {
            // Check max_tokens/max_output_tokens
            if let Some(max_tokens) = result.provider_request_json.get("max_tokens")
                .or_else(|| result.provider_request_json.get("max_output_tokens")) {
                if let Some(tokens_val) = max_tokens.as_u64() {
                    // Token limits should be positive if present
                    // The actual upper bound depends on the provider
                    assert!(tokens_val > 0, "max_tokens {} should be positive", tokens_val);
                    
                    // If a very large value was provided, check for clamping
                    if max_output > 1000000 {
                        let has_clamp = result.lossiness.items.iter()
                            .any(|item| matches!(item.code, LossinessCode::Clamp) && 
                                       item.path.contains("max_output_tokens"));
                        // Very large values should either be clamped or marked
                        assert!(tokens_val <= 1000000 || has_clamp,
                               "Very large token limits should be clamped or marked");
                    }
                }
            }
        }
    }
    
    /// Property: System prompt relocation is handled consistently
    #[test]
    fn prop_system_prompt_relocation_consistency(
        mut messages in proptest::collection::vec(message_strategy(), 2..8)
    ) {
        // Ensure we have a system message not at the beginning
        if messages.len() > 2 {
            messages[0].role = MessageRole::User;
            let mid_idx = messages.len() / 2;
            messages[mid_idx].role = MessageRole::System;
        }
        
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
            // Check if system prompt was relocated
            let has_system = messages.iter().any(|m| m.role == MessageRole::System);
            let system_not_first = messages.first()
                .map(|m| m.role != MessageRole::System)
                .unwrap_or(false);
            
            if has_system && system_not_first {
                // Check if system prompt was relocated or if there's a lossiness item
                let has_relocate = result.lossiness.items.iter()
                    .any(|item| matches!(item.code, LossinessCode::Relocate));
                
                // Check if system message is now first in output
                let system_now_first = if let Some(messages) = result.provider_request_json.get("messages") {
                    messages.as_array()
                        .and_then(|arr| arr.first())
                        .and_then(|msg| msg.get("role"))
                        .map(|role| role == "system")
                        .unwrap_or(false)
                } else {
                    false
                };
                
                // Either relocation happened (system is now first) or it's marked in lossiness
                // or the provider doesn't require system first
                assert!(system_now_first || has_relocate || 
                       provider_spec.models[0].constraints.system_prompt_location != "first_message",
                       "System prompt should be relocated or marked as relocated");
            }
        }
    }
    
    /// Property: Model mapping preserves essential properties
    #[test]
    fn prop_model_mapping_consistency(
        prompt_spec in prompt_spec_strategy()
    ) {
        // Create a provider spec with model mappings
        let mut provider_spec = minimal_provider_spec();
        provider_spec.models[0].mappings.paths.insert(
            "messages".to_string(),
            "messages".to_string()
        );
        provider_spec.models[0].mappings.paths.insert(
            "temperature".to_string(),
            "temperature".to_string()
        );
        
        let model_id = "test-model";
        
        if let Ok(result) = translate(&prompt_spec, &provider_spec, model_id, StrictMode::Warn) {
            // Messages should be present in some form
            let has_messages = result.provider_request_json.get("messages").is_some() ||
                               result.provider_request_json.get("prompt").is_some();
            let messages_dropped = result.lossiness.items.iter()
                .any(|item| item.path.contains("messages") && matches!(item.code, LossinessCode::Drop));
            
            assert!(has_messages || messages_dropped,
                   "Messages should be mapped or marked as dropped");
            
            // If we had sampling params, check if they're handled
            if let Some(sampling) = &prompt_spec.sampling {
                // At least one sampling parameter should be present or marked as lost
                let has_any_sampling = 
                    result.provider_request_json.get("temperature").is_some() ||
                    result.provider_request_json.get("top_p").is_some() ||
                    result.provider_request_json.get("top_k").is_some() ||
                    result.provider_request_json.get("frequency_penalty").is_some() ||
                    result.provider_request_json.get("presence_penalty").is_some();
                
                let sampling_lost = result.lossiness.items.iter()
                    .any(|item| item.path.contains("sampling") || 
                               item.path.contains("temperature") ||
                               item.path.contains("top_p"));
                
                // If sampling params were provided, they should be handled somehow
                if sampling.temperature.is_some() || sampling.top_p.is_some() || 
                   sampling.top_k.is_some() || sampling.frequency_penalty.is_some() ||
                   sampling.presence_penalty.is_some() {
                    assert!(has_any_sampling || sampling_lost,
                           "Sampling params should be mapped or marked as lost");
                }
            }
        }
    }

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
    
    /// Property: Transformation preserves information for theoretical reversibility
    #[test]
    fn prop_transformation_reversibility(
        prompt_spec in prompt_spec_strategy()
    ) {
        let provider_spec = minimal_provider_spec();
        let model_id = "test-model";
        
        if let Ok(result) = translate(&prompt_spec, &provider_spec, model_id, StrictMode::Warn) {
            // All critical information should be preserved or marked as lost
            
            // 1. Message count should be preserved or increased (system prompts added)
            if let Some(messages) = result.provider_request_json.get("messages") {
                if let Some(arr) = messages.as_array() {
                    assert!(arr.len() >= prompt_spec.messages.len(),
                           "Message count should not decrease");
                }
            }
            
            // 2. Every lossiness item should have proper before/after tracking
            for item in &result.lossiness.items {
                match item.code {
                    LossinessCode::Clamp => {
                        // Clamp should record original value
                        assert!(item.before.is_some() || item.message.contains("clamped"),
                               "Clamp should record original value");
                    }
                    LossinessCode::Drop => {
                        // Drop should record what was dropped
                        assert!(item.before.is_some() || item.path.len() > 0,
                               "Drop should record what was lost");
                    }
                    LossinessCode::Relocate => {
                        // Relocate should preserve content
                        assert!(item.path.len() > 0,
                               "Relocate should specify what was moved");
                    }
                    _ => {}
                }
            }
            
            // 3. Tool information should be preserved or marked as unsupported
            if let Some(tools) = &prompt_spec.tools {
                if !tools.is_empty() {
                    let has_tools = result.provider_request_json.get("tools").is_some() ||
                                   result.provider_request_json.get("functions").is_some();
                    let has_lossiness = result.lossiness.items.iter()
                        .any(|item| item.path.contains("tools"));
                    assert!(has_tools || has_lossiness,
                           "Tools should be preserved or marked as lost");
                }
            }
        }
    }
    
    /// Property: Lossiness severity ordering is consistent
    #[test]
    fn prop_lossiness_severity_ordering(
        prompt_spec in prompt_spec_strategy()
    ) {
        let provider_spec = minimal_provider_spec();
        let model_id = "test-model";
        
        // Test with all three strict modes
        let modes = vec![StrictMode::Strict, StrictMode::Warn, StrictMode::Coerce];
        let mut results = vec![];
        
        for mode in modes {
            let mut spec = prompt_spec.clone();
            spec.strict_mode = mode;
            if let Ok(result) = translate(&spec, &provider_spec, model_id, mode) {
                results.push((mode, result));
            }
        }
        
        // If we have results from multiple modes, check severity consistency
        if results.len() > 1 {
            for i in 0..results.len() - 1 {
                let (mode1, result1) = &results[i];
                let (mode2, result2) = &results[i + 1];
                
                // Count errors in each result
                let errors1 = result1.lossiness.items.iter()
                    .filter(|item| matches!(item.severity, Severity::Error))
                    .count();
                let errors2 = result2.lossiness.items.iter()
                    .filter(|item| matches!(item.severity, Severity::Error))
                    .count();
                
                // Stricter modes should have equal or more errors
                match (mode1, mode2) {
                    (StrictMode::Strict, StrictMode::Warn) |
                    (StrictMode::Strict, StrictMode::Coerce) |
                    (StrictMode::Warn, StrictMode::Coerce) => {
                        assert!(errors1 >= errors2,
                               "Stricter mode {:?} should have >= errors than {:?}", mode1, mode2);
                    }
                    _ => {}
                }
            }
        }
    }
    
    /// Property: Provider constraints are respected
    #[test]
    fn prop_provider_constraints_respected(
        mut prompt_spec in prompt_spec_strategy()
    ) {
        // Create a provider with specific constraints
        let mut provider_spec = minimal_provider_spec();
        provider_spec.models[0].constraints.limits.max_tool_schema_bytes = 1000;
        provider_spec.models[0].constraints.limits.max_system_prompt_bytes = 500;
        
        // Add a large system prompt
        if let Some(msg) = prompt_spec.messages.iter_mut().find(|m| m.role == MessageRole::System) {
            msg.content = "x".repeat(600); // Exceeds limit
        }
        
        let model_id = "test-model";
        
        match translate(&prompt_spec, &provider_spec, model_id, StrictMode::Strict) {
            Ok(result) => {
                // If translation succeeded, constraints should be enforced
                if let Some(messages) = result.provider_request_json.get("messages") {
                    if let Some(arr) = messages.as_array() {
                        for msg in arr {
                            if msg.get("role") == Some(&serde_json::json!("system")) {
                                if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                                    assert!(content.len() <= 500 || 
                                           result.lossiness.items.iter().any(|item| 
                                               matches!(item.code, LossinessCode::Clamp)),
                                           "System prompt should be clamped to constraint limit");
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => {
                // Failing due to constraint violation is acceptable in Strict mode
            }
        }
    }
    
    /// Property: Response format handling is consistent
    #[test]
    fn prop_response_format_consistency(
        mut prompt_spec in prompt_spec_strategy()
    ) {
        // Add JSON response format
        prompt_spec.response_format = Some(ResponseFormat::JsonObject);
        
        let provider_spec = minimal_provider_spec();
        let model_id = "test-model";
        
        if let Ok(result) = translate(&prompt_spec, &provider_spec, model_id, StrictMode::Warn) {
            // Check how JSON output was handled
            let has_response_format = result.provider_request_json.get("response_format").is_some();
            let has_json_mode = result.provider_request_json.get("json_mode").is_some();
            let has_system_prompt_json = if let Some(messages) = result.provider_request_json.get("messages") {
                messages.as_array().map(|arr| {
                    arr.iter().any(|msg| {
                        msg.get("role") == Some(&serde_json::json!("system")) &&
                        msg.get("content").and_then(|c| c.as_str())
                           .map(|s| s.contains("JSON")).unwrap_or(false)
                    })
                }).unwrap_or(false)
            } else {
                false
            };
            
            // Should use one of the strategies
            assert!(has_response_format || has_json_mode || has_system_prompt_json ||
                   result.lossiness.items.iter().any(|item| 
                       item.path.contains("response_format")),
                   "JSON output should be handled by some strategy");
        }
    }
}