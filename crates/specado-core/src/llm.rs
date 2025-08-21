//! High-level LLM interface for simplified usage
//! 
//! This module provides a user-friendly API that abstracts away the complexity
//! of provider specifications and parameter mapping.

use crate::{
    error::{Error, Result},
    types::{PromptSpec, ProviderSpec, UniformResponse, TranslationResult},
    translation::translate,
    StrictMode,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// High-level LLM interface for simplified interactions
pub struct LLM {
    /// The model identifier (e.g., "gpt-5", "claude-opus-4.1")
    model: String,
    /// The normalized model name for internal use
    normalized_model: String,
    /// The provider specification
    provider_spec: ProviderSpec,
    /// The API format type (responses or messages)
    api_format: ApiFormat,
}

/// API format types for different providers
#[derive(Debug, Clone, PartialEq)]
enum ApiFormat {
    /// GPT-5 family Responses API
    Responses,
    /// Claude Messages API
    Messages,
}

/// Preset modes for common parameter configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerationMode {
    /// Balanced parameters for general use
    Balanced,
    /// High creativity settings
    Creative,
    /// High precision settings
    Precise,
    /// Fast generation with lower quality
    Fast,
    /// Custom parameters
    Custom(HashMap<String, Value>),
}

impl LLM {
    /// Create a new LLM instance for the specified model
    pub fn new(model: &str) -> Result<Self> {
        let normalized_model = Self::normalize_model_name(model)?;
        let provider_spec = Self::load_builtin_spec(&normalized_model)?;
        let api_format = Self::detect_api_format(&normalized_model);
        
        Ok(Self {
            model: model.to_string(),
            normalized_model,
            provider_spec,
            api_format,
        })
    }
    
    /// Generate text with a simple prompt
    pub async fn generate(
        &self,
        prompt: &str,
        mode: GenerationMode,
        max_tokens: Option<u32>,
    ) -> Result<UniformResponse> {
        let prompt_spec = self.build_prompt_spec(prompt, mode, max_tokens)?;
        let translation_result = translate(
            &prompt_spec,
            &self.provider_spec,
            &self.normalized_model,
            StrictMode::Warn,
        )?;
        
        // Execute the request
        let response = crate::run(&translation_result.provider_request_json).await?;
        Ok(response)
    }
    
    /// Generate text with a simple prompt (synchronous version)
    pub fn generate_sync(
        &self,
        prompt: &str,
        mode: GenerationMode,
        max_tokens: Option<u32>,
    ) -> Result<UniformResponse> {
        let prompt_spec = self.build_prompt_spec(prompt, mode, max_tokens)?;
        let translation_result = translate(
            &prompt_spec,
            &self.provider_spec,
            &self.normalized_model,
            StrictMode::Warn,
        )?;
        
        // Use tokio runtime for sync execution
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| Error::Unsupported {
            message: format!("Failed to create runtime: {}", e),
            feature: Some("sync_runtime".to_string()),
        })?;
        
