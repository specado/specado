use anyhow::{anyhow, Context, Result};
use clap::CommandFactory;
use clap::{Args, Parser, Subcommand, ValueEnum};
use colored::*;
use specado_core::{
    execute, translate as core_translate, LossinessLevel, LossinessReport, Message, MessageRole,
    PromptSpec, ProviderSpec, ResponseConfig, SamplingConfig, StrictMode, UniformResponse,
};
use specado_schemas::get_validator;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::time::Duration;

#[cfg(feature = "audit-logging")]
use specado_core::audit::{AuditConfig, AuditContext, AuditTarget};
use specado_core::hot_reload::ProviderCache;
#[cfg(feature = "hot-reload")]
use specado_core::hot_reload::{set_global_config, HotReloadConfig};
use tokio::io::AsyncBufReadExt;
use tokio::signal;

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
        prompt: Option<String>,
        /// Provider name or path to the provider spec
        #[arg(long)]
        provider: Option<String>,
        /// Model identifier for the chosen provider
        #[arg(long)]
        model: Option<String>,
        /// Start an interactive chat session that preserves history
        #[arg(long, alias = "chat")]
        interactive: bool,
        /// Load prior conversation history from a PromptSpec or messages file
        #[arg(long = "messages-file", value_name = "PATH", requires = "interactive")]
        messages_file: Option<PathBuf>,
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
    /// Generate shell completion scripts for supported shells
    Completions {
        /// Target shell to generate completions for
        #[arg(value_enum)]
        shell: CompletionShell,
    },
}

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
        } => ask_command(prompt, provider, model, interactive, messages_file, runtime).await,
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
        Commands::Completions { shell } => completions_command(shell),
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

