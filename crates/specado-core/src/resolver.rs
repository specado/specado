use crate::error::{Error, Result};
use crate::hot_reload::ProviderCache;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn resolve_provider_path_internal(
    provider: Option<&str>,
    model: Option<&str>,
    providers_dir_override: Option<&Path>,
) -> Result<PathBuf> {
    if provider.is_none() {
        if let Some(model_flag) = model {
            if !model_flag.trim().is_empty() {
                return Err(Error::Config(
                    "--model requires --provider. Pass both flags or omit --model.".into(),
                ));
            }
        }
        return default_provider_path_internal(providers_dir_override);
    }

    let provider_flag = provider.unwrap().trim();
    if provider_flag.is_empty() {
        return Err(Error::Config("--provider cannot be empty".into()));
    }

    let cache = ProviderCache::new();
    let looks_like_path = Path::new(provider_flag).is_absolute()
        || provider_flag.contains(std::path::MAIN_SEPARATOR)
        || provider_flag.contains('/')
        || provider_flag.contains('\\')
        || provider_flag.starts_with('.')
        || provider_flag.ends_with(".yaml")
        || provider_flag.ends_with(".yml")
        || provider_flag.ends_with(".json");

    if looks_like_path {
        let provider_path = PathBuf::from(provider_flag);
        if provider_path.is_file() {
            validate_model_for_path(&provider_path, model, &cache)?;
            return Ok(provider_path);
        }

        if provider_path.is_dir() {
            return resolve_provider_from_dir(&provider_path, model, &cache);
        }

        if let Ok(resolved) = provider_path.canonicalize() {
            if resolved.is_file() {
                validate_model_for_path(&resolved, model, &cache)?;
                return Ok(resolved);
            }
            if resolved.is_dir() {
                return resolve_provider_from_dir(&resolved, model, &cache);
            }
        }
    }

    let providers_dir = locate_providers_dir(providers_dir_override)?;
    let joined_path = providers_dir.join(provider_flag);
    if joined_path.is_file() {
        validate_model_for_path(&joined_path, model, &cache)?;
        return Ok(joined_path);
    }
    if joined_path.is_dir() {
        return resolve_provider_from_dir(&joined_path, model, &cache);
    }

    if looks_like_path {
        return Err(Error::Config(format!(
            "Provider spec '{}' not found. Pass an existing path or provider name.",
            provider_flag
        )));
    }

    let provider_dir = providers_dir.join(provider_flag);
    if !provider_dir.is_dir() {
        let available = list_available_providers(&providers_dir)?;
        let hint = if available.is_empty() {
            "No providers found in the catalog. Set SPECADO_PROVIDERS_DIR or pass a provider spec path."
                .to_string()
        } else {
            format!("Known providers: {}", available.join(", "))
        };
        return Err(Error::Config(format!(
            "Unknown provider '{}'. {}",
            provider_flag, hint
        )));
    }

    resolve_provider_from_dir(&provider_dir, model, &cache)
}

pub(crate) fn default_provider_path_internal(
    providers_dir_override: Option<&Path>,
) -> Result<PathBuf> {
    if let Ok(path) = env::var("SPECADO_DEFAULT_PROVIDER") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Ok(path);
        }
        return Err(Error::Config(format!(
            "Provider path specified via SPECADO_DEFAULT_PROVIDER does not exist: {}",
            path.display()
        )));
    }

    let providers_dir = locate_providers_dir(providers_dir_override)?;
    let fallback = providers_dir.join("openai/gpt-5/base.yaml");
    if fallback.exists() {
        return Ok(fallback);
    }

    Err(Error::Config(
        "Default provider spec not found. Set SPECADO_DEFAULT_PROVIDER to a valid provider YAML."
            .into(),
    ))
}

fn validate_model_for_path(path: &Path, model: Option<&str>, cache: &ProviderCache) -> Result<()> {
    let Some(model_id) = model.filter(|m| !m.trim().is_empty()) else {
        return Ok(());
    };

    let spec = cache.load_or_read(path).map_err(|err| {
        Error::Config(format!(
            "Failed to load provider spec {}: {}",
            path.display(),
            err
        ))
    })?;

    if spec
        .models
        .iter()
        .any(|entry| entry.id.eq_ignore_ascii_case(model_id))
    {
        return Ok(());
    }

    let available: Vec<String> = spec.models.iter().map(|m| m.id.clone()).collect();
    if available.is_empty() {
        return Err(Error::Config(format!(
            "Provider spec {} does not list any models; cannot validate --model {}",
            path.display(),
            model_id
        )));
    }

    Err(Error::Config(format!(
        "Model '{}' not available in {}. Available models: {}",
        model_id,
        path.display(),
        available.join(", ")
    )))
}

fn resolve_provider_from_dir(
    dir: &Path,
    model: Option<&str>,
    cache: &ProviderCache,
) -> Result<PathBuf> {
    let candidates = collect_provider_candidates(dir, cache)?;
    if candidates.is_empty() {
        return Err(Error::Config(format!(
            "No provider specifications found under {}",
            dir.display()
        )));
    }

    if let Some(model_id) = model.filter(|m| !m.trim().is_empty()) {
        if let Some(candidate) = candidates.iter().find(|candidate| {
            candidate
                .models
                .iter()
                .any(|id| id.eq_ignore_ascii_case(model_id))
        }) {
            return Ok(candidate.path.clone());
        }

        let available = collect_unique_models(&candidates);
        let provider_name = dir
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .unwrap_or_else(|| dir.to_string_lossy().into_owned());

        if available.is_empty() {
            return Err(Error::Config(format!(
                "Model '{}' not found for provider '{}'. Specify a provider spec path instead.",
                model_id, provider_name
            )));
        }

        return Err(Error::Config(format!(
            "Model '{}' not found for provider '{}'. Available models: {}",
            model_id,
            provider_name,
            available.join(", ")
        )));
    }

    if let Some(candidate) = pick_default_candidate(&candidates) {
        return Ok(candidate.path.clone());
    }

    Err(Error::Config(format!(
        "Unable to determine a default spec for {}. Pass --model to disambiguate.",
        dir.display()
    )))
}

