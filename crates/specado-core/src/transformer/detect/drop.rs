use crate::types::{
    LossinessCode, LossinessEntry, LossinessLevel, LossinessReport, PromptSpec, ProviderSpec,
};
use serde_json::json;

pub fn detect_drops(prompt: &PromptSpec, provider: &ProviderSpec, report: &mut LossinessReport) {
    if let Some(top_k) = prompt.sampling.top_k {
        let has_top_k_mapping = provider
            .mappings
            .request
            .iter()
            .any(|mapping| mapping.from.contains("top_k"));

        if !has_top_k_mapping {
            report.add_entry(LossinessEntry {
                code: LossinessCode::Drop,
                level: LossinessLevel::Warn,
                path: "sampling.top_k".to_string(),
                reason: "Parameter not supported by provider".to_string(),
                suggested_fix: Some("Remove top_k or use a provider that supports it".to_string()),
                details: Some(json!({ "requested": top_k })),
            });
            report.add_omission("$.sampling.top_k".to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        Capabilities, Constraints, EndpointConfig, Endpoints, HttpMethod, Mappings, ModelConfig,
        RequestMapping, ResponseConfig, ResponseMapping, SamplingConfig, StrictMode, SupportFlags,
    };
    use std::collections::HashMap;

    fn provider_with_top_k_mapping(has_mapping: bool) -> ProviderSpec {
        let mut request = Vec::new();
        if has_mapping {
            request.push(RequestMapping {
                from: "sampling.top_k".into(),
                to: "body.top_k".into(),
                code: None,
                clamp: None,
            });
        }

        ProviderSpec {
            provider: "demo".into(),
            models: vec![ModelConfig { id: "demo".into() }],
            interface: Some("conversational.generate".into()),
            contract_version: Some("1.0.0".into()),
            inherits: None,
            endpoints: Endpoints {
                chat: EndpointConfig {
                    method: HttpMethod::Post,
                    url: "https://demo".into(),
                    headers: Default::default(),
                },
            },
            mappings: Mappings {
                request,
                response: vec![ResponseMapping {
                    from: "body".into(),
                    to: "response".into(),
                }],
            },
            constraints: Constraints {
                supports: SupportFlags {
                    json_mode: true,
                    tools: true,
                },
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

    fn prompt_with_top_k(value: Option<u32>) -> PromptSpec {
        PromptSpec {
            version: "1".into(),
            messages: vec![crate::types::Message {
                role: crate::types::MessageRole::User,
                content: "Hello".into(),
            }],
            sampling: SamplingConfig {
                top_k: value,
                ..Default::default()
            },
            response: ResponseConfig::default(),
            tools: Vec::new(),
            tool_choice: None,
            strict_mode: StrictMode::Warn,
            metadata: Default::default(),
        }
    }

    #[test]
    fn records_drop_when_mapping_missing() {
        let prompt = prompt_with_top_k(Some(50));
        let provider = provider_with_top_k_mapping(false);
        let mut report = LossinessReport::new(StrictMode::Warn);

        detect_drops(&prompt, &provider, &mut report);
        assert!(report.is_lossy);
        assert_eq!(report.entries[0].code, LossinessCode::Drop);
        assert_eq!(report.omissions, vec!["$.sampling.top_k"]);
    }

    #[test]
    fn no_drop_when_mapping_present() {
        let prompt = prompt_with_top_k(Some(50));
        let provider = provider_with_top_k_mapping(true);
        let mut report = LossinessReport::new(StrictMode::Warn);

        detect_drops(&prompt, &provider, &mut report);
        assert!(!report.is_lossy);
    }

    #[test]
    fn ignores_when_top_k_not_set() {
        let prompt = prompt_with_top_k(None);
        let provider = provider_with_top_k_mapping(false);
        let mut report = LossinessReport::new(StrictMode::Warn);

        detect_drops(&prompt, &provider, &mut report);
        assert!(!report.is_lossy);
    }
}
