//! Pre-validation logic for translation operations
//!
//! This module will implement comprehensive pre-validation logic in issue #16.
//! Currently provides a minimal placeholder implementation.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::{Error, Result};
use super::TranslationContext;

/// Pre-validator for checking input compatibility before translation
///
/// The PreValidator performs validation checks on the input PromptSpec
/// to ensure it can be successfully translated to the target provider format.
/// This includes checking for:
/// - Required fields based on model class
/// - Constraint violations
/// - Incompatible feature combinations
/// - Token limit violations
pub struct PreValidator<'a> {
    context: &'a TranslationContext,
}

impl<'a> PreValidator<'a> {
    /// Create a new pre-validator
    pub fn new(context: &'a TranslationContext) -> Self {
        Self { context }
    }

    /// Perform pre-validation checks
    ///
    /// This is a placeholder implementation. The full validation logic
    /// will be implemented in issue #16.
    pub fn validate(&self) -> Result<()> {
        // Validate messages are not empty
        if self.context.prompt_spec.messages.is_empty() {
            return Err(Error::Validation {
                field: "messages".to_string(),
                message: "Messages array cannot be empty".to_string(),
                expected: Some("At least one message".to_string()),
            });
        }

        // Validate model class is supported
        let supported_classes = ["Chat", "ReasoningChat", "VisionChat", "AudioChat", "MultimodalChat"];
        if !supported_classes.contains(&self.context.prompt_spec.model_class.as_str()) {
            return Err(Error::Validation {
                field: "model_class".to_string(),
                message: format!(
                    "Model class '{}' is not supported",
                    self.context.prompt_spec.model_class
                ),
                expected: Some(format!("One of: {:?}", supported_classes)),
            });
        }

        // Check for basic constraint violations
        if let Some(ref limits) = self.context.prompt_spec.limits {
            // Check max output tokens against provider limits
            if let Some(max_output) = limits.max_output_tokens {
                // TODO: Check against actual provider limits (issue #16)
                if max_output == 0 {
                    return Err(Error::Validation {
                        field: "limits.max_output_tokens".to_string(),
                        message: "Max output tokens must be greater than 0".to_string(),
                        expected: Some("Positive integer".to_string()),
                    });
                }
            }
        }

        // Check for tool compatibility
        if self.context.prompt_spec.tools.is_some() && !self.context.supports_tools() {
            if self.context.should_fail_on_error() {
                return Err(Error::Unsupported {
                    message: format!(
                        "Provider '{}' does not support tools",
                        self.context.provider_name()
                    ),
                    feature: Some("tools".to_string()),
                });
            }
        }

        // Check for image compatibility
        if let Some(ref media) = self.context.prompt_spec.media {
            if media.input_images.is_some() && !self.context.supports_images() {
                if self.context.should_fail_on_error() {
                    return Err(Error::Unsupported {
                        message: format!(
                            "Model '{}' does not support image inputs",
                            self.context.model_id()
                        ),
                        feature: Some("input_images".to_string()),
                    });
                }
            }
        }

        // TODO: Add more validation checks in issue #16:
        // - Token count validation
        // - System prompt size validation
        // - Tool schema size validation
        // - Mutually exclusive field validation
        // - Model-specific constraint validation

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Message, MessageRole, PromptSpec, ProviderSpec, ModelSpec, StrictMode,
        ProviderInfo, Endpoints, EndpointConfig, InputModes, ToolingConfig,
        JsonOutputConfig, Constraints, ConstraintLimits, Mappings,
        ResponseNormalization, SyncNormalization, StreamNormalization,
        Tool, Limits, MediaConfig,
    };
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
                    event_selector: crate::EventSelector {
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
        let result = validator.validate();
        
        assert!(result.is_err());
        if let Err(Error::Validation { field, .. }) = result {
            assert_eq!(field, "messages");
        } else {
            panic!("Expected Validation error");
        }
    }

    #[test]
    fn test_validate_unsupported_model_class() {
        let mut context = create_test_context_with_mode(StrictMode::Strict);
        context.prompt_spec.model_class = "UnsupportedClass".to_string();
        
        let validator = PreValidator::new(&context);
        let result = validator.validate();
        
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
        let result = validator.validate();
        
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
        let result = validator.validate();
        
        assert!(result.is_err());
        if let Err(Error::Unsupported { feature, .. }) = result {
            assert_eq!(feature, Some("tools".to_string()));
        } else {
            panic!("Expected Unsupported error");
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
        let result = validator.validate();
        
        // Should not fail in Warn mode
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_unsupported_images_strict() {
        let mut context = create_test_context_with_mode(StrictMode::Strict);
        context.prompt_spec.media = Some(MediaConfig {
            input_images: Some(vec![serde_json::json!({"url": "test.jpg"})]),
            input_audio: None,
            output_audio: None,
        });
        
        let validator = PreValidator::new(&context);
        let result = validator.validate();
        
        assert!(result.is_err());
        if let Err(Error::Unsupported { feature, .. }) = result {
            assert_eq!(feature, Some("input_images".to_string()));
        } else {
            panic!("Expected Unsupported error");
        }
    }

    #[test]
    fn test_validate_success() {
        let context = create_test_context_with_mode(StrictMode::Strict);
        let validator = PreValidator::new(&context);
        let result = validator.validate();
        
        assert!(result.is_ok());
    }
}