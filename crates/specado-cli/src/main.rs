use anyhow::{anyhow, Context, Result};
use clap::{Args, Parser, Subcommand};
use colored::*;
use specado_core::{
    execute, translate as core_translate, LossinessLevel, LossinessReport, Message, MessageRole,
    PromptSpec, ProviderSpec, ResponseConfig, SamplingConfig, StrictMode,
};
use specado_schemas::get_validator;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::Duration;

#[cfg(feature = "audit-logging")]
use specado_core::audit::{AuditConfig, AuditContext, AuditTarget};
#[cfg(feature = "hot-reload")]
use specado_core::hot_reload::{set_global_config, HotReloadConfig};

#[derive(Parser)]
#[command(name = "specado")]
#[command(version, about = "Spec-driven LLM abstraction", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ask a quick question using the default provider/model
    Ask {
        /// The user prompt to send to the model
        prompt: String,
        #[command(flatten)]
        runtime: RuntimeOptions,
    },
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
        #[command(flatten)]
        runtime: RuntimeOptions,
    },
    /// Execute the prompt against the provider and print the normalized response
    Run {
        #[arg(long)]
        prompt: PathBuf,
        #[arg(long)]
        provider: PathBuf,
        #[command(flatten)]
        runtime: RuntimeOptions,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Ask { prompt, runtime } => ask_command(prompt, runtime).await,
        Commands::Validate { spec } => validate_command(spec).await,
        Commands::Preview {
            prompt,
            provider,
            runtime,
        } => preview_command(prompt, provider, runtime).await,
        Commands::Run {
            prompt,
            provider,
            runtime,
        } => run_command(prompt, provider, runtime).await,
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

async fn ask_command(prompt: String, runtime: RuntimeOptions) -> Result<()> {
    let provider_path = default_provider_path()?;

    #[cfg(feature = "hot-reload")]
    apply_hot_reload_config(&runtime, &provider_path);

    #[cfg(feature = "audit-logging")]
    let audit_context = build_audit_context(&runtime)?;

    let prompt_spec = PromptSpec {
        version: "1".to_string(),
        messages: vec![
            Message {
                role: MessageRole::System,
                content: "You are a helpful assistant.".to_string(),
            },
            Message {
                role: MessageRole::User,
                content: prompt,
            },
        ],
        sampling: SamplingConfig::default(),
        response: ResponseConfig::default(),
        tools: Vec::new(),
        tool_choice: None,
        strict_mode: StrictMode::Warn,
        metadata: Default::default(),
    };

    let response = execute(
        prompt_spec,
        provider_path
            .to_str()
            .ok_or_else(|| anyhow!("Provider path contains invalid UTF-8"))?,
        #[cfg(feature = "audit-logging")]
        audit_context,
    )
    .await?;

    println!("{}", response.content.trim());

    Ok(())
}

async fn preview_command(
    prompt_path: PathBuf,
    provider_path: PathBuf,
    runtime: RuntimeOptions,
) -> Result<()> {
    #[cfg(feature = "hot-reload")]
    apply_hot_reload_config(&runtime, &provider_path);

    let prompt = load_prompt_spec(&prompt_path)?;
    let provider = load_provider_spec(&provider_path)?;

    let (translated, lossiness) = core_translate(&prompt, &provider)?;

    println!("{}", "=== Translated Request ===".cyan().bold());
    println!("{}", serde_json::to_string_pretty(&translated)?);
    println!();

    print_lossiness(&lossiness);

    Ok(())
}

async fn run_command(
    prompt_path: PathBuf,
    provider_path: PathBuf,
    runtime: RuntimeOptions,
) -> Result<()> {
    #[cfg(feature = "hot-reload")]
    apply_hot_reload_config(&runtime, &provider_path);

    #[cfg(feature = "audit-logging")]
    let audit_context = build_audit_context(&runtime)?;

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

fn default_provider_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("SPECADO_DEFAULT_PROVIDER") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Ok(path);
        }
        return Err(anyhow!(
            "Provider path specified via SPECADO_DEFAULT_PROVIDER does not exist: {}",
            path.display()
        ));
    }

    let fallback = PathBuf::from("crates/specado-providers/providers/openai/gpt-5/base.yaml");
    if fallback.exists() {
        return Ok(fallback);
    }

    Err(anyhow!(
        "Default provider spec not found. Set SPECADO_DEFAULT_PROVIDER to a valid provider YAML."
    ))
}

