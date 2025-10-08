use crate::error::{Error, Result};
use crate::types::ProviderSpec;
use once_cell::sync::Lazy;
use serde::Serialize;
use serde_json::to_value;
use specado_schemas::get_validator;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

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

        let contents = fs::read_to_string(&canonical)
            .map_err(|e| Error::Config(format!("Failed to read provider spec: {}", e)))?;

        let spec: ProviderSpec = serde_yaml::from_str(&contents)
            .map_err(|e| Error::Config(format!("Failed to parse provider spec: {}", e)))?;

        let spec_value = to_value(&spec)
            .map_err(|e| Error::Config(format!("Failed to serialize provider spec: {}", e)))?;
        get_validator()
            .validate_provider(&spec_value)
            .map_err(|e| Error::SchemaValidation(e.to_string()))?;

        let refreshed_modified = fs::metadata(&canonical)
            .ok()
            .and_then(|m| m.modified().ok())
            .or(modified)
            .unwrap_or_else(|| SystemTime::now());

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
    use tempfile::NamedTempFile;

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
}
