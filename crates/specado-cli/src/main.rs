//! Specado CLI - Command-line interface for spec-driven LLM translation
//!
//! This is the main entry point for the Specado CLI application, providing
//! commands for validating, previewing, and executing LLM prompts across
//! different providers.

mod cli;
mod config;
mod error;
mod handlers;
mod output;

use cli::{Cli, Commands};
use colored::control;
use config::Config;
use error::{Error, Result};
use output::OutputWriter;
use std::process;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Parse command-line arguments
    let cli = Cli::parse_args();
    
    // Set up colored output
    control::set_override(cli.use_color());
    
    // Initialize logging
    if let Err(e) = init_logging(cli.verbosity_level()) {
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
async fn run(cli: Cli) -> Result<()> {
    // Load configuration
    let config = Config::load_with_file(cli.config.as_deref())?;
    
    // Create output writer
    let mut output = OutputWriter::new(cli.output, cli.use_color(), cli.quiet);
    
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
        Commands::Completions(args) => {
            handlers::handle_completions(args)
        }
    }
}

/// Initialize the logging system
fn init_logging(verbosity: u8) -> Result<()> {
    let level = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(verbosity >= 2)
        .with_thread_ids(verbosity >= 3)
        .with_line_number(verbosity >= 3)
        .with_file(verbosity >= 3)
        .compact()
        .init();
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    
    #[test]
    fn test_cli_parsing() {
        // Test basic command parsing
        let cli = Cli::parse_from(&["specado", "--help"]);
        assert_eq!(cli.verbosity_level(), 0);
        
        // Test verbose flag
        let cli = Cli::parse_from(&["specado", "-vv", "validate", "test.json"]);
        assert_eq!(cli.verbosity_level(), 2);
        
        // Test quiet flag
        let cli = Cli::parse_from(&["specado", "--quiet", "validate", "test.json"]);
        assert_eq!(cli.verbosity_level(), 0);
    }
}