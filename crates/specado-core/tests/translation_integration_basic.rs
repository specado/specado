//! Basic end-to-end integration tests for the translation engine
//! 
//! These tests verify the core translation pipeline functionality.

use serde_json::{json, Value};
use specado_core::{translate, StrictMode};
use specado_core::types::{PromptSpec, ProviderSpec};

#[test]
fn test_basic_chat_translation() {
    // Create a simple prompt spec using JSON
    let prompt_json = json!({
        "model_class": "Chat",
        "messages": [
            {"role": "system", "content": "You are a helpful assistant."},
            {"role": "user", "content": "What is 2+2?"}
        ],
        "strict_mode": "Warn"
    });
    
    let prompt: PromptSpec = serde_json::from_value(prompt_json)
        .expect("Failed to parse prompt spec");
    
    // Load the actual OpenAI provider spec from golden corpus if available
    let provider_path = std::path::PathBuf::from("golden-corpus/providers/openai/openai-provider.json");
    let provider: ProviderSpec = if provider_path.exists() {
        let content = std::fs::read_to_string(provider_path)
            .expect("Failed to read provider spec");
        serde_json::from_str(&content)
            .expect("Failed to parse provider spec")
    } else {
        // Fallback to a minimal spec if golden corpus not available
        // This won't work due to missing required fields, but allows test compilation
        eprintln!("Warning: Golden corpus not found, test will be skipped");
        return;
    };
    
    // Perform translation
    let result = translate(&prompt, &provider, "gpt-5", StrictMode::Warn)
        .expect("Translation should succeed");
    
    // Basic assertions
    assert!(result.provider_request_json.is_object());
    let request = result.provider_request_json.as_object().unwrap();
    assert!(request.contains_key("model"));
    assert_eq!(request["model"], "gpt-5");
    assert!(request.contains_key("messages"));
    
    let messages = request["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2);
}

/// Test translation with sampling parameters
#[test]
fn test_translation_with_sampling() {
    let prompt_json = json!({
        "model_class": "Chat",
        "messages": [
            {"role": "user", "content": "Hello"}
        ],
        "sampling": {
            "temperature": 0.7,
            "top_p": 0.9,
            "top_k": 40,
            "frequency_penalty": 0.5,
            "presence_penalty": 0.2
        },
        "strict_mode": "Warn"
    });
    
    let prompt: PromptSpec = serde_json::from_value(prompt_json)
        .expect("Failed to parse prompt spec");
    
    let provider_json = json!({
        "spec_version": "1.0.0",
        "provider": {
            "name": "openai",
            "base_url": "https://api.openai.com/v1",
            "headers": {}
        },
        "models": [{
            "id": "gpt-5",
            "family": "gpt",
            "endpoints": {"chat": "/chat/completions"},
            "input_modes": {"messages": true},
            "tooling": {"tools_supported": false},
            "json_output": {"supported": false},
            "parameters": {},
            "constraints": {"max_tokens": 4096},
            "mappings": {},
            "response_normalization": {}
        }]
    });
    
    let provider: ProviderSpec = serde_json::from_value(provider_json)
        .expect("Failed to parse provider spec");
    
    let result = translate(&prompt, &provider, "gpt-5", StrictMode::Warn)
        .expect("Translation should succeed");
    
    let request = result.provider_request_json.as_object().unwrap();
    
    // Verify sampling parameters are present
    assert!(request.contains_key("temperature"));
    assert!(request.contains_key("top_p"));
}

/// Test translation with output limits
#[test]
fn test_translation_with_limits() {
    let prompt_json = json!({
        "model_class": "Chat",
        "messages": [
            {"role": "user", "content": "Tell me a story"}
        ],
        "limits": {
            "max_output_tokens": 1000,
            "stop_sequences": ["END", "STOP"]
        },
        "strict_mode": "Warn"
    });
    
    let prompt: PromptSpec = serde_json::from_value(prompt_json)
        .expect("Failed to parse prompt spec");
    
    let provider_json = json!({
        "spec_version": "1.0.0",
        "provider": {
            "name": "openai",
            "base_url": "https://api.openai.com/v1",
            "headers": {}
        },
        "models": [{
            "id": "gpt-5",
            "family": "gpt",
            "endpoints": {"chat": "/chat/completions"},
            "input_modes": {"messages": true},
            "tooling": {"tools_supported": false},
            "json_output": {"supported": false},
            "parameters": {},
            "constraints": {"max_tokens": 4096},
            "mappings": {},
            "response_normalization": {}
        }]
    });
    
    let provider: ProviderSpec = serde_json::from_value(provider_json)
        .expect("Failed to parse provider spec");
    
    let result = translate(&prompt, &provider, "gpt-5", StrictMode::Warn)
        .expect("Translation should succeed");
    
    let request = result.provider_request_json.as_object().unwrap();
    
    // Verify max_tokens is set
    assert!(request.contains_key("max_tokens"));
}

/// Test invalid model ID error
#[test]
fn test_invalid_model_id() {
    let prompt_json = json!({
        "model_class": "Chat",
        "messages": [
            {"role": "user", "content": "Hello"}
        ],
        "strict_mode": "Warn"
    });
    
    let prompt: PromptSpec = serde_json::from_value(prompt_json)
        .expect("Failed to parse prompt spec");
    
    let provider_json = json!({
        "spec_version": "1.0.0",
        "provider": {
            "name": "openai",
            "base_url": "https://api.openai.com/v1",
            "headers": {}
        },
        "models": [{
            "id": "gpt-5",
            "family": "gpt",
            "endpoints": {"chat": "/chat/completions"},
            "input_modes": {"messages": true},
            "tooling": {"tools_supported": false},
            "json_output": {"supported": false},
            "parameters": {},
            "constraints": {"max_tokens": 4096},
            "mappings": {},
            "response_normalization": {}
        }]
    });
    
    let provider: ProviderSpec = serde_json::from_value(provider_json)
        .expect("Failed to parse provider spec");
    
    // Try to use an invalid model ID
    let result = translate(&prompt, &provider, "invalid-model", StrictMode::Warn);
    
    // Should return an error
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Model 'invalid-model' not found"));
}

/// Test model alias resolution
#[test]
fn test_model_alias_resolution() {
    let prompt_json = json!({
        "model_class": "Chat",
        "messages": [
            {"role": "user", "content": "Hello"}
        ],
        "strict_mode": "Warn"
    });
    
    let prompt: PromptSpec = serde_json::from_value(prompt_json)
        .expect("Failed to parse prompt spec");
    
    let provider_json = json!({
        "spec_version": "1.0.0",
        "provider": {
            "name": "openai",
            "base_url": "https://api.openai.com/v1",
            "headers": {}
        },
        "models": [{
            "id": "gpt-5-turbo",
            "aliases": ["gpt5", "gpt-5"],
            "family": "gpt",
            "endpoints": {"chat": "/chat/completions"},
            "input_modes": {"messages": true},
            "tooling": {"tools_supported": false},
            "json_output": {"supported": false},
            "parameters": {},
            "constraints": {"max_tokens": 4096},
            "mappings": {},
            "response_normalization": {}
        }]
    });
    
    let provider: ProviderSpec = serde_json::from_value(provider_json)
        .expect("Failed to parse provider spec");
    
    // Use an alias instead of the actual model ID
    let result = translate(&prompt, &provider, "gpt5", StrictMode::Warn)
        .expect("Translation should succeed with alias");
    
    assert!(result.provider_request_json.is_object());
}

/// Test multi-turn conversation
#[test]
fn test_multi_turn_conversation() {
    let prompt_json = json!({
        "model_class": "Chat",
        "messages": [
            {"role": "system", "content": "You are a helpful assistant."},
            {"role": "user", "content": "What is the capital of France?"},
            {"role": "assistant", "content": "The capital of France is Paris."},
            {"role": "user", "content": "What is its population?"}
        ],
        "strict_mode": "Warn"
    });
    
    let prompt: PromptSpec = serde_json::from_value(prompt_json)
        .expect("Failed to parse prompt spec");
    
    let provider_json = json!({
        "spec_version": "1.0.0",
        "provider": {
            "name": "openai",
            "base_url": "https://api.openai.com/v1",
            "headers": {}
        },
        "models": [{
            "id": "gpt-5",
            "family": "gpt",
            "endpoints": {"chat": "/chat/completions"},
            "input_modes": {"messages": true},
            "tooling": {"tools_supported": false},
            "json_output": {"supported": false},
            "parameters": {},
            "constraints": {"max_tokens": 4096},
            "mappings": {},
            "response_normalization": {}
        }]
    });
    
    let provider: ProviderSpec = serde_json::from_value(provider_json)
        .expect("Failed to parse provider spec");
    
    let result = translate(&prompt, &provider, "gpt-5", StrictMode::Warn)
        .expect("Translation should succeed");
    
    let request = result.provider_request_json.as_object().unwrap();
    let messages = request.get("messages").unwrap().as_array().unwrap();
    
    // All messages should be preserved
    assert_eq!(messages.len(), 4);
}

/// Test metadata generation
#[test]
fn test_metadata_generation() {
    let prompt_json = json!({
        "model_class": "Chat",
        "messages": [
            {"role": "user", "content": "Test"}
        ],
        "strict_mode": "Strict"
    });
    
    let prompt: PromptSpec = serde_json::from_value(prompt_json)
        .expect("Failed to parse prompt spec");
    
    let provider_json = json!({
        "spec_version": "1.0.0",
        "provider": {
            "name": "anthropic",
            "base_url": "https://api.anthropic.com/v1",
            "headers": {}
        },
        "models": [{
            "id": "claude-opus-4.1",
            "family": "claude",
            "endpoints": {"chat": "/messages"},
            "input_modes": {"messages": true},
            "tooling": {"tools_supported": false},
            "json_output": {"supported": false},
            "parameters": {},
            "constraints": {"max_tokens": 4096},
            "mappings": {},
            "response_normalization": {}
        }]
    });
    
    let provider: ProviderSpec = serde_json::from_value(provider_json)
        .expect("Failed to parse provider spec");
    
    let result = translate(&prompt, &provider, "claude-opus-4.1", StrictMode::Strict)
        .expect("Translation should succeed");
    
    // Check metadata exists
    assert!(result.metadata.is_some());
    let metadata = result.metadata.unwrap();
    assert_eq!(metadata.provider, "anthropic");
    assert_eq!(metadata.model, "claude-opus-4.1");
    assert_eq!(metadata.strict_mode, StrictMode::Strict);
    assert!(metadata.timestamp.len() > 0);
    assert!(metadata.duration_ms.is_some());
}