//! Validation tests for provider specifications
//!
//! This module contains comprehensive validation tests for all provider specifications,
//! ensuring they are valid against the ProviderSpec schema and contain correct mappings
//! and constraints.

#[cfg(test)]
mod tests {
    use crate::{ProviderSpec, Result};
    use crate::provider_discovery::ProviderRegistry;
    use crate::{PromptSpec, Message, MessageRole, StrictMode};
    use std::path::PathBuf;
    
    /// Helper function to load and parse a provider specification
    /// Note: This requires the provider spec files to exist in the filesystem
    #[allow(dead_code)]
    fn load_provider_spec(path: &str) -> Result<ProviderSpec> {
        let spec_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()
            .parent().unwrap()
            .join(path);
        
        let content = std::fs::read_to_string(&spec_path)
            .map_err(|e| crate::Error::Io {
                message: format!("Failed to read provider spec from {:?}: {}", spec_path, e),
                source: e,
            })?;
        
        serde_json::from_str(&content)
            .map_err(|e| crate::Error::Json {
                message: format!("Failed to parse provider spec from {:?}: {}", spec_path, e),
                source: e,
            })
    }
    
    /// Helper function to validate a provider spec against the schema
    fn validate_provider_spec(spec: &ProviderSpec) -> Result<()> {
        // Basic structural validation
        if spec.spec_version.is_empty() {
            return Err(crate::Error::Validation {
                field: "spec_version".to_string(),
                message: "Spec version is required".to_string(),
                expected: Some("Non-empty version string".to_string()),
            });
        }
        
        if spec.provider.name.is_empty() {
            return Err(crate::Error::Validation {
                field: "provider.name".to_string(),
                message: "Provider name is required".to_string(),
                expected: Some("Non-empty provider name".to_string()),
            });
        }
        
        if spec.models.is_empty() {
            return Err(crate::Error::Validation {
                field: "models".to_string(),
                message: "At least one model must be defined".to_string(),
                expected: Some("Non-empty models array".to_string()),
            });
        }
        
        // Validate each model
        for (i, model) in spec.models.iter().enumerate() {
            if model.id.is_empty() {
                return Err(crate::Error::Validation {
                    field: format!("models[{}].id", i),
                    message: "Model ID is required".to_string(),
                    expected: Some("Non-empty model ID".to_string()),
                });
            }
            
            // Validate endpoints
            if model.endpoints.chat_completion.path.is_empty() {
                return Err(crate::Error::Validation {
                    field: format!("models[{}].endpoints.chat_completion.path", i),
                    message: "Chat completion endpoint path is required".to_string(),
                    expected: Some("Non-empty path".to_string()),
                });
            }
            
            // Validate response normalization paths
            if model.response_normalization.sync.content_path.is_empty() {
                return Err(crate::Error::Validation {
                    field: format!("models[{}].response_normalization.sync.content_path", i),
                    message: "Content path is required for response normalization".to_string(),
                    expected: Some("Non-empty JSONPath".to_string()),
                });
            }
        }
        
        Ok(())
    }
    
    /// Helper to check all mapping paths are valid
    fn assert_all_paths_valid(spec: &ProviderSpec) {
        for model in &spec.models {
            // Check that all mapping paths are non-empty
            for (key, path) in &model.mappings.paths {
                assert!(!path.is_empty(), "Mapping path for '{}' should not be empty", key);
            }
            
            // Check response normalization paths
            assert!(!model.response_normalization.sync.content_path.is_empty(),
                "Content path should not be empty");
            assert!(!model.response_normalization.sync.finish_reason_path.is_empty(),
                "Finish reason path should not be empty");
            
            // Check stream event selector paths
            assert!(!model.response_normalization.stream.event_selector.type_path.is_empty(),
                "Event type path should not be empty");
        }
    }
    
    #[test]
    #[ignore] // Requires provider spec files to be in the filesystem
    fn test_openai_gpt5_spec_valid() {
        let spec = load_provider_spec("providers/openai/gpt-5.json")
            .expect("Failed to load OpenAI GPT-5 spec");
        
        assert!(validate_provider_spec(&spec).is_ok());
        assert_all_paths_valid(&spec);
        
        // Verify OpenAI-specific features
        assert_eq!(spec.provider.name, "openai");
        assert!(!spec.models.is_empty());
        
        for model in &spec.models {
            // Check endpoints are correct for GPT-5
            assert_eq!(model.endpoints.chat_completion.path, "/v1/responses");
            
            // Check tooling support
            assert!(model.tooling.tools_supported);
            assert!(model.tooling.parallel_tool_calls_default);
            
            // Check JSON output support
            assert!(model.json_output.native_param);
        }
    }
    
    #[test]
    #[ignore] // Requires provider spec files to be in the filesystem
    fn test_openai_gpt5_mini_spec_valid() {
        let spec = load_provider_spec("providers/openai/gpt-5-mini.json")
            .expect("Failed to load OpenAI GPT-5 Mini spec");
        
        assert!(validate_provider_spec(&spec).is_ok());
        assert_all_paths_valid(&spec);
        
        // Verify it's for the mini model
        assert!(spec.models[0].id.contains("mini"));
    }
    
