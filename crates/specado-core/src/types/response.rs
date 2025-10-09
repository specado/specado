use super::lossiness::LossinessReport;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UniformResponse {
    pub content: String,
    #[serde(default)]
    pub tool_calls: Vec<serde_json::Value>,
    pub finish_reason: FinishReason,
    pub model: String,
    pub provider_used: String,
    #[serde(default)]
    pub usage: Option<Usage>,
    pub extensions: Extensions,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ToolCall,
    ContentFilter,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Extensions {
    pub lossiness: LossinessReport,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_capabilities: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::lossiness::{LossinessEntry, LossinessLevel};
    use crate::types::prompt::StrictMode;
    use serde_json::json;

    #[test]
    fn round_trip_uniform_response() {
        let mut report = LossinessReport::new(StrictMode::Warn);
        report.add_entry(LossinessEntry {
            code: crate::types::lossiness::LossinessCode::Drop,
            level: LossinessLevel::Info,
            path: "response.metadata".into(),
            reason: "Metadata omitted".into(),
            suggested_fix: None,
            details: None,
        });

        let response = UniformResponse {
            content: "Hello".into(),
            tool_calls: vec![json!({"id": "call-1"})],
            finish_reason: FinishReason::Stop,
            model: "gpt-4o".into(),
            provider_used: "openai".into(),
            usage: Some(Usage {
                prompt_tokens: 12,
                completion_tokens: 34,
            }),
            extensions: Extensions {
                lossiness: report,
                provider_capabilities: Some(json!({"context_window": 128000})),
            },
        };

        let serialized = serde_json::to_string(&response).expect("serialize");
        let decoded: UniformResponse = serde_json::from_str(&serialized).expect("deserialize");

        assert_eq!(decoded.finish_reason, FinishReason::Stop);
        assert_eq!(decoded.tool_calls.len(), 1);
        assert!(decoded.extensions.lossiness.is_lossy);
        assert_eq!(
            decoded
                .extensions
                .provider_capabilities
                .as_ref()
                .and_then(|value| value.get("context_window"))
                .and_then(|v| v.as_u64()),
            Some(128000)
        );
    }

    #[test]
    fn default_fields_populate() {
        let payload = json!({
            "content": "Hi",
            "finish_reason": "error",
            "model": "mistral",
            "provider_used": "anthropic",
            "extensions": {
                "lossiness": {
                    "is_lossy": false,
                    "strict_mode": "Warn",
                    "entries": [],
                    "omissions": []
                }
            }
        });

        let decoded: UniformResponse = serde_json::from_value(payload).expect("decode");
        assert!(decoded.tool_calls.is_empty());
        assert!(decoded.usage.is_none());
        assert!(!decoded.extensions.lossiness.is_lossy);
        assert!(decoded.extensions.provider_capabilities.is_none());
    }
}
