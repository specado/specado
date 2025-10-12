mod chat;
mod cli;
mod commands;
mod io;
mod resolver;
mod runtime;

use crate::cli::{Cli, Commands};
use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;
use std::process;

fn load_env() {
    let mut errors = Vec::new();

    if let Some(global) = config_env_path() {
        if global.exists() {
            if let Err(err) = dotenvy::from_path(&global) {
                errors.push(format!("Failed to load {}: {}", global.display(), err));
            }
        }
    }

    if let Err(err) = dotenvy::dotenv_override() {
        if !matches!(err, dotenvy::Error::Io(ref io_err) if io_err.kind() == std::io::ErrorKind::NotFound)
        {
            errors.push(format!("Failed to load .env: {}", err));
        }
    }

    if !errors.is_empty() {
        for error in errors {
            eprintln!("{} {}", "warning:".yellow().bold(), error);
        }
    }
}

fn config_env_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("specado").join(".env"))
}

#[tokio::main]
async fn main() {
    load_env();
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Ask {
            prompt,
            provider,
            model,
            interactive,
            messages_file,
            reason,
            reason_effort,
            reason_budget,
            reason_seed,
            runtime,
        } => {
            commands::ask_command(
                prompt,
                provider,
                model,
                commands::AskOptions {
                    interactive,
                    messages_file,
                    reason,
                    reason_effort,
                    reason_budget,
                    reason_seed,
                    runtime,
                },
            )
            .await
        }
        Commands::Validate { spec } => commands::validate_command(spec).await,
        Commands::Preview {
            prompt,
            provider,
            runtime,
        } => commands::preview_command(prompt, provider, runtime).await,
        Commands::Run {
            prompt,
            provider,
            runtime,
        } => commands::run_command(prompt, provider, runtime).await,
        Commands::Completions { shell } => commands::completions_command(shell),
    };

    if let Err(err) = result {
        eprintln!("{} {}", "Error:".red().bold(), err);
        process::exit(1);
    }
}
