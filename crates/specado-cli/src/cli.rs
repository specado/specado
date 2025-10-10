use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "specado")]
#[command(version, about = "Spec-driven LLM abstraction", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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

#[derive(Args, Default, Clone)]
pub struct RuntimeOptions {
    /// Enable experimental hot-reload and watch for spec changes
    #[arg(long)]
    pub watch: bool,
    /// Additional directories to watch for provider updates
    #[arg(long = "watch-provider-dir")]
    pub watch_dirs: Vec<PathBuf>,
    /// Audit logging target: stdout or file
    #[arg(long = "audit-target")]
    pub audit_target: Option<AuditTargetChoice>,
    /// Audit log file path (required when --audit-target file)
    #[arg(long = "audit-file")]
    pub audit_file: Option<PathBuf>,
    /// Additional case-insensitive redaction patterns
    #[arg(long = "audit-redact")]
    pub audit_redact: Vec<String>,
}

#[derive(Clone)]
pub enum AuditTargetChoice {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    #[allow(clippy::enum_variant_names)]
    Powershell,
    Elvish,
}

impl CompletionShell {
    pub fn to_clap_shell(self) -> clap_complete::Shell {
        match self {
            CompletionShell::Bash => clap_complete::Shell::Bash,
            CompletionShell::Zsh => clap_complete::Shell::Zsh,
            CompletionShell::Fish => clap_complete::Shell::Fish,
            CompletionShell::Powershell => clap_complete::Shell::PowerShell,
            CompletionShell::Elvish => clap_complete::Shell::Elvish,
        }
    }
}
