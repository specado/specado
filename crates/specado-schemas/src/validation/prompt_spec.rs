//! PromptSpec validation with custom business logic rules
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::loader::{SchemaLoader, LoaderConfig};
use crate::validation::base::{SchemaValidator, ValidationContext, ValidationMode};
use crate::validation::error::{ValidationError, ValidationResult};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// PromptSpec validator with custom rules
pub struct PromptSpecValidator {
    schema: Arc<Value>,
}

impl PromptSpecValidator {
    /// Create a new PromptSpec validator by loading the schema file
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let schema_path = Self::resolve_schema_path()?;
        // Create loader config that doesn't validate structure (for JSON Schema files)
        let config = LoaderConfig {
            validate_basic_structure: false,
            allow_env_expansion: false,  // Don't expand env vars in schema definitions
            ..LoaderConfig::default()
        };
        let mut loader = SchemaLoader::with_config(config);
        let schema = loader.load_schema(&schema_path)?;
        Ok(Self {
            schema: Arc::new(schema),
        })
    }
    
    /// Load validator with a specific schema path
    pub fn from_path(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        // Create loader config that doesn't validate structure (for JSON Schema files)
        let config = LoaderConfig {
            validate_basic_structure: false,
            allow_env_expansion: false,  // Don't expand env vars in schema definitions
            ..LoaderConfig::default()
        };
        let mut loader = SchemaLoader::with_config(config);
        let schema = loader.load_schema(path)?;
        Ok(Self {
            schema: Arc::new(schema),
        })
    }
    
    /// Resolve the path to the prompt-spec schema file
    fn resolve_schema_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        // First check environment variable
        if let Ok(path) = std::env::var("PROMPT_SPEC_SCHEMA_PATH") {
            return Ok(PathBuf::from(path));
        }
        
        // Try relative to CARGO_MANIFEST_DIR (for tests and development)
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let schema_path = PathBuf::from(manifest_dir)
                .parent() // go up from crate to workspace root
                .and_then(|p| p.parent()) // go up one more level
                .map(|p| p.join("schemas"))
                .ok_or("Failed to resolve schema path from manifest dir")?;
            
            // Make sure we return the full path to the JSON file, not just the directory
            let full_path = schema_path.join("prompt-spec.schema.json");
            if full_path.exists() {
                return Ok(full_path);
            }
        }
        
        // Try relative to current directory
        let current_dir_path = PathBuf::from("schemas/prompt-spec.schema.json");
        if current_dir_path.exists() {
            return Ok(current_dir_path);
        }
        
        // Try relative to executable location
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // Look for schemas directory relative to executable
                let schema_path = exe_dir.join("schemas").join("prompt-spec.schema.json");
                if schema_path.exists() {
                    return Ok(schema_path);
                }
                
                // Try going up directories to find schemas
                let mut current = exe_dir;
                for _ in 0..5 {  // Try up to 5 levels
                    if let Some(parent) = current.parent() {
                        let schema_path = parent.join("schemas").join("prompt-spec.schema.json");
                        if schema_path.exists() {
                            return Ok(schema_path);
                        }
                        current = parent;
                    } else {
                        break;
                    }
                }
            }
        }
        
        Err("Could not locate prompt-spec.schema.json. Set PROMPT_SPEC_SCHEMA_PATH environment variable.".into())
    }
    
    /// Get the loaded schema
    pub fn schema(&self) -> &Value {
        &self.schema
    }

    /// Validate custom rules for PromptSpec
    fn validate_custom_rules(&self, spec: &Value, ctx: &ValidationContext) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Rule: tool_choice requires tools array to be defined and non-empty
        if let Some(tool_choice) = spec.get("tool_choice") {
            if !tool_choice.is_null() {
            let tools = spec.get("tools");
            if tools.is_none() || !tools.unwrap().is_array() || tools.unwrap().as_array().unwrap().is_empty() {
                errors.push(ValidationError::new(
                    ctx.child("tool_choice").path.clone(),
                    "tool_choice requires tools array to be defined and non-empty",
                ));
            }
            }
        }

        // Rule: reasoning_tokens only valid when model_class is "ReasoningChat"
        if let Some(limits) = spec.get("limits") {
            if limits.get("reasoning_tokens").is_some() {
                let model_class = spec.get("model_class").and_then(|v| v.as_str());
                if model_class != Some("ReasoningChat") {
                    errors.push(ValidationError::new(
                        ctx.child("limits").child("reasoning_tokens").path.clone(),
                        format!("reasoning_tokens is only valid when model_class is ReasoningChat, found {:?}", model_class),
                    ));
                }
            }
        }

        // Rule: rag configuration only valid when model_class is "RAGChat"
        if spec.get("rag").is_some() {
            let model_class = spec.get("model_class").and_then(|v| v.as_str());
            if model_class != Some("RAGChat") {
                errors.push(ValidationError::new(
                    ctx.child("rag").path.clone(),
                    format!("rag configuration is only valid when model_class is RAGChat, found {:?}", model_class),
                ));
            }
        }

        // Rule: media.input_video only valid for "MultimodalChat" or "VideoChat"
        if let Some(media) = spec.get("media") {
            if media.get("input_video").is_some() {
                let model_class = spec.get("model_class").and_then(|v| v.as_str());
                if !matches!(model_class, Some("MultimodalChat") | Some("VideoChat")) {
                    errors.push(ValidationError::new(
                        ctx.child("media").child("input_video").path.clone(),
                        format!("input_video is only valid for MultimodalChat or VideoChat, found {:?}", model_class),
                    ));
                }
            }

            // Rule: media.input_audio only valid for "AudioChat" or "MultimodalChat"
            if media.get("input_audio").is_some() {
                let model_class = spec.get("model_class").and_then(|v| v.as_str());
                if !matches!(model_class, Some("AudioChat") | Some("MultimodalChat")) {
                    errors.push(ValidationError::new(
                        ctx.child("media").child("input_audio").path.clone(),
                        format!("input_audio is only valid for AudioChat or MultimodalChat, found {:?}", model_class),
                    ));
                }
            }
        }

        // Rule: conversation.parent_message_id requires valid message reference
        if let Some(conversation) = spec.get("conversation") {
            if let Some(parent_id) = conversation.get("parent_message_id").and_then(|v| v.as_str()) {
                // Basic validation: check format (should be UUID-like or similar)
                if parent_id.is_empty() || parent_id.len() < 8 {
                    errors.push(ValidationError::new(
                        ctx.child("conversation").child("parent_message_id").path.clone(),
                        format!("parent_message_id must be a valid message reference (at least 8 characters), got '{}'", parent_id),
                    ));
                }
            }
        }

        // Rule: When strict_mode is "Strict", no unknown fields allowed
        if let Some(strict_mode) = spec.get("strict_mode").and_then(|v| v.as_str()) {
            if strict_mode == "Strict" {
                // Always use the comprehensive list of known fields
                // Schema loading may not always have properties available during tests
                let known_fields = vec![
                    "spec_version", "id", "model_class", "messages", "model", "system",
                    "max_tokens", "temperature", "top_p", "top_k", "seed", "stop", 
                    "reasoning_tokens", "tools", "tool_choice", "media", "rag",
                    "conversation", "preferences", "frequency_penalty", "presence_penalty",
                    "repetition_penalty", "length_penalty", "strict_mode", "stream",
                    "metadata", "trace_id", "parent_span_id", "sampling", "limits",
                    "response_format"
                ];
                
                if let Some(obj) = spec.as_object() {
                    for key in obj.keys() {
                        if !known_fields.contains(&key.as_str()) {
                            errors.push(ValidationError::new(
                                ctx.child(key).path.clone(),
                                format!("Unknown field '{}' not allowed in Strict mode", key),
                            ));
                        }
                    }
                }
            }
        }

        errors
    }
}

