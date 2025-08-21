//! Validation command handler and related utilities

use crate::cli::{StrictMode, ValidateArgs};
use crate::config::Config;
use crate::error::{Error, Result};
use crate::logging::{timing::Timer, redaction};
use crate::output::OutputWriter;
use specado_schemas::validation::{
    create_prompt_spec_validator, create_provider_spec_validator,
    SchemaValidator, ValidationError, ValidationMode,
};
use std::fs;
use tracing::{instrument, info, warn, error, debug};

/// Handle the validate command
#[instrument(skip(_config, output), fields(file = %args.prompt_spec.display(), strict = ?args.strict))]
pub async fn handle_validate(
    args: ValidateArgs,
    _config: &Config,
    output: &mut OutputWriter,
) -> Result<()> {
    let _timer = Timer::with_details("validate_command", &format!("file: {}", args.prompt_spec.display()));
    info!("Starting validation process");
    output.info(&format!("Validating specification: {}", args.prompt_spec.display()))?;
    
    // Check if file exists
    if !args.prompt_spec.exists() {
        error!("File not found: {}", args.prompt_spec.display());
        return Err(Error::FileNotFound {
            path: args.prompt_spec.clone(),
        });
    }
    
    // Read the spec file
    debug!("Reading specification file");
    let content = fs::read_to_string(&args.prompt_spec)?;
    debug!("File read successfully, {} bytes", content.len());
    
    // Determine file format and parse to JSON value for validation
    let is_yaml = args.prompt_spec
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s == "yaml" || s == "yml")
        .unwrap_or(false);
    
    let spec_value: serde_json::Value = if is_yaml {
        serde_yaml::from_str(&content).map_err(|_e| Error::InvalidFormat {
            path: args.prompt_spec.clone(),
            expected: "YAML".to_string(),
        })?
    } else {
        serde_json::from_str(&content).map_err(|_e| Error::InvalidFormat {
            path: args.prompt_spec.clone(),
            expected: "JSON".to_string(),
        })?
    };
    
    // Determine spec type and validate
    let spec_type = detect_spec_type(&spec_value);
    debug!("Detected spec type: {:?}", spec_type);
    
    // Convert strict mode to validation mode
    let validation_mode = match args.strict {
        StrictMode::Strict => ValidationMode::Strict,
        StrictMode::Warn => ValidationMode::Partial,
        StrictMode::Coerce => ValidationMode::Basic,
    };
    debug!("Using validation mode: {:?}", validation_mode);
    
    // Run validation based on spec type
    let validation_result = {
        let _validation_timer = Timer::new("schema_validation");
        match spec_type {
            SpecType::PromptSpec => {
                info!("Validating as PromptSpec");
                output.info("Detected PromptSpec format")?;
                validate_prompt_spec(&spec_value, validation_mode)
            }
            SpecType::ProviderSpec => {
                info!("Validating as ProviderSpec");
                output.info("Detected ProviderSpec format")?;
                validate_provider_spec(&spec_value, validation_mode)
            }
            SpecType::Unknown => {
                error!("Unknown specification format");
                return Err(Error::InvalidFormat {
                    path: args.prompt_spec.clone(),
                    expected: "PromptSpec or ProviderSpec".to_string(),
                });
            }
        }
    };
    
    // Handle validation results
    match validation_result {
        Ok(()) => {
            info!("Validation completed successfully");
            output.success("✓ Specification is valid")?;
            
            if args.detailed {
                debug!("Showing detailed specification information");
                output.section("Specification Details")?;
                // Redact sensitive information before displaying
                let mut redacted_value = spec_value.clone();
                redaction::redact_json_value(&mut redacted_value);
                output.data(&redacted_value)?;
            }
        }
        Err(validation_error) => {
            warn!("Validation failed with {} violations", validation_error.schema_violations.len());
            output.error("✗ Specification validation failed")?;
            
            // Format and display validation errors
            format_validation_errors(output, &validation_error)?;
            
            if args.detailed {
                debug!("Showing failed specification details");
                output.section("Failed Specification")?;
                // Redact sensitive information before displaying
                let mut redacted_value = spec_value.clone();
                redaction::redact_json_value(&mut redacted_value);
                output.data(&redacted_value)?;
            }
            
            return Err(Error::other(format!(
                "Validation failed with {} violation(s)",
                validation_error.schema_violations.len()
            )));
        }
    }
    
    Ok(())
}

/// Spec type detection
#[derive(Debug, PartialEq)]
enum SpecType {
    PromptSpec,
    ProviderSpec,
    Unknown,
}

/// Detect whether a JSON value is a PromptSpec or ProviderSpec
fn detect_spec_type(value: &serde_json::Value) -> SpecType {
    if let Some(obj) = value.as_object() {
        // PromptSpec has 'model_class' and 'messages' (or 'prompt')
        if obj.contains_key("model_class") || 
           obj.contains_key("messages") || 
           obj.contains_key("prompt") {
            return SpecType::PromptSpec;
        }
        
        // ProviderSpec has 'provider' and 'models'
        if obj.contains_key("provider") || obj.contains_key("models") {
            return SpecType::ProviderSpec;
        }
    }
    
    SpecType::Unknown
}

/// Validate a PromptSpec
fn validate_prompt_spec(
    spec: &serde_json::Value,
    mode: ValidationMode,
) -> std::result::Result<(), ValidationError> {
    let validator = create_prompt_spec_validator()
        .map_err(|e| ValidationError::new(
            "$",
            format!("Failed to create PromptSpec validator: {}", e)
        ))?;
    
    let context = specado_schemas::validation::ValidationContext::new(mode);
    validator.validate_with_context(spec, &context)
}

/// Validate a ProviderSpec
fn validate_provider_spec(
    spec: &serde_json::Value,
    mode: ValidationMode,
) -> std::result::Result<(), ValidationError> {
    let validator = create_provider_spec_validator()
        .map_err(|e| ValidationError::new(
            "$",
            format!("Failed to create ProviderSpec validator: {}", e)
        ))?;
    
    let context = specado_schemas::validation::ValidationContext::new(mode);
    validator.validate_with_context(spec, &context)
}

/// Format validation errors for display (legacy function - use OutputWriter::validation_error instead)
fn format_validation_errors(
    output: &mut OutputWriter,
    error: &ValidationError,
) -> Result<()> {
    // Use the new specialized method
    output.validation_error(error)
}