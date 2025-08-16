//! PromptSpec validation with custom business logic rules
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::validation::base::{SchemaValidator, ValidationContext};
use crate::validation::error::{ValidationError, ValidationResult};
use serde_json::Value;
use std::path::Path;

// Embed the schema at compile time for reliability
const PROMPT_SPEC_SCHEMA: &str = include_str!("../../../../schemas/prompt-spec.schema.json");

/// PromptSpec validator with custom rules
pub struct PromptSpecValidator {
    schema: Value,
}

impl PromptSpecValidator {
    /// Create a new PromptSpec validator using embedded schema
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Try to load from environment variable first (for development)
        let schema = if let Ok(schema_path) = std::env::var("PROMPT_SPEC_SCHEMA_PATH") {
            // Load from disk if path is provided
            let path = Path::new(&schema_path);
            if path.exists() {
                let content = std::fs::read_to_string(path)?;
                serde_json::from_str(&content)?
            } else {
                // Fall back to embedded schema
                serde_json::from_str(PROMPT_SPEC_SCHEMA)?
            }
        } else {
            // Use embedded schema by default
            serde_json::from_str(PROMPT_SPEC_SCHEMA)?
        };
        
        Ok(Self { schema })
    }
    
    /// Load schema from a specific path (useful for testing)
    pub fn from_path(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let schema: Value = serde_json::from_str(&content)?;
        Ok(Self { schema })
    }

    /// Get the JSON Schema definition for PromptSpec
    pub fn schema(&self) -> &Value {
        &self.schema
    }
    
    #[allow(dead_code)]
    fn get_schema_definition() -> Value {
        serde_json::json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "https://schemas.specado.com/prompt-spec/v1.json",
            "title": "PromptSpec",
            "description": "Unified request format for LLM provider interactions",
            "type": "object",
            "properties": {
                "spec_version": {
                    "type": "string",
                    "pattern": "^\\d+\\.\\d+$",
                    "description": "Version of the PromptSpec specification"
                },
                "id": {
                    "type": "string",
                    "description": "Unique identifier for this prompt request"
                },
                "model_class": {
                    "type": "string",
                    "enum": ["Chat", "ReasoningChat", "RAGChat", "MultimodalChat", "AudioChat", "VideoChat"],
                    "description": "Classification of the model being targeted"
                },
                "messages": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "role": {
                                "type": "string",
                                "enum": ["system", "user", "assistant", "tool"]
                            },
                            "content": {
                                "oneOf": [
                                    { "type": "string" },
                                    { "type": "array" }
                                ]
                            }
                        },
                        "required": ["role", "content"]
                    },
                    "minItems": 1
                },
                "tools": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "type": { "type": "string" },
                            "function": { "type": "object" }
                        }
                    }
                },
                "tool_choice": {
                    "oneOf": [
                        { "type": "string", "enum": ["auto", "none", "required"] },
                        { "type": "object" }
                    ]
                },
                "reasoning_tokens": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 65536
                },
                "rag": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" },
                        "sources": { "type": "array" },
                        "retrieval_config": { "type": "object" }
                    }
                },
                "media": {
                    "type": "object",
                    "properties": {
                        "input_video": { "type": "boolean" },
                        "input_audio": { "type": "boolean" },
                        "output_audio": { "type": "boolean" }
                    }
                },
                "conversation": {
                    "type": "object",
                    "properties": {
                        "parent_message_id": { "type": "string" },
                        "conversation_id": { "type": "string" },
                        "metadata": { "type": "object" }
                    }
                },
                "parameters": {
                    "type": "object",
                    "properties": {
                        "temperature": { "type": "number", "minimum": 0, "maximum": 2 },
                        "max_tokens": { "type": "integer", "minimum": 1 },
                        "top_p": { "type": "number", "minimum": 0, "maximum": 1 },
                        "frequency_penalty": { "type": "number", "minimum": -2, "maximum": 2 },
                        "presence_penalty": { "type": "number", "minimum": -2, "maximum": 2 }
                    }
                },
                "strict_mode": {
                    "type": "string",
                    "enum": ["Strict", "Lenient"],
                    "default": "Lenient"
                }
            },
            "required": ["spec_version", "id", "model_class", "messages"],
            "additionalProperties": true
        })
    }

    /// Validate basic structure requirements
    fn validate_basic_structure(
        &self,
        data: &Value,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        // Check that required fields exist
        let required_fields = ["spec_version", "id", "model_class", "messages"];
        
        for field in &required_fields {
            if data.get(field).is_none() {
                return Err(ValidationError::with_violations(
                    &context.child(field).path,
                    format!("Required field {} is missing", field),
                    vec![ValidationError::create_violation(
                        "required_field",
                        format!("{} to be present", field),
                        "field is missing".to_string(),
                    )],
                ));
            }
        }

        // Validate messages array is non-empty
        if let Some(messages) = data.get("messages").and_then(|v| v.as_array()) {
            if messages.is_empty() {
                return Err(ValidationError::with_violations(
                    &context.child("messages").path,
                    "Messages array cannot be empty".to_string(),
                    vec![ValidationError::create_violation(
                        "non_empty_messages",
                        "non-empty messages array",
                        "empty array".to_string(),
                    )],
                ));
            }
        }

        Ok(())
    }

    /// Validate tool_choice requires tools array
    fn validate_tool_choice_requires_tools(
        &self,
        data: &Value,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        let tool_choice = data.get("tool_choice");
        let tools = data.get("tools");

        if let Some(_) = tool_choice {
            match tools {
                Some(tools_array) if tools_array.as_array().map_or(false, |arr| !arr.is_empty()) => {
                    Ok(())
                }
                _ => Err(ValidationError::with_violations(
                    &context.child("tool_choice").path,
                    "tool_choice requires tools array to be defined and non-empty".to_string(),
                    vec![ValidationError::create_violation(
                        "tool_choice_requires_tools",
                        "non-empty tools array when tool_choice is specified",
                        "tools array is missing or empty".to_string(),
                    )],
                )),
            }
        } else {
            Ok(())
        }
    }

    /// Validate reasoning_tokens only for ReasoningChat
    fn validate_reasoning_tokens_model_class(
        &self,
        data: &Value,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        let reasoning_tokens = data.get("reasoning_tokens");
        let model_class = data.get("model_class").and_then(|v| v.as_str());

        if let Some(_) = reasoning_tokens {
            if model_class != Some("ReasoningChat") {
                return Err(ValidationError::with_violations(
                    &context.child("reasoning_tokens").path,
                    "reasoning_tokens is only valid when model_class is 'ReasoningChat'".to_string(),
                    vec![ValidationError::create_violation(
                        "reasoning_tokens_model_class",
                        "model_class to be 'ReasoningChat'",
                        format!("model_class is '{}'", model_class.unwrap_or("undefined")),
                    )],
                ));
            }
        }

        Ok(())
    }

    /// Validate RAG configuration only for RAGChat
    fn validate_rag_model_class(
        &self,
        data: &Value,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        let rag = data.get("rag");
        let model_class = data.get("model_class").and_then(|v| v.as_str());

        if let Some(_) = rag {
            if model_class != Some("RAGChat") {
                return Err(ValidationError::with_violations(
                    &context.child("rag").path,
                    "rag configuration is only valid when model_class is 'RAGChat'".to_string(),
                    vec![ValidationError::create_violation(
                        "rag_model_class",
                        "model_class to be 'RAGChat'",
                        format!("model_class is '{}'", model_class.unwrap_or("undefined")),
                    )],
                ));
            }
        }

        Ok(())
    }

    /// Validate media input constraints
    fn validate_media_constraints(
        &self,
        data: &Value,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        let media = data.get("media");
        let model_class = data.get("model_class").and_then(|v| v.as_str());

        if let Some(media_obj) = media {
            let input_video = media_obj.get("input_video").and_then(|v| v.as_bool()).unwrap_or(false);
            let input_audio = media_obj.get("input_audio").and_then(|v| v.as_bool()).unwrap_or(false);

            // Validate input_video
            if input_video {
                let valid_for_video = matches!(model_class, Some("MultimodalChat") | Some("VideoChat"));
                if !valid_for_video {
                    return Err(ValidationError::with_violations(
                        &context.child("media").child("input_video").path,
                        "input_video is only valid for 'MultimodalChat' or 'VideoChat' model classes".to_string(),
                        vec![ValidationError::create_violation(
                            "input_video_model_class",
                            "model_class to be 'MultimodalChat' or 'VideoChat'",
                            format!("model_class is '{}'", model_class.unwrap_or("undefined")),
                        )],
                    ));
                }
            }

            // Validate input_audio
            if input_audio {
                let valid_for_audio = matches!(model_class, Some("AudioChat") | Some("MultimodalChat"));
                if !valid_for_audio {
                    return Err(ValidationError::with_violations(
                        &context.child("media").child("input_audio").path,
                        "input_audio is only valid for 'AudioChat' or 'MultimodalChat' model classes".to_string(),
                        vec![ValidationError::create_violation(
                            "input_audio_model_class",
                            "model_class to be 'AudioChat' or 'MultimodalChat'",
                            format!("model_class is '{}'", model_class.unwrap_or("undefined")),
                        )],
                    ));
                }
            }
        }

        Ok(())
    }

    /// Validate conversation parent_message_id reference
    fn validate_conversation_message_reference(
        &self,
        data: &Value,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        let conversation = data.get("conversation");
        
        if let Some(conv_obj) = conversation {
            if let Some(parent_id) = conv_obj.get("parent_message_id").and_then(|v| v.as_str()) {
                // In a real implementation, you would validate against a message store
                // For now, we just validate the format
                if parent_id.is_empty() {
                    return Err(ValidationError::with_violations(
                        &context.child("conversation").child("parent_message_id").path,
                        "parent_message_id cannot be empty".to_string(),
                        vec![ValidationError::create_violation(
                            "parent_message_id_format",
                            "non-empty message ID",
                            "empty string".to_string(),
                        )],
                    ));
                }
            }
        }

        Ok(())
    }

    /// Validate strict mode compliance
    fn validate_strict_mode(
        &self,
        data: &Value,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        let strict_mode = data.get("strict_mode").and_then(|v| v.as_str()).unwrap_or("Lenient");
        
        if strict_mode == "Strict" {
            // In strict mode, validate that no unknown fields are present
            let allowed_fields = [
                "spec_version", "id", "model_class", "messages", "tools", "tool_choice",
                "reasoning_tokens", "rag", "media", "conversation", "parameters", "strict_mode"
            ];

            if let Some(obj) = data.as_object() {
                for key in obj.keys() {
                    if !allowed_fields.contains(&key.as_str()) {
                        return Err(ValidationError::with_violations(
                            &context.child(key).path,
                            format!("Unknown field '{}' not allowed in strict mode", key),
                            vec![ValidationError::create_violation(
                                "strict_mode_unknown_field",
                                "only known fields in strict mode",
                                format!("unknown field '{}'", key),
                            )],
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

impl SchemaValidator for PromptSpecValidator {
    type Input = Value;

    fn validate_with_context(
        &self,
        input: &Self::Input,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        // Basic structural validation
        self.validate_basic_structure(input, context)?;

        // Then validate custom business rules
        match context.mode {
            crate::validation::base::ValidationMode::Basic => {
                // Only basic structure validation in basic mode
                Ok(())
            }
            crate::validation::base::ValidationMode::Partial => {
                // Run some custom validations in partial mode
                self.validate_tool_choice_requires_tools(input, context)?;
                self.validate_reasoning_tokens_model_class(input, context)?;
                Ok(())
            }
            crate::validation::base::ValidationMode::Strict => {
                // Run all custom validations in strict mode
                self.validate_tool_choice_requires_tools(input, context)?;
                self.validate_reasoning_tokens_model_class(input, context)?;
                self.validate_rag_model_class(input, context)?;
                self.validate_media_constraints(input, context)?;
                self.validate_conversation_message_reference(input, context)?;
                self.validate_strict_mode(input, context)?;
                Ok(())
            }
        }
    }
}

impl Default for PromptSpecValidator {
    fn default() -> Self {
        Self::new().expect("Failed to create PromptSpecValidator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use serde_json::json;
    
    #[test]
    fn test_embedded_schema_loading() {
        let validator = PromptSpecValidator::new().unwrap();
        let schema = validator.schema();
        assert!(schema.is_object());
        assert_eq!(schema.get("title").and_then(|v| v.as_str()), Some("PromptSpec"));
        assert_eq!(schema.get("$schema").and_then(|v| v.as_str()), Some("https://json-schema.org/draft/2020-12/schema"));
    }

    fn create_basic_prompt_spec() -> Value {
        json!({
            "spec_version": "1.0",
            "id": "test-123",
            "model_class": "Chat",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello"
                }
            ]
        })
    }

    #[test]
    fn test_valid_basic_prompt_spec() {
        let validator = PromptSpecValidator::new().unwrap();
        let spec = create_basic_prompt_spec();
        assert!(validator.validate(&spec).is_ok());
    }

    #[test]
    fn test_tool_choice_requires_tools() {
        let validator = PromptSpecValidator::new().unwrap();
        let mut spec = create_basic_prompt_spec();
        spec.as_object_mut().unwrap().insert("tool_choice".to_string(), json!("auto"));
        
        // Should fail without tools
        assert!(validator.validate(&spec).is_err());
        
        // Should pass with tools
        spec.as_object_mut().unwrap().insert("tools".to_string(), json!([{
            "type": "function",
            "function": {"name": "test"}
        }]));
        assert!(validator.validate(&spec).is_ok());
    }

    #[test]
    fn test_reasoning_tokens_model_class() {
        let validator = PromptSpecValidator::new().unwrap();
        let mut spec = create_basic_prompt_spec();
        spec.as_object_mut().unwrap().insert("reasoning_tokens".to_string(), json!(1000));
        
        // Should fail with Chat model class
        assert!(validator.validate(&spec).is_err());
        
        // Should pass with ReasoningChat
        spec.as_object_mut().unwrap().insert("model_class".to_string(), json!("ReasoningChat"));
        assert!(validator.validate(&spec).is_ok());
    }

    #[test]
    fn test_rag_model_class() {
        let validator = PromptSpecValidator::new().unwrap();
        let mut spec = create_basic_prompt_spec();
        spec.as_object_mut().unwrap().insert("rag".to_string(), json!({
            "query": "test query"
        }));
        
        // Should fail with Chat model class
        assert!(validator.validate(&spec).is_err());
        
        // Should pass with RAGChat
        spec.as_object_mut().unwrap().insert("model_class".to_string(), json!("RAGChat"));
        assert!(validator.validate(&spec).is_ok());
    }

    #[test]
    fn test_media_constraints() {
        let validator = PromptSpecValidator::new().unwrap();
        let mut spec = create_basic_prompt_spec();
        
        // Test input_video constraint
        spec.as_object_mut().unwrap().insert("media".to_string(), json!({
            "input_video": true
        }));
        assert!(validator.validate(&spec).is_err());
        
        spec.as_object_mut().unwrap().insert("model_class".to_string(), json!("VideoChat"));
        assert!(validator.validate(&spec).is_ok());
        
        // Test input_audio constraint
        spec.as_object_mut().unwrap().insert("media".to_string(), json!({
            "input_audio": true
        }));
        spec.as_object_mut().unwrap().insert("model_class".to_string(), json!("Chat"));
        assert!(validator.validate(&spec).is_err());
        
        spec.as_object_mut().unwrap().insert("model_class".to_string(), json!("AudioChat"));
        assert!(validator.validate(&spec).is_ok());
    }

    #[test]
    fn test_strict_mode_validation() {
        let validator = PromptSpecValidator::new().unwrap();
        let mut spec = create_basic_prompt_spec();
        spec.as_object_mut().unwrap().insert("strict_mode".to_string(), json!("Strict"));
        spec.as_object_mut().unwrap().insert("unknown_field".to_string(), json!("value"));
        
        // Should fail in strict mode with unknown field
        assert!(validator.validate(&spec).is_err());
        
        // Should pass in lenient mode
        spec.as_object_mut().unwrap().insert("strict_mode".to_string(), json!("Lenient"));
        assert!(validator.validate(&spec).is_ok());
    }

    #[test]
    fn test_validation_modes() {
        let validator = PromptSpecValidator::new().unwrap();
        let mut spec = create_basic_prompt_spec();
        spec.as_object_mut().unwrap().insert("reasoning_tokens".to_string(), json!(1000));
        
        // Basic mode should pass (no custom validations)
        assert!(validator.validate_basic(&spec).is_ok());
        
        // Partial and strict modes should fail
        assert!(validator.validate_partial(&spec).is_err());
        assert!(validator.validate(&spec).is_err());
    }
}