async fn ask_command(
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
    apply_hot_reload_config(&runtime, &provider_path);

    if interactive {
        let mut history = if let Some(path) = messages_file {
            load_history_messages(&path)?
        } else {
            Vec::new()
        };
        ensure_system_message(&mut history);

        let provider_spec = ProviderCache::new()
            .load_or_read(&provider_path)
            .map_err(|err| {
                anyhow!(
                    "Failed to load provider spec {}: {}",
                    provider_path.display(),
                    err
                )
            })?;
        run_interactive_chat(prompt, history, &provider_path, &provider_spec, &runtime).await?;
    } else {
        let mut messages = base_system_messages();
        messages.push(Message {
            role: MessageRole::User,
            content: prompt.expect("prompt required when not interactive"),
        });
        let response = execute_messages(&messages, &provider_path, &runtime).await?;
        println!("{}", response.content.trim());
    }

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

fn resolve_provider_path(provider: Option<&str>, model: Option<&str>) -> Result<PathBuf> {
    if provider.is_none() {
        if let Some(model_flag) = model {
            if !model_flag.trim().is_empty() {
                return Err(anyhow!(
                    "--model requires --provider. Pass both flags or omit --model."
                ));
            }
        }
        return default_provider_path();
    }

    let provider_flag = provider.unwrap().trim();
    if provider_flag.is_empty() {
        return Err(anyhow!("--provider cannot be empty"));
    }

    let cache = ProviderCache::new();
    let provider_path = PathBuf::from(provider_flag);
    if provider_path.is_file() {
        validate_model_for_path(&provider_path, model, &cache)?;
        return Ok(provider_path);
    }

    if provider_path.is_dir() {
        return resolve_provider_from_dir(&provider_path, model, &cache);
    }

    if let Ok(resolved) = provider_path.canonicalize() {
        if resolved.is_file() {
            validate_model_for_path(&resolved, model, &cache)?;
            return Ok(resolved);
        }
    }

    let providers_dir = locate_providers_dir()?;
    let joined_path = providers_dir.join(provider_flag);
    if joined_path.is_file() {
        validate_model_for_path(&joined_path, model, &cache)?;
        return Ok(joined_path);
    }
    if joined_path.is_dir() {
        return resolve_provider_from_dir(&joined_path, model, &cache);
    }

    if provider_flag.contains(std::path::MAIN_SEPARATOR)
        || provider_flag.contains('/')
        || provider_flag.contains('\\')
        || provider_flag.ends_with(".yaml")
        || provider_flag.ends_with(".yml")
    {
        return Err(anyhow!(
            "Provider spec '{}' not found. Pass an existing path or provider name.",
            provider_flag
        ));
    }

    let provider_dir = providers_dir.join(provider_flag);
    if !provider_dir.is_dir() {
        let available = list_available_providers(&providers_dir)?;
        let hint = if available.is_empty() {
            "No providers found in the catalog. Set SPECADO_PROVIDERS_DIR or pass a provider spec path."
                .to_string()
        } else {
            format!("Known providers: {}", available.join(", "))
        };
        return Err(anyhow!("Unknown provider '{}'. {}", provider_flag, hint));
    }

    resolve_provider_from_dir(&provider_dir, model, &cache)
}

fn validate_model_for_path(path: &Path, model: Option<&str>, cache: &ProviderCache) -> Result<()> {
    let Some(model_id) = model.filter(|m| !m.trim().is_empty()) else {
        return Ok(());
    };

    let spec = cache
        .load_or_read(path)
        .map_err(|err| anyhow!("Failed to load provider spec {}: {}", path.display(), err))?;

    if spec
        .models
        .iter()
        .any(|entry| entry.id.eq_ignore_ascii_case(model_id))
    {
        return Ok(());
    }

    let available: Vec<String> = spec.models.iter().map(|m| m.id.clone()).collect();
    if available.is_empty() {
        return Err(anyhow!(
            "Provider spec {} does not list any models; cannot validate --model {}",
            path.display(),
            model_id
        ));
    }

    Err(anyhow!(
        "Model '{}' not available in {}. Available models: {}",
        model_id,
        path.display(),
        available.join(", ")
    ))
}

fn resolve_provider_from_dir(
    dir: &Path,
    model: Option<&str>,
    cache: &ProviderCache,
) -> Result<PathBuf> {
    let candidates = collect_provider_candidates(dir, cache)?;
    if candidates.is_empty() {
        return Err(anyhow!(
            "No provider specifications found under {}",
            dir.display()
        ));
    }

    if let Some(model_id) = model.filter(|m| !m.trim().is_empty()) {
        if let Some(candidate) = candidates.iter().find(|candidate| {
            candidate
                .models
                .iter()
                .any(|id| id.eq_ignore_ascii_case(model_id))
        }) {
            return Ok(candidate.path.clone());
        }

        let available = collect_unique_models(&candidates);
        let provider_name = dir
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .unwrap_or_else(|| dir.to_string_lossy().into_owned());

        if available.is_empty() {
            return Err(anyhow!(
                "Model '{}' not found for provider '{}'. Specify a provider spec path instead.",
                model_id,
                provider_name
            ));
        }

        return Err(anyhow!(
            "Model '{}' not found for provider '{}'. Available models: {}",
            model_id,
            provider_name,
            available.join(", ")
        ));
    }

    if let Some(candidate) = pick_default_candidate(&candidates) {
        return Ok(candidate.path.clone());
    }

    Err(anyhow!(
        "Unable to determine a default spec for {}. Pass --model to disambiguate.",
        dir.display()
    ))
}

fn collect_provider_candidates(
    dir: &Path,
    cache: &ProviderCache,
) -> Result<Vec<ProviderCandidate>> {
    let mut stack = vec![dir.to_path_buf()];
    let mut candidates = Vec::new();

    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current)
            .with_context(|| format!("Failed to read provider directory: {}", current.display()))?
        {
            let entry = entry.with_context(|| {
                format!("Failed to read provider entry under {}", current.display())
            })?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if !is_spec_file(&path) {
                continue;
            }

            let spec = cache.load_or_read(&path).map_err(|err| {
                anyhow!("Failed to load provider spec {}: {}", path.display(), err)
            })?;

            let models = spec.models.iter().map(|m| m.id.clone()).collect();
            candidates.push(ProviderCandidate { path, models });
        }
    }

    Ok(candidates)
}

fn is_spec_file(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    if file_name.starts_with('_') || file_name.ends_with(".md") {
        return false;
    }

    matches!(
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext, "yaml" | "yml" | "json")),
        Some(true)
    )
}

