mod chat;
mod cli;
mod commands;
mod io;
mod resolver;
mod runtime;

use crate::cli::{Cli, Commands};
use clap::Parser;
use colored::Colorize;
use std::process;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Ask {
            prompt,
            provider,
            model,
            interactive,
            messages_file,
            runtime,
        } => {
            commands::ask_command(prompt, provider, model, interactive, messages_file, runtime)
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
