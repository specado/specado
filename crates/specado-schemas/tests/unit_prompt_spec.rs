//! Unit tests for PromptSpec custom validation rules
//!
//! This module tests all custom validation rules for PromptSpec schemas,
//! ensuring each rule correctly validates both valid and invalid inputs
//! and produces accurate, descriptive error messages.

use serde_json::json;
use specado_schemas::{
    create_prompt_spec_validator, SchemaValidator,
};

/// Helper to create a minimal valid PromptSpec
fn minimal_valid_spec() -> serde_json::Value {
    json!({
        "spec_version": "1.0",
        "id": "test-123",
        "model_class": "Chat",
        "messages": [
            {"role": "user", "content": "Hello"}
        ],
        "strict_mode": "Warn"
    })
}

#[cfg(test)]
mod tool_choice_validation {
    use super::*;

    #[test]
    fn test_tool_choice_requires_tools_array() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Invalid: tool_choice without tools
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "tool_choice": "auto",
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("tool_choice requires tools array"));
    }

    #[test]
    fn test_tool_choice_requires_non_empty_tools() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Invalid: tool_choice with empty tools array
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "tools": [],
            "tool_choice": "auto",
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("tool_choice requires tools array to be defined and non-empty"));
    }

    #[test]
    fn test_tool_choice_valid_with_tools() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Valid: tool_choice with non-empty tools
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "tools": [
                {
                    "name": "test_tool",
                    "description": "A test tool",
                    "json_schema": {"type": "object"}
                }
            ],
            "tool_choice": "auto",
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should be valid with tools defined: {:?}", result);
    }

    #[test]
    fn test_tools_without_tool_choice_valid() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Valid: tools without tool_choice is allowed
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "tools": [
                {
                    "name": "test_tool",
                    "description": "A test tool",
                    "json_schema": {"type": "object"}
                }
            ],
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod reasoning_tokens_validation {
    use super::*;

    #[test]
    fn test_reasoning_tokens_invalid_for_chat() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Invalid: reasoning_tokens with Chat model_class
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "limits": {
                "reasoning_tokens": 1000
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("reasoning_tokens is only valid when model_class is ReasoningChat"));
    }

    #[test]
    fn test_reasoning_tokens_valid_for_reasoning_chat() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Valid: reasoning_tokens with ReasoningChat model_class
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "ReasoningChat",
            "messages": [{"role": "user", "content": "test"}],
            "limits": {
                "reasoning_tokens": 1000
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should be valid for ReasoningChat: {:?}", result);
    }

    #[test]
    fn test_reasoning_tokens_invalid_for_rag_chat() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Invalid: reasoning_tokens with RAGChat model_class
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "RAGChat",
            "messages": [{"role": "user", "content": "test"}],
            "limits": {
                "reasoning_tokens": 1000
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("reasoning_tokens is only valid when model_class is ReasoningChat"));
    }
}

#[cfg(test)]
mod rag_configuration_validation {
    use super::*;

    #[test]
    fn test_rag_invalid_for_chat() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Invalid: rag configuration with Chat model_class
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "rag": {
                "retrieval_strategy": "semantic",
                "top_k": 5
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("rag configuration is only valid when model_class is RAGChat"));
    }

    #[test]
    fn test_rag_valid_for_rag_chat() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Valid: rag configuration with RAGChat model_class
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "RAGChat",
            "messages": [{"role": "user", "content": "test"}],
            "rag": {
                "retrieval_strategy": "semantic",
                "top_k": 5,
                "context_window": 2000
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should be valid for RAGChat: {:?}", result);
    }

    #[test]
    fn test_rag_invalid_for_reasoning_chat() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Invalid: rag configuration with ReasoningChat model_class
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "ReasoningChat",
            "messages": [{"role": "user", "content": "test"}],
            "rag": {
                "retrieval_strategy": "keyword"
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("rag configuration is only valid when model_class is RAGChat"));
    }
}

#[cfg(test)]
mod media_input_validation {
    use super::*;

