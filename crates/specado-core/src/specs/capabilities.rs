//! Capability types and detection for modern LLM models
//!
//! This module defines capability structures that extend the existing ModelSpec
//! with optional capability metadata. It's designed to work with the existing
//! provider_discovery module for runtime capability detection.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Capability metadata for models (optional extension to ModelSpec)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    /// Core capabilities (always present)
    pub text_generation: bool,
    pub vision: bool,
    pub function_calling: bool,
    pub streaming: bool,
    
    /// Modern capabilities (optional for new models)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<bool>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_context: Option<bool>,
    
    /// Multimodal capabilities (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multimodal: Option<Vec<String>>,
    
    /// Future-proofing for unknown capabilities
    #[serde(flatten)]
    pub experimental: HashMap<String, serde_json::Value>,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            text_generation: true,
            vision: false,
            function_calling: false,
            streaming: false,
            reasoning: None,
            extended_context: None,
            multimodal: None,
            experimental: HashMap::new(),
        }
    }
}

/// Cached capability information with TTL
#[derive(Debug, Clone)]
pub struct CachedCapabilities {
    pub capabilities: Capabilities,
    pub cached_at: DateTime<Utc>,
    pub confidence: f64,
}

impl CachedCapabilities {
    /// Check if cached capabilities are still fresh (1 hour TTL)
    pub fn is_fresh(&self) -> bool {
        let age = Utc::now().signed_duration_since(self.cached_at);
        age.num_seconds() < 3600 // 1 hour TTL
    }
}

/// Feature flags for capability discovery
#[derive(Debug, Clone)]
pub struct DiscoveryFlags {
    /// Enable capability discovery (default: false for safety)
    pub enable_capability_discovery: bool,
}

impl Default for DiscoveryFlags {
    fn default() -> Self {
        Self {
            enable_capability_discovery: false,
        }
    }
}

/// Capability detection utilities (spec-driven, no hardcoded model names)
pub struct CapabilityDetector;

impl CapabilityDetector {
    /// Extract capabilities from a ModelSpec's capabilities field or infer from spec structure
    pub fn extract_capabilities_from_spec(model_spec: &crate::types::ModelSpec) -> Capabilities {
        // First, check if capabilities are explicitly defined in the spec
        if let Some(capabilities_value) = model_spec.parameters.get("capabilities") {
            if let Ok(capabilities) = serde_json::from_value::<Capabilities>(capabilities_value.clone()) {
                return capabilities;
            }
        }
        
        // If no explicit capabilities, infer from spec structure
        Self::infer_capabilities_from_spec(model_spec)
    }
    
    /// Infer capabilities from ModelSpec structure (spec-driven detection)
    fn infer_capabilities_from_spec(model_spec: &crate::types::ModelSpec) -> Capabilities {
        let mut capabilities = Capabilities::default();
        
        // Infer from input_modes
        capabilities.vision = model_spec.input_modes.images.unwrap_or(false);
        
        // Infer from tooling config
        capabilities.function_calling = model_spec.tooling.tools_supported;
        capabilities.streaming = model_spec.endpoints.streaming_chat_completion.is_some();
        
        // Infer from parameters (check for reasoning/thinking parameters)
        if let Some(params_obj) = model_spec.parameters.as_object() {
            capabilities.reasoning = params_obj.contains_key("reasoning_depth")
                .or(params_obj.contains_key("thinking_budget"))
                .or(params_obj.contains_key("reasoning_mode"))
                .then_some(true);
                
            capabilities.extended_context = Self::detect_extended_context(params_obj);
            
            // Extract experimental features from parameters
            for (key, value) in params_obj {
                if key.contains("thinking") || key.contains("reasoning") || key.contains("experimental") {
                    capabilities.experimental.insert(key.clone(), value.clone());
                }
            }
        }
        
        // Infer multimodal from input modes
        let mut modalities = vec!["text".to_string()];
        if model_spec.input_modes.images.unwrap_or(false) {
            modalities.push("image".to_string());
        }
        if let Some(audio) = model_spec.input_modes.audio {
            if audio {
                modalities.push("audio".to_string());
            }
        }
        if modalities.len() > 1 {
            capabilities.multimodal = Some(modalities);
        }
        
        capabilities
    }
    
