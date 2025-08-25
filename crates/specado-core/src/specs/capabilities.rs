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
    
    /// Advanced capabilities for latest models
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_mode: Option<bool>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adaptive_reasoning: Option<bool>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deterministic_sampling: Option<bool>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advanced_coding: Option<bool>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balanced_performance: Option<bool>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agentic_tasks: Option<bool>,
    
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
            thinking_mode: None,
            adaptive_reasoning: None,
            deterministic_sampling: None,
            advanced_coding: None,
            balanced_performance: None,
            agentic_tasks: None,
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
        // First, check if capabilities are explicitly defined in the typed field
        if let Some(explicit_capabilities) = &model_spec.capabilities {
            log::debug!(
                "Using explicit capabilities for model '{}': vision={}, function_calling={}, streaming={}, reasoning={:?}",
                model_spec.id,
                explicit_capabilities.vision,
                explicit_capabilities.function_calling,
                explicit_capabilities.streaming,
                explicit_capabilities.reasoning
            );
            return explicit_capabilities.clone();
        }
        
        // If no explicit capabilities, infer from spec structure
        log::debug!(
            "No explicit capabilities found for model '{}', inferring from spec structure",
            model_spec.id
        );
        let inferred = Self::infer_capabilities_from_spec(model_spec);
        log::debug!(
            "Inferred capabilities for model '{}': vision={}, function_calling={}, streaming={}, reasoning={:?}",
            model_spec.id,
            inferred.vision,
            inferred.function_calling,
            inferred.streaming,
            inferred.reasoning
        );
        inferred
    }
    
    /// Infer capabilities from ModelSpec structure (spec-driven detection)
    fn infer_capabilities_from_spec(model_spec: &crate::types::ModelSpec) -> Capabilities {
        let mut capabilities = Capabilities::default();
        
        // Infer from input_modes
        capabilities.vision = model_spec.input_modes.images;
        
        // Infer from tooling config
        capabilities.function_calling = model_spec.tooling.tools_supported;
        
        // Infer streaming capability: check if streaming endpoint differs from regular endpoint
        capabilities.streaming = Self::detect_streaming_support(&model_spec.endpoints);
        
        // Infer from parameters (check for reasoning/thinking parameters)
        if let Some(params_obj) = model_spec.parameters.as_object() {
            capabilities.reasoning = if params_obj.contains_key("reasoning_depth") 
                || params_obj.contains_key("thinking_budget") 
                || params_obj.contains_key("reasoning_mode") {
                Some(true)
            } else {
                None
            };
                
            capabilities.extended_context = Self::detect_extended_context(params_obj);
            
            // Detect advanced capabilities from new parameters
            capabilities.thinking_mode = if params_obj.contains_key("thinking") 
                || params_obj.contains_key("min_thinking_tokens") {
                Some(true)
            } else {
                None
            };
            
            capabilities.adaptive_reasoning = if params_obj.contains_key("reasoning_effort") {
                Some(true)
            } else {
                None
            };
            
            capabilities.deterministic_sampling = if params_obj.contains_key("seed") {
                Some(true)
            } else {
                None
            };
            
            // Check for advanced capabilities from model ID patterns (no hardcoded names)
            let model_id_lower = model_spec.id.to_lowercase();
            
            capabilities.advanced_coding = if model_id_lower.contains("gpt-5") 
                || params_obj.contains_key("reasoning_effort") {
                Some(true)
            } else {
                None
            };
            
            capabilities.balanced_performance = if params_obj.contains_key("reasoning_mode") 
                && params_obj.contains_key("thinking_budget") {
                Some(true)
            } else {
                None
            };
            
            capabilities.agentic_tasks = if params_obj.contains_key("thinking") 
                && params_obj.contains_key("min_thinking_tokens") {
                Some(true)
            } else {
                None
            };
            
            // Extract experimental features from parameters
            for (key, value) in params_obj {
                if key.contains("thinking") || key.contains("reasoning") || key.contains("experimental") {
                    capabilities.experimental.insert(key.clone(), value.clone());
                }
            }
        }
        
        // Infer multimodal from input modes
        let mut modalities = vec!["text".to_string()];
        if model_spec.input_modes.images {
            modalities.push("image".to_string());
        }
        if modalities.len() > 1 {
            capabilities.multimodal = Some(modalities);
        }
        
        capabilities
    }
    
    /// Detect streaming support from endpoints configuration
    /// 
    /// Analyzes endpoint configuration to determine if streaming is supported through:
    /// 1. **Header Analysis**: Looks for SSE headers (Accept: text/event-stream) - primary method
    /// 2. **Endpoint Differentiation**: Compares regular vs streaming endpoints for differences
    /// 3. **Query Parameter Analysis**: Checks for streaming-specific parameters
    /// 
    /// # Note on Protocol Field
    /// The `protocol` field in EndpointConfig refers to transport protocol (http/https/ws/wss),
    /// not streaming technology. SSE runs over HTTP/HTTPS, so we rely on headers and other
    /// indicators to detect streaming capability.
    /// 
    /// # Resilience
    /// - Handles missing headers gracefully (None case)
    /// - Performs case-insensitive header/parameter checks
    /// - Falls back to endpoint/query comparison if headers missing
    /// 
    /// # Returns
    /// `true` if streaming support is detected, `false` otherwise
    fn detect_streaming_support(endpoints: &crate::types::Endpoints) -> bool {
        // 1. Check headers for SSE indicators (primary method - case-insensitive)
        if let Some(headers) = &endpoints.streaming_chat_completion.headers {
            for (key, value) in headers {
                let key_lower = key.to_lowercase();
                let value_lower = value.to_lowercase();
                
                // Look for SSE-specific headers (most reliable indicator)
                if key_lower.contains("accept") && value_lower.contains("text/event-stream") {
                    return true;
                }
                
                // Look for other streaming-related headers
                if value_lower.contains("stream") || key_lower.contains("stream") {
                    return true;
                }
            }
        }
        
        // 2. Check if streaming endpoint differs from regular endpoint
        // Different paths indicate separate streaming endpoint
        if endpoints.chat_completion.path.to_lowercase() != endpoints.streaming_chat_completion.path.to_lowercase() {
            return true;
        }
        
        // Different transport protocols (e.g., WebSocket for streaming)
        if endpoints.chat_completion.protocol.to_lowercase() != endpoints.streaming_chat_completion.protocol.to_lowercase() {
            return true;
        }
        
        // 3. Check for streaming-specific query parameters (case-insensitive)
        let regular_query = endpoints.chat_completion.query.as_ref();
        let streaming_query = endpoints.streaming_chat_completion.query.as_ref();
        
        if let (Some(reg_q), Some(stream_q)) = (regular_query, streaming_query) {
            // Compare query parameters (case-insensitive)
            let reg_params: std::collections::HashMap<String, String> = reg_q.iter()
                .map(|(k, v)| (k.to_lowercase(), v.to_lowercase()))
                .collect();
            let stream_params: std::collections::HashMap<String, String> = stream_q.iter()
                .map(|(k, v)| (k.to_lowercase(), v.to_lowercase()))
                .collect();
                
            // If streaming endpoint has stream=true or similar parameters
            if stream_params.values().any(|v| v.contains("true") && (v.contains("stream") || v == "true")) {
                return true;
            }
            
            // If queries differ, likely indicates streaming support
            if reg_params != stream_params {
                return true;
            }
        } else if streaming_query.is_some() && regular_query.is_none() {
            // Streaming endpoint has query params while regular doesn't
            return true;
        }
        
        false
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
        if model_spec.capabilities.is_some() {
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
        let mut confidence: f64 = 0.5; // Base confidence
        
        // Higher confidence if explicit capabilities are defined
        if model_spec.capabilities.is_some() {
            confidence = 0.95;
        } else {
            // Increase confidence based on spec completeness
            if model_spec.input_modes.images {
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
    
    use crate::types::{ModelSpec, Endpoints, EndpointConfig, InputModes, ToolingConfig, JsonOutputConfig, Constraints, ConstraintLimits, Mappings, ResponseNormalization, SyncNormalization, StreamNormalization, EventSelector};
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
                    query: None,
                    headers: None,
                },
                streaming_chat_completion: EndpointConfig {
                    method: "POST".to_string(),
                    path: "/chat".to_string(),
                    protocol: "sse".to_string(),
                    query: None,
                    headers: None,
                },
            },
            input_modes: InputModes {
                messages: true,
                single_text: true,
                images: true,
            },
            tooling: ToolingConfig {
                tools_supported: true,
                parallel_tool_calls_default: true,
                can_disable_parallel_tool_calls: true,
                disable_switch: None,
            },
            json_output: JsonOutputConfig {
                native_param: true,
                strategy: "json_schema".to_string(),
            },
            capabilities: None, // Test inference from spec structure
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
                forbid_unknown_top_level_fields: true,
                mutually_exclusive: vec![],
                resolution_preferences: vec!["max_tokens".to_string()],
                limits: ConstraintLimits {
                    max_tool_schema_bytes: 16384,
                    max_system_prompt_bytes: 32768,
                },
            },
            mappings: Mappings {
                paths: std::collections::HashMap::new(),
                flags: std::collections::HashMap::new(),
            },
            response_normalization: ResponseNormalization {
                sync: SyncNormalization {
                    content_path: "content".to_string(),
                    finish_reason_path: "finish_reason".to_string(),
                    finish_reason_map: std::collections::HashMap::new(),
                },
                stream: StreamNormalization {
                    protocol: "sse".to_string(),
                    event_selector: EventSelector {
                        type_path: "type".to_string(),
                        routes: vec![],
                    },
                },
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
        explicit_spec.capabilities = Some(Capabilities {
            text_generation: true,
            vision: true,
            function_calling: true,
            streaming: true,
            reasoning: Some(true),
            extended_context: None,
            thinking_mode: None,
            adaptive_reasoning: None,
            deterministic_sampling: None,
            advanced_coding: None,
            balanced_performance: None,
            agentic_tasks: None,
            multimodal: None,
            experimental: HashMap::new(),
        });
        
        let explicit_confidence = CapabilityDetector::get_detection_confidence(&explicit_spec);
        assert_eq!(explicit_confidence, 0.95); // Maximum confidence for explicit capabilities
    }
    
    #[test]
    fn test_discovery_flags_default() {
        let flags = DiscoveryFlags::default();
        assert!(!flags.enable_capability_discovery); // Safe default
    }
}