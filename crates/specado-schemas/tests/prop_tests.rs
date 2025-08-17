//! Property-based tests for schema validation
//!
//! These tests verify that schema validators behave correctly
//! across a wide range of inputs.

use proptest::prelude::*;
use serde_json::{json, Value};
use specado_schemas::{
    create_prompt_spec_validator, create_provider_spec_validator,
    SchemaValidator
};

/// Strategy for generating random JSON values with controlled complexity
fn json_value_strategy() -> impl Strategy<Value = Value> {
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
                proptest::collection::vec(inner.clone(), 0..5).prop_map(Value::Array),
                proptest::collection::hash_map(
                    "[a-zA-Z_][a-zA-Z0-9_]{0,20}",
                    inner,
                    0..5
                ).prop_map(|m| Value::Object(m.into_iter().collect())),
            ]
        },
    )
}

/// Strategy for generating message role values
fn message_role_strategy() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("system"),
        Just("user"),
        Just("assistant"),
    ]
}

/// Strategy for generating valid-looking messages
fn message_strategy() -> impl Strategy<Value = Value> {
    (
        message_role_strategy(),
        "[a-zA-Z0-9 .,!?]{1,200}",  // content
        proptest::option::of("[a-zA-Z0-9_]{1,20}"),  // name
    ).prop_map(|(role, content, name)| {
        let mut msg = json!({
            "role": role,
            "content": content,
        });
        if let Some(n) = name {
            msg["name"] = json!(n);
        }
        msg
    })
}

/// Strategy for generating PromptSpec-like JSON
fn prompt_spec_like_strategy() -> impl Strategy<Value = Value> {
    (
        proptest::option::of("1.0|2.0|1.0.0"),  // spec_version
        proptest::option::of("[a-zA-Z0-9-]{1,30}"),  // id
        proptest::option::of("Chat|ReasoningChat|RAGChat"),  // model_class
        proptest::collection::vec(message_strategy(), 0..5),  // messages
    ).prop_map(|(spec_version, id, model_class, messages)| {
        let mut spec = json!({});
        
        if let Some(v) = spec_version {
            spec["spec_version"] = json!(v);
        }
        if let Some(i) = id {
            spec["id"] = json!(i);
        }
        if let Some(mc) = model_class {
            spec["model_class"] = json!(mc);
        }
        if !messages.is_empty() {
            spec["messages"] = json!(messages);
        }
        
        spec
    })
}

/// Strategy for generating ProviderSpec-like JSON
fn provider_spec_like_strategy() -> impl Strategy<Value = Value> {
    (
        proptest::option::of("1.0|2.0|1.0.0"),  // spec_version
        proptest::option::of("[a-zA-Z0-9-]{1,30}"),  // provider name
    ).prop_map(|(spec_version, provider_name)| {
        let mut spec = json!({});
        
        if let Some(v) = spec_version {
            spec["spec_version"] = json!(v);
        }
        if let Some(p) = provider_name {
            spec["provider"] = json!({
                "name": p,
                "base_url": "https://api.example.com"
            });
        }
        
        spec
    })
}

