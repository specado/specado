//! Field-specific validation logic
//!
//! This module contains validation functions for specific fields in the PromptSpec,
//! including messages, model class, sampling parameters, limits, tools, media, and
//! response format validation.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::{Result, MessageRole};
use super::{ValidationError, ValidationSeverity};
use super::super::TranslationContext;

/// Validate messages array
pub fn validate_messages(context: &TranslationContext) -> Result<Vec<ValidationError>> {
    let mut errors = Vec::new();
    
    if context.prompt_spec.messages.is_empty() {
        errors.push(ValidationError {
            field_path: "messages".to_string(),
            message: "Messages array cannot be empty".to_string(),
            expected: Some("At least one message".to_string()),
            actual: Some("empty array".to_string()),
            severity: ValidationSeverity::Error,
        });
        return Ok(errors);
    }
    
    // Validate individual messages
    for (i, message) in context.prompt_spec.messages.iter().enumerate() {
        let field_prefix = format!("messages[{}]", i);
        
        // Check message content is not empty
        if message.content.trim().is_empty() {
            errors.push(ValidationError {
                field_path: format!("{}.content", field_prefix),
                message: "Message content cannot be empty".to_string(),
                expected: Some("Non-empty string".to_string()),
                actual: Some("empty or whitespace only".to_string()),
                severity: ValidationSeverity::Warning,
            });
        }
        
        // Check message content length against constraints
        let max_content_length = 1_000_000; // 1MB character limit
        if message.content.len() > max_content_length {
            errors.push(ValidationError {
                field_path: format!("{}.content", field_prefix),
                message: format!("Message content exceeds maximum length of {} characters", max_content_length),
                expected: Some(format!("≤ {} characters", max_content_length)),
                actual: Some(format!("{} characters", message.content.len())),
                severity: ValidationSeverity::Error,
            });
        }
    }
    
    // Validate system prompt requirements
    errors.extend(validate_system_prompt_requirements(context)?);
    
    Ok(errors)
}

/// Validate system prompt requirements based on provider constraints
fn validate_system_prompt_requirements(context: &TranslationContext) -> Result<Vec<ValidationError>> {
    let mut errors = Vec::new();
    
    let system_messages: Vec<_> = context.prompt_spec.messages
        .iter()
        .enumerate()
        .filter(|(_, msg)| msg.role == MessageRole::System)
        .collect();
    
    let system_location = &context.model_spec.constraints.system_prompt_location;
    
    match system_location.as_str() {
        "first" => {
            if !system_messages.is_empty() {
                let (index, _) = system_messages[0];
                if index != 0 {
                    errors.push(ValidationError {
                        field_path: format!("messages[{}]", index),
                        message: "System message must be the first message for this provider".to_string(),
                        expected: Some("System message at index 0".to_string()),
                        actual: Some(format!("System message at index {}", index)),
                        severity: ValidationSeverity::Error,
                    });
                }
            }
        }
        "any" => {
            // System messages can be anywhere
        }
        "none" => {
            if !system_messages.is_empty() {
                errors.push(ValidationError {
                    field_path: "messages".to_string(),
                    message: "This provider does not support system messages".to_string(),
                    expected: Some("No system messages".to_string()),
                    actual: Some(format!("{} system message(s)", system_messages.len())),
                    severity: ValidationSeverity::Error,
                });
            }
        }
        _ => {}
    }
    
    // Check system prompt size limits
    let max_system_bytes = context.model_spec.constraints.limits.max_system_prompt_bytes;
    for (index, message) in &system_messages {
        let byte_count = message.content.len();
        if byte_count > max_system_bytes as usize {
            errors.push(ValidationError {
                field_path: format!("messages[{}].content", index),
                message: format!("System prompt exceeds maximum size of {} bytes", max_system_bytes),
                expected: Some(format!("≤ {} bytes", max_system_bytes)),
                actual: Some(format!("{} bytes", byte_count)),
                severity: ValidationSeverity::Error,
            });
        }
    }
    
    Ok(errors)
}

/// Validate model class
pub fn validate_model_class(context: &TranslationContext) -> Result<Vec<ValidationError>> {
    let mut errors = Vec::new();
    
    let supported_classes = ["Chat", "ReasoningChat", "VisionChat", "AudioChat", "MultimodalChat"];
    let model_class = &context.prompt_spec.model_class;
    
    if !supported_classes.contains(&model_class.as_str()) {
        errors.push(ValidationError {
            field_path: "model_class".to_string(),
            message: format!("Model class '{}' is not supported", model_class),
            expected: Some(format!("One of: {:?}", supported_classes)),
            actual: Some(model_class.clone()),
            severity: ValidationSeverity::Error,
        });
    }
    
    Ok(errors)
}

