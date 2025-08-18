//! Fuzzing target for JSONPath execution
//!
//! This fuzzer tests that JSONPath execution can handle arbitrary paths
//! and documents without panicking, ensuring robust handling of edge cases.

#![no_main]

use libfuzzer_sys::fuzz_target;
use specado_core::translation::jsonpath::JSONPath;
use serde_json::{json, Value};

fuzz_target!(|data: &[u8]| {
    // Split the input into two parts: path and document
    if data.len() < 2 {
        return;
    }
    
    // Use first byte to determine split point
    let split_point = (data[0] as usize) % data.len().max(1);
    let (path_bytes, doc_bytes) = data.split_at(split_point.min(data.len()));
    
    // Try to create a path from the first part
    if let Ok(path_str) = std::str::from_utf8(path_bytes) {
        if let Ok(jsonpath) = JSONPath::parse(path_str) {
            // Create various test documents
            let test_docs = vec![
                json!(null),
                json!(true),
                json!(42),
                json!("string"),
                json!([]),
                json!({}),
                json!([1, 2, 3]),
                json!({"a": 1, "b": 2}),
                json!({"nested": {"deep": {"value": 42}}}),
                json!([[[[[]]]]]),  // Deeply nested arrays
            ];
            
            // Execute the path on each document
            for doc in &test_docs {
                let _ = jsonpath.execute(doc);
            }
            
            // Also try to parse the second part as JSON and execute on it
            if let Ok(json_str) = std::str::from_utf8(doc_bytes) {
                if let Ok(doc) = serde_json::from_str::<Value>(json_str) {
                    let _ = jsonpath.execute(&doc);
                }
            }
        }
    }
});