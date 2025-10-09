use crate::types::{
    LossinessCode, LossinessEntry, LossinessLevel, LossinessReport, PromptSpec, ProviderSpec,
    ResponseFormat,
};
use serde_json::json;

pub fn detect_unsupported(
    prompt: &PromptSpec,
    provider: &ProviderSpec,
    report: &mut LossinessReport,
) {
    if matches!(
        prompt.response.format,
        ResponseFormat::Json | ResponseFormat::JsonSchema
    ) && !provider.constraints.supports.json_mode
    {
        report.add_entry(LossinessEntry {
            code: LossinessCode::Unsupported,
            level: LossinessLevel::Warn,
            path: "response.format".to_string(),
            reason: "Provider does not support native JSON mode".to_string(),
            suggested_fix: Some(
                "Use a provider with native JSON support or accept emulation".to_string(),
            ),
            details: Some(json!({
                "requested": prompt.response.format,
                "supported": false,
            })),
        });
    }

    if !prompt.tools.is_empty() && !provider.constraints.supports.tools {
        report.add_entry(LossinessEntry {
            code: LossinessCode::Unsupported,
            level: LossinessLevel::Error,
            path: "tools".to_string(),
            reason: "Provider does not support tools".to_string(),
            suggested_fix: Some("Remove tools or use a different provider".to_string()),
            details: Some(json!({"tool_count": prompt.tools.len()})),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        Capabilities, Constraints, EndpointConfig, Endpoints, HttpMethod, Mappings, ModelConfig,
        RequestMapping, ResponseConfig, ResponseMapping, StrictMode, SupportFlags, Tool,
    };
    use std::collections::HashMap;

    fn provider_supports(json_mode: bool, tools: bool) -> ProviderSpec {
        ProviderSpec {
            provider: "fake".into(),
            models: vec![ModelConfig { id: "m".into() }],
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
                request: vec![RequestMapping {
                    from: "messages".into(),
                    to: "body.messages".into(),
                    code: None,
                    clamp: None,
                }],
                response: vec![ResponseMapping {
                    from: "body".into(),
                    to: "response".into(),
                }],
            },
            constraints: Constraints {
                supports: SupportFlags { json_mode, tools },
            },
            auth: crate::auth::AuthScheme::Bearer {
                token_env: "TOKEN".into(),
            },
            capabilities: Capabilities::default(),
            capabilities_extra: HashMap::new(),
            extensions: HashMap::new(),
            unsupported_parameters: Vec::new(),
        }
    }

    fn prompt_with_response(format: ResponseFormat, tool_count: usize) -> PromptSpec {
        PromptSpec {
            version: "1".into(),
            messages: vec![crate::types::Message {
                role: crate::types::MessageRole::User,
                content: "Hello".into(),
            }],
            sampling: Default::default(),
            response: ResponseConfig {
                format,
                ..Default::default()
            },
            tools: (0..tool_count)
                .map(|i| Tool {
                    name: format!("tool-{}", i),
                    description: None,
                    json_schema: serde_json::json!({"type": "object"}),
                })
                .collect(),
            tool_choice: None,
            strict_mode: StrictMode::Warn,
            metadata: Default::default(),
        }
    }

    #[test]
    fn warns_when_json_mode_not_supported() {
        let prompt = prompt_with_response(ResponseFormat::Json, 0);
        let provider = provider_supports(false, true);
        let mut report = LossinessReport::new(StrictMode::Warn);

        detect_unsupported(&prompt, &provider, &mut report);
        assert!(report.is_lossy);
        assert_eq!(report.entries[0].code, LossinessCode::Unsupported);
    }

    #[test]
    fn warns_for_json_schema_mode() {
        let prompt = prompt_with_response(ResponseFormat::JsonSchema, 0);
        let provider = provider_supports(false, true);
        let mut report = LossinessReport::new(StrictMode::Warn);

        detect_unsupported(&prompt, &provider, &mut report);
        assert!(report.is_lossy);
        assert_eq!(report.entries[0].path, "response.format");
    }

    #[test]
    fn errors_when_tools_unsupported() {
        let prompt = prompt_with_response(ResponseFormat::Text, 2);
        let provider = provider_supports(true, false);
        let mut report = LossinessReport::new(StrictMode::Warn);

        detect_unsupported(&prompt, &provider, &mut report);
        assert_eq!(report.entries.len(), 1);
        assert_eq!(report.entries[0].level, LossinessLevel::Error);
    }

    #[test]
    fn no_entries_when_supported() {
        let prompt = prompt_with_response(ResponseFormat::Json, 1);
        let provider = provider_supports(true, true);
        let mut report = LossinessReport::new(StrictMode::Warn);

        detect_unsupported(&prompt, &provider, &mut report);
        assert!(!report.is_lossy);
    }
}
