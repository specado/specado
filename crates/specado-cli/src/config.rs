//! Configuration management for the CLI
//!
//! This module handles loading and merging configuration from:
//! - Default values
//! - Configuration files (YAML/JSON)
//! - Environment variables
//! - Command-line arguments

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Default provider to use if not specified
    pub default_provider: Option<String>,
    
    /// Default model to use if not specified
    pub default_model: Option<String>,
    
    /// Provider configurations
    pub providers: HashMap<String, ProviderConfig>,
    
    /// Output settings
    pub output: OutputConfig,
    
    /// Logging settings
    pub logging: LoggingConfig,
    
    /// Path settings
    pub paths: PathConfig,
}

/// Provider-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API key for this provider
    pub api_key: Option<String>,
    
    /// Base URL override
    pub base_url: Option<String>,
    
    /// Default model for this provider
    pub default_model: Option<String>,
    
    /// Additional headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
    
    /// Timeout in seconds
    pub timeout: Option<u64>,
    
    /// Maximum retries
    pub max_retries: Option<u32>,
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputConfig {
    /// Default output format
    pub format: String,
    
    /// Use colored output by default
    pub color: bool,
    
    /// Show progress indicators
    pub progress: bool,
    
    /// Default verbosity level
    pub verbosity: u8,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    
    /// Log format (compact, full, json)
    pub format: String,
    
    /// Log file path
    pub file: Option<PathBuf>,
    
    /// Include timestamps
    pub timestamps: bool,
    
    /// Include thread IDs
    pub thread_ids: bool,
}

/// Path configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PathConfig {
    /// Directory for provider specifications
    pub providers_dir: PathBuf,
    
    /// Directory for prompt specifications
    pub prompts_dir: Option<PathBuf>,
    
    /// Directory for schemas
    pub schemas_dir: Option<PathBuf>,
    
    /// Cache directory
    pub cache_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_provider: None,
            default_model: None,
            providers: HashMap::new(),
            output: OutputConfig::default(),
            logging: LoggingConfig::default(),
            paths: PathConfig::default(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: "human".to_string(),
            color: true,
            progress: true,
            verbosity: 0,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "compact".to_string(),
            file: None,
            timestamps: true,
            thread_ids: false,
        }
    }
}

impl Default for PathConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let config_dir = home_dir.join(".config").join("specado");
        
        Self {
            providers_dir: config_dir.join("providers"),
            prompts_dir: None,
            schemas_dir: None,
            cache_dir: dirs::cache_dir()
                .map(|d| d.join("specado"))
                .unwrap_or_else(|| home_dir.join(".cache").join("specado")),
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        
        let config = if path.extension().and_then(|s| s.to_str()) == Some("yaml") 
            || path.extension().and_then(|s| s.to_str()) == Some("yml") {
            serde_yaml::from_str(&content)?
        } else {
            serde_json::from_str(&content)?
        };
        
        Ok(config)
    }
    
    /// Load configuration from default locations
    pub fn load() -> Result<Self> {
        // Try to load from default config locations
        let config_paths = Self::default_config_paths();
        
        for path in &config_paths {
            if path.exists() {
                match Self::from_file(path) {
                    Ok(config) => return Ok(config),
                    Err(e) => {
                        eprintln!("Warning: Failed to load config from {:?}: {}", path, e);
                    }
                }
            }
        }
        
        // Return default config if no config file found
        Ok(Self::default())
    }
    
    /// Load configuration from a specific file or default locations
    pub fn load_with_file(file: Option<&Path>) -> Result<Self> {
        if let Some(path) = file {
            Self::from_file(path)
        } else {
            Self::load()
        }
    }
    
    /// Get default configuration file paths to check
    fn default_config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        // Current directory
        paths.push(PathBuf::from(".specado.yaml"));
        paths.push(PathBuf::from(".specado.json"));
        paths.push(PathBuf::from("specado.yaml"));
        paths.push(PathBuf::from("specado.json"));
        
        // User config directory
        if let Some(config_dir) = dirs::config_dir() {
            let specado_dir = config_dir.join("specado");
            paths.push(specado_dir.join("config.yaml"));
            paths.push(specado_dir.join("config.json"));
        }
        
        // Home directory
        if let Some(home_dir) = dirs::home_dir() {
            paths.push(home_dir.join(".specado.yaml"));
            paths.push(home_dir.join(".specado.json"));
        }
        
        paths
    }
    
    /// Merge with another config (other takes precedence)
    pub fn merge(&mut self, other: Config) {
        if other.default_provider.is_some() {
            self.default_provider = other.default_provider;
        }
        if other.default_model.is_some() {
            self.default_model = other.default_model;
        }
        
        // Merge provider configs
        for (name, config) in other.providers {
            self.providers.insert(name, config);
        }
        
        // Merge output config
        self.output = other.output;
        self.logging = other.logging;
        self.paths = other.paths;
    }
    
    /// Get provider configuration
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.get(name)
    }
    
    /// Save configuration to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = if path.extension().and_then(|s| s.to_str()) == Some("yaml")
            || path.extension().and_then(|s| s.to_str()) == Some("yml") {
            serde_yaml::to_string(self)?
        } else {
            serde_json::to_string_pretty(self)?
        };
        
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Builder for creating configurations programmatically
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    /// Create a new config builder
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }
    
    /// Set default provider
    pub fn default_provider(mut self, provider: impl Into<String>) -> Self {
        self.config.default_provider = Some(provider.into());
        self
    }
    
    /// Set default model
    pub fn default_model(mut self, model: impl Into<String>) -> Self {
        self.config.default_model = Some(model.into());
        self
    }
    
    /// Add a provider configuration
    pub fn add_provider(
        mut self,
        name: impl Into<String>,
        config: ProviderConfig,
    ) -> Self {
        self.config.providers.insert(name.into(), config);
        self
    }
    
    /// Build the configuration
    pub fn build(self) -> Config {
        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}