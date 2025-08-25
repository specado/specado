//! Tests for TranslationResult builder
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use super::*;
use crate::{LossinessCode, LossinessItem, LossinessReport, LossinessSummary, 
           Severity, StrictMode, TranslationMetadata};
use std::collections::HashMap;

#[test]
fn test_builder_basic() {
    let request = serde_json::json!({
        "model": "test-model",
        "messages": []
    });

    let result = TranslationResultBuilder::new()
        .with_provider_request(request.clone())
        .build()
        .expect("Basic build should succeed");

    assert_eq!(result.provider_request_json, request);
    assert!(!result.has_lossiness());
    assert!(result.metadata.is_none());
}

#[test]
fn test_builder_with_metadata() {
    let metadata = TranslationMetadata {
        provider: "test-provider".to_string(),
        model: "test-model".to_string(),
        timestamp: "2025-01-01T00:00:00Z".to_string(),
        duration_ms: Some(100),
        strict_mode: StrictMode::Warn,
    };

    let result = TranslationResultBuilder::new()
        .with_provider_request(serde_json::json!({}))
        .with_metadata(metadata.clone())
        .build()
        .expect("Build with metadata should succeed");

    assert!(result.metadata.is_some());
    assert_eq!(result.provider_name(), Some("test-provider"));
    assert_eq!(result.model_name(), Some("test-model"));
    assert_eq!(result.duration_ms(), Some(100));
}

#[test]
fn test_builder_with_lossiness() {
    let lossiness = LossinessReport {
        items: vec![LossinessItem {
            code: LossinessCode::Drop,
            path: "test_field".to_string(),
            message: "Field dropped".to_string(),
            severity: Severity::Warning,
            before: Some(serde_json::json!("value")),
            after: None,
        }],
        max_severity: Severity::Warning,
        summary: LossinessSummary {
            total_items: 1,
            by_severity: {
                let mut map = HashMap::new();
                map.insert("warning".to_string(), 1);
                map
            },
            by_code: {
                let mut map = HashMap::new();
                map.insert("Drop".to_string(), 1);
                map
            },
        },
    };

    let result = TranslationResultBuilder::new()
        .with_provider_request(serde_json::json!({}))
        .with_lossiness_report(lossiness)
        .build()
        .expect("Build with lossiness should succeed");

    assert!(result.has_lossiness());
    assert!(result.has_warnings());
    assert!(!result.has_errors());
    assert!(!result.has_critical_issues());
}

#[test]
fn test_success_helper() {
    let request = serde_json::json!({
        "model": "test-model",
        "messages": []
    });

    let result = TranslationResultBuilder::success(request.clone());

    assert_eq!(result.provider_request_json, request);
    assert!(!result.has_lossiness());
    assert_eq!(result.lossiness.max_severity, Severity::Info);
}

#[test]
fn test_builder_completeness() {
    let builder = TranslationResultBuilder::new();
    assert!(!builder.is_complete());

    let builder = builder.with_provider_request(serde_json::json!({}));
    assert!(builder.is_complete());
    assert_eq!(builder.state(), &BuilderState::Ready);
}

#[test]
fn test_builder_getters() {
    let request = serde_json::json!({"test": "value"});
    let metadata = TranslationMetadata {
        provider: "test".to_string(),
        model: "model".to_string(),
        timestamp: "now".to_string(),
        duration_ms: None,
        strict_mode: StrictMode::Strict,
    };

    let builder = TranslationResultBuilder::new()
        .with_provider_request(request.clone())
        .with_metadata(metadata.clone());

    assert_eq!(builder.provider_request(), Some(&request));
    assert_eq!(builder.metadata(), Some(&metadata));
    assert_eq!(builder.lossiness(), None);
}

#[test]
fn test_result_severity_checks() {
    // Test critical severity
    let mut result = TranslationResultBuilder::success(serde_json::json!({}));
    result.lossiness.max_severity = Severity::Critical;
    assert!(result.has_critical_issues());
    assert!(result.has_errors());
    assert!(result.has_warnings());

    // Test error severity
    result.lossiness.max_severity = Severity::Error;
    assert!(!result.has_critical_issues());
    assert!(result.has_errors());
    assert!(result.has_warnings());

    // Test warning severity
    result.lossiness.max_severity = Severity::Warning;
    assert!(!result.has_critical_issues());
    assert!(!result.has_errors());
    assert!(result.has_warnings());

    // Test info severity
    result.lossiness.max_severity = Severity::Info;
    assert!(!result.has_critical_issues());
    assert!(!result.has_errors());
    assert!(!result.has_warnings());
}