impl SchemaValidator for PromptSpecValidator {
    type Input = Value;

    fn validate(&self, spec: &Value) -> ValidationResult<()> {
        let context = ValidationContext::new(ValidationMode::Strict);
        self.validate_with_context(spec, &context)
    }

    fn validate_partial(&self, spec: &Value) -> ValidationResult<()> {
        let context = ValidationContext::new(ValidationMode::Partial);
        self.validate_with_context(spec, &context)
    }

    fn validate_basic(&self, spec: &Value) -> ValidationResult<()> {
        let context = ValidationContext::new(ValidationMode::Basic);
        self.validate_with_context(spec, &context)
    }

    fn validate_with_context(&self, spec: &Value, ctx: &ValidationContext) -> ValidationResult<()> {
        let mut all_errors = Vec::new();

        // Check required fields (for all modes)
        // Required fields should always be validated
        // Check required fields from schema
        if let Some(required) = self.schema.get("required").and_then(|r| r.as_array()) {
            for field_value in required {
                if let Some(field) = field_value.as_str() {
                    if spec.get(field).is_none() {
                        all_errors.push(ValidationError::new(
                            ctx.child(field).path.clone(),
                            format!("Required field {} is missing", field),
                        ));
                    }
                }
            }
        } else {
            // Fallback if schema doesn't have required fields defined
            let required_fields = ["messages"];
            for field in &required_fields {
                if spec.get(field).is_none() {
                    all_errors.push(ValidationError::new(
                        ctx.child(field).path.clone(),
                        format!("Required field {} is missing", field),
                    ));
                }
            }
        }

        // Custom validation rules (for Partial and Strict modes)
        if ctx.mode == ValidationMode::Strict || ctx.mode == ValidationMode::Partial {
            all_errors.extend(self.validate_custom_rules(spec, ctx));
        }

        if all_errors.is_empty() {
            Ok(())
        } else {
            Err(all_errors.into_iter().next().unwrap())
        }
    }
}

