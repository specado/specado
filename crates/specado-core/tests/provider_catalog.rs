use serde_json::json;
use specado_core::hot_reload::ProviderCache;
use specado_core::transformer::translate;
use specado_core::types::{
    LossinessCode, Message, MessageRole, PromptSpec, SamplingConfig, StrictMode,
};
use specado_core::ProviderSpec;
use specado_schemas::get_validator;
use std::collections::HashMap;
use std::path::PathBuf;

fn provider_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../specado-providers/providers")
        .join(relative)
}

fn load_provider(relative: &str) -> ProviderSpec {
    let path = provider_path(relative);
    ProviderCache::new()
        .load_or_read(&path)
        .expect("merged provider spec")
}

fn sample_prompt() -> PromptSpec {
    let mut metadata: HashMap<String, serde_json::Value> = HashMap::new();
    metadata.insert("openai_model".into(), json!("gpt-5"));
    metadata.insert("openai_max_output_tokens".into(), json!(200));
    metadata.insert("openai_reasoning_effort".into(), json!("low"));
    metadata.insert("openai_text_verbosity".into(), json!("low"));
    metadata.insert(
        "anthropic_model".into(),
        json!("claude-sonnet-4-5-20250929"),
    );
    metadata.insert("anthropic_max_tokens".into(), json!(256));
    metadata.insert("anthropic_thinking_type".into(), json!("enabled"));
    metadata.insert("anthropic_thinking_budget".into(), json!(2000));

    PromptSpec {
        version: "1".into(),
        messages: vec![
            Message {
                role: MessageRole::System,
                content: "You are helpful.".into(),
            },
            Message {
                role: MessageRole::User,
                content: "Hello".into(),
            },
        ],
        sampling: SamplingConfig {
            temperature: Some(1.5),
            top_p: Some(0.9),
            frequency_penalty: Some(0.25),
            ..Default::default()
        },
        response: Default::default(),
        tools: Vec::new(),
        tool_choice: None,
        strict_mode: StrictMode::Warn,
        metadata,
    }
}

#[test]
fn openai_catalog_validates_and_translates() {
    let provider = load_provider("openai/gpt-5/base.yaml");
    let validator = get_validator();

    let provider_json = serde_json::to_value(&provider).expect("provider json");
    validator
        .validate_provider(&provider_json)
        .expect("provider spec valid");

    let (translated, report) = translate(&sample_prompt(), &provider).expect("translate");
    assert!(report.is_lossy);
    assert!(report
        .entries
        .iter()
        .any(|entry| entry.code == LossinessCode::Unsupported));
    assert_eq!(translated["model"], json!("gpt-5"));
    assert_eq!(translated["max_output_tokens"], json!(200));
    assert_eq!(translated["reasoning"]["effort"], json!("low"));
    assert_eq!(translated["text"]["verbosity"], json!("low"));
    assert_eq!(translated["instructions"], json!("You are helpful."));
    assert_eq!(translated["input"][0]["content"][0]["text"], json!("Hello"));
}

#[test]
fn anthropic_catalog_relocates_and_clamps() {
    let provider = load_provider("anthropic/claude-4.5/sonnet.yaml");
    let validator = get_validator();
    let provider_json = serde_json::to_value(&provider).expect("provider json");
    validator
        .validate_provider(&provider_json)
        .expect("provider spec valid");

    let (translated, report) = translate(&sample_prompt(), &provider).expect("translate");

    assert!(report.is_lossy);
    let codes: Vec<_> = report.entries.iter().map(|entry| entry.code).collect();
    assert!(codes.contains(&LossinessCode::Relocate));
    assert!(codes.contains(&LossinessCode::Clamp));

    assert_eq!(translated["model"], json!("claude-sonnet-4-5-20250929"));
    assert_eq!(translated["max_tokens"], json!(2064));
    assert_eq!(translated["system"], json!("You are helpful."));
    assert_eq!(translated["temperature"], json!(1.0));
    assert_eq!(
        translated["messages"][0]["content"][0]["text"],
        json!("Hello")
    );
}
