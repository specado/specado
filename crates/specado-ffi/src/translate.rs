//! Translation FFI implementation
//!
//! This module implements the translate function that converts
//! prompts to provider-specific requests.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use specado_core::{
    types::{ProviderSpec, ModelSpec},
    Error as CoreError,
};

use crate::types::SpecadoResult;
use crate::memory::set_last_error;
use crate::error::map_core_error;

/// Input structure for translation
#[derive(Debug, Deserialize)]
pub struct TranslateInput {
    /// The prompt to translate
    pub prompt: PromptInput,
    /// Optional configuration
    pub config: Option<TranslationConfig>,
}

/// Prompt input structure
#[derive(Debug, Deserialize)]
pub struct PromptInput {
    /// System message
    pub system: Option<String>,
    /// User messages
    pub messages: Vec<Message>,
    /// Temperature setting
    pub temperature: Option<f32>,
    /// Max tokens
    pub max_tokens: Option<u32>,
    /// Tool definitions
    pub tools: Option<Vec<Value>>,
}

/// Message structure
#[derive(Debug, Deserialize)]
pub struct Message {
    /// Role (user, assistant, system)
    pub role: String,
    /// Content
    pub content: String,
}

/// Translation configuration
#[derive(Debug, Deserialize)]
pub struct TranslationConfig {
    /// Strictness mode
    pub strict_mode: Option<String>,
    /// Enable validation
    pub validate: Option<bool>,
}

/// Translation result
#[derive(Debug, Serialize)]
pub struct TranslateResult {
    /// Success status
    pub success: bool,
    /// Translated request
    pub request: Option<Value>,
    /// Validation results
    pub validation: Option<ValidationResult>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Validation result
#[derive(Debug, Serialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub valid: bool,
    /// Warnings
    pub warnings: Vec<String>,
    /// Errors
    pub errors: Vec<String>,
}

/// Perform translation from prompt to provider request
pub fn translate(
    prompt_json: &str,
    provider_spec: &ProviderSpec,
    model_id: &str,
    mode: &str,
) -> Result<String, SpecadoResult> {
    // Parse the prompt input
    let input: TranslateInput = serde_json::from_str(prompt_json)
        .map_err(|e| {
            set_last_error(format!("Failed to parse prompt JSON: {}", e));
            SpecadoResult::JsonError
        })?;
    
    // Find the model in the provider spec
    let model = provider_spec
        .models
        .iter()
        .find(|m| m.id == model_id || m.aliases.as_ref().map_or(false, |a| a.contains(&model_id.to_string())))
        .ok_or_else(|| {
            set_last_error(format!("Model '{}' not found in provider spec", model_id));
            SpecadoResult::ModelNotFound
        })?;
    
    // Convert to Specado internal format
    let prompt_value = convert_to_internal_format(&input.prompt)?;
    
    // For now, use a simplified translation that just passes through
    // TODO: Integrate with full TranslationEngine when available
    let translated_request = perform_basic_translation(prompt_value, model, provider_spec)?;
    
    // Build result
    let result = TranslateResult {
        success: true,
        request: Some(translated_request),
        validation: Some(ValidationResult {
            valid: true,
            warnings: vec![],
            errors: vec![],
        }),
        error: None,
    };
    
    // Serialize result
    serde_json::to_string(&result)
        .map_err(|e| {
            set_last_error(format!("Failed to serialize result: {}", e));
            SpecadoResult::JsonError
        })
}

/// Convert input format to Specado internal format
fn convert_to_internal_format(prompt: &PromptInput) -> Result<Value, SpecadoResult> {
    let mut messages = Vec::new();
    
    // Add system message if present
    if let Some(system) = &prompt.system {
        messages.push(serde_json::json!({
            "role": "system",
            "content": system
        }));
    }
    
    // Add user messages
    for msg in &prompt.messages {
        messages.push(serde_json::json!({
            "role": msg.role,
            "content": msg.content
        }));
    }
    
    // Build the prompt object
    let mut prompt_obj = serde_json::json!({
        "messages": messages
    });
    
    // Add optional parameters
    if let Some(temp) = prompt.temperature {
        prompt_obj["temperature"] = Value::from(temp);
    }
    
    if let Some(max_tokens) = prompt.max_tokens {
        prompt_obj["max_tokens"] = Value::from(max_tokens);
    }
    
    if let Some(tools) = &prompt.tools {
        prompt_obj["tools"] = Value::Array(tools.clone());
    }
    
    Ok(prompt_obj)
}

/// Perform basic translation (simplified version)
fn perform_basic_translation(
    prompt: Value,
    model: &ModelSpec,
    provider_spec: &ProviderSpec,
) -> Result<Value, SpecadoResult> {
    // For now, just add provider-specific fields
    let mut request = prompt.clone();
    
    // Add model
    request["model"] = Value::String(model.id.clone());
    
    // Add any provider-specific defaults
    if provider_spec.provider.name.to_lowercase().contains("openai") {
        if !request.get("temperature").is_some() {
            request["temperature"] = Value::from(0.7);
        }
    }
    
    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_convert_to_internal_format() {
        let prompt = PromptInput {
            system: Some("You are a helpful assistant".to_string()),
            messages: vec![
                Message {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                },
            ],
            temperature: Some(0.7),
            max_tokens: Some(100),
            tools: None,
        };
        
        let result = convert_to_internal_format(&prompt).unwrap();
        assert!(result["messages"].is_array());
        assert_eq!(result["messages"].as_array().unwrap().len(), 2);
        assert!((result["temperature"].as_f64().unwrap() - 0.7).abs() < 0.001);
        assert_eq!(result["max_tokens"], 100);
    }
}