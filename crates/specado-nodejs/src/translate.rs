//! Translation function implementation
//!
//! This module implements the translate function that converts uniform
//! prompts to provider-specific requests.

use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::types::{PromptSpec, ProviderSpec, TranslateOptions, TranslateResult};

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

        if result != specado_ffi::SpecadoResult::Success {
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

    // Parse the JSON result as TranslationResult from core
    let core_result: specado_core::types::TranslationResult = serde_json::from_str(&result_json)
        .map_err(|e| Error::new(
            Status::GenericFailure,
            format!("Failed to parse translation result: {}", e)
        ))?;

    // Convert to Node.js compatible result
    let translate_result = convert_core_translation_result(core_result, &provider_spec.name, &model_id);

    Ok(translate_result)
}

/// Convert core TranslationResult to Node.js compatible format
fn convert_core_translation_result(
    core_result: specado_core::types::TranslationResult,
    provider_name: &str,
    model_id: &str,
) -> TranslateResult {
    use crate::types::*;
    
    // Convert lossiness report
    let lossiness = LossinessReport {
        items: core_result.lossiness.items.into_iter().map(|item| {
            LossinessItem {
                code: format!("{:?}", item.code),
                path: item.path,
                message: item.message,
                severity: format!("{:?}", item.severity),
                before: item.before,
                after: item.after,
            }
        }).collect(),
        max_severity: format!("{:?}", core_result.lossiness.max_severity),
        summary: LossinessSummary {
            total_items: core_result.lossiness.summary.total_items as u32,
            by_severity: core_result.lossiness.summary.by_severity
                .into_iter()
                .map(|(k, v)| (k, v as u32))
                .collect(),
            by_code: core_result.lossiness.summary.by_code
                .into_iter()
                .map(|(k, v)| (k, v as u32))
                .collect(),
        },
    };
    
    // Convert metadata if present
    let metadata = core_result.metadata.map(|meta| TranslationMetadata {
        source_version: "1.0".to_string(),
        target_provider: provider_name.to_string(),
        target_model: model_id.to_string(),
        timestamp: meta.timestamp,
        features_used: vec![], // Would need to extract from somewhere
        unsupported_features: vec![], // Would need to extract from lossiness
    });
    
    TranslateResult {
        provider_request_json: core_result.provider_request_json,
        lossiness,
        metadata,
    }
}

/// Extract features used from a prompt
#[allow(dead_code)]
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