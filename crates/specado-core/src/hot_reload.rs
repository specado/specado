use crate::adapter::AdapterRegistry;
use crate::error::{Error, Result};
use crate::types::ProviderSpec;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::{to_value, Map as JsonMap, Value as JsonValue};
use specado_schemas::get_validator;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

fn load_spec_value(path: &Path, visited: &mut HashSet<PathBuf>) -> Result<JsonValue> {
    let canonical = path
        .canonicalize()
        .map_err(|e| Error::Config(format!("Failed to canonicalize provider spec: {}", e)))?;

    if !visited.insert(canonical.clone()) {
        return Err(Error::Config(format!(
            "Cyclic provider inheritance detected at {}",
            canonical.display()
        )));
    }

    let contents = fs::read_to_string(&canonical)
        .map_err(|e| Error::Config(format!("Failed to read provider spec: {}", e)))?;

    let value: JsonValue = serde_yaml::from_str(&contents)
        .map_err(|e| Error::Config(format!("Failed to parse provider spec: {}", e)))?;

    let mut map = value.as_object().cloned().ok_or_else(|| {
        Error::Config(format!(
            "Provider spec at {} must be a YAML mapping",
            canonical.display()
        ))
    })?;

    let inherits = map
        .remove("inherits")
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    let mut result = if let Some(relative) = inherits {
        let base_path = canonical
            .parent()
            .map(|dir| dir.join(&relative))
            .unwrap_or_else(|| PathBuf::from(relative));
        let mut base_value = load_spec_value(&base_path, visited)?;
        merge_values(&mut base_value, &JsonValue::Object(map));
        base_value
    } else {
        JsonValue::Object(map)
    };

    visited.remove(&canonical);
    if let JsonValue::Object(ref mut final_map) = result {
        final_map.remove("inherits");
    }

    Ok(result)
}

fn merge_values(base: &mut JsonValue, overlay: &JsonValue) {
    if let JsonValue::Object(overlay_map) = overlay {
        if let JsonValue::Object(base_map) = base {
            for (key, value) in overlay_map {
                if value.is_null() {
                    base_map.remove(key);
                    continue;
                }

                match base_map.get_mut(key) {
                    Some(existing @ JsonValue::Object(_)) if value.is_object() => {
                        merge_values(existing, value);
                    }
                    _ => {
                        base_map.insert(key.clone(), value.clone());
                    }
                }
            }
            return;
        }
    }

    *base = overlay.clone();
}

fn load_overlays(spec_path: &Path) -> Result<Vec<OverlaySpec>> {
    let overlays_dir = spec_path.ancestors().find_map(|ancestor| {
        let candidate = ancestor.join("overlays");
        if candidate.is_dir() {
            Some(candidate)
        } else {
            None
        }
    });

    let overlays_dir = match overlays_dir {
        Some(dir) => dir,
        None => return Ok(Vec::new()),
    };

    let mut overlays = Vec::new();
    for entry in fs::read_dir(overlays_dir)
        .map_err(|e| Error::Config(format!("Failed to read overlays directory: {}", e)))?
    {
        let path = entry
            .map_err(|e| Error::Config(format!("Failed to read overlay entry: {}", e)))?
            .path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }

        let contents = fs::read_to_string(&path).map_err(|e| {
            Error::Config(format!("Failed to read overlay {}: {}", path.display(), e))
        })?;

        let file: OverlayFile = serde_yaml::from_str(&contents).map_err(|e| {
            Error::Config(format!("Failed to parse overlay {}: {}", path.display(), e))
        })?;

        overlays.push(OverlaySpec {
            target: file.overlay_for,
            data: file.data,
            source: path,
        });
    }

    Ok(overlays)
}

fn merge_overlays(value: JsonValue, overlays: Vec<OverlaySpec>) -> Result<JsonValue> {
    if overlays.is_empty() {
        return Ok(value);
    }

    let provider: ProviderSpec = serde_json::from_value(value.clone()).map_err(|e| {
        Error::Config(format!(
            "Failed to deserialize provider spec before overlays: {}",
            e
        ))
    })?;

    let mut combined = value;

    let adapter = AdapterRegistry::select(&provider);
    let adapter_key = adapter.kind().registry_key();
    let provider_contract = provider.contract_version().unwrap_or("1.0.0");

    for overlay in overlays {
        if !overlay
            .target
            .provider
            .eq_ignore_ascii_case(&provider.provider)
        {
            continue;
        }

        if !overlay.target.adapter.eq_ignore_ascii_case(adapter_key) {
            continue;
        }

        if overlay.target.contract_version != provider_contract {
            return Err(Error::Config(format!(
                "Overlay {} targets contract version {} but spec uses {}",
                overlay.source.display(),
                overlay.target.contract_version,
                provider_contract
            )));
        }

        merge_values(&mut combined, &JsonValue::Object(overlay.data.clone()));
    }

    Ok(combined)
}

