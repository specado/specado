//! Enhanced capabilities for modern LLM models
//!
//! This module provides minimal extensions to the existing ModelSpec type
//! to support capability discovery for modern models. It builds on the existing
//! provider_discovery module and maintains full backward compatibility.

pub mod capabilities;

// Re-export main types
pub use capabilities::{
    Capabilities, CachedCapabilities, CapabilityDetector, DiscoveryFlags
};

/// Current specification version for capability extensions
pub const ENHANCED_SPEC_VERSION: &str = "1.1.0";

/// Check if a model uses the modern specification system (based on capabilities field)
pub fn is_modern_model(model_spec: &crate::types::ModelSpec) -> bool {
    model_spec.capabilities.is_some() || CapabilityDetector::has_enhanced_capabilities(model_spec)
}

// TODO: Implement enhanced validation when registry types are available
// /// Validate a prompt specification against enhanced model capabilities
// pub async fn validate_enhanced_prompt(
//     registry: &EnhancedSpecRegistry,
//     provider: &str,
//     model: &str,
//     prompt_spec: &crate::PromptSpec,
// ) -> Result<ValidationResult, ValidationError> {
//     // Implementation will be added when registry types are defined
//     unimplemented!("Enhanced validation will be implemented in future task")
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use serde_json::json;
    use std::collections::HashMap;

    fn create_basic_model_spec() -> ModelSpec {
        ModelSpec {
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
                images: false,
            },
            tooling: ToolingConfig {
                tools_supported: false,
                parallel_tool_calls_default: false,
                can_disable_parallel_tool_calls: false,
                disable_switch: None,
            },
            json_output: JsonOutputConfig {
                native_param: false,
                strategy: "none".to_string(),
            },
            capabilities: None,
            parameters: json!({}),
            constraints: Constraints {
                system_prompt_location: "first".to_string(),
                forbid_unknown_top_level_fields: false,
                mutually_exclusive: vec![],
                resolution_preferences: vec![],
                limits: ConstraintLimits {
                    max_tool_schema_bytes: 8192,
                    max_system_prompt_bytes: 16384,
                },
            },
            mappings: Mappings {
                paths: HashMap::new(),
                flags: HashMap::new(),
            },
            response_normalization: ResponseNormalization {
                sync: SyncNormalization {
                    content_path: "$.content".to_string(),
                    finish_reason_path: "$.finish_reason".to_string(),
                    finish_reason_map: HashMap::new(),
                },
                stream: StreamNormalization {
                    protocol: "sse".to_string(),
                    event_selector: EventSelector {
                        type_path: "$.type".to_string(),
                        routes: vec![],
                    },
                },
            },
        }
    }

    #[test]
    fn test_is_modern_model_with_explicit_capabilities() {
        let mut model_spec = create_basic_model_spec();
        model_spec.capabilities = Some(capabilities::Capabilities {
            text_generation: true,
            vision: false,
            function_calling: false,
            streaming: false,
            reasoning: None,
            extended_context: None,
            multimodal: None,
            experimental: HashMap::new(),
        });

        assert!(is_modern_model(&model_spec), "Model with explicit capabilities should be considered modern");
    }

    #[test] 
    fn test_is_modern_model_inferable_via_parameters() {
        let mut model_spec = create_basic_model_spec();
        model_spec.capabilities = None;
        model_spec.parameters = json!({
            "reasoning_depth": {
                "type": "integer",
                "minimum": 1,
                "maximum": 10
            },
            "thinking_budget": {
                "type": "number",
                "minimum": 0.1,
                "maximum": 2.0
            }
        });

        assert!(is_modern_model(&model_spec), "Model with reasoning parameters should be considered modern");
    }

    #[test]
    fn test_is_modern_model_inferable_via_experimental() {
        let mut model_spec = create_basic_model_spec();
        model_spec.capabilities = None;
        model_spec.parameters = json!({
            "experimental_feature": true,
            "max_tokens": 4096
        });

        assert!(is_modern_model(&model_spec), "Model with experimental parameters should be considered modern");
    }

    #[test]
    fn test_is_modern_model_inferable_via_thinking() {
        let mut model_spec = create_basic_model_spec();
        model_spec.capabilities = None;
        model_spec.parameters = json!({
            "thinking_mode": "enabled",
            "temperature": 0.7
        });

        assert!(is_modern_model(&model_spec), "Model with thinking parameters should be considered modern");
    }

    #[test]
    fn test_is_modern_model_neither_present_nor_inferable() {
        let mut model_spec = create_basic_model_spec();
        model_spec.capabilities = None;
        model_spec.parameters = json!({
            "temperature": 0.7,
            "max_tokens": 1000,
            "top_p": 0.9
        });

        assert!(!is_modern_model(&model_spec), "Model without capabilities or enhanced parameters should not be considered modern");
    }

    #[test]
    fn test_is_modern_model_empty_parameters() {
        let mut model_spec = create_basic_model_spec();
        model_spec.capabilities = None;
        model_spec.parameters = json!({});

        assert!(!is_modern_model(&model_spec), "Model with empty parameters should not be considered modern");
    }

    #[test]
    fn test_is_modern_model_explicit_takes_priority() {
        let mut model_spec = create_basic_model_spec();
        model_spec.capabilities = Some(capabilities::Capabilities::default());
        // Even with no enhanced parameters, explicit capabilities make it modern
        model_spec.parameters = json!({
            "temperature": 0.7
        });

        assert!(is_modern_model(&model_spec), "Explicit capabilities should take priority over parameter inference");
    }
}