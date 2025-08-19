//! Command handlers for CLI subcommands
//!
//! This module contains the implementation logic for each CLI subcommand.

use crate::cli::{
    CompletionsArgs, ConfigAction, ConfigArgs, ConfigFormat, ConfigGetArgs, 
    ConfigGetFormat, ConfigInitArgs, ConfigSetArgs, ConfigShowArgs, 
    PreviewArgs, RunArgs, StrictMode, TranslateArgs, ValidateArgs,
};
use crate::config::Config;
use crate::error::{Error, Result};
use crate::logging::{timing::Timer, redaction};
use crate::output::OutputWriter;
use clap::CommandFactory;
use indicatif::{ProgressBar, ProgressStyle};
use specado_core::{PromptSpec, ProviderSpec};
use specado_schemas::validation::{
    create_prompt_spec_validator, create_provider_spec_validator,
    SchemaValidator, ValidationError, ValidationMode,
};
use std::fs;
use std::path::Path;
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

/// Handle the translate command (placeholder for L2)
pub async fn handle_translate(
    args: TranslateArgs,
    _config: &Config,
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

/// Handle the config command
pub async fn handle_config(
    args: ConfigArgs,
    config: &Config,
    output: &mut OutputWriter,
) -> Result<()> {
    match args.action {
        ConfigAction::Init(init_args) => handle_config_init(init_args, output).await,
        ConfigAction::Show(show_args) => handle_config_show(show_args, config, output).await,
        ConfigAction::Set(set_args) => handle_config_set(set_args, output).await,
        ConfigAction::Get(get_args) => handle_config_get(get_args, config, output).await,
        ConfigAction::Profiles => handle_config_profiles(config, output).await,
        ConfigAction::Validate => handle_config_validate(config, output).await,
    }
}

/// Handle config init subcommand
async fn handle_config_init(
    args: ConfigInitArgs,
    output: &mut OutputWriter,
) -> Result<()> {
    let mut created_any = false;
    
    // If no specific option is given, default to both
    let init_user = args.user || !args.project;
    let init_project = args.project || !args.user;
    
    if init_user {
        let user_config_path = Config::user_config_path()
            .ok_or_else(|| Error::config("Unable to determine user config directory"))?;
        
        if user_config_path.exists() && !args.force {
            output.warning(&format!(
                "User config already exists at {}", 
                user_config_path.display()
            ))?;
        } else {
            Config::create_default_user_config()?;
            output.success(&format!(
                "✓ Created user config at {}", 
                user_config_path.display()
            ))?;
            created_any = true;
        }
    }
    
    if init_project {
        let project_config_path = std::path::PathBuf::from(".specado.toml");
        
        if project_config_path.exists() && !args.force {
            output.warning("Project config already exists at .specado.toml")?;
        } else {
            Config::create_default_project_config()?;
            output.success("✓ Created project config at .specado.toml")?;
            created_any = true;
        }
    }
    
    if created_any {
        output.info("Configuration files created with default values.")?;
        output.info("Edit them to customize settings for your environment.")?;
    }
    
    Ok(())
}

/// Handle config show subcommand
async fn handle_config_show(
    args: ConfigShowArgs,
    config: &Config,
    output: &mut OutputWriter,
) -> Result<()> {
    let config_to_show = if args.user_only {
        // Load only user config
        if let Some(user_path) = Config::user_config_path() {
            if user_path.exists() {
                Config::from_file(&user_path)?
            } else {
                output.warning("No user config found")?;
                return Ok(());
            }
        } else {
            output.error("Unable to determine user config path")?;
            return Ok(());
        }
    } else if args.project_only {
        // Load only project config
        if let Some(project_path) = Config::find_project_config() {
            Config::from_file(&project_path)?
        } else {
            output.warning("No project config found")?;
            return Ok(());
        }
    } else {
        // Use merged config (default)
        config.clone()
    };
    
    let content = match args.format {
        ConfigFormat::Toml => {
            toml::to_string_pretty(&config_to_show)
                .map_err(|e| Error::config(format!("Failed to serialize as TOML: {}", e)))?
        }
        ConfigFormat::Json => {
            serde_json::to_string_pretty(&config_to_show)
                .map_err(|e| Error::config(format!("Failed to serialize as JSON: {}", e)))?
        }
        ConfigFormat::Yaml => {
            serde_yaml::to_string(&config_to_show)
                .map_err(|e| Error::config(format!("Failed to serialize as YAML: {}", e)))?
        }
    };
    
    println!("{}", content);
    Ok(())
}

/// Handle config set subcommand
async fn handle_config_set(
    args: ConfigSetArgs,
    _output: &mut OutputWriter,
) -> Result<()> {
    _output.warning("Config set command is not yet implemented")?;
    _output.info(&format!("Would set {} = {} in config", args.key, args.value))?;
    
    if args.user {
        _output.info("Target: User config (~/.specado/config.toml)")?;
    } else if args.project {
        _output.info("Target: Project config (.specado.toml)")?;
    } else {
        _output.info("Target: Auto-detect (project if exists, otherwise user)")?;
    }
    
    if let Some(profile) = args.profile {
        _output.info(&format!("Profile: {}", profile))?;
    }
    
    Err(Error::other("config set not yet implemented"))
}

/// Handle config get subcommand
async fn handle_config_get(
    args: ConfigGetArgs,
    config: &Config,
    _output: &mut OutputWriter,
) -> Result<()> {
    let value = get_config_value(config, &args.key)?;
    
    match args.format {
        ConfigGetFormat::Value => {
            println!("{}", value);
        }
        ConfigGetFormat::Json => {
            let json_value = serde_json::json!({
                "key": args.key,
                "value": value
            });
            println!("{}", serde_json::to_string_pretty(&json_value)?);
        }
    }
    
    Ok(())
}

/// Handle config profiles subcommand
async fn handle_config_profiles(
    config: &Config,
    output: &mut OutputWriter,
) -> Result<()> {
    if config.profiles.is_empty() {
        output.info("No profiles configured")?;
        return Ok(());
    }
    
    output.section("Available Profiles")?;
    
    for (name, profile) in &config.profiles {
        let is_active = config.active_profile.as_ref() == Some(name);
        let marker = if is_active { " (active)" } else { "" };
        
        output.info(&format!("• {}{}", name, marker))?;
        
        if let Some(provider) = &profile.default_provider {
            output.info(&format!("  Provider: {}", provider))?;
        }
        if let Some(model) = &profile.default_model {
            output.info(&format!("  Model: {}", model))?;
        }
        if let Some(strict) = profile.strict_mode {
            output.info(&format!("  Strict mode: {}", strict))?;
        }
    }
    
    Ok(())
}

/// Handle config validate subcommand
async fn handle_config_validate(
    _config: &Config,
    output: &mut OutputWriter,
) -> Result<()> {
    output.info("Validating configuration...")?;
    
    match _config.validate() {
        Ok(()) => {
            output.success("✓ Configuration is valid")?;
        }
        Err(e) => {
            output.error(&format!("✗ Configuration validation failed: {}", e))?;
            return Err(e);
        }
    }
    
    // Show configuration sources
    output.section("Configuration Sources")?;
    
    if let Some(user_path) = Config::user_config_path() {
        let exists = if user_path.exists() { "✓" } else { "✗" };
        output.info(&format!("{} User config: {}", exists, user_path.display()))?;
    }
    
    if let Some(project_path) = Config::find_project_config() {
        output.info(&format!("✓ Project config: {}", project_path.display()))?;
    } else {
        output.info("✗ No project config found")?;
    }
    
    // Show environment variables
    let env_vars = [
        "SPECADO_DEFAULT_PROVIDER",
        "SPECADO_DEFAULT_MODEL", 
        "SPECADO_STRICT_MODE",
        "SPECADO_PROVIDER_DIR",
        "SPECADO_PROFILE",
        "SPECADO_OUTPUT_FORMAT",
        "SPECADO_OUTPUT_COLOR",
        "SPECADO_LOG_LEVEL",
        "SPECADO_LOG_FORMAT",
    ];
    
    let mut active_env_vars = Vec::new();
    for var in &env_vars {
        if std::env::var(var).is_ok() {
            active_env_vars.push(*var);
        }
    }
    
    if !active_env_vars.is_empty() {
        output.section("Active Environment Variables")?;
        for var in active_env_vars {
            if let Ok(value) = std::env::var(var) {
                output.info(&format!("• {} = {}", var, value))?;
            }
        }
    }
    
    Ok(())
}

/// Get a configuration value by key path
fn get_config_value(config: &Config, key: &str) -> Result<String> {
    match key {
        "default_provider" => Ok(config.default_provider.clone().unwrap_or_default()),
        "default_model" => Ok(config.default_model.clone().unwrap_or_default()),
        "strict_mode" => Ok(config.strict_mode.to_string()),
        "provider_dir" => Ok(config.provider_dir.as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default()),
        "active_profile" => Ok(config.active_profile.clone().unwrap_or_default()),
        "output.format" => Ok(config.output.format.clone()),
        "output.color" => Ok(config.output.color.to_string()),
        "output.progress" => Ok(config.output.progress.to_string()),
        "output.verbosity" => Ok(config.output.verbosity.to_string()),
        "logging.level" => Ok(config.logging.level.clone()),
        "logging.format" => Ok(config.logging.format.clone()),
        "logging.timestamps" => Ok(config.logging.timestamps.to_string()),
        "logging.thread_ids" => Ok(config.logging.thread_ids.to_string()),
        "paths.providers_dir" => Ok(config.paths.providers_dir.display().to_string()),
        "paths.cache_dir" => Ok(config.paths.cache_dir.display().to_string()),
        _ => Err(Error::config(format!("Unknown configuration key: {}", key))),
    }
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