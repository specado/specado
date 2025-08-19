//! Main schema loader with caching and comprehensive functionality
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::loader::{
    cache::{CacheConfig, SchemaCache},
    error::{LoaderError, LoaderResult},
    parser::SchemaParser,
    resolver::{ReferenceResolver, ResolverContext},
};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Configuration for schema loader behavior
#[derive(Debug, Clone)]
pub struct LoaderConfig {
    /// Cache configuration
    pub cache: CacheConfig,
    /// Maximum reference resolution depth
    pub max_resolution_depth: usize,
    /// Whether to allow environment variable expansion
    pub allow_env_expansion: bool,
    /// Whether to perform basic validation during loading
    pub validate_basic_structure: bool,
    /// Whether to resolve references automatically
    pub auto_resolve_refs: bool,
    /// Base directory for relative path resolution
    pub base_dir: Option<PathBuf>,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            cache: CacheConfig::default(),
            max_resolution_depth: 10,
            allow_env_expansion: true,
            validate_basic_structure: true,
            auto_resolve_refs: true,
            base_dir: None,
        }
    }
}

/// Main schema loader with caching and comprehensive functionality
#[derive(Debug)]
pub struct SchemaLoader {
    config: LoaderConfig,
    cache: SchemaCache,
    parser: Arc<SchemaParser>,
    resolver: ReferenceResolver,
}

impl SchemaLoader {
    /// Create a new schema loader with default configuration
    pub fn new() -> Self {
        Self::with_config(LoaderConfig::default())
    }

    /// Create a new schema loader with custom configuration
    pub fn with_config(config: LoaderConfig) -> Self {
        let parser = Arc::new(SchemaParser::new());
        Self {
            cache: SchemaCache::with_config(config.cache.clone()),
            resolver: ReferenceResolver::with_parser(Arc::clone(&parser)),
            parser,
            config,
        }
    }

    /// Load a PromptSpec schema from file
    pub fn load_prompt_spec(&mut self, path: &Path) -> LoaderResult<Value> {
        let schema = self.load_schema(path)?;
        self.validate_prompt_spec(&schema, path)?;
        Ok(schema)
    }

    /// Load a ProviderSpec schema from file
    pub fn load_provider_spec(&mut self, path: &Path) -> LoaderResult<Value> {
        let schema = self.load_schema(path)?;
        self.validate_provider_spec(&schema, path)?;
        Ok(schema)
    }

    /// Load a generic schema from file
    pub fn load_schema(&mut self, path: &Path) -> LoaderResult<Value> {
        // Try to get from cache first
        if let Some(cached_entry) = self.cache.get(path)? {
            return Ok(cached_entry.content);
        }

        // Parse the file
        let mut schema = self.parser.parse_file(path)?;

        // Perform basic validation if enabled
        if self.config.validate_basic_structure {
            self.parser.validate_basic_structure(&schema, path)?;
        }

        // Resolve references if enabled
        if self.config.auto_resolve_refs {
            let base_dir = self.get_base_dir(path)?;
            let mut context = ResolverContext::new(base_dir);
            context.max_depth = self.config.max_resolution_depth;
            context.allow_env_expansion = self.config.allow_env_expansion;
            
            // Add the current file to the resolution stack for same-file references
            if let Ok(canonical_path) = path.canonicalize() {
                context.push_path(canonical_path)?;
            }

            schema = self.resolver.resolve(schema, &mut context)?;
            
            // Remove from stack after resolution
            context.pop_path();
        }

        // Cache the result
        self.cache.put(path, schema.clone())?;

        Ok(schema)
    }

    /// Load multiple schemas in batch
    pub fn load_schemas_batch(&mut self, paths: &[&Path]) -> LoaderResult<Vec<(PathBuf, Value)>> {
        let mut results = Vec::new();
        let mut errors = Vec::new();

        for path in paths {
            match self.load_schema(path) {
                Ok(schema) => results.push((path.to_path_buf(), schema)),
                Err(e) => errors.push((path.to_path_buf(), e)),
            }
        }

        // If there were any errors, return the first one
        // In a production system, you might want to collect all errors
        if let Some((_path, error)) = errors.into_iter().next() {
            return Err(error);
        }

        Ok(results)
    }

