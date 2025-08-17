//! Comprehensive pre-validation logic for translation operations
//!
//! This module implements comprehensive pre-validation that runs before the main
//! translation process to catch issues early and provide detailed validation errors.
//! The validation system supports both strict and lenient modes and provides
//! provider-specific validation rules.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::{Error, Result, StrictMode};
use super::TranslationContext;

/// Validation error with detailed field path information
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field_path: String,
    pub message: String,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub severity: ValidationSeverity,
}

/// Severity levels for validation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    /// Error that will cause translation to fail
    Error,
    /// Warning that may impact translation quality
    Warning,
    /// Information about potential issues
    Info,
}

/// Validation mode configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationMode {
    /// All validation rules enforced
    Strict,
    /// Relaxed validation, warnings only
    Lenient,
}

/// Pre-validator for checking input compatibility before translation
///
/// The PreValidator performs comprehensive validation checks on the input PromptSpec
/// to ensure it can be successfully translated to the target provider format.
/// This includes checking for:
/// - Required fields based on model class
/// - Field constraints (min/max lengths, allowed values, patterns)
/// - Type constraints match expected schemas
/// - Model compatibility checks
/// - Provider-specific limitations
/// - Token count and size validation
pub struct PreValidator<'a> {
    context: &'a TranslationContext,
    validation_mode: ValidationMode,
}

impl<'a> PreValidator<'a> {
    /// Create a new pre-validator with default validation mode
    pub fn new(context: &'a TranslationContext) -> Self {
        let validation_mode = match context.strict_mode {
            StrictMode::Strict => ValidationMode::Strict,
            StrictMode::Warn | StrictMode::Coerce => ValidationMode::Lenient,
        };
        
        Self {
            context,
            validation_mode,
        }
    }
    
    /// Create a new pre-validator with explicit validation mode
    pub fn with_mode(context: &'a TranslationContext, mode: ValidationMode) -> Self {
        Self {
            context,
            validation_mode: mode,
        }
    }
    
    /// Perform comprehensive pre-validation checks
    ///
    /// This method runs all validation rules and returns detailed validation errors.
    /// Depending on the validation mode, it may return early on the first error
    /// (strict mode) or collect all errors (lenient mode).
    pub fn validate(&self) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        // Run basic structural validation
        errors.extend(self.validate_basic_structure()?);
        
        // Run field-specific validation
        errors.extend(self.validate_messages()?);
        errors.extend(self.validate_model_class()?);
        errors.extend(self.validate_sampling_params()?);
        errors.extend(self.validate_limits()?);
        errors.extend(self.validate_tools()?);
        errors.extend(self.validate_media()?);
        errors.extend(self.validate_response_format()?);
        
        // Run provider-specific validation
        errors.extend(self.validate_provider_constraints()?);
        
        // Run mutually exclusive field validation
        errors.extend(self.validate_mutually_exclusive_fields()?);
        
        // Filter errors based on validation mode
        let filtered_errors = self.filter_errors_by_mode(&errors);
        
