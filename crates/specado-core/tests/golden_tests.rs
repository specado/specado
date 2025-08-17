//! Golden tests for the translation engine
//!
//! These tests use snapshot testing to ensure translation outputs
//! remain consistent across changes.

use specado_golden::{GoldenConfig, GoldenTestRunner};

/// Run all golden tests in the corpus
#[test]
#[ignore] // Ignore by default as these require the corpus to be set up
fn golden_test_suite() {
    let config = GoldenConfig::from_env();
    let runner = GoldenTestRunner::new(config);
    
    // Initialize corpus if it doesn't exist
    let _ = runner.init_corpus();
    
    // Run all tests
    match runner.run_batch("*") {
        Ok(results) => {
            println!("All {} golden tests passed!", results.len());
        }
        Err(e) => {
            panic!("Golden tests failed: {}", e);
        }
    }
}

/// Run only basic golden tests
#[test]
#[ignore]
fn golden_test_basic() {
    let config = GoldenConfig::from_env();
    let runner = GoldenTestRunner::new(config);
    
    runner.run_batch("basic").expect("Basic golden tests failed");
}

/// Run only edge case tests
#[test]
#[ignore]
fn golden_test_edge_cases() {
    let config = GoldenConfig::from_env();
    let runner = GoldenTestRunner::new(config);
    
    runner.run_batch("edge-cases").expect("Edge case golden tests failed");
}

/// Show corpus statistics
#[test]
#[ignore]
fn golden_corpus_stats() {
    let config = GoldenConfig::from_env();
    let runner = GoldenTestRunner::new(config);
    
    runner.get_statistics().expect("Failed to get corpus statistics");
}

// Individual test cases using the macro
#[cfg(test)]
mod individual_tests {
    use specado_golden::{golden_test, golden_test_batch};
    
    // Test individual cases
    golden_test!(test_simple_chat, "basic/simple-chat");
    golden_test!(test_with_sampling, "basic/with-sampling");
    golden_test!(test_temperature_clamp, "edge-cases/temperature-clamp");
    
    // Test batches by pattern
    golden_test_batch!("basic/*");
}