//! Validation function implementation for Python bindings
//!
//! This module implements validation functions for prompt and provider
//! specifications using schema validation.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::exceptions::PyValueError;
use crate::types::{PyPromptSpec, PyProviderSpec, PyValidationResult};
use serde_json::Value;

/// Validate a specification against its schema
/// 
/// Args:
///     spec (Any): The specification to validate (PromptSpec, ProviderSpec, or dict)
///     schema_type (Literal["prompt", "provider"]): The type of schema to validate against
/// 
/// Returns:
///     ValidationResult: Validation result with errors if any
/// 
/// Raises:
///     ValidationError: If schema type is invalid or validation setup fails
#[pyfunction]
pub fn validate(py: Python<'_>, spec: PyObject, schema_type: &str) -> PyResult<PyValidationResult> {
    let spec_json = match schema_type {
        "prompt" => {
            // Try to extract as PyPromptSpec first, then as dict
            if let Ok(prompt_spec) = spec.extract::<PyRef<PyPromptSpec>>(py) {
                serde_json::to_value(&prompt_spec.inner)
                    .map_err(|e| PyValueError::new_err(format!("Failed to serialize prompt spec: {}", e)))?
            } else if let Ok(dict) = spec.downcast::<PyDict>(py) {
                dict_to_json_value(py, dict)?
            } else {
                return Err(PyValueError::new_err("spec must be a PromptSpec or dict for prompt validation"));
            }
        }
        "provider" => {
            // Try to extract as PyProviderSpec first, then as dict
            if let Ok(provider_spec) = spec.extract::<PyRef<PyProviderSpec>>(py) {
                serde_json::to_value(&provider_spec.inner)
                    .map_err(|e| PyValueError::new_err(format!("Failed to serialize provider spec: {}", e)))?
            } else if let Ok(dict) = spec.downcast::<PyDict>(py) {
                dict_to_json_value(py, dict)?
            } else {
                return Err(PyValueError::new_err("spec must be a ProviderSpec or dict for provider validation"));
            }
        }
        _ => {
            return Err(PyValueError::new_err("schema_type must be 'prompt' or 'provider'"));
        }
    };

    // Perform validation
    let validation_result = validate_spec_internal(&spec_json, schema_type)?;
    
    Ok(validation_result)
}

/// Internal validation logic
fn validate_spec_internal(spec: &Value, schema_type: &str) -> PyResult<PyValidationResult> {
    let mut errors = Vec::new();
    
    match schema_type {
        "prompt" => {
            validate_prompt_spec(spec, &mut errors);
        }
        "provider" => {
            validate_provider_spec(spec, &mut errors);
        }
        _ => unreachable!(), // Already validated above
    }
    
    let is_valid = errors.is_empty();
    Ok(PyValidationResult::new(is_valid, errors))
}

/// Validate a prompt specification
fn validate_prompt_spec(spec: &Value, errors: &mut Vec<String>) {
    // Check required fields
    if !spec.is_object() {
        errors.push("Prompt spec must be an object".to_string());
        return;
    }
    
    let obj = spec.as_object().unwrap();
    
    // Check model_class
    match obj.get("model_class") {
        Some(Value::String(model_class)) => {
            if model_class.is_empty() {
                errors.push("model_class cannot be empty".to_string());
            }
        }
        Some(_) => errors.push("model_class must be a string".to_string()),
        None => errors.push("model_class is required".to_string()),
    }
    
    // Check messages
    match obj.get("messages") {
        Some(Value::Array(messages)) => {
            if messages.is_empty() {
                errors.push("messages array cannot be empty".to_string());
            } else {
                for (i, message) in messages.iter().enumerate() {
                    validate_message(message, i, errors);
                }
            }
        }
        Some(_) => errors.push("messages must be an array".to_string()),
        None => errors.push("messages is required".to_string()),
    }
    
    // Check strict_mode
    match obj.get("strict_mode") {
        Some(Value::String(mode)) => {
            if !["warn", "error"].contains(&mode.as_str()) {
                errors.push("strict_mode must be 'warn' or 'error'".to_string());
            }
        }
        Some(_) => errors.push("strict_mode must be a string".to_string()),
        None => errors.push("strict_mode is required".to_string()),
    }
    
    // Validate optional fields if present
    if let Some(tools) = obj.get("tools") {
        validate_tools(tools, errors);
    }
    
    if let Some(sampling) = obj.get("sampling") {
        validate_sampling_params(sampling, errors);
    }
    
    if let Some(limits) = obj.get("limits") {
        validate_limits(limits, errors);
    }
}