// New comprehensive tests for Issue #21 features

#[test]
fn test_builder_state_tracking() {
    let mut builder = TranslationResultBuilder::new();
    assert_eq!(builder.state(), &BuilderState::Incomplete);

    builder = builder.with_provider_request(serde_json::json!({"model": "test"}));
    assert_eq!(builder.state(), &BuilderState::Ready);

    let _result = builder.build().expect("Should build successfully");
    // builder is consumed, so we can't check state after build()
}

#[test]
fn test_validation() {
    let incomplete_builder = TranslationResultBuilder::new();
    assert!(incomplete_builder.validate().is_err());

    let complete_builder = TranslationResultBuilder::new()
        .with_provider_request(serde_json::json!({}));
    assert!(complete_builder.validate().is_ok());
}

#[test]
fn test_merge_builders() {
    let builder1 = TranslationResultBuilder::new()
        .with_metadata(TranslationMetadata {
            provider: "provider1".to_string(),
            model: "model1".to_string(),
            timestamp: "now".to_string(),
            duration_ms: None,
            strict_mode: StrictMode::Warn,
        });

    let builder2 = TranslationResultBuilder::new()
        .with_provider_request(serde_json::json!({"model": "test"}));

    let merged = builder1.merge(builder2).expect("Merge should succeed");
    assert!(merged.is_complete());
    assert!(merged.metadata().is_some());
    assert!(merged.provider_request().is_some());
}

#[test]
fn test_incremental_lossiness() {
    let result = TranslationResultBuilder::new()
        .with_provider_request(serde_json::json!({}))
        .with_provider_info("test-provider", "test-model", StrictMode::Warn)
        .add_dropped_field("tools", "Tools not supported", Some(serde_json::json!([])))
        .add_clamped_value("temperature", "Temperature clamped", 
            serde_json::json!(2.5), serde_json::json!(2.0))
        .build()
        .expect("Build with lossiness should succeed");

    assert!(result.has_lossiness());
    assert_eq!(result.lossiness.items.len(), 2);
}

#[test]
fn test_from_context() {
    let context = create_test_context();
    let builder = TranslationResultBuilder::from_context(&context);
    
    assert!(builder.metadata().is_some());
    assert_eq!(builder.metadata().unwrap().provider, "test-provider");
    assert!(builder.start_time.is_some());
    assert!(builder.lossiness_tracker().is_some());
}

#[test]
fn test_incremental_request_builder() {
    let result = TranslationResultBuilder::new()
        .with_provider_request_incremental()
        .set_model("test-model")
        .add_message(serde_json::json!({
            "role": "user",
            "content": "Hello"
        }))
        .set_temperature(0.7)
        .set_max_tokens(100)
        .done()
        .build()
        .expect("Incremental build should succeed");

    let request = &result.provider_request_json;
    assert_eq!(request["model"], "test-model");
    assert_eq!(request["temperature"], 0.7);
    assert_eq!(request["max_tokens"], 100);
    assert!(request["messages"].is_array());
}

#[test]
fn test_timing_features() {
    let builder = TranslationResultBuilder::with_timing();
    assert!(builder.start_time.is_some());
    assert!(builder.elapsed_time().is_some());

    // Test automatic duration calculation
    let result = builder
        .with_provider_request(serde_json::json!({}))
        .with_provider_info("test", "model", StrictMode::Warn)
        .build()
        .expect("Build with timing should succeed");

    assert!(result.metadata.is_some());
    assert!(result.duration_ms().is_some());
}

#[test]
fn test_success_from_context() {
    let context = create_test_context();
    let request = serde_json::json!({"model": "test"});
    
    let result = TranslationResultBuilder::success_from_context(&context, request.clone());
    
    assert_eq!(result.provider_request_json, request);
    assert!(!result.has_lossiness());
    assert!(result.metadata.is_some());
    assert_eq!(result.provider_name(), Some("test-provider"));
}