fn pick_default_candidate(candidates: &[ProviderCandidate]) -> Option<&ProviderCandidate> {
    if candidates.len() == 1 {
        return candidates.first();
    }

    let is_named = |candidate: &ProviderCandidate, target: &str| {
        candidate
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.eq_ignore_ascii_case(target))
            .unwrap_or(false)
    };

    if let Some(candidate) = candidates.iter().find(|c| is_named(c, "base.yaml")) {
        return Some(candidate);
    }
    if let Some(candidate) = candidates.iter().find(|c| is_named(c, "base.yml")) {
        return Some(candidate);
    }
    if let Some(candidate) = candidates.iter().find(|c| is_named(c, "chat.yaml")) {
        return Some(candidate);
    }
    if let Some(candidate) = candidates.iter().find(|c| is_named(c, "chat.yml")) {
        return Some(candidate);
    }

    candidates
        .iter()
        .min_by_key(|candidate| candidate.path.to_string_lossy().to_ascii_lowercase())
}

fn collect_unique_models(candidates: &[ProviderCandidate]) -> Vec<String> {
    let mut models: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for candidate in candidates {
        for model in &candidate.models {
            models.insert(model.clone());
        }
    }
    models.into_iter().collect()
}

fn locate_providers_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("SPECADO_PROVIDERS_DIR") {
        let path = PathBuf::from(dir);
        if path.is_dir() {
            return Ok(path);
        }
        return Err(anyhow!(
            "SPECADO_PROVIDERS_DIR points to {}, which is not a directory",
            path.display()
        ));
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = manifest_dir.join("../specado-providers/providers");
    if workspace_dir.is_dir() {
        return Ok(workspace_dir);
    }

    let repo_relative = PathBuf::from("crates/specado-providers/providers");
    if repo_relative.is_dir() {
        return Ok(repo_relative);
    }

    Err(anyhow!(
        "Unable to locate provider catalog. Set SPECADO_PROVIDERS_DIR or pass a provider spec path."
    ))
}

fn list_available_providers(root: &Path) -> Result<Vec<String>> {
    let mut providers = Vec::new();
    for entry in fs::read_dir(root)
        .with_context(|| format!("Failed to read providers directory: {}", root.display()))?
    {
        let entry =
            entry.with_context(|| format!("Failed to read entry under {}", root.display()))?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                providers.push(name.to_string());
            }
        }
    }
    providers.sort_unstable();
    Ok(providers)
}

struct ProviderCandidate {
    path: PathBuf,
    models: Vec<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    #[allow(clippy::enum_variant_names)]
    Powershell,
    Elvish,
}

impl CompletionShell {
    fn to_clap_shell(self) -> clap_complete::Shell {
        match self {
            CompletionShell::Bash => clap_complete::Shell::Bash,
            CompletionShell::Zsh => clap_complete::Shell::Zsh,
            CompletionShell::Fish => clap_complete::Shell::Fish,
            CompletionShell::Powershell => clap_complete::Shell::PowerShell,
            CompletionShell::Elvish => clap_complete::Shell::Elvish,
        }
    }
}

fn base_system_messages() -> Vec<Message> {
    vec![Message {
        role: MessageRole::System,
        content: "You are a helpful assistant.".to_string(),
    }]
}

fn ensure_system_message(messages: &mut Vec<Message>) {
    let has_system = messages
        .iter()
        .any(|message| matches!(message.role, MessageRole::System));
    if !has_system {
        messages.insert(
            0,
            Message {
                role: MessageRole::System,
                content: "You are a helpful assistant.".to_string(),
            },
        );
    }
}

fn completions_command(shell: CompletionShell) -> Result<()> {
    use clap_complete::generate;
    use std::io;

    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    generate(shell.to_clap_shell(), &mut cmd, name, &mut io::stdout());

    Ok(())
}

fn load_history_messages(path: &Path) -> Result<Vec<Message>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read messages file: {}", path.display()))?;

    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    let value = parse_to_json_value(&content, path)?;

    if let Some(messages_value) = value.get("messages") {
        let messages: Vec<Message> =
            serde_json::from_value(messages_value.clone()).map_err(|err| {
                anyhow!(
                    "Failed to parse 'messages' array in {}: {}",
                    path.display(),
                    err
                )
            })?;
        return Ok(messages);
    }

    if value.is_array() {
        let messages: Vec<Message> = serde_json::from_value(value).map_err(|err| {
            anyhow!(
                "Failed to parse messages array in {}: {}",
                path.display(),
                err
            )
        })?;
        return Ok(messages);
    }

    Err(anyhow!(
        "Messages file {} must be a PromptSpec with a 'messages' array or an array of messages.",
        path.display()
    ))
}

