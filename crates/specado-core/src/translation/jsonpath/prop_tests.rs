//! Property-based tests for JSONPath functionality
//!
//! These tests verify that JSONPath operations are safe, deterministic,
//! and handle edge cases correctly.

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use serde_json::Value;
    use crate::translation::jsonpath::{JsonPath, JsonPathError};
    
    /// Strategy for generating simple JSON values with controlled depth
    fn json_value_strategy() -> impl Strategy<Value = Value> {
        let leaf = prop_oneof![
            Just(Value::Null),
            any::<bool>().prop_map(Value::Bool),
            any::<i64>().prop_map(|n| Value::Number(n.into())),
            "[a-zA-Z0-9 ]{0,50}".prop_map(Value::String),
        ];
        
        leaf.prop_recursive(
            3,  // max depth
            10, // max size
            5,  // items per collection
            |inner| {
                prop_oneof![
                    proptest::collection::vec(inner.clone(), 0..5).prop_map(Value::Array),
                    proptest::collection::hash_map(
                        "[a-zA-Z_][a-zA-Z0-9_]{0,20}",
                        inner,
                        0..5
                    ).prop_map(|m| Value::Object(m.into_iter().collect())),
                ]
            },
        )
    }
    
    /// Strategy for generating JSONPath expressions
    fn jsonpath_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            // Root paths
            Just("$".to_string()),
            
            // Simple property access
            "[a-zA-Z_][a-zA-Z0-9_]{0,20}".prop_map(|s| format!("$.{}", s)),
            
            // Nested property access
            (
                "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
                "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            ).prop_map(|(a, b)| format!("$.{}.{}", a, b)),
            
            // Array access
            (
                "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
                0usize..10,
            ).prop_map(|(prop, idx)| format!("$.{}[{}]", prop, idx)),
            
            // Wildcard
            "[a-zA-Z_][a-zA-Z0-9_]{0,10}".prop_map(|s| format!("$.{}.*", s)),
            
            // Recursive descent
            "[a-zA-Z_][a-zA-Z0-9_]{0,10}".prop_map(|s| format!("$..{}", s)),
        ]
    }
    
    proptest! {
        /// Property: JSONPath parsing should never panic
        #[test]
        fn prop_jsonpath_parse_never_panics(
            path in jsonpath_strategy()
        ) {
            // Parsing should either succeed or return an error, never panic
            let _ = JsonPath::parse(&path);
        }
        
        /// Property: Valid JSONPath execution should never panic
        #[test]
        fn prop_jsonpath_execute_never_panics(
            path in jsonpath_strategy(),
            json in json_value_strategy()
        ) {
            // Try to parse and execute the path
            if let Ok(jsonpath) = JsonPath::parse(&path) {
                // Execution should either succeed or return an error, never panic
                let _ = jsonpath.execute(&json);
            }
        }
        
        /// Property: JSONPath execution should be deterministic
        #[test]
        fn prop_jsonpath_execution_deterministic(
            path in jsonpath_strategy(),
            json in json_value_strategy()
        ) {
            if let Ok(jsonpath) = JsonPath::parse(&path) {
                // Execute the same path twice on the same JSON
                let result1 = jsonpath.execute(&json);
                let result2 = jsonpath.execute(&json);
                
                // Results should be identical
                match (result1, result2) {
                    (Ok(v1), Ok(v2)) => assert_eq!(v1, v2),
                    (Err(e1), Err(e2)) => assert_eq!(e1.to_string(), e2.to_string()),
                    _ => panic!("Non-deterministic results"),
                }
            }
        }
        
        /// Property: Root path "$" should return the entire document
        #[test]
        fn prop_root_path_returns_whole_document(
            json in json_value_strategy()
        ) {
            let jsonpath = JsonPath::parse("$").expect("$ should always parse");
            let result = jsonpath.execute(&json);
            
            assert!(result.is_ok());
            if let Ok(values) = result {
                assert_eq!(values.len(), 1);
                assert_eq!(values[0], &json);
            }
        }
        
        /// Property: Array indices should be bounds-checked
        #[test]
        fn prop_array_access_bounds_checked(
            arr in proptest::collection::vec(json_value_strategy(), 0..10),
            index in 0usize..20
        ) {
            let json = Value::Array(arr.clone());
            let path = format!("$[{}]", index);
            
            if let Ok(jsonpath) = JsonPath::parse(&path) {
                let result = jsonpath.execute(&json);
                
                if index < arr.len() {
                    // Should succeed and return the element
                    assert!(result.is_ok());
                    if let Ok(values) = result {
                        assert_eq!(values.len(), 1);
                        assert_eq!(values[0], &arr[index]);
                    }
                } else {
                    // Should either return empty or error, but not panic
                    match result {
                        Ok(values) => assert!(values.is_empty()),
                        Err(_) => {} // Error is acceptable
                    }
                }
            }
        }
        
        /// Property: Property access on non-objects should handle gracefully
        #[test]
        fn prop_property_access_type_safe(
            value in json_value_strategy(),
            property in "[a-zA-Z_][a-zA-Z0-9_]{0,20}"
        ) {
            let path = format!("$.{}", property);
            
            if let Ok(jsonpath) = JsonPath::parse(&path) {
                let result = jsonpath.execute(&value);
                
                match &value {
                    Value::Object(map) => {
                        // Should work on objects
                        if let Ok(values) = result {
                            if map.contains_key(&property) {
                                assert_eq!(values.len(), 1);
                                assert_eq!(values[0], &map[&property]);
                            } else {
                                assert!(values.is_empty());
                            }
                        }
                    }
                    _ => {
                        // Should handle non-objects gracefully
                        match result {
                            Ok(values) => assert!(values.is_empty()),
                            Err(_) => {} // Error is acceptable
                        }
                    }
                }
            }
        }
        
        /// Property: Wildcard should return all array elements or object values
        #[test]
        fn prop_wildcard_returns_all_elements(
            json in json_value_strategy()
        ) {
            let path = "$.*";
            
            if let Ok(jsonpath) = JsonPath::parse(path) {
                let result = jsonpath.execute(&json);
                
                if let Ok(values) = result {
                    match &json {
                        Value::Array(arr) => {
                            assert_eq!(values.len(), arr.len());
                            for (i, val) in values.iter().enumerate() {
                                assert_eq!(*val, &arr[i]);
                            }
                        }
                        Value::Object(map) => {
                            assert_eq!(values.len(), map.len());
                        }
                        _ => {
                            // Wildcard on scalar should return empty
                            assert!(values.is_empty());
                        }
                    }
                }
            }
        }
        
        /// Property: Nested path access should be equivalent to step-by-step access
        #[test]
        fn prop_nested_path_consistency(
            prop1 in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            prop2 in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
        ) {
            // Create nested object
            let json = serde_json::json!({
                prop1.clone(): {
                    prop2.clone(): "value"
                }
            });
            
            // Try nested path
            let nested_path = format!("$.{}.{}", prop1, prop2);
            let direct_result = if let Ok(jp) = JsonPath::parse(&nested_path) {
                jp.execute(&json).ok()
            } else {
                None
            };
            
            // Try step-by-step
            let step1_path = format!("$.{}", prop1);
            let step1_result = if let Ok(jp) = JsonPath::parse(&step1_path) {
                jp.execute(&json).ok()
            } else {
                None
            };
            
            let step2_result = if let Some(values) = step1_result {
                if !values.is_empty() {
                    let step2_path = format!("$.{}", prop2);
                    if let Ok(jp) = JsonPath::parse(&step2_path) {
                        jp.execute(values[0]).ok()
                    } else {
                        None
                    }
                } else {
                    Some(vec![])
                }
            } else {
                None
            };
            
            // Results should be consistent
            match (direct_result, step2_result) {
                (Some(d), Some(s)) => {
                    assert_eq!(d.len(), s.len());
                    if !d.is_empty() && !s.is_empty() {
                        assert_eq!(d[0], s[0]);
                    }
                }
                (None, None) => {} // Both failed, that's consistent
                _ => {} // One succeeded, one failed - acceptable for invalid paths
            }
        }
        
        /// Property: Empty path should be invalid
        #[test]
        fn prop_empty_path_invalid(
            json in json_value_strategy()
        ) {
            let result = JsonPath::parse("");
            assert!(result.is_err());
        }
        
        /// Property: Path without $ should be invalid
        #[test]
        fn prop_path_must_start_with_root(
            suffix in "[a-zA-Z_][a-zA-Z0-9_]{0,20}"
        ) {
            let result = JsonPath::parse(&suffix);
            // Should be invalid unless it starts with $ or ..
            if !suffix.starts_with('$') && !suffix.starts_with("..") {
                assert!(result.is_err());
            }
        }
    }
}