//! High-level LLM interface for simplified usage
//! 
//! This module provides a user-friendly API that abstracts away the complexity
//! of provider specifications and parameter mapping.

use crate::{
    error::{Error, Result},
    types::{
        Message, MessageRole, PromptSpec, ProviderSpec, UniformResponse,
        SamplingParams, Limits, AdvancedParams, ReasoningEffort, ReasoningMode, VerbosityLevel,
    },
    translation::translate,
    StrictMode,
};
use serde::{Deserialize, Serialize};

/// High-level LLM interface for simplified interactions
pub struct LLM {
    /// The model identifier (e.g., "gpt-5", "claude-opus-4.1")
    #[allow(dead_code)] // May be used for debugging or future features
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
    /// Quick one-shot API call with minimal setup
    pub async fn quick_ask(model: &str, prompt: &str) -> Result<String> {
        let llm = Self::new(model)?;
        llm.ask(prompt).await
    }
    
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
    
    /// Generate text with minimal configuration (uses defaults)
    pub async fn ask(&self, prompt: &str) -> Result<String> {
        let response = self.generate(prompt, GenerationMode::Balanced, None).await?;
        Ok(response.content)
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
    
    /// Generate with advanced parameters for thinking mode (Claude Opus 4.1)
    pub async fn generate_with_thinking(
        &self,
        prompt: &str,
        thinking_enabled: bool,
        min_thinking_tokens: Option<u32>,
        max_tokens: Option<u32>,
    ) -> Result<UniformResponse> {
        let mut prompt_spec = self.build_prompt_spec(prompt, GenerationMode::Balanced, max_tokens)?;
        
        // Add advanced thinking parameters
        prompt_spec.advanced = Some(AdvancedParams {
            thinking: Some(thinking_enabled),
            min_thinking_tokens,
            reasoning_effort: None,
            seed: None,
            reasoning_mode: None,
            thinking_budget: None,
            verbosity: None,
        });
        
        self.execute_request(prompt_spec).await
    }
    
    /// Generate with reasoning effort control (GPT-5)
    pub async fn generate_with_reasoning(
        &self,
        prompt: &str,
        reasoning_effort: ReasoningEffort,
        seed: Option<u32>,
        max_tokens: Option<u32>,
    ) -> Result<UniformResponse> {
        let mut prompt_spec = self.build_prompt_spec(prompt, GenerationMode::Balanced, max_tokens)?;
        
        // Add advanced reasoning parameters
        prompt_spec.advanced = Some(AdvancedParams {
            thinking: None,
            min_thinking_tokens: None,
            reasoning_effort: Some(reasoning_effort),
            seed,
            reasoning_mode: None,
            thinking_budget: None,
            verbosity: None,
        });
        
        self.execute_request(prompt_spec).await
    }
    
    /// Generate with balanced reasoning mode (Claude 4 Sonnet)
    pub async fn generate_with_balanced_reasoning(
        &self,
        prompt: &str,
        reasoning_mode: ReasoningMode,
        thinking_budget: Option<u32>,
        verbosity: Option<VerbosityLevel>,
        max_tokens: Option<u32>,
    ) -> Result<UniformResponse> {
        let mut prompt_spec = self.build_prompt_spec(prompt, GenerationMode::Balanced, max_tokens)?;
        
        // Add advanced balanced reasoning parameters
        prompt_spec.advanced = Some(AdvancedParams {
            thinking: None,
            min_thinking_tokens: None,
            reasoning_effort: None,
            seed: None,
            reasoning_mode: Some(reasoning_mode),
            thinking_budget,
            verbosity,
        });
        
        self.execute_request(prompt_spec).await
    }
    
    /// Generate with full advanced parameters control
    pub async fn generate_advanced(
        &self,
        prompt: &str,
        advanced: AdvancedParams,
        mode: GenerationMode,
        max_tokens: Option<u32>,
    ) -> Result<UniformResponse> {
        let mut prompt_spec = self.build_prompt_spec(prompt, mode, max_tokens)?;
        prompt_spec.advanced = Some(advanced);
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
            advanced: None,
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
            advanced: None,
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
    
    /// Normalize model name using provider discovery (spec-driven approach)
    fn normalize_model_name(model: &str) -> Result<String> {
        // Use provider discovery to find the model instead of hardcoded checks
        use crate::provider_discovery::ProviderRegistry;
        
        let registry = ProviderRegistry::new();
        
        // First try exact match
        match registry.discover_provider(model) {
            Ok(_provider_info) => {
                // For exact matches, return the canonical (lowercase) form
                // Check if this model is in our available models list to get the canonical form
                let available_models = registry.list_available_models();
                for available in &available_models {
                    if available.eq_ignore_ascii_case(model) {
                        return Ok(available.clone());
                    }
                }
                // If not found in available list, return lowercase
                return Ok(model.to_lowercase());
            }
            Err(_) => {
                // No exact match - try case-insensitive matching
                let model_lower = model.to_lowercase();
                let available_models = registry.list_available_models();
                
                // Look for case-insensitive match in available models
                for available in &available_models {
                    if available.to_lowercase() == model_lower {
                        // Return the canonical model name (from the spec)
                        return Ok(available.clone());
                    }
                }
                
                // Also try provider discovery with the lowercase version
                match registry.discover_provider(&model_lower) {
                    Ok(_provider_info) => {
                        return Ok(model_lower);
                    }
                    Err(_) => {}
                }
                
                Err(Error::Unsupported {
                    message: format!(
                        "Model '{}' is not supported. Available models: {}",
                        model,
                        available_models.join(", ")
                    ),
                    feature: None,
                })
            }
        }
    }
    
    /// Load provider specification using spec-driven discovery
    fn load_builtin_spec(model: &str) -> Result<ProviderSpec> {
        use crate::provider_discovery::ProviderRegistry;
        
        let mut registry = ProviderRegistry::new();
        
        // Discover provider for the model using spec-driven approach
        let provider_info = registry.discover_provider(model)
            .map_err(|e| Error::Unsupported {
                message: format!("Model '{}' not supported: {}", model, e),
                feature: None,
            })?.clone(); // Clone to avoid borrowing issues
        
        // Load the provider specification from the discovered provider
        let spec = registry.load_provider_spec(&provider_info)
            .map_err(|e| Error::Unsupported {
                message: format!("Failed to load provider spec for model '{}': {}", model, e),
                feature: None,
            })?;
        
        // Parse the spec as ProviderSpec
        let provider_spec: ProviderSpec = serde_json::from_value(spec)
            .map_err(|e| Error::Unsupported {
                message: format!("Failed to parse provider spec for model '{}': {}", model, e),
                feature: None,
            })?;
        
        // Validate that the model exists in the spec
        if !provider_spec.models.is_empty() {
            let _model_spec = provider_spec.models.iter()
                .find(|m| {
                    m.id == model || 
                    m.aliases.as_ref().map_or(false, |a| a.contains(&model.to_string())) ||
                    // Also check if model matches any pattern for this provider
                    provider_info.model_patterns.iter().any(|pattern| {
                        registry.matches_pattern(model, pattern)
                    })
                })
                .ok_or_else(|| Error::Unsupported {
                    message: format!("Model '{}' not found in provider spec", model),
                    feature: None,
                })?;
        }
        
        Ok(provider_spec)
    }
}

/// Helper functions for creating messages

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_normalization() {
        // Test case normalization
        assert_eq!(LLM::normalize_model_name("gpt-5").unwrap(), "gpt-5");
        assert_eq!(LLM::normalize_model_name("GPT-5").unwrap(), "gpt-5");
        
        // Test alias resolution (sonnet-4 is a direct alias based on available models)
        assert_eq!(LLM::normalize_model_name("sonnet-4").unwrap(), "sonnet-4");
        
        // Test that full model names work
        let result = LLM::normalize_model_name("claude-opus-4.1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "claude-opus-4.1");
        
        // Test unknown model - in the spec-driven system, unknown models fall back to default provider
        let unknown_result = LLM::normalize_model_name("unknown-model");
        assert!(unknown_result.is_ok()); // Fallback provider handles unknown models
        
        // The normalized name should be the input (since no specific normalization applies)
        assert_eq!(unknown_result.unwrap(), "unknown-model");
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