//! Test corpus management for golden tests

use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A test case in the corpus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Name of the test case
    pub name: String,
    
    /// Category/group of the test
    pub category: String,
    
    /// Input data (PromptSpec)
    pub input: TestInput,
    
    /// Provider configuration for the test
    pub provider: Option<String>,
    
    /// Expected behavior configuration
    pub expectations: TestExpectations,
    
    /// Test metadata
    pub metadata: TestMetadata,
}

/// Input for a test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestInput {
    /// The PromptSpec or raw JSON input
    pub prompt_spec: Value,
    
    /// Optional provider spec override
    pub provider_spec: Option<Value>,
}

/// Expected behavior for a test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExpectations {
    /// Whether the translation should succeed
    pub should_succeed: bool,
    
    /// Expected error pattern if should_succeed is false
    pub error_pattern: Option<String>,
    
    /// Fields to ignore in comparison
    #[serde(default)]
    pub ignore_fields: Vec<String>,
    
    /// Volatile fields that may change
    #[serde(default)]
    pub volatile_fields: Vec<VolatileFieldSpec>,
    
    /// Expected lossiness codes (if any)
    #[serde(default)]
    pub expected_lossiness: Vec<String>,
}

/// Specification for a volatile field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatileFieldSpec {
    pub path: String,
    pub pattern: String,
}

/// Metadata about a test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetadata {
    /// Description of what this tests
    pub description: String,
    
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Whether this test is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Priority level (lower = higher priority)
    #[serde(default = "default_priority")]
    pub priority: u32,
}

fn default_true() -> bool {
    true
}

fn default_priority() -> u32 {
    100
}

/// Manages the test corpus
pub struct CorpusManager {
    corpus_dir: PathBuf,
}

impl CorpusManager {
    /// Create a new corpus manager
    pub fn new(corpus_dir: impl AsRef<Path>) -> Self {
        Self {
            corpus_dir: corpus_dir.as_ref().to_path_buf(),
        }
    }
    
    /// Discover all test cases in the corpus
    pub fn discover_tests(&self) -> Result<Vec<TestCase>> {
        let mut tests = Vec::new();
        
        if !self.corpus_dir.exists() {
            return Ok(tests);
        }
        
        // Walk through all subdirectories
        for entry in WalkDir::new(&self.corpus_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            // Look for test case files
            if path.is_file() && path.file_name() == Some(std::ffi::OsStr::new("test.json")) {
                match self.load_test_case(path) {
                    Ok(test_case) => tests.push(test_case),
                    Err(e) => {
                        eprintln!("Warning: Failed to load test case {:?}: {}", path, e);
                    }
                }
            }
        }
        
        // Sort by priority
        tests.sort_by_key(|t| t.metadata.priority);
        
        Ok(tests)
    }
    
    /// Load a specific test case
    pub fn load_test_case(&self, path: &Path) -> Result<TestCase> {
        let content = fs::read_to_string(path)?;
        let mut test_case: TestCase = serde_json::from_str(&content)?;
        
        // If the test case has separate input files, load them
        let test_dir = path.parent().unwrap();
        
        // Load prompt spec if it's a file reference
        if let Value::String(ref filename) = test_case.input.prompt_spec {
            if filename.ends_with(".json") {
                let input_path = test_dir.join(filename);
                let input_content = fs::read_to_string(input_path)?;
                test_case.input.prompt_spec = serde_json::from_str(&input_content)?;
            }
        }
        
        // Load provider spec if it's a file reference
        if let Some(Value::String(ref filename)) = test_case.input.provider_spec {
            if filename.ends_with(".json") {
                let provider_path = test_dir.join(filename);
                let provider_content = fs::read_to_string(provider_path)?;
                test_case.input.provider_spec = Some(serde_json::from_str(&provider_content)?);
            }
        }
        
        Ok(test_case)
    }
    
    /// Filter tests by category
    pub fn filter_by_category(&self, tests: Vec<TestCase>, category: &str) -> Vec<TestCase> {
        tests
            .into_iter()
            .filter(|t| t.category == category || category == "*")
            .collect()
    }
    
    /// Filter tests by tags
    pub fn filter_by_tags(&self, tests: Vec<TestCase>, tags: &[String]) -> Vec<TestCase> {
        if tags.is_empty() {
            return tests;
        }
        
        tests
            .into_iter()
            .filter(|t| {
                tags.iter().any(|tag| t.metadata.tags.contains(tag))
            })
            .collect()
    }
    
    /// Get enabled tests only
    pub fn filter_enabled(&self, tests: Vec<TestCase>) -> Vec<TestCase> {
        tests
            .into_iter()
            .filter(|t| t.metadata.enabled)
            .collect()
    }
    
    /// Create the corpus directory structure
    pub fn init_corpus(&self) -> Result<()> {
        // Create main directories
        let dirs = [
            "basic",
            "complex",
            "edge-cases",
            "providers/openai",
            "providers/anthropic",
            "providers/google",
            "regression",
        ];
        
        for dir in &dirs {
            let path = self.corpus_dir.join(dir);
            fs::create_dir_all(&path)?;
        }
        
        // Create a sample test case
        self.create_sample_test()?;
        
        Ok(())
    }
    
