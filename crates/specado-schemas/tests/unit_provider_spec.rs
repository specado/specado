//! Unit tests for ProviderSpec custom validation rules
//!
//! This module tests all custom validation rules for ProviderSpec schemas,
//! ensuring each rule correctly validates both valid and invalid inputs
//! and produces accurate, descriptive error messages.

use serde_json::json;
use specado_schemas::{
    create_provider_spec_validator, SchemaValidator,
};

/// Helper to create a minimal valid ProviderSpec
fn minimal_valid_spec() -> serde_json::Value {
    json!({
        "spec_version": "1.0.0",
        "provider": {
            "name": "test-provider",
            "base_url": "https://api.example.com"
        },
        "models": []
    })
}

#[cfg(test)]
mod jsonpath_validation {
    use super::*;

    #[test]
    fn test_invalid_jsonpath_in_mappings() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Invalid: malformed JSONPath expression
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "models": [
                {
                    "id": "model-1",
                    "aliases": ["m1"],
                    "mappings": {
                        "paths": {
                            "messages": "$...[invalid",
                            "temperature": "$.params.temp"
                        }
                    }
                }
            ]
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid JSONPath expression"));
    }

    #[test]
    fn test_valid_jsonpath_expressions() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Valid: well-formed JSONPath expressions
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "models": [
                {
                    "id": "model-1",
                    "aliases": ["m1"],
                    "mappings": {
                        "paths": {
                            "messages": "$.messages",
                            "temperature": "$.parameters.temperature",
                            "max_tokens": "$.config.max_output_tokens",
                            "tools": "$..tools[*]"
                        }
                    }
                }
            ]
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should accept valid JSONPath expressions: {:?}", result);
    }

    #[test]
    fn test_complex_jsonpath_expressions() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Valid: complex JSONPath expressions
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "models": [
                {
                    "id": "model-1",
                    "aliases": ["m1"],
                    "mappings": {
                        "paths": {
                            "messages": "$['messages'][*]",
                            "system": "$.messages[?(@.role=='system')].content",
                            "user": "$.messages[?(@.role=='user')]",
                            "assistant": "$..messages[?(@.role=='assistant')]"
                        }
                    }
                }
            ]
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should accept complex JSONPath: {:?}", result);
    }

    #[test]
    fn test_invalid_jsonpath_in_response_normalization() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Invalid: malformed JSONPath in response_normalization
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "models": [
                {
                    "id": "model-1",
                    "aliases": ["m1"],
                    "response_normalization": {
                        "content": "$.choices[0.content",  // Missing closing bracket
                        "role": "$.choices[0].message.role"
                    }
                }
            ]
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid JSONPath in response_normalization"));
    }
}

#[cfg(test)]
mod environment_variable_validation {
    use super::*;

    #[test]
    fn test_invalid_env_var_format() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Invalid: incorrect environment variable format
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "authentication": {
                "env_var": "$ENV:API_KEY"  // Missing braces
            },
            "models": []
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid environment variable reference"));
    }

    #[test]
    fn test_valid_env_var_format() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Valid: correct environment variable format
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "authentication": {
                "env_var": "${ENV:API_KEY}"
            },
            "models": []
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should accept valid env var format: {:?}", result);
    }

    #[test]
    fn test_multiple_env_vars() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Valid: multiple environment variables
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "authentication": {
                "env_var": "${ENV:PRIMARY_API_KEY}",
                "fallback_env_var": "${ENV:SECONDARY_API_KEY}"
            },
            "models": []
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should accept multiple env vars: {:?}", result);
    }
}

#[cfg(test)]
mod input_mode_validation {
    use super::*;

    #[test]
    fn test_chat_model_with_image_input_invalid() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Invalid: chat model with image input mode
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
                    "input_modes": ["text", "image"]
                }
            ]
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Chat models cannot have image input mode"));
    }

    #[test]
    fn test_chat_model_with_text_input_valid() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Valid: chat model with only text input
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
                    "input_modes": ["text"]
                }
            ]
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Chat model with text input should be valid: {:?}", result);
    }

    #[test]
    fn test_multimodal_model_with_multiple_inputs() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Valid: multimodal model with multiple input types
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "models": [
                {
                    "id": "multimodal-model",
                    "aliases": ["mm"],
                    "model_family": "multimodal",
                    "input_modes": ["text", "image", "audio", "video"]
                }
            ]
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Multimodal model should accept all input types: {:?}", result);
    }
}

#[cfg(test)]
mod tooling_validation {
    use super::*;

