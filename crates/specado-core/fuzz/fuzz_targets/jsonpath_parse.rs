//! Fuzzing target for JSONPath parsing
//!
//! This fuzzer tests that the JSONPath parser can handle arbitrary input
//! without panicking, ensuring memory safety and robust error handling.

#![no_main]

use libfuzzer_sys::fuzz_target;
use specado_core::translation::jsonpath::JSONPath;

fuzz_target!(|data: &[u8]| {
    // Convert arbitrary bytes to a string
    if let Ok(path_str) = std::str::from_utf8(data) {
        // The parser should handle ANY input without panicking
        // It should either parse successfully or return an error
        let _ = JSONPath::parse(path_str);
    }
    
    // Also test with forced UTF-8 (lossy conversion)
    let lossy_str = String::from_utf8_lossy(data);
    let _ = JSONPath::parse(&lossy_str);
});