#[derive(Debug)]
struct OverlaySpec {
    target: OverlayTarget,
    data: JsonMap<String, JsonValue>,
    source: PathBuf,
}

#[derive(Debug, Deserialize)]
struct OverlayFile {
    overlay_for: OverlayTarget,
    #[serde(flatten)]
    data: JsonMap<String, JsonValue>,
}

#[derive(Debug, Deserialize)]
struct OverlayTarget {
    provider: String,
    adapter: String,
    contract_version: String,
}

/// Configuration for experimental hot-reload support.
#[derive(Debug, Clone, Serialize)]
pub struct HotReloadConfig {
    pub enable: bool,
    pub watch_paths: Vec<PathBuf>,
    pub debounce_ms: u64,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            enable: false,
            watch_paths: Vec::new(),
            debounce_ms: 250,
        }
    }
}

impl HotReloadConfig {
    pub fn disabled() -> Self {
        Self::default()
    }

    pub fn enabled(paths: Vec<PathBuf>, debounce: Duration) -> Self {
        Self {
            enable: true,
            watch_paths: paths,
            debounce_ms: debounce.as_millis() as u64,
        }
    }

    pub fn debounce_duration(&self) -> Duration {
        Duration::from_millis(self.debounce_ms.max(1))
    }
}

#[derive(Debug, Clone)]
struct CachedProvider {
    spec: ProviderSpec,
    last_loaded: SystemTime,
}

/// Lightweight cache that will be backed by a file watcher once implemented.
#[derive(Debug, Clone)]
pub struct ProviderCache {
    inner: Arc<RwLock<HashMap<PathBuf, CachedProvider>>>,
}

impl Default for ProviderCache {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl ProviderCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Read and cache the provider spec, updating the entry if it has changed.
    pub fn load_or_read(&self, provider_path: &Path) -> Result<ProviderSpec> {
        let path = provider_path;
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        let metadata = fs::metadata(&canonical)
            .map_err(|e| Error::Config(format!("Failed to read provider spec metadata: {}", e)))?;
        let modified = metadata.modified().ok();

        if let Some(cached) = self
            .inner
            .read()
            .expect("provider cache poisoned while reading")
            .get(&canonical)
        {
            if let Some(modified_time) = modified {
                if modified_time <= cached.last_loaded {
                    return Ok(cached.spec.clone());
                }
            }
        }

        let mut visited = HashSet::new();
        let merged_value = load_spec_value(&canonical, &mut visited)?;
        let overlays = load_overlays(&canonical)?;
        let merged_value = merge_overlays(merged_value, overlays)?;
        let mut spec: ProviderSpec = serde_json::from_value(merged_value)
            .map_err(|e| Error::Config(format!("Failed to parse provider spec: {}", e)))?;
        spec.inherits = None;

        let spec_value = to_value(&spec)
            .map_err(|e| Error::Config(format!("Failed to serialize provider spec: {}", e)))?;
        get_validator()
            .validate_provider(&spec_value)
            .map_err(|e| Error::SchemaValidation(e.to_string()))?;

        let refreshed_modified = fs::metadata(&canonical)
            .ok()
            .and_then(|m| m.modified().ok())
            .or(modified)
            .unwrap_or_else(SystemTime::now);

        {
            let mut cache = self
                .inner
                .write()
                .expect("provider cache poisoned while writing");
            cache.insert(
                canonical,
                CachedProvider {
                    spec: spec.clone(),
                    last_loaded: refreshed_modified,
                },
            );
        }

        Ok(spec)
    }
}

static GLOBAL_CACHE: Lazy<ProviderCache> = Lazy::new(ProviderCache::new);
static GLOBAL_CONFIG: Lazy<RwLock<HotReloadConfig>> =
    Lazy::new(|| RwLock::new(HotReloadConfig::disabled()));

pub fn global_cache() -> &'static ProviderCache {
    &GLOBAL_CACHE
}

pub fn set_global_config(config: HotReloadConfig) {
    let mut guard = GLOBAL_CONFIG
        .write()
        .expect("hot reload global config poisoned");
    *guard = config;
}

pub fn current_config() -> HotReloadConfig {
    GLOBAL_CONFIG
        .read()
        .expect("hot reload global config poisoned")
        .clone()
}

/// Handle returned by the hot-reload runtime (currently a stub).
#[cfg(feature = "hot-reload")]
#[derive(Debug)]
pub struct HotReloadHandle {
    _private: (),
}

#[cfg(feature = "hot-reload")]
impl HotReloadHandle {
    pub fn stop(self) {}
}