    #[test]
    fn test_tool_choice_modes_missing_auto() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Invalid: supports_tools but tool_choice_modes doesn't include "auto"
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "capabilities": {
                "supports_tools": true
            },
            "tooling": {
                "tool_choice_modes": ["none", "required"],
                "max_tools": 10
            },
            "models": []
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("tool_choice_modes must include 'auto'"));
    }

    #[test]
    fn test_tool_choice_modes_with_auto() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Valid: tool_choice_modes includes "auto"
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "capabilities": {
                "supports_tools": true
            },
            "tooling": {
                "tool_choice_modes": ["auto", "none", "required", "specific"],
                "max_tools": 128,
                "parallel_tool_calls": true
            },
            "models": []
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should be valid with 'auto' in tool_choice_modes: {:?}", result);
    }

    #[test]
    fn test_tooling_without_capability() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Valid: tooling config without supports_tools is allowed (no constraint)
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "capabilities": {
                "supports_tools": false
            },
            "tooling": {
                "tool_choice_modes": ["none"],
                "max_tools": 0
            },
            "models": []
        });
        
        let result = validator.validate(&spec);
        // This should be valid - the rule only applies when supports_tools is true
        assert!(result.is_ok(), "Tooling config without capability should be allowed: {:?}", result);
    }
}

#[cfg(test)]
mod rag_capability_validation {
    use super::*;

    #[test]
    fn test_rag_config_without_capability() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Invalid: rag_config without supports_rag capability
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "capabilities": {
                "supports_rag": false
            },
            "rag_config": {
                "retrieval_strategies": ["semantic", "keyword"],
                "max_context_tokens": 4000
            },
            "models": []
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("rag_config is only valid when capabilities.supports_rag is true"));
    }

    #[test]
    fn test_rag_config_with_capability() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Valid: rag_config with supports_rag capability
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "capabilities": {
                "supports_rag": true
            },
            "rag_config": {
                "retrieval_strategies": ["semantic", "keyword", "hybrid"],
                "max_context_tokens": 8000,
                "chunk_size": 512,
                "chunk_overlap": 128
            },
            "models": []
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should be valid with RAG capability: {:?}", result);
    }

    #[test]
    fn test_missing_rag_capability_defaults_to_false() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Invalid: rag_config when supports_rag is missing (defaults to false)
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "capabilities": {
                // supports_rag not specified
            },
            "rag_config": {
                "retrieval_strategies": ["semantic"]
            },
            "models": []
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("rag_config is only valid when capabilities.supports_rag is true"));
    }
}

#[cfg(test)]
mod conversation_capability_validation {
    use super::*;

    #[test]
    fn test_conversation_management_without_capability() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Invalid: conversation_management without supports_conversation_persistence
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "capabilities": {
                "supports_conversation_persistence": false
            },
            "conversation_management": {
                "max_history_tokens": 2000,
                "message_retention_policy": "last_n"
            },
            "models": []
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("conversation_management is only valid when capabilities.supports_conversation_persistence is true"));
    }

    #[test]
    fn test_conversation_management_with_capability() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Valid: conversation_management with supports_conversation_persistence
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "capabilities": {
                "supports_conversation_persistence": true
            },
            "conversation_management": {
                "max_history_tokens": 4000,
                "message_retention_policy": "sliding_window",
                "context_window_size": 10,
                "supports_branching": true
            },
            "models": []
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should be valid with conversation capability: {:?}", result);
    }
}

#[cfg(test)]
mod endpoint_protocol_validation {
    use super::*;

    #[test]
    fn test_mismatched_http_https_protocols() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Invalid: HTTPS base_url with HTTP endpoint
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "models": [
                {
                    "id": "model-1",
                    "aliases": ["m1"],
                    "endpoint": {
                        "protocol": "http",
                        "path": "/v1/chat"
                    }
                }
            ]
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Endpoint protocol http doesn't match base_url security"));
    }

    #[test]
    fn test_matched_https_protocols() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Valid: HTTPS base_url with HTTPS endpoint
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "models": [
                {
                    "id": "model-1",
                    "aliases": ["m1"],
                    "endpoint": {
                        "protocol": "https",
                        "path": "/v1/chat/completions"
                    }
                }
            ]
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should be valid with matching HTTPS: {:?}", result);
    }

    #[test]
    fn test_websocket_protocol_matching() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Invalid: WSS base_url with WS endpoint
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "wss://api.example.com"
            },
            "models": [
                {
                    "id": "model-1",
                    "aliases": ["m1"],
                    "endpoint": {
                        "protocol": "ws",
                        "path": "/v1/stream"
                    }
                }
            ]
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("doesn't match base_url security"));
    }

    #[test]
    fn test_matched_websocket_protocols() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Valid: WSS base_url with WSS endpoint
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "wss://api.example.com"
            },
            "models": [
                {
                    "id": "model-1",
                    "aliases": ["m1"],
                    "endpoint": {
                        "protocol": "wss",
                        "path": "/v1/stream"
                    }
                }
            ]
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should be valid with matching WSS: {:?}", result);
    }
}

