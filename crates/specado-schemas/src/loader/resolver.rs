//! Reference resolution and environment variable expansion
//!
//! This module handles:
//! - JSON Schema $ref resolution
//! - Environment variable expansion using ${ENV:VAR} syntax
//! - Circular reference detection
//! - Path traversal security
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::loader::error::{LoaderError, LoaderResult};
use crate::loader::parser::SchemaParser;
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Context for resolution operations
#[derive(Debug, Clone)]
pub struct ResolverContext {
    /// Base directory for relative path resolution
    pub base_dir: PathBuf,
    /// Stack for circular reference detection
    pub resolution_stack: Vec<PathBuf>,
    /// Maximum resolution depth to prevent infinite recursion
    pub max_depth: usize,
    /// Whether to allow environment variable expansion
    pub allow_env_expansion: bool,
    /// Custom environment variables (for testing)
    pub custom_env: HashMap<String, String>,
}

impl ResolverContext {
    /// Create a new resolver context
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            resolution_stack: Vec::new(),
            max_depth: 10,
            allow_env_expansion: true,
            custom_env: HashMap::new(),
        }
    }

    /// Create a context for testing with custom environment
    pub fn for_testing(base_dir: PathBuf, env_vars: HashMap<String, String>) -> Self {
        Self {
            base_dir,
            resolution_stack: Vec::new(),
            max_depth: 10,
            allow_env_expansion: true,
            custom_env: env_vars,
        }
    }

    /// Push a path onto the resolution stack
    pub fn push_path(&mut self, path: PathBuf) -> LoaderResult<()> {
        if self.resolution_stack.len() >= self.max_depth {
            return Err(LoaderError::circular_reference(self.resolution_stack.clone()));
        }

        if self.resolution_stack.contains(&path) {
            let mut chain = self.resolution_stack.clone();
            chain.push(path);
            return Err(LoaderError::circular_reference(chain));
        }

        self.resolution_stack.push(path);
        Ok(())
    }

    /// Pop a path from the resolution stack
    pub fn pop_path(&mut self) -> Option<PathBuf> {
        self.resolution_stack.pop()
    }

    /// Get environment variable value
    pub fn get_env_var(&self, name: &str) -> Option<String> {
        self.custom_env
            .get(name)
            .cloned()
            .or_else(|| env::var(name).ok())
    }

    /// Check if a path is safe (no path traversal)
    pub fn is_safe_path(&self, path: &Path) -> bool {
        // Convert to absolute path and check if it's within base directory
        let full_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.base_dir.join(path)
        };
        
        if let Ok(canonical_path) = full_path.canonicalize() {
            if let Ok(canonical_base) = self.base_dir.canonicalize() {
                canonical_path.starts_with(&canonical_base)
            } else {
                false
            }
        } else {
            // If the file doesn't exist yet, check the parent directory
            if let Some(parent) = full_path.parent() {
                if let Ok(canonical_parent) = parent.canonicalize() {
                    if let Ok(canonical_base) = self.base_dir.canonicalize() {
                        canonical_parent.starts_with(&canonical_base)
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        }
    }
}

/// Reference resolver for JSON Schema $ref and environment variables
#[derive(Debug)]
pub struct ReferenceResolver {
    parser: Arc<SchemaParser>,
    cache: HashMap<PathBuf, Value>,
    sensitive_patterns: Vec<String>,
}

impl ReferenceResolver {
    /// Create a new reference resolver
    pub fn new() -> Self {
        Self {
            parser: Arc::new(SchemaParser::new()),
            cache: HashMap::new(),
            sensitive_patterns: Self::default_sensitive_patterns(),
        }
    }

    /// Create a resolver with a shared parser
    pub fn with_parser(parser: Arc<SchemaParser>) -> Self {
        Self {
            parser,
            cache: HashMap::new(),
            sensitive_patterns: Self::default_sensitive_patterns(),
        }
    }
    
    /// Get default sensitive patterns for redaction
    fn default_sensitive_patterns() -> Vec<String> {
        vec![
            "API_KEY".to_string(),
            "SECRET".to_string(),
            "PASSWORD".to_string(),
            "TOKEN".to_string(),
            "CREDENTIAL".to_string(),
            "PRIVATE_KEY".to_string(),
            "ACCESS_KEY".to_string(),
        ]
    }
    
    /// Check if a variable name contains sensitive patterns
    pub fn is_sensitive(&self, var_name: &str) -> bool {
        let var_upper = var_name.to_uppercase();
        self.sensitive_patterns.iter().any(|pattern| var_upper.contains(pattern))
    }
    
    /// Redact a sensitive value for logging
    pub fn redact_value(&self, var_name: &str, value: &str) -> String {
        if self.is_sensitive(var_name) {
            // Show first 3 chars if long enough, otherwise fully redact
            if value.len() > 8 {
                format!("{}***", &value[..3])
            } else {
                "[REDACTED]".to_string()
            }
        } else {
            value.to_string()
        }
    }

    /// Resolve all references and expand environment variables in a schema
    pub fn resolve(&mut self, mut value: Value, context: &mut ResolverContext) -> LoaderResult<Value> {
        // First expand environment variables
        if context.allow_env_expansion {
            value = self.expand_env_vars(value, context)?;
        }

        // Then resolve $ref references
        self.resolve_refs(value, context)
    }

    /// Resolve $ref references recursively
    fn resolve_refs(&mut self, value: Value, context: &mut ResolverContext) -> LoaderResult<Value> {
        match value {
            Value::Object(mut obj) => {
                // Check for $ref property
                if let Some(ref_value) = obj.get("$ref") {
                    if let Some(ref_str) = ref_value.as_str() {
                        return self.resolve_reference(ref_str, context);
                    }
                }

                // Recursively process all object properties
                let keys: Vec<String> = obj.keys().cloned().collect();
                for key in keys {
                    if let Some(val) = obj.get(&key).cloned() {
                        let resolved = self.resolve_refs(val, context)?;
                        obj.insert(key, resolved);
                    }
                }

                Ok(Value::Object(obj))
            }
            Value::Array(arr) => {
                let mut resolved_arr = Vec::new();
                for item in arr {
                    resolved_arr.push(self.resolve_refs(item, context)?);
                }
                Ok(Value::Array(resolved_arr))
            }
            _ => Ok(value),
        }
    }

    /// Resolve a single $ref reference
    fn resolve_reference(&mut self, reference: &str, context: &mut ResolverContext) -> LoaderResult<Value> {
        // Parse the reference
        let (file_path, json_pointer) = self.parse_reference(reference, context)?;

        // Check if this is a same-file reference (just a JSON pointer)
        let is_same_file = reference.starts_with('#');
        
        if is_same_file {
            // For same-file references, use the current file from the stack
            if let Some(current_file) = context.resolution_stack.last() {
                // Load the current file content
                let file_content = self.load_referenced_file(current_file)?;
                
                // Apply the JSON pointer to get the referenced part
                if json_pointer.is_empty() {
                    Ok(file_content)
                } else {
                    self.apply_json_pointer(&file_content, &json_pointer, reference, current_file)
                }
            } else {
                Err(LoaderError::reference_error(
                    reference.to_string(),
                    context.base_dir.clone(),
                    "Cannot resolve same-file reference without current file context".to_string(),
                ))
            }
        } else {
            // For external file references, proceed as before
            
            // Security check: ensure path is safe
            if !context.is_safe_path(&file_path) {
                return Err(LoaderError::path_traversal(
                    reference.to_string(),
                    context.base_dir.clone(),
                ));
            }

            // Resolve to absolute path
            let absolute_path = context.base_dir.join(&file_path);
            let canonical_path = absolute_path
                .canonicalize()
                .map_err(|e| LoaderError::io_error(absolute_path.clone(), e))?;

            // Check for circular references
            context.push_path(canonical_path.clone())?;

            // Load the referenced file (with caching)
            let file_content = self.load_referenced_file(&canonical_path)?;

            // Resolve references in the loaded content recursively
            let resolved_content = self.resolve_refs(file_content, context)?;

            // Pop from resolution stack
            context.pop_path();

            // Apply JSON pointer if specified
            if json_pointer.is_empty() {
                Ok(resolved_content)
            } else {
                self.apply_json_pointer(&resolved_content, &json_pointer, reference, &canonical_path)
            }
        }
    }

    /// Parse a reference string into file path and JSON pointer
    fn parse_reference(&self, reference: &str, context: &ResolverContext) -> LoaderResult<(PathBuf, String)> {
        if let Some(hash_pos) = reference.find('#') {
            let file_part = &reference[..hash_pos];
            let pointer_part = &reference[hash_pos + 1..];

            let file_path = if file_part.is_empty() {
                // Same-file reference - use the current file being processed
                if let Some(current_file) = context.resolution_stack.last() {
                    current_file.clone()
                } else {
                    // If no file in resolution stack, this is an error
                    return Err(LoaderError::reference_error(
                        reference.to_string(),
                        context.base_dir.clone(),
                        "Cannot resolve same-file reference without current file context".to_string(),
                    ));
                }
            } else {
                PathBuf::from(file_part)
            };

            Ok((file_path, pointer_part.to_string()))
        } else {
            // File reference without pointer
            Ok((PathBuf::from(reference), String::new()))
        }
    }

    /// Load a referenced file with caching
    fn load_referenced_file(&mut self, path: &Path) -> LoaderResult<Value> {
        if let Some(cached_content) = self.cache.get(path) {
            return Ok(cached_content.clone());
        }

        let content = self.parser.parse_file(path)?;
        self.cache.insert(path.to_path_buf(), content.clone());
        Ok(content)
    }

    /// Apply a JSON pointer to extract a specific part of the document
    fn apply_json_pointer(
        &self,
        document: &Value,
        pointer: &str,
        reference: &str,
        source_path: &Path,
    ) -> LoaderResult<Value> {
        if pointer.is_empty() {
            return Ok(document.clone());
        }

        let mut current = document;
        let segments = pointer.split('/').skip(1); // Skip the first empty segment

        for segment in segments {
            // Decode JSON pointer segment
            let decoded_segment = segment.replace("~1", "/").replace("~0", "~");

            match current {
                Value::Object(obj) => {
                    if let Some(value) = obj.get(&decoded_segment) {
                        current = value;
                    } else {
                        return Err(LoaderError::reference_error(
                            reference.to_string(),
                            source_path.to_path_buf(),
                            format!("Property '{}' not found", decoded_segment),
                        ));
                    }
                }
                Value::Array(arr) => {
                    if let Ok(index) = decoded_segment.parse::<usize>() {
                        if let Some(value) = arr.get(index) {
                            current = value;
                        } else {
                            return Err(LoaderError::reference_error(
                                reference.to_string(),
                                source_path.to_path_buf(),
                                format!("Array index {} out of bounds", index),
                            ));
                        }
                    } else {
                        return Err(LoaderError::reference_error(
                            reference.to_string(),
                            source_path.to_path_buf(),
                            format!("Invalid array index '{}'", decoded_segment),
                        ));
                    }
                }
                _ => {
                    return Err(LoaderError::reference_error(
                        reference.to_string(),
                        source_path.to_path_buf(),
                        format!("Cannot access property '{}' on non-object/array", decoded_segment),
                    ));
                }
            }
        }

        Ok(current.clone())
    }

    /// Expand environment variables in the format ${ENV:VAR_NAME}
    fn expand_env_vars(&self, value: Value, context: &ResolverContext) -> LoaderResult<Value> {
        match value {
            Value::String(s) => Ok(Value::String(self.expand_string_env_vars(&s, context)?)),
            Value::Object(obj) => {
                let mut expanded_obj = Map::new();
                for (key, val) in obj {
                    let expanded_key = self.expand_string_env_vars(&key, context)?;
                    let expanded_val = self.expand_env_vars(val, context)?;
                    expanded_obj.insert(expanded_key, expanded_val);
                }
                Ok(Value::Object(expanded_obj))
            }
            Value::Array(arr) => {
                let mut expanded_arr = Vec::new();
                for item in arr {
                    expanded_arr.push(self.expand_env_vars(item, context)?);
                }
                Ok(Value::Array(expanded_arr))
            }
            _ => Ok(value),
        }
    }

    /// Expand environment variables in a string
    fn expand_string_env_vars(&self, s: &str, context: &ResolverContext) -> LoaderResult<String> {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            // Handle escaped sequences: \${ becomes literal ${
            if ch == '\\' && chars.peek() == Some(&'$') {
                chars.next(); // consume '$'
                if chars.peek() == Some(&'{') {
                    chars.next(); // consume '{'
                    result.push_str("${");
                } else {
                    result.push('\\');
                    result.push('$');
                }
            } else if ch == '$' && chars.peek() == Some(&'{') {
                chars.next(); // consume '{'
                
                // Look for ENV: prefix
                let mut var_spec = String::new();
                let mut brace_count = 1;
                
                for ch in chars.by_ref() {
                    if ch == '{' {
                        brace_count += 1;
                    } else if ch == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            break;
                        }
                    }
                    var_spec.push(ch);
                }

                if brace_count != 0 {
                    return Err(LoaderError::environment_error(
                        var_spec,
                        context.base_dir.clone(),
                        "Unclosed environment variable reference".to_string(),
                    ));
                }

                // Parse ENV:VAR_NAME or ENV:VAR_NAME:default format
                if let Some(var_spec_without_prefix) = var_spec.strip_prefix("ENV:") {
                    // Check for default value after the variable name
                    let (var_name, default_value) = if let Some(colon_pos) = var_spec_without_prefix.find(':') {
                        let var_name = &var_spec_without_prefix[..colon_pos];
                        let default = &var_spec_without_prefix[colon_pos + 1..];
                        (var_name, Some(default))
                    } else {
                        (var_spec_without_prefix, None)
                    };
                    
                    if let Some(var_value) = context.get_env_var(var_name) {
                        result.push_str(&var_value);
                    } else if let Some(default) = default_value {
                        // Use default value if environment variable is not found
                        result.push_str(default);
                    } else {
                        return Err(LoaderError::environment_error(
                            var_name.to_string(),
                            context.base_dir.clone(),
                            "Environment variable not found and no default provided".to_string(),
                        ));
                    }
                } else {
                    return Err(LoaderError::environment_error(
                        var_spec,
                        context.base_dir.clone(),
                        "Environment variable must use ENV: prefix".to_string(),
                    ));
                }
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }

    /// Clear the resolution cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

impl Default for ReferenceResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for reference validation
pub struct ReferenceValidator;

impl ReferenceValidator {
    /// Validate that a reference string is well-formed
    pub fn validate_reference(reference: &str) -> LoaderResult<()> {
        if reference.is_empty() {
            return Err(LoaderError::ValidationError {
                path: PathBuf::from("<reference>"),
                reason: "Reference cannot be empty".to_string(),
            });
        }

        // Check for invalid characters
        let invalid_chars = ['<', '>', '"', '|', '?', '*'];
        for ch in invalid_chars {
            if reference.contains(ch) {
                return Err(LoaderError::ValidationError {
                    path: PathBuf::from("<reference>"),
                    reason: format!("Reference contains invalid character: {}", ch),
                });
            }
        }

        // Validate JSON pointer if present
        if let Some(hash_pos) = reference.find('#') {
            let pointer = &reference[hash_pos + 1..];
            Self::validate_json_pointer(pointer)?;
        }

        Ok(())
    }

    /// Validate JSON pointer syntax
    pub fn validate_json_pointer(pointer: &str) -> LoaderResult<()> {
        if pointer.is_empty() {
            return Ok(()); // Empty pointer is valid (refers to root)
        }

        if !pointer.starts_with('/') {
            return Err(LoaderError::ValidationError {
                path: PathBuf::from("<pointer>"),
                reason: "JSON pointer must start with '/' or be empty".to_string(),
            });
        }

        // Check for proper escaping
        let segments = pointer.split('/').skip(1);
        for segment in segments {
            // Check for unescaped ~ characters
            let mut chars = segment.chars().peekable();
            while let Some(ch) = chars.next() {
                if ch == '~' {
                    match chars.peek() {
                        Some('0') | Some('1') => {
                            chars.next(); // Valid escape sequence
                        }
                        _ => {
                            return Err(LoaderError::ValidationError {
                                path: PathBuf::from("<pointer>"),
                                reason: "Invalid escape sequence in JSON pointer".to_string(),
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Extract all references from a schema
    pub fn extract_references(value: &Value) -> HashSet<String> {
        let mut references = HashSet::new();
        Self::extract_refs_recursive(value, &mut references);
        references
    }

    fn extract_refs_recursive(value: &Value, references: &mut HashSet<String>) {
        match value {
            Value::Object(obj) => {
                if let Some(ref_value) = obj.get("$ref") {
                    if let Some(ref_str) = ref_value.as_str() {
                        references.insert(ref_str.to_string());
                    }
                }
                for val in obj.values() {
                    Self::extract_refs_recursive(val, references);
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    Self::extract_refs_recursive(item, references);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_resolver_context() {
        let base_dir = PathBuf::from("/tmp");
        let mut context = ResolverContext::new(base_dir.clone());

        assert_eq!(context.base_dir, base_dir);
        assert_eq!(context.resolution_stack.len(), 0);
        assert_eq!(context.max_depth, 10);

        // Test path operations
        let path1 = PathBuf::from("file1.yaml");
        assert!(context.push_path(path1.clone()).is_ok());
        assert_eq!(context.resolution_stack.len(), 1);

        // Test circular reference detection
        assert!(context.push_path(path1.clone()).is_err());

        // Test pop
        let popped = context.pop_path();
        assert_eq!(popped, Some(path1));
        assert_eq!(context.resolution_stack.len(), 0);
    }

    #[test]
    fn test_env_var_expansion() -> LoaderResult<()> {
        let dir = tempdir().unwrap();
        let context = ResolverContext::for_testing(
            dir.path().to_path_buf(),
            [("TEST_VAR".to_string(), "test_value".to_string())]
                .iter()
                .cloned()
                .collect(),
        );

        let resolver = ReferenceResolver::new();

        // Test simple expansion
        let value = json!({
            "api_key": "${ENV:TEST_VAR}",
            "url": "https://api.example.com"
        });

        let expanded = resolver.expand_env_vars(value, &context)?;
        assert_eq!(expanded["api_key"], "test_value");
        assert_eq!(expanded["url"], "https://api.example.com");

        // Test expansion in nested structures
        let nested_value = json!({
            "config": {
                "database": {
                    "password": "${ENV:TEST_VAR}"
                }
            }
        });

        let expanded_nested = resolver.expand_env_vars(nested_value, &context)?;
        assert_eq!(expanded_nested["config"]["database"]["password"], "test_value");

        Ok(())
    }
    
    #[test]
    fn test_env_var_with_defaults() -> LoaderResult<()> {
        let dir = tempdir().unwrap();
        let context = ResolverContext::for_testing(
            dir.path().to_path_buf(),
            [("EXISTING_VAR".to_string(), "exists".to_string())]
                .iter()
                .cloned()
                .collect(),
        );

        let resolver = ReferenceResolver::new();

        // Test default value when var doesn't exist
        let value = json!({
            "missing": "${ENV:MISSING_VAR:default_value}",
            "existing": "${ENV:EXISTING_VAR:not_used}",
            "complex_default": "${ENV:NOT_SET:http://localhost:8080}"
        });

        let expanded = resolver.expand_env_vars(value, &context)?;
        assert_eq!(expanded["missing"], "default_value");
        assert_eq!(expanded["existing"], "exists");
        assert_eq!(expanded["complex_default"], "http://localhost:8080");

        Ok(())
    }
    
    #[test]
    fn test_escaping() -> LoaderResult<()> {
        let dir = tempdir().unwrap();
        let context = ResolverContext::for_testing(
            dir.path().to_path_buf(),
            HashMap::new(),
        );

        let resolver = ReferenceResolver::new();

        // Test escaped sequences
        let value = json!({
            "literal": "\\${ENV:NOT_EXPANDED}",
            "mixed": "prefix \\${literal} and ${ENV:TEST:expanded}"
        });

        let expanded = resolver.expand_env_vars(value, &context)?;
        assert_eq!(expanded["literal"], "${ENV:NOT_EXPANDED}");
        assert_eq!(expanded["mixed"], "prefix ${literal} and expanded");

        Ok(())
    }
    
    #[test]
    fn test_sensitive_value_redaction() {
        let resolver = ReferenceResolver::new();
        
        // Test sensitive patterns
        assert!(resolver.is_sensitive("API_KEY"));
        assert!(resolver.is_sensitive("DATABASE_PASSWORD"));
        assert!(resolver.is_sensitive("SECRET_TOKEN"));
        assert!(resolver.is_sensitive("auth_token"));
        assert!(!resolver.is_sensitive("USERNAME"));
        assert!(!resolver.is_sensitive("DATABASE_HOST"));
        
        // Test redaction
        assert_eq!(resolver.redact_value("API_KEY", "sk-1234567890abcdef"), "sk-***");
        assert_eq!(resolver.redact_value("PASSWORD", "short"), "[REDACTED]");
        assert_eq!(resolver.redact_value("USERNAME", "john_doe"), "john_doe");
    }

    #[test]
    fn test_env_var_errors() {
        let dir = tempdir().unwrap();
        let context = ResolverContext::for_testing(dir.path().to_path_buf(), HashMap::new());
        let resolver = ReferenceResolver::new();

        // Test missing variable
        let value = json!({"key": "${ENV:MISSING_VAR}"});
        assert!(resolver.expand_env_vars(value, &context).is_err());

        // Test invalid format
        let invalid_value = json!({"key": "${INVALID_FORMAT}"});
        assert!(resolver.expand_env_vars(invalid_value, &context).is_err());

        // Test unclosed brace
        let unclosed_value = json!({"key": "${ENV:TEST"});
        assert!(resolver.expand_env_vars(unclosed_value, &context).is_err());
    }

    #[test]
    fn test_json_pointer_application() -> LoaderResult<()> {
        let resolver = ReferenceResolver::new();
        let document = json!({
            "definitions": {
                "User": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"}
                    }
                }
            },
            "items": [1, 2, 3]
        });

        let path = Path::new("test.json");

        // Test object access
        let result = resolver.apply_json_pointer(
            &document,
            "/definitions/User",
            "#/definitions/User",
            path,
        )?;
        assert_eq!(result["type"], "object");

        // Test array access
        let result = resolver.apply_json_pointer(&document, "/items/1", "#/items/1", path)?;
        assert_eq!(result, 2);

        // Test root access
        let result = resolver.apply_json_pointer(&document, "", "#", path)?;
        assert_eq!(result, document);

        Ok(())
    }

    #[test]
    fn test_reference_validation() {
        // Valid references
        assert!(ReferenceValidator::validate_reference("file.yaml").is_ok());
        assert!(ReferenceValidator::validate_reference("file.yaml#/definitions/User").is_ok());
        assert!(ReferenceValidator::validate_reference("#/local/reference").is_ok());

        // Invalid references
        assert!(ReferenceValidator::validate_reference("").is_err());
        assert!(ReferenceValidator::validate_reference("file<invalid>.yaml").is_err());
        assert!(ReferenceValidator::validate_reference("file.yaml#invalid_pointer").is_err());
    }

    #[test]
    fn test_json_pointer_validation() {
        // Valid pointers
        assert!(ReferenceValidator::validate_json_pointer("").is_ok());
        assert!(ReferenceValidator::validate_json_pointer("/definitions/User").is_ok());
        assert!(ReferenceValidator::validate_json_pointer("/items/0").is_ok());
        assert!(ReferenceValidator::validate_json_pointer("/escaped~0property").is_ok());
        assert!(ReferenceValidator::validate_json_pointer("/escaped~1property").is_ok());

        // Invalid pointers
        assert!(ReferenceValidator::validate_json_pointer("invalid").is_err());
        assert!(ReferenceValidator::validate_json_pointer("/invalid~escape").is_err());
    }

    #[test]
    fn test_reference_extraction() {
        let schema = json!({
            "type": "object",
            "properties": {
                "user": {"$ref": "#/definitions/User"},
                "nested": {
                    "items": {"$ref": "external.yaml#/User"}
                }
            },
            "definitions": {
                "User": {"$ref": "user.yaml"}
            }
        });

        let refs = ReferenceValidator::extract_references(&schema);
        assert_eq!(refs.len(), 3);
        assert!(refs.contains("#/definitions/User"));
        assert!(refs.contains("external.yaml#/User"));
        assert!(refs.contains("user.yaml"));
    }

    #[test]
    fn test_file_reference_resolution() -> LoaderResult<()> {
        let dir = tempdir().unwrap();

        // Create referenced file
        let referenced_file = dir.path().join("user.yaml");
        let user_schema = r#"
type: object
properties:
  name:
    type: string
  age:
    type: integer
"#;
        fs::write(&referenced_file, user_schema)?;

        // Create main file with reference
        let main_file = dir.path().join("main.yaml");
        let main_schema = r#"
type: object
properties:
  user:
    $ref: "user.yaml"
"#;
        fs::write(&main_file, main_schema)?;

        let mut resolver = ReferenceResolver::new();
        let mut context = ResolverContext::new(dir.path().to_path_buf());

        let parsed_main = SchemaParser::new().parse_file(&main_file)?;
        let resolved = resolver.resolve(parsed_main, &mut context)?;

        // Check that the reference was resolved
        assert_eq!(resolved["properties"]["user"]["type"], "object");
        assert_eq!(
            resolved["properties"]["user"]["properties"]["name"]["type"],
            "string"
        );

        Ok(())
    }

    #[test]
    fn test_circular_reference_detection() -> LoaderResult<()> {
        let dir = tempdir().unwrap();

        // Create file A that references B
        let file_a = dir.path().join("a.yaml");
        let schema_a = r#"
type: object
properties:
  b_ref:
    $ref: "b.yaml"
"#;
        fs::write(&file_a, schema_a)?;

        // Create file B that references A (circular)
        let file_b = dir.path().join("b.yaml");
        let schema_b = r#"
type: object
properties:
  a_ref:
    $ref: "a.yaml"
"#;
        fs::write(&file_b, schema_b)?;

        let mut resolver = ReferenceResolver::new();
        let mut context = ResolverContext::new(dir.path().to_path_buf());
        context.max_depth = 3; // Set a low depth to trigger the error faster

        let parsed_main = SchemaParser::new().parse_file(&file_a)?;
        let result = resolver.resolve(parsed_main, &mut context);

        // Should detect circular reference
        assert!(result.is_err());
        if let Err(LoaderError::CircularReference { .. }) = result {
            // Expected error type
        } else {
            panic!("Expected CircularReference error");
        }

        Ok(())
    }
}