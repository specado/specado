//! Schema parsing functionality for YAML and JSON formats
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::loader::error::{LoaderError, LoaderResult};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Supported file formats for schema parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// YAML format (.yaml, .yml)
    Yaml,
    /// JSON format (.json)
    Json,
}

impl Format {
    /// Detect format from file extension
    pub fn from_path(path: &Path) -> LoaderResult<Self> {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            match extension.to_lowercase().as_str() {
                "yaml" | "yml" => Ok(Format::Yaml),
                "json" => Ok(Format::Json),
                _ => Err(LoaderError::unsupported_format(path.to_path_buf())),
            }
        } else {
            Err(LoaderError::unsupported_format(path.to_path_buf()))
        }
    }

    /// Get file extensions for this format
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Format::Yaml => &["yaml", "yml"],
            Format::Json => &["json"],
        }
    }

    /// Get the primary file extension for this format
    pub fn primary_extension(&self) -> &'static str {
        match self {
            Format::Yaml => "yaml",
            Format::Json => "json",
        }
    }

    /// Check if this format supports comments
    pub fn supports_comments(&self) -> bool {
        match self {
            Format::Yaml => true,
            Format::Json => false,
        }
    }
}

/// Schema parser with support for multiple formats
#[derive(Debug)]
pub struct SchemaParser;

impl SchemaParser {
    /// Create a new schema parser
    pub fn new() -> Self {
        Self
    }

    /// Parse a schema file, detecting format from extension
    pub fn parse_file(&self, path: &Path) -> LoaderResult<Value> {
        let format = Format::from_path(path)?;
        let content = std::fs::read_to_string(path)
            .map_err(|e| LoaderError::io_error(path.to_path_buf(), e))?;

        self.parse_content(&content, format, path)
    }

    /// Parse schema content with explicit format
    pub fn parse_content(&self, content: &str, format: Format, path: &Path) -> LoaderResult<Value> {
        match format {
            Format::Yaml => self.parse_yaml(content, path),
            Format::Json => self.parse_json(content, path),
        }
    }

    /// Parse YAML content
    pub fn parse_yaml(&self, content: &str, path: &Path) -> LoaderResult<Value> {
        // First parse as YAML Value to catch YAML-specific errors
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(content)
            .map_err(|e| LoaderError::yaml_parse_error(path.to_path_buf(), e))?;

        // Convert to JSON Value for consistent handling
        serde_json::to_value(yaml_value)
            .map_err(|e| LoaderError::json_parse_error(path.to_path_buf(), e))
    }

    /// Parse JSON content
    pub fn parse_json(&self, content: &str, path: &Path) -> LoaderResult<Value> {
        serde_json::from_str(content)
            .map_err(|e| LoaderError::json_parse_error(path.to_path_buf(), e))
    }

    /// Validate basic schema structure
    pub fn validate_basic_structure(&self, value: &Value, path: &Path) -> LoaderResult<()> {
        // Check that we have an object at the root
        if !value.is_object() {
            return Err(LoaderError::validation_error(
                path.to_path_buf(),
                "Schema must be a JSON object at the root level".to_string(),
            ));
        }

        let obj = value.as_object().unwrap();

        // Check for common required fields based on schema type
        self.validate_schema_type(obj, path)?;

        Ok(())
    }

    /// Try to parse content with multiple formats (for auto-detection)
    pub fn parse_with_fallback(&self, content: &str, path: &Path) -> LoaderResult<(Value, Format)> {
        // Try to detect from extension first
        if let Ok(format) = Format::from_path(path) {
            match self.parse_content(content, format, path) {
                Ok(value) => return Ok((value, format)),
                Err(_) => {
                    // Fall through to try other formats
                }
            }
        }

        // Try JSON first (stricter format)
        if let Ok(value) = self.parse_json(content, path) {
            return Ok((value, Format::Json));
        }

        // Try YAML as fallback
        if let Ok(value) = self.parse_yaml(content, path) {
            return Ok((value, Format::Yaml));
        }

        // If all formats fail, return the original error
        Err(LoaderError::unsupported_format(path.to_path_buf()))
    }