/// Validate sampling parameters
pub fn validate_sampling_params(context: &TranslationContext) -> Result<Vec<ValidationError>> {
    let mut errors = Vec::new();
    
    if let Some(ref sampling) = context.prompt_spec.sampling {
        if let Some(temp) = sampling.temperature {
            if !(0.0..=2.0).contains(&temp) {
                errors.push(ValidationError {
                    field_path: "sampling.temperature".to_string(),
                    message: "Temperature must be between 0.0 and 2.0".to_string(),
                    expected: Some("0.0 ≤ temperature ≤ 2.0".to_string()),
                    actual: Some(temp.to_string()),
                    severity: ValidationSeverity::Warning,
                });
            }
        }
        
        if let Some(top_p) = sampling.top_p {
            if !(0.0..=1.0).contains(&top_p) {
                errors.push(ValidationError {
                    field_path: "sampling.top_p".to_string(),
                    message: "Top-p must be between 0.0 and 1.0".to_string(),
                    expected: Some("0.0 ≤ top_p ≤ 1.0".to_string()),
                    actual: Some(top_p.to_string()),
                    severity: ValidationSeverity::Warning,
                });
            }
        }
        
        if let Some(freq_penalty) = sampling.frequency_penalty {
            if !(-2.0..=2.0).contains(&freq_penalty) {
                errors.push(ValidationError {
                    field_path: "sampling.frequency_penalty".to_string(),
                    message: "Frequency penalty must be between -2.0 and 2.0".to_string(),
                    expected: Some("-2.0 ≤ frequency_penalty ≤ 2.0".to_string()),
                    actual: Some(freq_penalty.to_string()),
                    severity: ValidationSeverity::Warning,
                });
            }
        }
        
        if let Some(pres_penalty) = sampling.presence_penalty {
            if !(-2.0..=2.0).contains(&pres_penalty) {
                errors.push(ValidationError {
                    field_path: "sampling.presence_penalty".to_string(),
                    message: "Presence penalty must be between -2.0 and 2.0".to_string(),
                    expected: Some("-2.0 ≤ presence_penalty ≤ 2.0".to_string()),
                    actual: Some(pres_penalty.to_string()),
                    severity: ValidationSeverity::Warning,
                });
            }
        }
    }
    
    Ok(errors)
}

/// Validate token and output limits
pub fn validate_limits(context: &TranslationContext) -> Result<Vec<ValidationError>> {
    let mut errors = Vec::new();
    
    if let Some(ref limits) = context.prompt_spec.limits {
        if let Some(max_output) = limits.max_output_tokens {
            if max_output == 0 {
                errors.push(ValidationError {
                    field_path: "limits.max_output_tokens".to_string(),
                    message: "Max output tokens must be greater than 0".to_string(),
                    expected: Some("Positive integer".to_string()),
                    actual: Some("0".to_string()),
                    severity: ValidationSeverity::Error,
                });
            }
            
            // Check against provider limits (if specified in model spec)
            let provider_max = 32768; // Default reasonable limit
            if max_output > provider_max {
                errors.push(ValidationError {
                    field_path: "limits.max_output_tokens".to_string(),
                    message: format!("Max output tokens exceeds provider limit of {}", provider_max),
                    expected: Some(format!("≤ {}", provider_max)),
                    actual: Some(max_output.to_string()),
                    severity: ValidationSeverity::Warning,
                });
            }
        }
        
        if let Some(max_prompt) = limits.max_prompt_tokens {
            if max_prompt == 0 {
                errors.push(ValidationError {
                    field_path: "limits.max_prompt_tokens".to_string(),
                    message: "Max prompt tokens must be greater than 0".to_string(),
                    expected: Some("Positive integer".to_string()),
                    actual: Some("0".to_string()),
                    severity: ValidationSeverity::Error,
                });
            }
        }
        
        if let Some(reasoning) = limits.reasoning_tokens {
            if reasoning == 0 {
                errors.push(ValidationError {
                    field_path: "limits.reasoning_tokens".to_string(),
                    message: "Reasoning tokens must be greater than 0".to_string(),
                    expected: Some("Positive integer".to_string()),
                    actual: Some("0".to_string()),
                    severity: ValidationSeverity::Error,
                });
            }
            
            // Check if model supports reasoning tokens
            if context.prompt_spec.model_class != "ReasoningChat" {
                errors.push(ValidationError {
                    field_path: "limits.reasoning_tokens".to_string(),
                    message: "Reasoning tokens are only supported for ReasoningChat model class".to_string(),
                    expected: Some("ReasoningChat model class".to_string()),
                    actual: Some(context.prompt_spec.model_class.clone()),
                    severity: ValidationSeverity::Warning,
                });
            }
        }
    }
    
    Ok(errors)
}