        Ok(filtered_errors)
    }
    
    /// Validate only and return the first error if any (legacy compatibility)
    pub fn validate_strict(&self) -> Result<()> {
        let errors = self.validate()?;
        
        // Find the first error-level validation issue
        if let Some(error) = errors.iter().find(|e| e.severity == ValidationSeverity::Error) {
            return Err(Error::Validation {
                field: error.field_path.clone(),
                message: error.message.clone(),
                expected: error.expected.clone(),
            });
        }
        
        // Check for compatibility issues based on strict mode
        if self.context.should_fail_on_error() {
            if let Some(warning) = errors.iter().find(|e| e.severity == ValidationSeverity::Warning) {
                return Err(Error::Validation {
                    field: warning.field_path.clone(),
                    message: warning.message.clone(),
                    expected: warning.expected.clone(),
                });
            }
        }
        
        Ok(())
    }
    
    /// Validate basic structural requirements
    fn validate_basic_structure(&self) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        // Check that required top-level fields are present
        if self.context.prompt_spec.model_class.is_empty() {
            errors.push(ValidationError {
                field_path: "model_class".to_string(),
                message: "Model class is required".to_string(),
                expected: Some("Non-empty string".to_string()),
                actual: Some("empty".to_string()),
                severity: ValidationSeverity::Error,
            });
        }
        
        Ok(errors)
    }
    
    /// Validate messages array
    fn validate_messages(&self) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        if self.context.prompt_spec.messages.is_empty() {
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
        for (i, message) in self.context.prompt_spec.messages.iter().enumerate() {
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
        errors.extend(self.validate_system_prompt_requirements()?);
        
        Ok(errors)
    }
    
    /// Validate system prompt requirements based on provider constraints
    fn validate_system_prompt_requirements(&self) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        let system_messages: Vec<_> = self.context.prompt_spec.messages
            .iter()
            .enumerate()
            .filter(|(_, msg)| msg.role == crate::MessageRole::System)
            .collect();
        
        let system_location = &self.context.model_spec.constraints.system_prompt_location;
        
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
        let max_system_bytes = self.context.model_spec.constraints.limits.max_system_prompt_bytes;
        for (index, message) in &system_messages {
            let byte_count = message.content.as_bytes().len();
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
    fn validate_model_class(&self) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        let supported_classes = ["Chat", "ReasoningChat", "VisionChat", "AudioChat", "MultimodalChat"];
        let model_class = &self.context.prompt_spec.model_class;
        
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
    fn validate_sampling_params(&self) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        if let Some(ref sampling) = self.context.prompt_spec.sampling {
            if let Some(temp) = sampling.temperature {
                if temp < 0.0 || temp > 2.0 {
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
                if top_p < 0.0 || top_p > 1.0 {
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
                if freq_penalty < -2.0 || freq_penalty > 2.0 {
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
                if pres_penalty < -2.0 || pres_penalty > 2.0 {
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
    fn validate_limits(&self) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        if let Some(ref limits) = self.context.prompt_spec.limits {
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
                if self.context.prompt_spec.model_class != "ReasoningChat" {
                    errors.push(ValidationError {
                        field_path: "limits.reasoning_tokens".to_string(),
                        message: "Reasoning tokens are only supported for ReasoningChat model class".to_string(),
                        expected: Some("ReasoningChat model class".to_string()),
                        actual: Some(self.context.prompt_spec.model_class.clone()),
                        severity: ValidationSeverity::Warning,
                    });
                }
            }
        }
        
        Ok(errors)
    }
    
    /// Validate tools configuration
    fn validate_tools(&self) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        if let Some(ref tools) = self.context.prompt_spec.tools {
            // Check if provider supports tools
            if !self.context.supports_tools() {
                let severity = if self.context.should_fail_on_error() {
                    ValidationSeverity::Error
                } else {
                    ValidationSeverity::Warning
                };
                
                errors.push(ValidationError {
                    field_path: "tools".to_string(),
                    message: format!("Provider '{}' does not support tools", self.context.provider_name()),
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
                let max_schema_bytes = self.context.model_spec.constraints.limits.max_tool_schema_bytes;
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
            if let Some(ref tool_choice) = self.context.prompt_spec.tool_choice {
                match tool_choice {
                    crate::ToolChoice::Specific { name } => {
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
                    _ => {} // Auto and Required are always valid
                }
            }
        } else if self.context.prompt_spec.tool_choice.is_some() {
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
    fn validate_media(&self) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        if let Some(ref media) = self.context.prompt_spec.media {
            // Check image support
            if media.input_images.is_some() && !self.context.supports_images() {
                let severity = if self.context.should_fail_on_error() {
                    ValidationSeverity::Error
                } else {
                    ValidationSeverity::Warning
                };
                
                errors.push(ValidationError {
                    field_path: "media.input_images".to_string(),
                    message: format!("Model '{}' does not support image inputs", self.context.model_id()),
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
    fn validate_response_format(&self) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        if let Some(ref format) = self.context.prompt_spec.response_format {
            match format {
                crate::ResponseFormat::JsonSchema { json_schema, strict } => {
                    // Check if provider supports JSON schema
                    if !self.context.supports_native_json() && self.context.model_spec.json_output.strategy != "system_prompt" {
                        let severity = if self.context.should_fail_on_error() {
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
                    if strict == &Some(true) && !self.context.supports_native_json() {
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
                    if !self.context.supports_native_json() && self.context.model_spec.json_output.strategy != "system_prompt" {
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
    
    /// Validate provider-specific constraints
    fn validate_provider_constraints(&self) -> Result<Vec<ValidationError>> {
        let errors = Vec::new();
        
        // Check for unknown top-level fields if provider forbids them
        if self.context.model_spec.constraints.forbid_unknown_top_level_fields {
            // This would require introspection of the JSON structure
            // For now, we'll add a placeholder that could be implemented
            // when we have more detailed provider specifications
        }
        
        Ok(errors)
    }
    
    /// Validate mutually exclusive field combinations
    fn validate_mutually_exclusive_fields(&self) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        for exclusive_group in &self.context.model_spec.constraints.mutually_exclusive {
            let present_fields: Vec<_> = exclusive_group.iter()
                .filter(|field| self.is_field_present(field))
                .collect();
            
            if present_fields.len() > 1 {
                let present_str = present_fields.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
                let expected_str = exclusive_group.join(", ");
                
                errors.push(ValidationError {
                    field_path: "constraint_violation".to_string(),
                    message: format!(
                        "Mutually exclusive fields detected: {}",
                        present_str
                    ),
                    expected: Some(format!("Only one of: {}", expected_str)),
                    actual: Some(format!("Present: {}", present_str)),
                    severity: ValidationSeverity::Error,
                });
            }
        }
        
        Ok(errors)
    }
    
    /// Check if a field is present in the prompt spec
    fn is_field_present(&self, field_path: &str) -> bool {
        match field_path {
            "tools" => self.context.prompt_spec.tools.is_some(),
            "tool_choice" => self.context.prompt_spec.tool_choice.is_some(),
            "response_format" => self.context.prompt_spec.response_format.is_some(),
            "sampling" => self.context.prompt_spec.sampling.is_some(),
            "limits" => self.context.prompt_spec.limits.is_some(),
            "media" => self.context.prompt_spec.media.is_some(),
            _ => false, // For more complex paths, we'd need JSONPath evaluation
        }
    }
    
    /// Filter validation errors based on the current validation mode
    fn filter_errors_by_mode(&self, errors: &[ValidationError]) -> Vec<ValidationError> {
        match self.validation_mode {
            ValidationMode::Strict => {
                // Return all errors and warnings
                errors.to_vec()
            }
            ValidationMode::Lenient => {
                // Return only errors, filter out warnings and info
                errors.iter()
                    .filter(|e| e.severity == ValidationSeverity::Error)
                    .cloned()
                    .collect()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Constraints, ConstraintLimits, EndpointConfig, Endpoints, InputModes, JsonOutputConfig,
        Mappings, Message, MessageRole, ProviderInfo, ResponseNormalization, StreamNormalization,
        SyncNormalization, ToolingConfig, EventSelector, ProviderSpec, ModelSpec, PromptSpec,
        Tool, Limits, MediaConfig, SamplingParams,
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