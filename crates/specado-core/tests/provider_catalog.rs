use serde_json::json;
use specado_core::transformer::translate;
use specado_core::types::{
    LossinessCode, Message, MessageRole, PromptSpec, SamplingConfig, StrictMode,
};
use specado_core::ProviderSpec;
use specado_schemas::get_validator;
use std::fs;
use std::path::PathBuf;

fn provider_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../specado-providers/providers")
        .join(relative)
}

fn load_provider(relative: &str) -> ProviderSpec {
    let path = provider_path(relative);
    let contents = fs::read_to_string(&path).expect("provider yaml");
    serde_yaml::from_str(&contents).expect("valid provider spec")
}

fn sample_prompt() -> PromptSpec {
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
            ..Default::default()
        },
        response: Default::default(),
        tools: Vec::new(),
        tool_choice: None,
        strict_mode: StrictMode::Warn,
        metadata: Default::default(),
    }
}

#[test]
fn openai_catalog_validates_and_translates() {
    let provider = load_provider("openai/gpt-5.yaml");
    let validator = get_validator();

    let provider_json: serde_json::Value =
        serde_yaml::from_str(&fs::read_to_string(provider_path("openai/gpt-5.yaml")).unwrap())
            .expect("provider json");
    validator
        .validate_provider(&provider_json)
        .expect("provider spec valid");

    let (translated, report) = translate(&sample_prompt(), &provider).expect("translate");
    assert!(!report.is_lossy);
    assert_eq!(translated["temperature"], json!(1.5));
    assert_eq!(translated["top_p"], json!(0.9));
}

#[test]
fn anthropic_catalog_relocates_and_clamps() {
    let provider = load_provider("anthropic/claude-sonnet-45.yaml");
    let validator = get_validator();
    let provider_json: serde_json::Value = serde_yaml::from_str(
        &fs::read_to_string(provider_path("anthropic/claude-sonnet-45.yaml")).unwrap(),
    )
    .expect("provider json");
    validator
        .validate_provider(&provider_json)
        .expect("provider spec valid");

    let (translated, report) = translate(&sample_prompt(), &provider).expect("translate");

    assert!(report.is_lossy);
    let codes: Vec<_> = report.entries.iter().map(|entry| entry.code).collect();
    assert!(codes.contains(&LossinessCode::Relocate));
    assert!(codes.contains(&LossinessCode::Clamp));

    assert_eq!(translated["temperature"], json!(1.0));
    match &translated["system"] {
        serde_json::Value::String(s) => assert_eq!(s, "You are helpful."),
        serde_json::Value::Array(arr) => {
            assert_eq!(arr.len(), 1);
            assert_eq!(arr[0], json!("You are helpful."));
        }
        other => panic!("unexpected system mapping: {other:?}"),
    }
}
