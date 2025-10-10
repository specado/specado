use crate::cli::RuntimeOptions;
use crate::io::parse_to_json_value;
use crate::runtime;
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use serde_json::{Map as JsonMap, Value as JsonValue};
use specado_core::{
    execute, Message, MessageRole, PromptSpec, ProviderSpec, ResponseConfig, SamplingConfig,
    StrictMode, UniformResponse,
};
use std::fs;
use std::io::Write;
use std::path::Path;
use tokio::io::AsyncBufReadExt;
use tokio::signal;

pub fn base_system_messages() -> Vec<Message> {
    vec![Message {
        role: MessageRole::System,
        content: "You are a helpful assistant.".to_string(),
    }]
}

pub fn ensure_system_message(messages: &mut Vec<Message>) {
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

pub fn load_history_messages(path: &Path) -> Result<Vec<Message>> {
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

pub fn build_prompt_spec(
    messages: &[Message],
    sampling: &SamplingConfig,
    metadata: &JsonMap<String, JsonValue>,
) -> PromptSpec {
    PromptSpec {
        version: "1".to_string(),
        messages: messages.to_vec(),
        sampling: sampling.clone(),
        response: ResponseConfig::default(),
        tools: Vec::new(),
        tool_choice: None,
        strict_mode: StrictMode::Warn,
        metadata: metadata
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    }
}

pub async fn execute_messages(
    messages: &[Message],
    provider_path: &Path,
    runtime_options: &RuntimeOptions,
    sampling: &SamplingConfig,
    metadata: &JsonMap<String, JsonValue>,
) -> Result<UniformResponse> {
    let provider_str = provider_path
        .to_str()
        .ok_or_else(|| anyhow!("Provider path contains invalid UTF-8"))?;

    #[cfg(feature = "audit-logging")]
    let audit_context = runtime::build_audit_context(runtime_options)?;

    let response = execute(
        build_prompt_spec(messages, sampling, metadata),
        provider_str,
        #[cfg(feature = "audit-logging")]
        audit_context,
    )
    .await?;

    Ok(response)
}

pub fn warn_if_context_near_limit(messages: &[Message], provider: &ProviderSpec) {
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

pub async fn run_interactive_chat(
    initial_prompt: Option<String>,
    mut messages: Vec<Message>,
    metadata: &JsonMap<String, JsonValue>,
    provider_path: &Path,
    provider_spec: &ProviderSpec,
    runtime_options: &RuntimeOptions,
    sampling: &SamplingConfig,
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
            match execute_messages(
                &messages,
                provider_path,
                runtime_options,
                sampling,
                metadata,
            )
            .await
            {
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

        match execute_messages(
            &messages,
            provider_path,
            runtime_options,
            sampling,
            metadata,
        )
        .await
        {
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
