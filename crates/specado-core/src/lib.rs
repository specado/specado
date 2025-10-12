#![allow(dead_code)]

pub mod adapter;
#[cfg(feature = "audit-logging")]
pub mod audit;
pub mod auth;
pub mod circuit_breaker;
pub mod error;
pub mod hot_reload;
pub mod http;
pub mod retry;
pub mod router;
pub mod transformer;
pub mod types;

pub use adapter::{AdapterMatchRule, AdapterRegistry, AdapterSelection};
pub use auth::{AuthError, AuthHandler, AuthScheme};
pub use circuit_breaker::CircuitBreaker;
pub use error::{Error, ProviderErrorKind, Result};
pub use retry::RetryPolicy;
pub use router::{PrimaryFallbackRouter, Router};
pub use types::*;

use crate::hot_reload::global_cache;
use crate::http::get_client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::StatusCode;
use serde_json::Value;
use std::path::Path;

#[cfg(feature = "audit-logging")]
use crate::audit::AuditContext;

pub async fn execute(
    prompt: PromptSpec,
    provider_path: &str,
    #[cfg(feature = "audit-logging")] mut audit: Option<AuditContext>,
) -> Result<UniformResponse> {
    #[cfg(feature = "audit-logging")]
    if let Some(ctx) = audit.as_mut() {
        ctx.reset_timer();
    }

    let provider_spec = match global_cache().load_or_read(Path::new(provider_path)) {
        Ok(spec) => {
            #[cfg(feature = "audit-logging")]
            if let Some(ctx) = audit.as_mut() {
                ctx.note_provider(&spec);
            }
            spec
        }
        Err(err) => {
            #[cfg(feature = "audit-logging")]
            if let Some(ctx) = audit.as_mut() {
                ctx.record_error(None, &err, None);
            }
            return Err(err);
        }
    };

    let auth_handler = AuthHandler::new(provider_spec.auth.clone());
    auth_handler.validate()?;

    let (translated, lossiness) = translate(&prompt, &provider_spec)?;

    if prompt.strict_mode == StrictMode::Strict && lossiness.is_lossy {
        #[cfg(feature = "audit-logging")]
        if let Some(ctx) = audit.as_mut() {
            ctx.record_error(Some(&translated), &Error::StrictModeViolation, None);
        }
        return Err(Error::StrictModeViolation);
    }

    let mut headers = provider_spec.endpoints.chat.headers.clone();
    auth_handler.inject_headers(&mut headers)?;

    let mut header_map = HeaderMap::with_capacity(headers.len());
    for (key, value) in headers {
        let name = HeaderName::from_bytes(key.as_bytes())
            .map_err(|e| Error::Config(format!("Invalid header name '{}': {}", key, e)))?;
        let header_value = HeaderValue::from_str(&value)
            .map_err(|e| Error::Config(format!("Invalid header value for '{}': {}", key, e)))?;
        header_map.insert(name, header_value);
    }

    let client = get_client();
    let response = match client
        .post(&provider_spec.endpoints.chat.url)
        .headers(header_map)
        .json(&translated)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(err) => {
            let error = Error::Http(err);
            #[cfg(feature = "audit-logging")]
            if let Some(ctx) = audit.as_mut() {
                ctx.record_error(Some(&translated), &error, None);
            }
            return Err(error);
        }
    };

    let status = response.status();
    if !status.is_success() {
        let kind = map_status_to_provider_error(status);
        #[cfg(feature = "audit-logging")]
        if let Some(ctx) = audit.as_mut() {
            let body_text = response.text().await.unwrap_or_default();
            let body_value = if body_text.trim().is_empty() {
                Value::Null
            } else {
                serde_json::from_str(&body_text).unwrap_or(Value::String(body_text))
            };
            ctx.record_error(
                Some(&translated),
                &Error::Provider {
                    provider: provider_spec.provider.clone(),
                    kind: kind.clone(),
                },
                Some(&body_value),
            );
            return Err(Error::Provider {
                provider: provider_spec.provider.clone(),
                kind,
            });
        }
        #[cfg(not(feature = "audit-logging"))]
        {
            let _ = response.text().await;
        }
        return Err(Error::Provider {
            provider: provider_spec.provider.clone(),
            kind,
        });
    }

    let raw_response: Value = response.json().await.map_err(Error::Http)?;

    let mut uniform_response = transformer::normalize(raw_response, &provider_spec)?;
    uniform_response.extensions.lossiness = lossiness;

    #[cfg(feature = "audit-logging")]
    if let Some(ctx) = audit.as_mut() {
        ctx.record_success(
            &translated,
            &uniform_response,
            &uniform_response.extensions.lossiness,
        );
    }

    Ok(uniform_response)
}

