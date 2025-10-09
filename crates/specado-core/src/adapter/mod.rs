use crate::types::ProviderApi;
use crate::types::ProviderSpec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterSelection {
    kind: ProviderApi,
    match_rule: AdapterMatchRule,
    overlays: Vec<String>,
}

impl AdapterSelection {
    fn new(kind: ProviderApi, match_rule: AdapterMatchRule) -> Self {
        Self {
            kind,
            match_rule,
            overlays: Vec::new(),
        }
    }

    pub fn kind(&self) -> ProviderApi {
        self.kind
    }

    pub fn match_rule(&self) -> &AdapterMatchRule {
        &self.match_rule
    }

    pub fn overlays(&self) -> &[String] {
        &self.overlays
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterMatchRule {
    InterfaceHint(String),
    EndpointUrl(String),
    ProviderName(String),
    Default,
}

pub struct AdapterRegistry;

impl AdapterRegistry {
    pub fn select(provider: &ProviderSpec) -> AdapterSelection {
        if let Some(interface) = provider
            .interface_hint()
            .filter(|hint| !hint.starts_with("x_"))
        {
            return AdapterSelection::new(
                provider.api_kind(),
                AdapterMatchRule::InterfaceHint(interface.to_string()),
            );
        }

        let url = provider.endpoints.chat.url.to_ascii_lowercase();
        if url.contains("/responses") {
            return AdapterSelection::new(
                ProviderApi::OpenaiResponses,
                AdapterMatchRule::EndpointUrl(url),
            );
        }
        if url.contains("/messages") {
            return AdapterSelection::new(
                ProviderApi::AnthropicMessagesClaude4,
                AdapterMatchRule::EndpointUrl(url),
            );
        }
        if provider.provider.eq_ignore_ascii_case("anthropic") {
            return AdapterSelection::new(
                ProviderApi::AnthropicMessagesClaude4,
                AdapterMatchRule::ProviderName(provider.provider.clone()),
            );
        }

        AdapterSelection::new(ProviderApi::ChatCompletions, AdapterMatchRule::Default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        Capabilities, Constraints, EndpointConfig, Endpoints, HttpMethod, Mappings, ModelConfig,
        ProviderSpec, RequestMapping, ResponseMapping, SupportFlags,
    };
    use std::collections::HashMap;

    fn provider(url: &str, interface: Option<&str>, provider_name: &str) -> ProviderSpec {
        ProviderSpec {
            provider: provider_name.into(),
            models: vec![ModelConfig { id: "m".into() }],
            interface: interface.map(|s| s.into()),
            contract_version: Some("1.0.0".into()),
            inherits: None,
            endpoints: Endpoints {
                chat: EndpointConfig {
                    method: HttpMethod::Post,
                    url: url.into(),
                    headers: HashMap::new(),
                },
            },
            mappings: Mappings {
                request: vec![RequestMapping {
                    from: "$.messages".into(),
                    to: "$.messages".into(),
                    code: None,
                    clamp: None,
                }],
                response: vec![ResponseMapping {
                    from: "$.choices[0].message".into(),
                    to: "content".into(),
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

    #[test]
    fn selects_by_interface_hint() {
        let spec = provider(
            "https://api.openai.com/v1/chat/completions",
            Some("conversational.generate"),
            "openai",
        );
        let selection = AdapterRegistry::select(&spec);
        assert_eq!(selection.kind(), ProviderApi::ChatCompletions);
        assert!(matches!(
            selection.match_rule(),
            AdapterMatchRule::InterfaceHint(hint) if hint == "conversational.generate"
        ));
    }

    #[test]
    fn selects_by_endpoint_url() {
        let spec = provider("https://api.openai.com/v1/responses", None, "openai");
        let selection = AdapterRegistry::select(&spec);
        assert_eq!(selection.kind(), ProviderApi::OpenaiResponses);
        assert!(matches!(
            selection.match_rule(),
            AdapterMatchRule::EndpointUrl(_)
        ));
    }

    #[test]
    fn selects_by_provider_name() {
        let spec = provider("https://api.anthropic.com/v1/other", None, "anthropic");
        let selection = AdapterRegistry::select(&spec);
        assert_eq!(selection.kind(), ProviderApi::AnthropicMessagesClaude4);
        assert!(
            matches!(selection.match_rule(), AdapterMatchRule::ProviderName(name) if name == "anthropic")
        );
    }

    #[test]
    fn falls_back_to_default() {
        let spec = provider("https://example.com/chat", None, "custom");
        let selection = AdapterRegistry::select(&spec);
        assert_eq!(selection.kind(), ProviderApi::ChatCompletions);
        assert_eq!(selection.match_rule(), &AdapterMatchRule::Default);
    }
}
