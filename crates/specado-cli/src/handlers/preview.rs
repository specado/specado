//! Preview command handler

use crate::cli::PreviewArgs;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::logging::timing::Timer;
use crate::output::OutputWriter;
use specado_core::PromptSpec;
use std::fs;
use tracing::{instrument, info, debug, error};

use super::utils::load_provider_spec;

/// Handle the preview command
#[instrument(skip(config, output), fields(
    file = %args.prompt_spec.display(),
    provider = %args.provider,
    model = %args.model
))]
pub async fn handle_preview(
    args: PreviewArgs,
    config: &Config,
    output: &mut OutputWriter,
) -> Result<()> {
    let _timer = Timer::with_details("preview_command", &format!("provider: {}, model: {}", args.provider, args.model));
    info!("Starting preview operation");
    output.info(&format!(
        "Previewing translation for {} with provider {} and model {}",
        args.prompt_spec.display(),
        args.provider,
        args.model
    ))?;
    
    // Check if prompt spec file exists
    if !args.prompt_spec.exists() {
        error!("Prompt spec file not found: {}", args.prompt_spec.display());
        return Err(Error::FileNotFound {
            path: args.prompt_spec.clone(),
        });
    }
    
    // Load prompt spec
    let prompt_spec = {
        let _load_timer = Timer::new("prompt_spec_loading");
        debug!("Loading prompt specification");
        let prompt_content = fs::read_to_string(&args.prompt_spec)?;
        debug!("Prompt spec file read, {} bytes", prompt_content.len());
        
        let spec: PromptSpec = if args.prompt_spec
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s == "yaml" || s == "yml")
            .unwrap_or(false)
        {
            debug!("Parsing prompt spec as YAML");
            serde_yaml::from_str(&prompt_content)?
        } else {
            debug!("Parsing prompt spec as JSON");
            serde_json::from_str(&prompt_content)?
        };
        
        debug!("Prompt spec loaded successfully");
        spec
    };
    
    // Load provider spec
    let provider_spec = {
        let _load_timer = Timer::new("provider_spec_loading");
        debug!("Loading provider specification: {}", args.provider);
        load_provider_spec(&args.provider, config)?
    };
    
    // Perform translation with progress indication
    let progress = output.spinner("Translating prompt specification...");
    
    let translation_result = {
        let _translation_timer = Timer::new("translation_process");
        info!("Starting translation to provider format");
        
        let result = specado_core::translate(
            &prompt_spec,
            &provider_spec,
            &args.model,
            args.strict.into(),
        )?;
        
        info!(
            lossiness_items = result.lossiness.items.len(),
            "Translation completed"
        );
        
        result
    };
    
    if let Some(pb) = progress {
        pb.finish_and_clear();
    }
    
    // Output the translation result
    output.success("✓ Translation completed successfully")?;
    
    // Use specialized formatting based on format and flags
    match output.format() {
        crate::cli::OutputFormat::Human => {
            // For human format, show sections based on flags
            if args.show_metadata {
                if let Some(metadata) = &translation_result.metadata {
                    output.section("Translation Metadata")?;
                    output.data(metadata)?;
                }
            }
            
            if args.show_lossiness && !translation_result.lossiness.items.is_empty() {
                output.section("Lossiness Report")?;
                output.lossiness_report(&translation_result.lossiness)?;
            }
            
            if args.diff {
                output.section("Translation Changes")?;
                // TODO: Implement diff view showing original vs translated
                output.info("Diff view not yet implemented")?;
            }
            
            output.section("Translated Request")?;
            output.data(&translation_result.provider_request_json)?;
        }
        _ => {
            // For machine formats, output the complete result
            output.translation_result(&translation_result)?;
        }
    }
    
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