    /// Create a sample test case
    fn create_sample_test(&self) -> Result<()> {
        let test_dir = self.corpus_dir.join("basic/hello-world");
        fs::create_dir_all(&test_dir)?;
        
        let test_case = TestCase {
            name: "hello-world".to_string(),
            category: "basic".to_string(),
            input: TestInput {
                prompt_spec: serde_json::json!({
                    "model_class": "Chat",
                    "messages": [
                        {
                            "role": "user",
                            "content": "Hello, world!"
                        }
                    ],
                    "strict_mode": "Warn"
                }),
                provider_spec: None,
            },
            provider: Some("openai".to_string()),
            expectations: TestExpectations {
                should_succeed: true,
                error_pattern: None,
                ignore_fields: vec!["metadata.timestamp".to_string()],
                volatile_fields: vec![
                    VolatileFieldSpec {
                        path: "metadata.duration_ms".to_string(),
                        pattern: r"^\d+$".to_string(),
                    }
                ],
                expected_lossiness: vec![],
            },
            metadata: TestMetadata {
                description: "Basic hello world translation test".to_string(),
                tags: vec!["basic".to_string(), "smoke".to_string()],
                enabled: true,
                priority: 1,
            },
        };
        
        let test_path = test_dir.join("test.json");
        let content = serde_json::to_string_pretty(&test_case)?;
        fs::write(test_path, content)?;
        
        Ok(())
    }
    
    /// List all test categories
    pub fn list_categories(&self) -> Result<Vec<String>> {
        let mut categories = Vec::new();
        
        if !self.corpus_dir.exists() {
            return Ok(categories);
        }
        
        for entry in fs::read_dir(&self.corpus_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    categories.push(name.to_string());
                }
            }
        }
        
        categories.sort();
        Ok(categories)
    }
    
    /// Get statistics about the corpus
    pub fn get_statistics(&self) -> Result<CorpusStatistics> {
        let tests = self.discover_tests()?;
        
        let mut stats = CorpusStatistics {
            total_tests: tests.len(),
            ..Default::default()
        };
        
        for test in tests {
            if test.metadata.enabled {
                stats.enabled_tests += 1;
            } else {
                stats.disabled_tests += 1;
            }
            
            *stats.tests_by_category.entry(test.category).or_insert(0) += 1;
            
            for tag in test.metadata.tags {
                *stats.tests_by_tag.entry(tag).or_insert(0) += 1;
            }
        }
        
        Ok(stats)
    }
}

/// Statistics about the test corpus
#[derive(Debug, Default)]
pub struct CorpusStatistics {
    pub total_tests: usize,
    pub enabled_tests: usize,
    pub disabled_tests: usize,
    pub tests_by_category: std::collections::HashMap<String, usize>,
    pub tests_by_tag: std::collections::HashMap<String, usize>,
}

impl CorpusStatistics {
    /// Print statistics to stdout
    pub fn print(&self) {
        println!("=== Corpus Statistics ===");
        println!("Total tests: {}", self.total_tests);
        println!("Enabled: {}", self.enabled_tests);
        println!("Disabled: {}", self.disabled_tests);
        
        if !self.tests_by_category.is_empty() {
            println!("\nTests by category:");
            let mut categories: Vec<_> = self.tests_by_category.iter().collect();
            categories.sort_by_key(|(k, _)| k.as_str());
            for (category, count) in categories {
                println!("  {}: {}", category, count);
            }
        }
        
        if !self.tests_by_tag.is_empty() {
            println!("\nTests by tag:");
            let mut tags: Vec<_> = self.tests_by_tag.iter().collect();
            tags.sort_by_key(|(k, _)| k.as_str());
            for (tag, count) in tags {
                println!("  {}: {}", tag, count);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_corpus_manager_init() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CorpusManager::new(temp_dir.path());
        
        manager.init_corpus().unwrap();
        
        // Check that directories were created
        assert!(temp_dir.path().join("basic").exists());
        assert!(temp_dir.path().join("complex").exists());
        assert!(temp_dir.path().join("providers/openai").exists());
        
        // Check that sample test was created
        let sample_test = temp_dir.path().join("basic/hello-world/test.json");
        assert!(sample_test.exists());
    }
    
    #[test]
    fn test_discover_tests() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CorpusManager::new(temp_dir.path());
        
        manager.init_corpus().unwrap();
        
        let tests = manager.discover_tests().unwrap();
        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].name, "hello-world");
    }
    
    #[test]
    fn test_filter_by_category() {
        let test1 = TestCase {
            name: "test1".to_string(),
            category: "basic".to_string(),
            input: TestInput {
                prompt_spec: serde_json::json!({}),
                provider_spec: None,
            },
            provider: None,
            expectations: TestExpectations {
                should_succeed: true,
                error_pattern: None,
                ignore_fields: vec![],
                volatile_fields: vec![],
                expected_lossiness: vec![],
            },
            metadata: TestMetadata {
                description: "Test 1".to_string(),
                tags: vec![],
                enabled: true,
                priority: 1,
            },
        };
        
        let test2 = test1.clone();
        let mut test2_mut = test2;
        test2_mut.category = "complex".to_string();
        
        let tests = vec![test1.clone(), test2_mut];
        
        let manager = CorpusManager::new(".");
        let filtered = manager.filter_by_category(tests, "basic");
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].category, "basic");
    }
}