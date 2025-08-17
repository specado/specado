//! Translation engine for converting PromptSpec to provider-specific formats
//!
//! This module implements the core translation functionality that converts
//! uniform prompt specifications into provider-specific API request formats.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

pub mod builder;
pub mod context;
pub mod lossiness;
pub mod mapper;
pub mod validator;

use crate::{
    Error, PromptSpec, ProviderSpec, Result, StrictMode, TranslationMetadata,
    TranslationResult,
};
use std::time::Instant;

pub use builder::TranslationResultBuilder;
pub use context::TranslationContext;
pub use lossiness::LossinessTracker;
pub use mapper::JSONPathMapper;
pub use validator::PreValidator;

/// Main translation function that converts a PromptSpec to provider-specific format
///
/// This function is the primary public API for the translation engine. It takes
/// a validated `PromptSpec` and converts it to a provider-specific JSON format
/// based on the `ProviderSpec` configuration.
///
/// # Arguments
///
/// * `prompt_spec` - The uniform prompt specification to translate
/// * `provider_spec` - The provider configuration and mapping rules
/// * `model_id` - The specific model ID to use from the provider's models
/// * `strict_mode` - The strictness policy for handling translation issues
///
/// # Returns
///
/// A `TranslationResult` containing:
/// - `provider_request_json`: The translated provider-specific JSON
/// - `lossiness`: Report of any deviations or limitations during translation
/// - `metadata`: Optional metadata about the translation process
///
/// # Errors
///
/// Returns an error if:
/// - The model_id is not found in the provider spec
/// - Pre-validation fails and strict_mode is Strict
/// - Translation encounters an unrecoverable error
///
/// # Example
///
/// ```no_run
/// use specado_core::{translate, PromptSpec, ProviderSpec, StrictMode};
///
/// # fn example() -> specado_core::Result<()> {
/// let prompt = PromptSpec {
///     // ... prompt configuration
/// #   model_class: "Chat".to_string(),
/// #   messages: vec![],
/// #   tools: None,
/// #   tool_choice: None,
/// #   response_format: None,
/// #   sampling: None,
/// #   limits: None,
/// #   media: None,
/// #   strict_mode: StrictMode::Warn,
/// };
///
/// let provider = ProviderSpec {
///     // ... provider configuration
/// #   spec_version: "1.0.0".to_string(),
/// #   provider: specado_core::ProviderInfo {
/// #       name: "test".to_string(),
/// #       base_url: "https://api.test.com".to_string(),
/// #       headers: std::collections::HashMap::new(),
/// #   },
/// #   models: vec![],
/// };
///
/// let result = translate(&prompt, &provider, "gpt-4", StrictMode::Warn)?;
/// # Ok(())
/// # }
/// ```
pub fn translate(
    prompt_spec: &PromptSpec,
    provider_spec: &ProviderSpec,
    model_id: &str,
    strict_mode: StrictMode,
) -> Result<TranslationResult> {
    let start_time = Instant::now();

    // Step 1: Find the model in the provider spec
    let model_spec = provider_spec
        .models
        .iter()
        .find(|m| m.id == model_id || m.aliases.as_ref().map_or(false, |a| a.contains(&model_id.to_string())))
        .ok_or_else(|| Error::Validation {
            field: "model_id".to_string(),
            message: format!("Model '{}' not found in provider '{}'", model_id, provider_spec.provider.name),
            expected: Some(format!("One of: {:?}", provider_spec.models.iter().map(|m| &m.id).collect::<Vec<_>>())),
        })?;

    // Step 2: Create translation context
    let context = TranslationContext::new(
        prompt_spec.clone(),
        provider_spec.clone(),
        model_spec.clone(),
        strict_mode,
    );

    // Step 3: Pre-validation (placeholder for issue #16)
    let validator = PreValidator::new(&context);
    validator.validate()?;

    // Step 4: Initialize lossiness tracker (placeholder for issue #18)
    let mut lossiness_tracker = LossinessTracker::new(strict_mode);

    // Step 5: Create JSONPath mapper (placeholder for issue #10)
    let _mapper = JSONPathMapper::new(&context);

    // Step 6: Build base provider request structure
    // This is a placeholder implementation - the actual mapping logic
    // will be implemented in subsequent issues
    let mut provider_request = serde_json::json!({
        "model": model_id,
        "messages": prompt_spec.messages.iter().map(|msg| {
            serde_json::json!({
                "role": msg.role,
                "content": msg.content,
            })
        }).collect::<Vec<_>>(),
    });

    // Step 7: Apply field transformations (placeholder for issue #17)
    // Future: Apply transformations based on provider mappings

    // Step 8: Handle tools if present
    if let Some(ref tools) = prompt_spec.tools {
        if model_spec.tooling.tools_supported {
            provider_request["tools"] = serde_json::json!(tools);
            
            if let Some(ref tool_choice) = prompt_spec.tool_choice {
                provider_request["tool_choice"] = serde_json::json!(tool_choice);
            }
        } else {
            // Track lossiness for unsupported tools
            lossiness_tracker.add_unsupported(
                "tools",
                "Provider does not support tools",
                Some(serde_json::json!(tools)),
            );
        }
    }

    // Step 9: Apply sampling parameters
    if let Some(ref sampling) = prompt_spec.sampling {
        if let Some(temp) = sampling.temperature {
            provider_request["temperature"] = serde_json::json!(temp);
        }
        if let Some(top_p) = sampling.top_p {
            provider_request["top_p"] = serde_json::json!(top_p);
        }
        // Add other sampling parameters as needed
    }

    // Step 10: Apply limits
    if let Some(ref limits) = prompt_spec.limits {
        if let Some(max_tokens) = limits.max_output_tokens {
            provider_request["max_tokens"] = serde_json::json!(max_tokens);
        }
    }

    // Step 11: Handle response format
    if let Some(ref format) = prompt_spec.response_format {
        if model_spec.json_output.native_param {
            provider_request["response_format"] = serde_json::json!(format);
        } else if model_spec.json_output.strategy == "system_prompt" {
            // Emulate via system prompt modification
            lossiness_tracker.add_emulated(
                "response_format",
                "JSON mode emulated via system prompt",
                Some(serde_json::json!(format)),
            );
        }
    }

    // Step 12: Apply strictness policy (placeholder for issue #19)
    // Future: Apply strictness policy engine rules

    // Step 13: Resolve conflicts (placeholder for issue #20)
    // Future: Apply conflict resolution logic

    // Step 14: Build final result (placeholder for issue #21)
    let duration_ms = start_time.elapsed().as_millis() as u64;
    
    let metadata = TranslationMetadata {
        provider: provider_spec.provider.name.clone(),
        model: model_id.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        duration_ms: Some(duration_ms),
        strict_mode,
    };

    let result = TranslationResultBuilder::new()
        .with_provider_request(provider_request)
        .with_lossiness_report(lossiness_tracker.build_report())
        .with_metadata(metadata)
        .build();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Message, MessageRole, ProviderInfo, ModelSpec, Endpoints, EndpointConfig, InputModes, ToolingConfig, JsonOutputConfig};
    use std::collections::HashMap;

    fn create_test_prompt() -> PromptSpec {
        PromptSpec {
            model_class: "Chat".to_string(),
            messages: vec![
                Message {
                    role: MessageRole::System,
                    content: "You are a helpful assistant.".to_string(),
                    name: None,
                    metadata: None,
                },
                Message {
                    role: MessageRole::User,
                    content: "Hello!".to_string(),
                    name: None,
                    metadata: None,
                },
            ],
            tools: None,
            tool_choice: None,
            response_format: None,
            sampling: None,
            limits: None,
            media: None,
            strict_mode: StrictMode::Warn,
        }
    }

    fn create_test_provider() -> ProviderSpec {
        ProviderSpec {
            spec_version: "1.0.0".to_string(),
            provider: ProviderInfo {
                name: "test-provider".to_string(),
                base_url: "https://api.test.com".to_string(),
                headers: HashMap::new(),
            },
            models: vec![
                ModelSpec {
                    id: "test-model".to_string(),
                    aliases: Some(vec!["model-alias".to_string()]),
                    family: "test-family".to_string(),
                    endpoints: Endpoints {
                        chat_completion: EndpointConfig {
                            method: "POST".to_string(),
                            path: "/v1/chat/completions".to_string(),
                            protocol: "https".to_string(),
                            query: None,
                            headers: None,
                        },
                        streaming_chat_completion: EndpointConfig {
                            method: "POST".to_string(),
                            path: "/v1/chat/completions".to_string(),
                            protocol: "https".to_string(),
                            query: None,
                            headers: None,
                        },
                    },
                    input_modes: InputModes {
                        messages: true,
                        single_text: false,
                        images: false,
                    },
                    tooling: ToolingConfig {
                        tools_supported: true,
                        parallel_tool_calls_default: true,
                        can_disable_parallel_tool_calls: false,
                        disable_switch: None,
                    },
                    json_output: JsonOutputConfig {
                        native_param: true,
                        strategy: "native".to_string(),
                    },
                    parameters: serde_json::json!({}),
                    constraints: crate::Constraints {
                        system_prompt_location: "first".to_string(),
                        forbid_unknown_top_level_fields: false,
                        mutually_exclusive: vec![],
                        resolution_preferences: vec![],
                        limits: crate::ConstraintLimits {
                            max_tool_schema_bytes: 100000,
                            max_system_prompt_bytes: 10000,
                        },
                    },
                    mappings: crate::Mappings {
                        paths: HashMap::new(),
                        flags: HashMap::new(),
                    },
                    response_normalization: crate::ResponseNormalization {
                        sync: crate::SyncNormalization {
                            content_path: "choices[0].message.content".to_string(),
                            finish_reason_path: "choices[0].finish_reason".to_string(),
                            finish_reason_map: HashMap::new(),
                        },
                        stream: crate::StreamNormalization {
                            protocol: "sse".to_string(),
                            event_selector: crate::EventSelector {
                                type_path: "object".to_string(),
                                routes: vec![],
                            },
                        },
                    },
                },
            ],
        }
    }

    #[test]
    fn test_translate_basic() {
        let prompt = create_test_prompt();
        let provider = create_test_provider();
        
        let result = translate(&prompt, &provider, "test-model", StrictMode::Warn);
        assert!(result.is_ok());
        
        let translation = result.unwrap();
        assert!(translation.provider_request_json.is_object());
        assert_eq!(
            translation.provider_request_json["model"],
            serde_json::json!("test-model")
        );
    }

    #[test]
    fn test_translate_with_alias() {
        let prompt = create_test_prompt();
        let provider = create_test_provider();
        
        let result = translate(&prompt, &provider, "model-alias", StrictMode::Warn);
        assert!(result.is_ok());
    }

    #[test]
    fn test_translate_invalid_model() {
        let prompt = create_test_prompt();
        let provider = create_test_provider();
        
        let result = translate(&prompt, &provider, "invalid-model", StrictMode::Warn);
        assert!(result.is_err());
        
        if let Err(Error::Validation { field, .. }) = result {
            assert_eq!(field, "model_id");
        } else {
            panic!("Expected Validation error");
        }
    }

    #[test]
    fn test_translate_with_metadata() {
        let prompt = create_test_prompt();
        let provider = create_test_provider();
        
        let result = translate(&prompt, &provider, "test-model", StrictMode::Strict).unwrap();
        
        assert!(result.metadata.is_some());
        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.provider, "test-provider");
        assert_eq!(metadata.model, "test-model");
        assert_eq!(metadata.strict_mode, StrictMode::Strict);
        assert!(metadata.duration_ms.is_some());
    }
}