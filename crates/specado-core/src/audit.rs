use crate::error::Error;
use crate::types::{LossinessReport, ProviderSpec, UniformResponse};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tracing::warn;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AuditTarget {
    Stdout,
    File { path: PathBuf },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    #[serde(default)]
    pub target: Option<AuditTarget>,
    #[serde(default)]
    pub redact: Vec<String>,
}

impl AuditConfig {
    pub fn disabled() -> Self {
        Self {
            target: None,
            redact: Vec::new(),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.target.is_some()
    }
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self::disabled()
    }
}

#[derive(Debug)]
pub struct AuditContext {
    config: AuditConfig,
    logger: Option<JsonlAuditLogger>,
    redactor: Redactor,
    correlation_id: Uuid,
    start_time: Instant,
    provider: Option<String>,
    model: Option<String>,
}

impl AuditContext {
    pub fn new(config: AuditConfig) -> Self {
        let logger = config
            .target
            .as_ref()
            .and_then(|target| JsonlAuditLogger::new(target.clone()).ok());

        let mut patterns = DEFAULT_REDACTION
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        patterns.extend(config.redact.iter().cloned());
        let redactor = Redactor::new(patterns);

        Self {
            config,
            logger,
            redactor,
            correlation_id: Uuid::new_v4(),
            start_time: Instant::now(),
            provider: None,
            model: None,
        }
    }

    pub fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }

    pub fn reset_timer(&mut self) {
        self.start_time = Instant::now();
    }

    pub fn note_provider(&mut self, spec: &ProviderSpec) {
        self.provider = Some(spec.provider.clone());
        self.model = spec.models.first().map(|m| m.id.clone());
    }

    pub fn record_success(
        &mut self,
        translated_request: &Value,
        response: &UniformResponse,
        lossiness: &LossinessReport,
    ) {
        if !self.config.is_enabled() {
            return;
        }

        let latency_ms = self.start_time.elapsed().as_millis();
        let mut request = translated_request.clone();
        self.redactor.redact(&mut request);

        let response_excerpt = serde_json::to_value(response)
            .unwrap_or(Value::String("<serialization failed>".into()));

        let event = AuditEvent {
            timestamp: now_rfc3339(),
            correlation_id: self.correlation_id.to_string(),
            provider: self.provider.clone().unwrap_or_else(|| "unknown".into()),
            model: self.model.clone(),
            latency_ms,
            status: AuditStatus::Success,
            error_kind: None,
            error_message: None,
            lossiness: Some(lossiness.clone()),
            request_redacted: request,
            response_excerpt: Some(response_excerpt),
        };

        if let Some(logger) = self.logger.as_mut() {
            if let Err(err) = logger.log_event(&event) {
                warn!(target: "specado::audit", "failed to write audit event: {}", err);
            }
        }
    }

    pub fn record_error(
        &mut self,
        translated_request: Option<&Value>,
        error: &Error,
        response_excerpt: Option<&Value>,
    ) {
        if !self.config.is_enabled() {
            return;
        }

        let latency_ms = self.start_time.elapsed().as_millis();
        let mut request = translated_request.cloned().unwrap_or(Value::Null);
        self.redactor.redact(&mut request);

        let event = AuditEvent {
            timestamp: now_rfc3339(),
            correlation_id: self.correlation_id.to_string(),
            provider: self.provider.clone().unwrap_or_else(|| "unknown".into()),
            model: self.model.clone(),
            latency_ms,
            status: AuditStatus::Error,
            error_kind: Some(error_to_kind(error)),
            error_message: Some(error.to_string()),
            lossiness: None,
            request_redacted: request,
            response_excerpt: response_excerpt.cloned(),
        };

        if let Some(logger) = self.logger.as_mut() {
            if let Err(err) = logger.log_event(&event) {
                warn!(target: "specado::audit", "failed to write audit event: {}", err);
            }
        }
    }
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into())
}

fn error_to_kind(error: &Error) -> String {
    match error {
        Error::StrictModeViolation => "strict_mode_violation".into(),
        Error::Config(_) => "config_error".into(),
        Error::SchemaValidation(_) => "schema_validation".into(),
        Error::Provider { kind, .. } => format!("provider::{:?}", kind),
        Error::Transform(_) => "transform_error".into(),
        Error::CircuitBreakerOpen => "circuit_breaker_open".into(),
        Error::CircuitBreakerHalfOpen => "circuit_breaker_half_open".into(),
        Error::Http(_) => "http_error".into(),
        Error::Json(_) => "json_error".into(),
        Error::Io(_) => "io_error".into(),
        Error::Auth(_) => "auth_error".into(),
    }
}

#[derive(Debug)]
struct Redactor {
    patterns: Vec<Regex>,
}

