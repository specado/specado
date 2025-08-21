//! Tests for the validation system
//!
//! This module contains comprehensive tests for all validation functionality,
//! ensuring that the refactored modules work correctly together.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

#[cfg(test)]
mod tests {
    use super::super::{ValidationError, ValidationSeverity, ValidationMode, PreValidator};
    use crate::{
        Error, Result, StrictMode, Constraints, ConstraintLimits, EndpointConfig, Endpoints, 
        InputModes, JsonOutputConfig, Mappings, Message, MessageRole, ProviderInfo, 
        ResponseNormalization, StreamNormalization, SyncNormalization, ToolingConfig, 
        EventSelector, ProviderSpec, ModelSpec, PromptSpec, Tool, Limits, MediaConfig, 
        SamplingParams,
    };
    use super::super::super::TranslationContext;
    use std::collections::HashMap;

    fn create_test_context_with_mode(strict_mode: StrictMode) -> TranslationContext {
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
            strict_mode,
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
                images: false, // Images not supported for testing
            },
            tooling: ToolingConfig {
                tools_supported: false, // Tools not supported for testing
                parallel_tool_calls_default: false,
                can_disable_parallel_tool_calls: false,
                disable_switch: None,
            },
            json_output: JsonOutputConfig {
                native_param: true,
                strategy: "native".to_string(),
            },
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
                    event_selector: EventSelector {
                        type_path: "type".to_string(),
                        routes: vec![],
                    },
                },
            },
        };

        TranslationContext::new(prompt_spec, provider_spec, model_spec, strict_mode)
    }

    #[test]
    fn test_validate_empty_messages() {
        let mut context = create_test_context_with_mode(StrictMode::Strict);
        context.prompt_spec.messages.clear();
        
        let validator = PreValidator::new(&context);
        let result = validator.validate_strict();
        
        assert!(result.is_err());
        if let Err(Error::Validation { field, .. }) = result {
            assert_eq!(field, "messages");
        } else {
            panic!("Expected Validation error");
        }
    }
    
    #[test]
    fn test_validate_comprehensive_errors() {
        let mut context = create_test_context_with_mode(StrictMode::Warn);
        context.prompt_spec.messages.clear();
        
        let validator = PreValidator::new(&context);
        let errors = validator.validate().unwrap();
        
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.field_path == "messages"));
    }

    #[test]
    fn test_validate_unsupported_model_class() {
        let mut context = create_test_context_with_mode(StrictMode::Strict);
        context.prompt_spec.model_class = "UnsupportedClass".to_string();
        
        let validator = PreValidator::new(&context);
        let result = validator.validate_strict();
        
        assert!(result.is_err());
        if let Err(Error::Validation { field, .. }) = result {
            assert_eq!(field, "model_class");
        } else {
            panic!("Expected Validation error");
        }
    }

    #[test]
    fn test_validate_zero_max_tokens() {
        let mut context = create_test_context_with_mode(StrictMode::Strict);
        context.prompt_spec.limits = Some(Limits {
            max_output_tokens: Some(0),
            reasoning_tokens: None,
            max_prompt_tokens: None,
        });
        
        let validator = PreValidator::new(&context);
        let result = validator.validate_strict();
        
        assert!(result.is_err());
        if let Err(Error::Validation { field, .. }) = result {
            assert_eq!(field, "limits.max_output_tokens");
        } else {
            panic!("Expected Validation error");
        }
    }

    #[test]
    fn test_validate_unsupported_tools_strict() {
        let mut context = create_test_context_with_mode(StrictMode::Strict);
        context.prompt_spec.tools = Some(vec![Tool {
            name: "test_tool".to_string(),
            description: Some("Test tool".to_string()),
            json_schema: serde_json::json!({}),
        }]);
        
        let validator = PreValidator::new(&context);
        let result = validator.validate_strict();
        
        assert!(result.is_err());
        // With comprehensive validation, this should now be a Validation error
        if let Err(Error::Validation { field, .. }) = result {
            assert_eq!(field, "tools");
        } else {
            panic!("Expected Validation error, got: {:?}", result);
        }
    }

    #[test]
    fn test_validate_unsupported_tools_warn() {
        let mut context = create_test_context_with_mode(StrictMode::Warn);
        context.prompt_spec.tools = Some(vec![Tool {
            name: "test_tool".to_string(),
            description: Some("Test tool".to_string()),
            json_schema: serde_json::json!({}),
        }]);
        
        let validator = PreValidator::new(&context);
        let result = validator.validate_strict();
        
        // Should not fail in Warn mode with validate_strict
        assert!(result.is_ok());
        
        // In Warn mode (lenient validation), warnings are filtered out, so no errors expected
        let errors = validator.validate().unwrap();
        assert!(errors.is_empty(), "Expected no errors in lenient mode, but found: {:?}", errors);
        
        // But warnings should be present in strict validation mode
        let validator_strict_mode = PreValidator::with_mode(&context, ValidationMode::Strict);
        let strict_errors = validator_strict_mode.validate().unwrap();
        assert!(strict_errors.iter().any(|e| e.field_path == "tools" && e.severity == ValidationSeverity::Warning));
    }

    #[test]
    fn test_validate_unsupported_images_strict() {
        let mut context = create_test_context_with_mode(StrictMode::Strict);
        context.prompt_spec.media = Some(MediaConfig {
            input_images: Some(vec![serde_json::json!({"url": "test.jpg"})]),
            input_audio: None,
            output_audio: None,
        });
        
        // Set the model to not support images for this test
        context.model_spec.input_modes.images = false;
        
        let validator = PreValidator::new(&context);
        let result = validator.validate_strict();
        
        assert!(result.is_err());
        if let Err(Error::Validation { field, .. }) = result {
            assert_eq!(field, "media.input_images");
        } else {
            panic!("Expected Validation error, got: {:?}", result);
        }
    }

    #[test]
    fn test_validate_success() {
        let context = create_test_context_with_mode(StrictMode::Strict);
        let validator = PreValidator::new(&context);
        let result = validator.validate_strict();
        
        assert!(result.is_ok());
        
        // Also test comprehensive validation
        let errors = validator.validate().unwrap();
        assert!(errors.is_empty() || errors.iter().all(|e| e.severity == ValidationSeverity::Info));
    }
    
    #[test]
    fn test_validation_modes() {
        let mut context = create_test_context_with_mode(StrictMode::Warn);
        
        // Add some issues that would generate warnings
        context.prompt_spec.sampling = Some(SamplingParams {
            temperature: Some(3.0), // Invalid temperature
            top_p: Some(1.5), // Invalid top_p
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
        });
        
        // Test strict mode
        let strict_validator = PreValidator::with_mode(&context, ValidationMode::Strict);
        let strict_errors = strict_validator.validate().unwrap();
        assert!(strict_errors.len() >= 2); // Should include warnings
        
        // Test lenient mode  
        let lenient_validator = PreValidator::with_mode(&context, ValidationMode::Lenient);
        let lenient_errors = lenient_validator.validate().unwrap();
        assert!(lenient_errors.len() <= strict_errors.len()); // Should have fewer errors
    }
    
    #[test]
    fn test_detailed_validation_errors() {
        let mut context = create_test_context_with_mode(StrictMode::Warn);
        context.prompt_spec.limits = Some(Limits {
            max_output_tokens: Some(0),
            reasoning_tokens: Some(100),
            max_prompt_tokens: None,
        });
        
        let validator = PreValidator::new(&context);
        let errors = validator.validate().unwrap();
        
        // Should have detailed error information
        let token_error = errors.iter().find(|e| e.field_path == "limits.max_output_tokens").unwrap();
        assert_eq!(token_error.severity, ValidationSeverity::Error);
        assert!(token_error.expected.is_some());
        assert!(token_error.actual.is_some());
        
        // Should warn about reasoning tokens on non-reasoning model
        // Note: In lenient mode (StrictMode::Warn), warnings are filtered out
        // So let's test with strict validation mode
        let validator_strict_mode = PreValidator::with_mode(&context, ValidationMode::Strict);
        let strict_errors = validator_strict_mode.validate().unwrap();
        let reasoning_warning = strict_errors.iter().find(|e| e.field_path == "limits.reasoning_tokens");
        assert!(reasoning_warning.is_some());
        assert_eq!(reasoning_warning.unwrap().severity, ValidationSeverity::Warning);
    }

    #[test]
    fn test_comprehensive_validation_features() {
        // Test comprehensive validation with multiple issues
        let mut context = create_test_context_with_mode(StrictMode::Strict);
        
        // Set up a problematic prompt spec
        context.prompt_spec.model_class = "UnsupportedClass".to_string();
        context.prompt_spec.messages.clear(); // Empty messages
        context.prompt_spec.sampling = Some(SamplingParams {
            temperature: Some(3.0), // Invalid temperature
            top_p: Some(1.5), // Invalid top_p  
            top_k: None,
            frequency_penalty: Some(3.0), // Invalid frequency penalty
            presence_penalty: None,
        });
        context.prompt_spec.limits = Some(Limits {
            max_output_tokens: Some(0), // Invalid zero value
            reasoning_tokens: Some(100), // Invalid for non-reasoning model
            max_prompt_tokens: None,
        });
        context.prompt_spec.tools = Some(vec![
            Tool {
                name: "".to_string(), // Empty tool name
                description: Some("Bad tool".to_string()),
                json_schema: serde_json::json!({}),
            },
            Tool {
                name: "invalid-name!".to_string(), // Invalid characters
                description: Some("Another bad tool".to_string()),
                json_schema: serde_json::json!({}),
            }
        ]);
        
        let validator = PreValidator::with_mode(&context, ValidationMode::Strict);
        let errors = validator.validate().unwrap();
        
        // Should have multiple validation errors
        assert!(!errors.is_empty());
        
        // Check specific error types
        assert!(errors.iter().any(|e| e.field_path == "model_class"));
        assert!(errors.iter().any(|e| e.field_path == "messages"));
        assert!(errors.iter().any(|e| e.field_path == "sampling.temperature"));
        assert!(errors.iter().any(|e| e.field_path == "sampling.top_p"));
        assert!(errors.iter().any(|e| e.field_path == "sampling.frequency_penalty"));
        assert!(errors.iter().any(|e| e.field_path == "limits.max_output_tokens"));
        assert!(errors.iter().any(|e| e.field_path == "limits.reasoning_tokens"));
        assert!(errors.iter().any(|e| e.field_path == "tools[0].name"));
        assert!(errors.iter().any(|e| e.field_path == "tools[1].name"));
        
        // Check that errors have detailed information
        for error in &errors {
            assert!(!error.message.is_empty());
            assert!(!error.field_path.is_empty());
            // Most errors should have expected/actual values
            if error.field_path.contains("temperature") || error.field_path.contains("max_output_tokens") {
                assert!(error.expected.is_some());
                assert!(error.actual.is_some());
            }
        }
        
        // Test that lenient mode filters to only errors
        let validator_lenient = PreValidator::with_mode(&context, ValidationMode::Lenient);
        let lenient_errors = validator_lenient.validate().unwrap();
        assert!(lenient_errors.len() <= errors.len());
        assert!(lenient_errors.iter().all(|e| e.severity == ValidationSeverity::Error));
    }

    #[test]
    fn test_provider_compatibility_validation() {
        // Test provider-specific validation features
        let mut context = create_test_context_with_mode(StrictMode::Strict);
        
        // Test image support validation
        context.model_spec.input_modes.images = false;
        context.prompt_spec.media = Some(MediaConfig {
            input_images: Some(vec![serde_json::json!({"url": "test.jpg"})]),
            input_audio: None,
            output_audio: None,
        });
        
        // Test JSON format support
        context.model_spec.json_output.native_param = false;
        context.model_spec.json_output.strategy = "unsupported".to_string();
        context.prompt_spec.response_format = Some(crate::ResponseFormat::JsonObject);
        
        let validator = PreValidator::new(&context);
        let errors = validator.validate().unwrap();
        
        // Should detect image incompatibility  
        assert!(errors.iter().any(|e| e.field_path == "media.input_images"));
        
        // Should detect JSON format incompatibility
        assert!(errors.iter().any(|e| e.field_path == "response_format"));
    }

    #[test]  
    fn test_system_prompt_location_validation() {
        // Test system prompt location constraints
        let mut context = create_test_context_with_mode(StrictMode::Strict);
        
        // Set up messages with system message not first
        context.prompt_spec.messages = vec![
            Message {
                role: MessageRole::User,
                content: "Hello".to_string(),
                name: None,
                metadata: None,
            },
            Message {
                role: MessageRole::System,
                content: "You are a helpful assistant.".to_string(),
                name: None,
                metadata: None,
            },
        ];
        
        // Provider requires system message to be first
        context.model_spec.constraints.system_prompt_location = "first".to_string();
        
        let validator = PreValidator::new(&context);
        let errors = validator.validate().unwrap();
        
        // Should detect system message position error
        assert!(errors.iter().any(|e| e.field_path == "messages[1]"));
        
        // Test "none" constraint - create a new context
        let mut context_none = create_test_context_with_mode(StrictMode::Strict);
        context_none.prompt_spec.messages = vec![
            Message {
                role: MessageRole::User,
                content: "Hello".to_string(),
                name: None,
                metadata: None,
            },
            Message {
                role: MessageRole::System,
                content: "You are a helpful assistant.".to_string(),
                name: None,
                metadata: None,
            },
        ];
        context_none.model_spec.constraints.system_prompt_location = "none".to_string();
        
        let validator_none = PreValidator::new(&context_none);
        let errors_none = validator_none.validate().unwrap();
        assert!(errors_none.iter().any(|e| e.field_path == "messages"));
    }

    #[test]
    fn test_validation_error_details() {
        // Test that validation errors have comprehensive details
        let mut context = create_test_context_with_mode(StrictMode::Strict);
        context.prompt_spec.limits = Some(Limits {
            max_output_tokens: Some(0),
            reasoning_tokens: None,
            max_prompt_tokens: None,
        });
        
        let validator = PreValidator::new(&context);
        let errors = validator.validate().unwrap();
        
        let token_error = errors.iter().find(|e| e.field_path == "limits.max_output_tokens").unwrap();
        
        // Check all fields are populated
        assert!(!token_error.field_path.is_empty());
        assert!(!token_error.message.is_empty());
        assert!(token_error.expected.is_some());
        assert!(token_error.actual.is_some());
        assert_eq!(token_error.severity, ValidationSeverity::Error);
        
        // Check specific content
        assert_eq!(token_error.expected.as_ref().unwrap(), "Positive integer");
        assert_eq!(token_error.actual.as_ref().unwrap(), "0");
        assert!(token_error.message.contains("greater than 0"));
    }
}