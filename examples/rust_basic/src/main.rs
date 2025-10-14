//! Specado Rust Example: Execute Prompts with the Hybrid API
//!
//! This example demonstrates two ways to execute prompts with Specado:
//! 1. Using the friendly provider name API with `execute()`
//! 2. Using an explicit provider path with `execute_from_path()`

use specado::{
    execute, execute_from_path, load_prompt_from_path, simple_prompt, ExecuteOptions, Result,
    SimplePromptOptions,
};
use std::{env, path::PathBuf};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Specado Rust Example: Hybrid API");
    println!("{}", "=".repeat(60));

    // Build a prompt specification in-memory using the helper constructor.
    let prompt = simple_prompt(SimplePromptOptions {
        message: Some("Explain what a static type system is in one paragraph.".into()),
        system: Some(
            "You are a helpful assistant that explains technical concepts clearly.".into(),
        ),
        temperature: Some(0.5),
        ..Default::default()
    })?;

    let providers_dir = resolve_providers_dir();

    if let Some(dir) = providers_dir.clone() {
        println!("\n1. Using friendly provider name (\"openai\")");
        println!("{}", "-".repeat(60));

        let mut options = ExecuteOptions::for_model("gpt-5");
        options.providers_dir = Some(dir.clone());

        let friendly_result = execute(prompt.clone(), "openai", options).await?;

        println!("Response: {}", friendly_result.content);
    } else {
        println!(
            "\n⚠️  Skipping friendly provider demo (set SPECADO_PROVIDERS_DIR \
             or build the providers catalog)."
        );
    }

    println!("\n2. Using explicit provider path");
    println!("{}", "-".repeat(60));

    if let Some(dir) = providers_dir {
        let explicit_path = dir.join("openai/gpt-5/base.yaml");
        println!("Using provider spec at: {}", explicit_path.display());

        let prompt_from_file = load_prompt_from_path(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../prompts/summarize_article.yaml"),
        )?;

        let explicit_result = execute_from_path(prompt_from_file, explicit_path).await?;

        println!("Response: {}", explicit_result.content);
    } else {
        println!(
            "⚠️  Skipping explicit provider demo (unable to locate provider catalog automatically)."
        );
    }

    println!("\n{}", "=".repeat(60));
    println!("Both approaches use the same prompt but different resolution strategies.");
    println!("The friendly name approach is recommended for most use cases.");

    Ok(())
}

fn resolve_providers_dir() -> Option<PathBuf> {
    if let Ok(env_dir) = env::var("SPECADO_PROVIDERS_DIR") {
        let dir = PathBuf::from(env_dir);
        if dir.exists() {
            return Some(dir);
        }
    }

    let bundled =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../crates/specado-providers/providers");
    if bundled.exists() {
        return Some(bundled);
    }

    None
}
