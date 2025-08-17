//! Command handlers for CLI subcommands
//!
//! This module contains the implementation logic for each CLI subcommand.

use crate::cli::{CompletionsArgs, PreviewArgs, TranslateArgs, ValidateArgs};
use crate::config::Config;
use crate::error::{Error, Result};
use crate::output::{OutputFormatter, OutputWriter};
use clap::CommandFactory;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use specado_core::{PromptSpec, ProviderSpec};
use std::fs;
use std::path::Path;

/// Handle the validate command
pub async fn handle_validate(
    args: ValidateArgs,
    config: &Config,
    output: &mut OutputWriter,
) -> Result<()> {
    output.info(&format!("Validating prompt spec: {}", args.prompt_spec.display()))?;
    
    // Check if file exists
    if !args.prompt_spec.exists() {
        return Err(Error::FileNotFound {
            path: args.prompt_spec.clone(),
        });
    }
    
    // Read the prompt spec file
    let content = fs::read_to_string(&args.prompt_spec)?;
    
    // Determine file format and parse
    let prompt_spec: PromptSpec = if args.prompt_spec
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s == "yaml" || s == "yml")
        .unwrap_or(false)
    {
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
    
    // TODO: Implement actual schema validation (Issue #11)
    // For now, just check that we can parse the file
    
    output.success("✓ Prompt specification is valid")?;
    
    if args.detailed {
        output.info("Prompt specification details:")?;
        output.data(&prompt_spec)?;
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