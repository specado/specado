//! ProviderSpec validation with custom business logic rules
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::loader::{SchemaLoader, LoaderConfig};
use crate::validation::base::{SchemaValidator, ValidationContext, ValidationHelpers, ValidationMode};
use crate::validation::error::{ValidationError, ValidationResult};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// ProviderSpec validator with custom rules
pub struct ProviderSpecValidator {
    schema: Arc<Value>,
}

impl ProviderSpecValidator {
    /// Create a new ProviderSpec validator by loading the schema file
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
    
    /// Resolve the path to the provider-spec schema file
    fn resolve_schema_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        // First check environment variable
        if let Ok(path) = std::env::var("PROVIDER_SPEC_SCHEMA_PATH") {
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
            let full_path = schema_path.join("provider-spec.schema.json");
            if full_path.exists() {
                return Ok(full_path);
            }
        }
        
        // Try relative to current directory
        let current_dir_path = PathBuf::from("schemas/provider-spec.schema.json");
        if current_dir_path.exists() {
            return Ok(current_dir_path);
        }
        
        // Try relative to executable location
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // Look for schemas directory relative to executable
                let schema_path = exe_dir.join("schemas").join("provider-spec.schema.json");
                if schema_path.exists() {
                    return Ok(schema_path);
                }
                