    /// Serialize a value back to string format
    pub fn serialize(&self, value: &Value, format: Format) -> LoaderResult<String> {
        match format {
            Format::Json => serde_json::to_string_pretty(value)
                .map_err(|e| LoaderError::CacheError {
                    reason: format!("Failed to serialize JSON: {}", e),
                }),
            Format::Yaml => {
                // Convert from JSON Value to YAML Value for proper serialization
                let yaml_value: serde_yaml::Value = serde_json::from_value(value.clone())
                    .map_err(|e| LoaderError::CacheError {
                        reason: format!("Failed to convert to YAML value: {}", e),
                    })?;
                
                serde_yaml::to_string(&yaml_value)
                    .map_err(|e| LoaderError::CacheError {
                        reason: format!("Failed to serialize YAML: {}", e),
                    })
            }
        }
    }

    fn validate_schema_type(&self, obj: &serde_json::Map<String, Value>, path: &Path) -> LoaderResult<()> {
        // Look for version fields
        let has_spec_version = obj.contains_key("spec_version");
        let has_version = obj.contains_key("version");

        if !has_spec_version && !has_version {
            return Err(LoaderError::validation_error(
                path.to_path_buf(),
                "Schema must contain either 'spec_version' or 'version' field".to_string(),
            ));
        }

        // Validate version format if present
        if let Some(version) = obj.get("spec_version").or_else(|| obj.get("version")) {
            if !version.is_string() {
                return Err(LoaderError::validation_error(
                    path.to_path_buf(),
                    "Version field must be a string".to_string(),
                ));
            }

            let version_str = version.as_str().unwrap();
            if version_str.is_empty() {
                return Err(LoaderError::validation_error(
                    path.to_path_buf(),
                    "Version field cannot be empty".to_string(),
                ));
            }

            // Basic semver-like validation
            if !version_str.chars().next().unwrap().is_ascii_digit() {
                return Err(LoaderError::validation_error(
                    path.to_path_buf(),
                    "Version must start with a digit".to_string(),
                ));
            }
        }

        // Check for schema type indicators
        let is_prompt_spec = obj.contains_key("model_class") || obj.contains_key("messages");
        let is_provider_spec = obj.contains_key("provider_name") || obj.contains_key("base_url");

        if !is_prompt_spec && !is_provider_spec {
            // This might be a fragment or reference file, which is okay
            // Just warn but don't fail
        }

        Ok(())
    }
}

impl Default for SchemaParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for content preprocessing
pub struct ContentPreprocessor;

impl ContentPreprocessor {
    /// Strip comments from content (for formats that support them)
    pub fn strip_comments(content: &str, format: Format) -> String {
        if !format.supports_comments() {
            return content.to_string();
        }

        match format {
            Format::Yaml => Self::strip_yaml_comments(content),
            Format::Json => content.to_string(), // JSON doesn't support comments
        }
    }

