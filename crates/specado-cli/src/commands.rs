use crate::chat;
use crate::cli::{CompletionShell, RuntimeOptions};
use crate::io::{load_prompt_spec, load_provider_spec, parse_to_json_value};
use crate::resolver::resolve_provider_path;
use crate::runtime;
use anyhow::{anyhow, Context, Result};
use clap::CommandFactory;
use colored::Colorize;
use specado_core::hot_reload::ProviderCache;
use specado_core::{
    execute, translate as core_translate, LossinessLevel, LossinessReport, Message, MessageRole,
};
use specado_schemas::get_validator;
use std::path::PathBuf;

pub async fn validate_command(spec_path: PathBuf) -> Result<()> {
    let content = std::fs::read_to_string(&spec_path)
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

pub async fn preview_command(
    prompt_path: PathBuf,
    provider_path: PathBuf,
    runtime: RuntimeOptions,
) -> Result<()> {
    #[cfg(feature = "hot-reload")]
    runtime::apply_hot_reload_config(&runtime, &provider_path);

    let prompt = load_prompt_spec(&prompt_path)?;
    let provider = load_provider_spec(&provider_path)?;

    let (translated, lossiness) = core_translate(&prompt, &provider)?;

    println!("{}", "=== Translated Request ===".cyan().bold());
    println!("{}", serde_json::to_string_pretty(&translated)?);
    println!();

    print_lossiness(&lossiness);

    Ok(())
}

pub async fn run_command(
    prompt_path: PathBuf,
    provider_path: PathBuf,
    runtime: RuntimeOptions,
) -> Result<()> {
    #[cfg(feature = "hot-reload")]
    runtime::apply_hot_reload_config(&runtime, &provider_path);

    #[cfg(feature = "audit-logging")]
    let audit_context = runtime::build_audit_context(&runtime)?;

    let prompt = load_prompt_spec(&prompt_path)?;
    let response = execute(
        prompt,
        provider_path
            .to_str()
            .ok_or_else(|| anyhow!("Provider path contains invalid UTF-8"))?,
        #[cfg(feature = "audit-logging")]
        audit_context,
    )
    .await?;

    println!("{}", serde_json::to_string_pretty(&response)?);

    Ok(())
}

pub async fn ask_command(
    prompt: Option<String>,
    provider_flag: Option<String>,
    model_flag: Option<String>,
    interactive: bool,
    messages_file: Option<PathBuf>,
    runtime: RuntimeOptions,
) -> Result<()> {
    if !interactive && prompt.as_ref().map(|p| p.trim().is_empty()).unwrap_or(true) {
        return Err(anyhow!(
            "Prompt argument is required unless --interactive is provided."
        ));
    }

    let provider_path = resolve_provider_path(provider_flag.as_deref(), model_flag.as_deref())?;

    #[cfg(feature = "hot-reload")]
    runtime::apply_hot_reload_config(&runtime, &provider_path);

    if interactive {
        let mut history = if let Some(path) = messages_file {
            chat::load_history_messages(&path)?
        } else {
            Vec::new()
        };
        chat::ensure_system_message(&mut history);

        let provider_spec = ProviderCache::new()
            .load_or_read(&provider_path)
            .map_err(|err| {
                anyhow!(
                    "Failed to load provider spec {}: {}",
                    provider_path.display(),
                    err
                )
            })?;

        chat::run_interactive_chat(prompt, history, &provider_path, &provider_spec, &runtime)
            .await?;
    } else {
        let mut messages = chat::base_system_messages();
        messages.push(Message {
            role: MessageRole::User,
            content: prompt.expect("prompt required when not interactive"),
        });
        let response = chat::execute_messages(&messages, &provider_path, &runtime).await?;
        println!("{}", response.content.trim());
    }

    Ok(())
}

pub fn completions_command(shell: CompletionShell) -> Result<()> {
    use clap_complete::generate;
    use std::io;

    let mut cmd = crate::cli::Cli::command();
    let name = cmd.get_name().to_string();
    generate(shell.to_clap_shell(), &mut cmd, name, &mut io::stdout());

    Ok(())
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
                println!("    details: {}", details);
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
