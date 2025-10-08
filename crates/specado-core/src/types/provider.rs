use crate::auth::AuthScheme;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderSpec {
    pub provider: String,
    pub models: Vec<ModelConfig>,
    pub endpoints: Endpoints,
    pub mappings: Mappings,
    pub constraints: Constraints,
    pub auth: AuthScheme,
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