/// Starts the hot-reload runtime. This is a stub while the watcher integration is designed.
#[cfg(feature = "hot-reload")]
pub fn start_hot_reload(_config: HotReloadConfig, _cache: ProviderCache) -> HotReloadHandle {
    HotReloadHandle { _private: () }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn hot_reload_config_defaults_to_disabled() {
        let cfg = HotReloadConfig::default();
        assert!(!cfg.enable);
        assert_eq!(cfg.watch_paths.len(), 0);
        assert_eq!(cfg.debounce_duration(), Duration::from_millis(250));
    }

    #[test]
    fn provider_cache_reads_from_disk() {
        let yaml = r#"
provider: demo
models:
  - id: demo
auth:
  type: bearer
  token_env: TEST_TOKEN
endpoints:
  chat:
    method: POST
    url: https://example.com
    headers: {}
mappings:
  request: []
  response: []
constraints:
  supports:
    json_mode: false
    tools: false
"#;

        let mut tmp = NamedTempFile::new().expect("temp file");
        std::io::Write::write_all(&mut tmp, yaml.as_bytes()).expect("write spec");

        let cache = ProviderCache::new();
        let spec = cache.load_or_read(tmp.path()).expect("provider spec loads");
        assert_eq!(spec.provider, "demo");
    }

    #[test]
    fn provider_cache_reloads_when_file_changes() {
        let valid = r#"
provider: demo
models:
  - id: demo
auth:
  type: bearer
  token_env: TEST_TOKEN
endpoints:
  chat:
    method: POST
    url: https://example.com
    headers: {}
mappings:
  request: []
  response: []
constraints:
  supports:
    json_mode: false
    tools: false
"#;

        let invalid = "not: yaml";

        let tmp = NamedTempFile::new().expect("temp file");
        std::fs::write(tmp.path(), valid).expect("write spec");

        let cache = ProviderCache::new();
        cache
            .load_or_read(tmp.path())
            .expect("initial provider spec loads");

        std::thread::sleep(std::time::Duration::from_millis(50));
        std::fs::write(tmp.path(), invalid).expect("write invalid spec");

        let err = cache
            .load_or_read(tmp.path())
            .expect_err("invalid spec should trigger reload failure");
        match err {
            Error::Config(message) => {
                assert!(
                    message.contains("Failed to parse provider spec"),
                    "unexpected error: {message}"
                );
            }
            other => panic!("expected config error, got {other:?}"),
        }
    }

    #[test]
    fn load_or_read_merges_inherited_specs() {
        let dir = tempdir().expect("temp dir");
        let base_path = dir.path().join("base.yaml");
        fs::write(
            &base_path,
            r#"provider: demo
models:
  - id: base
api: chat_completions
auth:
  type: bearer
  token_env: BASE_KEY
endpoints:
  chat:
    method: POST
    url: https://api.example.com
    headers: {}
mappings:
  request:
    - from: "$.messages"
      to: "$.body.messages"
  response:
    - from: "$.body.result"
      to: "content"
constraints:
  supports:
    json_mode: true
    tools: true
capabilities:
  supports_tools: true
"#,
        )
        .expect("write base spec");

        let child_path = dir.path().join("child.yaml");
        fs::write(
            &child_path,
            r#"inherits: base.yaml
models:
  - id: child
capabilities:
  context_window: 2000
unsupported_parameters:
  - "$.sampling.temperature"
"#,
        )
        .expect("write child spec");

        let cache = ProviderCache::new();
        let spec = cache
            .load_or_read(&child_path)
            .expect("merged provider spec");

        assert_eq!(spec.provider, "demo");
        assert_eq!(spec.models[0].id, "child");
        assert_eq!(spec.endpoints.chat.url, "https://api.example.com");
        assert!(spec.capabilities.supports_tools);
        assert_eq!(spec.capabilities.context_window, Some(2000));
        assert_eq!(spec.unsupported_parameters, vec!["$.sampling.temperature"]);
    }

    #[test]
    fn load_or_read_applies_overlays() {
        let dir = tempdir().expect("temp dir");
        let spec_path = dir.path().join("openai.yaml");
        let overlays_dir = dir.path().join("overlays");
        fs::create_dir(&overlays_dir).expect("overlay dir");
        fs::write(
            overlays_dir.join("openai.responses.yaml"),
            r#"overlay_for:
  provider: openai
  adapter: openai_responses
  contract_version: "1.0.0"

extensions:
  x-specado:
    request_defaults:
      max_output_tokens: 1024
"#,
        )
        .expect("overlay file");

        fs::write(
            &spec_path,
            r#"provider: openai
models:
  - id: test
interface: text.generate
contract_version: "1.0.0"
endpoints:
  chat:
    method: POST
    url: https://api.openai.com/v1/responses
    headers: {}
mappings:
  request: []
  response: []
constraints:
  supports:
    json_mode: true
    tools: true
auth:
  type: bearer
  token_env: KEY
"#,
        )
        .expect("write spec");

        let cache = ProviderCache::new();
        let spec = cache.load_or_read(&spec_path).expect("spec with overlays");

        assert_eq!(spec.provider, "openai");
        let defaults = spec
            .extensions
            .get("x-specado")
            .and_then(|value| value.get("request_defaults"));
        assert_eq!(
            defaults
                .and_then(|value| value.get("max_output_tokens"))
                .and_then(|v| v.as_i64()),
            Some(1024)
        );
    }
}