fn build_prompt_spec(messages: &[Message]) -> PromptSpec {
    PromptSpec {
        version: "1".to_string(),
        messages: messages.to_vec(),
        sampling: SamplingConfig::default(),
        response: ResponseConfig::default(),
        tools: Vec::new(),
        tool_choice: None,
        strict_mode: StrictMode::Warn,
        metadata: Default::default(),
    }
}

async fn execute_messages(
    messages: &[Message],
    provider_path: &Path,
    runtime: &RuntimeOptions,
) -> Result<UniformResponse> {
    let provider_str = provider_path
        .to_str()
        .ok_or_else(|| anyhow!("Provider path contains invalid UTF-8"))?;

    #[cfg(feature = "audit-logging")]
    let audit_context = build_audit_context(runtime)?;

    let response = execute(
        build_prompt_spec(messages),
        provider_str,
        #[cfg(feature = "audit-logging")]
        audit_context,
    )
    .await?;

    Ok(response)
}

fn warn_if_context_near_limit(messages: &[Message], provider: &ProviderSpec) {
    let Some(window) = provider.capabilities.context_window else {
        return;
    };

    if window == 0 {
        return;
    }

    let total_chars: usize = messages.iter().map(|message| message.content.len()).sum();
    let approx_tokens = ((total_chars as f64) / 4.0).ceil() as u64;
    let threshold = ((window as f64) * 0.8).floor() as u64;

    if approx_tokens >= threshold && approx_tokens < window {
        println!(
            "{} Approaching context window (~{} of {} tokens). Consider clearing history.",
            "⚠".yellow(),
            approx_tokens,
            window
        );
    } else if approx_tokens >= window {
        println!(
            "{} Estimated conversation length (~{} tokens) exceeds provider context window ({}).",
            "⚠".yellow(),
            approx_tokens,
            window
        );
    }
}

async fn run_interactive_chat(
    initial_prompt: Option<String>,
    mut messages: Vec<Message>,
    provider_path: &Path,
    provider_spec: &ProviderSpec,
    runtime: &RuntimeOptions,
) -> Result<()> {
    println!(
        "{} Type ':exit' or ':quit' (or press Ctrl+C) to end the session.",
        "Starting interactive chat.".cyan()
    );

    if !messages.is_empty() {
        warn_if_context_near_limit(&messages, provider_spec);
    }

    if let Some(initial) = initial_prompt {
        if !initial.trim().is_empty() {
            messages.push(Message {
                role: MessageRole::User,
                content: initial,
            });
            warn_if_context_near_limit(&messages, provider_spec);
            match execute_messages(&messages, provider_path, runtime).await {
                Ok(response) => {
                    let content = response.content.clone();
                    println!("{}", content.trim());
                    messages.push(Message {
                        role: MessageRole::Assistant,
                        content,
                    });
                }
                Err(err) => {
                    eprintln!("{} {}", "Error:".red().bold(), err);
                    messages.pop();
                }
            }
        }
    }

    let stdin = tokio::io::stdin();
    let mut reader = tokio::io::BufReader::new(stdin);
    let mut line = String::new();

    loop {
        print!("{} ", ">>>".cyan());
        std::io::stdout()
            .flush()
            .with_context(|| "Failed to flush prompt")?;

        line.clear();
        let read = tokio::select! {
            result = reader.read_line(&mut line) => result,
            _ = signal::ctrl_c() => {
                println!();
                println!("{}", "Exiting interactive chat.".yellow());
                return Ok(());
            }
        }?;

        if read == 0 {
            println!();
            println!("{}", "End of input. Exiting interactive chat.".yellow());
            break;
        }

        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if matches!(trimmed.to_ascii_lowercase().as_str(), ":exit" | ":quit") {
            println!("{}", "Exiting interactive chat.".yellow());
            break;
        }

        messages.push(Message {
            role: MessageRole::User,
            content: trimmed.to_string(),
        });

        warn_if_context_near_limit(&messages, provider_spec);

        match execute_messages(&messages, provider_path, runtime).await {
            Ok(response) => {
                let content = response.content.clone();
                println!("{}", content.trim());
                messages.push(Message {
                    role: MessageRole::Assistant,
                    content,
                });
            }
            Err(err) => {
                eprintln!("{} {}", "Error:".red().bold(), err);
                messages.pop();
            }
        }
    }

    Ok(())
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
