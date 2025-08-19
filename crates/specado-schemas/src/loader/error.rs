//! Error types for schema loading operations
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use std::path::PathBuf;
use thiserror::Error;

/// Result type for loader operations
pub type LoaderResult<T> = Result<T, LoaderError>;

/// Comprehensive error types for schema loading operations
#[derive(Error, Debug)]
pub enum LoaderError {
    /// File I/O errors
    #[error("Failed to read file '{path}': {source}")]
    IoError {
        path: PathBuf,
        source: std::io::Error,
    },

    /// YAML parsing errors
    #[error("Failed to parse YAML file '{path}': {source}")]
    YamlParseError {
        path: PathBuf,
        source: serde_yaml::Error,
    },

    /// JSON parsing errors
    #[error("Failed to parse JSON file '{path}': {source}")]
    JsonParseError {
        path: PathBuf,
        source: serde_json::Error,
    },

    /// Unsupported file format
    #[error("Unsupported file format for '{path}'. Expected .yaml, .yml, or .json")]
    UnsupportedFormat { path: PathBuf },

    /// Reference resolution errors
    #[error("Failed to resolve reference '{reference}' in '{source_path}': {reason}")]
    ReferenceError {
        reference: String,
        source_path: PathBuf,
        reason: String,
    },

    /// Circular reference detection
    #[error("Circular reference detected: {chain}")]
    CircularReference { chain: String },

    /// Environment variable expansion errors
    #[error("Failed to expand environment variable '{var_name}' in '{path}': {reason}")]
    EnvironmentError {
        var_name: String,
        path: PathBuf,
        reason: String,
    },

    /// Cache operation errors
    #[error("Cache operation failed: {reason}")]
    CacheError { reason: String },

    /// Path traversal security error
    #[error("Path traversal detected in reference '{reference}' from '{source_path}'")]
    PathTraversal {
        reference: String,
        source_path: PathBuf,
    },

    /// Schema version validation errors
    #[error("Invalid schema version '{version}' in '{path}': {reason}")]
    VersionError {
        version: String,
        path: PathBuf,
        reason: String,
    },

    /// Generic validation errors
    #[error("Validation failed for '{path}': {reason}")]
    ValidationError { path: PathBuf, reason: String },
}

impl From<std::io::Error> for LoaderError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError {
            path: PathBuf::from("<unknown>"),
            source: error,
        }
    }
}

impl LoaderError {
    /// Create an I/O error with path context
    pub fn io_error(path: PathBuf, error: std::io::Error) -> Self {
        Self::IoError {
            path,
            source: error,
        }
    }

    /// Create a YAML parsing error with path context
    pub fn yaml_parse_error(path: PathBuf, error: serde_yaml::Error) -> Self {
        Self::YamlParseError {
            path,
            source: error,
        }
    }

    /// Create a JSON parsing error with path context
    pub fn json_parse_error(path: PathBuf, error: serde_json::Error) -> Self {
        Self::JsonParseError {
            path,
            source: error,
        }
    }

    /// Create an unsupported format error
    pub fn unsupported_format(path: PathBuf) -> Self {
        Self::UnsupportedFormat { path }
    }

    /// Create a reference resolution error
    pub fn reference_error(reference: String, source_path: PathBuf, reason: String) -> Self {
        Self::ReferenceError {
            reference,
            source_path,
            reason,
        }
    }

    /// Create a circular reference error
    pub fn circular_reference(chain: Vec<PathBuf>) -> Self {
        let chain_str = chain
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(" -> ");
        Self::CircularReference { chain: chain_str }
    }

    /// Create an environment variable error
    pub fn environment_error(var_name: String, path: PathBuf, reason: String) -> Self {
        Self::EnvironmentError {
            var_name,
            path,
            reason,
        }
    }

    /// Create a cache error
    pub fn cache_error(reason: String) -> Self {
        Self::CacheError { reason }
    }

    /// Create a path traversal error
    pub fn path_traversal(reference: String, source_path: PathBuf) -> Self {
        Self::PathTraversal {
            reference,
            source_path,
        }
    }

    /// Create a version error
    pub fn version_error(version: String, path: PathBuf, reason: String) -> Self {
        Self::VersionError {
            version,
            path,
            reason,
        }
    }

    /// Create a validation error
    pub fn validation_error(path: PathBuf, reason: String) -> Self {
        Self::ValidationError { path, reason }
    }

    /// Get the path associated with this error, if any
    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            Self::IoError { path, .. } => Some(path),
            Self::YamlParseError { path, .. } => Some(path),
            Self::JsonParseError { path, .. } => Some(path),
            Self::UnsupportedFormat { path } => Some(path),
            Self::ReferenceError { source_path, .. } => Some(source_path),
            Self::EnvironmentError { path, .. } => Some(path),
            Self::PathTraversal { source_path, .. } => Some(source_path),
            Self::VersionError { path, .. } => Some(path),
            Self::ValidationError { path, .. } => Some(path),
            _ => None,
        }
    }

    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::EnvironmentError { .. } | Self::CacheError { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let path = PathBuf::from("test.yaml");
        
        let io_err = LoaderError::io_error(
            path.clone(),
            std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
        );
        assert!(matches!(io_err, LoaderError::IoError { .. }));
        assert_eq!(io_err.path(), Some(&path));

        let circular_err = LoaderError::circular_reference(vec![
            PathBuf::from("a.yaml"),
            PathBuf::from("b.yaml"),
            PathBuf::from("a.yaml"),
        ]);
        assert!(matches!(circular_err, LoaderError::CircularReference { .. }));
    }

    #[test]
    fn test_error_recovery() {
        let cache_err = LoaderError::cache_error("Test cache error".to_string());
        assert!(cache_err.is_recoverable());

        let parse_err = LoaderError::yaml_parse_error(
            PathBuf::from("test.yaml"),
            serde_yaml::from_str::<serde_yaml::Value>("{").unwrap_err(),
        );
        assert!(!parse_err.is_recoverable());
    }
}