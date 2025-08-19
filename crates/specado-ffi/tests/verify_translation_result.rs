//! Test to verify FFI returns proper TranslationResult

use specado_ffi::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[test]
fn test_ffi_returns_translation_result() {
    unsafe {
        // Create test inputs with a valid provider spec
        let prompt_json = r#"{
            "prompt": {
                "messages": [{"role": "user", "content": "Hello"}],
                "temperature": 0.7,
                "max_tokens": 100
            }
        }"#;
        
        // Read actual provider spec from golden corpus
        let provider_spec = std::fs::read_to_string(
            "/Users/jfeinblum/code/specado/golden-corpus/providers/openai/openai-provider.json"
        ).expect("Should read golden corpus provider spec");
        
        let prompt_cstr = CString::new(prompt_json).unwrap();
        let provider_cstr = CString::new(provider_spec).unwrap();
        let model_cstr = CString::new("gpt-5").unwrap();
        let mode_cstr = CString::new("standard").unwrap();
        
        let mut out_json: *mut c_char = std::ptr::null_mut();
        
        let result = specado_translate(
            prompt_cstr.as_ptr(),
            provider_cstr.as_ptr(),
            model_cstr.as_ptr(),
            mode_cstr.as_ptr(),
            &mut out_json,
        );
        
        // Check result
        if result != SpecadoResult::Success {
            let error = specado_get_last_error();
            if !error.is_null() {
                let error_str = CStr::from_ptr(error).to_string_lossy();
                panic!("Translation failed: {}", error_str);
            }
            panic!("Translation failed with code: {:?}", result);
        }
        
        assert!(!out_json.is_null(), "Result should not be null");
        
        let result_str = CStr::from_ptr(out_json).to_string_lossy().to_string();
        
        // Parse and verify structure
        let json: serde_json::Value = serde_json::from_str(&result_str)
            .expect("Should parse as JSON");
        
        // Verify it has TranslationResult structure
        assert!(json.get("provider_request_json").is_some(), 
                "Should have provider_request_json field");
        assert!(json.get("lossiness").is_some(), 
                "Should have lossiness field");
        
        // The lossiness should have items array
        let lossiness = json.get("lossiness").unwrap();
        assert!(lossiness.get("items").is_some(), 
                "Lossiness should have items field");
        
        println!("✅ FFI returns proper TranslationResult structure!");
        println!("Result keys: {:?}", json.as_object().unwrap().keys().collect::<Vec<_>>());
        
        // Clean up
        specado_string_free(out_json);
    }
}