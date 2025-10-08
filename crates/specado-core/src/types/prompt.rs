use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSpec {
    pub version: String,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub sampling: SamplingConfig,
    #[serde(default)]
    pub response: ResponseConfig,
    #[serde(default)]
    pub tools: Vec<Tool>,
    #[serde(default)]
    pub tool_choice: Option<ToolChoice>,
    #[serde(default)]
    pub strict_mode: StrictMode,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SamplingConfig {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<u32>,
    pub frequency_penalty: Option<f64>,
    pub presence_penalty: Option<f64>,
    pub seed: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseConfig {
    #[serde(default = "default_format")]
    pub format: ResponseFormat,
    pub json_schema: Option<JsonSchema>,
}

fn default_format() -> ResponseFormat {
    ResponseFormat::Text
}

impl Default for ResponseConfig {
    fn default() -> Self {
        Self {
            format: ResponseFormat::Text,
            json_schema: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormat {
    Text,
    Json,
    JsonSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonSchema {
    pub name: String,
    pub description: Option<String>,
    pub schema: serde_json::Value,
    #[serde(default)]
    pub strict: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    pub json_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ToolChoice {
    String(ToolChoiceString),
    Object { name: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceString {
    Auto,
    Required,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "PascalCase")]
pub enum StrictMode {
    Strict,
    #[default]
    Warn,
    Coerce,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn response_config_defaults_to_text() {
        let config = ResponseConfig::default();
        assert_eq!(config.format, ResponseFormat::Text);
        assert!(config.json_schema.is_none());
    }

    #[test]
    fn strict_mode_default_is_warn() {
        let prompt: PromptSpec = serde_json::from_value(json!({
            "version": "1",
            "messages": [
                {"role": "user", "content": "Hello"}
            ]
        }))
        .expect("prompt should deserialize");

        assert_eq!(prompt.strict_mode, StrictMode::Warn);
    }

    #[test]
    fn round_trip_prompt_spec() {
        let prompt = PromptSpec {
            version: "1".to_string(),
            messages: vec![Message {
                role: MessageRole::System,
                content: "You are helpful.".into(),
            }],
            sampling: SamplingConfig {
                temperature: Some(0.7),
                top_p: Some(0.9),
                top_k: Some(20),
                frequency_penalty: None,
                presence_penalty: None,
                seed: Some(42),
            },
            response: ResponseConfig {
                format: ResponseFormat::JsonSchema,
                json_schema: Some(JsonSchema {
                    name: "summary".into(),
                    description: Some("Short summary".into()),
                    schema: json!({"type": "object"}),
                    strict: true,
                }),
            },
            tools: vec![Tool {
                name: "search".into(),
                description: Some("Lookup".into()),
                json_schema: json!({"type": "object"}),
            }],
            tool_choice: Some(ToolChoice::String(ToolChoiceString::Auto)),
            strict_mode: StrictMode::Strict,
            metadata: HashMap::from([(String::from("source"), json!("test"))]),
        };

        let serialized = serde_json::to_string_pretty(&prompt).expect("serialize");
        let deserialized: PromptSpec = serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(deserialized.version, prompt.version);
        assert_eq!(deserialized.messages, prompt.messages);
        assert_eq!(deserialized.response.format, ResponseFormat::JsonSchema);
        assert_eq!(deserialized.tools.len(), 1);
        assert!(matches!(
            deserialized.tool_choice,
            Some(ToolChoice::String(ToolChoiceString::Auto))
        ));
    }
}
