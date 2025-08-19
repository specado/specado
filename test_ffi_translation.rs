// Test FFI translation to verify it returns proper TranslationResult
use specado_ffi::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

fn main() {
    unsafe {
        // Create test inputs
        let prompt_json = r#"{
            "prompt": {
                "messages": [{"role": "user", "content": "Hello"}],
                "temperature": 0.7,
                "max_tokens": 100
            }
        }"#;
        
        let provider_spec = r#"{
            "spec_version": "1.0.0",
            "provider": {
                "name": "openai",
                "base_url": "https://api.openai.com",
                "api_version": "v1"
            },
            "models": [{
                "id": "gpt-4",
                "family": "gpt",
                "capabilities": ["chat"],
                "endpoints": {
                    "chat_completion": {
                        "method": "POST",
                        "path": "/v1/chat/completions",
                        "protocol": "http"
                    }
                },
                "input_modes": {
                    "messages": true,
                    "single_text": false,
                    "images": false
                }
            }]
        }"#;
        
        let prompt_cstr = CString::new(prompt_json).unwrap();
        let provider_cstr = CString::new(provider_spec).unwrap();
        let model_cstr = CString::new("gpt-4").unwrap();
        let mode_cstr = CString::new("standard").unwrap();
        
        let mut out_json: *mut c_char = std::ptr::null_mut();
        
        let result = specado_translate(
            prompt_cstr.as_ptr(),
            provider_cstr.as_ptr(),
            model_cstr.as_ptr(),
            mode_cstr.as_ptr(),
            &mut out_json,
        );
        
        if result == SpecadoResult::Success {
            if !out_json.is_null() {
                let result_str = CStr::from_ptr(out_json).to_string_lossy();
                println!("FFI returned JSON:");
                println!("{}", result_str);
                
                // Try to parse as TranslationResult
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(&result_str);
                match parsed {
                    Ok(json) => {
                        println!("\nParsed JSON structure:");
                        println!("Has provider_request_json: {}", json.get("provider_request_json").is_some());
                        println!("Has lossiness: {}", json.get("lossiness").is_some());
                        println!("Has metadata: {}", json.get("metadata").is_some());
                    }
                    Err(e) => println!("Failed to parse JSON: {}", e),
                }
                
                specado_string_free(out_json);
            }
        } else {
            let error = specado_get_last_error();
            if !error.is_null() {
                let error_str = CStr::from_ptr(error).to_string_lossy();
                println!("Error: {}", error_str);
            }
        }
    }
}