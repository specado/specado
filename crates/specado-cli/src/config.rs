//! Configuration management for the CLI
//!
//! This module handles loading and merging configuration from:
//! - Default values
//! - User configuration (~/.specado/config.toml)
//! - Project configuration (.specado.toml)
//! - Environment variables (SPECADO_* prefix)
//! - Command-line arguments
//!
//! Configuration precedence: CLI args > env vars > project config > user config > defaults

use crate::error::{Error, Result};
use crate::logging::timing::Timer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use tracing::{instrument, info, warn, debug};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct Config {
    /// Default provider to use if not specified
    pub default_provider: Option<String>,
    
    /// Default model to use if not specified
    pub default_model: Option<String>,
    
    /// Default strict mode for operations
    pub strict_mode: bool,
    
    /// Directory containing provider specifications
    pub provider_dir: Option<PathBuf>,
    
    /// Provider configurations
    pub providers: HashMap<String, ProviderConfig>,
    
    /// Output settings
    pub output: OutputConfig,
    
    /// Logging settings
    pub logging: LoggingConfig,
    
    /// Path settings
    pub paths: PathConfig,
    
    /// Configuration profiles (development, production, etc.)
    pub profiles: HashMap<String, ProfileConfig>,
    
    /// Active profile name
    pub active_profile: Option<String>,
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

impl ProviderConfig {
    /// Validate the provider configuration
    pub fn validate(&self) -> Result<()> {
        // Validate timeout is reasonable
        if let Some(timeout) = self.timeout {
            if timeout == 0 {
                return Err(Error::config("Timeout cannot be zero"));
            }
            if timeout > 3600 {
                return Err(Error::config("Timeout cannot exceed 3600 seconds"));
            }
        }
        
        // Validate max_retries is reasonable
        if let Some(retries) = self.max_retries {
            if retries > 100 {
                return Err(Error::config("max_retries cannot exceed 100"));
            }
        }
        
        // Validate base_url if provided
        if let Some(url) = &self.base_url {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(Error::config("base_url must start with http:// or https://"));
            }
        }
        
        Ok(())
    }
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

/// Profile configuration for different environments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct ProfileConfig {
    /// Override default provider for this profile
    pub default_provider: Option<String>,
    
    /// Override default model for this profile
    pub default_model: Option<String>,
    
    /// Override strict mode for this profile
    pub strict_mode: Option<bool>,
    
    /// Profile-specific provider configurations
    pub providers: HashMap<String, ProviderConfig>,
    
    /// Profile-specific output settings
    pub output: Option<OutputConfig>,
    
    /// Profile-specific logging settings
    pub logging: Option<LoggingConfig>,
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


impl OutputConfig {
    /// Merge with another output config (other takes precedence)
    pub fn merge(&mut self, other: OutputConfig) {
        self.format = other.format;
        self.color = other.color;
        self.progress = other.progress;
        self.verbosity = other.verbosity;
    }
}

impl LoggingConfig {
    /// Merge with another logging config (other takes precedence)
    pub fn merge(&mut self, other: LoggingConfig) {
        self.level = other.level;
        self.format = other.format;
        if other.file.is_some() {
            self.file = other.file;
        }
        self.timestamps = other.timestamps;
        self.thread_ids = other.thread_ids;
    }
}

impl PathConfig {
    /// Merge with another path config (other takes precedence)
    pub fn merge(&mut self, other: PathConfig) {
        self.providers_dir = other.providers_dir;
        if other.prompts_dir.is_some() {
            self.prompts_dir = other.prompts_dir;
        }
        if other.schemas_dir.is_some() {
            self.schemas_dir = other.schemas_dir;
        }
        self.cache_dir = other.cache_dir;
    }
}

impl Config {
    /// Load configuration from a file
    #[instrument(fields(path = %path.display()))]
    pub fn from_file(path: &Path) -> Result<Self> {
        let _timer = Timer::with_details("config_file_loading", &format!("path: {}", path.display()));
        debug!("Reading configuration file");
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::config(format!("Failed to read config file {}: {}", path.display(), e)))?;
        
        debug!("Config file read, {} bytes", content.len());
        
        let config = match path.extension().and_then(|s| s.to_str()) {
            Some("toml") => {
                debug!("Parsing config as TOML");
                toml::from_str(&content)
                    .map_err(|e| Error::config(format!("Failed to parse TOML config: {}", e)))?
            }
            Some("yaml") | Some("yml") => {
                debug!("Parsing config as YAML");
                serde_yaml::from_str(&content)
                    .map_err(|e| Error::config(format!("Failed to parse YAML config: {}", e)))?
            }
            Some("json") => {
                debug!("Parsing config as JSON");
                serde_json::from_str(&content)
                    .map_err(|e| Error::config(format!("Failed to parse JSON config: {}", e)))?
            }
            _ => {
                debug!("Auto-detecting config format");
                // Try to auto-detect format
                if let Ok(config) = toml::from_str::<Self>(&content) {
                    debug!("Auto-detected TOML format");
                    config
                } else if let Ok(config) = serde_yaml::from_str::<Self>(&content) {
                    debug!("Auto-detected YAML format");
                    config
                } else {
                    debug!("Attempting JSON format");
                    serde_json::from_str(&content)
                        .map_err(|e| Error::config(format!("Failed to parse config file as TOML, YAML, or JSON: {}", e)))?
                }
            }
        };
        
        info!("Configuration loaded successfully");
        
        Ok(config)
    }
    
    /// Load configuration with full precedence handling
    #[instrument]
    #[allow(dead_code)]
    pub fn load() -> Result<Self> {
        Self::load_with_file(None)
    }
    
    /// Load configuration from a specific file or use precedence chain
    #[instrument(fields(explicit_file = ?explicit_file.map(|p| p.display())))]
    pub fn load_with_file(explicit_file: Option<&Path>) -> Result<Self> {
        let _timer = Timer::new("config_loading");
        info!("Loading configuration with precedence chain");
        let mut config = Self::default();
        
        // Step 1: Start with defaults (already done via default())
        
        // Step 2: Load user config (~/.specado/config.toml)
        if let Some(user_config_path) = Self::user_config_path() {
            if user_config_path.exists() {
                debug!("Loading user config from: {}", user_config_path.display());
                match Self::from_file(&user_config_path) {
                    Ok(user_config) => {
                        info!("User config loaded successfully");
                        config.merge(user_config);
                    }
                    Err(e) => {
                        warn!("Failed to load user config from {}: {}", user_config_path.display(), e);
                        eprintln!("Warning: Failed to load user config from {}: {}", user_config_path.display(), e);
                    }
                }
            } else {
                debug!("No user config found at: {}", user_config_path.display());
            }
        }
        
        // Step 3: Load project config (.specado.toml in current directory or parents)
        if let Some(project_config_path) = Self::find_project_config() {
            match Self::from_file(&project_config_path) {
                Ok(project_config) => {
                    config.merge(project_config);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load project config from {}: {}", project_config_path.display(), e);
                }
            }
        }
        
        // Step 4: Load explicit config file if provided
        if let Some(path) = explicit_file {
            let explicit_config = Self::from_file(path)?;
            config.merge(explicit_config);
        }
        
        // Step 5: Apply environment variables
        config.apply_env_overrides();
        
        // Step 6: Apply active profile if set
        config.apply_active_profile()?;
        
        Ok(config)
    }
    
    /// Get the user config file path (~/.specado/config.toml)
    pub fn user_config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".specado").join("config.toml"))
    }
    
    /// Find project config file (.specado.toml) in current directory or parent directories
    pub fn find_project_config() -> Option<PathBuf> {
        let mut current_dir = env::current_dir().ok()?;
        
        loop {
            let config_path = current_dir.join(".specado.toml");
            if config_path.exists() {
                return Some(config_path);
            }
            
            if !current_dir.pop() {
                break;
            }
        }
        
        None
    }
    
    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) {
        // SPECADO_DEFAULT_PROVIDER
        if let Ok(provider) = env::var("SPECADO_DEFAULT_PROVIDER") {
            self.default_provider = Some(provider);
        }
        
        // SPECADO_DEFAULT_MODEL
        if let Ok(model) = env::var("SPECADO_DEFAULT_MODEL") {
            self.default_model = Some(model);
        }
        
        // SPECADO_STRICT_MODE
        if let Ok(strict) = env::var("SPECADO_STRICT_MODE") {
            self.strict_mode = strict.to_lowercase() == "true" || strict == "1";
        }
        
        // SPECADO_PROVIDER_DIR
        if let Ok(dir) = env::var("SPECADO_PROVIDER_DIR") {
            self.provider_dir = Some(PathBuf::from(dir));
        }
        
        // SPECADO_PROFILE
        if let Ok(profile) = env::var("SPECADO_PROFILE") {
            self.active_profile = Some(profile);
        }
        
        // Output settings
        if let Ok(format) = env::var("SPECADO_OUTPUT_FORMAT") {
            self.output.format = format;
        }
        
        if let Ok(color) = env::var("SPECADO_OUTPUT_COLOR") {
            self.output.color = color.to_lowercase() == "true" || color == "1";
        }
        
        // Logging settings
        if let Ok(level) = env::var("SPECADO_LOG_LEVEL") {
            self.logging.level = level;
        }
        
        if let Ok(format) = env::var("SPECADO_LOG_FORMAT") {
            self.logging.format = format;
        }
    }
    
    /// Apply the active profile configuration
    fn apply_active_profile(&mut self) -> Result<()> {
        if let Some(profile_name) = &self.active_profile.clone() {
            if let Some(profile) = self.profiles.get(profile_name) {
                // Apply profile overrides
                if let Some(provider) = &profile.default_provider {
                    self.default_provider = Some(provider.clone());
                }
                
                if let Some(model) = &profile.default_model {
                    self.default_model = Some(model.clone());
                }
                
                if let Some(strict) = profile.strict_mode {
                    self.strict_mode = strict;
                }
                
                // Merge profile providers
                for (name, config) in &profile.providers {
                    self.providers.insert(name.clone(), config.clone());
                }
                
                // Apply profile output settings
                if let Some(output) = &profile.output {
                    self.output = output.clone();
                }
                
                // Apply profile logging settings
                if let Some(logging) = &profile.logging {
                    self.logging = logging.clone();
                }
            } else {
                return Err(Error::config(format!("Profile '{}' not found", profile_name)));
            }
        }
        
        Ok(())
    }
    
    /// Merge with another config (other takes precedence)
    pub fn merge(&mut self, other: Config) {
        if other.default_provider.is_some() {
            self.default_provider = other.default_provider;
        }
        if other.default_model.is_some() {
            self.default_model = other.default_model;
        }
        
        // Only override if explicitly set in other config
        if other.strict_mode {
            self.strict_mode = other.strict_mode;
        }
        
        if other.provider_dir.is_some() {
            self.provider_dir = other.provider_dir;
        }
        
        if other.active_profile.is_some() {
            self.active_profile = other.active_profile;
        }
        
        // Merge provider configs
        for (name, config) in other.providers {
            self.providers.insert(name, config);
        }
        
        // Merge profiles
        for (name, profile) in other.profiles {
            self.profiles.insert(name, profile);
        }
        
        // Always merge structured configs
        self.output.merge(other.output);
        self.logging.merge(other.logging);
        self.paths.merge(other.paths);
    }
    
    /// Get provider configuration, considering active profile
    #[allow(dead_code)]
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.get(name)
    }
    
    /// Save configuration to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = match path.extension().and_then(|s| s.to_str()) {
            Some("toml") => {
                toml::to_string_pretty(self)
                    .map_err(|e| Error::config(format!("Failed to serialize config as TOML: {}", e)))?
            }
            Some("yaml") | Some("yml") => {
                serde_yaml::to_string(self)
                    .map_err(|e| Error::config(format!("Failed to serialize config as YAML: {}", e)))?
            }
            Some("json") => {
                serde_json::to_string_pretty(self)
                    .map_err(|e| Error::config(format!("Failed to serialize config as JSON: {}", e)))?
            }
            _ => {
                // Default to TOML for unknown extensions
                toml::to_string_pretty(self)
                    .map_err(|e| Error::config(format!("Failed to serialize config as TOML: {}", e)))?
            }
        };
        
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::config(format!("Failed to create config directory: {}", e)))?;
        }
        
        std::fs::write(path, content)
            .map_err(|e| Error::config(format!("Failed to write config file: {}", e)))?;
        Ok(())
    }
    
    /// Create a default user config file
    pub fn create_default_user_config() -> Result<()> {
        let config_path = Self::user_config_path()
            .ok_or_else(|| Error::config("Unable to determine user config directory"))?;
        
        let default_config = Self::default();
        default_config.save(&config_path)?;
        
        Ok(())
    }
    
    /// Create a default project config file
    pub fn create_default_project_config() -> Result<()> {
        let config_path = PathBuf::from(".specado.toml");
        let default_config = Self::default();
        default_config.save(&config_path)?;
        
        Ok(())
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate that active profile exists
        if let Some(profile_name) = &self.active_profile {
            if !self.profiles.contains_key(profile_name) {
                return Err(Error::config(format!("Active profile '{}' does not exist", profile_name)));
            }
        }
        
        // Validate provider configurations
        for (name, provider_config) in &self.providers {
            provider_config.validate()
                .map_err(|e| Error::config(format!("Invalid provider config for '{}': {}", name, e)))?;
        }
        
        // Validate paths exist if specified
        if let Some(provider_dir) = &self.provider_dir {
            if !provider_dir.exists() {
                return Err(Error::config(format!("Provider directory does not exist: {}", provider_dir.display())));
            }
        }
        
        Ok(())
    }
}

/// Builder for creating configurations programmatically
#[allow(dead_code)]
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    /// Create a new config builder
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }
    
    /// Set default provider
    #[allow(dead_code)]
    pub fn default_provider(mut self, provider: impl Into<String>) -> Self {
        self.config.default_provider = Some(provider.into());
        self
    }
    
    /// Set default model
    #[allow(dead_code)]
    pub fn default_model(mut self, model: impl Into<String>) -> Self {
        self.config.default_model = Some(model.into());
        self
    }
    
    /// Add a provider configuration
    #[allow(dead_code)]
    pub fn add_provider(
        mut self,
        name: impl Into<String>,
        config: ProviderConfig,
    ) -> Self {
        self.config.providers.insert(name.into(), config);
        self
    }
    
    /// Build the configuration
    #[allow(dead_code)]
    pub fn build(self) -> Config {
        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}