    /// Detect extended context from parameters (look for large max_tokens values)
    fn detect_extended_context(params: &serde_json::Map<String, serde_json::Value>) -> Option<bool> {
        // Look for max_tokens or context_window parameters
        for key in ["max_tokens", "max_output_tokens", "context_window"] {
            if let Some(value) = params.get(key) {
                if let Some(max_val) = value.as_u64() {
                    // Consider >100k tokens as "extended context"
                    if max_val > 100_000 {
                        return Some(true);
                    }
                }
                if let Some(obj) = value.as_object() {
                    if let Some(max_val) = obj.get("maximum").and_then(|v| v.as_u64()) {
                        if max_val > 100_000 {
                            return Some(true);
                        }
                    }
                }
            }
        }
        None
    }
    
    /// Check if a model has capabilities defined (either explicit or inferable)
    pub fn has_enhanced_capabilities(model_spec: &crate::types::ModelSpec) -> bool {
        // Check for explicit capabilities field
        if model_spec.parameters.get("capabilities").is_some() {
            return true;
        }
        
        // Check for modern capability indicators in parameters
        if let Some(params_obj) = model_spec.parameters.as_object() {
            return params_obj.keys().any(|k| {
                k.contains("reasoning") || k.contains("thinking") || k.contains("experimental")
            });
        }
        
        false
    }
    
    /// Get confidence score based on spec completeness
    pub fn get_detection_confidence(model_spec: &crate::types::ModelSpec) -> f64 {
        let mut confidence = 0.5; // Base confidence
        
        // Higher confidence if explicit capabilities are defined
        if model_spec.parameters.get("capabilities").is_some() {
            confidence = 0.95;
        } else {
            // Increase confidence based on spec completeness
            if model_spec.input_modes.images.is_some() {
                confidence += 0.1;
            }
            if model_spec.tooling.tools_supported {
                confidence += 0.1;
            }
            if let Some(params_obj) = model_spec.parameters.as_object() {
                if params_obj.contains_key("reasoning_depth") || params_obj.contains_key("thinking_budget") {
                    confidence += 0.2;
                }
            }
        }
        
        confidence.min(0.95) // Cap at 95%
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_capabilities_default() {
        let caps = Capabilities::default();
        assert!(caps.text_generation);
        assert!(!caps.vision);
        assert!(!caps.function_calling);
        assert!(!caps.streaming);
        assert!(caps.reasoning.is_none());
        assert!(caps.extended_context.is_none());
    }
    
    #[test]
    fn test_capabilities_serialization() {
        let mut caps = Capabilities::default();
        caps.reasoning = Some(true);
        caps.experimental.insert("test_feature".to_string(), serde_json::Value::Bool(true));
        
        let json = serde_json::to_string(&caps).unwrap();
        let deserialized: Capabilities = serde_json::from_str(&json).unwrap();
        
        assert_eq!(caps.reasoning, deserialized.reasoning);
        assert_eq!(caps.experimental, deserialized.experimental);
    }
    
    #[test]
    fn test_cached_capabilities_freshness() {
        let cached = CachedCapabilities {
            capabilities: Capabilities::default(),
            cached_at: Utc::now(),
            confidence: 0.9,
        };
        
        assert!(cached.is_fresh());
        
        let old_cached = CachedCapabilities {
            capabilities: Capabilities::default(),
            cached_at: Utc::now() - chrono::Duration::hours(2),
            confidence: 0.9,
        };
        
        assert!(!old_cached.is_fresh());
    }
    
    use crate::types::{ModelSpec, ProviderInfo, Endpoints, EndpointConfig, InputModes, ToolingConfig, JsonOutputConfig, Constraints, Mappings, ResponseNormalization};
    use serde_json::json;
    
    fn create_test_model_spec() -> ModelSpec {
        ModelSpec {
            id: "test-model".to_string(),
            aliases: None,
            family: "test".to_string(),
            endpoints: Endpoints {
                chat_completion: EndpointConfig {
                    method: "POST".to_string(),
                    path: "/chat".to_string(),
                    protocol: "https".to_string(),
                },
                streaming_chat_completion: Some(EndpointConfig {
                    method: "POST".to_string(),
                    path: "/chat".to_string(),
                    protocol: "sse".to_string(),
                }),
            },
            input_modes: InputModes {
                messages: true,
                single_text: Some(true),
                images: Some(true),
                audio: Some(false),
                video: Some(false),
            },
            tooling: ToolingConfig {
                tools_supported: true,
                parallel_tool_calls_default: true,
                can_disable_parallel_tool_calls: Some(true),
                custom_tools: Some(true),
                preambles_supported: Some(true),
                tool_types: Some(vec!["function".to_string()]),
                context_free_grammars: Some(false),
                tool_choice_modes: Some(vec!["auto".to_string(), "required".to_string()]),
            },
            json_output: JsonOutputConfig {
                native_param: true,
                strategy: "json_schema".to_string(),
                notes: Some("Test JSON output".to_string()),
            },
            parameters: json!({
                "max_tokens": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 200000
                },
                "reasoning_depth": {
                    "type": "string",
                    "enum": ["shallow", "deep"],
                    "default": "shallow"
                },
                "thinking_budget": {
                    "type": "integer",
                    "minimum": 1000,
                    "maximum": 65536
                }
            }),
            constraints: Constraints {
                system_prompt_location: "system_parameter".to_string(),
                forbid_unknown_top_level_fields: Some(true),
                required_fields: Some(vec!["model".to_string()]),
                mutually_exclusive: Some(vec![]),
                resolution_preferences: Some(vec!["max_tokens".to_string()]),
                limits: Some(json!({})),
            },
            mappings: Mappings {
                paths: Some(std::collections::HashMap::new()),
                transformations: Some(std::collections::HashMap::new()),
                flags: Some(std::collections::HashMap::new()),
            },
            response_normalization: ResponseNormalization {
                sync: json!({}),
                stream: Some(json!({})),
            },
        }
    }
    