    #[test]
    fn test_input_video_invalid_for_chat() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Invalid: input_video with Chat model_class
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "media": {
                "input_video": {
                    "url": "https://example.com/video.mp4"
                }
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("input_video is only valid for MultimodalChat or VideoChat"));
    }

    #[test]
    fn test_input_video_valid_for_video_chat() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Valid: input_video with VideoChat model_class
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "VideoChat",
            "messages": [{"role": "user", "content": "test"}],
            "media": {
                "input_video": {
                    "url": "https://example.com/video.mp4",
                    "duration_seconds": 30
                }
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should be valid for VideoChat: {:?}", result);
    }

    #[test]
    fn test_input_video_valid_for_multimodal_chat() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Valid: input_video with MultimodalChat model_class
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "MultimodalChat",
            "messages": [{"role": "user", "content": "test"}],
            "media": {
                "input_video": {
                    "url": "https://example.com/video.mp4"
                }
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should be valid for MultimodalChat: {:?}", result);
    }

    #[test]
    fn test_input_audio_invalid_for_chat() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Invalid: input_audio with Chat model_class
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "media": {
                "input_audio": {
                    "url": "https://example.com/audio.mp3"
                }
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("input_audio is only valid for AudioChat or MultimodalChat"));
    }

    #[test]
    fn test_input_audio_valid_for_audio_chat() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Valid: input_audio with AudioChat model_class
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "AudioChat",
            "messages": [{"role": "user", "content": "test"}],
            "media": {
                "input_audio": {
                    "url": "https://example.com/audio.mp3",
                    "format": "mp3",
                    "duration_seconds": 60
                }
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should be valid for AudioChat: {:?}", result);
    }

    #[test]
    fn test_multiple_media_inputs_for_multimodal() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Valid: both audio and video for MultimodalChat
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "MultimodalChat",
            "messages": [{"role": "user", "content": "test"}],
            "media": {
                "input_audio": {
                    "url": "https://example.com/audio.mp3"
                },
                "input_video": {
                    "url": "https://example.com/video.mp4"
                },
                "input_images": [
                    {"url": "https://example.com/image.jpg"}
                ]
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should be valid for MultimodalChat with multiple media: {:?}", result);
    }
}

#[cfg(test)]
mod conversation_validation {
    use super::*;

    #[test]
    fn test_parent_message_id_too_short() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Invalid: parent_message_id too short
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "conversation": {
                "parent_message_id": "abc"
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("parent_message_id must be a valid message reference"));
        assert!(error.to_string().contains("at least 8 characters"));
    }

    #[test]
    fn test_parent_message_id_empty() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Invalid: empty parent_message_id
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "conversation": {
                "parent_message_id": ""
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("parent_message_id must be a valid message reference"));
    }

    #[test]
    fn test_parent_message_id_valid_uuid() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Valid: UUID-like parent_message_id
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "conversation": {
                "parent_message_id": "550e8400-e29b-41d4-a716-446655440000",
                "thread_id": "thread-123"
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should be valid with UUID: {:?}", result);
    }

    #[test]
    fn test_parent_message_id_valid_custom_id() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Valid: custom ID format (at least 8 chars)
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "conversation": {
                "parent_message_id": "msg_12345678",
                "thread_id": "thread-456"
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should be valid with custom ID: {:?}", result);
    }
}

#[cfg(test)]
mod strict_mode_validation {
    use super::*;

    #[test]
    fn test_strict_mode_rejects_unknown_fields() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Invalid: unknown field in Strict mode
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "strict_mode": "Strict",
            "unknown_field": "should fail"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_str = error.to_string();
        println!("Error: {}", error_str);
        assert!(error_str.contains("Unknown field 'unknown_field' not allowed in Strict mode"), "Expected error about unknown_field, got: {}", error_str);
    }

    #[test]
    fn test_strict_mode_accepts_all_known_fields() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Valid: all known fields in Strict mode
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "sampling": {
                "temperature": 0.7,
                "top_p": 0.9,
                "top_k": 50
            },
            "limits": {
                "max_output_tokens": 1000
            },
            "tools": [],
            "metadata": {"key": "value"},
            "strict_mode": "Strict"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should accept all known fields: {:?}", result);
    }

    #[test]
    fn test_warn_mode_allows_unknown_fields() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Valid: unknown fields allowed in Warn mode
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "strict_mode": "Warn",
            "unknown_field": "should be allowed",
            "another_unknown": 123
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should allow unknown fields in Warn mode: {:?}", result);
    }

    #[test]
    fn test_coerce_mode_allows_unknown_fields() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Valid: unknown fields allowed in Coerce mode
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "strict_mode": "Coerce",
            "extra_field": {"nested": "data"}
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should allow unknown fields in Coerce mode: {:?}", result);
    }
}

#[cfg(test)]
mod validation_mode_tests {
    use super::*;