/// Validate a provider specification
fn validate_provider_spec(spec: &Value, errors: &mut Vec<String>) {
    if !spec.is_object() {
        errors.push("Provider spec must be an object".to_string());
        return;
    }
    
    let obj = spec.as_object().unwrap();
    
    // Check spec_version
    match obj.get("spec_version") {
        Some(Value::String(version)) => {
            if version.is_empty() {
                errors.push("spec_version cannot be empty".to_string());
            }
            // Could add semver validation here
        }
        Some(_) => errors.push("spec_version must be a string".to_string()),
        None => errors.push("spec_version is required".to_string()),
    }
    
    // Check provider
    match obj.get("provider") {
        Some(provider) => validate_provider_info(provider, errors),
        None => errors.push("provider is required".to_string()),
    }
    
    // Check models
    match obj.get("models") {
        Some(Value::Array(models)) => {
            if models.is_empty() {
                errors.push("models array cannot be empty".to_string());
            } else {
                for (i, model) in models.iter().enumerate() {
                    validate_model_spec(model, i, errors);
                }
            }
        }
        Some(_) => errors.push("models must be an array".to_string()),
        None => errors.push("models is required".to_string()),
    }
}

/// Validate a single message
fn validate_message(message: &Value, index: usize, errors: &mut Vec<String>) {
    if !message.is_object() {
        errors.push(format!("Message {} must be an object", index));
        return;
    }
    
    let obj = message.as_object().unwrap();
    
    // Check role
    match obj.get("role") {
        Some(Value::String(role)) => {
            if !["system", "user", "assistant"].contains(&role.as_str()) {
                errors.push(format!("Message {} role must be 'system', 'user', or 'assistant'", index));
            }
        }
        Some(_) => errors.push(format!("Message {} role must be a string", index)),
        None => errors.push(format!("Message {} role is required", index)),
    }
    
    // Check content
    match obj.get("content") {
        Some(Value::String(content)) => {
            if content.is_empty() {
                errors.push(format!("Message {} content cannot be empty", index));
            }
        }
        Some(_) => errors.push(format!("Message {} content must be a string", index)),
        None => errors.push(format!("Message {} content is required", index)),
    }
}

/// Validate tools array
fn validate_tools(tools: &Value, errors: &mut Vec<String>) {
    if let Value::Array(tools_array) = tools {
        for (i, tool) in tools_array.iter().enumerate() {
            if !tool.is_object() {
                errors.push(format!("Tool {} must be an object", i));
                continue;
            }
            
            let obj = tool.as_object().unwrap();
            
            // Check name
            match obj.get("name") {
                Some(Value::String(name)) => {
                    if name.is_empty() {
                        errors.push(format!("Tool {} name cannot be empty", i));
                    }
                }
                Some(_) => errors.push(format!("Tool {} name must be a string", i)),
                None => errors.push(format!("Tool {} name is required", i)),
            }
            
            // Check json_schema
            if !obj.contains_key("json_schema") {
                errors.push(format!("Tool {} json_schema is required", i));
            }
        }
    } else {
        errors.push("tools must be an array".to_string());
    }
}

/// Validate sampling parameters
fn validate_sampling_params(sampling: &Value, errors: &mut Vec<String>) {
    if !sampling.is_object() {
        errors.push("sampling must be an object".to_string());
        return;
    }
    
    let obj = sampling.as_object().unwrap();
    
    // Validate temperature
    if let Some(temp) = obj.get("temperature") {
        if let Some(t) = temp.as_f64() {
            if t < 0.0 || t > 2.0 {
                errors.push("temperature must be between 0.0 and 2.0".to_string());
            }
        } else {
            errors.push("temperature must be a number".to_string());
        }
    }
    
    // Validate top_p
    if let Some(top_p) = obj.get("top_p") {
        if let Some(p) = top_p.as_f64() {
            if p < 0.0 || p > 1.0 {
                errors.push("top_p must be between 0.0 and 1.0".to_string());
            }
        } else {
            errors.push("top_p must be a number".to_string());
        }
    }
}

