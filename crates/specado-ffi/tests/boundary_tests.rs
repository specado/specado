//! FFI boundary tests
//!
//! These tests verify the safety and correctness of the FFI layer,
//! including null pointer handling, memory management, and error propagation.

use specado_ffi::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

/// Helper to convert Rust string to C string
fn to_c_string(s: &str) -> CString {
    CString::new(s).unwrap()
}

/// Helper to convert C string pointer to Rust string
unsafe fn from_c_string(s: *const c_char) -> String {
    if s.is_null() {
        String::new()
    } else {
        CStr::from_ptr(s).to_string_lossy().into_owned()
    }
}

#[test]
fn test_null_pointer_handling() {
    unsafe {
        let mut output: *mut c_char = ptr::null_mut();
        
        // Test with null inputs
        let result = specado_translate(
            ptr::null(),
            ptr::null(),
            ptr::null(),
            ptr::null(),
            &mut output,
        );
        
        // Should return error for null pointer
        assert_ne!(result, SpecadoResult::Success);
        assert_eq!(result, SpecadoResult::NullPointer);
        
        // Error message should be set
        let error = specado_get_last_error();
        assert!(!error.is_null());
        
        // Clear error
        specado_clear_error();
        let error = specado_get_last_error();
        assert!(error.is_null());
    }
}

#[test]
fn test_invalid_utf8_handling() {
    unsafe {
        let mut output: *mut c_char = ptr::null_mut();
        
        // Create invalid UTF-8 sequence
        let invalid_utf8 = [0xFF, 0xFE, 0x00];
        let invalid_ptr = invalid_utf8.as_ptr() as *const c_char;
        
        let valid_json = to_c_string("{}");
        let model_id = to_c_string("test-model");
        let mode = to_c_string("standard");
        
        let result = specado_translate(
            invalid_ptr,
            valid_json.as_ptr(),
            model_id.as_ptr(),
            mode.as_ptr(),
            &mut output,
        );
        
        // Should return UTF-8 error
        assert_ne!(result, SpecadoResult::Success);
    }
}

