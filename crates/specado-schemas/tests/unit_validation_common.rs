//! Common unit tests for schema validation functionality
//!
//! This module tests general validation scenarios including required fields,
//! valid/invalid documents, error reporting quality, and edge cases that
//! apply to both PromptSpec and ProviderSpec schemas.

use serde_json::json;
use specado_schemas::{
    create_prompt_spec_validator, create_provider_spec_validator,
    SchemaValidator,
};

#[cfg(test)]
mod required_field_validation {
    use super::*;

    #[test]
    fn test_prompt_spec_missing_required_fields() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Missing messages (required field)
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Required field messages is missing"));
    }

    #[test]
    fn test_prompt_spec_with_all_required_fields() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // All required fields present
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [
                {"role": "user", "content": "Hello"}
            ],
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should be valid with all required fields: {:?}", result);
    }

    #[test]
    fn test_provider_spec_missing_required_fields() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Missing provider.base_url (required)
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider"
                // base_url missing
            },
            "models": []
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        // The error message will depend on schema structure
        assert!(error.to_string().contains("base_url") || 
                error.to_string().contains("required"));
    }

    #[test]
    fn test_provider_spec_with_all_required_fields() {
        let validator = create_provider_spec_validator().unwrap();
        
        // All required fields present
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "models": []
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should be valid with all required fields: {:?}", result);
    }
}

#[cfg(test)]
mod valid_document_tests {
    use super::*;

    #[test]
    fn test_comprehensive_valid_prompt_spec() {
        let validator = create_prompt_spec_validator().unwrap();
        
        let spec = json!({
            "spec_version": "1.0",
            "id": "comprehensive-test",
            "model_class": "Chat",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a helpful assistant."
                },
                {
                    "role": "user",
                    "content": "What is the weather like?",
                    "name": "user_123"
                },
                {
                    "role": "assistant",
                    "content": "I'd be happy to help with weather information.",
                    "metadata": {
                        "confidence": 0.95
                    }
                }
            ],
            "sampling": {
                "temperature": 0.7,
                "top_p": 0.9,
                "top_k": 50,
                "frequency_penalty": 0.1,
                "presence_penalty": 0.2
            },
            "limits": {
                "max_output_tokens": 2000,
                "max_prompt_tokens": 4000
            },
            "tools": [
                {
                    "name": "get_weather",
                    "description": "Get current weather for a location",
                    "json_schema": {
                        "type": "object",
                        "properties": {
                            "location": {"type": "string"},
                            "units": {"type": "string", "enum": ["celsius", "fahrenheit"]}
                        },
                        "required": ["location"]
                    }
                }
            ],
            "tool_choice": "auto",
            "response_format": {
                "type": "json_object"
            },
            "metadata": {
                "user_id": "user_123",
                "session_id": "session_456",
                "request_id": "req_789"
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Comprehensive valid spec should pass: {:?}", result);
    }

    #[test]
    fn test_comprehensive_valid_provider_spec() {
        let validator = create_provider_spec_validator().unwrap();
        
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "comprehensive-provider",
                "base_url": "https://api.example.com",
                "headers": {
                    "X-API-Version": "v1"
                },
                "query_params": {
                    "format": "json"
                }
            },
            "authentication": {
                "env_var": "${ENV:API_KEY}",
                "header_name": "Authorization",
                "header_format": "Bearer {key}"
            },
            "capabilities": {
                "supports_tools": true,
                "supports_streaming": true,
                "supports_conversation_persistence": true,
                "supports_rag": true,
                "supports_reasoning": false
            },
            "models": [
                {
                    "id": "chat-model-v1",
                    "aliases": ["chat", "default"],
                    "model_family": "chat",
                    "input_modes": ["text"],
                    "mappings": {
                        "paths": {
                            "messages": "$.messages",
                            "temperature": "$.parameters.temperature",
                            "max_tokens": "$.parameters.max_tokens"
                        },
                        "transformations": {
                            "temperature": {
                                "scale": [0.0, 2.0],
                                "default": 0.7
                            }
                        }
                    },
                    "endpoint": {
                        "protocol": "https",
                        "path": "/v1/chat/completions",
                        "method": "POST"
                    },
                    "response_normalization": {
                        "content": "$.choices[0].message.content",
                        "role": "$.choices[0].message.role",
                        "usage": "$.usage"
                    },
                    "constraints": {
                        "max_tokens": 4096,
                        "max_messages": 100,
                        "max_temperature": 2.0,
                        "min_temperature": 0.0
                    }
                }
            ],
            "tooling": {
                "tool_choice_modes": ["auto", "none", "required"],
                "max_tools": 128,
                "parallel_tool_calls": true
            },
            "rag_config": {
                "retrieval_strategies": ["semantic", "keyword", "hybrid"],
                "max_context_tokens": 8000,
                "chunk_size": 512
            },
            "conversation_management": {
                "max_history_tokens": 4000,
                "message_retention_policy": "sliding_window",
                "supports_branching": true
            },
            "rate_limits": [
                {
                    "tier": "free",
                    "requests_per_minute": 20,
                    "tokens_per_minute": 10000
                },
                {
                    "tier": "paid",
                    "requests_per_minute": 100,
                    "tokens_per_minute": 100000
                }
            ]
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Comprehensive valid spec should pass: {:?}", result);
    }