    #[test]
    fn test_spec_driven_capability_extraction() {
        let model_spec = create_test_model_spec();
        let capabilities = CapabilityDetector::extract_capabilities_from_spec(&model_spec);
        
        // Should infer from spec structure
        assert!(capabilities.text_generation);
        assert!(capabilities.vision); // From input_modes.images = true
        assert!(capabilities.function_calling); // From tooling.tools_supported = true
        assert!(capabilities.streaming); // From streaming_chat_completion endpoint
        assert_eq!(capabilities.reasoning, Some(true)); // From reasoning_depth parameter
        assert_eq!(capabilities.extended_context, Some(true)); // From max_tokens: 200000
    }
    
    #[test]
    fn test_extended_context_detection() {
        let params = json!({
            "max_tokens": {
                "maximum": 200000
            }
        });
        
        let extended = CapabilityDetector::detect_extended_context(params.as_object().unwrap());
        assert_eq!(extended, Some(true));
        
        let params_small = json!({
            "max_tokens": {
                "maximum": 4000
            }
        });
        
        let not_extended = CapabilityDetector::detect_extended_context(params_small.as_object().unwrap());
        assert_eq!(not_extended, None);
    }
    
    #[test]
    fn test_has_enhanced_capabilities() {
        let model_spec = create_test_model_spec();
        assert!(CapabilityDetector::has_enhanced_capabilities(&model_spec));
        
        // Test spec without enhanced capabilities
        let mut basic_spec = create_test_model_spec();
        basic_spec.parameters = json!({
            "temperature": {
                "type": "number",
                "minimum": 0.0,
                "maximum": 2.0
            }
        });
        
        assert!(!CapabilityDetector::has_enhanced_capabilities(&basic_spec));
    }
    
    #[test]
    fn test_confidence_scoring() {
        let model_spec = create_test_model_spec();
        let confidence = CapabilityDetector::get_detection_confidence(&model_spec);
        assert!(confidence > 0.7); // Should be high confidence due to reasoning parameters
        
        // Test with explicit capabilities field
        let mut explicit_spec = create_test_model_spec();
        explicit_spec.parameters.as_object_mut().unwrap().insert(
            "capabilities".to_string(), 
            json!({
                "text_generation": true,
                "vision": true,
                "reasoning": true
            })
        );
        
        let explicit_confidence = CapabilityDetector::get_detection_confidence(&explicit_spec);
        assert_eq!(explicit_confidence, 0.95); // Maximum confidence for explicit capabilities
    }
    
    #[test]
    fn test_discovery_flags_default() {
        let flags = DiscoveryFlags::default();
        assert!(!flags.enable_capability_discovery); // Safe default
    }
}