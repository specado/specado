use anyhow::Result;
use std::path::PathBuf;

pub fn resolve_provider_path(provider: Option<&str>, model: Option<&str>) -> Result<PathBuf> {
    specado::resolve_provider_path(provider, model, None).map_err(Into::into)
}
