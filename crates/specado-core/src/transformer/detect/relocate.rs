use crate::types::{
    LossinessCode, LossinessEntry, LossinessLevel, LossinessReport, MessageRole, PromptSpec,
    ProviderSpec,
};
use serde_json::json;

pub fn detect_relocate(prompt: &PromptSpec, provider: &ProviderSpec, report: &mut LossinessReport) {
    let has_system = prompt
        .messages
        .iter()
        .any(|m| m.role == MessageRole::System);

    if !has_system {
        return;
    }

    if let Some(mapping) = provider
        .mappings
        .request
        .iter()
        .find(|m| m.code.as_deref() == Some("Relocate"))
    {
        report.add_entry(LossinessEntry {
            code: LossinessCode::Relocate,
            level: LossinessLevel::Info,
            path: "messages[0]".to_string(),
            reason: "System message relocated to provider-specific location".to_string(),
            suggested_fix: None,
            details: Some(json!({
                "from": mapping.from,
                "to": mapping.to,
            })),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        Capabilities, Constraints, EndpointConfig, Endpoints, HttpMethod, Mappings, Message,
        ModelConfig, RequestMapping, ResponseMapping, StrictMode, SupportFlags,
    };
    use std::collections::HashMap;

    fn base_provider(code: Option<&str>) -> ProviderSpec {
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
                request: vec![RequestMapping {
                    from: "messages[0]".into(),
                    to: "body.system".into(),
                    code: code.map(|c| c.to_string()),
                    clamp: None,
                }],
                response: vec![ResponseMapping {
                    from: "body.choices[0].message".into(),
                    to: "prompt.response".into(),
                }],
            },
            constraints: Constraints {
                supports: SupportFlags {
                    json_mode: true,
                    tools: true,
                },
            },
            auth: crate::auth::AuthScheme::Bearer {
                token_env: "DUMMY".into(),
            },
            capabilities: Capabilities::default(),
            capabilities_extra: HashMap::new(),
            extensions: HashMap::new(),
            unsupported_parameters: Vec::new(),
        }
    }

    fn prompt_with_system(has_system: bool) -> PromptSpec {
        let mut messages = vec![Message {
            role: MessageRole::User,
            content: "Hello".into(),
        }];
        if has_system {
            messages.insert(
                0,
                Message {
                    role: MessageRole::System,
                    content: "System".into(),
                },
            );
        }

        PromptSpec {
            version: "1".into(),
            messages,
            sampling: Default::default(),
            response: Default::default(),
            tools: Vec::new(),
            tool_choice: None,
            strict_mode: StrictMode::Warn,
            metadata: Default::default(),
        }
    }

    #[test]
    fn records_relocate_when_mapping_present() {
        let prompt = prompt_with_system(true);
        let provider = base_provider(Some("Relocate"));
        let mut report = LossinessReport::new(StrictMode::Warn);

        detect_relocate(&prompt, &provider, &mut report);
        assert!(report.is_lossy);
        assert_eq!(report.entries.len(), 1);
        assert_eq!(report.entries[0].code, LossinessCode::Relocate);
    }

    #[test]
    fn no_entry_without_system_message() {
        let prompt = prompt_with_system(false);
        let provider = base_provider(Some("Relocate"));
        let mut report = LossinessReport::new(StrictMode::Warn);

        detect_relocate(&prompt, &provider, &mut report);
        assert!(!report.is_lossy);
    }

    #[test]
    fn no_entry_without_relocate_mapping() {
        let prompt = prompt_with_system(true);
        let provider = base_provider(None);
        let mut report = LossinessReport::new(StrictMode::Warn);

        detect_relocate(&prompt, &provider, &mut report);
        assert!(!report.is_lossy);
    }
}
