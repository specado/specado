//! Translation engine for converting PromptSpec to provider-specific formats
//!
//! This module implements the core translation functionality that converts
//! uniform prompt specifications into provider-specific API request formats.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

pub mod builder;
pub mod conflict;
pub mod context;
pub mod jsonpath;
pub mod lossiness;
pub mod mapper;
pub mod provider_loader;
pub mod strictness;
pub mod transformer;
pub mod validator;

#[cfg(test)]
pub mod provider_spec_tests;

use crate::{
    Error, PromptSpec, ProviderSpec, Result, StrictMode, TranslationMetadata,
    TranslationResult,
};
use std::time::Instant;
use std::sync::{Arc, Mutex};

pub use builder::{TranslationResultBuilder, BuilderState, BuilderError, ProviderRequestBuilder};
pub use conflict::{ConflictResolver, FieldConflict, ResolutionStrategy, ConflictResolutionConfig};
pub use context::TranslationContext;
pub use lossiness::LossinessTracker;
pub use mapper::JSONPathMapper;
pub use strictness::{StrictnessAction, StrictnessPolicy, PolicyResult};
pub use transformer::{
    TransformationPipeline, TransformationRule, TransformationRuleBuilder,
    TransformationType, TransformationDirection, TransformationError,
    TransformationContext, ValueType, ConversionFormula, Condition,
};
pub use validator::{PreValidator, ValidationError, ValidationSeverity, ValidationMode};

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
/// let result = translate(&prompt, &provider, "gpt-5", StrictMode::Warn)?;
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
        .find(|m| m.id == model_id || m.aliases.as_ref().is_some_and(|a| a.contains(&model_id.to_string())))
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

    // Step 3: Pre-validation - comprehensive validation with detailed error reporting
    let validator = PreValidator::new(&context);
    validator.validate_strict()?;

    // Step 4: Create shared lossiness tracker for issue #18
    let lossiness_tracker = Arc::new(Mutex::new(LossinessTracker::new(strict_mode)));

    // Step 5: Initialize strictness policy engine
    let strictness_policy = StrictnessPolicy::new(context.clone());

    // Step 6: Create JSONPath mapper with lossiness tracking (issue #18)
    let mut mapper = JSONPathMapper::new(&context);

    // Step 7: Build base provider request structure
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

    // Step 7.5: Apply JSONPath mappings with lossiness tracking
    // This demonstrates integration of mapper with tracking
    let prompt_as_json = serde_json::to_value(prompt_spec).unwrap_or_default();
    if let Ok(mapped_fields) = mapper.apply_mappings_with_tracker(&prompt_as_json, Some(&lossiness_tracker)) {
        // Merge mapped fields into provider request
        if let (serde_json::Value::Object(ref mut request_obj), serde_json::Value::Object(mapped_obj)) = 
            (&mut provider_request, mapped_fields) {
            for (key, value) in mapped_obj {
                request_obj.insert(key, value);
            }
        }
    }

    // Step 8: Apply field transformations using the transformation pipeline
    let mut transformation_pipeline = TransformationPipeline::new();
    
    // Build transformation pipeline from provider mappings
    // In a real implementation, this would be configured from the provider spec
    // For now, create some example transformations based on common provider needs
    
    // Add temperature scaling if needed (example: OpenAI uses 0-2, Anthropic uses 0-1)
    if context.provider_name() == "anthropic" {
        if let Ok(temp_rule) = TransformationRuleBuilder::new("temperature_scale", "$.temperature")
            .transformation(TransformationType::UnitConversion {
                from_unit: "openai_range".to_string(),
                to_unit: "anthropic_range".to_string(),
                formula: ConversionFormula::Linear { scale: 0.5, offset: 0.0 },
            })
            .direction(TransformationDirection::Forward)
            .priority(10)
            .optional()
            .build() {
            transformation_pipeline = transformation_pipeline.add_rule(temp_rule);
        }
    }
    
    // Apply transformations to the provider request
    provider_request = transformation_pipeline
        .transform(&provider_request, TransformationDirection::Forward, &context)
        .unwrap_or(provider_request); // Fall back to original on error

    // Step 9: Handle tools if present
    if let Some(ref tools) = prompt_spec.tools {
        if model_spec.tooling.tools_supported {
            provider_request["tools"] = serde_json::json!(tools);
            
            if let Some(ref tool_choice) = prompt_spec.tool_choice {
                provider_request["tool_choice"] = serde_json::json!(tool_choice);
            }
        } else {
            // Track field dropped due to provider limitations
            mapper.track_field_dropped_due_to_provider(
                "$.tools",
                Some(serde_json::json!(tools)),
                &format!("Provider {} doesn't support tools", context.provider_name()),
                Some(&lossiness_tracker),
            );
            
            // Use strictness policy to handle unsupported tools
            let policy_result = strictness_policy.evaluate_unsupported_feature(
                "tools",
                "function_calling",
                Some(serde_json::json!(tools)),
            );
            
            // Add lossiness item if provided
            if let Some(lossiness_item) = policy_result.lossiness_item {
                if let Ok(mut tracker) = lossiness_tracker.lock() {
                    tracker.add_item(lossiness_item);
                }
            }
            
            // Handle the policy action
            match policy_result.action {
                StrictnessAction::Fail { error } => return Err(error),
                StrictnessAction::Warn { message } => {
                    log::warn!("{}", message);
                }
                StrictnessAction::Proceed | StrictnessAction::Coerce { .. } => {
                    // Continue processing - tools will be dropped
                }
            }
        }
    }

    // Step 10: Apply sampling parameters
    if let Some(ref sampling) = prompt_spec.sampling {
        if let Some(temp) = sampling.temperature {
            // Use strictness policy to validate temperature range
            let policy_result = strictness_policy.evaluate_value_clamping(
                "temperature",
                serde_json::json!(temp),
                0.0,
                2.0,
                context.provider_name(),
            );
            
            match policy_result.action {
                StrictnessAction::Coerce { adjusted_value, .. } => {
                    provider_request["temperature"] = adjusted_value;
                }
                StrictnessAction::Proceed => {
                    provider_request["temperature"] = serde_json::json!(temp);
                }
                StrictnessAction::Warn { message } => {
                    log::warn!("{}", message);
                    provider_request["temperature"] = serde_json::json!(temp);
                }
                StrictnessAction::Fail { error } => return Err(error),
            }
            
            if let Some(lossiness_item) = policy_result.lossiness_item {
                if let Ok(mut tracker) = lossiness_tracker.lock() {
                    tracker.add_item(lossiness_item);
                }
            }
        }
        if let Some(top_p) = sampling.top_p {
            provider_request["top_p"] = serde_json::json!(top_p);
        }
        if let Some(top_k) = sampling.top_k {
            provider_request["top_k"] = serde_json::json!(top_k);
        }
        if let Some(freq_penalty) = sampling.frequency_penalty {
            provider_request["frequency_penalty"] = serde_json::json!(freq_penalty);
        }
        if let Some(pres_penalty) = sampling.presence_penalty {
            provider_request["presence_penalty"] = serde_json::json!(pres_penalty);
        }
    }

    // Step 11: Apply limits
    if let Some(ref limits) = prompt_spec.limits {
        if let Some(max_tokens) = limits.max_output_tokens {
            provider_request["max_tokens"] = serde_json::json!(max_tokens);
        }
    }

    // Step 12: Handle response format
    if let Some(ref format) = prompt_spec.response_format {
        if model_spec.json_output.native_param {
            provider_request["response_format"] = serde_json::json!(format);
        } else if model_spec.json_output.strategy == "system_prompt" {
            // Use strictness policy for feature emulation
            let policy_result = strictness_policy.evaluate_feature_emulation(
                "response_format",
                "JSON mode",
                "system prompt modification",
                Some(serde_json::json!(format)),
            );
            
            // Add lossiness item if provided
            if let Some(lossiness_item) = policy_result.lossiness_item {
                if let Ok(mut tracker) = lossiness_tracker.lock() {
                    tracker.add_item(lossiness_item);
                }
            }
            
            // Handle the policy action
            match policy_result.action {
                StrictnessAction::Proceed => {
                    // Emulate via system prompt - actual implementation would modify system prompt
                }
                StrictnessAction::Warn { message } => {
                    log::warn!("{}", message);
                    // Still proceed with emulation
                }
                StrictnessAction::Fail { error } => return Err(error),
                StrictnessAction::Coerce { .. } => {
                    // Proceed with emulation
                }
            }
        } else {
            // Response format not supported at all
            let policy_result = strictness_policy.evaluate_unsupported_feature(
                "response_format",
                "structured_output",
                Some(serde_json::json!(format)),
            );
            
            if let Some(lossiness_item) = policy_result.lossiness_item {
                if let Ok(mut tracker) = lossiness_tracker.lock() {
                    tracker.add_item(lossiness_item);
                }
            }
            
            match policy_result.action {
                StrictnessAction::Fail { error } => return Err(error),
                StrictnessAction::Warn { message } => {
                    log::warn!("{}", message);
                }
                StrictnessAction::Proceed | StrictnessAction::Coerce { .. } => {
                    // Continue processing - response format will be dropped
                }
            }
        }
    }

    // Step 13: Apply strictness policy evaluation
    // Check if we should proceed based on accumulated lossiness
    if let Ok(tracker) = lossiness_tracker.lock() {
        strictness_policy.evaluate_proceeding(&tracker)?;
    }

    // Step 14: Resolve conflicts using the conflict resolution system (issue #20)
    let conflict_resolver = ConflictResolver::new(context.clone());
    let conflicts = conflict_resolver.resolve_conflicts(&mut provider_request, Some(&lossiness_tracker))?;
    
    // Log resolved conflicts if any
    if !conflicts.is_empty() {
        log::info!("Resolved {} field conflicts during translation", conflicts.len());
        for conflict in &conflicts {
            if let Some(winner) = &conflict.winner {
                log::debug!("  - Kept '{}', dropped {:?}", winner, conflict.losers);
            }
        }
    }

    // Step 15: Build final result
    let duration_ms = start_time.elapsed().as_millis() as u64;
    
    let metadata = TranslationMetadata {
        provider: provider_spec.provider.name.clone(),
        model: model_id.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        duration_ms: Some(duration_ms),
        strict_mode,
    };

    // Build final result with lossiness tracking
    let result = if let Ok(tracker) = Arc::try_unwrap(lossiness_tracker) {
        // If we can unwrap the Arc, consume the tracker to build the report
        let tracker = tracker.into_inner().map_err(|_| Error::Translation {
            message: "Failed to access lossiness tracker".to_string(),
            context: None,
        })?;
        
        TranslationResultBuilder::new()
            .with_provider_request(provider_request)
            .with_lossiness_report(tracker.build_report())
            .with_metadata(metadata)
            .build()
            .map_err(|_| Error::Translation {
                message: "Failed to build translation result".to_string(),
                context: None,
            })?
    } else {
        // Fallback: create result without consuming the tracker
        TranslationResultBuilder::new()
            .with_provider_request(provider_request)
            .with_metadata(metadata)
            .build()
            .map_err(|_| Error::Translation {
                message: "Failed to build translation result".to_string(),
                context: None,
            })?
    };

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
                    capabilities: None,
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

    #[test]
    fn test_translate_with_lossiness_tracking() {
        let mut prompt = create_test_prompt();
        let provider = create_test_provider();
        
        // Add tools to test tracking when they're not supported
        use crate::Tool;
        prompt.tools = Some(vec![Tool {
            name: "test_function".to_string(),
            description: Some("A test function".to_string()),
            json_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }]);
        
        // Use a provider that doesn't support tools for this test
        let mut test_provider = provider;
        test_provider.models[0].tooling.tools_supported = false;
        
        let result = translate(&prompt, &test_provider, "test-model", StrictMode::Warn).unwrap();
        
        // Should have lossiness due to dropped tools
        assert!(result.has_lossiness());
        assert!(!result.lossiness.items.is_empty());
        
        // Check that metadata includes timing
        assert!(result.metadata.is_some());
        assert!(result.duration_ms().is_some());
    }

    #[test]
    fn test_translate_with_conflict_resolution() {
        let mut prompt = create_test_prompt();
        let mut provider = create_test_provider();
        
        // Add mutually exclusive fields to the provider constraints
        provider.models[0].constraints.mutually_exclusive = vec![
            vec!["temperature".to_string(), "top_k".to_string()],
        ];
        provider.models[0].constraints.resolution_preferences = vec![
            "temperature".to_string(),
        ];
        
        // Add sampling parameters that conflict
        use crate::SamplingParams;
        prompt.sampling = Some(SamplingParams {
            temperature: Some(0.7),
            top_p: Some(0.9),
            top_k: Some(40),  // This conflicts with temperature
            frequency_penalty: None,
            presence_penalty: None,
        });
        
        let result = translate(&prompt, &provider, "test-model", StrictMode::Warn).unwrap();
        
        // Check that the request was built successfully
        assert!(result.provider_request_json.is_object());
        
        // Temperature should be present (winner based on resolution_preferences)
        assert!(result.provider_request_json["temperature"].is_number());
        
        // top_k should NOT be present (loser in the conflict)
        assert!(result.provider_request_json.get("top_k").is_none());
        
        // top_p should still be present (not in conflict)
        assert!(result.provider_request_json["top_p"].is_number());
        
        // Should have lossiness due to the dropped field
        assert!(result.has_lossiness());
        let lossiness_items: Vec<_> = result.lossiness.items.iter()
            .filter(|item| item.path.contains("top_k"))
            .collect();
        assert!(!lossiness_items.is_empty(), "Should have lossiness for dropped top_k field");
    }
}
