use crate::chat;
use crate::cli::{CompletionShell, ReasonEffort, RuntimeOptions};
use crate::io::{load_prompt_spec, parse_to_json_value};
use crate::resolver::resolve_provider_path;
use crate::runtime;
use anyhow::{anyhow, Context, Result};
use clap::CommandFactory;
use colored::Colorize;
use serde_json::{Map as JsonMap, Value as JsonValue};
use specado::hot_reload::ProviderCache;
use specado::{
    execute_from_path, translate as core_translate, LossinessLevel, LossinessReport, Message,
    MessageRole, SamplingConfig,
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
    let provider = ProviderCache::new()
        .load_or_read(&provider_path)
        .map_err(|err| {
            anyhow!(
                "Failed to load provider spec {}: {}",
                provider_path.display(),
                err
            )
        })?;

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
    let response = execute_from_path(
        prompt,
        &provider_path,
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
    pub reason: bool,
    pub reason_effort: Option<ReasonEffort>,
    pub reason_budget: Option<u32>,
    pub reason_seed: Option<i64>,
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
        reason,
        reason_effort,
        reason_budget,
        reason_seed,
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

    if reason || reason_effort.is_some() || reason_budget.is_some() || reason_seed.is_some() {
        let controls = provider_spec
            .capabilities
            .reasoning_controls
            .iter()
            .map(|control| control.to_ascii_lowercase())
            .collect::<Vec<_>>();

        if !controls.is_empty() {
            let supports_effort = controls.iter().any(|c| c == "effort");
            if reason_effort.is_some() && !supports_effort {
                return Err(anyhow!(
                    "Provider '{}' does not expose reasoning effort control required by --reason-effort",
                    provider_spec.provider
                ));
            }

            let effort = reason_effort
                .or(if reason {
                    Some(ReasonEffort::Medium)
                } else {
                    None
                })
                .unwrap_or(ReasonEffort::Medium);

            let mut reasoning_map = JsonMap::new();
            reasoning_map.insert(
                "effort".into(),
                JsonValue::String(effort.as_str().to_string()),
            );
            if let Some(budget) = reason_budget {
                reasoning_map.insert("budget_tokens".into(), JsonValue::from(budget));
            }
            metadata.insert("reasoning".into(), JsonValue::Object(reasoning_map));
        } else if provider_spec.capabilities.supports_extended_thinking {
            if reason_effort.is_some() {
                return Err(anyhow!(
                    "Provider '{}' maps --reason to thinking mode and does not support --reason-effort",
                    provider_spec.provider
                ));
            }

            let thinking_obj = metadata
                .entry("thinking".to_string())
                .or_insert_with(|| JsonValue::Object(JsonMap::new()));

            if !thinking_obj.is_object() {
                *thinking_obj = JsonValue::Object(JsonMap::new());
            }

            if let Some(map) = thinking_obj.as_object_mut() {
                map.entry("type".to_string())
                    .or_insert_with(|| JsonValue::String("enabled".into()));
                if let Some(budget) = reason_budget {
                    map.insert("budget_tokens".to_string(), JsonValue::from(budget));
                }
            }
        } else {
            return Err(anyhow!(
                "Provider '{}' does not support reasoning or extended thinking capabilities",
                provider_spec.provider
            ));
        }
    }

    if let Some(seed_value) = reason_seed {
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