    #[test]
    fn test_reasoning_chat_prompt_spec() {
        let validator = create_prompt_spec_validator().unwrap();
        
        let spec = json!({
            "spec_version": "1.0",
            "id": "reasoning-test",
            "model_class": "ReasoningChat",
            "messages": [
                {
                    "role": "user",
                    "content": "Solve this complex problem..."
                }
            ],
            "limits": {
                "max_output_tokens": 2000,
                "reasoning_tokens": 5000  // Valid for ReasoningChat
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "ReasoningChat spec should be valid: {:?}", result);
    }

    #[test]
    fn test_rag_chat_prompt_spec() {
        let validator = create_prompt_spec_validator().unwrap();
        
        let spec = json!({
            "spec_version": "1.0",
            "id": "rag-test",
            "model_class": "RAGChat",
            "messages": [
                {
                    "role": "user",
                    "content": "Find information about..."
                }
            ],
            "rag": {
                "retrieval_strategy": "semantic",
                "top_k": 10,
                "context_window": 4000,
                "sources": ["knowledge_base_1", "knowledge_base_2"]
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "RAGChat spec should be valid: {:?}", result);
    }

    #[test]
    fn test_multimodal_chat_prompt_spec() {
        let validator = create_prompt_spec_validator().unwrap();
        
        let spec = json!({
            "spec_version": "1.0",
            "id": "multimodal-test",
            "model_class": "MultimodalChat",
            "messages": [
                {
                    "role": "user",
                    "content": "Describe what you see and hear"
                }
            ],
            "media": {
                "input_images": [
                    {
                        "url": "https://example.com/image1.jpg",
                        "alt_text": "A landscape photo"
                    },
                    {
                        "url": "https://example.com/image2.jpg",
                        "alt_text": "A portrait photo"
                    }
                ],
                "input_audio": {
                    "url": "https://example.com/audio.mp3",
                    "format": "mp3",
                    "duration_seconds": 180
                },
                "input_video": {
                    "url": "https://example.com/video.mp4",
                    "format": "mp4",
                    "duration_seconds": 60,
                    "fps": 30
                }
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "MultimodalChat spec should be valid: {:?}", result);
    }
}

#[cfg(test)]
mod invalid_document_tests {
    use super::*;

    #[test]
    fn test_invalid_message_structure() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Messages without content
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [
                {
                    "role": "user"
                    // content missing
                }
            ],
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err(), "Should fail with missing message content");
    }

    #[test]
    fn test_invalid_model_family_configuration() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Chat model with incompatible input modes
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "models": [
                {
                    "id": "chat-model",
                    "aliases": ["chat"],
                    "model_family": "chat",
                    "input_modes": ["text", "image", "video"]  // Invalid for chat
                }
            ]
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err(), "Should fail with incompatible input modes");
    }

    #[test]
    fn test_conflicting_capabilities_and_config() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Has RAG config but capability is false
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "capabilities": {
                "supports_rag": false,
                "supports_conversation_persistence": false
            },
            "rag_config": {
                "retrieval_strategies": ["semantic"]
            },
            "conversation_management": {
                "max_history_tokens": 2000
            },
            "models": []
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err(), "Should fail with conflicting capabilities");
        let error = result.unwrap_err();
        let error_str = error.to_string();
        assert!(error_str.contains("rag_config") || error_str.contains("conversation_management"));
    }
}

#[cfg(test)]
mod error_aggregation_tests {
    use super::*;

    #[test]
    fn test_multiple_validation_errors() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Multiple violations in one spec
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [
                {"role": "user", "content": "test"}
            ],
            "tool_choice": "auto",  // Missing tools
            "limits": {
                "reasoning_tokens": 1000  // Invalid for Chat
            },
            "rag": {  // Invalid for Chat
                "retrieval_strategy": "semantic"
            },
            "strict_mode": "Strict",
            "unknown_field": "value"  // Unknown field in Strict mode
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err(), "Should fail with multiple errors");
        
        let error = result.unwrap_err();
        let error_str = error.to_string();
        
        // Should contain multiple error messages
        assert!(error_str.contains("tool_choice"));
        assert!(error_str.contains("reasoning_tokens"));
        assert!(error_str.contains("rag"));
        assert!(error_str.contains("unknown_field"));
    }

    #[test]
    fn test_collect_all_errors() {
        let validator = create_prompt_spec_validator().unwrap();
        
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [
                {"role": "user", "content": "test"}
            ],
            "tool_choice": "auto",  // Missing tools
            "media": {
                "input_video": {"url": "test.mp4"}  // Invalid for Chat
            },
            "strict_mode": "Warn"
        });
        
        let _errors = validator.collect_errors(&spec);
        // collect_errors should return ValidationErrors with all issues
        // The exact API may vary, but it should collect multiple errors
    }
}

#[cfg(test)]
mod boundary_value_tests {
    use super::*;