/// Validate tools configuration
pub fn validate_tools(context: &TranslationContext) -> Result<Vec<ValidationError>> {
    let mut errors = Vec::new();
    
    if let Some(ref tools) = context.prompt_spec.tools {
        // Check if provider supports tools
        if !context.supports_tools() {
            let severity = if context.should_fail_on_error() {
                ValidationSeverity::Error
            } else {
                ValidationSeverity::Warning
            };
            
            errors.push(ValidationError {
                field_path: "tools".to_string(),
                message: format!("Provider '{}' does not support tools", context.provider_name()),
                expected: Some("Provider with tool support".to_string()),
                actual: Some("Provider without tool support".to_string()),
                severity,
            });
        }
        
        // Validate individual tools
        for (i, tool) in tools.iter().enumerate() {
            let field_prefix = format!("tools[{}]", i);
            
            // Validate tool name
            if tool.name.trim().is_empty() {
                errors.push(ValidationError {
                    field_path: format!("{}.name", field_prefix),
                    message: "Tool name cannot be empty".to_string(),
                    expected: Some("Non-empty string".to_string()),
                    actual: Some("empty".to_string()),
                    severity: ValidationSeverity::Error,
                });
            }
            
            // Validate tool name format (alphanumeric + underscores)
            if !tool.name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                errors.push(ValidationError {
                    field_path: format!("{}.name", field_prefix),
                    message: "Tool name must contain only alphanumeric characters and underscores".to_string(),
                    expected: Some("Alphanumeric string with underscores".to_string()),
                    actual: Some(tool.name.clone()),
                    severity: ValidationSeverity::Warning,
                });
            }
            
            // Check tool schema size
            let schema_size = serde_json::to_string(&tool.json_schema)?.len();
            let max_schema_bytes = context.model_spec.constraints.limits.max_tool_schema_bytes;
            if schema_size > max_schema_bytes as usize {
                errors.push(ValidationError {
                    field_path: format!("{}.json_schema", field_prefix),
                    message: format!("Tool schema exceeds maximum size of {} bytes", max_schema_bytes),
                    expected: Some(format!("≤ {} bytes", max_schema_bytes)),
                    actual: Some(format!("{} bytes", schema_size)),
                    severity: ValidationSeverity::Error,
                });
            }
        }
        
        // Validate tool_choice if present
        if let Some(crate::ToolChoice::Specific { name }) = &context.prompt_spec.tool_choice {
            // Check that the specified tool exists
            if !tools.iter().any(|t| t.name == *name) {
                errors.push(ValidationError {
                    field_path: "tool_choice".to_string(),
                    message: format!("Tool choice '{}' does not match any available tool", name),
                    expected: Some(format!("One of: {:?}", tools.iter().map(|t| &t.name).collect::<Vec<_>>())),
                    actual: Some(name.clone()),
                    severity: ValidationSeverity::Error,
                });
            }
        }
    } else if context.prompt_spec.tool_choice.is_some() {
        // tool_choice specified but no tools provided
        errors.push(ValidationError {
            field_path: "tool_choice".to_string(),
            message: "Tool choice specified but no tools provided".to_string(),
            expected: Some("Tools array with at least one tool".to_string()),
            actual: Some("No tools".to_string()),
            severity: ValidationSeverity::Error,
        });
    }
    
    Ok(errors)
}

