//! Fuzzing target for memory safety with large JSONPath operations
//!
//! This fuzzer tests memory safety with large paths and documents,
//! checking for memory leaks and buffer overflows.

#![no_main]

use libfuzzer_sys::fuzz_target;
use specado_core::translation::jsonpath::JSONPath;
use serde_json::{json, Value};

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    
    let path_str = String::from_utf8_lossy(data);
    
    // Test with extremely large documents
    let large_array: Vec<Value> = (0..1000).map(|i| json!(i)).collect();
    let large_object: serde_json::Map<String, Value> = (0..100)
        .map(|i| (format!("key_{}", i), json!(i)))
        .collect();
    
    let test_docs = vec![
        json!(large_array),
        Value::Object(large_object.clone()),
        json!({
            "deeply": {
                "nested": {
                    "structure": {
                        "with": {
                            "many": {
                                "levels": {
                                    "value": 42
                                }
                            }
                        }
                    }
                }
            }
        }),
    ];
    
    // Try parsing the path
    if let Ok(jsonpath) = JSONPath::parse(&path_str) {
        for doc in &test_docs {
            // Execute should handle large documents without issues
            let _ = jsonpath.execute(doc);
        }
    }
    
    // Test with paths that could cause exponential growth
    let recursive_patterns = vec![
        "$..*..*",           // Multiple recursive descents
        "$..*[*]..*",        // Recursive with wildcards
        "$[*][*][*]",        // Multiple wildcards
    ];
    
    for pattern in &recursive_patterns {
        if let Ok(jsonpath) = JSONPath::parse(pattern) {
            // Create a document that could trigger exponential behavior
            let nested_doc = json!({
                "a": {"b": {"c": {"d": {"e": {"f": 1}}}}},
                "x": [
                    {"y": [1, 2, 3]},
                    {"y": [4, 5, 6]},
                    {"y": [7, 8, 9]}
                ]
            });
            
            // Should complete without excessive memory usage
            let _ = jsonpath.execute(&nested_doc);
        }
    }
    
    // Test circular reference detection (if applicable)
    // Note: serde_json doesn't support circular references, but we test the concept
    let circular_like = json!({
        "ref": "$",
        "self": "$.ref",
        "loop": "$.loop"
    });
    
    if let Ok(jsonpath) = JSONPath::parse(&path_str) {
        let _ = jsonpath.execute(&circular_like);
    }
});