                // Try going up directories to find schemas
                let mut current = exe_dir;
                for _ in 0..5 {  // Try up to 5 levels
                    if let Some(parent) = current.parent() {
                        let schema_path = parent.join("schemas").join("provider-spec.schema.json");
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
        
        Err("Could not locate provider-spec.schema.json. Set PROVIDER_SPEC_SCHEMA_PATH environment variable.".into())
    }
    
    /// Get the loaded schema
    pub fn schema(&self) -> &Value {
        &self.schema
    }

    /// Validate custom rules for ProviderSpec
    fn validate_custom_rules(&self, spec: &Value, ctx: &ValidationContext) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Rule: JSONPath expressions in mappings.paths should be non-empty strings
        if let Some(models) = spec.get("models").and_then(|m| m.as_array()) {
            for (model_idx, model) in models.iter().enumerate() {
                if let Some(mappings) = model.get("mappings") {
                    if let Some(paths) = mappings.get("paths").and_then(|p| p.as_object()) {
                        for (field, path_value) in paths {
                            if let Some(path) = path_value.as_str() {
                                if path.is_empty() {
                                    errors.push(ValidationError::new(
                                        format!("$.models[{}].mappings.paths.{}", model_idx, field),
                                        "JSONPath cannot be empty".to_string(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Rule: Environment variable references ${ENV:VAR} must be properly formatted
        if let Some(provider) = spec.get("provider") {
            if let Some(auth) = provider.get("auth") {
                if let Some(value_template) = auth.get("value_template").and_then(|v| v.as_str()) {
                    if !value_template.is_empty() && !value_template.contains("${ENV:") {
                        errors.push(ValidationError::new(
                            "$.provider.auth.value_template".to_string(),
                            "Environment variable references should use ${ENV:VARIABLE_NAME} format".to_string(),
                        ));
                    }
                }
            }
        }

        // Rule: input_modes must be compatible with model family constraints
        if let Some(models) = spec.get("models").and_then(|m| m.as_array()) {
            for (idx, model) in models.iter().enumerate() {
                if let Some(input_modes) = model.get("input_modes").and_then(|m| m.as_array()) {
                    let model_family = model.get("model_family").and_then(|f| f.as_str());
                    
                    // Example validation: chat models shouldn't have image input
                    if model_family == Some("chat") {
                        for mode in input_modes {
                            if mode.as_str() == Some("image") {
                                errors.push(ValidationError::new(
                                    format!("$.models[{}].input_modes", idx),
                                    "Chat models cannot have image input mode".to_string(),
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Rule: response_normalization paths should be non-empty strings
        if let Some(models) = spec.get("models").and_then(|m| m.as_array()) {
            for (idx, model) in models.iter().enumerate() {
                if let Some(norm) = model.get("response_normalization") {
                    // Check sync paths
                    if let Some(sync) = norm.get("sync").and_then(|s| s.as_object()) {
                        for (field, path_value) in sync {
                            if field.ends_with("_path") {
                                if let Some(path) = path_value.as_str() {
                                    if path.is_empty() {
                                        errors.push(ValidationError::new(
                                            format!("$.models[{}].response_normalization.sync.{}", idx, field),
                                            "JSONPath cannot be empty".to_string(),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    // Check stream event selector paths
                    if let Some(stream) = norm.get("stream") {
                        if let Some(event_selector) = stream.get("event_selector") {
                            // Check type_path
                            if let Some(type_path) = event_selector.get("type_path").and_then(|p| p.as_str()) {
                                if type_path.is_empty() {
                                    errors.push(ValidationError::new(
                                        format!("$.models[{}].response_normalization.stream.event_selector.type_path", idx),
                                        "JSONPath cannot be empty".to_string(),
                                    ));
                                }
                            }
                            // Check route paths
                            if let Some(routes) = event_selector.get("routes").and_then(|r| r.as_array()) {
                                for (route_idx, route) in routes.iter().enumerate() {
                                    if let Some(route_obj) = route.as_object() {
                                        for (field, path_value) in route_obj {
                                            if field.ends_with("_path") {
                                                if let Some(path) = path_value.as_str() {
                                                    if path.is_empty() {
                                                        errors.push(ValidationError::new(
                                                            format!("$.models[{}].response_normalization.stream.event_selector.routes[{}].{}", idx, route_idx, field),
                                                            "JSONPath cannot be empty".to_string(),
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Rule: tooling.tool_choice_modes must include "auto" if tools supported
        if let Some(capabilities) = spec.get("capabilities") {
            if capabilities.get("supports_tools").and_then(|v| v.as_bool()) == Some(true) {
                if let Some(tooling) = spec.get("tooling") {
                    if let Some(modes) = tooling.get("tool_choice_modes").and_then(|m| m.as_array()) {
                        let has_auto = modes.iter().any(|m| m.as_str() == Some("auto"));
                        if !has_auto {
                            errors.push(ValidationError::new(
                                ctx.child("tooling").child("tool_choice_modes").path.clone(),
                                "tool_choice_modes must include 'auto' when tools are supported".to_string(),
                            ));
                        }
                    }
                }
            }
        }

        // Rule: rag_config only valid when capabilities.supports_rag is true
        if spec.get("rag_config").is_some() {
            let supports_rag = spec.get("capabilities")
                .and_then(|c| c.get("supports_rag"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            
            if !supports_rag {
                errors.push(ValidationError::new(
                    ctx.child("rag_config").path.clone(),
                    "rag_config is only valid when capabilities.supports_rag is true".to_string(),
                ));
            }
        }

        // Rule: conversation_management only valid when capabilities.supports_conversation_persistence is true
        if spec.get("conversation_management").is_some() {
            let supports_conversation = spec.get("capabilities")
                .and_then(|c| c.get("supports_conversation_persistence"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            
            if !supports_conversation {
                errors.push(ValidationError::new(
                    ctx.child("conversation_management").path.clone(),
                    "conversation_management is only valid when capabilities.supports_conversation_persistence is true".to_string(),
                ));
            }
        }

        // Rule: Endpoint protocols (http/https/ws/wss) must match base_url scheme
        if let Some(base_url) = spec.get("base_url").and_then(|u| u.as_str()) {
            if let Some(models) = spec.get("models").and_then(|m| m.as_array()) {
                for (idx, model) in models.iter().enumerate() {
                    if let Some(endpoint) = model.get("endpoint") {
                        if let Some(protocol) = endpoint.get("protocol").and_then(|p| p.as_str()) {
                            let base_is_secure = base_url.starts_with("https://") || base_url.starts_with("wss://");
                            let endpoint_is_secure = protocol == "https" || protocol == "wss";
                            
                            if base_is_secure != endpoint_is_secure {
                                errors.push(ValidationError::new(
                                    format!("$.models[{}].endpoint.protocol", idx),
                                    format!("Endpoint protocol {} doesn't match base_url security", protocol),
                                ));
                            }
                        }
                    }
                }
            }
        }

        errors
    }
}

impl SchemaValidator for ProviderSpecValidator {
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
            let required_fields = ["spec_version", "provider_id", "base_url"];
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
    fn test_schema_loading() {
        let validator = ProviderSpecValidator::new().unwrap();
        let schema = validator.schema();
        assert!(schema.is_object());
        assert_eq!(schema.get("title").and_then(|v| v.as_str()), Some("ProviderSpec"));
    }

    fn create_basic_provider_spec() -> Value {
        json!({
            "spec_version": "1.0",
            "provider": {
                "id": "test-provider",
                "name": "Test Provider",
                "organization": "Test Org"
            },
            "models": []
        })
    }

    #[test]
    fn test_valid_basic_provider_spec() {
        let validator = ProviderSpecValidator::new().unwrap();
        let spec = create_basic_provider_spec();
        assert!(validator.validate_basic(&spec).is_ok());
    }

    #[test]
    fn test_environment_variable_validation() {
        let validator = ProviderSpecValidator::new().unwrap();
        
        // Valid env var format - add authentication field with valid env var
        let mut spec = create_basic_provider_spec();
        spec["authentication"] = json!({
            "type": "api_key",
            "location": "header",
            "key_name": "Authorization",
            "env_var": "${ENV:TEST_API_KEY}"
        });
        assert!(validator.validate(&spec).is_ok());
        
        // Invalid env var format
        let mut spec = create_basic_provider_spec();
        spec["authentication"] = json!({
            "type": "api_key",
            "location": "header",
            "key_name": "Authorization",
            "env_var": "${env:invalid_format}"
        });
        let result = validator.validate(&spec);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid environment variable reference"));
    }

    #[test]
    fn test_jsonpath_validation() {
        let validator = ProviderSpecValidator::new().unwrap();
        let mut spec = create_basic_provider_spec();
        
        // Add model with invalid JSONPath (must not start with $)
        spec["models"] = json!([{
            "model_id": "test-model",
            "model_family": "chat",
            "mappings": {
                "paths": {
                    "content": "invalid.path"  // Invalid JSONPath - doesn't start with $
                }
            }
        }]);
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid JSONPath"));
    }

    #[test]
    fn test_input_modes_compatibility() {
        let validator = ProviderSpecValidator::new().unwrap();
        let mut spec = create_basic_provider_spec();
        
        // Chat model with image input (invalid)
        spec["models"] = json!([{
            "model_id": "test-model",
            "model_family": "chat",
            "input_modes": ["text", "image"]
        }]);
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Chat models cannot have image input"));
    }

    #[test]
    fn test_tooling_configuration() {
        let validator = ProviderSpecValidator::new().unwrap();
        let mut spec = create_basic_provider_spec();
        
        // Enable tools support but don't include "auto" in tool_choice_modes
        spec["capabilities"] = json!({
            "supports_tools": true
        });
        spec["tooling"] = json!({
            "tool_choice_modes": ["required", "none"]
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must include 'auto'"));
    }

    #[test]
    fn test_rag_configuration() {
        let validator = ProviderSpecValidator::new().unwrap();
        let mut spec = create_basic_provider_spec();
        
        // Add rag_config without capability
        spec["rag_config"] = json!({
            "max_documents": 10
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("rag_config is only valid when"));
        
        // Enable capability - should pass
        spec["capabilities"]["supports_rag"] = json!(true);
        assert!(validator.validate(&spec).is_ok());
    }

    #[test]
    fn test_conversation_management() {
        let validator = ProviderSpecValidator::new().unwrap();
        let mut spec = create_basic_provider_spec();
        
        // Add conversation_management without capability
        spec["conversation_management"] = json!({
            "max_history": 100
        });
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("conversation_management is only valid"));
        
        // Enable capability - should pass
        spec["capabilities"]["supports_conversation_persistence"] = json!(true);
        assert!(validator.validate(&spec).is_ok());
    }

    #[test]
    fn test_endpoint_protocols() {
        let validator = ProviderSpecValidator::new().unwrap();
        let mut spec = create_basic_provider_spec();
        
        // Add base_url for this test
        spec["base_url"] = json!("https://api.test.com");
        
        // HTTPS base URL with HTTP endpoint (mismatch)
        spec["models"] = json!([{
            "model_id": "test-model",
            "endpoint": {
                "protocol": "http",
                "method": "POST",
                "path": "/v1/chat"
            }
        }]);
        
        let result = validator.validate(&spec);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("doesn't match base_url security"));
        
        // Fix protocol - should pass
        spec["models"][0]["endpoint"]["protocol"] = json!("https");
        assert!(validator.validate(&spec).is_ok());
    }

    #[test]
    fn test_validation_modes() {
        let validator = ProviderSpecValidator::new().unwrap();
        let mut spec = create_basic_provider_spec();
        
        // Add invalid env var format
        spec["authentication"]["env_var"] = json!("invalid_format");
        
        // Basic mode: only checks structure
        assert!(validator.validate_basic(&spec).is_ok());
        
        // Strict mode: checks custom rules
        assert!(validator.validate(&spec).is_err());
    }
}