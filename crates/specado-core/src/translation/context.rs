//! Translation context for managing translation state and configuration
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::{ModelSpec, PromptSpec, ProviderSpec, StrictMode};
use std::collections::HashMap;

/// Context for managing translation state and configuration
///
/// The TranslationContext holds all the necessary information for translating
/// a PromptSpec to a provider-specific format. It maintains references to the
/// input specifications and tracks state during the translation process.
#[derive(Debug, Clone)]
pub struct TranslationContext {
    /// The input prompt specification
    pub prompt_spec: PromptSpec,
    
    /// The provider specification
    pub provider_spec: ProviderSpec,
    
    /// The specific model being used
    pub model_spec: ModelSpec,
    
    /// The strictness mode for this translation
    pub strict_mode: StrictMode,
    
    /// Custom context data for tracking state
    pub custom_data: HashMap<String, serde_json::Value>,
}

impl TranslationContext {
    /// Create a new translation context
    pub fn new(
        prompt_spec: PromptSpec,
        provider_spec: ProviderSpec,
        model_spec: ModelSpec,
        strict_mode: StrictMode,
    ) -> Self {
        Self {
            prompt_spec,
            provider_spec,
            model_spec,
            strict_mode,
            custom_data: HashMap::new(),
        }
    }

    /// Get the provider name
    pub fn provider_name(&self) -> &str {
        &self.provider_spec.provider.name
    }

    /// Get the model ID
    pub fn model_id(&self) -> &str {
        &self.model_spec.id
    }

    /// Check if tools are supported
    pub fn supports_tools(&self) -> bool {
        self.model_spec.tooling.tools_supported
    }

    /// Check if native JSON output is supported
    pub fn supports_native_json(&self) -> bool {
        self.model_spec.json_output.native_param
    }

    /// Check if images are supported
    pub fn supports_images(&self) -> bool {
        self.model_spec.input_modes.images
    }

    /// Get the system prompt location preference
    pub fn system_prompt_location(&self) -> &str {
        &self.model_spec.constraints.system_prompt_location
    }

    /// Add custom data to the context
    pub fn set_custom_data(&mut self, key: String, value: serde_json::Value) {
        self.custom_data.insert(key, value);
    }

    /// Get custom data from the context
    pub fn get_custom_data(&self, key: &str) -> Option<&serde_json::Value> {
        self.custom_data.get(key)
    }

    /// Check if we should fail on errors based on strict mode
    pub fn should_fail_on_error(&self) -> bool {
        matches!(self.strict_mode, StrictMode::Strict)
    }

    /// Check if we should warn on issues
    pub fn should_warn(&self) -> bool {
        matches!(self.strict_mode, StrictMode::Warn | StrictMode::Strict)
    }

    /// Check if we should auto-coerce values
    pub fn should_coerce(&self) -> bool {
        matches!(self.strict_mode, StrictMode::Coerce)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Constraints, ConstraintLimits, EndpointConfig, Endpoints, InputModes, JsonOutputConfig,
        Mappings, Message, MessageRole, ProviderInfo, ResponseNormalization, StreamNormalization,
        SyncNormalization, ToolingConfig,
    };
    use std::collections::HashMap;

    fn create_test_context() -> TranslationContext {
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
            advanced: None,
            strict_mode: StrictMode::Warn,
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
                images: true,
            },
            tooling: ToolingConfig {
                tools_supported: true,
                parallel_tool_calls_default: false,
                can_disable_parallel_tool_calls: false,
                disable_switch: None,
            },
            json_output: JsonOutputConfig {
                native_param: true,
                strategy: "native".to_string(),
            },
            capabilities: None,
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

        TranslationContext::new(prompt_spec, provider_spec, model_spec, StrictMode::Warn)
    }

    #[test]
    fn test_context_creation() {
        let context = create_test_context();
        assert_eq!(context.provider_name(), "test-provider");
        assert_eq!(context.model_id(), "test-model");
    }

    #[test]
    fn test_capability_checks() {
        let context = create_test_context();
        assert!(context.supports_tools());
        assert!(context.supports_native_json());
        assert!(context.supports_images());
    }

    #[test]
    fn test_custom_data() {
        let mut context = create_test_context();
        context.set_custom_data("test_key".to_string(), serde_json::json!("test_value"));
        
        assert_eq!(
            context.get_custom_data("test_key"),
            Some(&serde_json::json!("test_value"))
        );
        assert_eq!(context.get_custom_data("missing_key"), None);
    }

    #[test]
    fn test_strict_mode_checks() {
        let mut context = create_test_context();
        
        // Test Warn mode
        context.strict_mode = StrictMode::Warn;
        assert!(!context.should_fail_on_error());
        assert!(context.should_warn());
        assert!(!context.should_coerce());
        
        // Test Strict mode
        context.strict_mode = StrictMode::Strict;
        assert!(context.should_fail_on_error());
        assert!(context.should_warn());
        assert!(!context.should_coerce());
        
        // Test Coerce mode
        context.strict_mode = StrictMode::Coerce;
        assert!(!context.should_fail_on_error());
        assert!(!context.should_warn());
        assert!(context.should_coerce());
    }
}