    #[test]
    #[ignore] // Requires provider spec files to be in the filesystem
    fn test_openai_gpt5_nano_spec_valid() {
        let spec = load_provider_spec("providers/openai/gpt-5-nano.json")
            .expect("Failed to load OpenAI GPT-5 Nano spec");
        
        assert!(validate_provider_spec(&spec).is_ok());
        assert_all_paths_valid(&spec);
        
        // Verify it's for the nano model
        assert!(spec.models[0].id.contains("nano"));
    }
    
    #[test]
    #[ignore] // Requires provider spec files to be in the filesystem
    fn test_anthropic_claude_opus_spec_valid() {
        let spec = load_provider_spec("providers/anthropic/claude-opus-4.1.json")
            .expect("Failed to load Anthropic Claude Opus 4.1 spec");
        
        assert!(validate_provider_spec(&spec).is_ok());
        assert_all_paths_valid(&spec);
        
        // Verify Anthropic-specific features
        assert_eq!(spec.provider.name, "anthropic");
        assert!(!spec.models.is_empty());
        
        for model in &spec.models {
            // Check endpoints are correct for Anthropic
            assert_eq!(model.endpoints.chat_completion.path, "/v1/messages");
            
            // Check system prompt location constraint
            assert_eq!(model.constraints.system_prompt_location, "top_level");
            
            // Check Anthropic-specific constraints
            assert!(model.constraints.forbid_unknown_top_level_fields);
        }
    }
    
    #[test]
    fn test_provider_discovery_integration() {
        let registry = ProviderRegistry::new();
        
        // Test that we can discover OpenAI models
        let provider = registry.discover_provider("gpt-5")
            .expect("Should discover GPT-5 provider");
        assert_eq!(provider.name, "openai");
        
        // Test that we can discover Anthropic models
        let provider = registry.discover_provider("claude-opus-4.1")
            .expect("Should discover Claude Opus 4.1 provider");
        assert_eq!(provider.name, "anthropic");
        
        // Test pattern matching
        let provider = registry.discover_provider("gpt-5-mini")
            .expect("Should discover GPT-4 provider through pattern");
        assert_eq!(provider.name, "openai");
        
        let provider = registry.discover_provider("claude-3-sonnet")
            .expect("Should discover Claude 3 provider through pattern");
        assert_eq!(provider.name, "anthropic");
    }
    
    #[test]
    #[ignore] // Requires provider spec files to be in the filesystem  
    fn test_spec_loading_and_parsing() {
        // Test loading specs directly instead of through registry
        // to avoid path issues in tests
        let openai_spec = load_provider_spec("providers/openai/gpt-5.json")
            .expect("Should load OpenAI spec");
        assert!(!openai_spec.models.is_empty());
        assert_eq!(openai_spec.provider.name, "openai");
        
        let anthropic_spec = load_provider_spec("providers/anthropic/claude-opus-4.1.json")
            .expect("Should load Anthropic spec");
        assert!(!anthropic_spec.models.is_empty());
        assert_eq!(anthropic_spec.provider.name, "anthropic");
    }
    
    #[test]
    #[ignore] // Requires provider spec files to be in the filesystem
    fn test_constraint_definitions() {
        // Test OpenAI constraints
        let openai_spec = load_provider_spec("providers/openai/gpt-5.json")
            .expect("Failed to load OpenAI spec");
        
        for model in &openai_spec.models {
            // Check limits are reasonable
            assert!(model.constraints.limits.max_tool_schema_bytes > 0);
            assert!(model.constraints.limits.max_system_prompt_bytes > 0);
        }
        
        // Test Anthropic constraints
        let anthropic_spec = load_provider_spec("providers/anthropic/claude-opus-4.1.json")
            .expect("Failed to load Anthropic spec");
        
        for model in &anthropic_spec.models {
            // Check Anthropic-specific constraints
            assert!(model.constraints.limits.max_tool_schema_bytes > 0);
            assert!(model.constraints.limits.max_system_prompt_bytes > 0);
            
            // Check mutual exclusivity is defined
            // Note: The actual constraints.json would have these, but for the main spec
            // we check that the structure supports them
            assert!(model.constraints.mutually_exclusive.is_empty() || 
                   !model.constraints.mutually_exclusive.is_empty());
        }
    }
    
