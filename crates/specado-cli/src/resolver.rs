use anyhow::{anyhow, Context, Result};
use specado_core::hot_reload::ProviderCache;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn resolve_provider_path(provider: Option<&str>, model: Option<&str>) -> Result<PathBuf> {
    if provider.is_none() {
        if let Some(model_flag) = model {
            if !model_flag.trim().is_empty() {
                return Err(anyhow!(
                    "--model requires --provider. Pass both flags or omit --model."
                ));
            }
        }
        return default_provider_path();
    }

    let provider_flag = provider.unwrap().trim();
    if provider_flag.is_empty() {
        return Err(anyhow!("--provider cannot be empty"));
    }

    let cache = ProviderCache::new();
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
    }

    let providers_dir = locate_providers_dir()?;
    let joined_path = providers_dir.join(provider_flag);
    if joined_path.is_file() {
        validate_model_for_path(&joined_path, model, &cache)?;
        return Ok(joined_path);
    }
    if joined_path.is_dir() {
        return resolve_provider_from_dir(&joined_path, model, &cache);
    }

    if provider_flag.contains(std::path::MAIN_SEPARATOR)
        || provider_flag.contains('/')
        || provider_flag.contains('\\')
        || provider_flag.ends_with(".yaml")
        || provider_flag.ends_with(".yml")
    {
        return Err(anyhow!(
            "Provider spec '{}' not found. Pass an existing path or provider name.",
            provider_flag
        ));
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
        return Err(anyhow!("Unknown provider '{}'. {}", provider_flag, hint));
    }

    resolve_provider_from_dir(&provider_dir, model, &cache)
}

pub fn default_provider_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("SPECADO_DEFAULT_PROVIDER") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Ok(path);
        }
        return Err(anyhow!(
            "Provider path specified via SPECADO_DEFAULT_PROVIDER does not exist: {}",
            path.display()
        ));
    }

    let fallback = PathBuf::from("crates/specado-providers/providers/openai/gpt-5/base.yaml");
    if fallback.exists() {
        return Ok(fallback);
    }

    Err(anyhow!(
        "Default provider spec not found. Set SPECADO_DEFAULT_PROVIDER to a valid provider YAML."
    ))
}

fn validate_model_for_path(path: &Path, model: Option<&str>, cache: &ProviderCache) -> Result<()> {
    let Some(model_id) = model.filter(|m| !m.trim().is_empty()) else {
        return Ok(());
    };

    let spec = cache
        .load_or_read(path)
        .map_err(|err| anyhow!("Failed to load provider spec {}: {}", path.display(), err))?;

    if spec
        .models
        .iter()
        .any(|entry| entry.id.eq_ignore_ascii_case(model_id))
    {
        return Ok(());
    }

    let available: Vec<String> = spec.models.iter().map(|m| m.id.clone()).collect();
    if available.is_empty() {
        return Err(anyhow!(
            "Provider spec {} does not list any models; cannot validate --model {}",
            path.display(),
            model_id
        ));
    }

    Err(anyhow!(
        "Model '{}' not available in {}. Available models: {}",
        model_id,
        path.display(),
        available.join(", ")
    ))
}

fn resolve_provider_from_dir(
    dir: &Path,
    model: Option<&str>,
    cache: &ProviderCache,
) -> Result<PathBuf> {
    let candidates = collect_provider_candidates(dir, cache)?;
    if candidates.is_empty() {
        return Err(anyhow!(
            "No provider specifications found under {}",
            dir.display()
        ));
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
            return Err(anyhow!(
                "Model '{}' not found for provider '{}'. Specify a provider spec path instead.",
                model_id,
                provider_name
            ));
        }

        return Err(anyhow!(
            "Model '{}' not found for provider '{}'. Available models: {}",
            model_id,
            provider_name,
            available.join(", ")
        ));
    }

    if let Some(candidate) = pick_default_candidate(&candidates) {
        return Ok(candidate.path.clone());
    }

    Err(anyhow!(
        "Unable to determine a default spec for {}. Pass --model to disambiguate.",
        dir.display()
    ))
}

fn collect_provider_candidates(
    dir: &Path,
    cache: &ProviderCache,
) -> Result<Vec<ProviderCandidate>> {
    let mut stack = vec![dir.to_path_buf()];
    let mut candidates = Vec::new();

    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current)
            .with_context(|| format!("Failed to read provider directory: {}", current.display()))?
        {
            let entry = entry.with_context(|| {
                format!("Failed to read provider entry under {}", current.display())
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
                anyhow!("Failed to load provider spec {}: {}", path.display(), err)
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

fn locate_providers_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("SPECADO_PROVIDERS_DIR") {
        let path = PathBuf::from(dir);
        if path.is_dir() {
            return Ok(path);
        }
        return Err(anyhow!(
            "SPECADO_PROVIDERS_DIR points to {}, which is not a directory",
            path.display()
        ));
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = manifest_dir.join("../specado-providers/providers");
    if workspace_dir.is_dir() {
        return Ok(workspace_dir);
    }

    let repo_relative = PathBuf::from("crates/specado-providers/providers");
    if repo_relative.is_dir() {
        return Ok(repo_relative);
    }

    Err(anyhow!(
        "Unable to locate provider catalog. Set SPECADO_PROVIDERS_DIR or pass a provider spec path."
    ))
}

fn list_available_providers(root: &Path) -> Result<Vec<String>> {
    let mut providers = Vec::new();
    for entry in fs::read_dir(root)
        .with_context(|| format!("Failed to read providers directory: {}", root.display()))?
    {
        let entry =
            entry.with_context(|| format!("Failed to read entry under {}", root.display()))?;
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
