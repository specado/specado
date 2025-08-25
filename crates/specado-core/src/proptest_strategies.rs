//! Property-based testing strategies for generating test data
//!
//! This module provides proptest strategies for generating random
//! but valid instances of core Specado types for property testing.

#![cfg(test)]

use crate::types::*;
use crate::error::{LossinessCode, Severity, StrictMode};
use proptest::prelude::*;
use proptest::collection::{vec, hash_map};
use proptest::option;
use serde_json::Value;

/// Strategy for generating message roles
pub fn message_role_strategy() -> impl Strategy<Value = MessageRole> {
    prop_oneof![
        Just(MessageRole::System),
        Just(MessageRole::User),
        Just(MessageRole::Assistant),
    ]
}

/// Strategy for generating messages
pub fn message_strategy() -> impl Strategy<Value = Message> {
    (
        message_role_strategy(),
        "[a-zA-Z0-9 .,!?]{1,200}",  // content
        option::of("[a-zA-Z0-9_]{1,20}"),  // name
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
pub fn sampling_params_strategy() -> impl Strategy<Value = SamplingParams> {
    (
        option::of(0.0f32..=2.0),  // temperature
        option::of(0.0f32..=1.0),  // top_p
        option::of(1u32..=100),     // top_k
        option::of(-2.0f32..=2.0),  // frequency_penalty
        option::of(-2.0f32..=2.0),  // presence_penalty
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
pub fn limits_strategy() -> impl Strategy<Value = Limits> {
    (
        option::of(1u32..=100000),  // max_output_tokens
        option::of(1u32..=10000),   // reasoning_tokens
        option::of(1u32..=100000),  // max_prompt_tokens
    ).prop_map(|(max_output_tokens, reasoning_tokens, max_prompt_tokens)| {
        Limits {
            max_output_tokens,
            reasoning_tokens,
            max_prompt_tokens,
        }
    })
}

/// Strategy for generating strict modes
pub fn strict_mode_strategy() -> impl Strategy<Value = StrictMode> {
    prop_oneof![
        Just(StrictMode::Strict),
        Just(StrictMode::Warn),
        Just(StrictMode::Coerce),
    ]
}

/// Strategy for generating simple tools
pub fn tool_strategy() -> impl Strategy<Value = Tool> {
    (
        "[a-zA-Z_][a-zA-Z0-9_]{0,30}",  // name
        option::of("[a-zA-Z0-9 .,]{1,100}"),  // description
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
pub fn prompt_spec_strategy() -> impl Strategy<Value = PromptSpec> {
    (
        Just("Chat".to_string()),  // model_class
        vec(message_strategy(), 1..5),  // messages
        option::of(vec(tool_strategy(), 0..3)),  // tools
        option::of(sampling_params_strategy()),  // sampling
        option::of(limits_strategy()),  // limits
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
            advanced: None,
            strict_mode,
        }
    })
}

/// Strategy for generating simple JSON values with controlled depth
pub fn json_value_strategy() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(|n| Value::Number(n.into())),
        "[a-zA-Z0-9 ]{0,50}".prop_map(Value::String),
    ];
    
    leaf.prop_recursive(
        3,  // max depth
        10, // max size
        5,  // items per collection
        |inner| {
            prop_oneof![
                vec(inner.clone(), 0..5).prop_map(Value::Array),
                hash_map(
                    "[a-zA-Z_][a-zA-Z0-9_]{0,20}",
                    inner,
                    0..5
                ).prop_map(|m| Value::Object(m.into_iter().collect())),
            ]
        },
    )
}

/// Strategy for generating JSONPath expressions
pub fn jsonpath_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Root paths
        Just("$".to_string()),
        
        // Simple property access
        "[a-zA-Z_][a-zA-Z0-9_]{0,20}".prop_map(|s| format!("$.{}", s)),
        
        // Nested property access
        (
            "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
        ).prop_map(|(a, b)| format!("$.{}.{}", a, b)),
        
        // Array access
        (
            "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            0usize..10,
        ).prop_map(|(prop, idx)| format!("$.{}[{}]", prop, idx)),
        
        // Wildcard
        "[a-zA-Z_][a-zA-Z0-9_]{0,10}".prop_map(|s| format!("$.{}.*", s)),
        
        // Recursive descent
        "[a-zA-Z_][a-zA-Z0-9_]{0,10}".prop_map(|s| format!("$..{}", s)),
    ]
}

/// Strategy for generating provider names
pub fn provider_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("openai".to_string()),
        Just("anthropic".to_string()),
        Just("google".to_string()),
        Just("custom".to_string()),
    ]
}

/// Strategy for generating model IDs
pub fn model_id_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("gpt-5".to_string()),
        Just("claude-opus-4.1".to_string()),
        Just("gemini-pro".to_string()),
        "[a-zA-Z0-9-]{3,20}".prop_map(String::from),
    ]
}

/// Strategy for generating lossiness codes
pub fn lossiness_code_strategy() -> impl Strategy<Value = LossinessCode> {
    prop_oneof![
        Just(LossinessCode::Clamp),
        Just(LossinessCode::Drop),
        Just(LossinessCode::Emulate),
        Just(LossinessCode::Conflict),
        Just(LossinessCode::Relocate),
        Just(LossinessCode::Unsupported),
        Just(LossinessCode::MapFallback),
        Just(LossinessCode::PerformanceImpact),
    ]
}

/// Strategy for generating severities
pub fn severity_strategy() -> impl Strategy<Value = Severity> {
    prop_oneof![
        Just(Severity::Error),
        Just(Severity::Warning),
        Just(Severity::Info),
    ]
}

/// Strategy for generating lossiness items
pub fn lossiness_item_strategy() -> impl Strategy<Value = LossinessItem> {
    (
        lossiness_code_strategy(),
        jsonpath_strategy(),
        "[a-zA-Z0-9 .,]{1,100}",  // message
        severity_strategy(),
        option::of(json_value_strategy()),  // before
        option::of(json_value_strategy()),  // after
    ).prop_map(|(code, path, message, severity, before, after)| {
        LossinessItem {
            code,
            path,
            message,
            severity,
            before,
            after,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    proptest! {
        #[test]
        fn test_message_strategy_generates_valid_messages(msg in message_strategy()) {
            assert!(!msg.content.is_empty());
            // Roles are always valid by construction
        }

        #[test]
        fn test_prompt_spec_strategy_generates_valid_specs(spec in prompt_spec_strategy()) {
            assert!(!spec.messages.is_empty());
            assert_eq!(spec.model_class, "Chat");
        }

        #[test]
        fn test_json_value_strategy_generates_valid_json(value in json_value_strategy()) {
            // Should be serializable
            let serialized = serde_json::to_string(&value);
            assert!(serialized.is_ok());
        }

        #[test]
        fn test_jsonpath_strategy_generates_paths(path in jsonpath_strategy()) {
            assert!(path.starts_with('$') || path.starts_with(".."));
        }
    }
}