fn collect_provider_candidates(
    dir: &Path,
    cache: &ProviderCache,
) -> Result<Vec<ProviderCandidate>> {
    let mut stack = vec![dir.to_path_buf()];
    let mut candidates = Vec::new();

    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current).map_err(|err| {
            Error::Config(format!(
                "Failed to read provider directory {}: {}",
                current.display(),
                err
            ))
        })? {
            let entry = entry.map_err(|err| {
                Error::Config(format!(
                    "Failed to read provider entry under {}: {}",
                    current.display(),
                    err
                ))
            })?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if !is_spec_file(&path) {
                continue;
            }

            let spec = cache.load_or_read(&path).map_err(|err| {
                Error::Config(format!(
                    "Failed to load provider spec {}: {}",
                    path.display(),
                    err
                ))
            })?;

            let models = spec.models.iter().map(|m| m.id.clone()).collect();
            candidates.push(ProviderCandidate { path, models });
        }
    }

    Ok(candidates)
}

fn is_spec_file(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    if file_name.starts_with('_') || file_name.ends_with(".md") {
        return false;
    }

    matches!(
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext, "yaml" | "yml" | "json")),
        Some(true)
    )
}

fn pick_default_candidate(candidates: &[ProviderCandidate]) -> Option<&ProviderCandidate> {
    if candidates.len() == 1 {
        return candidates.first();
    }

    let priority_names = ["base.yaml", "base.yml", "chat.yaml", "chat.yml"];
    for name in priority_names {
        if let Some(candidate) = candidates.iter().find(|candidate| {
            candidate
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|entry| entry.eq_ignore_ascii_case(name))
                .unwrap_or(false)
        }) {
            return Some(candidate);
        }
    }

    candidates
        .iter()
        .min_by_key(|candidate| candidate.path.to_string_lossy().to_ascii_lowercase())
}

fn collect_unique_models(candidates: &[ProviderCandidate]) -> Vec<String> {
    let mut models = BTreeSet::new();
    for candidate in candidates {
        for model in &candidate.models {
            models.insert(model.clone());
        }
    }
    models.into_iter().collect()
}

fn locate_providers_dir(providers_dir_override: Option<&Path>) -> Result<PathBuf> {
    if let Some(dir) = providers_dir_override {
        if dir.is_dir() {
            return Ok(dir.to_path_buf());
        }
        return Err(Error::Config(format!(
            "Provided providers directory {} is not a directory",
            dir.display()
        )));
    }

    if let Ok(dir) = env::var("SPECADO_PROVIDERS_DIR") {
        let path = PathBuf::from(dir);
        if path.is_dir() {
            return Ok(path);
        }
        return Err(Error::Config(format!(
            "SPECADO_PROVIDERS_DIR points to {}, which is not a directory",
            path.display()
        )));
    }

    #[cfg(feature = "cli-resolver")]
    {
        // During CLI development we fall back to the workspace catalog so that
        // `cargo run` works without additional configuration. Published
        // libraries rely on an explicit providers directory or the
        // SPECADO_PROVIDERS_DIR environment variable and will skip this branch.
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_dir = manifest_dir.join("../specado-providers/providers");
        if workspace_dir.is_dir() {
            return Ok(workspace_dir);
        }

        let repo_relative = PathBuf::from("crates/specado-providers/providers");
        if repo_relative.is_dir() {
            return Ok(repo_relative);
        }
    }

    Err(Error::Config(
        "Unable to locate provider catalog. Set SPECADO_PROVIDERS_DIR or pass a provider spec path."
            .into(),
    ))
}

fn list_available_providers(root: &Path) -> Result<Vec<String>> {
    let mut providers = Vec::new();
    for entry in fs::read_dir(root).map_err(|err| {
        Error::Config(format!(
            "Failed to read providers directory {}: {}",
            root.display(),
            err
        ))
    })? {
        let entry = entry.map_err(|err| {
            Error::Config(format!(
                "Failed to read entry under {}: {}",
                root.display(),
                err
            ))
        })?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                providers.push(name.to_string());
            }
        }
    }
    providers.sort_unstable();
    Ok(providers)
}

struct ProviderCandidate {
    path: PathBuf,
    models: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn catalog_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../specado-providers/providers")
    }

    #[test]
    fn resolves_friendly_provider_with_override() {
        let path =
            resolve_provider_path_internal(Some("openai"), None, Some(catalog_dir().as_path()))
                .expect("resolve provider");

        assert!(path.ends_with("openai/gpt-5/base.yaml"));
        assert!(path.exists());
    }

    #[test]
    fn validates_model_against_spec() {
        let path = resolve_provider_path_internal(
            Some("openai"),
            Some("gpt-5"),
            Some(catalog_dir().as_path()),
        )
        .expect("resolve provider with model");

        assert!(path.exists());

        let err = resolve_provider_path_internal(
            Some("openai"),
            Some("non-existent-model"),
            Some(catalog_dir().as_path()),
        )
        .expect_err("unknown model should error");

        match err {
            Error::Config(message) => {
                assert!(message.contains("non-existent-model"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
