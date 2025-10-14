//! Public Specado API re-exported for downstream consumers.
//!
//! This crate wraps the internal `specado-core` implementation and exposes
//! the same types and helpers while reserving the `specado` name on crates.io.

#![allow(clippy::all)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

pub use specado_core::*;
pub use specado_core::{
    Error, Message, MessageRole, PromptSpec, ResponseConfig, SamplingConfig, StrictMode, Tool,
    ToolChoice,
};

/// Load a prompt specification (YAML or JSON) from disk.
///
/// This helper mirrors the CLI behaviour and provides a frictionless way to
/// execute specs from files in Rust applications.
pub fn load_prompt_from_path(
    path: impl AsRef<Path>,
) -> specado_core::Result<specado_core::PromptSpec> {
    let path = path.as_ref();
    let contents = std::fs::read_to_string(path).map_err(|err| {
        specado_core::Error::Config(format!("Failed to read prompt {}: {}", path.display(), err))
    })?;

    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .unwrap_or_default();

    if ext == "json" {
        serde_json::from_str(&contents).map_err(|err| {
            specado_core::Error::Config(format!("Invalid JSON prompt {}: {}", path.display(), err))
        })
    } else {
        match serde_yaml::from_str(&contents) {
            Ok(spec) => Ok(spec),
            Err(yaml_err) if ext == "json" => Err(specado_core::Error::Config(format!(
                "Invalid JSON prompt {}: {}",
                path.display(),
                yaml_err
            ))),
            Err(yaml_err) => serde_json::from_str(&contents).map_err(|json_err| {
                specado_core::Error::Config(format!(
                    "Failed to parse prompt {} as YAML ({}) or JSON ({}).",
                    path.display(),
                    yaml_err,
                    json_err
                ))
            }),
        }
    }
}

/// Structured options for building a prompt specification.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptBuilder {
    pub messages: Vec<Message>,
    #[serde(default)]
    pub sampling: Option<SamplingConfig>,
    #[serde(default, alias = "strict_mode")]
    pub strict_mode: Option<StrictMode>,
    #[serde(default)]
    pub response: Option<ResponseConfig>,
    #[serde(default)]
    pub tools: Option<Vec<Tool>>,
    #[serde(default, alias = "tool_choice")]
    pub tool_choice: Option<ToolChoice>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Construct a prompt specification from the provided builder options.
pub fn create_prompt(builder: PromptBuilder) -> PromptSpec {
    PromptSpec {
        version: "1".to_string(),
        messages: builder.messages,
        sampling: builder.sampling.unwrap_or_default(),
        strict_mode: builder.strict_mode.unwrap_or_default(),
        response: builder.response.unwrap_or_default(),
        tools: builder.tools.unwrap_or_default(),
        tool_choice: builder.tool_choice,
        metadata: builder.metadata.unwrap_or_default(),
    }
}

/// Convenience options for constructing a simple user/system prompt.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimplePromptOptions {
    pub message: Option<String>,
    pub user: Option<String>,
    pub system: Option<String>,
    pub temperature: Option<f64>,
    #[serde(default)]
    pub sampling: Option<SamplingConfig>,
    #[serde(alias = "strict_mode")]
    pub strict_mode: Option<StrictMode>,
}

/// Create a minimal prompt specification from simple text inputs.
pub fn simple_prompt(mut options: SimplePromptOptions) -> specado_core::Result<PromptSpec> {
    let primary = options
        .user
        .take()
        .or_else(|| options.message.take())
        .unwrap_or_default();

    if primary.trim().is_empty() {
        return Err(Error::Config(
            "simple prompt requires a non-empty user message".to_string(),
        ));
    }

    let mut messages = Vec::new();
    if let Some(system) = options.system.take() {
        if system.trim().len() > 0 {
            messages.push(Message {
                role: MessageRole::System,
                content: system,
            });
        }
    }
    messages.push(Message {
        role: MessageRole::User,
        content: primary,
    });

    let mut sampling = options.sampling.unwrap_or_default();
    if let Some(temperature) = options.temperature {
        sampling.temperature = Some(temperature);
    }

    Ok(create_prompt(PromptBuilder {
        messages,
        sampling: Some(sampling),
        strict_mode: options.strict_mode,
        ..PromptBuilder::default()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_prompt_adds_user_and_system_messages() {
        let prompt = simple_prompt(SimplePromptOptions {
            message: Some("Hello assistant".into()),
            system: Some("Behave helpfully".into()),
            temperature: Some(0.3),
            ..SimplePromptOptions::default()
        })
        .expect("prompt should construct");

        assert_eq!(prompt.messages.len(), 2);
        assert_eq!(prompt.messages[0].role, MessageRole::System);
        assert_eq!(prompt.messages[1].role, MessageRole::User);
        assert_eq!(prompt.sampling.temperature, Some(0.3));
    }

    #[test]
    fn simple_prompt_requires_user_content() {
        let err = simple_prompt(SimplePromptOptions::default()).unwrap_err();
        assert!(matches!(err, Error::Config(_)));
    }
}