#[derive(Args, Default, Clone)]
struct RuntimeOptions {
    /// Enable experimental hot-reload and watch for spec changes
    #[arg(long)]
    watch: bool,
    /// Additional directories to watch for provider updates
    #[arg(long = "watch-provider-dir")]
    watch_dirs: Vec<PathBuf>,
    /// Audit logging target: stdout or file
    #[arg(long = "audit-target")]
    audit_target: Option<AuditTargetChoice>,
    /// Audit log file path (required when --audit-target file)
    #[arg(long = "audit-file")]
    audit_file: Option<PathBuf>,
    /// Additional case-insensitive redaction patterns
    #[arg(long = "audit-redact")]
    audit_redact: Vec<String>,
}

#[derive(Clone)]
enum AuditTargetChoice {
    Stdout,
    File,
}

impl std::str::FromStr for AuditTargetChoice {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "stdout" => Ok(Self::Stdout),
            "file" => Ok(Self::File),
            other => Err(format!(
                "Unsupported audit target '{other}'. Use stdout or file."
            )),
        }
    }
}

#[cfg(feature = "hot-reload")]
fn apply_hot_reload_config(options: &RuntimeOptions, provider_path: &Path) {
    if !options.watch {
        return;
    }

    let mut paths = if options.watch_dirs.is_empty() {
        vec![provider_path.to_path_buf()]
    } else {
        options.watch_dirs.clone()
    };

    if paths.is_empty() {
        paths.push(provider_path.to_path_buf());
    }

    let config = HotReloadConfig::enabled(paths, Duration::from_millis(250));
    set_global_config(config);
    eprintln!(
        "{} Hot reload is experimental; no watcher is started until the feature is fully implemented.",
        "⚠".yellow()
    );
}

#[cfg(feature = "audit-logging")]
fn build_audit_context(options: &RuntimeOptions) -> Result<Option<AuditContext>> {
    if options.audit_file.is_some()
        && !matches!(options.audit_target, Some(AuditTargetChoice::File))
    {
        return Err(anyhow!(
            "--audit-file can only be used with --audit-target file"
        ));
    }

    let target = match &options.audit_target {
        None if options.audit_redact.is_empty() => return Ok(None),
        None => Some(AuditTarget::Stdout),
        Some(AuditTargetChoice::Stdout) => Some(AuditTarget::Stdout),
        Some(AuditTargetChoice::File) => {
            let path = options
                .audit_file
                .clone()
                .ok_or_else(|| anyhow!("--audit-file is required when --audit-target file"))?;
            Some(AuditTarget::File { path })
        }
    };

    let config = AuditConfig {
        target,
        redact: options.audit_redact.clone(),
    };

    if !config.is_enabled() {
        return Ok(None);
    }

    eprintln!(
        "{} Audit logging is experimental and currently writes JSONL synchronously.",
        "⚠".yellow()
    );

    Ok(Some(AuditContext::new(config)))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    #[cfg(feature = "audit-logging")]
    fn audit_file_requires_path() {
        let opts = RuntimeOptions {
            watch: false,
            watch_dirs: vec![],
            audit_target: Some(AuditTargetChoice::File),
            audit_file: None,
            audit_redact: vec![],
        };

        let err = build_audit_context(&opts).expect_err("file target needs path");
        assert!(err.to_string().contains("--audit-file"));
    }

    #[test]
    #[cfg(feature = "audit-logging")]
    fn audit_flags_validate_mutual_exclusion() {
        let opts = RuntimeOptions {
            watch: false,
            watch_dirs: vec![],
            audit_target: Some(AuditTargetChoice::Stdout),
            audit_file: Some(PathBuf::from("ignored.jsonl")),
            audit_redact: vec![],
        };

        let err =
            build_audit_context(&opts).expect_err("should reject conflicting audit flag values");
        assert!(err
            .to_string()
            .contains("--audit-file can only be used with --audit-target file"));
    }

    #[test]
    #[cfg(feature = "audit-logging")]
    fn audit_file_requires_explicit_file_target() {
        let opts = RuntimeOptions {
            watch: false,
            watch_dirs: vec![],
            audit_target: None,
            audit_file: Some(PathBuf::from("audit.jsonl")),
            audit_redact: vec![],
        };

        let err =
            build_audit_context(&opts).expect_err("audit file should require --audit-target file");
        assert!(err
            .to_string()
            .contains("--audit-file can only be used with --audit-target file"));
    }

    #[test]
    fn runtime_options_default_watch_is_disabled() {
        let opts = RuntimeOptions::default();
        assert!(!opts.watch);
    }
}
