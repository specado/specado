use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use specado_core::{
    execute, translate as core_translate, LossinessLevel, LossinessReport, PromptSpec, ProviderSpec,
};
use specado_schemas::get_validator;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

#[derive(Parser)]
#[command(name = "specado")]
#[command(version, about = "Spec-driven LLM abstraction", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a prompt or provider specification against the schemas
    Validate {
        #[arg(long)]
        spec: PathBuf,
    },
    /// Preview the translated provider payload alongside the lossiness report
    Preview {
        #[arg(long)]
        prompt: PathBuf,
        #[arg(long)]
        provider: PathBuf,
    },
    /// Execute the prompt against the provider and print the normalized response
    Run {
        #[arg(long)]
        prompt: PathBuf,
        #[arg(long)]
        provider: PathBuf,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Validate { spec } => validate_command(spec).await,
        Commands::Preview { prompt, provider } => preview_command(prompt, provider).await,
        Commands::Run { prompt, provider } => run_command(prompt, provider).await,
    };

    if let Err(err) = result {
        eprintln!("{} {}", "Error:".red().bold(), err);
        process::exit(1);
    }
}

async fn validate_command(spec_path: PathBuf) -> Result<()> {
    let content = fs::read_to_string(&spec_path)
        .with_context(|| format!("Failed to read spec file: {}", spec_path.display()))?;

    let value = parse_to_json_value(&content, &spec_path)?;
    let validator = get_validator();

    let looks_provider = value.get("provider").is_some()
        || value.get("endpoints").is_some()
        || value.get("auth").is_some();

    if looks_provider {
        validator
            .validate_provider(&value)
            .map_err(|e| anyhow!("Provider spec invalid: {}", e))?;
        println!("{} Provider spec is valid", "✓".green().bold());
    } else {
        validator
            .validate_prompt(&value)
            .map_err(|e| anyhow!("Prompt spec invalid: {}", e))?;
        println!("{} Prompt spec is valid", "✓".green().bold());
    }

    Ok(())
}

async fn preview_command(prompt_path: PathBuf, provider_path: PathBuf) -> Result<()> {
    let prompt = load_prompt_spec(&prompt_path)?;
    let provider = load_provider_spec(&provider_path)?;

    let (translated, lossiness) = core_translate(&prompt, &provider)?;

    println!("{}", "=== Translated Request ===".cyan().bold());
    println!("{}", serde_json::to_string_pretty(&translated)?);
    println!();

    print_lossiness(&lossiness);

    Ok(())
}

async fn run_command(prompt_path: PathBuf, provider_path: PathBuf) -> Result<()> {
    let prompt = load_prompt_spec(&prompt_path)?;
    let response = execute(
        prompt,
        provider_path
            .to_str()
            .ok_or_else(|| anyhow!("Provider path contains invalid UTF-8"))?,
    )
    .await?;

    println!("{}", serde_json::to_string_pretty(&response)?);

    Ok(())
}

fn parse_to_json_value(content: &str, path: &Path) -> Result<serde_json::Value> {
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if ext == "yaml" || ext == "yml" {
        Ok(serde_yaml::from_str(content)?)
    } else {
        match serde_json::from_str(content) {
            Ok(value) => Ok(value),
            Err(_) => Ok(serde_yaml::from_str(content)?),
        }
    }
}

fn load_prompt_spec(path: &Path) -> Result<PromptSpec> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read prompt spec: {}", path.display()))?;

    match serde_json::from_str(&content) {
        Ok(spec) => Ok(spec),
        Err(json_err) => serde_yaml::from_str(&content).map_err(|yaml_err| {
            anyhow!("Failed to parse prompt spec as JSON ({json_err}) or YAML ({yaml_err})")
        }),
    }
}

fn load_provider_spec(path: &Path) -> Result<ProviderSpec> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read provider spec: {}", path.display()))?;

    match serde_yaml::from_str(&content) {
        Ok(spec) => Ok(spec),
        Err(yaml_err) => serde_json::from_str(&content).map_err(|json_err| {
            anyhow!("Failed to parse provider spec as YAML ({yaml_err}) or JSON ({json_err})")
        }),
    }
}

fn print_lossiness(report: &LossinessReport) {
    println!("{}", "=== Lossiness Report ===".yellow().bold());
    if report.is_lossy {
        for entry in &report.entries {
            let level = match entry.level {
                LossinessLevel::Info => "INFO".blue(),
                LossinessLevel::Warn => "WARN".yellow(),
                LossinessLevel::Error => "ERROR".red(),
            };
            println!(
                "{} [{:?}] {} ({})",
                level, entry.code, entry.reason, entry.path
            );
            if let Some(details) = &entry.details {
                println!("    details: {}", details.to_string());
            }
            if let Some(fix) = &entry.suggested_fix {
                println!("    suggested fix: {}", fix);
            }
        }
        if !report.omissions.is_empty() {
            println!("{}", "Omissions:".yellow().bold());
            for omission in &report.omissions {
                println!("  - {}", omission);
            }
        }
    } else {
        println!("{} No lossiness detected", "✓".green().bold());
    }
}
