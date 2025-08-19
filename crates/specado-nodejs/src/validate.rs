//! Validation function implementation
//!
//! This module implements validation functions for prompt and provider specifications.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::Value;

use crate::error::SpecadoError;
use crate::types::{ValidationResult, ValidationError, ValidationWarning};

/// Schema types for validation
#[napi]
pub enum SchemaType {
    /// Prompt specification schema
    Prompt,
    /// Provider specification schema
    Provider,
}

/// Validate a specification against its schema
///
/// # Arguments
/// * `spec` - The specification to validate (as JSON object)
/// * `schema_type` - The type of schema to validate against
///
/// # Returns
/// A `ValidationResult` containing validation status, errors, and warnings
///
/// # Example
/// ```typescript
/// import { validate, SchemaType } from '@specado/nodejs';
/// 
/// const prompt = {
///   model_class: "Chat",
///   messages: [
///     { role: "user", content: "Hello!" }
///   ],
///   strict_mode: "standard"
/// };
/// 
/// const result = validate(prompt, SchemaType.Prompt);
/// if (result.valid) {
///   console.log("Prompt is valid!");
/// } else {
///   console.error("Validation errors:", result.errors);
/// }
/// ```
#[napi]
pub fn validate(spec: Value, schema_type: SchemaType) -> Result<ValidationResult> {
    // Convert schema type to string
    let schema_type_str = match schema_type {
        SchemaType::Prompt => "prompt",
        SchemaType::Provider => "provider",
    };

    // Serialize the spec to JSON string
    let spec_json = serde_json::to_string(&spec)
        .map_err(|e| Error::new(
            Status::InvalidArg,
            format!("Failed to serialize spec: {}", e)
        ))?;

    // Call the validation function
    let result = validate_internal(&spec_json, schema_type_str)?;

    Ok(result)
}

/// Internal validation function that calls the core validation logic
fn validate_internal(spec_json: &str, schema_type: &str) -> Result<ValidationResult> {
    // For now, implement basic validation
    // In a full implementation, this would use the core validation logic
    
    let spec_value: Value = serde_json::from_str(spec_json)
        .map_err(|e| Error::new(
            Status::InvalidArg,
            format!("Invalid JSON: {}", e)
        ))?;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    match schema_type {
        "prompt" => validate_prompt_spec(&spec_value, &mut errors, &mut warnings),
        "provider" => validate_provider_spec(&spec_value, &mut errors, &mut warnings),
        _ => {
            errors.push(ValidationError {
                path: "".to_string(),
                message: format!("Unknown schema type: {}", schema_type),
                code: "UNKNOWN_SCHEMA".to_string(),
            });
        }
    }

    Ok(ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
        schema_version: "1.0".to_string(),
    })
}

/// Validate a prompt specification
fn validate_prompt_spec(spec: &Value, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>) {
    // Check required fields
    if !spec.is_object() {
        errors.push(ValidationError {
            path: "".to_string(),
            message: "Prompt spec must be an object".to_string(),
            code: "INVALID_TYPE".to_string(),
        });
        return;
    }

    let obj = spec.as_object().unwrap();

    // Check model_class
    if !obj.contains_key("model_class") {
        errors.push(ValidationError {
            path: "model_class".to_string(),
            message: "Missing required field 'model_class'".to_string(),
            code: "MISSING_FIELD".to_string(),
        });
    } else if !obj["model_class"].is_string() {
        errors.push(ValidationError {
            path: "model_class".to_string(),
            message: "Field 'model_class' must be a string".to_string(),
            code: "INVALID_TYPE".to_string(),
        });
    }

    // Check messages
    if !obj.contains_key("messages") {
        errors.push(ValidationError {
            path: "messages".to_string(),
            message: "Missing required field 'messages'".to_string(),
            code: "MISSING_FIELD".to_string(),
        });
    } else if !obj["messages"].is_array() {
        errors.push(ValidationError {
            path: "messages".to_string(),
            message: "Field 'messages' must be an array".to_string(),
            code: "INVALID_TYPE".to_string(),
        });
    } else {
        validate_messages(&obj["messages"], errors, warnings);
    }

    // Check strict_mode
    if !obj.contains_key("strict_mode") {
        errors.push(ValidationError {
            path: "strict_mode".to_string(),
            message: "Missing required field 'strict_mode'".to_string(),
            code: "MISSING_FIELD".to_string(),
        });
    } else if !obj["strict_mode"].is_string() {
        errors.push(ValidationError {
            path: "strict_mode".to_string(),
            message: "Field 'strict_mode' must be a string".to_string(),
            code: "INVALID_TYPE".to_string(),
        });
    } else {
        let strict_mode = obj["strict_mode"].as_str().unwrap();
        if strict_mode != "standard" && strict_mode != "strict" {
            errors.push(ValidationError {
                path: "strict_mode".to_string(),
                message: "Field 'strict_mode' must be 'standard' or 'strict'".to_string(),
                code: "INVALID_VALUE".to_string(),
            });
        }
    }

    // Validate optional fields
    if let Some(tools) = obj.get("tools") {
        if !tools.is_array() {
            errors.push(ValidationError {
                path: "tools".to_string(),
                message: "Field 'tools' must be an array".to_string(),
                code: "INVALID_TYPE".to_string(),
            });
        } else {
            validate_tools(tools, errors, warnings);
        }
    }

    if let Some(sampling) = obj.get("sampling") {
        if !sampling.is_object() {
            errors.push(ValidationError {
                path: "sampling".to_string(),
                message: "Field 'sampling' must be an object".to_string(),
                code: "INVALID_TYPE".to_string(),
            });
        } else {
            validate_sampling(sampling, errors, warnings);
        }
    }
}