impl Default for PromptSpecValidator {
    fn default() -> Self {
        Self::new().expect("Failed to create default PromptSpecValidator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_schema_loading() {
        let validator = PromptSpecValidator::new().unwrap();
        let schema = validator.schema();
        assert!(schema.is_object());
        assert_eq!(schema.get("title").and_then(|v| v.as_str()), Some("PromptSpec"));
    }

    fn create_basic_prompt_spec() -> Value {
        json!({
            "model_class": "Chat",
            "messages": [
                {"role": "user", "content": "Hello"}
            ],
            "strict_mode": "Standard"
        })
    }

    #[test]
    fn test_valid_basic_prompt_spec() {
        let validator = PromptSpecValidator::new().unwrap();
        let spec = create_basic_prompt_spec();
        assert!(validator.validate_basic(&spec).is_ok());
    }

    #[test]
    fn test_tool_choice_requires_tools() {
        let validator = PromptSpecValidator::new().unwrap();
        let mut spec = create_basic_prompt_spec();
        spec["tool_choice"] = json!("auto");
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("tool_choice requires tools"));
    }

    #[test]
    fn test_reasoning_tokens_model_class() {
        let validator = PromptSpecValidator::new().unwrap();
        
        // Invalid: reasoning_tokens with wrong model_class
        let mut spec = create_basic_prompt_spec();
        spec["limits"] = json!({"reasoning_tokens": 100});
        assert!(validator.validate(&spec).is_err());
        
        // Valid: reasoning_tokens with ReasoningChat
        spec["model_class"] = json!("ReasoningChat");
        assert!(validator.validate(&spec).is_ok());
    }

    #[test]
    fn test_rag_model_class() {
        let validator = PromptSpecValidator::new().unwrap();
        
        // Invalid: rag with wrong model_class
        let mut spec = create_basic_prompt_spec();
        spec["rag"] = json!({"documents": []});
        assert!(validator.validate(&spec).is_err());
        
        // Valid: rag with RAGChat
        spec["model_class"] = json!("RAGChat");
        assert!(validator.validate(&spec).is_ok());
    }

    #[test]
    fn test_media_constraints() {
        let validator = PromptSpecValidator::new().unwrap();
        
        // Test input_video constraints
        let mut spec = create_basic_prompt_spec();
        spec["media"] = json!({"input_video": "video.mp4"});
        assert!(validator.validate(&spec).is_err());
        
        spec["model_class"] = json!("MultimodalChat");
        assert!(validator.validate(&spec).is_ok());
        
        // Test input_audio constraints
        let mut spec = create_basic_prompt_spec();
        spec["media"] = json!({"input_audio": {"data": "base64..."}});
        assert!(validator.validate(&spec).is_err());
        
        spec["model_class"] = json!("AudioChat");
        assert!(validator.validate(&spec).is_ok());
    }

    #[test]
    fn test_strict_mode_validation() {
        let validator = PromptSpecValidator::new().unwrap();
        
        // Invalid: unknown field in Strict mode
        let mut spec = create_basic_prompt_spec();
        spec["strict_mode"] = json!("Strict");
        spec["unknown_field"] = json!("value");
        assert!(validator.validate(&spec).is_err());
        
        // Valid: remove unknown field and test with a known field
        spec.as_object_mut().unwrap().remove("unknown_field");
        assert!(validator.validate(&spec).is_ok());
    }

    #[test]
    fn test_validation_modes() {
        let validator = PromptSpecValidator::new().unwrap();
        
        // Basic mode: only checks structure
        let mut spec = create_basic_prompt_spec();
        spec["tool_choice"] = json!("auto");  // Would fail in strict mode
        
        assert!(validator.validate_basic(&spec).is_ok());
        assert!(validator.validate(&spec).is_err());
    }
}