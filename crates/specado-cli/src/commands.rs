use crate::chat;
use crate::cli::{CompletionShell, ReasoningEffort, RuntimeOptions};
use crate::io::{load_prompt_spec, load_provider_spec, parse_to_json_value};
use crate::resolver::resolve_provider_path;
use crate::runtime;
use anyhow::{anyhow, Context, Result};
use clap::CommandFactory;
use colored::Colorize;
use serde_json::{Map as JsonMap, Value as JsonValue};
use specado_core::hot_reload::ProviderCache;
use specado_core::{
    execute, translate as core_translate, LossinessLevel, LossinessReport, Message, MessageRole,
    SamplingConfig,
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

pub struct AskOptions {
    pub interactive: bool,
    pub messages_file: Option<PathBuf>,
    pub thinking: bool,
    pub thinking_budget: Option<u32>,
    pub reasoning: bool,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub seed: Option<i64>,
    pub runtime: RuntimeOptions,
}

pub async fn ask_command(
    prompt: Option<String>,
    provider_flag: Option<String>,
    model_flag: Option<String>,
    options: AskOptions,
) -> Result<()> {
    let AskOptions {
        interactive,
        messages_file,
        thinking,
        thinking_budget,
        reasoning,
        reasoning_effort,
        seed,
        runtime,
    } = options;

    if !interactive && prompt.as_ref().map(|p| p.trim().is_empty()).unwrap_or(true) {
        return Err(anyhow!(
            "Prompt argument is required unless --interactive is provided."
        ));
    }

    let provider_path = resolve_provider_path(provider_flag.as_deref(), model_flag.as_deref())?;

    #[cfg(feature = "hot-reload")]
    runtime::apply_hot_reload_config(&runtime, &provider_path);

    let provider_spec = ProviderCache::new()
        .load_or_read(&provider_path)
        .map_err(|err| {
            anyhow!(
                "Failed to load provider spec {}: {}",
                provider_path.display(),
                err
            )
        })?;

    let mut metadata = JsonMap::new();
    let mut sampling = SamplingConfig::default();

    if thinking || thinking_budget.is_some() {
        if !provider_spec.capabilities.supports_extended_thinking {
            return Err(anyhow!(
                "Provider '{}' does not support --thinking",
                provider_spec.provider
            ));
        }

        let mut thinking_map = JsonMap::new();
        thinking_map.insert("type".into(), JsonValue::String("enabled".into()));
        if let Some(budget) = thinking_budget {
            thinking_map.insert("budget_tokens".into(), JsonValue::from(budget));
        }
        metadata.insert("thinking".into(), JsonValue::Object(thinking_map));
    }

    if reasoning || reasoning_effort.is_some() {
        let controls = provider_spec
            .capabilities
            .reasoning_controls
            .iter()
            .map(|control| control.to_ascii_lowercase())
            .collect::<Vec<_>>();

        if controls.is_empty() {
            return Err(anyhow!(
                "Provider '{}' does not support reasoning controls",
                provider_spec.provider
            ));
        }

        let supports_effort = controls.iter().any(|c| c == "effort");
        if !supports_effort {
            return Err(anyhow!(
                "Provider '{}' does not expose reasoning effort control required by --reasoning",
                provider_spec.provider
            ));
        }

        let effort = reasoning_effort
            .or(if reasoning { Some(ReasoningEffort::Medium) } else { None })
            .ok_or_else(|| {
                anyhow!(
                    "--reasoning requires an effort level. Pass --reasoning-effort to select a value."
                )
            })?;

        let mut reasoning_map = JsonMap::new();
        reasoning_map.insert(
            "effort".into(),
            JsonValue::String(effort.as_str().to_string()),
        );
        metadata.insert("reasoning".into(), JsonValue::Object(reasoning_map));
    }

    if let Some(seed_value) = seed {
        if !provider_spec.capabilities.supports_seed {
            return Err(anyhow!(
                "Provider '{}' does not support deterministic seeding",
                provider_spec.provider
            ));
        }
        sampling.seed = Some(seed_value);
    }

    if interactive {
        let mut history = if let Some(path) = messages_file {
            chat::load_history_messages(&path)?
        } else {
            Vec::new()
        };
        chat::ensure_system_message(&mut history);

        chat::run_interactive_chat(
            prompt,
            history,
            &metadata,
            &provider_path,
            &provider_spec,
            &runtime,
            &sampling,
        )
        .await?;
    } else {
        let mut messages = chat::base_system_messages();
        messages.push(Message {
            role: MessageRole::User,
            content: prompt.expect("prompt required when not interactive"),
        });
        let response =
            chat::execute_messages(&messages, &provider_path, &runtime, &sampling, &metadata)
                .await?;
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
