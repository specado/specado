//! Shared utilities for command handlers

use crate::config::Config;
use crate::error::{Error, Result};
use specado_core::ProviderSpec;
use std::fs;
use std::path::Path;

/// Load a provider specification
pub fn load_provider_spec(provider: &str, config: &Config) -> Result<ProviderSpec> {
    // Check if provider is a file path
    let provider_path = Path::new(provider);
    
    let spec_path = if provider_path.exists() {
        provider_path.to_path_buf()
    } else {
        // Try to find provider in standard locations
        let mut paths_to_try = vec![
            // Local providers directory
            Path::new("providers").join(provider).with_extension("json"),
            Path::new("providers").join(provider).with_extension("yaml"),
            // Providers/{provider}/{provider}.json
            Path::new("providers")
                .join(provider)
                .join(format!("{}.json", provider)),
            Path::new("providers")
                .join(provider)
                .join(format!("{}.yaml", provider)),
        ];
        
        // Add config providers directory
        paths_to_try.push(config.paths.providers_dir.join(format!("{}.json", provider)));
        paths_to_try.push(config.paths.providers_dir.join(format!("{}.yaml", provider)));
        
        paths_to_try
            .into_iter()
            .find(|p| p.exists())
            .ok_or_else(|| Error::ProviderNotFound {
                name: provider.to_string(),
            })?
    };
    
    // Load and parse the provider spec
    let content = fs::read_to_string(&spec_path)?;
    
    let provider_spec: ProviderSpec = if spec_path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s == "yaml" || s == "yml")
        .unwrap_or(false)
    {
        serde_yaml::from_str(&content)?
    } else {
        serde_json::from_str(&content)?
    };
    
    Ok(provider_spec)
}