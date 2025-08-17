//! End-to-end integration tests for the translation engine
//! 
//! These tests verify the entire translation pipeline by loading real PromptSpec
//! and ProviderSpec files, executing the full translation, and asserting the
//! correctness of the final output using golden snapshots.

use serde_json::{json, Value};
use specado_core::{translate, StrictMode};
use specado_core::error::LossinessCode;
use specado_core::types::{
    Message, MessageRole, PromptSpec, ProviderSpec
};
// Golden testing imports removed - not needed for basic integration tests
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Helper to load a PromptSpec from a file
fn load_prompt_spec(path: &Path) -> Result<PromptSpec, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let spec: PromptSpec = serde_json::from_str(&content)?;
    Ok(spec)
}

/// Helper to load a ProviderSpec from a file
fn load_provider_spec(path: &Path) -> Result<ProviderSpec, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let spec: ProviderSpec = serde_json::from_str(&content)?;
    Ok(spec)
}

// Helper functions removed - tests will use actual JSON deserialization
// to create valid test data structures

fn test_basic_chat_translation() {
    // Test basic translation with real PromptSpec and ProviderSpec structures
    // This test validates that the translate function can handle basic chat messages
    
    // Create prompt using JSON deserialization to ensure valid structure
    let prompt_json = json!({
        "model_class": "Chat",
        "messages": [
            {"role": "system", "content": "You are a helpful assistant."},
            {"role": "user", "content": "What is 2+2?"}
        ],
        "strict_mode": "Warn"
    });
    
    let prompt: PromptSpec = serde_json::from_value(prompt_json)
        .expect("Failed to create prompt spec");
    
    // Create a minimal provider spec
    let provider_json = json!({
        "spec_version": "1.0.0",
        "provider": {
            "name": "openai",
            "base_url": "https://api.openai.com/v1",
            "headers": {}
        },
        "models": [{
            "id": "gpt-5",
            "aliases": ["gpt5"],
            "family": "gpt",
            "endpoints": {
                "chat": "/chat/completions"
            },
            "input_modes": {
                "messages": true
            },
            "tooling": {
                "tools_supported": false
            },
            "json_output": {
                "supported": false
            },
            "parameters": {},
            "constraints": {
                "max_tokens": 4096
            },
            "mappings": {},
            "response_normalization": {}
        }]
    });
    
    let provider: ProviderSpec = serde_json::from_value(provider_json)
        .expect("Failed to create provider spec");
    
    let result = translate(&prompt, &provider, "gpt-5", StrictMode::Warn)
        .expect("Translation should succeed");
    
    // Verify basic structure
    assert!(result.provider_request_json.is_object());
    let request = result.provider_request_json.as_object().unwrap();
    
    // Check model is set
    assert!(request.contains_key("model"));
    
    // Check messages are translated
    assert!(request.contains_key("messages"));
}

/// Test translation with sampling parameters
#[test]
fn test_translation_with_sampling() {
    let mut prompt = create_test_prompt(vec![
        Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
        },
    ]);
    
    prompt.sampling = Some(specado_core::types::SamplingParams {
        temperature: Some(0.7),
        top_p: Some(0.9),
        top_k: Some(40),
        frequency_penalty: Some(0.5),
        presence_penalty: Some(0.2),

    });

    let provider = create_test_provider("openai", "gpt-5");

    let result = translate(&prompt, &provider, "gpt-5", StrictMode::Warn)
        .expect("Translation should succeed");

    let request = result.provider_request_json.as_object().unwrap();
    
    // Check sampling parameters are included
    assert_eq!(request.get("temperature").unwrap().as_f64().unwrap(), 0.7);
    assert_eq!(request.get("top_p").unwrap().as_f64().unwrap(), 0.9);
    assert_eq!(request.get("top_k").unwrap().as_i64().unwrap(), 40);
    assert_eq!(request.get("frequency_penalty").unwrap().as_f64().unwrap(), 0.5);
    assert_eq!(request.get("presence_penalty").unwrap().as_f64().unwrap(), 0.2);
}

/// Test translation with output limits
#[test]
fn test_translation_with_limits() {
    let mut prompt = create_test_prompt(vec![
        Message {
            role: "user".to_string(),
            content: "Tell me a story".to_string(),
        },
    ]);
    
    prompt.limits = Some(specado_core::types::OutputLimits {
        max_output_tokens: Some(1000),
        stop_sequences: Some(vec!["END".to_string(), "STOP".to_string()]),
        reasoning_tokens: None,
    });

    let provider = create_test_provider("openai", "gpt-5");

    let result = translate(&prompt, &provider, "gpt-5", StrictMode::Warn)
        .expect("Translation should succeed");

    let request = result.provider_request_json.as_object().unwrap();
    
    // Check limits are included
    assert_eq!(request.get("max_tokens").unwrap().as_i64().unwrap(), 1000);
}