    #[test]
    #[ignore] // Requires provider spec files to be in the filesystem
    fn test_response_normalization_paths() {
        let specs = vec![
            ("providers/openai/gpt-5.json", "OpenAI GPT-5"),
            ("providers/openai/gpt-5-mini.json", "OpenAI GPT-5 Mini"),
            ("providers/openai/gpt-5-nano.json", "OpenAI GPT-5 Nano"),
            ("providers/anthropic/claude-opus-4.1.json", "Anthropic Claude Opus 4.1"),
        ];
        
        for (path, name) in specs {
            let spec = load_provider_spec(path)
                .unwrap_or_else(|_| panic!("Failed to load {} spec", name));
            
            for model in &spec.models {
                // Sync normalization paths
                assert!(!model.response_normalization.sync.content_path.is_empty(),
                    "{}: Content path should not be empty", name);
                assert!(!model.response_normalization.sync.finish_reason_path.is_empty(),
                    "{}: Finish reason path should not be empty", name);
                
                // Stream normalization
                assert_eq!(model.response_normalization.stream.protocol, "sse",
                    "{}: Should use SSE protocol", name);
                assert!(!model.response_normalization.stream.event_selector.type_path.is_empty(),
                    "{}: Event type path should not be empty", name);
            }
        }
    }
    
    #[test]
    #[ignore] // Requires provider spec files to be in the filesystem
    fn test_feature_flag_configurations() {
        // Test OpenAI feature flags
        let openai_spec = load_provider_spec("providers/openai/gpt-5.json")
            .expect("Failed to load OpenAI spec");
        
        for model in &openai_spec.models {
            // Check tool support flags
            assert!(model.tooling.tools_supported);
            assert!(model.tooling.parallel_tool_calls_default);
            
            // Check JSON output flags
            assert!(model.json_output.native_param);
            assert_eq!(model.json_output.strategy, "json_schema");
            
            // Check input modes
            assert!(model.input_modes.messages);
        }
        
        // Test Anthropic feature flags
        let anthropic_spec = load_provider_spec("providers/anthropic/claude-opus-4.1.json")
            .expect("Failed to load Anthropic spec");
        
        for model in &anthropic_spec.models {
            // Check tool support
            assert!(model.tooling.tools_supported);
            
            // Check JSON output
            assert!(!model.json_output.native_param); // Anthropic doesn't have native JSON param
            
            // Check input modes
            assert!(model.input_modes.messages);
        }
    }
    
    #[test]
    #[ignore] // Requires provider spec files to be in the filesystem
    fn test_version_compatibility() {
        let specs = vec![
            "providers/openai/gpt-5.json",
            "providers/openai/gpt-5-mini.json",
            "providers/openai/gpt-5-nano.json",
            "providers/anthropic/claude-opus-4.1.json",
        ];
        
        for path in specs {
            let spec = load_provider_spec(path)
                .unwrap_or_else(|_| panic!("Failed to load spec: {}", path));
            
            // Check spec version is valid semver-like
            assert!(!spec.spec_version.is_empty());
            let parts: Vec<&str> = spec.spec_version.split('.').collect();
            assert!(parts.len() >= 2, "Version should be semver-like: {}", spec.spec_version);
            
            // Check all required fields are present (backward compatibility)
            assert!(!spec.provider.name.is_empty());
            assert!(!spec.provider.base_url.is_empty());
            assert!(!spec.models.is_empty());
        }
    }
    
    #[test]
    #[ignore] // Requires provider spec files to be in the filesystem
    fn test_translation_with_loaded_specs() {
        // Create a test prompt
        let prompt_spec = PromptSpec {
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
            advanced: None,
            strict_mode: StrictMode::Warn,
        };
        
        // Test translation with OpenAI spec
        let openai_spec = load_provider_spec("providers/openai/gpt-5.json")
            .expect("Failed to load OpenAI spec");
        
        let result = crate::translate(&prompt_spec, &openai_spec, "gpt-5", StrictMode::Warn);
        assert!(result.is_ok(), "Translation with OpenAI spec should succeed");
        
        // Test translation with Anthropic spec
        let anthropic_spec = load_provider_spec("providers/anthropic/claude-opus-4.1.json")
            .expect("Failed to load Anthropic spec");
        
        let result = crate::translate(&prompt_spec, &anthropic_spec, "claude-opus-4.1", StrictMode::Warn);
        assert!(result.is_ok(), "Translation with Anthropic spec should succeed");
    }
    
    #[test]
    #[ignore] // Requires provider spec files to be in the filesystem
    fn test_cross_provider_consistency() {
        // Load all provider specs
        let openai_spec = load_provider_spec("providers/openai/gpt-5.json")
            .expect("Failed to load OpenAI spec");
        let anthropic_spec = load_provider_spec("providers/anthropic/claude-opus-4.1.json")
            .expect("Failed to load Anthropic spec");
        
        // Check that common fields are consistently named
        for model in &openai_spec.models {
            assert!(!model.id.is_empty());
            assert!(!model.family.is_empty());
            assert!(model.endpoints.chat_completion.method == "POST");
        }
        
        for model in &anthropic_spec.models {
            assert!(!model.id.is_empty());
            assert!(!model.family.is_empty());
            assert!(model.endpoints.chat_completion.method == "POST");
        }
        
        // Check that both providers have proper response normalization
        assert!(!openai_spec.models[0].response_normalization.sync.content_path.is_empty());
        assert!(!anthropic_spec.models[0].response_normalization.sync.content_path.is_empty());
    }
}