#[test]
fn test_memory_allocation_and_deallocation() {
    unsafe {
        let mut output: *mut c_char = ptr::null_mut();
        
        // Create valid inputs
        let prompt = to_c_string(r#"{"prompt": {"messages": [{"role": "user", "content": "test"}]}}"#);
        let provider_spec = to_c_string(r#"{
            "spec_version": "1.0.0",
            "provider": {
                "name": "test-provider",
                "base_url": "https://api.test.com"
            },
            "models": [{
                "id": "test-model",
                "family": "test",
                "endpoints": {
                    "chat_completion": {
                        "method": "POST",
                        "path": "/v1/chat",
                        "protocol": "http"
                    },
                    "streaming_chat_completion": {
                        "method": "POST",
                        "path": "/v1/chat",
                        "protocol": "sse"
                    }
                },
                "input_modes": {
                    "messages": true,
                    "single_text": false,
                    "images": false
                }
            }]
        }"#);
        let model_id = to_c_string("test-model");
        let mode = to_c_string("standard");
        
        // Call translate
        let result = specado_translate(
            prompt.as_ptr(),
            provider_spec.as_ptr(),
            model_id.as_ptr(),
            mode.as_ptr(),
            &mut output,
        );
        
        if result == SpecadoResult::Success {
            // Output should be allocated
            assert!(!output.is_null());
            
            // Should be valid UTF-8
            let output_str = from_c_string(output);
            assert!(!output_str.is_empty());
            
            // Free the allocated string
            specado_string_free(output);
            
            // After freeing, we shouldn't access it
            // (In a real test, we'd use valgrind to verify no leaks)
        }
    }
}

#[test]
fn test_context_lifecycle() {
    unsafe {
        // Create context
        let ctx = specado_context_new();
        assert!(!ctx.is_null());
        
        // Free context
        specado_context_free(ctx);
        
        // Double-free should be safe (no-op)
        specado_context_free(ptr::null_mut());
    }
}

#[test]
fn test_error_propagation() {
    unsafe {
        let mut output: *mut c_char = ptr::null_mut();
        
        // Invalid JSON should cause error
        let invalid_json = to_c_string("not valid json");
        let provider_spec = to_c_string("{}");
        let model_id = to_c_string("test");
        let mode = to_c_string("standard");
        
        let result = specado_translate(
            invalid_json.as_ptr(),
            provider_spec.as_ptr(),
            model_id.as_ptr(),
            mode.as_ptr(),
            &mut output,
        );
        
        // Should return JSON error
        assert_eq!(result, SpecadoResult::JsonError);
        
        // Error message should be available
        let error = specado_get_last_error();
        assert!(!error.is_null());
        let error_msg = from_c_string(error);
        assert!(error_msg.contains("JSON") || error_msg.contains("parse"));
    }
}

#[test]
fn test_version_string() {
    unsafe {
        let version = specado_version();
        assert!(!version.is_null());
        
        let version_str = from_c_string(version);
        assert!(version_str.contains("specado"));
        
        // Version string should NOT be freed (it's static)
    }
}

#[test]
fn test_concurrent_access() {
        use std::thread;
    
    let threads: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                unsafe {
                    let mut output: *mut c_char = ptr::null_mut();
                    
                    // Each thread makes a request
                    let prompt = to_c_string(&format!(
                        r#"{{"prompt": {{"messages": [{{"role": "user", "content": "test{}"}}]}}}}"#,
                        i
                    ));
                    let provider_spec = to_c_string(r#"{"spec_version": "1.0.0", "provider": {"name": "test", "base_url": "http://test"}, "models": []}"#);
                    let model_id = to_c_string("model");
                    let mode = to_c_string("standard");
                    
                    let result = specado_translate(
                        prompt.as_ptr(),
                        provider_spec.as_ptr(),
                        model_id.as_ptr(),
                        mode.as_ptr(),
                        &mut output,
                    );
                    
                    // Each thread should have its own error state
                    if result != SpecadoResult::Success {
                        let error = specado_get_last_error();
                        if !error.is_null() {
                            let _error_msg = from_c_string(error);
                            // Thread-local error should be independent
                        }
                    }
                    
                    if !output.is_null() {
                        specado_string_free(output);
                    }
                }
            })
        })
        .collect();
    
    // Wait for all threads
    for t in threads {
        t.join().unwrap();
    }
}

#[cfg(test)]
mod memory_tests {
    use super::*;
    
    #[test]
    fn test_large_input_handling() {
        unsafe {
            let mut output: *mut c_char = ptr::null_mut();
            
            // Create a large input (1MB)
            let large_content = "x".repeat(1024 * 1024);
            let prompt = to_c_string(&format!(
                r#"{{"prompt": {{"messages": [{{"role": "user", "content": "{}"}}]}}}}"#,
                large_content
            ));
            let provider_spec = to_c_string(r#"{"spec_version": "1.0.0", "provider": {"name": "test", "base_url": "http://test"}, "models": []}"#);
            let model_id = to_c_string("model");
            let mode = to_c_string("standard");
            
            let result = specado_translate(
                prompt.as_ptr(),
                provider_spec.as_ptr(),
                model_id.as_ptr(),
                mode.as_ptr(),
                &mut output,
            );
            
            // Should handle large inputs gracefully
            // (May fail with ModelNotFound, but shouldn't crash)
            assert_ne!(result, SpecadoResult::MemoryError);
            
            if !output.is_null() {
                specado_string_free(output);
            }
        }
    }
    
    #[test]
    fn test_repeated_allocations() {
        // Test repeated allocations and deallocations
        for _ in 0..100 {
            unsafe {
                let ctx = specado_context_new();
                assert!(!ctx.is_null());
                specado_context_free(ctx);
            }
        }
    }
}