#[cfg(test)]
mod validation_mode_tests {
    use super::*;

    #[test]
    fn test_basic_mode_skips_custom_rules() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Should pass basic validation even with custom rule violations
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "authentication": {
                "env_var": "INVALID_FORMAT"  // Missing ${ENV:} wrapper
            },
            "models": []
        });
        
        let result = validator.validate_basic(&spec);
        assert!(result.is_ok(), "Basic mode should skip custom rules: {:?}", result);
    }

    #[test]
    fn test_strict_mode_checks_all_rules() {
        let validator = create_provider_spec_validator().unwrap();
        
        // Strict mode should check all custom rules
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "capabilities": {
                "supports_rag": false
            },
            "rag_config": {
                "retrieval_strategies": ["semantic"]
            },
            "models": []
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
        let validator = create_provider_spec_validator().unwrap();
        
        // Test various errors for message quality
        let specs_and_expected = vec![
            (
                json!({
                    "spec_version": "1.0.0",
                    "provider": {
                        "name": "test",
                        "base_url": "https://api.example.com"
                    },
                    "models": [{
                        "id": "m1",
                        "aliases": ["model1"],
                        "mappings": {
                            "paths": {
                                "messages": "$[invalid"
                            }
                        }
                    }]
                }),
                "Invalid JSONPath expression"
            ),
            (
                json!({
                    "spec_version": "1.0.0",
                    "provider": {
                        "name": "test",
                        "base_url": "https://api.example.com"
                    },
                    "authentication": {
                        "env_var": "BAD_FORMAT"
                    },
                    "models": []
                }),
                "Invalid environment variable reference"
            ),
            (
                json!({
                    "spec_version": "1.0.0",
                    "provider": {
                        "name": "test",
                        "base_url": "https://api.example.com"
                    },
                    "capabilities": {
                        "supports_tools": true
                    },
                    "tooling": {
                        "tool_choice_modes": ["none"]
                    },
                    "models": []
                }),
                "tool_choice_modes must include 'auto'"
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
        let validator = create_provider_spec_validator().unwrap();
        
        // Test that error paths correctly identify the problematic field
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "models": [
                {
                    "id": "model-1",
                    "aliases": ["m1"],
                    "mappings": {
                        "paths": {
                            "temperature": "$[invalid"
                        }
                    }
                }
            ]
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_str = error.to_string();
        
        // Should include the model index and field path
        assert!(
            error_str.contains("models[0]") && error_str.contains("temperature"),
            "Error should include detailed path, got: {}",
            error_str
        );
    }
}

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_null_values_handled_gracefully() {
        let validator = create_provider_spec_validator().unwrap();
        
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "authentication": null,
            "capabilities": null,
            "tooling": null,
            "rag_config": null,
            "models": []
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should handle null values gracefully: {:?}", result);
    }

    #[test]
    fn test_empty_arrays_and_objects() {
        let validator = create_provider_spec_validator().unwrap();
        
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com",
                "headers": {},
                "query_params": {}
            },
            "models": [],
            "capabilities": {},
            "rate_limits": []
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should handle empty collections: {:?}", result);
    }

    #[test]
    fn test_deeply_nested_validation() {
        let validator = create_provider_spec_validator().unwrap();
        
        let spec = json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.example.com"
            },
            "models": [
                {
                    "id": "model-1",
                    "aliases": ["m1", "model-one"],
                    "mappings": {
                        "paths": {
                            "messages": "$.messages",
                            "system": "$.system_prompt"
                        },
                        "transformations": {
                            "temperature": {
                                "scale": [0.0, 1.0],
                                "default": 0.7
                            }
                        }
                    },
                    "endpoint": {
                        "protocol": "https",
                        "path": "/v1/chat",
                        "headers": {
                            "X-Model-Version": "latest"
                        }
                    },
                    "constraints": {
                        "max_tokens": 4096,
                        "max_messages": 100
                    }
                }
            ]
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should handle deeply nested structures: {:?}", result);
    }

    #[test]
    fn test_multiple_models_with_different_configs() {
        let validator = create_provider_spec_validator().unwrap();
        
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
                    "input_modes": ["text"]
                },
                {
                    "id": "multimodal-model",
                    "aliases": ["mm"],
                    "model_family": "multimodal",
                    "input_modes": ["text", "image", "audio"]
                },
                {
                    "id": "code-model",
                    "aliases": ["code"],
                    "model_family": "code",
                    "input_modes": ["text"],
                    "mappings": {
                        "paths": {
                            "code": "$.code_input",
                            "language": "$.programming_language"
                        }
                    }
                }
            ]
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Should handle multiple models: {:?}", result);
    }
}