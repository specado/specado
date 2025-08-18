//! Golden test runner for executing snapshot tests

use crate::{
    corpus::{CorpusManager, TestCase},
    diff::{DiffEngine, DiffOptions},
    snapshot::SnapshotManager,
    GoldenConfig, GoldenError, Result,
};
use colored::*;
use serde_json::Value;
use specado_core::translate;
use std::time::Instant;

/// Result of running a golden test
#[derive(Debug)]
pub struct TestResult {
    /// Name of the test
    pub name: String,
    
    /// Whether the test passed
    pub passed: bool,
    
    /// Error message if failed
    pub error: Option<String>,
    
    /// Diff output if comparison failed
    pub diff: Option<String>,
    
    /// Execution time in milliseconds
    pub duration_ms: u64,
    
    /// Whether snapshot was updated
    pub updated: bool,
}

impl TestResult {
    /// Print the test result
    pub fn print(&self, verbose: bool) {
        let status = if self.passed {
            "PASS".green().bold()
        } else {
            "FAIL".red().bold()
        };
        
        println!("{} {} ({}ms)", status, self.name, self.duration_ms);
        
        if let Some(ref error) = self.error {
            println!("  {}: {}", "Error".red(), error);
        }
        
        if verbose || !self.passed {
            if let Some(ref diff) = self.diff {
                println!("{}", diff);
            }
        }
        
        if self.updated {
            println!("  {}", "Snapshot updated".yellow());
        }
    }
}

/// Runner for golden tests
pub struct GoldenTestRunner {
    config: GoldenConfig,
    corpus_manager: CorpusManager,
    snapshot_manager: SnapshotManager,
}

impl GoldenTestRunner {
    /// Create a new test runner
    pub fn new(config: GoldenConfig) -> Self {
        let corpus_manager = CorpusManager::new(&config.corpus_dir);
        let snapshot_manager = SnapshotManager::new(&config.snapshot_dir);
        
        Self {
            config,
            corpus_manager,
            snapshot_manager,
        }
    }
    
    /// Run a single test by name
    pub fn run_test(&self, test_name: &str) -> Result<TestResult> {
        let start = Instant::now();
        
        // Load the test case
        let test_path = self.config.corpus_dir.join(test_name).join("test.json");
        let test_case = self.corpus_manager.load_test_case(&test_path)?;
        
        // Execute the test
        let result = self.execute_test(&test_case);
        
        let duration_ms = start.elapsed().as_millis() as u64;
        
        // Create test result
        let test_result = match result {
            Ok((passed, diff, updated)) => TestResult {
                name: test_name.to_string(),
                passed,
                error: if passed { None } else { Some("Snapshot mismatch".to_string()) },
                diff,
                duration_ms,
                updated,
            },
            Err(e) => TestResult {
                name: test_name.to_string(),
                passed: false,
                error: Some(e.to_string()),
                diff: None,
                duration_ms,
                updated: false,
            },
        };
        
        if self.config.verbose {
            test_result.print(true);
        }
        
        if test_result.passed {
            Ok(test_result)
        } else {
            Err(GoldenError::TestFailed(format!(
                "Test '{}' failed: {}",
                test_name,
                test_result.error.as_ref().unwrap_or(&"Unknown error".to_string())
            )))
        }
    }
    
    /// Run a batch of tests matching a pattern
    pub fn run_batch(&self, pattern: &str) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();
        
        // Discover all tests
        let tests = self.corpus_manager.discover_tests()?;
        
        // Filter by pattern
        let filtered_tests: Vec<_> = if pattern == "*" {
            tests
        } else {
            tests.into_iter()
                .filter(|t| t.name.contains(pattern) || t.category.contains(pattern))
                .collect()
        };
        
        if filtered_tests.is_empty() {
            return Err(GoldenError::CorpusError(format!(
                "No tests found matching pattern '{}'",
                pattern
            )));
        }
        
        println!("Running {} tests...\n", filtered_tests.len());
        
        let mut passed = 0;
        let mut failed = 0;
        