/// Validate messages array
fn validate_messages(messages: &Value, errors: &mut Vec<ValidationError>, _warnings: &mut Vec<ValidationWarning>) {
    let messages_array = messages.as_array().unwrap();
    
    if messages_array.is_empty() {
        errors.push(ValidationError {
            path: "messages".to_string(),
            message: "Messages array cannot be empty".to_string(),
            code: "EMPTY_ARRAY".to_string(),
        });
        return;
    }

    for (i, message) in messages_array.iter().enumerate() {
        let path = format!("messages[{}]", i);
        
        if !message.is_object() {
            errors.push(ValidationError {
                path: path.clone(),
                message: "Message must be an object".to_string(),
                code: "INVALID_TYPE".to_string(),
            });
            continue;
        }

        let msg_obj = message.as_object().unwrap();

        // Check role
        if !msg_obj.contains_key("role") {
            errors.push(ValidationError {
                path: format!("{}.role", path),
                message: "Missing required field 'role'".to_string(),
                code: "MISSING_FIELD".to_string(),
            });
        } else if !msg_obj["role"].is_string() {
            errors.push(ValidationError {
                path: format!("{}.role", path),
                message: "Field 'role' must be a string".to_string(),
                code: "INVALID_TYPE".to_string(),
            });
        }

        // Check content
        if !msg_obj.contains_key("content") {
            errors.push(ValidationError {
                path: format!("{}.content", path),
                message: "Missing required field 'content'".to_string(),
                code: "MISSING_FIELD".to_string(),
            });
        } else if !msg_obj["content"].is_string() {
            errors.push(ValidationError {
                path: format!("{}.content", path),
                message: "Field 'content' must be a string".to_string(),
                code: "INVALID_TYPE".to_string(),
            });
        }
    }
}

/// Validate tools array
fn validate_tools(tools: &Value, errors: &mut Vec<ValidationError>, _warnings: &mut Vec<ValidationWarning>) {
    let tools_array = tools.as_array().unwrap();
    
    for (i, tool) in tools_array.iter().enumerate() {
        let path = format!("tools[{}]", i);
        
        if !tool.is_object() {
            errors.push(ValidationError {
                path: path.clone(),
                message: "Tool must be an object".to_string(),
                code: "INVALID_TYPE".to_string(),
            });
            continue;
        }

        let tool_obj = tool.as_object().unwrap();

        // Check required fields
        for field in &["name", "description", "parameters"] {
            if !tool_obj.contains_key(*field) {
                errors.push(ValidationError {
                    path: format!("{}.{}", path, field),
                    message: format!("Missing required field '{}'", field),
                    code: "MISSING_FIELD".to_string(),
                });
            }
        }
    }
}

/// Validate sampling parameters
fn validate_sampling(sampling: &Value, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>) {
    let sampling_obj = sampling.as_object().unwrap();

    // Check temperature range
    if let Some(temp) = sampling_obj.get("temperature") {
        if let Some(temp_val) = temp.as_f64() {
            if temp_val < 0.0 || temp_val > 2.0 {
                errors.push(ValidationError {
                    path: "sampling.temperature".to_string(),
                    message: "Temperature must be between 0.0 and 2.0".to_string(),
                    code: "INVALID_RANGE".to_string(),
                });
            }
        } else {
            errors.push(ValidationError {
                path: "sampling.temperature".to_string(),
                message: "Temperature must be a number".to_string(),
                code: "INVALID_TYPE".to_string(),
            });
        }
    }

    // Check top_p range
    if let Some(top_p) = sampling_obj.get("top_p") {
        if let Some(top_p_val) = top_p.as_f64() {
            if top_p_val < 0.0 || top_p_val > 1.0 {
                errors.push(ValidationError {
                    path: "sampling.top_p".to_string(),
                    message: "top_p must be between 0.0 and 1.0".to_string(),
                    code: "INVALID_RANGE".to_string(),
                });
            }
        } else {
            errors.push(ValidationError {
                path: "sampling.top_p".to_string(),
                message: "top_p must be a number".to_string(),
                code: "INVALID_TYPE".to_string(),
            });
        }
    }

    // Warn about conflicting parameters
    if sampling_obj.contains_key("temperature") && sampling_obj.contains_key("top_p") {
        warnings.push(ValidationWarning {
            path: "sampling".to_string(),
            message: "Using both temperature and top_p may lead to unexpected behavior".to_string(),
            code: "CONFLICTING_PARAMS".to_string(),
        });
    }
}

/// Validate a provider specification
fn validate_provider_spec(spec: &Value, errors: &mut Vec<ValidationError>, _warnings: &mut Vec<ValidationWarning>) {
    if !spec.is_object() {
        errors.push(ValidationError {
            path: "".to_string(),
            message: "Provider spec must be an object".to_string(),
            code: "INVALID_TYPE".to_string(),
        });
        return;
    }

    let obj = spec.as_object().unwrap();

    // Check required fields
    for field in &["name", "version", "base_url", "auth", "models", "mappings"] {
        if !obj.contains_key(*field) {
            errors.push(ValidationError {
                path: field.to_string(),
                message: format!("Missing required field '{}'", field),
                code: "MISSING_FIELD".to_string(),
            });
        }
    }

    // Validate models array
    if let Some(models) = obj.get("models") {
        if !models.is_array() {
            errors.push(ValidationError {
                path: "models".to_string(),
                message: "Field 'models' must be an array".to_string(),
                code: "INVALID_TYPE".to_string(),
            });
        } else {
            let models_array = models.as_array().unwrap();
            if models_array.is_empty() {
                errors.push(ValidationError {
                    path: "models".to_string(),
                    message: "Models array cannot be empty".to_string(),
                    code: "EMPTY_ARRAY".to_string(),
                });
            }
        }
    }
}