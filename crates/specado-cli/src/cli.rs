//! Command-line interface argument parsing and definitions
//!
//! This module defines the CLI structure using clap's derive API,
//! providing a type-safe and well-documented command interface.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Specado CLI - Spec-driven LLM prompt translation and validation
///
/// A powerful command-line tool for validating, previewing, and executing
/// LLM prompts across different providers with comprehensive translation support.
#[derive(Parser, Debug)]
#[command(
    name = "specado",
    version,
    author,
    about,
    long_about = None,
    propagate_version = true,
    arg_required_else_help = true
)]
pub struct Cli {
    /// Enable verbose output (can be used multiple times for increased verbosity)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Suppress all non-essential output
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Path to configuration file
    #[arg(short, long, global = true, env = "SPECADO_CONFIG")]
    pub config: Option<PathBuf>,

    /// Output format for results
    #[arg(short, long, value_enum, global = true, default_value = "human")]
    pub output: OutputFormat,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// The subcommand to run
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Validate a prompt specification against the schema
    Validate(ValidateArgs),
    
    /// Preview the translation of a prompt to provider format
    Preview(PreviewArgs),
    
    /// Translate and execute a prompt against a provider (L2 feature)
    Translate(TranslateArgs),
    
    /// Execute a provider request and get the normalized response
    Run(RunArgs),
    
    /// Manage configuration files and settings
    Config(ConfigArgs),
    
    /// Generate shell completions for the specified shell
    Completions(CompletionsArgs),
}

/// Arguments for the validate command
#[derive(Parser, Debug)]
pub struct ValidateArgs {
    /// Path to the prompt specification file (JSON or YAML)
    #[arg(value_name = "PROMPT_SPEC")]
    pub prompt_spec: PathBuf,

    /// Validation strictness mode
    #[arg(short, long, value_enum, default_value = "warn")]
    pub strict: StrictMode,

    /// Schema version to validate against
    #[arg(long)]
    pub schema_version: Option<String>,

    /// Show detailed validation errors
    #[arg(long)]
    pub detailed: bool,
}

/// Arguments for the preview command
#[derive(Parser, Debug)]
pub struct PreviewArgs {
    /// Path to the prompt specification file (JSON or YAML)
    #[arg(value_name = "PROMPT_SPEC")]
    pub prompt_spec: PathBuf,

    /// Provider name or path to provider specification
    #[arg(short, long, value_name = "PROVIDER")]
    pub provider: String,

    /// Model ID to use for translation
    #[arg(short, long)]
    pub model: String,

    /// Translation strictness mode
    #[arg(short, long, value_enum, default_value = "warn")]
    pub strict: StrictMode,

    /// Show lossiness report
    #[arg(long)]
    pub show_lossiness: bool,

    /// Show translation metadata
    #[arg(long)]
    pub show_metadata: bool,

    /// Highlight differences from original
    #[arg(long)]
    pub diff: bool,

    /// Output file path (stdout if not specified)
    #[arg(long = "save-to")]
    pub output_file: Option<PathBuf>,
}

/// Arguments for the translate command (placeholder for L2)
#[derive(Parser, Debug)]
pub struct TranslateArgs {
    /// Path to the prompt specification file (JSON or YAML)
    #[arg(value_name = "PROMPT_SPEC")]
    pub prompt_spec: PathBuf,

    /// Provider name or path to provider specification
    #[arg(short, long, value_name = "PROVIDER")]
    pub provider: String,

    /// Model ID to use for translation
    #[arg(short, long)]
    pub model: String,

    /// API key for the provider (can also be set via environment)
    #[arg(long, env = "SPECADO_API_KEY")]
    pub api_key: Option<String>,

    /// Base URL override for the provider
    #[arg(long)]
    pub base_url: Option<String>,

    /// Translation strictness mode
    #[arg(short, long, value_enum, default_value = "warn")]
    pub strict: StrictMode,

    /// Enable streaming mode
    #[arg(long)]
    pub stream: bool,

    /// Maximum retries for failed requests
    #[arg(long, default_value = "3")]
    pub max_retries: u32,

    /// Timeout in seconds
    #[arg(long, default_value = "30")]
    pub timeout: u64,

    /// Save raw response to file
    #[arg(long)]
    pub save_raw: Option<PathBuf>,
}

/// Arguments for the run command (Issue #56, #57, #58)
#[derive(Parser, Debug)]
pub struct RunArgs {
    /// Path to the provider request JSON file containing provider_spec, model_id, and request_body
    #[arg(value_name = "REQUEST_FILE")]
    pub request_file: PathBuf,

    /// Save the normalized response to a file (Issue #57)
    #[arg(long = "save-to", value_name = "OUTPUT_FILE")]
    pub save_to: Option<PathBuf>,

    /// Show execution metrics (timing, tokens, etc.) (Issue #58)
    #[arg(long)]
    pub metrics: bool,

    /// Pretty-print JSON output
    #[arg(long)]
    pub pretty: bool,

    /// Suppress all output except the response
    #[arg(long)]
    pub silent: bool,
}