    /// Reload a schema, bypassing cache
    pub fn reload_schema(&mut self, path: &Path) -> LoaderResult<Value> {
        // Remove from cache first
        self.cache.remove(path)?;
        
        // Clear resolver cache as well
        self.resolver.clear_cache();
        
        // Load fresh
        self.load_schema(path)
    }

    /// Check if a schema is cached
    pub fn is_cached(&self, path: &Path) -> LoaderResult<bool> {
        self.cache.contains(path)
    }

    /// Clear all caches
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.resolver.clear_cache();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> crate::loader::cache::CacheStats {
        self.cache.stats()
    }

    /// Cleanup expired cache entries
    pub fn cleanup_cache(&mut self) -> LoaderResult<usize> {
        self.cache.cleanup_expired()
    }

    /// Validate schema version compatibility
    pub fn validate_version(&self, schema: &Value, path: &Path) -> LoaderResult<String> {
        let version = schema
            .get("spec_version")
            .or_else(|| schema.get("version"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                LoaderError::version_error(
                    "unknown".to_string(),
                    path.to_path_buf(),
                    "No version field found".to_string(),
                )
            })?;

        // Validate version format (basic semver check)
        if !self.is_valid_version_format(version) {
            return Err(LoaderError::version_error(
                version.to_string(),
                path.to_path_buf(),
                "Invalid version format".to_string(),
            ));
        }

        // Check compatibility (this could be more sophisticated)
        if !self.is_compatible_version(version) {
            return Err(LoaderError::version_error(
                version.to_string(),
                path.to_path_buf(),
                format!("Unsupported version: {}", version),
            ));
        }

        Ok(version.to_string())
    }

    /// Set configuration
    pub fn set_config(&mut self, config: LoaderConfig) {
        self.config = config;
        self.cache = SchemaCache::with_config(self.config.cache.clone());
    }

    /// Get current configuration
    pub fn config(&self) -> &LoaderConfig {
        &self.config
    }

    /// Preload schemas from a directory
    pub fn preload_directory(&mut self, dir: &Path, recursive: bool) -> LoaderResult<usize> {
        let mut loaded_count = 0;
        let entries = std::fs::read_dir(dir)
            .map_err(|e| LoaderError::io_error(dir.to_path_buf(), e))?;

        for entry in entries {
            let entry = entry.map_err(|e| LoaderError::io_error(dir.to_path_buf(), e))?;
            let path = entry.path();

            if path.is_file() {
                // Check if it's a schema file
                if crate::loader::parser::Format::from_path(&path).is_ok() {
                    match self.load_schema(&path) {
                        Ok(_) => loaded_count += 1,
                        Err(_) => {
                            // Log error but continue loading other files
                            // In a real implementation, you might want to collect errors
                        }
                    }
                }
            } else if path.is_dir() && recursive {
                loaded_count += self.preload_directory(&path, recursive)?;
            }
        }

        Ok(loaded_count)
    }

    /// Get schema metadata without fully loading
    pub fn get_schema_metadata(&mut self, path: &Path) -> LoaderResult<SchemaMetadata> {
        // Try to get basic info from cache first
        if let Some(cached_entry) = self.cache.get(path)? {
            return Ok(SchemaMetadata::from_cached_entry(&cached_entry));
        }

        // Read just the beginning of the file to extract metadata
        let content = std::fs::read_to_string(path)
            .map_err(|e| LoaderError::io_error(path.to_path_buf(), e))?;

        let format = crate::loader::parser::Format::from_path(path)?;
        let schema = self.parser.parse_content(&content, format, path)?;

        Ok(SchemaMetadata::from_schema(&schema, path, format))
    }