        runtime.block_on(crate::run(&translation_result.provider_request_json))
    }
    
    /// Chat with message history
    pub async fn chat(
        &self,
        messages: Vec<Message>,
        mode: GenerationMode,
        max_tokens: Option<u32>,
    ) -> Result<UniformResponse> {
        let prompt_spec = self.build_chat_spec(messages, mode, max_tokens)?;
        let translation_result = translate(
            &prompt_spec,
            &self.provider_spec,
            &self.normalized_model,
            StrictMode::Warn,
        )?;
        
        let response = crate::run(&translation_result.provider_request_json).await?;
        Ok(response)
    }
    
    // Stream responses will be implemented in a future version
    // For now, this is commented out to avoid type inference issues
    /*
    /// Stream responses (returns an iterator of response chunks)
    pub async fn stream(
        &self,
        prompt: &str,
        mode: GenerationMode,
        max_tokens: Option<u32>,
    ) -> Result<impl futures::Stream<Item = Result<String>>> {
        // TODO: Implement streaming support
        Err(Error::Unsupported {
            message: "Streaming not yet implemented".to_string(),
            feature: Some("streaming".to_string()),
        })
    }
    */
    
    /// Build a prompt specification from parameters
    fn build_prompt_spec(
        &self,
        prompt: &str,
        mode: GenerationMode,
        max_tokens: Option<u32>,
    ) -> Result<PromptSpec> {
        let params = self.map_mode_to_params(&mode);
        
        match self.api_format {
            ApiFormat::Responses => {
                // GPT-5 uses single input field
                let mut spec = serde_json::json!({
                    "model_class": "Responses",
                    "input": prompt,
                    "strict_mode": "Warn"
                });
                
                // Add mapped parameters
                if let Some(obj) = spec.as_object_mut() {
                    for (key, value) in params {
                        obj.insert(key, value);
                    }
                    if let Some(tokens) = max_tokens {
                        obj.insert("max_output_tokens".to_string(), tokens.into());
                    }
                }
                
                serde_json::from_value(spec)
                    .map_err(|e| Error::Unsupported {
            message: format!("Failed to build prompt spec: {}", e),
            feature: None,
        })
            }
            ApiFormat::Messages => {
                // Claude uses messages array
                let mut spec = serde_json::json!({
                    "model_class": "Chat",
                    "messages": [
                        {"role": "user", "content": prompt}
                    ],
                    "strict_mode": "Warn"
                });
                
                // Add sampling parameters
                let mut sampling = serde_json::Map::new();
                for (key, value) in params {
                    if key.starts_with("sampling.") {
                        let param_name = key.strip_prefix("sampling.").unwrap_or(&key);
                        sampling.insert(param_name.to_string(), value);
                    }
                }
                if let Some(tokens) = max_tokens {
                    sampling.insert("max_tokens".to_string(), tokens.into());
                }
                
                if let Some(obj) = spec.as_object_mut() {
                    if !sampling.is_empty() {
                        obj.insert("sampling".to_string(), sampling.into());
                    }
                }
                
                serde_json::from_value(spec)
                    .map_err(|e| Error::Unsupported {
            message: format!("Failed to build prompt spec: {}", e),
            feature: None,
        })
            }
        }
    }
    
    /// Build a chat specification from messages
    fn build_chat_spec(
        &self,
        messages: Vec<Message>,
        mode: GenerationMode,
        max_tokens: Option<u32>,
    ) -> Result<PromptSpec> {
        let params = self.map_mode_to_params(&mode);
        
        // Convert messages to JSON format
        let json_messages: Vec<Value> = messages
            .into_iter()
            .map(|m| serde_json::json!({
                "role": m.role,
                "content": m.content
            }))
            .collect();
        
        match self.api_format {
            ApiFormat::Responses => {
                // For GPT-5, concatenate messages into single input
                let input = json_messages
                    .iter()
                    .map(|m| {
                        format!(
                            "{}: {}",
                            m.get("role").and_then(|r| r.as_str()).unwrap_or("user"),
                            m.get("content").and_then(|c| c.as_str()).unwrap_or("")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n");
                
                self.build_prompt_spec(&input, mode, max_tokens)
            }
            ApiFormat::Messages => {
                // Claude uses messages directly
                let mut spec = serde_json::json!({
                    "model_class": "Chat",
                    "messages": json_messages,
                    "strict_mode": "Warn"
                });
                
                // Add sampling parameters
                let mut sampling = serde_json::Map::new();
                for (key, value) in params {
                    if key.starts_with("sampling.") {
                        let param_name = key.strip_prefix("sampling.").unwrap_or(&key);
                        sampling.insert(param_name.to_string(), value);
                    }
                }
                if let Some(tokens) = max_tokens {
                    sampling.insert("max_tokens".to_string(), tokens.into());
                }
                
                if let Some(obj) = spec.as_object_mut() {
                    if !sampling.is_empty() {
                        obj.insert("sampling".to_string(), sampling.into());
                    }
                }
                
                serde_json::from_value(spec)
                    .map_err(|e| Error::Unsupported {
            message: format!("Failed to build chat spec: {}", e),
            feature: None,
        })
            }
        }
    }
    
    /// Map generation mode to provider-specific parameters
    fn map_mode_to_params(&self, mode: &GenerationMode) -> HashMap<String, Value> {
        let mut params = HashMap::new();
        
        match (&self.api_format, mode) {
            (ApiFormat::Responses, GenerationMode::Creative) => {
                params.insert("reasoning".to_string(), serde_json::json!({
                    "effort": "high"
                }));
                params.insert("text".to_string(), serde_json::json!({
                    "verbosity": "high"
                }));
            }
            (ApiFormat::Responses, GenerationMode::Precise) => {
                params.insert("reasoning".to_string(), serde_json::json!({
                    "effort": "max"
                }));
                params.insert("text".to_string(), serde_json::json!({
                    "verbosity": "low"
                }));
            }
            (ApiFormat::Responses, GenerationMode::Fast) => {
                params.insert("reasoning".to_string(), serde_json::json!({
                    "effort": "low"
                }));
                params.insert("text".to_string(), serde_json::json!({
                    "verbosity": "low"
                }));
            }
            (ApiFormat::Responses, GenerationMode::Balanced) => {
                params.insert("reasoning".to_string(), serde_json::json!({
                    "effort": "medium"
                }));
                params.insert("text".to_string(), serde_json::json!({
                    "verbosity": "medium"
                }));
            }
            (ApiFormat::Messages, GenerationMode::Creative) => {
                params.insert("sampling.temperature".to_string(), 0.9.into());
                params.insert("sampling.top_p".to_string(), 0.95.into());
            }
            (ApiFormat::Messages, GenerationMode::Precise) => {
                params.insert("sampling.temperature".to_string(), 0.2.into());
                params.insert("sampling.top_p".to_string(), 0.5.into());
            }
            (ApiFormat::Messages, GenerationMode::Fast) => {
                params.insert("sampling.temperature".to_string(), 0.7.into());
                params.insert("sampling.top_p".to_string(), 0.8.into());
            }
            (ApiFormat::Messages, GenerationMode::Balanced) => {
                params.insert("sampling.temperature".to_string(), 0.5.into());
                params.insert("sampling.top_p".to_string(), 0.9.into());
            }
            (_, GenerationMode::Custom(custom_params)) => {
                params = custom_params.clone();
            }
        }
        
        params
    }
    
    /// Normalize model name to standard form
    fn normalize_model_name(model: &str) -> Result<String> {
        let normalized = match model.to_lowercase().as_str() {
            "gpt-5" | "gpt5" => "gpt-5",
            "gpt-5-mini" | "gpt5-mini" => "gpt-5-mini",
            "gpt-5-thinking" | "gpt5-thinking" => "gpt-5",
            "claude-opus-4.1" | "opus-4.1" | "opus4.1" => "claude-opus-4.1",
            "claude-sonnet-4" | "sonnet-4" | "sonnet4" => "claude-sonnet-4",
            _ => return Err(Error::Unsupported {
            message: format!("Model '{}' is not supported", model),
            feature: None,
        }),
        };
        
        Ok(normalized.to_string())
    }
    
    /// Detect API format for model
    fn detect_api_format(model: &str) -> ApiFormat {
        match model {
            "gpt-5" | "gpt-5-mini" => ApiFormat::Responses,
            "claude-opus-4.1" | "claude-sonnet-4" => ApiFormat::Messages,
            _ => ApiFormat::Messages, // Default to messages
        }
    }
    
    /// Load built-in provider specification
    fn load_builtin_spec(model: &str) -> Result<ProviderSpec> {
        // For now, load from files. Later we'll embed these.
        let spec_path = match model {
            "gpt-5" => "providers/openai/gpt-5.json",
            "gpt-5-mini" => "providers/openai/gpt-5-mini.json",
            "claude-opus-4.1" => "providers/anthropic/claude-opus-4.1.json",
            "claude-sonnet-4" => "providers/anthropic/claude-sonnet-4.json",
            _ => return Err(Error::Unsupported {
            message: format!("No built-in spec for model '{}'", model),
            feature: None,
        }),
        };
        
        let spec_content = std::fs::read_to_string(spec_path)
            .map_err(|e| Error::Unsupported {
            message: format!("Failed to load provider spec: {}", e),
            feature: None,
        })?;
        
        // Parse and validate the spec
        let spec: ProviderSpec = serde_json::from_str(&spec_content)
            .map_err(|e| Error::Unsupported {
            message: format!("Failed to parse provider spec: {}", e),
            feature: None,
        })?;
        
        // Update model ID to use the actual model ID from the spec
        // This ensures we use the correct model ID when calling the API
        if !spec.models.is_empty() {
            // The normalized model name should match an alias or the main ID
            let model_spec = spec.models.iter()
                .find(|m| m.id == model || m.aliases.as_ref().map_or(false, |a| a.contains(&model.to_string())))
                .ok_or_else(|| Error::Unsupported {
            message: format!("Model '{}' not found in provider spec", model),
            feature: None,
        })?;
            
            // For now, we'll just validate the model exists
            // The actual model ID will be passed through the translate function
        }
        
        Ok(spec)
    }
}

/// Simple message structure for chat interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    /// Create a new message
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }
    
    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self::new("user", content)
    }
    
    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new("assistant", content)
    }
    
    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self::new("system", content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_normalization() {
        assert_eq!(LLM::normalize_model_name("gpt-5").unwrap(), "gpt-5");
        assert_eq!(LLM::normalize_model_name("GPT-5").unwrap(), "gpt-5");
        assert_eq!(LLM::normalize_model_name("opus-4.1").unwrap(), "claude-opus-4.1");
        assert_eq!(LLM::normalize_model_name("sonnet-4").unwrap(), "claude-sonnet-4");
        assert!(LLM::normalize_model_name("unknown-model").is_err());
    }
    
    #[test]
    fn test_api_format_detection() {
        assert_eq!(LLM::detect_api_format("gpt-5"), ApiFormat::Responses);
        assert_eq!(LLM::detect_api_format("gpt-5-mini"), ApiFormat::Responses);
        assert_eq!(LLM::detect_api_format("claude-opus-4.1"), ApiFormat::Messages);
        assert_eq!(LLM::detect_api_format("claude-sonnet-4"), ApiFormat::Messages);
    }
}