/// Arguments for the config command
#[derive(Parser, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: ConfigAction,
}

/// Configuration management actions
#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Initialize default configuration files
    Init(ConfigInitArgs),
    
    /// Show current configuration values
    Show(ConfigShowArgs),
    
    /// Set a configuration value
    Set(ConfigSetArgs),
    
    /// Get a configuration value
    Get(ConfigGetArgs),
    
    /// List available profiles
    Profiles,
    
    /// Validate current configuration
    Validate,
}

/// Arguments for config init
#[derive(Parser, Debug)]
pub struct ConfigInitArgs {
    /// Initialize user config (~/.specado/config.toml)
    #[arg(long)]
    pub user: bool,
    
    /// Initialize project config (.specado.toml)
    #[arg(long)]
    pub project: bool,
    
    /// Force overwrite existing config files
    #[arg(long)]
    pub force: bool,
}

/// Arguments for config show
#[derive(Parser, Debug)]
pub struct ConfigShowArgs {
    /// Show configuration in specified format
    #[arg(short, long, value_enum, default_value = "toml")]
    pub format: ConfigFormat,
    
    /// Show only user configuration
    #[arg(long)]
    pub user_only: bool,
    
    /// Show only project configuration
    #[arg(long)]
    pub project_only: bool,
    
    /// Show merged configuration (default)
    #[arg(long)]
    pub merged: bool,
}

/// Arguments for config set
#[derive(Parser, Debug)]
pub struct ConfigSetArgs {
    /// Configuration key (e.g., default_provider, output.format)
    pub key: String,
    
    /// Configuration value
    pub value: String,
    
    /// Set in user config (~/.specado/config.toml)
    #[arg(long)]
    pub user: bool,
    
    /// Set in project config (.specado.toml)
    #[arg(long)]
    pub project: bool,
    
    /// Profile to modify (if not specified, modifies global config)
    #[arg(long)]
    pub profile: Option<String>,
}

/// Arguments for config get
#[derive(Parser, Debug)]
pub struct ConfigGetArgs {
    /// Configuration key (e.g., default_provider, output.format)
    pub key: String,
    
    /// Output format
    #[arg(short, long, value_enum, default_value = "value")]
    pub format: ConfigGetFormat,
}

/// Configuration file formats
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum ConfigFormat {
    /// TOML format
    Toml,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
}

/// Configuration get output formats
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum ConfigGetFormat {
    /// Just the value
    Value,
    /// JSON formatted
    Json,
}

/// Arguments for generating shell completions
#[derive(Parser, Debug)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: Shell,
}

/// Output format options
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable formatted output
    Human,
    /// JSON output
    Json,
    /// YAML output
    Yaml,
    /// Pretty-printed JSON output
    JsonPretty,
}

/// Strictness mode for validation and translation
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum StrictMode {
    /// Fail on any validation issue
    Strict,
    /// Warn on issues but continue
    Warn,
    /// Attempt to coerce values to valid ranges
    Coerce,
}

/// Supported shells for completion generation
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Shell {
    /// Bash shell
    Bash,
    /// Zsh shell
    Zsh,
    /// Fish shell
    Fish,
    /// PowerShell
    PowerShell,
    /// Elvish shell
    Elvish,
}

impl Cli {
    /// Parse command-line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Get the effective verbosity level (considering quiet flag)
    pub fn verbosity_level(&self) -> u8 {
        if self.quiet {
            0
        } else {
            self.verbose
        }
    }

    /// Check if colored output should be used
    pub fn use_color(&self) -> bool {
        !self.no_color && atty::is(atty::Stream::Stdout)
    }
}

impl From<StrictMode> for specado_core::StrictMode {
    fn from(mode: StrictMode) -> Self {
        match mode {
            StrictMode::Strict => specado_core::StrictMode::Strict,
            StrictMode::Warn => specado_core::StrictMode::Warn,
            StrictMode::Coerce => specado_core::StrictMode::Coerce,
        }
    }
}

impl Shell {
    /// Convert to clap_complete shell type
    pub fn to_clap_shell(self) -> clap_complete::Shell {
        match self {
            Shell::Bash => clap_complete::Shell::Bash,
            Shell::Zsh => clap_complete::Shell::Zsh,
            Shell::Fish => clap_complete::Shell::Fish,
            Shell::PowerShell => clap_complete::Shell::PowerShell,
            Shell::Elvish => clap_complete::Shell::Elvish,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        // Verify that the CLI structure is valid
        Cli::command().debug_assert();
    }

    #[test]
    fn test_verbosity_level() {
        let cli = Cli {
            verbose: 2,
            quiet: false,
            config: None,
            output: OutputFormat::Human,
            no_color: false,
            command: Commands::Validate(ValidateArgs {
                prompt_spec: PathBuf::from("test.json"),
                strict: StrictMode::Warn,
                schema_version: None,
                detailed: false,
            }),
        };
        assert_eq!(cli.verbosity_level(), 2);

        let quiet_cli = Cli {
            verbose: 2,
            quiet: true,
            ..cli
        };
        assert_eq!(quiet_cli.verbosity_level(), 0);
    }
}