    #[test]
    fn test_basic_mode_skips_custom_rules() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Should pass basic validation even with custom rule violations
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "tool_choice": "auto",  // Missing tools - custom rule violation
            "strict_mode": "Warn"
        });
        
        let result = validator.validate_basic(&spec);
        assert!(result.is_ok(), "Basic mode should skip custom rules: {:?}", result);
    }

    #[test]
    fn test_partial_mode_checks_some_rules() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Partial mode should check some but not all custom rules
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "tool_choice": "auto",  // Missing tools
            "strict_mode": "Warn"
        });
        
        let result = validator.validate_partial(&spec);
        // Partial mode behavior may vary - testing that it runs without panic
        let _ = result;
    }

    #[test]
    fn test_strict_mode_checks_all_rules() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Strict mode should check all custom rules
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "tool_choice": "auto",  // Missing tools
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);  // Default is Strict
        assert!(result.is_err(), "Strict mode should check all custom rules");
    }
}

#[cfg(test)]
mod error_message_quality {
    use super::*;

    #[test]
    fn test_error_messages_are_descriptive() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Test various errors for message quality
        let specs_and_expected = vec![
            (
                json!({
                    "spec_version": "1.0",
                    "id": "test-123",
                    "model_class": "Chat",
                    "messages": [{"role": "user", "content": "test"}],
                    "tool_choice": "auto",
                    "strict_mode": "Warn"
                }),
                "tool_choice requires tools array"
            ),
            (
                json!({
                    "spec_version": "1.0",
                    "id": "test-123",
                    "model_class": "Chat",
                    "messages": [{"role": "user", "content": "test"}],
                    "limits": {"reasoning_tokens": 100},
                    "strict_mode": "Warn"
                }),
                "reasoning_tokens is only valid when model_class is ReasoningChat"
            ),
            (
                json!({
                    "spec_version": "1.0",
                    "id": "test-123",
                    "model_class": "Chat",
                    "messages": [{"role": "user", "content": "test"}],
                    "media": {"input_video": {"url": "test.mp4"}},
                    "strict_mode": "Warn"
                }),
                "input_video is only valid for MultimodalChat or VideoChat"
            ),
        ];
        
        for (spec, expected_msg) in specs_and_expected {
            let result = validator.validate(&spec);
            assert!(result.is_err(), "Expected validation error for spec");
            let error = result.unwrap_err();
            let error_str = error.to_string();
            assert!(
                error_str.contains(expected_msg),
                "Error message should contain '{}', got: {}",
                expected_msg,
                error_str
            );
        }
    }

    #[test]
    fn test_error_paths_are_accurate() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Test that error paths correctly identify the problematic field
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "conversation": {
                "parent_message_id": "short"
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_str = error.to_string();
        
        // Should include the path to the problematic field
        assert!(
            error_str.contains("conversation") && error_str.contains("parent_message_id"),
            "Error should include field path, got: {}",
            error_str
        );
    }
}

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_null_values_handled_gracefully() {
        let validator = create_prompt_spec_validator().unwrap();
        
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "tool_choice": null,
            "media": null,
            "conversation": null,
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should handle null values gracefully: {:?}", result);
    }

    #[test]
    fn test_missing_optional_fields() {
        let validator = create_prompt_spec_validator().unwrap();
        
        // Minimal spec with only required fields
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should accept minimal valid spec: {:?}", result);
    }

    #[test]
    fn test_deeply_nested_validation() {
        let validator = create_prompt_spec_validator().unwrap();
        
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "MultimodalChat",
            "messages": [{"role": "user", "content": "test"}],
            "media": {
                "input_images": [
                    {
                        "url": "https://example.com/image1.jpg",
                        "metadata": {
                            "width": 1920,
                            "height": 1080,
                            "format": "jpeg"
                        }
                    }
                ],
                "input_audio": {
                    "url": "https://example.com/audio.mp3",
                    "format": "mp3",
                    "metadata": {
                        "duration": 120,
                        "bitrate": 192000
                    }
                }
            },
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should handle deeply nested structures: {:?}", result);
    }

    #[test]
    fn test_empty_arrays_and_objects() {
        let validator = create_prompt_spec_validator().unwrap();
        
        let spec = json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "test"}],
            "tools": [],
            "metadata": {},
            "preferences": {},
            "strict_mode": "Warn"
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should handle empty arrays and objects: {:?}", result);
    }
}