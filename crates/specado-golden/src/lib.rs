//! Golden test infrastructure for Specado translation engine
//!
//! This crate provides snapshot testing capabilities for validating
//! translation outputs remain consistent across changes.

pub mod corpus;
pub mod diff;
pub mod runner;
pub mod snapshot;

use std::path::PathBuf;
use thiserror::Error;

pub use corpus::CorpusManager;
pub use diff::{DiffEngine, DiffOptions};
pub use runner::{GoldenTestRunner, TestResult};
pub use snapshot::{Snapshot, SnapshotManager};

/// Golden test error types
#[derive(Debug, Error)]
pub enum GoldenError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Snapshot mismatch: {0}")]
    SnapshotMismatch(String),
    
    #[error("Corpus error: {0}")]
    CorpusError(String),
    
    #[error("Test failed: {0}")]
    TestFailed(String),
}

pub type Result<T> = std::result::Result<T, GoldenError>;

/// Configuration for golden tests
#[derive(Debug, Clone)]
pub struct GoldenConfig {
    /// Root directory for test corpus
    pub corpus_dir: PathBuf,
    
    /// Directory for snapshots
    pub snapshot_dir: PathBuf,
    
    /// Whether to update snapshots
    pub update_snapshots: bool,
    
    /// Whether to create missing snapshots
    pub create_missing: bool,
    
    /// Diff options
    pub diff_options: DiffOptions,
    
    /// Verbose output
    pub verbose: bool,
}

impl Default for GoldenConfig {
    fn default() -> Self {
        let update_snapshots = std::env::var("UPDATE_GOLDEN")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);
            
        Self {
            corpus_dir: PathBuf::from("../../golden-corpus"),
            snapshot_dir: PathBuf::from("../../golden-corpus/snapshots"),
            update_snapshots,
            create_missing: update_snapshots,
            diff_options: DiffOptions::default(),
            verbose: false,
        }
    }
}

impl GoldenConfig {
    /// Create config from environment and defaults
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(corpus_dir) = std::env::var("GOLDEN_CORPUS_DIR") {
            config.corpus_dir = PathBuf::from(corpus_dir);
        }
        
        if let Ok(snapshot_dir) = std::env::var("GOLDEN_SNAPSHOT_DIR") {
            config.snapshot_dir = PathBuf::from(snapshot_dir);
        }
        
        if let Ok(verbose) = std::env::var("GOLDEN_VERBOSE") {
            config.verbose = verbose == "1" || verbose.to_lowercase() == "true";
        }
        
        config
    }
}

/// Macro for defining golden tests
#[macro_export]
macro_rules! golden_test {
    ($name:ident, $test_path:expr) => {
        #[test]
        fn $name() {
            use $crate::{GoldenConfig, GoldenTestRunner};
            
            let config = GoldenConfig::from_env();
            let runner = GoldenTestRunner::new(config);
            
            runner
                .run_test($test_path)
                .expect(&format!("Golden test failed: {}", $test_path));
        }
    };
}

/// Macro for batch golden tests
#[macro_export]
macro_rules! golden_test_batch {
    ($pattern:expr) => {
        #[test]
        fn golden_tests() {
            use $crate::{GoldenConfig, GoldenTestRunner};
            
            let config = GoldenConfig::from_env();
            let runner = GoldenTestRunner::new(config);
            
            runner
                .run_batch($pattern)
                .expect(&format!("Golden test batch failed: {}", $pattern));
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_from_env() {
        let config = GoldenConfig::from_env();
        assert!(!config.corpus_dir.as_os_str().is_empty());
        assert!(!config.snapshot_dir.as_os_str().is_empty());
    }
}