        for test_case in filtered_tests {
            let test_name = format!("{}/{}", test_case.category, test_case.name);
            let result = self.run_test(&test_name).unwrap_or_else(|e| TestResult {
                name: test_name.clone(),
                passed: false,
                error: Some(e.to_string()),
                diff: None,
                duration_ms: 0,
                updated: false,
            });
            
            if result.passed {
                passed += 1;
            } else {
                failed += 1;
            }
            
            result.print(self.config.verbose);
            results.push(result);
        }
        
        // Print summary
        println!("\n{}", "=== Test Summary ===".bold());
        println!(
            "{}: {} passed, {} failed",
            "Results".bold(),
            passed.to_string().green(),
            failed.to_string().red()
        );
        
        if failed > 0 {
            Err(GoldenError::TestFailed(format!(
                "{} test(s) failed",
                failed
            )))
        } else {
            Ok(results)
        }
    }
    
    /// Execute a single test case
    fn execute_test(&self, test_case: &TestCase) -> Result<(bool, Option<String>, bool)> {
        // Skip disabled tests
        if !test_case.metadata.enabled {
            return Ok((true, None, false));
        }
        
        // Perform the translation
        let translation_result = self.perform_translation(test_case)?;
        
        // Get snapshot name
        let snapshot_name = format!("{}/{}", test_case.category, test_case.name);
        
        // Check if snapshot exists
        let snapshot_exists = self.snapshot_manager.exists(&snapshot_name);
        
        if !snapshot_exists {
            if self.config.create_missing || self.config.update_snapshots {
                // Create new snapshot
                self.snapshot_manager.create(
                    &snapshot_name,
                    translation_result,
                    Some(test_case.metadata.description.clone()),
                )?;
                
                return Ok((true, None, true));
            } else {
                return Err(GoldenError::SnapshotMismatch(format!(
                    "Snapshot '{}' does not exist. Run with UPDATE_GOLDEN=1 to create it.",
                    snapshot_name
                )));
            }
        }
        
        // Load existing snapshot
        let mut snapshot = self.snapshot_manager.load(&snapshot_name)?;
        
        // Apply test-specific ignore fields
        snapshot.ignore_fields = test_case.expectations.ignore_fields.clone();
        
        // Create a new diff engine for this test with volatile patterns
        let mut diff_engine = DiffEngine::new(self.config.diff_options.clone());
        for volatile in &test_case.expectations.volatile_fields {
            diff_engine.add_volatile_pattern(&volatile.path, &volatile.pattern)?;
        }
        
        // Prepare values for comparison
        let mut expected = snapshot.content.clone();
        let mut actual = translation_result.clone();
        
        // Apply ignore fields
        crate::snapshot::apply_ignores(&mut expected, &snapshot.ignore_fields);
        crate::snapshot::apply_ignores(&mut actual, &snapshot.ignore_fields);
        
        // Compare
        let diff_result = diff_engine.compare(&expected, &actual);
        
        if diff_result.matches {
            Ok((true, None, false))
        } else if self.config.update_snapshots {
            // Update the snapshot
            self.snapshot_manager.backup(&snapshot_name)?;
            self.snapshot_manager.update(&snapshot_name, translation_result)?;
            Ok((true, Some(diff_result.diff_output), true))
        } else {
            Ok((false, Some(diff_result.diff_output), false))
        }
    }
    
    /// Perform the actual translation
    fn perform_translation(&self, test_case: &TestCase) -> Result<Value> {
        // Convert input to PromptSpec
        let prompt_spec: specado_core::types::PromptSpec = serde_json::from_value(test_case.input.prompt_spec.clone())
            .map_err(|e| GoldenError::Json(e))?;
        
        // Get provider spec (use default if not specified)
        let provider_name = test_case.provider.as_deref().unwrap_or("openai");
        let provider_spec = if let Some(ref spec) = test_case.input.provider_spec {
            serde_json::from_value(spec.clone())
                .map_err(|e| GoldenError::Json(e))?
        } else {
            // Load default provider spec
            self.load_default_provider_spec(provider_name)?
        };
        
        // Get model ID - use first model from provider or default
        let model_id = provider_spec.models.first()
            .map(|m| m.id.as_str())
            .unwrap_or("gpt-5");
        
        // Perform translation
        let result = translate(&prompt_spec, &provider_spec, model_id, prompt_spec.strict_mode)
            .map_err(|e| GoldenError::TestFailed(format!("Translation failed: {}", e)))?;
        
        // Convert to JSON for comparison
        let json_result = serde_json::to_value(result)
            .map_err(|e| GoldenError::Json(e))?;
        
        Ok(json_result)
    }
    
    /// Load default provider spec
    fn load_default_provider_spec(&self, provider: &str) -> Result<specado_core::types::ProviderSpec> {
        // For now, create a minimal provider spec with a basic model
        // In production, this would load from provider files
        let spec = match provider {
            "openai" => serde_json::json!({
                "spec_version": "1.0.0",
                "provider": {
                    "name": provider,
                    "base_url": "https://api.openai.com/v1",
                    "headers": {"Authorization": "Bearer $OPENAI_API_KEY"}
                },
                "models": [{
                    "id": "gpt-5",
                    "aliases": ["gpt5"],
                    "family": "gpt",
                    "endpoints": {
                        "chat_completion": {
                            "method": "POST",
                            "path": "/chat/completions",
                            "protocol": "http"
                        },
                        "streaming_chat_completion": {
                            "method": "POST",
                            "path": "/chat/completions",
                            "protocol": "sse"
                        }
                    },
                    "input_modes": {
                        "messages": true,
                        "single_text": false,
                        "images": false
                    },
                    "tooling": {
                        "tools_supported": true,
                        "parallel_tool_calls_default": true,
                        "can_disable_parallel_tool_calls": true,
                        "disable_switch": {"parallel_tool_calls": false}
                    },
                    "json_output": {
                        "native_param": true,
                        "strategy": "response_format"
                    },
                    "parameters": {},
                    "constraints": {
                        "system_prompt_location": "first_message",
                        "forbid_unknown_top_level_fields": true,
                        "mutually_exclusive": [],
                        "resolution_preferences": [],
                        "limits": {
                            "max_tool_schema_bytes": 16384,
                            "max_system_prompt_bytes": 32768
                        }
                    },
                    "mappings": {
                        "paths": {},
                        "flags": {}
                    },
                    "response_normalization": {
                        "sync": {
                            "content_path": "$.choices[0].message.content",
                            "finish_reason_path": "$.choices[0].finish_reason",
                            "finish_reason_map": {}
                        },
                        "stream": {
                            "protocol": "sse",
                            "event_selector": {
                                "type_path": "$.choices[0].delta",
                                "routes": []
                            }
                        }
                    }
                }]
            }),
            _ => serde_json::json!({
                "spec_version": "1.0.0",
                "provider": {
                    "name": provider,
                    "base_url": format!("https://api.{}.com", provider),
                    "headers": {}
                },
                "models": []
            })
        };
        
        serde_json::from_value(spec)
            .map_err(|e| GoldenError::Json(e))
    }
    
    /// Initialize the corpus with sample tests
    pub fn init_corpus(&self) -> Result<()> {
        self.corpus_manager.init_corpus()
    }
    
    /// List all available tests
    pub fn list_tests(&self) -> Result<Vec<String>> {
        let tests = self.corpus_manager.discover_tests()?;
        Ok(tests.into_iter().map(|t| format!("{}/{}", t.category, t.name)).collect())
    }
    
    /// Get corpus statistics
    pub fn get_statistics(&self) -> Result<()> {
        let stats = self.corpus_manager.get_statistics()?;
        stats.print();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_runner_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = GoldenConfig {
            corpus_dir: temp_dir.path().to_path_buf(),
            snapshot_dir: temp_dir.path().join("snapshots"),
            update_snapshots: false,
            create_missing: false,
            diff_options: DiffOptions::default(),
            verbose: false,
        };
        
        let runner = GoldenTestRunner::new(config);
        runner.init_corpus().unwrap();
        
        let tests = runner.list_tests().unwrap();
        assert!(tests.len() > 0);
    }
}