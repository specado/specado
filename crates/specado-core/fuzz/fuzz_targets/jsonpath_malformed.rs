//! Fuzzing target for malformed JSONPath expressions
//!
//! This fuzzer specifically targets edge cases and malformed inputs
//! to ensure the parser handles invalid syntax gracefully.

#![no_main]

use libfuzzer_sys::fuzz_target;
use specado_core::translation::jsonpath::JSONPath;
use serde_json::json;

fuzz_target!(|data: &[u8]| {
    // Create potentially malformed paths by manipulating the input
    let base_str = String::from_utf8_lossy(data);
    
    // Test various malformed patterns
    let malformed_patterns = vec![
        format!("${}", base_str),                    // Root with suffix
        format!("$[{}]", base_str),                  // Array access
        format!("$.{}", base_str),                   // Property access
        format!("$..{}", base_str),                  // Recursive descent
        format!("$[*].{}", base_str),                // Wildcard with property
        format!("$['{}']", base_str),                // Quoted property
        format!("$[{}:{}]", base_str, base_str),     // Slice notation
        format!("$[?(@.{} == {})]", base_str, base_str), // Filter expression
        format!("${{{}}}{{{}}}[[[{}]]]", base_str, base_str, base_str), // Nested brackets
        format!("${}{}", ".".repeat(1000), base_str),  // Deeply nested path
        format!("$['\\x00\\xFF{}']", base_str),      // Special characters
        format!("$[{}{}{}{}]", base_str, base_str, base_str, base_str), // Repeated content
    ];
    
    for pattern in &malformed_patterns {
        // Parser should handle these without panicking
        match JSONPath::parse(pattern) {
            Ok(path) => {
                // If it parses, try to execute it
                let test_doc = json!({
                    "test": [1, 2, 3],
                    "nested": {"value": 42},
                    "array": [[[]]]
                });
                let _ = path.execute(&test_doc);
            }
            Err(_) => {
                // Error is expected for malformed input
            }
        }
    }
    
    // Test extreme cases
    if !base_str.is_empty() {
        // Very long paths
        let long_path = format!("${}", ".a".repeat(1000));
        let _ = JSONPath::parse(&long_path);
        
        // Many array indices
        let many_indices = format!("${}]", "[0".repeat(100));
        let _ = JSONPath::parse(&many_indices);
        
        // Unicode and special characters
        let unicode_path = format!("$.{}\u{1F600}\u{0000}", base_str);
        let _ = JSONPath::parse(&unicode_path);
    }
});