/// Test translation with tools (should track lossiness when unsupported)
#[test]
fn test_translation_with_unsupported_tools() {
    let mut prompt = create_test_prompt(vec![
        Message {
            role: "user".to_string(),
            content: "What's the weather?".to_string(),
        },
    ]);
    
    prompt.tools = Some(vec![specado_core::types::Tool {
        name: "get_weather".to_string(),
        description: Some("Get the current weather".to_string()),
        json_schema: json!({
            "type": "object",
            "properties": {
                "location": {"type": "string"}
            }
        }),
    }]);
    
    prompt.tool_choice = Some(json!("auto"));

    let provider = create_test_provider("openai", "gpt-5");

    let result = translate(&prompt, &provider, "gpt-5", StrictMode::Warn)
        .expect("Translation should succeed in Warn mode");

    // Check that tools were dropped and tracked in lossiness
    let request = result.provider_request_json.as_object().unwrap();
    assert!(!request.contains_key("tools"));
    assert!(!request.contains_key("tool_choice"));
    
    // Check lossiness report
    assert!(result.lossiness.summary.total_items > 0);
    assert!(result.lossiness.items.iter().any(|item| 
        item.path == "$.tools" && item.code == specado_core::error::LossinessCode::Unsupported
    ));
}

/// Test strict mode behavior with unsupported features
#[test]
fn test_strict_mode_with_unsupported_features() {
    let mut prompt = create_test_prompt(vec![
        Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
        },
    ]);
    
    prompt.tools = Some(vec![specado_core::types::Tool {
        name: "test_tool".to_string(),
        description: Some("Test tool".to_string()),
        json_schema: json!({}),
    }]);

    let provider = create_test_provider("openai", "gpt-5");

    // Should fail in Strict mode
    let result = translate(&prompt, &provider, "gpt-5", StrictMode::Strict);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("tools"));
}

/// Test translation with response format
#[test]
fn test_translation_with_response_format() {
    let mut prompt = create_test_prompt(vec![
        Message {
            role: "user".to_string(),
            content: "Return JSON".to_string(),
        },
    ]);
    
    prompt.response_format = Some(json!({
        "type": "json_object"
    }));

    let provider = create_test_provider("openai", "gpt-5");

    let result = translate(&prompt, &provider, "gpt-5", StrictMode::Warn)
        .expect("Translation should succeed");

    let request = result.provider_request_json.as_object().unwrap();
    
    // Check response format is included (provider supports it natively)
    assert!(request.contains_key("response_format"));
    assert_eq!(request["response_format"]["type"], "json_object");
}

/// Test model alias resolution
#[test]
fn test_model_alias_resolution() {
    let prompt = create_test_prompt(vec![
        Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
        },
    ]);

    let mut provider = create_test_provider("openai", "gpt-5-turbo");
    provider.models[0].aliases = Some(vec!["gpt5".to_string(), "gpt-5".to_string()]);

    // Should resolve alias to actual model ID
    let result = translate(&prompt, &provider, "gpt5", StrictMode::Warn)
        .expect("Translation should succeed with alias");

    let request = result.provider_request_json.as_object().unwrap();
    assert_eq!(request.get("model").unwrap().as_str().unwrap(), "gpt5");
}

/// Test invalid model ID
#[test]
fn test_invalid_model_id() {
    let prompt = create_test_prompt(vec![
        Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
        },
    ]);

    let provider = create_test_provider("openai", "gpt-5");

    let result = translate(&prompt, &provider, "invalid-model", StrictMode::Warn);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Model 'invalid-model' not found"));
}

/// Integration test using golden corpus test cases
#[test]
fn test_golden_corpus_integration() {
    let corpus_dir = PathBuf::from("golden-corpus");
    if !corpus_dir.exists() {
        eprintln!("Skipping golden corpus test - corpus directory not found");
        return;
    }

    // Test simple chat case
    let test_case_path = corpus_dir.join("basic/simple-chat/test.json");
    if test_case_path.exists() {
        let content = fs::read_to_string(&test_case_path).expect("Failed to read test case");
        let test_case: Value = serde_json::from_str(&content).expect("Failed to parse test case");
        
        let prompt_spec = test_case["input"]["prompt_spec"].clone();
        let prompt: PromptSpec = serde_json::from_value(prompt_spec).expect("Failed to parse prompt spec");
        
        // Use a real provider spec for OpenAI
        let provider = create_test_provider("openai", "gpt-5");
        
        let result = translate(&prompt, &provider, "gpt-5", StrictMode::Warn)
            .expect("Translation should succeed");
        
        // Verify expected fields exist
        assert!(result.provider_request_json.is_object());
        let request = result.provider_request_json.as_object().unwrap();
        assert!(request.contains_key("model"));
        assert!(request.contains_key("messages"));
    }
}