#[test]
fn test_critical_error_builder() {
    let result = TranslationResultBuilder::new()
        .with_provider_request(serde_json::json!({}))
        .with_critical_error("tools", "Tools not supported", Some(serde_json::json!([])))
        .build()
        .expect("Build with critical error should succeed");

    assert!(result.has_critical_issues());
    assert_eq!(result.lossiness.max_severity, Severity::Critical);
}

#[test]
fn test_try_build_for_error_recovery() {
    let builder = TranslationResultBuilder::new()
        .with_provider_request(serde_json::json!({}));

    // try_build doesn't consume the builder
    let result = builder.try_build().expect("try_build should succeed");
    assert_eq!(result.provider_request_json, serde_json::json!({}));

    // We can still use the builder after try_build
    let final_result = builder.build().expect("Final build should succeed");
    assert_eq!(final_result.provider_request_json, serde_json::json!({}));
}

#[test]
fn test_merge_lossiness_trackers() {
    let builder1 = TranslationResultBuilder::new()
        .with_provider_info("test", "model", StrictMode::Warn)
        .add_dropped_field("field1", "Dropped", None);

    let builder2 = TranslationResultBuilder::new()
        .with_provider_info("test", "model", StrictMode::Warn)
        .add_dropped_field("field2", "Also dropped", None);

    let merged = builder1.merge(builder2).expect("Merge should succeed");
    let result = merged
        .with_provider_request(serde_json::json!({}))
        .build()
        .expect("Build merged should succeed");

    // Should have both lossiness items
    assert_eq!(result.lossiness.items.len(), 2);
}

#[test]
fn test_error_handling() {
    // Test building incomplete builder
    let incomplete = TranslationResultBuilder::new();
    assert!(incomplete.build().is_err());

    // Test merging with built builder would panic, so we test the error case differently
    let builder1 = TranslationResultBuilder::new();
    let builder2 = TranslationResultBuilder::new();
    
    // This should work fine
    let _merged = builder1.merge(builder2);
}

// Helper function for tests
fn create_test_context() -> crate::translation::TranslationContext {
    use crate::{
        Constraints, ConstraintLimits, EndpointConfig, Endpoints, InputModes, JsonOutputConfig,
        Mappings, Message, MessageRole, ProviderInfo, ProviderSpec, PromptSpec, 
        ResponseNormalization, StreamNormalization, SyncNormalization, ToolingConfig, ModelSpec,
    };
    use std::collections::HashMap;

    let prompt_spec = PromptSpec {
        model_class: "Chat".to_string(),
        messages: vec![Message {
            role: MessageRole::User,
            content: "Test".to_string(),
            name: None,
            metadata: None,
        }],
        tools: None,
        tool_choice: None,
        response_format: None,
        sampling: None,
        limits: None,
        media: None,
        advanced: None,
        strict_mode: StrictMode::Warn,
    };

    let provider_spec = ProviderSpec {
        spec_version: "1.0.0".to_string(),
        provider: ProviderInfo {
            name: "test-provider".to_string(),
            base_url: "https://api.test.com".to_string(),
            headers: HashMap::new(),
        },
        models: vec![],
    };

    let model_spec = ModelSpec {
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
            images: true,
        },
        tooling: ToolingConfig {
            tools_supported: true,
            parallel_tool_calls_default: false,
            can_disable_parallel_tool_calls: false,
            disable_switch: None,
        },
        json_output: JsonOutputConfig {
            native_param: true,
            strategy: "native".to_string(),
        },
        capabilities: None,
        parameters: serde_json::json!({}),
        constraints: Constraints {
            system_prompt_location: "first".to_string(),
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
                content_path: "content".to_string(),
                finish_reason_path: "finish".to_string(),
                finish_reason_map: HashMap::new(),
            },
            stream: StreamNormalization {
                protocol: "sse".to_string(),
                event_selector: crate::EventSelector {
                    type_path: "type".to_string(),
                    routes: vec![],
                },
            },
        },
    };

    crate::translation::TranslationContext::new(prompt_spec, provider_spec, model_spec, StrictMode::Warn)
}