impl Redactor {
    fn new(patterns: Vec<String>) -> Self {
        let compiled = patterns
            .into_iter()
            .filter_map(|p| Regex::new(&p).ok())
            .collect();
        Self { patterns: compiled }
    }

    fn redact(&self, value: &mut Value) {
        match value {
            Value::Object(map) => {
                for (key, entry) in map.iter_mut() {
                    if self.should_redact(key) {
                        *entry = Value::String("[REDACTED]".into());
                    } else {
                        self.redact(entry);
                    }
                }
            }
            Value::Array(items) => {
                for item in items.iter_mut() {
                    self.redact(item);
                }
            }
            _ => {}
        }
    }

    fn should_redact(&self, key: &str) -> bool {
        self.patterns.iter().any(|pattern| pattern.is_match(key))
    }
}

#[derive(Debug)]
struct JsonlAuditLogger {
    target: AuditTarget,
}

impl JsonlAuditLogger {
    fn new(target: AuditTarget) -> std::io::Result<Self> {
        match &target {
            AuditTarget::Stdout => Ok(Self { target }),
            AuditTarget::File { path } => {
                OpenOptions::new().create(true).append(true).open(path)?;
                Ok(Self { target })
            }
        }
    }

    fn log_event(&mut self, event: &AuditEvent) -> std::io::Result<()> {
        let json = serde_json::to_string(event)?;
        match &self.target {
            AuditTarget::Stdout => {
                let mut stdout = std::io::stdout().lock();
                stdout.write_all(json.as_bytes())?;
                stdout.write_all(b"\n")?;
            }
            AuditTarget::File { path } => {
                let mut file = OpenOptions::new().create(true).append(true).open(path)?;
                file.write_all(json.as_bytes())?;
                file.write_all(b"\n")?;
            }
        };
        Ok(())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum AuditStatus {
    Success,
    Error,
}

#[derive(Debug, Serialize)]
struct AuditEvent {
    timestamp: String,
    correlation_id: String,
    provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    latency_ms: u128,
    status: AuditStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lossiness: Option<LossinessReport>,
    request_redacted: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_excerpt: Option<Value>,
}

const DEFAULT_REDACTION: &[&str] = &[
    "(?i)^authorization$",
    "(?i)^bearer$",
    "(?i)secret",
    "(?i)api[-_]?key",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Capabilities;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn redacts_matching_keys() {
        let mut value = json!({
            "authorization": "Bearer secret",
            "nested": {
                "apiKey": "123",
                "visible": "ok"
            }
        });

        let redactor = Redactor::new(vec!["(?i)authorization".into(), "(?i)apikey".into()]);
        redactor.redact(&mut value);

        assert_eq!(value["authorization"], "[REDACTED]");
        assert_eq!(value["nested"]["apiKey"], "[REDACTED]");
        assert_eq!(value["nested"]["visible"], "ok");
    }

    #[test]
    fn audit_context_records_success_without_panic() {
        let config = AuditConfig {
            target: None,
            redact: vec![],
        };
        let mut ctx = AuditContext::new(config);
        ctx.note_provider(&ProviderSpec {
            provider: "demo".into(),
            models: vec![crate::types::ModelConfig { id: "m".into() }],
            interface: Some("conversational.generate".into()),
            contract_version: Some("1.0.0".into()),
            inherits: None,
            endpoints: crate::types::Endpoints {
                chat: crate::types::EndpointConfig {
                    method: crate::types::HttpMethod::Post,
                    url: "https://example.com".into(),
                    headers: Default::default(),
                },
            },
            mappings: crate::types::Mappings {
                request: vec![],
                response: vec![],
            },
            constraints: crate::types::Constraints {
                supports: crate::types::SupportFlags {
                    json_mode: false,
                    tools: false,
                },
            },
            auth: crate::auth::AuthScheme::Custom {
                headers: HashMap::new(),
            },
            capabilities: Capabilities::default(),
            capabilities_extra: HashMap::new(),
            extensions: HashMap::new(),
            unsupported_parameters: Vec::new(),
        });

        ctx.record_success(
            &json!({"messages": [{"role": "user", "content": "hi"}]}),
            &UniformResponse {
                content: "ok".into(),
                tool_calls: Vec::new(),
                finish_reason: crate::types::FinishReason::Stop,
                model: "demo".into(),
                provider_used: "demo".into(),
                usage: None,
                extensions: crate::types::Extensions {
                    lossiness: LossinessReport::new(crate::types::StrictMode::Warn),
                    provider_capabilities: None,
                },
            },
            &LossinessReport::new(crate::types::StrictMode::Warn),
        );
    }

    #[test]
    fn audit_context_records_error_without_panic() {
        let mut ctx = AuditContext::new(AuditConfig::disabled());
        ctx.record_error(None, &Error::StrictModeViolation, None);
    }
}