    #[test]
    fn test_extremely_long_strings() {
        let validator = create_prompt_spec_validator().unwrap();
        
        let long_content = "a".repeat(100000);  // 100K characters
        let spec = json!({
            "spec_version": "1.0",
            "id": "long-test",
            "model_class": "Chat",
            "messages": [
                {
                    "role": "user",
                    "content": long_content
                }
            ],
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        // Should handle long strings without panicking
        // Result depends on schema constraints
        let _ = result;
    }

    #[test]
    fn test_deeply_nested_objects() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Create deeply nested metadata
        let mut nested = json!({"value": "deep"});
        for _ in 0..50 {
            nested = json!({"nested": nested});
        }
        
        let spec = json!({
            "spec_version": "1.0",
            "id": "nested-test",
            "model_class": "Chat",
            "messages": [
                {
                    "role": "user",
                    "content": "test",
                    "metadata": nested
                }
            ],
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        // Should handle deep nesting without stack overflow
        let _ = result;
    }

    #[test]
    fn test_large_arrays() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Create a large message array
        let mut messages = Vec::new();
        for i in 0..1000 {
            messages.push(json!({
                "role": if i % 2 == 0 { "user" } else { "assistant" },
                "content": format!("Message {}", i)
            }));
        }
        
        let spec = json!({
            "spec_version": "1.0",
            "id": "large-array-test",
            "model_class": "Chat",
            "messages": messages,
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        // Should handle large arrays efficiently
        assert!(result.is_ok(), "Should handle large message arrays: {:?}", result);
    }

    #[test]
    fn test_extreme_numeric_values() {
        let validator = create_prompt_spec_validator().unwrap();
        
        let spec = json!({
            "spec_version": "1.0",
            "id": "numeric-test",
            "model_class": "Chat",
            "messages": [
                {"role": "user", "content": "test"}
            ],
            "sampling": {
                "temperature": 1e308,  // Near f64::MAX
                "top_p": 1e-308,       // Near f64::MIN_POSITIVE
                "top_k": 2147483647,   // i32::MAX
                "frequency_penalty": -1e308,
                "presence_penalty": 1e308
            },
            "limits": {
                "max_output_tokens": 2147483647  // i32::MAX
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        // Should handle extreme values without panic
        let _ = result;
    }
}

#[cfg(test)]
mod unicode_and_special_chars {
    use super::*;

    #[test]
    fn test_unicode_in_messages() {
        let validator = create_prompt_spec_validator().unwrap();
        
        let spec = json!({
            "spec_version": "1.0",
            "id": "unicode-test",
            "model_class": "Chat",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello 世界 🌍 مرحبا мир",
                    "name": "用户_123"
                },
                {
                    "role": "assistant",
                    "content": "Unicode test: 你好 👋 السلام עליכם"
                }
            ],
            "metadata": {
                "emoji_key": "🔑",
                "chinese": "测试",
                "arabic": "اختبار",
                "hebrew": "בדיקה"
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should handle Unicode characters: {:?}", result);
    }

    #[test]
    fn test_special_characters_in_fields() {
        let validator = create_prompt_spec_validator().unwrap();
        
        let spec = json!({
            "spec_version": "1.0",
            "id": "special-chars-!@#$%",
            "model_class": "Chat",
            "messages": [
                {
                    "role": "user",
                    "content": "Special chars: \n\t\r\"'\\/<>{}[]()!@#$%^&*"
                }
            ],
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        // Should handle special characters appropriately
        let _ = result;
    }

    #[test]
    fn test_empty_strings() {
        let validator = create_prompt_spec_validator().unwrap();
        
        let spec = json!({
            "spec_version": "1.0",
            "id": "",  // Empty ID
            "model_class": "Chat",
            "messages": [
                {
                    "role": "user",
                    "content": ""  // Empty content
                }
            ],
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        // Empty strings may or may not be valid depending on schema
        let _ = result;
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_validation_performance() {
        let validator = create_prompt_spec_validator().unwrap();
        
        let spec = json!({
            "spec_version": "1.0",
            "id": "perf-test",
            "model_class": "Chat",
            "messages": [
                {"role": "user", "content": "Performance test"}
            ],
            "strict_mode": "Warn"
        });
        
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = validator.validate(&spec);
        }
        let duration = start.elapsed();
        
        // Should complete 1000 validations reasonably quickly
        assert!(
            duration.as_secs() < 5,
            "Validation too slow: {:?} for 1000 iterations",
            duration
        );
    }

    #[test]
    fn test_large_document_performance() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Create a large provider spec with many models
        let mut models = Vec::new();
        for i in 0..100 {
            models.push(json!({
                "id": format!("model-{}", i),
                "aliases": [format!("m{}", i)],
                "mappings": {
                    "paths": {
                        "messages": "$.messages",
                        "temperature": "$.temperature"
                    }
                }
            }));
        }
        
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "large-provider",
                "base_url": "https://api.example.com"
            },
            "models": models
        });
        
        let start = Instant::now();
        let result = validator.validate(&spec);
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Large spec should be valid");
        assert!(
            duration.as_millis() < 1000,
            "Large document validation too slow: {:?}",
            duration
        );
    }
}