//! Provider discovery and registry system for automatic provider selection.
//!
//! This module implements a registry system that maps model names to their respective
//! providers, enabling automatic provider selection during translation.

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::Value;
mod error;
pub use error::{SpecadoError, SpecadoResult};

/// Provider information and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Provider name (e.g., "openai", "anthropic")
    pub name: String,
    /// Provider specification file path
    pub spec_path: PathBuf,
    /// List of model patterns this provider supports
    pub model_patterns: Vec<String>,
    /// Provider priority for fallback selection (higher = preferred)
    pub priority: u32,
}

/// Provider discovery and registry system
pub struct ProviderRegistry {
    /// Map of exact model names to providers
    exact_matches: HashMap<String, ProviderInfo>,
    /// List of pattern-based provider matches
    pattern_matches: Vec<(String, ProviderInfo)>,
    /// Default provider for fallback
    default_provider: Option<ProviderInfo>,
    /// Cache of loaded provider specs
    spec_cache: HashMap<String, Value>,
}

impl ProviderRegistry {
    /// Create a new provider registry
    pub fn new() -> Self {
        let mut registry = Self {
            exact_matches: HashMap::new(),
            pattern_matches: Vec::new(),
            default_provider: None,
            spec_cache: HashMap::new(),
        };
        
        // Initialize with built-in providers
        registry.register_builtin_providers();
        registry
    }
    
    /// Register built-in providers with their model patterns
    fn register_builtin_providers(&mut self) {
        // OpenAI GPT-5 models
        self.register_provider(ProviderInfo {
            name: "openai".to_string(),
            spec_path: PathBuf::from("providers/openai/gpt-5.json"),
            model_patterns: vec![
                "gpt-5".to_string(),
                "gpt-5-thinking".to_string(),
            ],
            priority: 100,
        });
        
        self.register_provider(ProviderInfo {
            name: "openai".to_string(),
            spec_path: PathBuf::from("providers/openai/gpt-5-mini.json"),
            model_patterns: vec![
                "gpt-5-mini".to_string(),
                "gpt-5-thinking-mini".to_string(),
            ],
            priority: 100,
        });
        
        self.register_provider(ProviderInfo {
            name: "openai".to_string(),
            spec_path: PathBuf::from("providers/openai/gpt-5-nano.json"),
            model_patterns: vec![
                "gpt-5-nano".to_string(),
                "gpt-5-thinking-nano".to_string(),
            ],
            priority: 100,
        });
        
        // OpenAI GPT-4 models (patterns)
        self.register_pattern_provider(ProviderInfo {
            name: "openai".to_string(),
            spec_path: PathBuf::from("providers/openai/gpt-4.json"),
            model_patterns: vec![
                "gpt-4*".to_string(),
                "gpt-3.5*".to_string(),
            ],
            priority: 90,
        });
        
        // Anthropic Claude models
        self.register_provider(ProviderInfo {
            name: "anthropic".to_string(),
            spec_path: PathBuf::from("providers/anthropic/claude-opus-4.1.json"),
            model_patterns: vec![
                "claude-opus-4-1-20250805".to_string(),
                "claude-opus-4.1".to_string(),
                "claude-4-opus".to_string(),
            ],
            priority: 100,
        });
        
        // Anthropic Claude patterns
        self.register_pattern_provider(ProviderInfo {
            name: "anthropic".to_string(),
            spec_path: PathBuf::from("providers/anthropic/claude-3.json"),
            model_patterns: vec![
                "claude-3*".to_string(),
                "claude-instant*".to_string(),
                "claude-2*".to_string(),
            ],
            priority: 90,
        });
        
        // Set OpenAI as default fallback provider
        self.default_provider = Some(ProviderInfo {
            name: "openai".to_string(),
            spec_path: PathBuf::from("providers/openai/gpt-4.json"),
            model_patterns: vec![],
            priority: 50,
        });
    }
    
    /// Register a provider with exact model name matching
    pub fn register_provider(&mut self, provider: ProviderInfo) {
        for model in &provider.model_patterns {
            if !model.contains('*') {
                self.exact_matches.insert(model.clone(), provider.clone());
            }
        }
    }
    