    fn validate_prompt_spec(&self, schema: &Value, path: &Path) -> LoaderResult<()> {
        // Check for PromptSpec-specific fields
        let required_fields = ["spec_version", "model_class"];
        for field in &required_fields {
            if schema.get(field).is_none() {
                return Err(LoaderError::validation_error(
                    path.to_path_buf(),
                    format!("PromptSpec requires '{}' field", field),
                ));
            }
        }

        // Validate model_class
        if let Some(model_class) = schema.get("model_class").and_then(|v| v.as_str()) {
            if !["Chat", "Completion", "ReasoningChat", "RAGChat"].contains(&model_class) {
                return Err(LoaderError::validation_error(
                    path.to_path_buf(),
                    format!("Invalid model_class: {}", model_class),
                ));
            }
        }

        Ok(())
    }

    fn validate_provider_spec(&self, schema: &Value, path: &Path) -> LoaderResult<()> {
        // Check for ProviderSpec-specific fields
        let required_fields = ["spec_version", "provider_name"];
        for field in &required_fields {
            if schema.get(field).is_none() {
                return Err(LoaderError::validation_error(
                    path.to_path_buf(),
                    format!("ProviderSpec requires '{}' field", field),
                ));
            }
        }

        // Validate provider_name is not empty
        if let Some(provider_name) = schema.get("provider_name").and_then(|v| v.as_str()) {
            if provider_name.is_empty() {
                return Err(LoaderError::validation_error(
                    path.to_path_buf(),
                    "provider_name cannot be empty".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn get_base_dir(&self, path: &Path) -> LoaderResult<PathBuf> {
        if let Some(ref base_dir) = self.config.base_dir {
            Ok(base_dir.clone())
        } else {
            path.parent()
                .map(|p| p.to_path_buf())
                .ok_or_else(|| {
                    LoaderError::validation_error(
                        path.to_path_buf(),
                        "Cannot determine base directory".to_string(),
                    )
                })
        }
    }

    fn is_valid_version_format(&self, version: &str) -> bool {
        // Basic semver validation
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() < 2 || parts.len() > 3 {
            return false;
        }

        for part in parts {
            if part.parse::<u32>().is_err() {
                return false;
            }
        }

        true
    }

    fn is_compatible_version(&self, version: &str) -> bool {
        // For now, accept all valid versions
        // In a real implementation, you'd check against supported versions
        let parts: Vec<&str> = version.split('.').collect();
        if let Ok(major) = parts[0].parse::<u32>() {
            // Support versions 1.x and 2.x
            (1..=2).contains(&major)
        } else {
            false
        }
    }
}

impl Default for SchemaLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata about a schema file
#[derive(Debug, Clone)]
pub struct SchemaMetadata {
    pub path: PathBuf,
    pub format: crate::loader::parser::Format,
    pub version: Option<String>,
    pub schema_type: SchemaType,
    pub size_bytes: Option<u64>,
    pub last_modified: Option<std::time::SystemTime>,
}

impl SchemaMetadata {
    fn from_schema(schema: &Value, path: &Path, format: crate::loader::parser::Format) -> Self {
        let version = schema
            .get("spec_version")
            .or_else(|| schema.get("version"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let schema_type = if schema.get("model_class").is_some() {
            SchemaType::PromptSpec
        } else if schema.get("provider_name").is_some() {
            SchemaType::ProviderSpec
        } else {
            SchemaType::Unknown
        };

        let metadata = std::fs::metadata(path).ok();
        let size_bytes = metadata.as_ref().map(|m| m.len());
        let last_modified = metadata.and_then(|m| m.modified().ok());

        Self {
            path: path.to_path_buf(),
            format,
            version,
            schema_type,
            size_bytes,
            last_modified,
        }
    }

    fn from_cached_entry(entry: &crate::loader::cache::CacheEntry) -> Self {
        let schema_type = if entry.content.get("model_class").is_some() {
            SchemaType::PromptSpec
        } else if entry.content.get("provider_name").is_some() {
            SchemaType::ProviderSpec
        } else {
            SchemaType::Unknown
        };

        Self {
            path: entry.file_path.clone(),
            format: crate::loader::parser::Format::from_path(&entry.file_path)
                .unwrap_or(crate::loader::parser::Format::Yaml),
            version: entry.version.clone(),
            schema_type,
            size_bytes: None,
            last_modified: Some(entry.file_mtime),
        }
    }
}

/// Type of schema detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaType {
    PromptSpec,
    ProviderSpec,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_schema_loader_creation() {
        let loader = SchemaLoader::new();
        assert!(loader.config.validate_basic_structure);
        assert!(loader.config.auto_resolve_refs);

        let custom_config = LoaderConfig {
            validate_basic_structure: false,
            ..Default::default()
        };
        let custom_loader = SchemaLoader::with_config(custom_config);
        assert!(!custom_loader.config.validate_basic_structure);
    }

    #[test]
    fn test_prompt_spec_loading() -> LoaderResult<()> {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("prompt.yaml");

        let prompt_spec = r#"
spec_version: "1.0"
id: "test-prompt"
model_class: "Chat"
messages:
  - role: "user"
    content: "Hello, world!"
"#;

        fs::write(&file_path, prompt_spec)?;

        let mut loader = SchemaLoader::new();
        let loaded_spec = loader.load_prompt_spec(&file_path)?;

        assert_eq!(loaded_spec["spec_version"], "1.0");
        assert_eq!(loaded_spec["id"], "test-prompt");
        assert_eq!(loaded_spec["model_class"], "Chat");

        Ok(())
    }

    #[test]
    fn test_provider_spec_loading() -> LoaderResult<()> {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("provider.yaml");

        let provider_spec = r#"
spec_version: "1.0"
provider_name: "openai"
base_url: "https://api.openai.com/v1"
"#;

        fs::write(&file_path, provider_spec)?;

        let mut loader = SchemaLoader::new();
        let loaded_spec = loader.load_provider_spec(&file_path)?;

        assert_eq!(loaded_spec["spec_version"], "1.0");
        assert_eq!(loaded_spec["provider_name"], "openai");

        Ok(())
    }

    #[test]
    fn test_caching_behavior() -> LoaderResult<()> {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("cached.yaml");

        let schema_content = r#"
spec_version: "1.0"
id: "cached-test"
model_class: "Chat"
"#;

        fs::write(&file_path, schema_content)?;

        let mut loader = SchemaLoader::new();

        // First load should parse and cache
        assert!(!loader.is_cached(&file_path)?);
        let schema1 = loader.load_schema(&file_path)?;
        assert!(loader.is_cached(&file_path)?);

        // Second load should come from cache
        let schema2 = loader.load_schema(&file_path)?;
        assert_eq!(schema1, schema2);

        // Reload should bypass cache
        let schema3 = loader.reload_schema(&file_path)?;
        assert_eq!(schema1, schema3);

        Ok(())
    }

    #[test]
    fn test_batch_loading() -> LoaderResult<()> {
        let dir = tempdir().unwrap();

        // Create multiple schema files
        let files = ["schema1.yaml", "schema2.json"];
        for (i, filename) in files.iter().enumerate() {
            let file_path = dir.path().join(filename);
            let content = if filename.ends_with(".yaml") {
                format!(
                    r#"
spec_version: "1.0"
id: "schema-{}"
model_class: "Chat"
"#,
                    i + 1
                )
            } else {
                format!(
                    r#"{{
  "spec_version": "1.0",
  "id": "schema-{}",
  "model_class": "Chat"
}}"#,
                    i + 1
                )
            };
            fs::write(&file_path, content)?;
        }

        let mut loader = SchemaLoader::new();
        let file_paths: Vec<PathBuf> = files
            .iter()
            .map(|f| dir.path().join(f))
            .collect();
        let paths: Vec<&Path> = file_paths
            .iter()
            .map(|p| p.as_path())
            .collect();

        let results = loader.load_schemas_batch(&paths)?;
        assert_eq!(results.len(), 2);

        for (i, (_path, schema)) in results.iter().enumerate() {
            assert_eq!(schema["id"], format!("schema-{}", i + 1));
        }

        Ok(())
    }

    #[test]
    fn test_version_validation() -> LoaderResult<()> {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("versioned.yaml");

        let schema = json!({
            "spec_version": "1.2",
            "id": "test"
        });

        fs::write(&file_path, serde_yaml::to_string(&schema).unwrap())?;

        let loader = SchemaLoader::new();
        let loaded_schema = loader.parser.parse_file(&file_path)?;
        let version = loader.validate_version(&loaded_schema, &file_path)?;

        assert_eq!(version, "1.2");

        // Test invalid version
        let invalid_schema = json!({
            "spec_version": "invalid.version.format.too.long",
            "id": "test"
        });

        fs::write(&file_path, serde_yaml::to_string(&invalid_schema).unwrap())?;
        let invalid_loaded = loader.parser.parse_file(&file_path)?;
        assert!(loader.validate_version(&invalid_loaded, &file_path).is_err());

        Ok(())
    }

    #[test]
    fn test_schema_metadata() -> LoaderResult<()> {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("metadata_test.yaml");

        let schema_content = r#"
spec_version: "1.0"
id: "metadata-test"
model_class: "Chat"
"#;

        fs::write(&file_path, schema_content)?;

        let mut loader = SchemaLoader::new();
        let metadata = loader.get_schema_metadata(&file_path)?;

        assert_eq!(metadata.path, file_path);
        assert_eq!(metadata.format, crate::loader::parser::Format::Yaml);
        assert_eq!(metadata.version, Some("1.0".to_string()));
        assert_eq!(metadata.schema_type, SchemaType::PromptSpec);
        assert!(metadata.size_bytes.is_some());
        assert!(metadata.last_modified.is_some());

        Ok(())
    }

    #[test]
    fn test_validation_errors() {
        let dir = tempdir().unwrap();

        // Test invalid PromptSpec
        let invalid_prompt = dir.path().join("invalid_prompt.yaml");
        fs::write(&invalid_prompt, "spec_version: '1.0'\nid: 'test'").unwrap();

        let mut loader = SchemaLoader::new();
        assert!(loader.load_prompt_spec(&invalid_prompt).is_err());

        // Test invalid ProviderSpec
        let invalid_provider = dir.path().join("invalid_provider.yaml");
        fs::write(&invalid_provider, "spec_version: '1.0'\nid: 'test'").unwrap();

        assert!(loader.load_provider_spec(&invalid_provider).is_err());
    }

    #[test]
    fn test_directory_preloading() -> LoaderResult<()> {
        let dir = tempdir().unwrap();

        // Create subdirectory with schemas
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir)?;

        // Create schema files
        for i in 1..=3 {
            let file_path = dir.path().join(format!("schema{}.yaml", i));
            let content = format!(
                r#"
spec_version: "1.0"
id: "schema-{}"
model_class: "Chat"
"#,
                i
            );
            fs::write(&file_path, &content)?;

            // Also create one in subdirectory
            let sub_file_path = subdir.join(format!("sub_schema{}.yaml", i));
            fs::write(&sub_file_path, &content)?;
        }

        let mut loader = SchemaLoader::new();

        // Test non-recursive preloading
        let loaded_count = loader.preload_directory(dir.path(), false)?;
        assert_eq!(loaded_count, 3); // Only files in root directory

        // Clear cache and test recursive preloading
        loader.clear_cache();
        let loaded_count_recursive = loader.preload_directory(dir.path(), true)?;
        assert_eq!(loaded_count_recursive, 6); // Files in root + subdirectory

        Ok(())
    }
}