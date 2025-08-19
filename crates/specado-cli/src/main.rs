//! Specado CLI - Command-line interface for spec-driven LLM translation
//!
//! This is the main entry point for the Specado CLI application, providing
//! commands for validating, previewing, and executing LLM prompts across
//! different providers.

mod cli;
mod config;
mod error;
mod handlers;
mod logging;
mod output;

use cli::{Cli, Commands};
use colored::control;
use config::Config;
use error::Result;
use logging::{LoggingConfig, timing::Timer};
use output::OutputWriter;
use std::process;
use tracing::instrument;

#[tokio::main]
async fn main() {
    // Parse command-line arguments
    let cli = Cli::parse_args();
    
    // Set up colored output
    control::set_override(cli.use_color());
    
    // Initialize logging
    if let Err(e) = init_logging(&cli) {
        eprintln!("Failed to initialize logging: {}", e);
    }
    
    // Run the application
    let result = run(cli).await;
    
    // Handle the result
    match result {
        Ok(()) => {
            process::exit(0);
        }
        Err(e) => {
            eprintln!("{}", error::format_error(&e, control::SHOULD_COLORIZE.should_colorize()));
            
            if e.should_show_help() {
                eprintln!("\nFor more information, try '--help'");
            }
            
            process::exit(e.exit_code());
        }
    }
}

/// Main application logic
#[instrument(skip(cli), fields(command = ?cli.command))]
async fn run(cli: Cli) -> Result<()> {
    let _timer = Timer::new("cli_execution");
    
    // Load configuration
    let config = {
        let _config_timer = Timer::new("config_loading");
        tracing::info!("Loading configuration");
        Config::load_with_file(cli.config.as_deref())?
    };
    
    // Create output writer
    let mut output = OutputWriter::new(cli.output, cli.use_color(), cli.quiet, cli.verbosity_level());
    
    tracing::info!(
        command = ?cli.command,
        verbosity = cli.verbosity_level(),
        "Executing command"
    );
    
    // Handle the subcommand
    match cli.command {
        Commands::Validate(args) => {
            handlers::handle_validate(args, &config, &mut output).await
        }
        Commands::Preview(args) => {
            handlers::handle_preview(args, &config, &mut output).await
        }
        Commands::Translate(args) => {
            handlers::handle_translate(args, &config, &mut output).await
        }
        Commands::Run(args) => {
            handlers::handle_run(args, &config, &mut output).await
        }
        Commands::Config(args) => {
            handlers::handle_config(args, &config, &mut output).await
        }
        Commands::Completions(args) => {
            handlers::handle_completions(args)
        }
    }
}

/// Initialize the logging system
fn init_logging(cli: &Cli) -> Result<()> {
    // Create logging configuration from CLI args and environment
    let mut logging_config = LoggingConfig::from_verbosity(cli.verbosity_level());
    
    // Apply environment overrides
    logging_config.merge_with_env();
    
    // If quiet mode, only log errors
    if cli.quiet {
        logging_config.level = "error".to_string();
        logging_config.console = false;
    }
    
    // Initialize the logging system
    logging::init_logging(logging_config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    
    #[test]
    fn test_cli_parsing() {
        // Test basic command parsing
        let cli = Cli::parse_from(["specado", "--help"]);
        assert_eq!(cli.verbosity_level(), 0);
        
        // Test verbose flag
        let cli = Cli::parse_from(["specado", "-vv", "validate", "test.json"]);
        assert_eq!(cli.verbosity_level(), 2);
        
        // Test quiet flag
        let cli = Cli::parse_from(["specado", "--quiet", "validate", "test.json"]);
        assert_eq!(cli.verbosity_level(), 0);
    }
}