proptest! {
    /// Property: Validators should never panic on any JSON input
    #[test]
    fn prop_prompt_validator_never_panics(
        input in json_value_strategy()
    ) {
        let validator = create_prompt_spec_validator().expect("validator creation should succeed");
        
        // Validation should either succeed or fail, but never panic
        let _ = validator.validate(&input);
        let _ = validator.validate_basic(&input);
        let _ = validator.validate_partial(&input);
        let _ = validator.validate(&input); // validate() is Strict mode
    }
    
    /// Property: Provider validators should never panic on any JSON input
    #[test]
    fn prop_provider_validator_never_panics(
        input in json_value_strategy()
    ) {
        let validator = create_provider_spec_validator().expect("validator creation should succeed");
        
        // Validation should either succeed or fail, but never panic
        let _ = validator.validate(&input);
        let _ = validator.validate_basic(&input);
        let _ = validator.validate_partial(&input);
        let _ = validator.validate(&input); // validate() is Strict mode
    }
    
    /// Property: Validation mode ordering - stricter modes should produce equal or more errors
    #[test]
    fn prop_validation_mode_ordering(
        input in prompt_spec_like_strategy()
    ) {
        let validator = create_prompt_spec_validator().expect("validator creation should succeed");
        
        let basic_result = validator.validate_basic(&input);
        let partial_result = validator.validate_partial(&input);
        let strict_result = validator.validate(&input); // validate() is Strict mode
        
        // Count errors for each mode
        let basic_errors = basic_result.err().map(|e| e.to_string().len()).unwrap_or(0);
        let partial_errors = partial_result.err().map(|e| e.to_string().len()).unwrap_or(0);
        let strict_errors = strict_result.err().map(|e| e.to_string().len()).unwrap_or(0);
        
        // Stricter modes should have at least as many validation checks
        // (approximated by error message length)
        assert!(strict_errors >= partial_errors || (strict_errors == 0 && partial_errors == 0));
        assert!(partial_errors >= basic_errors || (partial_errors == 0 && basic_errors == 0));
    }
    
    /// Property: Valid minimal PromptSpec should always pass Basic validation
    #[test]
    fn prop_minimal_valid_prompt_spec_passes_basic(
        id in "[a-zA-Z0-9-]{5,20}",
        content in "[a-zA-Z0-9 .,!?]{10,100}",
    ) {
        let spec = json!({
            "spec_version": "1.0",
            "id": id,
            "model_class": "Chat",
            "messages": [
                {"role": "user", "content": content}
            ],
            "strict_mode": "Warn"
        });
        
        let validator = create_prompt_spec_validator().expect("validator creation should succeed");
        let result = validator.validate_basic(&spec);
        
        // Minimal valid spec should pass basic validation
        assert!(result.is_ok(), "Minimal valid spec failed: {:?}", result);
    }
    
    /// Property: Invalid spec_version should always fail
    #[test]
    fn prop_invalid_spec_version_fails(
        invalid_version in "[a-zA-Z0-9.]{1,20}".prop_filter("Not a standard version", |v| {
            v != "1.0" && v != "2.0" && v != "1.0.0" && v != "2.0.0"
        })
    ) {
        let spec = json!({
            "spec_version": invalid_version,
            "id": "test-123",
            "model_class": "Chat",
            "messages": [
                {"role": "user", "content": "test"}
            ],
            "strict_mode": "Warn"
        });
        
        let validator = create_prompt_spec_validator().expect("validator creation should succeed");
        let result = validator.validate(&spec);
        
        // Invalid spec version should fail validation
        assert!(result.is_err());
    }
    
    /// Property: Missing required fields should always fail
    #[test]
    fn prop_missing_required_fields_fail(
        include_version in any::<bool>(),
        include_id in any::<bool>(),
        include_model_class in any::<bool>(),
        include_messages in any::<bool>(),
        include_strict_mode in any::<bool>(),
    ) {
        let mut spec = json!({});
        
        if include_version {
            spec["spec_version"] = json!("1.0");
        }
        if include_id {
            spec["id"] = json!("test-123");
        }
        if include_model_class {
            spec["model_class"] = json!("Chat");
        }
        if include_messages {
            spec["messages"] = json!([{"role": "user", "content": "test"}]);
        }
        if include_strict_mode {
            spec["strict_mode"] = json!("Warn");
        }
        
        // Only valid if all fields are present
        let should_be_valid = include_version && include_id && include_model_class && include_messages && include_strict_mode;
        
        let validator = create_prompt_spec_validator().expect("validator creation should succeed");
        let result = validator.validate_basic(&spec);
        
        if should_be_valid {
            assert!(result.is_ok(), "Valid spec failed: {:?}", result);
        } else {
            assert!(result.is_err(), "Invalid spec passed: {:?}", spec);
        }
    }
    
    /// Property: Empty messages array should fail validation
    #[test]
    fn prop_empty_messages_fail(
        id in "[a-zA-Z0-9-]{5,20}",
    ) {
        let spec = json!({
            "spec_version": "1.0",
            "id": id,
            "model_class": "Chat",
            "messages": [],
            "strict_mode": "Warn"
        });
        
        let validator = create_prompt_spec_validator().expect("validator creation should succeed");
        let result = validator.validate(&spec);
        
        // Empty messages should fail
        assert!(result.is_err());
    }
    
    /// Property: Invalid message roles should fail
    #[test]
    fn prop_invalid_message_role_fails(
        invalid_role in "[a-zA-Z0-9_]{1,20}".prop_filter("Not a valid role", |r| {
            r != "system" && r != "user" && r != "assistant"
        })
    ) {
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [
                {"role": invalid_role, "content": "test"}
            ],
            "strict_mode": "Warn"
        });
        
        let validator = create_prompt_spec_validator().expect("validator creation should succeed");
        let result = validator.validate(&spec);
        
        // Invalid role should fail
        assert!(result.is_err());
    }
    
    /// Property: Validation should be deterministic
    #[test]
    fn prop_validation_deterministic(
        input in json_value_strategy()
    ) {
        let validator = create_prompt_spec_validator().expect("validator creation should succeed");
        
        // Run validation multiple times
        let result1 = validator.validate(&input);
        let result2 = validator.validate(&input);
        let result3 = validator.validate(&input);
        
        // All results should be the same
        match (result1, result2, result3) {
            (Ok(_), Ok(_), Ok(_)) => {},
            (Err(e1), Err(e2), Err(e3)) => {
                // Error messages should be consistent
                assert_eq!(e1.to_string(), e2.to_string());
                assert_eq!(e2.to_string(), e3.to_string());
            },
            _ => panic!("Non-deterministic validation results"),
        }
    }
    
    /// Property: Provider spec with valid minimal fields should pass basic validation
    #[test]
    fn prop_minimal_provider_spec_passes_basic(
        provider_name in "[a-zA-Z0-9-]{3,20}",
    ) {
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": provider_name,
                "base_url": "https://api.example.com"
            },
            "models": []
        });
        
        let validator = create_provider_spec_validator().expect("validator creation should succeed");
        let result = validator.validate_basic(&spec);
        
        // Minimal valid provider spec should pass basic validation
        assert!(result.is_ok(), "Minimal provider spec failed: {:?}", result);
    }
}