/// Validate media configuration
pub fn validate_media(context: &TranslationContext) -> Result<Vec<ValidationError>> {
    let mut errors = Vec::new();
    
    if let Some(ref media) = context.prompt_spec.media {
        // Check image support
        if media.input_images.is_some() && !context.supports_images() {
            let severity = if context.should_fail_on_error() {
                ValidationSeverity::Error
            } else {
                ValidationSeverity::Warning
            };
            
            errors.push(ValidationError {
                field_path: "media.input_images".to_string(),
                message: format!("Model '{}' does not support image inputs", context.model_id()),
                expected: Some("Model with image support".to_string()),
                actual: Some("Model without image support".to_string()),
                severity,
            });
        }
        
        // Validate image array if present
        if let Some(ref images) = media.input_images {
            if images.is_empty() {
                errors.push(ValidationError {
                    field_path: "media.input_images".to_string(),
                    message: "Input images array cannot be empty if specified".to_string(),
                    expected: Some("At least one image".to_string()),
                    actual: Some("Empty array".to_string()),
                    severity: ValidationSeverity::Warning,
                });
            }
            
            // Check image count limits
            let max_images = 20; // Reasonable default limit
            if images.len() > max_images {
                errors.push(ValidationError {
                    field_path: "media.input_images".to_string(),
                    message: format!("Too many input images. Maximum {} supported", max_images),
                    expected: Some(format!("≤ {} images", max_images)),
                    actual: Some(format!("{} images", images.len())),
                    severity: ValidationSeverity::Warning,
                });
            }
        }
        
        // Check audio support
        if media.input_audio.is_some() {
            // Most providers don't support audio yet
            errors.push(ValidationError {
                field_path: "media.input_audio".to_string(),
                message: "Audio input is not yet supported by most providers".to_string(),
                expected: Some("Provider with audio support".to_string()),
                actual: Some("Audio input specified".to_string()),
                severity: ValidationSeverity::Warning,
            });
        }
        
        if media.output_audio.is_some() {
            // Most providers don't support audio output
            errors.push(ValidationError {
                field_path: "media.output_audio".to_string(),
                message: "Audio output is not yet supported by most providers".to_string(),
                expected: Some("Provider with audio output support".to_string()),
                actual: Some("Audio output specified".to_string()),
                severity: ValidationSeverity::Warning,
            });
        }
    }
    
    Ok(errors)
}

/// Validate response format configuration
pub fn validate_response_format(context: &TranslationContext) -> Result<Vec<ValidationError>> {
    let mut errors = Vec::new();
    
    if let Some(ref format) = context.prompt_spec.response_format {
        match format {
            crate::ResponseFormat::JsonSchema { json_schema, strict } => {
                // Check if provider supports JSON schema
                if !context.supports_native_json() && context.model_spec.json_output.strategy != "system_prompt" {
                    let severity = if context.should_fail_on_error() {
                        ValidationSeverity::Error
                    } else {
                        ValidationSeverity::Warning
                    };
                    
                    errors.push(ValidationError {
                        field_path: "response_format".to_string(),
                        message: "JSON schema response format is not supported by this provider".to_string(),
                        expected: Some("Provider with JSON schema support".to_string()),
                        actual: Some("Provider without JSON schema support".to_string()),
                        severity,
                    });
                }
                
                // Validate schema is valid JSON
                if json_schema.is_null() {
                    errors.push(ValidationError {
                        field_path: "response_format.json_schema".to_string(),
                        message: "JSON schema cannot be null".to_string(),
                        expected: Some("Valid JSON schema object".to_string()),
                        actual: Some("null".to_string()),
                        severity: ValidationSeverity::Error,
                    });
                }
                
                // Check strict mode compatibility
                if strict == &Some(true) && !context.supports_native_json() {
                    errors.push(ValidationError {
                        field_path: "response_format.strict".to_string(),
                        message: "Strict JSON mode requires native JSON support".to_string(),
                        expected: Some("Provider with native JSON support".to_string()),
                        actual: Some("Provider without native JSON support".to_string()),
                        severity: ValidationSeverity::Warning,
                    });
                }
            }
            crate::ResponseFormat::JsonObject => {
                if !context.supports_native_json() && context.model_spec.json_output.strategy != "system_prompt" {
                    errors.push(ValidationError {
                        field_path: "response_format".to_string(),
                        message: "JSON object response format is not supported by this provider".to_string(),
                        expected: Some("Provider with JSON support".to_string()),
                        actual: Some("Provider without JSON support".to_string()),
                        severity: ValidationSeverity::Warning,
                    });
                }
            }
            crate::ResponseFormat::Text => {
                // Text format is always supported
            }
        }
    }
    
    Ok(errors)
}