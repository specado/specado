// Universal LLM Adapter - Model Definitions
// Based on AIML API research (archived in research/aiml-analysis-2025-01-31/)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core model capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelCapability {
    TextGeneration,
    VisionProcessing,
    AudioProcessing,
    CodeGeneration,
    ReasoningMode,
    ThinkingMode,
    ToolUsage,
    MultiModal,
    AdaptiveReasoning,
}

/// Reasoning effort levels for advanced models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
}

/// Universal model definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDefinition {
    pub model_ids: Vec<String>,
    pub provider: String,
    pub name: String,
    pub description: String,
    pub capabilities: Vec<ModelCapability>,
    pub max_tokens: Option<u32>,
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub special_features: HashMap<String, serde_json::Value>,
}

/// Latest flagship models - PRIORITY TIER 1
pub fn get_tier1_models() -> Vec<ModelDefinition> {
    vec![
        // Claude Opus 4.1 - Most Advanced Reasoning
        ModelDefinition {
            model_ids: vec![
                "anthropic/claude-opus-4.1".to_string(),
                "claude-opus-4-1".to_string(),
                "claude-opus-4-1-20250805".to_string(),
            ],
            provider: "Anthropic".to_string(),
            name: "Claude Opus 4.1".to_string(),
            description: "Advanced agentic tasks, real-world coding, and thinking mode".to_string(),
            capabilities: vec![
                ModelCapability::TextGeneration,
                ModelCapability::CodeGeneration,
                ModelCapability::ReasoningMode,
                ModelCapability::ThinkingMode,
                ModelCapability::ToolUsage,
                ModelCapability::MultiModal,
            ],
            max_tokens: Some(200000),
            supports_streaming: true,
            supports_tools: true,
            special_features: {
                let mut features = HashMap::new();
                features.insert("thinking_mode".to_string(), true.into());
                features.insert("min_thinking_tokens".to_string(), 1024.into());
                features.insert("release_date".to_string(), "2025-08-05".into());
                features
            },
        },
        
        // Claude 4 Sonnet - Balanced Excellence
        ModelDefinition {
            model_ids: vec!["anthropic/claude-sonnet-4".to_string()],
            provider: "Anthropic".to_string(),
            name: "Claude 4 Sonnet".to_string(),
            description: "Major improvement over Claude 3.7 Sonnet with better coding and reasoning".to_string(),
            capabilities: vec![
                ModelCapability::TextGeneration,
                ModelCapability::CodeGeneration,
                ModelCapability::ReasoningMode,
                ModelCapability::ToolUsage,
            ],
            max_tokens: Some(200000),
            supports_streaming: true,
            supports_tools: true,
            special_features: {
                let mut features = HashMap::new();
                features.insert("version".to_string(), "20250514".into());
                features.insert("coding_improvement".to_string(), "major".into());
                features
            },
        },
        
        // GPT-5 - Most Advanced Coding
        ModelDefinition {
            model_ids: vec!["openai/gpt-5-2025-08-07".to_string()],
            provider: "OpenAI".to_string(),
            name: "GPT-5".to_string(),
            description: "OpenAI's most advanced model with adaptive reasoning and coding excellence".to_string(),
            capabilities: vec![
                ModelCapability::TextGeneration,
                ModelCapability::CodeGeneration,
                ModelCapability::AdaptiveReasoning,
                ModelCapability::ToolUsage,
                ModelCapability::VisionProcessing,
                ModelCapability::AudioProcessing,
            ],
            max_tokens: Some(128000),
            supports_streaming: true,
            supports_tools: true,
            special_features: {
                let mut features = HashMap::new();
                features.insert("reasoning_effort_configurable".to_string(), true.into());
                features.insert("context_router".to_string(), "real_time".into());
                features.insert("release_date".to_string(), "2025-08-07".into());
                features.insert("deterministic_sampling".to_string(), true.into());
                features
            },
        },
        
        // Gemini 2.5 Pro - Advanced Analytics
        ModelDefinition {
            model_ids: vec!["google/gemini-2.5-pro".to_string()],
            provider: "Google".to_string(),
            name: "Gemini 2.5 Pro".to_string(),
            description: "Reasoning through thoughts with enhanced performance and accuracy".to_string(),
            capabilities: vec![
                ModelCapability::TextGeneration,
                ModelCapability::ReasoningMode,
                ModelCapability::CodeGeneration,
                ModelCapability::VisionProcessing,
            ],
            max_tokens: Some(50000),
            supports_streaming: true,
            supports_tools: true,
            special_features: {
                let mut features = HashMap::new();
                features.insert("reasoning_tokens".to_string(), true.into());
                features.insert("high_token_generation".to_string(), 45000.into());
                features.insert("thought_process_visible".to_string(), true.into());
                features
            },
        },
    ]
}

/// Standard parameters that work across all models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalParameters {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,    // 0.0-2.0
    pub stream: Option<bool>,
    pub top_p: Option<f32>,         // 0.0-1.0
    pub presence_penalty: Option<f32>,
    pub frequency_penalty: Option<f32>,
}

/// Advanced parameters for latest models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedParameters {
    pub reasoning_effort: Option<ReasoningEffort>,
    pub thinking: Option<bool>,
    pub tools: Option<Vec<serde_json::Value>>,
    pub tool_choice: Option<String>,
    pub seed: Option<u32>,
    pub system: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Get model by ID
pub fn get_model_by_id(model_id: &str) -> Option<ModelDefinition> {
    get_tier1_models()
        .into_iter()
        .find(|model| model.model_ids.contains(&model_id.to_string()))
}

/// Check if model supports capability
pub fn model_supports_capability(model_id: &str, capability: &ModelCapability) -> bool {
    if let Some(model) = get_model_by_id(model_id) {
        model.capabilities.contains(capability)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier1_models_loaded() {
        let models = get_tier1_models();
        assert_eq!(models.len(), 4);
        
        // Verify critical models are present
        assert!(get_model_by_id("anthropic/claude-opus-4.1").is_some());
        assert!(get_model_by_id("anthropic/claude-sonnet-4").is_some());
        assert!(get_model_by_id("openai/gpt-5-2025-08-07").is_some());
        assert!(get_model_by_id("google/gemini-2.5-pro").is_some());
    }

    #[test]
    fn test_capability_detection() {
        assert!(model_supports_capability("anthropic/claude-opus-4.1", &ModelCapability::ThinkingMode));
        assert!(model_supports_capability("openai/gpt-5-2025-08-07", &ModelCapability::AdaptiveReasoning));
        assert!(model_supports_capability("google/gemini-2.5-pro", &ModelCapability::ReasoningMode));
    }
}