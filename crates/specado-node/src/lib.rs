#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::Deserialize;
#[cfg(feature = "audit-logging")]
use specado::audit::{AuditConfig, AuditContext, AuditTarget};
use specado::{execute, PromptSpec, UniformResponse};
use std::path::PathBuf;

#[napi]
pub struct Client {
    provider_path: String,
    _watch_enabled: bool,
    #[cfg(feature = "audit-logging")]
    audit_config: Option<AuditConfig>,
}

#[napi]
impl Client {
    #[napi(constructor)]
    pub fn new(provider_path: String, options: Option<serde_json::Value>) -> Result<Self> {
        let parsed: NodeClientOptions = options
            .map(|value| {
                serde_json::from_value(value).map_err(|e| Error::from_reason(e.to_string()))
            })
            .transpose()?
            .unwrap_or_default();

        let watch_enabled = parsed.watch.as_ref().map(|opt| opt.enable).unwrap_or(false);

        #[cfg(feature = "audit-logging")]
        let audit_config = if let Some(audit) = parsed.audit.as_ref() {
            audit.to_config()?
        } else {
            None
        };

        Ok(Self {
            provider_path,
            _watch_enabled: watch_enabled,
            #[cfg(feature = "audit-logging")]
            audit_config,
        })
    }

    #[napi]
    pub async fn complete(&self, prompt: serde_json::Value) -> Result<serde_json::Value> {
        let prompt_spec: PromptSpec = serde_json::from_value(prompt)
            .map_err(|e| Error::from_reason(format!("Invalid prompt spec: {e}")))?;

        #[cfg(feature = "audit-logging")]
        let audit_context = self.audit_config.clone().map(AuditContext::new);

        let response: UniformResponse = execute(
            prompt_spec,
            &self.provider_path,
            #[cfg(feature = "audit-logging")]
            audit_context,
        )
        .await
        .map_err(|e| Error::from_reason(format!("Execution failed: {e}")))?;

        serde_json::to_value(&response)
            .map_err(|e| Error::from_reason(format!("Serialization failed: {e}")))
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct NodeClientOptions {
    watch: Option<NodeWatchOptions>,
    audit: Option<NodeAuditOptions>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct NodeWatchOptions {
    enable: bool,
    paths: Vec<String>,
    debounce_ms: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct NodeAuditOptions {
    target: Option<NodeAuditTarget>,
    redact: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum NodeAuditTarget {
    StdoutLiteral(String),
    FileObject { file: String },
}

impl NodeAuditOptions {
    #[cfg(feature = "audit-logging")]
    fn to_config(&self) -> Result<Option<AuditConfig>> {
        let target = match self.target.as_ref() {
            None => return Ok(None),
            Some(NodeAuditTarget::StdoutLiteral(value)) => {
                if value.eq_ignore_ascii_case("stdout") {
                    Some(AuditTarget::Stdout)
                } else {
                    return Err(Error::from_reason(
                        "Unsupported audit target. Use 'stdout' or { file: ... }".to_string(),
                    ));
                }
            }
            Some(NodeAuditTarget::FileObject { file }) => Some(AuditTarget::File {
                path: PathBuf::from(file),
            }),
        };

        Ok(Some(AuditConfig {
            target,
            redact: self.redact.clone(),
        }))
    }

    #[cfg(not(feature = "audit-logging"))]
    fn to_config(&self) -> Result<Option<()>> {
        Ok(None)
    }
}
