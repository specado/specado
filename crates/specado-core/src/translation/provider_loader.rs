//! Provider specification loading with discovery integration

use std::path::PathBuf;
use crate::provider_discovery::ProviderRegistry;
use crate::error::{Error, Result};
use crate::types::ProviderSpec;

/// Load a provider specification for a given model
pub fn load_provider_for_model(model_name: &str) -> Result<ProviderSpec> {
    let mut registry = ProviderRegistry::new();
    
    // Discover the provider
    let provider_info = registry.discover_provider(model_name)
        .map_err(|e| Error::Provider {
            provider: "unknown".to_string(),
            message: format!("Failed to discover provider for model '{}': {}", model_name, e),
            source: None,
        })?
        .clone(); // Clone to avoid borrow issues
    
    // Load the provider spec
    let spec_json = registry.load_provider_spec(&provider_info)
        .map_err(|e| Error::Provider {
            provider: provider_info.name.clone(),
            message: format!("Failed to load provider spec: {}", e),
            source: None,
        })?;
    
    // Parse into ProviderSpec
    serde_json::from_value(spec_json)
        .map_err(|e| Error::Json {
            message: format!("Failed to parse provider spec: {}", e),
            source: e,
        })
}

/// Load a provider specification from a specific path
pub fn load_provider_from_path(path: &PathBuf) -> Result<ProviderSpec> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| Error::Io {
            message: format!("Failed to read provider spec from {:?}", path),
            source: e,
        })?;
    
    serde_json::from_str(&content)
        .map_err(|e| Error::Json {
            message: format!("Failed to parse provider spec from {:?}", path),
            source: e,
        })
}

/// Get available providers and their models
pub fn list_available_providers() -> Vec<(String, Vec<String>)> {
    let registry = ProviderRegistry::new();
    let models = registry.list_available_models();
    
    // Group by provider (simplified - in real implementation would be more sophisticated)
    let mut providers = vec![
        ("openai".to_string(), vec![]),
        ("anthropic".to_string(), vec![]),
    ];
    
    for model in models {
        if model.starts_with("gpt") || model.contains("gpt") {
            providers[0].1.push(model);
        } else if model.starts_with("claude") || model.contains("claude") {
            providers[1].1.push(model);
        }
    }
    
    providers
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_list_available_providers() {
        let providers = list_available_providers();
        assert!(!providers.is_empty());
        
        // Should have OpenAI and Anthropic
        assert!(providers.iter().any(|(name, _)| name == "openai"));
        assert!(providers.iter().any(|(name, _)| name == "anthropic"));
    }
}