pub fn translate(prompt: &PromptSpec, provider: &ProviderSpec) -> Result<(Value, LossinessReport)> {
    transformer::translate(prompt, provider)
}

fn map_status_to_provider_error(status: StatusCode) -> ProviderErrorKind {
    match status {
        StatusCode::TOO_MANY_REQUESTS => ProviderErrorKind::RateLimit,
        StatusCode::REQUEST_TIMEOUT => ProviderErrorKind::Timeout,
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => ProviderErrorKind::AuthenticationFailed,
        code if code.is_client_error() => ProviderErrorKind::InvalidRequest,
        code if code.is_server_error() => ProviderErrorKind::ServerError,
        _ => ProviderErrorKind::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn sample_prompt() -> PromptSpec {
        PromptSpec {
            version: "1".into(),
            messages: vec![
                Message {
                    role: MessageRole::System,
                    content: "You are helpful.".into(),
                },
                Message {
                    role: MessageRole::User,
                    content: "Hello".into(),
                },
            ],
            sampling: SamplingConfig {
                temperature: Some(0.5),
                top_k: Some(20),
                ..Default::default()
            },
            response: ResponseConfig::default(),
            tools: Vec::new(),
            tool_choice: None,
            strict_mode: StrictMode::Warn,
            metadata: Default::default(),
        }
    }

    fn provider_yaml(url: &str, token_env: &str) -> String {
        format!(
            r#"provider: openai
models:
  - id: gpt-4o
auth:
  type: bearer
  token_env: {token_env}
endpoints:
  chat:
    method: POST
    url: {url}
    headers:
      content-type: application/json
mappings:
  request:
    - from: $.messages
      to: $.body.messages
  response:
    - from: $.data.content
      to: content
    - from: $.data.finish_reason
      to: finish_reason
constraints:
  supports:
    json_mode: true
    tools: true
"#,
            url = url,
            token_env = token_env
        )
    }

    #[tokio::test]
    async fn execute_sends_request_and_normalizes_response() {
        let server = MockServer::start();
        let token_env = "SPECADO_TEST_TOKEN";
        std::env::set_var(token_env, "secret-token");

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/chat")
                .header("authorization", "Bearer secret-token");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "data": {
                        "content": "hi there",
                        "finish_reason": "stop"
                    }
                }));
        });

        let mut tmp = NamedTempFile::new().expect("temp file");
        write!(tmp, "{}", provider_yaml(&server.url("/chat"), token_env)).expect("write spec");

        let response = execute(
            sample_prompt(),
            tmp.path().to_str().unwrap(),
            #[cfg(feature = "audit-logging")]
            None,
        )
        .await
        .expect("execute succeeds");

        mock.assert_hits(1);
        assert_eq!(response.content, "hi there");
        assert_eq!(response.finish_reason, FinishReason::Stop);
        assert_eq!(response.provider_used, "openai");

        std::env::remove_var(token_env);
    }

    #[tokio::test]
    async fn execute_enforces_strict_mode_before_http() {
        let server = MockServer::start();
        let token_env = "SPECADO_TEST_STRICT_TOKEN";
        std::env::set_var(token_env, "strict-token");

        let mock = server.mock(|when, then| {
            when.method(POST).path("/chat");
            then.status(200)
                .json_body(json!({"data": {"content": "unused"}}));
        });

        let provider_yaml = format!(
            r#"provider: strict
models:
  - id: m
auth:
  type: bearer
  token_env: {token_env}
endpoints:
  chat:
    method: POST
    url: {url}
    headers: {{}}
mappings:
  request: []
  response:
    - from: $.data.content
      to: content
constraints:
  supports:
    json_mode: false
    tools: false
"#,
            url = server.url("/chat"),
            token_env = token_env
        );

        let mut tmp = NamedTempFile::new().expect("temp file");
        write!(tmp, "{}", provider_yaml).expect("write spec");

        let mut prompt = sample_prompt();
        prompt.strict_mode = StrictMode::Strict;

        let err = execute(
            prompt,
            tmp.path().to_str().unwrap(),
            #[cfg(feature = "audit-logging")]
            None,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, Error::StrictModeViolation));
        mock.assert_hits(0);

        std::env::remove_var(token_env);
    }
}