    /// Strip YAML comments while preserving structure
    fn strip_yaml_comments(content: &str) -> String {
        content
            .lines()
            .map(|line| {
                if let Some(comment_pos) = line.find('#') {
                    // Check if the # is inside a quoted string
                    let before_comment = &line[..comment_pos];
                    let quote_count = before_comment.matches('"').count();
                    
                    // If even number of quotes, the # is outside quotes (a comment)
                    if quote_count % 2 == 0 {
                        before_comment.trim_end()
                    } else {
                        line
                    }
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Normalize line endings to Unix style
    pub fn normalize_line_endings(content: &str) -> String {
        content.replace("\r\n", "\n").replace('\r', "\n")
    }

    /// Validate content encoding (ensure it's valid UTF-8)
    pub fn validate_encoding(content: &[u8]) -> LoaderResult<String> {
        String::from_utf8(content.to_vec()).map_err(|e| LoaderError::ValidationError {
            path: PathBuf::from("<content>"),
            reason: format!("Invalid UTF-8 encoding: {}", e),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_format_detection() {
        assert_eq!(Format::from_path(Path::new("test.yaml")).unwrap(), Format::Yaml);
        assert_eq!(Format::from_path(Path::new("test.yml")).unwrap(), Format::Yaml);
        assert_eq!(Format::from_path(Path::new("test.json")).unwrap(), Format::Json);
        
        assert!(Format::from_path(Path::new("test.txt")).is_err());
        assert!(Format::from_path(Path::new("test")).is_err());
    }

    #[test]
    fn test_format_properties() {
        assert!(Format::Yaml.supports_comments());
        assert!(!Format::Json.supports_comments());
        
        assert_eq!(Format::Yaml.primary_extension(), "yaml");
        assert_eq!(Format::Json.primary_extension(), "json");
    }

    #[test]
    fn test_yaml_parsing() -> LoaderResult<()> {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.yaml");
        
        let yaml_content = r#"
spec_version: "1.0"
id: test-prompt
model_class: Chat
messages:
  - role: user
    content: "Hello, world!"
"#;
        
        fs::write(&file_path, yaml_content)?;
        
        let parser = SchemaParser::new();
        let result = parser.parse_file(&file_path)?;
        
        assert_eq!(result["spec_version"], "1.0");
        assert_eq!(result["id"], "test-prompt");
        assert_eq!(result["model_class"], "Chat");
        
        Ok(())
    }

    #[test]
    fn test_json_parsing() -> LoaderResult<()> {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.json");
        
        let json_content = r#"{
  "spec_version": "1.0",
  "id": "test-prompt",
  "model_class": "Chat",
  "messages": [
    {
      "role": "user",
      "content": "Hello, world!"
    }
  ]
}"#;
        
        fs::write(&file_path, json_content)?;
        
        let parser = SchemaParser::new();
        let result = parser.parse_file(&file_path)?;
        
        assert_eq!(result["spec_version"], "1.0");
        assert_eq!(result["id"], "test-prompt");
        assert_eq!(result["model_class"], "Chat");
        
        Ok(())
    }

    #[test]
    fn test_basic_validation() -> LoaderResult<()> {
        let parser = SchemaParser::new();
        let path = Path::new("test.yaml");
        
        // Valid schema
        let valid_schema = json!({
            "spec_version": "1.0",
            "id": "test",
            "model_class": "Chat"
        });
        assert!(parser.validate_basic_structure(&valid_schema, path).is_ok());
        
        // Invalid: not an object
        let invalid_schema = json!("not an object");
        assert!(parser.validate_basic_structure(&invalid_schema, path).is_err());
        
        // Invalid: no version
        let no_version = json!({"id": "test"});
        assert!(parser.validate_basic_structure(&no_version, path).is_err());
        
        // Invalid: empty version
        let empty_version = json!({"spec_version": ""});
        assert!(parser.validate_basic_structure(&empty_version, path).is_err());
        
        Ok(())
    }

    #[test]
    fn test_fallback_parsing() -> LoaderResult<()> {
        let parser = SchemaParser::new();
        
        // Valid JSON
        let json_content = r#"{"spec_version": "1.0", "id": "test"}"#;
        let (value, format) = parser.parse_with_fallback(json_content, Path::new("unknown.txt"))?;
        assert_eq!(format, Format::Json);
        assert_eq!(value["id"], "test");
        
        // Valid YAML
        let yaml_content = "spec_version: '1.0'\nid: test";
        let (value, format) = parser.parse_with_fallback(yaml_content, Path::new("unknown.txt"))?;
        assert_eq!(format, Format::Yaml);
        assert_eq!(value["id"], "test");
        
        Ok(())
    }

    #[test]
    fn test_serialization() -> LoaderResult<()> {
        let parser = SchemaParser::new();
        let value = json!({
            "spec_version": "1.0",
            "id": "test",
            "numbers": [1, 2, 3]
        });
        
        // Test JSON serialization
        let json_str = parser.serialize(&value, Format::Json)?;
        assert!(json_str.contains("\"spec_version\": \"1.0\""));
        
        // Test YAML serialization
        let yaml_str = parser.serialize(&value, Format::Yaml)?;
        assert!(yaml_str.contains("spec_version: '1.0'"));
        
        Ok(())
    }

    #[test]
    fn test_comment_stripping() {
        let yaml_with_comments = r#"
# This is a comment
spec_version: "1.0"  # Version comment
id: test
# Another comment
messages: []
"#;
        
        let stripped = ContentPreprocessor::strip_comments(yaml_with_comments, Format::Yaml);
        assert!(!stripped.contains("# This is a comment"));
        assert!(!stripped.contains("# Version comment"));
        assert!(!stripped.contains("# Another comment"));
        assert!(stripped.contains("spec_version: \"1.0\""));
    }

    #[test]
    fn test_line_ending_normalization() {
        let content_with_mixed_endings = "line1\r\nline2\rline3\n";
        let normalized = ContentPreprocessor::normalize_line_endings(content_with_mixed_endings);
        assert_eq!(normalized, "line1\nline2\nline3\n");
    }

    #[test]
    fn test_encoding_validation() {
        // Valid UTF-8
        let valid_utf8 = "Hello, 世界!".as_bytes();
        assert!(ContentPreprocessor::validate_encoding(valid_utf8).is_ok());
        
        // Invalid UTF-8
        let invalid_utf8 = &[0xFF, 0xFE, 0xFD];
        assert!(ContentPreprocessor::validate_encoding(invalid_utf8).is_err());
    }
}