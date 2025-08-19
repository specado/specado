//! Test to verify FFI translate returns proper TranslationResult

use specado_ffi::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[test]
fn test_ffi_translate_output() {
    unsafe {
        // Create test inputs
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
        assert_eq!(result, SpecadoResult::Success, "Translation should succeed");
        assert!(!out_json.is_null(), "Result should not be null");
        
        let result_str = CStr::from_ptr(out_json).to_string_lossy().to_string();
        
        // Parse and verify structure
        let json: serde_json::Value = serde_json::from_str(&result_str)
            .expect("Should parse as JSON");
        
        println!("FFI returned JSON keys: {:?}", json.as_object().unwrap().keys().collect::<Vec<_>>());
        
        // Check what structure we got
        if json.get("provider_request_json").is_some() {
            println!("✅ Has provider_request_json (correct TranslationResult)");
            assert!(json.get("lossiness").is_some(), "Should have lossiness field");
            println!("✅ Has lossiness field");
        } else if json.get("success").is_some() {
            panic!("❌ Has simplified format with 'success' field - FFI is not returning TranslationResult!");
        } else {
            panic!("❌ Unknown format: {:?}", json);
        }
        
        // Clean up
        specado_string_free(out_json);
    }
}