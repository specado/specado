//! Translation FFI implementation
//!
//! This module implements the translate function that converts
//! prompts to provider-specific requests using the core translation engine.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use specado_core::{
    types::{ProviderSpec, PromptSpec, StrictMode, TranslationResult},
    translation::translate as core_translate,
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

<<<<<<< HEAD

=======
>>>>>>> feat/epic47-nodejs-binding
/// Perform translation from prompt to provider request
pub fn translate(
    prompt_json: &str,
    provider_spec: &ProviderSpec,
    model_id: &str,
    mode: &str,
) -> Result<String, SpecadoResult> {
    // Parse the prompt input to TranslateInput first
    let input: TranslateInput = serde_json::from_str(prompt_json)
        .map_err(|e| {
            set_last_error(format!("Failed to parse prompt JSON: {}", e));
            SpecadoResult::JsonError
        })?;
    
    // Convert to PromptSpec
    let prompt_spec = convert_to_prompt_spec(&input.prompt, &input.config)?;
    
    // Parse strict mode from the mode parameter
    let strict_mode = match mode {
        "strict" => StrictMode::Strict,
        "warn" => StrictMode::Warn,
        "coerce" => StrictMode::Coerce,
        _ => StrictMode::Warn, // Default fallback
    };
    
    // Use the actual core translation engine
    let translation_result = core_translate(&prompt_spec, provider_spec, model_id, strict_mode)
        .map_err(|e| {
            set_last_error(format!("Translation failed: {}", e));
            map_core_error(e)
        })?;
    
    // Serialize the complete TranslationResult
    serde_json::to_string(&translation_result)
        .map_err(|e| {
            set_last_error(format!("Failed to serialize result: {}", e));
            SpecadoResult::JsonError
        })
}

/// Convert input format to PromptSpec
fn convert_to_prompt_spec(prompt: &PromptInput, config: &Option<TranslationConfig>) -> Result<PromptSpec, SpecadoResult> {
    use specado_core::types::*;
    
    let mut messages = Vec::new();
    
    // Add system message if present
    if let Some(system) = &prompt.system {
        messages.push(Message {
            role: MessageRole::System,
            content: system.clone(),
            name: None,
            metadata: None,
        });
    }
    
    // Add user messages
    for msg in &prompt.messages {
        let role = match msg.role.as_str() {
            "system" => MessageRole::System,
            "user" => MessageRole::User,
            "assistant" => MessageRole::Assistant,
            _ => {
                set_last_error(format!("Invalid message role: {}", msg.role));
                return Err(SpecadoResult::InvalidInput);
            }
<<<<<<< HEAD
        };
        
        messages.push(Message {
            role,
            content: msg.content.clone(),
            name: None,
            metadata: None,
        });
    }
    
    // Build sampling parameters if present
    let sampling = if prompt.temperature.is_some() {
        Some(SamplingParams {
            temperature: prompt.temperature,
            top_p: None,
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
        })
    } else {
        None
    };
    
    // Build limits if max_tokens is present
    let limits = if let Some(max_tokens) = prompt.max_tokens {
        Some(Limits {
            max_output_tokens: Some(max_tokens),
            reasoning_tokens: None,
            max_prompt_tokens: None,
        })
    } else {
        None
    };
    
    // Convert tools if present
    let tools = if let Some(tool_values) = &prompt.tools {
        let mut converted_tools = Vec::new();
        for tool_value in tool_values {
            // For now, pass through as-is since we'd need more complex conversion
            let tool: Tool = serde_json::from_value(tool_value.clone())
                .map_err(|e| {
                    set_last_error(format!("Failed to parse tool: {}", e));
                    SpecadoResult::JsonError
                })?;
            converted_tools.push(tool);
        }
        Some(converted_tools)
    } else {
        None
    };
    
    // Determine strict mode from config
    let strict_mode = if let Some(cfg) = config {
        match cfg.strict_mode.as_deref() {
            Some("strict") => StrictMode::Strict,
            Some("warn") => StrictMode::Warn,
            Some("coerce") => StrictMode::Coerce,
            _ => StrictMode::Warn,
        }
    } else {
        StrictMode::Warn
    };
    
    Ok(PromptSpec {
        model_class: "Chat".to_string(), // Default to Chat for FFI compatibility
        messages,
        tools,
        tool_choice: None,
        response_format: None,
        sampling,
        limits,
        media: None,
        strict_mode,
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_convert_to_prompt_spec() {
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
        
        let config = Some(TranslationConfig {
            strict_mode: Some("warn".to_string()),
            validate: Some(true),
        });
        
        let result = convert_to_prompt_spec(&prompt, &config).unwrap();
        assert_eq!(result.messages.len(), 2); // System + user message
        assert!(result.sampling.is_some());
        let sampling = result.sampling.unwrap();
        assert!((sampling.temperature.unwrap() - 0.7).abs() < 0.001);
        assert_eq!(sampling.max_tokens.unwrap(), 100);
=======
        };
        
        messages.push(Message {
            role,
            content: msg.content.clone(),
            name: None,
            metadata: None,
        });
>>>>>>> feat/epic47-nodejs-binding
    }
    
    // Build sampling parameters if present
    let sampling = if prompt.temperature.is_some() {
        Some(SamplingParams {
            temperature: prompt.temperature,
            top_p: None,
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
        })
    } else {
        None
    };
    
    // Build limits if max_tokens is present
    let limits = if let Some(max_tokens) = prompt.max_tokens {
        Some(Limits {
            max_output_tokens: Some(max_tokens),
            reasoning_tokens: None,
            max_prompt_tokens: None,
        })
    } else {
        None
    };
    
    // Convert tools if present
    let tools = if let Some(tool_values) = &prompt.tools {
        let mut converted_tools = Vec::new();
        for tool_value in tool_values {
            // For now, pass through as-is since we'd need more complex conversion
            let tool: Tool = serde_json::from_value(tool_value.clone())
                .map_err(|e| {
                    set_last_error(format!("Failed to parse tool: {}", e));
                    SpecadoResult::JsonError
                })?;
            converted_tools.push(tool);
        }
        Some(converted_tools)
    } else {
        None
    };
    
    // Determine strict mode from config
    let strict_mode = if let Some(cfg) = config {
        match cfg.strict_mode.as_deref() {
            Some("strict") => StrictMode::Strict,
            Some("warn") => StrictMode::Warn,
            Some("coerce") => StrictMode::Coerce,
            _ => StrictMode::Warn,
        }
    } else {
        StrictMode::Warn
    };
    
    Ok(PromptSpec {
        model_class: "Chat".to_string(), // Default to Chat for FFI compatibility
        messages,
        tools,
        tool_choice: None,
        response_format: None,
        sampling,
        limits,
        media: None,
        strict_mode,
    })
}