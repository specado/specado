use crate::auth::AuthScheme;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderApi {
    ChatCompletions,
    OpenaiResponses,
    AnthropicMessagesClaude4,
}

impl ProviderApi {
    pub fn registry_key(&self) -> &'static str {
        match self {
            ProviderApi::ChatCompletions => "chat_completions",
            ProviderApi::OpenaiResponses => "openai_responses",
            ProviderApi::AnthropicMessagesClaude4 => "anthropic_messages_claude4",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderSpec {
    pub provider: String,
    pub models: Vec<ModelConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inherits: Option<String>,
    pub endpoints: Endpoints,
    pub mappings: Mappings,
    pub constraints: Constraints,
    pub auth: AuthScheme,
    #[serde(default)]
    pub capabilities: Capabilities,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub capabilities_extra: HashMap<String, JsonValue>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub extensions: HashMap<String, JsonValue>,
    #[serde(default)]
    pub unsupported_parameters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelConfig {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Endpoints {
    pub chat: EndpointConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EndpointConfig {
    pub method: HttpMethod,
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Post,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mappings {
    pub request: Vec<RequestMapping>,
    pub response: Vec<ResponseMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestMapping {
    pub from: String,
    pub to: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clamp: Option<[f64; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseMapping {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Constraints {
    pub supports: SupportFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SupportFlags {
    pub json_mode: bool,
    pub tools: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Capabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u64>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub supports_tools: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub supports_custom_tools: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub supports_parallel_tool_calls: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub supports_allowed_tools: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub supports_json_mode: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub supports_seed: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub supports_frequency_penalty: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub supports_presence_penalty: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub supports_logit_bias: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub supports_logprobs: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub supports_top_logprobs: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub supports_n: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub supports_response_format: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub supports_stop: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub supports_temperature: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub supports_top_p: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub supports_top_k: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub supports_image_input: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub supports_extended_thinking: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub responses_api: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reasoning_controls: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thought_signatures: Option<String>,
}

fn is_false(value: &bool) -> bool {
    !*value
}

impl ProviderSpec {
    pub fn interface_hint(&self) -> Option<&str> {
        self.interface.as_deref()
    }

    pub fn contract_version(&self) -> Option<&str> {
        self.contract_version.as_deref()
    }

    pub fn api_kind(&self) -> ProviderApi {
        if let Some(interface) = self.interface_hint() {
            if interface.starts_with("x_") {
                return self.infer_api_from_context();
            }

            match interface {
                "conversational.generate"
                | "conversational.stream"
                | "text.generate"
                | "text.extract"
                | "tools.call" => {
                    return self.infer_api_from_context();
                }
                _ => {}
            }
        }

        self.infer_api_from_context()
    }

    fn infer_api_from_context(&self) -> ProviderApi {
        let url = self.endpoints.chat.url.to_ascii_lowercase();

        if url.contains("/responses") {
            ProviderApi::OpenaiResponses
        } else if url.contains("/messages") || self.provider.eq_ignore_ascii_case("anthropic") {
            ProviderApi::AnthropicMessagesClaude4
        } else {
            ProviderApi::ChatCompletions
        }
    }

    pub fn capabilities_json(&self) -> Option<JsonValue> {
        let mut map = JsonMap::new();

        if let Ok(JsonValue::Object(obj)) = serde_json::to_value(&self.capabilities) {
            map.extend(obj);
        }

        for (key, value) in &self.capabilities_extra {
            map.insert(key.clone(), value.clone());
        }

        if map.is_empty() {
            None
        } else {
            Some(JsonValue::Object(map))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;

    #[test]
    fn deserialize_bearer_provider() {
        let yaml = r#"
provider: openai
models:
  - id: gpt-4o
endpoints:
  chat:
    method: POST
    url: https://api.openai.com/v1/chat/completions
    headers:
      content-type: application/json
mappings:
  request:
    - from: prompt.messages
      to: body.messages
  response:
    - from: body.choices[0].message
      to: prompt.response
constraints:
  supports:
    json_mode: true
    tools: true
auth:
  type: bearer
  token_env: OPENAI_API_KEY
"#;

        let spec: ProviderSpec = serde_yaml::from_str(yaml).expect("valid provider spec");

        assert_eq!(spec.provider, "openai");
        assert_eq!(spec.models.len(), 1);
        assert!(matches!(spec.auth, AuthScheme::Bearer { .. }));
    }

    #[test]
    fn deserialize_apikey_provider() {
        let yaml = r#"
provider: custom
models:
  - id: custom-model
endpoints:
  chat:
    method: POST
    url: https://example.com/chat
    headers: {}
mappings:
  request: []
  response: []
constraints:
  supports:
    json_mode: false
    tools: false
auth:
  type: apikey
  header: X-API-Key
  key_env: CUSTOM_KEY
"#;

        let spec: ProviderSpec = serde_yaml::from_str(yaml).expect("valid provider spec");

        match spec.auth {
            AuthScheme::ApiKey { header, key_env } => {
                assert_eq!(header, "X-API-Key");
                assert_eq!(key_env, "CUSTOM_KEY");
            }
            _ => panic!("expected apikey variant"),
        }
    }
}
