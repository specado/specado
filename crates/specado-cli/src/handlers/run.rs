//! Run command handler

use crate::cli::RunArgs;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::logging::timing::Timer;
use crate::output::OutputWriter;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use tracing::{instrument, info, debug, error};

/// Handle the run command (Issue #56, #57, #58)
#[instrument(skip(_config, output), fields(file = %args.request_file.display()))]
pub async fn handle_run(
    args: RunArgs,
    _config: &Config,
    output: &mut OutputWriter,
) -> Result<()> {
    let _timer = Timer::with_details("run_command", &format!("file: {}", args.request_file.display()));
    info!("Starting run command execution");
    
    // Check if file exists
    if !args.request_file.exists() {
        error!("Request file not found: {}", args.request_file.display());
        return Err(Error::FileNotFound {
            path: args.request_file.clone(),
        });
    }
    
    // Show progress indicator if not silent
    let pb = if !args.silent {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message("Reading request file...");
        Some(pb)
    } else {
        None
    };
    
    // Read the request file
    debug!("Reading request file");
    let content = fs::read_to_string(&args.request_file)?;
    debug!("Request file read successfully, {} bytes", content.len());
    
    // Parse as JSON
    let request_json: serde_json::Value = serde_json::from_str(&content)?;
    
    // Validate required fields
    if request_json.get("provider_spec").is_none() {
        return Err(Error::other(
            "Missing required field 'provider_spec' in request file",
        ));
    }
    if request_json.get("model_id").is_none() {
        return Err(Error::other(
            "Missing required field 'model_id' in request file",
        ));
    }
    if request_json.get("request_body").is_none() {
        return Err(Error::other(
            "Missing required field 'request_body' in request file",
        ));
    }
    
    // Update progress
    if let Some(ref pb) = pb {
        pb.set_message("Executing request...");
    }
    
    // Start timing for metrics (Issue #58)
    let start_time = std::time::Instant::now();
    
    // Execute the request using specado_core::run
    let response = specado_core::run(&request_json).await
        .map_err(|e| Error::other(format!("Request execution failed: {}", e)))?;
    
    // Calculate execution time
    let execution_time = start_time.elapsed();
    
    // Clear progress indicator
    if let Some(pb) = pb {
        pb.finish_and_clear();
    }
    
    // Display metrics if requested (Issue #58)
    if args.metrics && !args.silent {
        output.section("Execution Metrics")?;
        output.info(&format!("  • Model: {}", response.model))?;
        output.info(&format!("  • Execution Time: {:.2}s", execution_time.as_secs_f64()))?;
        output.info(&format!("  • Finish Reason: {:?}", response.finish_reason))?;
        
        // Check for token usage in raw metadata
        if let Some(usage) = response.raw_metadata.get("usage") {
            if let Some(prompt_tokens) = usage.get("prompt_tokens").and_then(|v| v.as_u64()) {
                output.info(&format!("  • Prompt Tokens: {}", prompt_tokens))?;
            }
            if let Some(completion_tokens) = usage.get("completion_tokens").and_then(|v| v.as_u64()) {
                output.info(&format!("  • Completion Tokens: {}", completion_tokens))?;
            }
            if let Some(total_tokens) = usage.get("total_tokens").and_then(|v| v.as_u64()) {
                output.info(&format!("  • Total Tokens: {}", total_tokens))?;
            }
        }
        
        // Tool calls info
        if let Some(ref tool_calls) = response.tool_calls {
            output.info(&format!("  • Tool Calls: {}", tool_calls.len()))?;
        }
    }
    
    // Prepare output JSON
    let output_json = if args.pretty {
        serde_json::to_string_pretty(&response)?
    } else {
        serde_json::to_string(&response)?
    };
    
    // Save to file if requested (Issue #57)
    if let Some(output_file) = args.save_to {
        debug!("Writing response to file: {}", output_file.display());
        fs::write(&output_file, &output_json)?;
        if !args.silent {
            output.success(&format!("✓ Response saved to {}", output_file.display()))?;
        }
    } else if !args.silent {
        // Print response to stdout
        output.section("Response")?;
        println!("{}", output_json);
    } else {
        // Silent mode: only print the response content
        println!("{}", response.content);
    }
    
    Ok(())
}