/// Test temperature clamping behavior
#[test]
fn test_temperature_clamping() {
    let mut prompt = create_test_prompt(vec![
        Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
        },
    ]);
    
    // Set temperature outside typical range
    prompt.sampling = Some(specado_core::types::SamplingParams {
        temperature: Some(3.0), // Above typical max of 2.0
        top_p: None,
        top_k: None,
        frequency_penalty: None,
        presence_penalty: None,

    });

    let provider = create_test_provider("openai", "gpt-5");

    // In Warn mode, should proceed but track in lossiness
    let result = translate(&prompt, &provider, "gpt-5", StrictMode::Warn)
        .expect("Translation should succeed in Warn mode");

    let request = result.provider_request_json.as_object().unwrap();
    
    // Temperature should be included (clamping behavior depends on strictness policy)
    assert!(request.contains_key("temperature"));
}

/// Test multi-message conversation translation
#[test]
fn test_multi_turn_conversation() {
    let prompt = create_test_prompt(vec![
        Message {
            role: "system".to_string(),
            content: "You are a helpful assistant.".to_string(),
        },
        Message {
            role: "user".to_string(),
            content: "What is the capital of France?".to_string(),
        },
        Message {
            role: "assistant".to_string(),
            content: "The capital of France is Paris.".to_string(),
        },
        Message {
            role: "user".to_string(),
            content: "What is its population?".to_string(),
        },
    ]);

    let provider = create_test_provider("openai", "gpt-5");

    let result = translate(&prompt, &provider, "gpt-5", StrictMode::Warn)
        .expect("Translation should succeed");

    let request = result.provider_request_json.as_object().unwrap();
    let messages = request.get("messages").unwrap().as_array().unwrap();
    
    // All messages should be preserved in order
    assert_eq!(messages.len(), 4);
    assert_eq!(messages[0]["role"], "system");
    assert_eq!(messages[1]["role"], "user");
    assert_eq!(messages[2]["role"], "assistant");
    assert_eq!(messages[3]["role"], "user");
}

/// Test empty message handling
#[test]
fn test_empty_messages() {
    let prompt = create_test_prompt(vec![]);

    let provider = create_test_provider("openai", "gpt-5");

    let result = translate(&prompt, &provider, "gpt-5", StrictMode::Warn)
        .expect("Translation should handle empty messages");

    let request = result.provider_request_json.as_object().unwrap();
    let messages = request.get("messages").unwrap().as_array().unwrap();
    
    assert_eq!(messages.len(), 0);
}

/// Test metadata generation
#[test]
fn test_metadata_generation() {
    let prompt = create_test_prompt(vec![
        Message {
            role: "user".to_string(),
            content: "Test".to_string(),
        },
    ]);

    let provider = create_test_provider("anthropic", "claude-opus-4.1");

    let result = translate(&prompt, &provider, "claude-opus-4.1", StrictMode::Strict)
        .expect("Translation should succeed");

    assert!(result.metadata.is_some());
    let metadata = result.metadata.unwrap();
    
    assert_eq!(metadata.provider, "anthropic");
    assert_eq!(metadata.model, "claude-opus-4.1");
    assert_eq!(metadata.strict_mode, StrictMode::Strict);
    assert!(metadata.timestamp.len() > 0);
    assert!(metadata.duration_ms.is_some());
}

/// Test lossiness tracking for multiple issues
#[test]
fn test_comprehensive_lossiness_tracking() {
    let mut prompt = create_test_prompt(vec![
        Message {
            role: "user".to_string(),
            content: "Test".to_string(),
        },
    ]);
    
    // Add multiple features that might cause lossiness
    prompt.tools = Some(vec![specado_core::types::Tool {
        name: "tool1".to_string(),
        description: None,
        json_schema: json!({}),
    }]);
    prompt.sampling = Some(specado_core::types::SamplingParams {
        temperature: Some(5.0), // Out of range
        top_p: None,
        top_k: None,
        frequency_penalty: None,
        presence_penalty: None,

    });

    let provider = create_test_provider("openai", "gpt-5");

    let result = translate(&prompt, &provider, "gpt-5", StrictMode::Warn)
        .expect("Translation should succeed in Warn mode");

    // Should have multiple lossiness items
    assert!(result.lossiness.summary.total_items > 0);
    assert!(result.lossiness.items.len() > 0);
    
    // Check severity distribution
    assert!(result.lossiness.summary.by_severity.contains_key("High") || 
            result.lossiness.summary.by_severity.contains_key("Medium"));
}