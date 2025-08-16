//! ProviderSpec validation with custom business logic rules
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::validation::base::{SchemaValidator, ValidationContext, ValidationHelpers};
use crate::validation::error::{ValidationError, ValidationResult};
use serde_json::Value;
use std::path::Path;

// Embed the schema at compile time for reliability
const PROVIDER_SPEC_SCHEMA: &str = include_str!("../../../../schemas/provider-spec.schema.json");

/// ProviderSpec validator with custom rules
pub struct ProviderSpecValidator {
    schema: Value,
}

impl ProviderSpecValidator {
    /// Create a new ProviderSpec validator using embedded schema
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Try to load from environment variable first (for development)
        let schema = if let Ok(schema_path) = std::env::var("PROVIDER_SPEC_SCHEMA_PATH") {
            // Load from disk if path is provided
            let path = Path::new(&schema_path);
            if path.exists() {
                let content = std::fs::read_to_string(path)?;
                serde_json::from_str(&content)?
            } else {
                // Fall back to embedded schema
                serde_json::from_str(PROVIDER_SPEC_SCHEMA)?
            }
        } else {
            // Use embedded schema by default
            serde_json::from_str(PROVIDER_SPEC_SCHEMA)?
        };
        
        Ok(Self { schema })
    }
    
    /// Load schema from a specific path (useful for testing)
    pub fn from_path(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let schema: Value = serde_json::from_str(&content)?;
        Ok(Self { schema })
    }

    /// Get the JSON Schema definition for ProviderSpec
    pub fn schema(&self) -> &Value {
        &self.schema
    }
    
    #[allow(dead_code)]
    fn get_schema_definition() -> Value {
        serde_json::json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "https://schemas.specado.com/provider-spec/v1.json",
            "title": "ProviderSpec",
            "description": "Provider capabilities and mapping configuration",
            "type": "object",
            "properties": {
                "spec_version": {
                    "type": "string",
                    "pattern": "^\\d+\\.\\d+$",
                    "description": "Version of the ProviderSpec specification"
                },
                "provider_id": {
                    "type": "string",
                    "description": "Unique identifier for this provider"
                },
                "base_url": {
                    "type": "string",
                    "format": "uri",
                    "description": "Base URL for the provider's API"
                },
                "authentication": {
                    "type": "object",
                    "properties": {
                        "type": {
                            "type": "string",
                            "enum": ["api_key", "bearer_token", "oauth2", "basic_auth", "custom"]
                        },
                        "api_key_header": { "type": "string" },
                        "env_var": { "type": "string" }
                    },
                    "required": ["type"]
                },
                "capabilities": {
                    "type": "object",
                    "properties": {
                        "supports_tools": { "type": "boolean" },
                        "supports_rag": { "type": "boolean" },
                        "supports_conversation_persistence": { "type": "boolean" },
                        "supports_streaming": { "type": "boolean" },
                        "model_families": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["chat", "reasoning", "multimodal", "audio", "video", "embedding", "rag"]
                            }
                        },
                        "input_modes": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["text", "image", "audio", "video", "document"]
                            }
                        }
                    },
                    "required": ["supports_tools", "supports_rag", "supports_streaming", "model_families"]
                },
                "mappings": {
                    "type": "object",
                    "properties": {
                        "request_mapping": {
                            "type": "object",
                            "properties": {
                                "paths": {
                                    "type": "object",
                                    "additionalProperties": { "type": "string" }
                                },
                                "transformations": { "type": "object" }
                            }
                        },
                        "response_mapping": {
                            "type": "object",
                            "properties": {
                                "paths": {
                                    "type": "object",
                                    "additionalProperties": { "type": "string" }
                                },
                                "transformations": { "type": "object" }
                            }
                        }
                    }
                },
                "response_normalization": {
                    "type": "object",
                    "properties": {
                        "content_path": { "type": "string" },
                        "role_path": { "type": "string" },
                        "usage_path": { "type": "string" },
                        "error_path": { "type": "string" }
                    }
                },
                "tooling": {
                    "type": "object",
                    "properties": {
                        "tool_choice_modes": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["auto", "none", "required", "specific"]
                            }
                        },
                        "tool_format": {
                            "type": "string",
                            "enum": ["openai", "anthropic", "custom"]
                        }
                    }
                },
                "rag_config": {
                    "type": "object",
                    "properties": {
                        "retrieval_endpoint": { "type": "string" },
                        "embedding_model": { "type": "string" },
                        "chunk_size": { "type": "integer" },
                        "similarity_threshold": { "type": "number" }
                    }
                },
                "conversation_management": {
                    "type": "object",
                    "properties": {
                        "session_endpoint": { "type": "string" },
                        "persistence_type": {
                            "type": "string",
                            "enum": ["memory", "database", "external"]
                        }
                    }
                },
                "endpoints": {
                    "type": "object",
                    "properties": {
                        "chat": { "type": "string" },
                        "embeddings": { "type": "string" },
                        "models": { "type": "string" },
                        "health": { "type": "string" }
                    }
                },
                "rate_limits": {
                    "type": "object",
                    "properties": {
                        "requests_per_minute": { "type": "integer" },
                        "tokens_per_minute": { "type": "integer" },
                        "concurrent_requests": { "type": "integer" }
                    }
                }
            },
            "required": ["spec_version", "provider_id", "base_url", "authentication", "capabilities"],
            "additionalProperties": false
        })
    }

    /// Validate basic structure requirements
    fn validate_basic_structure(
        &self,
        data: &Value,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        // Check that required fields exist
        let required_fields = ["spec_version", "provider_id", "base_url", "authentication", "capabilities"];
        
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

        Ok(())
    }

    /// Validate JSONPath expressions in mappings
    fn validate_jsonpath_expressions(
        &self,
        data: &Value,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        let mappings = data.get("mappings");
        
        if let Some(mappings_obj) = mappings {
            // Validate request mapping paths
            if let Some(request_mapping) = mappings_obj.get("request_mapping") {
                if let Some(paths) = request_mapping.get("paths") {
                    if let Some(paths_obj) = paths.as_object() {
                        for (key, value) in paths_obj {
                            if let Some(path_str) = value.as_str() {
                                let path_context = context.child("mappings")
                                    .child("request_mapping")
                                    .child("paths")
                                    .child(key);
                                ValidationHelpers::validate_jsonpath(path_str, &path_context)?;
                            }
                        }
                    }
                }
            }

            // Validate response mapping paths
            if let Some(response_mapping) = mappings_obj.get("response_mapping") {
                if let Some(paths) = response_mapping.get("paths") {
                    if let Some(paths_obj) = paths.as_object() {
                        for (key, value) in paths_obj {
                            if let Some(path_str) = value.as_str() {
                                let path_context = context.child("mappings")
                                    .child("response_mapping")
                                    .child("paths")
                                    .child(key);
                                ValidationHelpers::validate_jsonpath(path_str, &path_context)?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate environment variable references
    fn validate_environment_variables(
        &self,
        data: &Value,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        // Check authentication env_var
        if let Some(auth) = data.get("authentication") {
            if let Some(env_var) = auth.get("env_var").and_then(|v| v.as_str()) {
                let auth_context = context.child("authentication").child("env_var");
                ValidationHelpers::validate_env_var_reference(env_var, &auth_context)?;
            }
        }

        // Check for environment variables in any string value recursively
        self.validate_env_vars_recursive(data, context)?;

        Ok(())
    }

    /// Recursively validate environment variable references
    fn validate_env_vars_recursive(
        &self,
        value: &Value,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        match value {
            Value::String(s) => {
                // Simple check for environment variable pattern
                if s.contains("${ENV:") {
                    ValidationHelpers::validate_env_var_reference(s, context)?;
                }
            }
            Value::Object(obj) => {
                for (key, val) in obj {
                    let child_context = context.child(key);
                    self.validate_env_vars_recursive(val, &child_context)?;
                }
            }
            Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    let child_context = context.child_index(i);
                    self.validate_env_vars_recursive(val, &child_context)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Validate input modes compatibility with model families
    fn validate_input_modes_compatibility(
        &self,
        data: &Value,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        let capabilities = data.get("capabilities");
        
        if let Some(caps) = capabilities {
            let model_families = caps.get("model_families")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                .unwrap_or_default();

            let input_modes = caps.get("input_modes")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                .unwrap_or_default();

            // Validate that input modes are compatible with model families
            for mode in &input_modes {
                match *mode {
                    "image" => {
                        if !model_families.contains(&"multimodal") {
                            return Err(ValidationError::with_violations(
                                &context.child("capabilities").child("input_modes").path,
                                "input_mode 'image' requires 'multimodal' in model_families".to_string(),
                                vec![ValidationError::create_violation(
                                    "input_mode_compatibility",
                                    "'multimodal' in model_families",
                                    format!("model_families: {:?}", model_families),
                                )],
                            ));
                        }
                    }
                    "audio" => {
                        if !model_families.contains(&"audio") && !model_families.contains(&"multimodal") {
                            return Err(ValidationError::with_violations(
                                &context.child("capabilities").child("input_modes").path,
                                "input_mode 'audio' requires 'audio' or 'multimodal' in model_families".to_string(),
                                vec![ValidationError::create_violation(
                                    "input_mode_compatibility",
                                    "'audio' or 'multimodal' in model_families",
                                    format!("model_families: {:?}", model_families),
                                )],
                            ));
                        }
                    }
                    "video" => {
                        if !model_families.contains(&"video") && !model_families.contains(&"multimodal") {
                            return Err(ValidationError::with_violations(
                                &context.child("capabilities").child("input_modes").path,
                                "input_mode 'video' requires 'video' or 'multimodal' in model_families".to_string(),
                                vec![ValidationError::create_violation(
                                    "input_mode_compatibility",
                                    "'video' or 'multimodal' in model_families",
                                    format!("model_families: {:?}", model_families),
                                )],
                            ));
                        }
                    }
                    _ => {} // text and document are always valid
                }
            }
        }

        Ok(())
    }

    /// Validate response normalization paths are valid JSONPath
    fn validate_response_normalization_paths(
        &self,
        data: &Value,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        let response_norm = data.get("response_normalization");
        
        if let Some(norm_obj) = response_norm {
            let path_fields = ["content_path", "role_path", "usage_path", "error_path"];
            
            for field in &path_fields {
                if let Some(path_str) = norm_obj.get(field).and_then(|v| v.as_str()) {
                    let field_context = context.child("response_normalization").child(field);
                    ValidationHelpers::validate_jsonpath(path_str, &field_context)?;
                }
            }
        }

        Ok(())
    }

    /// Validate tooling configuration
    fn validate_tooling_configuration(
        &self,
        data: &Value,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        let capabilities = data.get("capabilities");
        let tooling = data.get("tooling");
        
        if let Some(tooling_obj) = tooling {
            // Check if tools are supported
            let supports_tools = capabilities
                .and_then(|c| c.get("supports_tools"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if !supports_tools {
                return Err(ValidationError::with_violations(
                    &context.child("tooling").path,
                    "tooling configuration requires capabilities.supports_tools to be true".to_string(),
                    vec![ValidationError::create_violation(
                        "tooling_requires_support",
                        "capabilities.supports_tools to be true",
                        "supports_tools is false".to_string(),
                    )],
                ));
            }

            // Validate tool_choice_modes includes "auto" if tools are supported
            if let Some(choice_modes) = tooling_obj.get("tool_choice_modes").and_then(|v| v.as_array()) {
                let modes: Vec<&str> = choice_modes.iter().filter_map(|v| v.as_str()).collect();
                if !modes.contains(&"auto") {
                    return Err(ValidationError::with_violations(
                        &context.child("tooling").child("tool_choice_modes").path,
                        "tool_choice_modes must include 'auto' when tools are supported".to_string(),
                        vec![ValidationError::create_violation(
                            "tool_choice_auto_required",
                            "'auto' in tool_choice_modes",
                            format!("modes: {:?}", modes),
                        )],
                    ));
                }
            }
        }

        Ok(())
    }

    /// Validate RAG configuration requires RAG support
    fn validate_rag_configuration(
        &self,
        data: &Value,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        let capabilities = data.get("capabilities");
        let rag_config = data.get("rag_config");
        
        if let Some(_) = rag_config {
            let supports_rag = capabilities
                .and_then(|c| c.get("supports_rag"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if !supports_rag {
                return Err(ValidationError::with_violations(
                    &context.child("rag_config").path,
                    "rag_config requires capabilities.supports_rag to be true".to_string(),
                    vec![ValidationError::create_violation(
                        "rag_config_requires_support",
                        "capabilities.supports_rag to be true",
                        "supports_rag is false".to_string(),
                    )],
                ));
            }
        }

        Ok(())
    }

    /// Validate conversation management configuration
    fn validate_conversation_management(
        &self,
        data: &Value,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        let capabilities = data.get("capabilities");
        let conv_mgmt = data.get("conversation_management");
        
        if let Some(_) = conv_mgmt {
            let supports_conversation = capabilities
                .and_then(|c| c.get("supports_conversation_persistence"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if !supports_conversation {
                return Err(ValidationError::with_violations(
                    &context.child("conversation_management").path,
                    "conversation_management requires capabilities.supports_conversation_persistence to be true".to_string(),
                    vec![ValidationError::create_violation(
                        "conversation_management_requires_support",
                        "capabilities.supports_conversation_persistence to be true",
                        "supports_conversation_persistence is false".to_string(),
                    )],
                ));
            }
        }

        Ok(())
    }

    /// Validate endpoint protocols match base_url scheme
    fn validate_endpoint_protocols(
        &self,
        data: &Value,
        context: &ValidationContext,
    ) -> ValidationResult<()> {
        let base_url = data.get("base_url").and_then(|v| v.as_str());
        let endpoints = data.get("endpoints");
        
        if let (Some(base), Some(endpoints_obj)) = (base_url, endpoints) {
            // Extract scheme from base_url (simple approach)
            let base_scheme = if let Some(colon_pos) = base.find(':') {
                base[..colon_pos].to_lowercase()
            } else {
                return Err(ValidationError::with_violations(
                    &context.child("base_url").path,
                    format!("Invalid base_url format: {}", base),
                    vec![ValidationError::create_violation(
                        "base_url_format",
                        "valid URL with scheme",
                        base.to_string(),
                    )],
                ));
            };

            let allowed_schemes = match base_scheme.as_str() {
                "http" | "https" => vec!["http", "https"],
                "ws" | "wss" => vec!["ws", "wss"],
                _ => vec![base_scheme.as_str()],
            };

            // Validate each endpoint
            if let Some(endpoints_map) = endpoints_obj.as_object() {
                for (endpoint_name, endpoint_value) in endpoints_map {
                    if let Some(endpoint_url) = endpoint_value.as_str() {
                        let endpoint_context = context.child("endpoints").child(endpoint_name);
                        ValidationHelpers::validate_url_scheme(
                            endpoint_url,
                            &allowed_schemes.iter().map(|s| *s).collect::<Vec<_>>(),
                            &endpoint_context,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl SchemaValidator for ProviderSpecValidator {
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
                self.validate_environment_variables(input, context)?;
                self.validate_input_modes_compatibility(input, context)?;
                Ok(())
            }
            crate::validation::base::ValidationMode::Strict => {
                // Run all custom validations in strict mode
                self.validate_jsonpath_expressions(input, context)?;
                self.validate_environment_variables(input, context)?;
                self.validate_input_modes_compatibility(input, context)?;
                self.validate_response_normalization_paths(input, context)?;
                self.validate_tooling_configuration(input, context)?;
                self.validate_rag_configuration(input, context)?;
                self.validate_conversation_management(input, context)?;
                self.validate_endpoint_protocols(input, context)?;
                Ok(())
            }
        }
    }
}

impl Default for ProviderSpecValidator {
    fn default() -> Self {
        Self::new().expect("Failed to create ProviderSpecValidator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use serde_json::json;
    
    #[test]
    fn test_embedded_schema_loading() {
        let validator = ProviderSpecValidator::new().unwrap();
        let schema = validator.schema();
        assert!(schema.is_object());
        assert_eq!(schema.get("title").and_then(|v| v.as_str()), Some("ProviderSpec"));
        assert_eq!(schema.get("$schema").and_then(|v| v.as_str()), Some("https://json-schema.org/draft/2020-12/schema"));
    }

    fn create_basic_provider_spec() -> Value {
        json!({
            "spec_version": "1.0",
            "provider_id": "test-provider",
            "base_url": "https://api.test.com",
            "authentication": {
                "type": "api_key",
                "api_key_header": "Authorization",
                "env_var": "${ENV:API_KEY}"
            },
            "capabilities": {
                "supports_tools": true,
                "supports_rag": false,
                "supports_streaming": true,
                "model_families": ["chat"]
            }
        })
    }

    #[test]
    fn test_valid_basic_provider_spec() {
        let validator = ProviderSpecValidator::new().unwrap();
        let spec = create_basic_provider_spec();
        assert!(validator.validate(&spec).is_ok());
    }

    #[test]
    fn test_jsonpath_validation() {
        let validator = ProviderSpecValidator::new().unwrap();
        let mut spec = create_basic_provider_spec();
        
        spec.as_object_mut().unwrap().insert("mappings".to_string(), json!({
            "request_mapping": {
                "paths": {
                    "content": "$.messages[0].content",
                    "invalid": "not-a-jsonpath"
                }
            }
        }));
        
        // Should fail with invalid JSONPath
        assert!(validator.validate(&spec).is_err());
        
        // Should pass with valid JSONPath
        spec["mappings"]["request_mapping"]["paths"]["invalid"] = json!("$.valid.path");
        assert!(validator.validate(&spec).is_ok());
    }

    #[test]
    fn test_input_modes_compatibility() {
        let validator = ProviderSpecValidator::new().unwrap();
        let mut spec = create_basic_provider_spec();
        
        // Add image input mode without multimodal support
        spec["capabilities"]["input_modes"] = json!(["text", "image"]);
        assert!(validator.validate(&spec).is_err());
        
        // Add multimodal support
        spec["capabilities"]["model_families"] = json!(["chat", "multimodal"]);
        assert!(validator.validate(&spec).is_ok());
    }

    #[test]
    fn test_tooling_configuration() {
        let validator = ProviderSpecValidator::new().unwrap();
        let mut spec = create_basic_provider_spec();
        
        // Add tooling without tool support
        spec["capabilities"]["supports_tools"] = json!(false);
        spec.as_object_mut().unwrap().insert("tooling".to_string(), json!({
            "tool_choice_modes": ["auto", "none"]
        }));
        
        assert!(validator.validate(&spec).is_err());
        
        // Enable tool support
        spec["capabilities"]["supports_tools"] = json!(true);
        assert!(validator.validate(&spec).is_ok());
        
        // Test missing "auto" mode
        spec["tooling"]["tool_choice_modes"] = json!(["none", "required"]);
        assert!(validator.validate(&spec).is_err());
    }

    #[test]
    fn test_rag_configuration() {
        let validator = ProviderSpecValidator::new().unwrap();
        let mut spec = create_basic_provider_spec();
        
        // Add RAG config without RAG support
        spec.as_object_mut().unwrap().insert("rag_config".to_string(), json!({
            "retrieval_endpoint": "/retrieve",
            "embedding_model": "text-embedding-ada-002"
        }));
        
        assert!(validator.validate(&spec).is_err());
        
        // Enable RAG support
        spec["capabilities"]["supports_rag"] = json!(true);
        assert!(validator.validate(&spec).is_ok());
    }

    #[test]
    fn test_conversation_management() {
        let validator = ProviderSpecValidator::new().unwrap();
        let mut spec = create_basic_provider_spec();
        
        // Add conversation management without support
        spec.as_object_mut().unwrap().insert("conversation_management".to_string(), json!({
            "session_endpoint": "/sessions",
            "persistence_type": "memory"
        }));
        
        assert!(validator.validate(&spec).is_err());
        
        // Enable conversation persistence support
        spec["capabilities"]["supports_conversation_persistence"] = json!(true);
        assert!(validator.validate(&spec).is_ok());
    }

    #[test]
    fn test_endpoint_protocols() {
        let validator = ProviderSpecValidator::new().unwrap();
        let mut spec = create_basic_provider_spec();
        
        // Add endpoints with mismatched protocols
        spec.as_object_mut().unwrap().insert("endpoints".to_string(), json!({
            "chat": "ws://api.test.com/chat",  // WebSocket with HTTPS base
            "models": "https://api.test.com/models"
        }));
        
        assert!(validator.validate(&spec).is_err());
        
        // Fix protocol mismatch
        spec["endpoints"]["chat"] = json!("https://api.test.com/chat");
        assert!(validator.validate(&spec).is_ok());
    }

    #[test]
    fn test_environment_variable_validation() {
        let validator = ProviderSpecValidator::new().unwrap();
        let mut spec = create_basic_provider_spec();
        
        // Invalid environment variable format
        spec["authentication"]["env_var"] = json!("${env:invalid}");
        assert!(validator.validate(&spec).is_err());
        
        // Valid format
        spec["authentication"]["env_var"] = json!("${ENV:VALID_VAR}");
        assert!(validator.validate(&spec).is_ok());
    }

    #[test]
    fn test_validation_modes() {
        let validator = ProviderSpecValidator::new().unwrap();
        let mut spec = create_basic_provider_spec();
        
        // Add invalid JSONPath that should be caught in strict mode
        spec.as_object_mut().unwrap().insert("mappings".to_string(), json!({
            "request_mapping": {
                "paths": {
                    "content": "invalid-jsonpath"
                }
            }
        }));
        
        // Basic mode should pass (no custom validations)
        assert!(validator.validate_basic(&spec).is_ok());
        
        // Strict mode should fail
        assert!(validator.validate(&spec).is_err());
    }
}