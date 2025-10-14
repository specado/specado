#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::Deserialize;
#[cfg(feature = "audit-logging")]
use specado::audit::{AuditConfig, AuditContext, AuditTarget};
use specado::{
    create_prompt as core_create_prompt, execute, load_prompt_from_path,
    simple_prompt as core_simple_prompt, ExecuteOptions, PromptBuilder, PromptSpec,
    SimplePromptOptions, UniformResponse,
};
use std::path::PathBuf;

#[napi]
pub fn load_prompt(path: String) -> Result<serde_json::Value> {
    let prompt = load_prompt_from_path(&path).map_err(|e| Error::from_reason(e.to_string()))?;
    serde_json::to_value(&prompt)
        .map_err(|e| Error::from_reason(format!("Serialization failed: {e}")))
}

#[napi]
pub fn create_prompt(options: serde_json::Value) -> Result<serde_json::Value> {
    let builder: PromptBuilder = serde_json::from_value(options)
        .map_err(|e| Error::from_reason(format!("Invalid prompt options: {e}")))?;
    let prompt = core_create_prompt(builder);
    serde_json::to_value(&prompt)
        .map_err(|e| Error::from_reason(format!("Serialization failed: {e}")))
}

#[napi]
pub fn simple_prompt(options: Option<serde_json::Value>) -> Result<serde_json::Value> {
    let opts: SimplePromptOptions = options
        .map(|value| {
            serde_json::from_value(value)
                .map_err(|e| Error::from_reason(format!("Invalid prompt options: {e}")))
        })
        .transpose()?
        .unwrap_or_default();

    let prompt = core_simple_prompt(opts).map_err(|e| Error::from_reason(e.to_string()))?;
    serde_json::to_value(&prompt)
        .map_err(|e| Error::from_reason(format!("Serialization failed: {e}")))
}

#[napi]
pub struct Client {
    provider: String,
    model: Option<String>,
    providers_dir: Option<PathBuf>,
    _watch_enabled: bool,
    #[cfg(feature = "audit-logging")]
    audit_config: Option<AuditConfig>,
}

#[napi]
impl Client {
    #[napi(constructor)]
    pub fn new(provider: String, options: Option<serde_json::Value>) -> Result<Self> {
        let parsed: NodeClientOptions = options
            .map(|value| {
                serde_json::from_value(value).map_err(|e| Error::from_reason(e.to_string()))
            })
            .transpose()?
            .unwrap_or_default();

        let watch_enabled = parsed.watch.as_ref().map(|opt| opt.enable).unwrap_or(false);
        let model = parsed.model.clone();
        let providers_dir = parsed.providers_dir.as_ref().map(PathBuf::from);

        #[cfg(feature = "audit-logging")]
        let audit_config = if let Some(audit) = parsed.audit.as_ref() {
            audit.to_config()?
        } else {
            None
        };

        Ok(Self {
            provider,
            model,
            providers_dir,
            _watch_enabled: watch_enabled,
            #[cfg(feature = "audit-logging")]
            audit_config,
        })
    }

    #[napi]
    pub async fn complete(&self, prompt: serde_json::Value) -> Result<serde_json::Value> {
        let prompt_spec: PromptSpec = serde_json::from_value(prompt)
            .map_err(|e| Error::from_reason(format!("Invalid prompt spec: {e}")))?;
        self.execute_prompt(prompt_spec).await
    }

    #[napi]
    pub async fn complete_file(&self, path: String) -> Result<serde_json::Value> {
        let prompt = load_prompt_from_path(&path).map_err(|e| Error::from_reason(e.to_string()))?;
        self.execute_prompt(prompt).await
    }

    #[napi]
    pub async fn complete_text(
        &self,
        message: String,
        options: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let mut opts: SimplePromptOptions = options
            .map(|value| {
                serde_json::from_value(value)
                    .map_err(|e| Error::from_reason(format!("Invalid prompt options: {e}")))
            })
            .transpose()?
            .unwrap_or_default();

        if opts.message.is_none() && opts.user.is_none() {
            opts.message = Some(message);
        }

        let prompt = core_simple_prompt(opts).map_err(|e| Error::from_reason(e.to_string()))?;
        self.execute_prompt(prompt).await
    }
}

impl Client {
    async fn execute_prompt(&self, prompt_spec: PromptSpec) -> Result<serde_json::Value> {
        #[cfg(feature = "audit-logging")]
        let audit_context = self.audit_config.clone().map(AuditContext::new);

        let mut options = ExecuteOptions::default();
        if let Some(model) = self.model.as_ref() {
            options.model = Some(model.clone());
        }
        if let Some(dir) = self.providers_dir.as_ref() {
            options.providers_dir = Some(dir.clone());
        }

        let response: UniformResponse = execute(
            prompt_spec,
            &self.provider,
            options,
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
    model: Option<String>,
    providers_dir: Option<String>,
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
