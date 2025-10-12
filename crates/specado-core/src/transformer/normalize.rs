use crate::error::{Error, Result};
use crate::types::{
    Extensions, FinishReason, LossinessReport, ProviderSpec, StrictMode, UniformResponse,
};
use serde_json::Value;
use serde_json_path::JsonPath;

pub fn normalize(raw: Value, provider: &ProviderSpec) -> Result<UniformResponse> {
    let mut content = String::new();
    let mut finish_reason = FinishReason::Stop;

    for mapping in &provider.mappings.response {
        let path = JsonPath::parse(&mapping.from)
            .map_err(|e| Error::Transform(format!("Invalid JSONPath '{}': {}", mapping.from, e)))?;
        let matches = path.query(&raw).all();
        if matches.is_empty() {
            continue;
        }

        let value = if matches.len() == 1 {
            matches[0].clone()
        } else {
            Value::Array(matches.iter().map(|v| (*v).clone()).collect())
        };

        match mapping.to.as_str() {
            "content" => {
                if let Some(text) = value.as_str() {
                    if !text.is_empty() {
                        content = text.to_string();
                    }
                } else if let Some(array) = value.as_array() {
                    let joined = array
                        .iter()
                        .filter_map(Value::as_str)
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                        .join(" ");
                    if !joined.is_empty() {
                        content = joined;
                    }
                }
            }
            "finish_reason" => {
                if let Some(reason) = value.as_str() {
                    finish_reason = map_finish_reason(reason);
                }
            }
            _ => {}
        }
    }

    let provider_capabilities = provider.capabilities_json();

    Ok(UniformResponse {
        content,
        tool_calls: Vec::new(),
        finish_reason,
        model: provider
            .models
            .first()
            .map(|m| m.id.clone())
            .unwrap_or_default(),
        provider_used: provider.provider.clone(),
        usage: None,
        extensions: Extensions {
            lossiness: LossinessReport::new(StrictMode::Warn),
            provider_capabilities,
        },
    })
}

fn map_finish_reason(raw: &str) -> FinishReason {
    match raw {
        "stop" | "end_turn" | "completed" => FinishReason::Stop,
        "length" | "max_tokens" => FinishReason::Length,
        "tool_calls" | "tool_use" => FinishReason::ToolCall,
        "content_filter" => FinishReason::ContentFilter,
        _ => FinishReason::Error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        Capabilities, Constraints, EndpointConfig, Endpoints, HttpMethod, Mappings, ModelConfig,
        ProviderSpec, ResponseMapping, SupportFlags,
    };
    use serde_json::json;
    use std::collections::HashMap;

    fn provider() -> ProviderSpec {
        ProviderSpec {
            provider: "openai".into(),
            models: vec![ModelConfig {
                id: "gpt-4o".into(),
            }],
            interface: Some("conversational.generate".into()),
            contract_version: Some("1.0.0".into()),
            inherits: None,
            endpoints: Endpoints {
                chat: EndpointConfig {
                    method: HttpMethod::Post,
                    url: "https://example.com".into(),
                    headers: Default::default(),
                },
            },
            mappings: Mappings {
                request: Vec::new(),
                response: vec![
                    ResponseMapping {
                        from: "$.choices[0].message.content".into(),
                        to: "content".into(),
                    },
                    ResponseMapping {
                        from: "$.choices[0].finish_reason".into(),
                        to: "finish_reason".into(),
                    },
                ],
            },
            constraints: Constraints {
                supports: SupportFlags {
                    json_mode: true,
                    tools: true,
                },
            },
            auth: crate::auth::AuthScheme::Bearer {
                token_env: "KEY".into(),
            },
            capabilities: Capabilities::default(),
            capabilities_extra: HashMap::new(),
            extensions: HashMap::new(),
            unsupported_parameters: Vec::new(),
        }
    }

    #[test]
    fn extracts_content_and_finish_reason() {
        let raw = json!({
            "choices": [
                {
                    "message": {"content": "Hello"},
                    "finish_reason": "stop"
                }
            ]
        });

        let response = normalize(raw, &provider()).expect("normalize");
        assert_eq!(response.content, "Hello");
        assert_eq!(response.finish_reason, FinishReason::Stop);
        assert_eq!(response.model, "gpt-4o");
    }

    #[test]
    fn joins_array_content() {
        let raw = json!({
            "choices": [
                {
                    "message": {
                        "content": [
                            {"text": "Hello"},
                            {"text": "world"}
                        ]
                    },
                    "finish_reason": "stop"
                }
            ]
        });

        let mut provider = provider();
        provider.mappings.response[0].from = "$.choices[0].message.content[*].text".into();

        let response = normalize(raw, &provider).expect("normalize");
        assert_eq!(response.content, "Hello world");
    }

    #[test]
    fn maps_unknown_reason_to_error() {
        let raw = json!({
            "choices": [
                {
                    "message": {"content": "Hi"},
                    "finish_reason": "rate_limited"
                }
            ]
        });

        let response = normalize(raw, &provider()).expect("normalize");
        assert_eq!(response.finish_reason, FinishReason::Error);
    }
}
