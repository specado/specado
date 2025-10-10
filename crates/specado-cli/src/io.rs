use anyhow::{anyhow, Context, Result};
use serde_json::Value as JsonValue;
use specado_core::{PromptSpec, ProviderSpec};
use std::fs;
use std::path::Path;

pub fn parse_to_json_value(content: &str, path: &Path) -> Result<JsonValue> {
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if ext == "yaml" || ext == "yml" {
        Ok(serde_yaml::from_str(content)?)
    } else {
        match serde_json::from_str(content) {
            Ok(value) => Ok(value),
            Err(_) => Ok(serde_yaml::from_str(content)?),
        }
    }
}

pub fn load_prompt_spec(path: &Path) -> Result<PromptSpec> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read prompt spec: {}", path.display()))?;

    match serde_json::from_str(&content) {
        Ok(spec) => Ok(spec),
        Err(json_err) => serde_yaml::from_str(&content).map_err(|yaml_err| {
            anyhow!("Failed to parse prompt spec as JSON ({json_err}) or YAML ({yaml_err})")
        }),
    }
}

pub fn load_provider_spec(path: &Path) -> Result<ProviderSpec> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read provider spec: {}", path.display()))?;

    match serde_yaml::from_str(&content) {
        Ok(spec) => Ok(spec),
        Err(yaml_err) => serde_json::from_str(&content).map_err(|json_err| {
            anyhow!("Failed to parse provider spec as YAML ({yaml_err}) or JSON ({json_err})")
        }),
    }
}