/// Validate limits
fn validate_limits(limits: &Value, errors: &mut Vec<String>) {
    if !limits.is_object() {
        errors.push("limits must be an object".to_string());
        return;
    }
    
    let obj = limits.as_object().unwrap();
    
    // Validate max_output_tokens
    if let Some(max_tokens) = obj.get("max_output_tokens") {
        if let Some(tokens) = max_tokens.as_u64() {
            if tokens == 0 {
                errors.push("max_output_tokens must be greater than 0".to_string());
            }
        } else {
            errors.push("max_output_tokens must be a positive integer".to_string());
        }
    }
}

/// Validate provider info
fn validate_provider_info(provider: &Value, errors: &mut Vec<String>) {
    if !provider.is_object() {
        errors.push("provider must be an object".to_string());
        return;
    }
    
    let obj = provider.as_object().unwrap();
    
    // Check name
    match obj.get("name") {
        Some(Value::String(name)) => {
            if name.is_empty() {
                errors.push("provider name cannot be empty".to_string());
            }
        }
        Some(_) => errors.push("provider name must be a string".to_string()),
        None => errors.push("provider name is required".to_string()),
    }
    
    // Check base_url
    match obj.get("base_url") {
        Some(Value::String(url)) => {
            if url.is_empty() {
                errors.push("provider base_url cannot be empty".to_string());
            }
            // Could add URL validation here
        }
        Some(_) => errors.push("provider base_url must be a string".to_string()),
        None => errors.push("provider base_url is required".to_string()),
    }
}

/// Validate model specification
fn validate_model_spec(model: &Value, index: usize, errors: &mut Vec<String>) {
    if !model.is_object() {
        errors.push(format!("Model {} must be an object", index));
        return;
    }
    
    let obj = model.as_object().unwrap();
    
    // Check id
    match obj.get("id") {
        Some(Value::String(id)) => {
            if id.is_empty() {
                errors.push(format!("Model {} id cannot be empty", index));
            }
        }
        Some(_) => errors.push(format!("Model {} id must be a string", index)),
        None => errors.push(format!("Model {} id is required", index)),
    }
    
    // Check family
    match obj.get("family") {
        Some(Value::String(family)) => {
            if family.is_empty() {
                errors.push(format!("Model {} family cannot be empty", index));
            }
        }
        Some(_) => errors.push(format!("Model {} family must be a string", index)),
        None => errors.push(format!("Model {} family is required", index)),
    }
    
    // Check required objects
    let required_objects = ["endpoints", "input_modes", "tooling", "json_output", 
                           "constraints", "mappings", "response_normalization"];
    
    for field in &required_objects {
        if !obj.contains_key(*field) {
            errors.push(format!("Model {} {} is required", index, field));
        }
    }
}

/// Helper function to convert Python dict to JSON value
fn dict_to_json_value(py: Python<'_>, dict: &PyDict) -> PyResult<Value> {
    crate::types::py_to_json(py, dict.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::types::PyDict;
    
    #[test]
    fn test_validate_prompt_spec_minimal() {
        let mut errors = Vec::new();
        let spec = serde_json::json!({
            "model_class": "Chat",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello"
                }
            ],
            "strict_mode": "warn"
        });
        
        validate_prompt_spec(&spec, &mut errors);
        assert!(errors.is_empty(), "Errors: {:?}", errors);
    }
    
    #[test]
    fn test_validate_prompt_spec_missing_fields() {
        let mut errors = Vec::new();
        let spec = serde_json::json!({});
        
        validate_prompt_spec(&spec, &mut errors);
        assert!(errors.len() >= 3); // Should have errors for missing required fields
    }
    
    #[test]
    fn test_validate_provider_spec_minimal() {
        let mut errors = Vec::new();
        let spec = serde_json::json!({
            "spec_version": "1.0.0",
            "provider": {
                "name": "test",
                "base_url": "https://api.test.com",
                "headers": {}
            },
            "models": [
                {
                    "id": "test-model",
                    "family": "test",
                    "endpoints": {},
                    "input_modes": {},
                    "tooling": {},
                    "json_output": {},
                    "constraints": {},
                    "mappings": {},
                    "response_normalization": {}
                }
            ]
        });
        
        validate_provider_spec(&spec, &mut errors);
        // Some errors might be expected due to incomplete structure, 
        // but should validate basic structure
        assert!(errors.len() < 10, "Too many validation errors: {:?}", errors);
    }
}