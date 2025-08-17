//! Command handlers for CLI subcommands
//!
//! This module contains the implementation logic for each CLI subcommand.

use crate::cli::{CompletionsArgs, PreviewArgs, StrictMode, TranslateArgs, ValidateArgs};
use crate::config::Config;
use crate::error::{Error, Result};
use crate::output::{OutputFormatter, OutputWriter};
use clap::CommandFactory;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use specado_core::{PromptSpec, ProviderSpec};
use specado_schemas::validation::{
    create_prompt_spec_validator, create_provider_spec_validator,
    SchemaValidator, ValidationError, ValidationMode,
};
use std::fs;
use std::path::Path;

/// Handle the validate command
pub async fn handle_validate(
    args: ValidateArgs,
    config: &Config,
    output: &mut OutputWriter,
) -> Result<()> {
    output.info(&format!("Validating specification: {}", args.prompt_spec.display()))?;
    
    // Check if file exists
    if !args.prompt_spec.exists() {
        return Err(Error::FileNotFound {
            path: args.prompt_spec.clone(),
        });
    }
    
    // Read the spec file
    let content = fs::read_to_string(&args.prompt_spec)?;
    
    // Determine file format and parse to JSON value for validation
    let is_yaml = args.prompt_spec
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s == "yaml" || s == "yml")
        .unwrap_or(false);
    
    let spec_value: serde_json::Value = if is_yaml {
        serde_yaml::from_str(&content).map_err(|e| Error::InvalidFormat {
            path: args.prompt_spec.clone(),
            expected: "YAML".to_string(),
        })?
    } else {
        serde_json::from_str(&content).map_err(|e| Error::InvalidFormat {
            path: args.prompt_spec.clone(),
            expected: "JSON".to_string(),
        })?
    };
    
    // Determine spec type and validate
    let spec_type = detect_spec_type(&spec_value);
    
    // Convert strict mode to validation mode
    let validation_mode = match args.strict {
        StrictMode::Strict => ValidationMode::Strict,
        StrictMode::Warn => ValidationMode::Partial,
        StrictMode::Coerce => ValidationMode::Basic,
    };
    
    // Run validation based on spec type
    let validation_result = match spec_type {
        SpecType::PromptSpec => {
            output.info("Detected PromptSpec format")?;
            validate_prompt_spec(&spec_value, validation_mode)
        }
        SpecType::ProviderSpec => {
            output.info("Detected ProviderSpec format")?;
            validate_provider_spec(&spec_value, validation_mode)
        }
        SpecType::Unknown => {
            return Err(Error::InvalidFormat {
                path: args.prompt_spec.clone(),
                expected: "PromptSpec or ProviderSpec".to_string(),
            });
        }
    };
    
    // Handle validation results
    match validation_result {
        Ok(()) => {
            output.success("✓ Specification is valid")?;
            
            if args.detailed {
                output.section("Specification Details")?;
                output.data(&spec_value)?;
            }
        }
        Err(validation_error) => {
            output.error("✗ Specification validation failed")?;
            
            // Format and display validation errors
            format_validation_errors(output, &validation_error)?;
            
            if args.detailed {
                output.section("Failed Specification")?;
                output.data(&spec_value)?;
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

/// Format validation errors for display
fn format_validation_errors(
    output: &mut OutputWriter,
    error: &ValidationError,
) -> Result<()> {
    output.error(&format!("Validation error at '{}':", error.path))?;
    output.error(&format!("  {}", error.message))?;
    
    if !error.schema_violations.is_empty() {
        output.error("")?;
        output.error("Schema violations:")?;
        
        for violation in &error.schema_violations {
            output.error(&format!("  • Rule: {}", violation.rule))?;
            output.error(&format!("    Expected: {}", violation.expected))?;
            output.error(&format!("    Actual: {}", violation.actual))?;
            output.error("")?;
        }
    }
    
    Ok(())
}

/// Handle the preview command
pub async fn handle_preview(
    args: PreviewArgs,
    config: &Config,
    output: &mut OutputWriter,
) -> Result<()> {
    output.info(&format!(
        "Previewing translation for {} with provider {} and model {}",
        args.prompt_spec.display(),
        args.provider,
        args.model
    ))?;
    
    // Check if prompt spec file exists
    if !args.prompt_spec.exists() {
        return Err(Error::FileNotFound {
            path: args.prompt_spec.clone(),
        });
    }
    
    // Load prompt spec
    let prompt_content = fs::read_to_string(&args.prompt_spec)?;
    let prompt_spec: PromptSpec = if args.prompt_spec
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s == "yaml" || s == "yml")
        .unwrap_or(false)
    {
        serde_yaml::from_str(&prompt_content)?
    } else {
        serde_json::from_str(&prompt_content)?
    };
    
    // Load provider spec
    let provider_spec = load_provider_spec(&args.provider, config)?;
    
    // Perform translation
    let progress = if output.show_progress() {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message("Translating prompt specification...");
        Some(pb)
    } else {
        None
    };
    
    let translation_result = specado_core::translate(
        &prompt_spec,
        &provider_spec,
        &args.model,
        args.strict.into(),
    )?;
    
    if let Some(pb) = progress {
        pb.finish_and_clear();
    }
    
    // Output the translation result
    output.success("✓ Translation completed successfully")?;
    
    if args.show_metadata {
        if let Some(metadata) = &translation_result.metadata {
            output.section("Translation Metadata")?;
            output.data(metadata)?;
        }
    }
    
    if args.show_lossiness && translation_result.has_lossiness() {
        output.section("Lossiness Report")?;
        output.data(&translation_result.lossiness)?;
    }
    
    output.section("Translated Request")?;
    output.data(&translation_result.provider_request_json)?;
    
    // Save to file if requested
    if let Some(output_file) = args.output_file {
        let output_content = match output.format() {
            crate::cli::OutputFormat::Json | crate::cli::OutputFormat::JsonPretty => {
                serde_json::to_string_pretty(&translation_result.provider_request_json)?
            }
            crate::cli::OutputFormat::Yaml => {
                serde_yaml::to_string(&translation_result.provider_request_json)?
            }
            _ => {
                serde_json::to_string_pretty(&translation_result.provider_request_json)?
            }
        };
        
        fs::write(&output_file, output_content)?;
        output.success(&format!("✓ Output saved to {}", output_file.display()))?;
    }
    
    Ok(())
}

/// Handle the translate command (placeholder for L2)
pub async fn handle_translate(
    args: TranslateArgs,
    config: &Config,
    output: &mut OutputWriter,
) -> Result<()> {
    output.warning("⚠ The translate command is not yet implemented (L2 feature)")?;
    output.info("This command will:")?;
    output.info("  • Translate the prompt specification to provider format")?;
    output.info("  • Execute the request against the provider API")?;
    output.info("  • Return the normalized response")?;
    
    if args.stream {
        output.info("  • Support streaming responses")?;
    }
    
    Err(Error::other("translate command not yet implemented (L2)"))
}

/// Handle the completions command
pub fn handle_completions(args: CompletionsArgs) -> Result<()> {
    use clap_complete::generate;
    use std::io;
    
    let mut cmd = crate::cli::Cli::command();
    let name = cmd.get_name().to_string();
    
    generate(
        args.shell.to_clap_shell(),
        &mut cmd,
        name,
        &mut io::stdout(),
    );
    
    Ok(())
}

/// Load a provider specification
fn load_provider_spec(provider: &str, config: &Config) -> Result<ProviderSpec> {
    // Check if provider is a file path
    let provider_path = Path::new(provider);
    
    let spec_path = if provider_path.exists() {
        provider_path.to_path_buf()
    } else {
        // Try to find provider in standard locations
        let mut paths_to_try = vec![
            // Local providers directory
            Path::new("providers").join(provider).with_extension("json"),
            Path::new("providers").join(provider).with_extension("yaml"),
            // Providers/{provider}/{provider}.json
            Path::new("providers")
                .join(provider)
                .join(format!("{}.json", provider)),
            Path::new("providers")
                .join(provider)
                .join(format!("{}.yaml", provider)),
        ];
        
        // Add config providers directory
        paths_to_try.push(config.paths.providers_dir.join(format!("{}.json", provider)));
        paths_to_try.push(config.paths.providers_dir.join(format!("{}.yaml", provider)));
        
        paths_to_try
            .into_iter()
            .find(|p| p.exists())
            .ok_or_else(|| Error::ProviderNotFound {
                name: provider.to_string(),
            })?
    };
    
    // Load and parse the provider spec
    let content = fs::read_to_string(&spec_path)?;
    
    let provider_spec: ProviderSpec = if spec_path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s == "yaml" || s == "yml")
        .unwrap_or(false)
    {
        serde_yaml::from_str(&content)?
    } else {
        serde_json::from_str(&content)?
    };
    
    Ok(provider_spec)
}