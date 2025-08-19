//! Translation function implementation
//!
//! This module implements the translate function that converts uniform
//! prompts to provider-specific requests.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::Value;

use crate::error::SpecadoError;
use crate::types::{PromptSpec, ProviderSpec, TranslateOptions, TranslateResult, TranslationMetadata};
use chrono;

/// Translate a uniform prompt to a provider-specific request
///
/// # Arguments
/// * `prompt` - The prompt specification to translate
/// * `provider_spec` - The target provider specification
/// * `model_id` - The target model identifier
/// * `options` - Optional translation options
///
/// # Returns
/// A `TranslateResult` containing the translated request and metadata
///
/// # Example
/// ```typescript
/// import { translate } from '@specado/nodejs';
/// 
/// const prompt = {
///   model_class: "Chat",
///   messages: [
///     { role: "user", content: "Hello, world!" }
///   ],
///   strict_mode: "standard"
/// };
/// 
/// const result = translate(prompt, providerSpec, "gpt-4", { mode: "standard" });
/// console.log(result.request);
/// ```
#[napi]
pub fn translate(
    prompt: PromptSpec,
    provider_spec: ProviderSpec,
    model_id: String,
    options: Option<TranslateOptions>,
) -> Result<TranslateResult> {
    // Set default options
    let opts = options.unwrap_or(TranslateOptions {
        mode: Some("standard".to_string()),
        include_metadata: Some(true),
        custom_rules: None,
    });

    // Serialize inputs to JSON for FFI layer
    let prompt_json = serde_json::to_string(&prompt)
        .map_err(|e| Error::new(
            Status::InvalidArg,
            format!("Failed to serialize prompt: {}", e)
        ))?;

    let provider_json = serde_json::to_string(&provider_spec)
        .map_err(|e| Error::new(
            Status::InvalidArg,
            format!("Failed to serialize provider spec: {}", e)
        ))?;

    let mode = opts.mode.unwrap_or_else(|| "standard".to_string());

    // Call the FFI translation function
    let result_json = unsafe {
        let mut out_ptr: *mut std::os::raw::c_char = std::ptr::null_mut();
        
        let prompt_cstr = std::ffi::CString::new(prompt_json)
            .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid prompt JSON: {}", e)))?;
        let provider_cstr = std::ffi::CString::new(provider_json)
            .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid provider JSON: {}", e)))?;
        let model_cstr = std::ffi::CString::new(model_id.clone())
            .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid model ID: {}", e)))?;
        let mode_cstr = std::ffi::CString::new(mode.clone())
            .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid mode: {}", e)))?;

        let result = specado_ffi::specado_translate(
            prompt_cstr.as_ptr(),
            provider_cstr.as_ptr(),
            model_cstr.as_ptr(),
            mode_cstr.as_ptr(),
            &mut out_ptr,
        );

        if result != specado_ffi::types::SpecadoResult::Success {
            // Get error message from FFI
            let error_msg = specado_ffi::specado_get_last_error();
            let error_str = if error_msg.is_null() {
                "Translation failed".to_string()
            } else {
                std::ffi::CStr::from_ptr(error_msg)
                    .to_string_lossy()
                    .to_string()
            };
            
            return Err(Error::new(Status::GenericFailure, error_str));
        }

        if out_ptr.is_null() {
            return Err(Error::new(Status::GenericFailure, "Translation returned null result"));
        }

        let result_cstr = std::ffi::CStr::from_ptr(out_ptr);
        let result_string = result_cstr.to_string_lossy().to_string();
        
        // Free the allocated string
        specado_ffi::specado_string_free(out_ptr);
        
        result_string
    };

    // Parse the JSON result
    let result_value: Value = serde_json::from_str(&result_json)
        .map_err(|e| Error::new(
            Status::GenericFailure,
            format!("Failed to parse translation result: {}", e)
        ))?;

    // Extract the translated request
    let request = result_value.get("request")
        .ok_or_else(|| Error::new(Status::GenericFailure, "Missing request in translation result"))?
        .clone();

    // Build metadata
    let metadata = TranslationMetadata {
        source_version: "1.0".to_string(),
        target_provider: provider_spec.name.clone(),
        target_model: model_id,
        timestamp: chrono::Utc::now().to_rfc3339(),
        features_used: extract_features_used(&prompt),
        unsupported_features: result_value
            .get("unsupported_features")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default(),
    };

    // Extract warnings
    let warnings = result_value
        .get("warnings")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    Ok(TranslateResult {
        request,
        metadata,
        warnings,
    })
}

/// Extract features used from a prompt
fn extract_features_used(prompt: &PromptSpec) -> Vec<String> {
    let mut features = vec!["messages".to_string()];

    if prompt.tools.is_some() {
        features.push("tools".to_string());
    }
    if prompt.tool_choice.is_some() {
        features.push("tool_choice".to_string());
    }
    if prompt.response_format.is_some() {
        features.push("response_format".to_string());
    }
    if prompt.sampling.is_some() {
        features.push("sampling".to_string());
    }
    if prompt.limits.is_some() {
        features.push("limits".to_string());
    }
    if prompt.media.is_some() {
        features.push("media".to_string());
    }

    features
}