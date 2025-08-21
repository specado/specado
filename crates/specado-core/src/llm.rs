//! High-level LLM interface for simplified usage
//! 
//! This module provides a user-friendly API that abstracts away the complexity
//! of provider specifications and parameter mapping.

use crate::{
    error::{Error, Result},
    types::{
        Message, MessageRole, PromptSpec, ProviderSpec, UniformResponse,
        SamplingParams, Limits,
    },
    translation::translate,
    StrictMode,
};
use serde::{Deserialize, Serialize};

/// High-level LLM interface for simplified interactions
pub struct LLM {
    /// The model identifier (e.g., "gpt-5", "claude-opus-4.1")
    model: String,
    /// The normalized model name for internal use
    normalized_model: String,
    /// The provider specification
    provider_spec: ProviderSpec,
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
}

impl LLM {
    /// Create a new LLM instance for the specified model
    pub fn new(model: &str) -> Result<Self> {
        let normalized_model = Self::normalize_model_name(model)?;
        let provider_spec = Self::load_builtin_spec(&normalized_model)?;
        
        Ok(Self {
            model: model.to_string(),
            normalized_model,
            provider_spec,
        })
    }
    
    /// Create a new LLM instance with a custom provider spec
    pub fn with_provider_spec(model: &str, provider_spec: ProviderSpec) -> Result<Self> {
        let normalized_model = Self::normalize_model_name(model)?;
        
        // Validate that the model exists in the provider spec
        let model_exists = provider_spec.models.iter()
            .any(|m| m.id == normalized_model || 
                     m.aliases.as_ref().map_or(false, |a| a.contains(&normalized_model)));
        
        if !model_exists {
            return Err(Error::Unsupported {
                message: format!("Model '{}' not found in provider spec", model),
                feature: None,
            });
        }
        
        Ok(Self {
            model: model.to_string(),
            normalized_model,
            provider_spec,
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
        self.execute_request(prompt_spec).await
    }
    
    /// Generate text with a simple prompt (synchronous version)
    #[cfg(feature = "blocking")]
    pub fn generate_sync(
        &self,
        prompt: &str,
        mode: GenerationMode,
        max_tokens: Option<u32>,
    ) -> Result<UniformResponse> {
        // Note: This creates a runtime internally, which is not ideal for library code.
        // Consider using the async version when possible.
        let prompt_spec = self.build_prompt_spec(prompt, mode, max_tokens)?;
        
        // Use tokio runtime for sync execution
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| Error::Unsupported {
                message: format!("Failed to create runtime: {}", e),
                feature: Some("blocking".to_string()),
            })?;
        
        runtime.block_on(self.execute_request(prompt_spec))
    }
    
    /// Chat with message history
    pub async fn chat(
        &self,
        messages: Vec<Message>,
        mode: GenerationMode,
        max_tokens: Option<u32>,
    ) -> Result<UniformResponse> {
        let prompt_spec = self.build_chat_spec(messages, mode, max_tokens)?;
        self.execute_request(prompt_spec).await
    }
    
    /// Execute a request with the configured provider
    async fn execute_request(&self, prompt_spec: PromptSpec) -> Result<UniformResponse> {
        // Translate the prompt spec to provider format
        let translation_result = translate(
            &prompt_spec,
            &self.provider_spec,
            &self.normalized_model,
            StrictMode::Warn,
        )?;
        
        // Wrap the translated request for the run function
        let request = serde_json::json!({
            "provider_spec": self.provider_spec,
            "model_id": self.normalized_model,
            "request_body": translation_result.provider_request_json,
        });
        
        // Execute the request
        let response = crate::run(&request).await?;
        Ok(response)
    }
    
    /// Build a prompt specification from parameters
    fn build_prompt_spec(
        &self,
        prompt: &str,
        mode: GenerationMode,
        max_tokens: Option<u32>,
    ) -> Result<PromptSpec> {
        // Always use Chat model class - let translation handle provider differences
        let messages = vec![
            Message {
                role: MessageRole::User,
                content: prompt.to_string(),
                name: None,
                metadata: None,
            }
        ];
        
        let (sampling, limits) = self.build_params(mode, max_tokens);
        
        Ok(PromptSpec {
            model_class: "Chat".to_string(),
            messages,
            tools: None,
            tool_choice: None,
            response_format: None,
            sampling: Some(sampling),
            limits: Some(limits),
            media: None,
            strict_mode: StrictMode::Warn,
        })
    }
    
    /// Build a chat specification from messages
    fn build_chat_spec(
        &self,
        messages: Vec<Message>,
        mode: GenerationMode,
        max_tokens: Option<u32>,
    ) -> Result<PromptSpec> {
        let (sampling, limits) = self.build_params(mode, max_tokens);
        
        Ok(PromptSpec {
            model_class: "Chat".to_string(),
            messages,
            tools: None,
            tool_choice: None,
            response_format: None,
            sampling: Some(sampling),
            limits: Some(limits),
            media: None,
            strict_mode: StrictMode::Warn,
        })
    }
    
    /// Build sampling parameters and limits from mode
    fn build_params(&self, mode: GenerationMode, max_tokens: Option<u32>) -> (SamplingParams, Limits) {
        // Standard parameters that work across providers
        // The translation layer will handle provider-specific mapping
        let sampling = match mode {
            GenerationMode::Creative => SamplingParams {
                temperature: Some(0.9),
                top_p: Some(0.95),
                top_k: None,
                frequency_penalty: None,
                presence_penalty: None,
            },
            GenerationMode::Precise => SamplingParams {
                temperature: Some(0.2),
                top_p: Some(0.5),
                top_k: None,
                frequency_penalty: None,
                presence_penalty: None,
            },
            GenerationMode::Fast => SamplingParams {
                temperature: Some(0.7),
                top_p: Some(0.8),
                top_k: None,
                frequency_penalty: None,
                presence_penalty: None,
            },
            GenerationMode::Balanced => SamplingParams {
                temperature: Some(0.5),
                top_p: Some(0.9),
                top_k: None,
                frequency_penalty: None,
                presence_penalty: None,
            },
        };
        
        let limits = Limits {
            max_output_tokens: max_tokens,
            reasoning_tokens: None,
            max_prompt_tokens: None,
        };
        
        (sampling, limits)
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
    
    /// Load built-in provider specification
    fn load_builtin_spec(model: &str) -> Result<ProviderSpec> {
        // For now, load from files. Later we can embed these with include_str!
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
        
        // Validate that the model exists in the spec
        if !spec.models.is_empty() {
            let _model_spec = spec.models.iter()
                .find(|m| m.id == model || m.aliases.as_ref().map_or(false, |a| a.contains(&model.to_string())))
                .ok_or_else(|| Error::Unsupported {
                    message: format!("Model '{}' not found in provider spec", model),
                    feature: None,
                })?;
        }
        
        Ok(spec)
    }
}

/// Helper functions for creating messages
impl Message {
    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
            name: None,
            metadata: None,
        }
    }
    
    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
            name: None,
            metadata: None,
        }
    }
    
    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
            name: None,
            metadata: None,
        }
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
    fn test_message_helpers() {
        let user_msg = Message::user("Hello");
        assert_eq!(user_msg.role, MessageRole::User);
        assert_eq!(user_msg.content, "Hello");
        
        let system_msg = Message::system("You are helpful");
        assert_eq!(system_msg.role, MessageRole::System);
        
        let assistant_msg = Message::assistant("I can help");
        assert_eq!(assistant_msg.role, MessageRole::Assistant);
    }
}