    /// Register a provider with pattern-based matching
    pub fn register_pattern_provider(&mut self, provider: ProviderInfo) {
        for pattern in &provider.model_patterns {
            if pattern.contains('*') {
                self.pattern_matches.push((pattern.clone(), provider.clone()));
            }
        }
        // Sort by priority (highest first)
        self.pattern_matches.sort_by(|a, b| b.1.priority.cmp(&a.1.priority));
    }
    
    /// Discover the provider for a given model name
    pub fn discover_provider(&self, model_name: &str) -> SpecadoResult<&ProviderInfo> {
        // First, check exact matches
        if let Some(provider) = self.exact_matches.get(model_name) {
            return Ok(provider);
        }
        
        // Then, check pattern matches
        for (pattern, provider) in &self.pattern_matches {
            if self.matches_pattern(model_name, pattern) {
                return Ok(provider);
            }
        }
        
        // Finally, use default provider if available
        self.default_provider.as_ref()
            .ok_or_else(|| SpecadoError::ProviderNotFound {
                model: model_name.to_string(),
                available: self.list_available_models(),
            })
    }
    
    /// Check if a model name matches a pattern
    fn matches_pattern(&self, model: &str, pattern: &str) -> bool {
        if let Some(prefix) = pattern.strip_suffix('*') {
            model.starts_with(prefix)
        } else if let Some(suffix) = pattern.strip_prefix('*') {
            model.ends_with(suffix)
        } else if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                model.starts_with(parts[0]) && model.ends_with(parts[1])
            } else {
                false
            }
        } else {
            model == pattern
        }
    }
    
    /// List all available model names
    pub fn list_available_models(&self) -> Vec<String> {
        let mut models = Vec::new();
        
        // Add exact matches
        models.extend(self.exact_matches.keys().cloned());
        
        // Add pattern descriptions
        for (pattern, _) in &self.pattern_matches {
            models.push(format!("Pattern: {}", pattern));
        }
        
        models.sort();
        models
    }
    
    /// Load a provider specification from file
    pub fn load_provider_spec(&mut self, provider: &ProviderInfo) -> SpecadoResult<Value> {
        // Check cache first
        if let Some(spec) = self.spec_cache.get(&provider.name) {
            return Ok(spec.clone());
        }
        
        // Load from file
        let spec_path = &provider.spec_path;
        let spec_content = std::fs::read_to_string(spec_path)
            .map_err(|e| SpecadoError::IoError {
                path: spec_path.display().to_string(),
                operation: "read provider spec".to_string(),
                details: e.to_string(),
            })?;
        
        let spec: Value = serde_json::from_str(&spec_content)
            .map_err(|e| SpecadoError::ParseError {
                path: spec_path.display().to_string(),
                line: e.line(),
                column: e.column(),
                message: e.to_string(),
            })?;
        
        // Cache the loaded spec
        self.spec_cache.insert(provider.name.clone(), spec.clone());
        
        Ok(spec)
    }
    
    /// Get provider information by name
    pub fn get_provider_by_name(&self, name: &str) -> Option<&ProviderInfo> {
        self.exact_matches.values()
            .find(|p| p.name == name)
            .or_else(|| {
                self.pattern_matches.iter()
                    .map(|(_, p)| p)
                    .find(|p| p.name == name)
            })
            .or_else(|| {
                self.default_provider.as_ref()
                    .filter(|p| p.name == name)
            })
    }
    
    /// Set a custom default provider
    pub fn set_default_provider(&mut self, provider: ProviderInfo) {
        self.default_provider = Some(provider);
    }
    
    /// Clear the provider specification cache
    pub fn clear_cache(&mut self) {
        self.spec_cache.clear();
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for custom provider registration
pub struct ProviderRegistryBuilder {
    registry: ProviderRegistry,
}

impl Default for ProviderRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderRegistryBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            registry: ProviderRegistry {
                exact_matches: HashMap::new(),
                pattern_matches: Vec::new(),
                default_provider: None,
                spec_cache: HashMap::new(),
            },
        }
    }
    
    /// Add a provider with exact model matching
    pub fn with_provider(mut self, provider: ProviderInfo) -> Self {
        self.registry.register_provider(provider);
        self
    }
    
    /// Add a provider with pattern matching
    pub fn with_pattern_provider(mut self, provider: ProviderInfo) -> Self {
        self.registry.register_pattern_provider(provider);
        self
    }
    
    /// Set the default fallback provider
    pub fn with_default(mut self, provider: ProviderInfo) -> Self {
        self.registry.set_default_provider(provider);
        self
    }
    
    /// Include built-in providers
    pub fn with_builtin_providers(mut self) -> Self {
        self.registry.register_builtin_providers();
        self
    }
    
    /// Build the registry
    pub fn build(self) -> ProviderRegistry {
        self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exact_match_discovery() {
        let registry = ProviderRegistry::new();
        
        // Test exact matches
        let provider = registry.discover_provider("gpt-5").unwrap();
        assert_eq!(provider.name, "openai");
        
        let provider = registry.discover_provider("claude-opus-4.1").unwrap();
        assert_eq!(provider.name, "anthropic");
    }
    
    #[test]
    fn test_pattern_match_discovery() {
        let registry = ProviderRegistry::new();
        
        // Test pattern matches
        let provider = registry.discover_provider("gpt-4-turbo").unwrap();
        assert_eq!(provider.name, "openai");
        
        let provider = registry.discover_provider("claude-3-sonnet").unwrap();
        assert_eq!(provider.name, "anthropic");
    }
    
    #[test]
    fn test_fallback_to_default() {
        let registry = ProviderRegistry::new();
        
        // Test fallback for unknown model
        let provider = registry.discover_provider("unknown-model-xyz").unwrap();
        assert_eq!(provider.name, "openai"); // Default fallback
    }
    
    #[test]
    fn test_custom_provider_registration() {
        let mut registry = ProviderRegistry::new();
        
        // Register custom provider
        registry.register_provider(ProviderInfo {
            name: "custom".to_string(),
            spec_path: PathBuf::from("providers/custom/model.json"),
            model_patterns: vec!["custom-model-1".to_string()],
            priority: 150,
        });
        
        let provider = registry.discover_provider("custom-model-1").unwrap();
        assert_eq!(provider.name, "custom");
    }
    
    #[test]
    fn test_pattern_matching_logic() {
        let registry = ProviderRegistry::new();
        
        // Test prefix matching
        assert!(registry.matches_pattern("gpt-4-turbo", "gpt-4*"));
        assert!(!registry.matches_pattern("gpt-3.5-turbo", "gpt-4*"));
        
        // Test suffix matching
        assert!(registry.matches_pattern("my-model-gpt", "*gpt"));
        assert!(!registry.matches_pattern("my-model-claude", "*gpt"));
        
        // Test infix matching
        assert!(registry.matches_pattern("gpt-model-turbo", "gpt*turbo"));
        assert!(!registry.matches_pattern("claude-model-turbo", "gpt*turbo"));
    }
    
    #[test]
    fn test_list_available_models() {
        let registry = ProviderRegistry::new();
        let models = registry.list_available_models();
        
        assert!(models.contains(&"gpt-5".to_string()));
        assert!(models.contains(&"claude-opus-4.1".to_string()));
        assert!(models.iter().any(|m| m.contains("Pattern:")));
    }
    
    #[test]
    fn test_provider_priority() {
        let mut registry = ProviderRegistry::new();
        
        // Register overlapping patterns with different priorities
        registry.register_pattern_provider(ProviderInfo {
            name: "provider1".to_string(),
            spec_path: PathBuf::from("provider1.json"),
            model_patterns: vec!["test-*".to_string()],
            priority: 50,
        });
        
        registry.register_pattern_provider(ProviderInfo {
            name: "provider2".to_string(),
            spec_path: PathBuf::from("provider2.json"),
            model_patterns: vec!["test-*".to_string()],
            priority: 100,
        });
        
        // Higher priority should win
        let provider = registry.discover_provider("test-model").unwrap();
        assert_eq!(provider.name, "provider2");
    }
    
    #[test]
    fn test_builder_pattern() {
        let registry = ProviderRegistryBuilder::new()
            .with_provider(ProviderInfo {
                name: "test".to_string(),
                spec_path: PathBuf::from("test.json"),
                model_patterns: vec!["test-model".to_string()],
                priority: 100,
            })
            .with_builtin_providers()
            .build();
        
        // Custom provider should be registered
        let provider = registry.discover_provider("test-model").unwrap();
        assert_eq!(provider.name, "test");
        
        // Built-in providers should also be available
        let provider = registry.discover_provider("gpt-5").unwrap();
        assert_eq!(provider.name, "openai");
    }
}