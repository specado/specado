//! Configuration command handlers

use crate::cli::{
    ConfigAction, ConfigArgs, ConfigFormat, ConfigGetArgs, 
    ConfigGetFormat, ConfigInitArgs, ConfigSetArgs, ConfigShowArgs,
};
use crate::config::Config;
use crate::error::{Error, Result};
use crate::output::OutputWriter;

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