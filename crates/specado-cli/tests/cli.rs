use assert_cmd::Command;
use httpmock::prelude::*;
use predicates::prelude::*;
use serde_json::json;
use std::fs::write;
use tempfile::TempDir;

fn write_file(dir: &TempDir, name: &str, contents: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    write(&path, contents).expect("write file");
    path
}

fn sample_prompt_json() -> serde_json::Value {
    json!({
        "version": "1",
        "messages": [
            {"role": "system", "content": "You are helpful."},
            {"role": "user", "content": "Hello"}
        ],
        "sampling": {
            "temperature": 0.5,
            "top_k": 10
        }
    })
}

fn sample_provider_yaml(url: &str, token_env: &str) -> String {
    format!(
        r#"provider: test
interface: text.generate
contract_version: "1.0.0"
auth:
  type: bearer
  token_env: {token_env}
models:
  - id: test-model
endpoints:
  chat:
    method: POST
    url: {url}
    headers:
      content-type: application/json
mappings:
  request:
    - from: $.messages
      to: $.body.messages
    - from: $.sampling.temperature
      to: $.body.temperature
      clamp: [0.0, 1.0]
  response:
    - from: $.data.content
      to: content
    - from: $.data.finish_reason
      to: finish_reason
constraints:
  supports:
    json_mode: true
    tools: true
"#,
        url = url,
        token_env = token_env
    )
}

fn provider_yaml_with_models(
    provider: &str,
    models: &[&str],
    url: &str,
    token_env: &str,
) -> String {
    let models_block = models
        .iter()
        .map(|id| format!("  - id: {}", id))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"provider: {provider}
interface: text.generate
contract_version: "1.0.0"

auth:
  type: bearer
  token_env: {token_env}

models:
{models_block}

endpoints:
  chat:
    method: POST
    url: {url}
    headers:
      content-type: application/json

mappings:
  request:
    - from: $.messages
      to: $.body.messages
  response:
    - from: $.data.content
      to: content
    - from: $.data.finish_reason
      to: finish_reason

constraints:
  supports:
    json_mode: true
    tools: true
"#,
        provider = provider,
        models_block = models_block,
        url = url,
        token_env = token_env
    )
}

#[test]
fn validate_prompt_and_provider() {
    let dir = TempDir::new().expect("temp dir");
    let prompt_path = write_file(
        &dir,
        "prompt.json",
        &serde_json::to_string_pretty(&sample_prompt_json()).unwrap(),
    );
    let provider_path = write_file(
        &dir,
        "provider.yaml",
        sample_provider_yaml("https://example.com", "SPECADO_CLI_VALID_TOKEN").as_str(),
    );

    // Prompt should validate
    let mut cmd = Command::cargo_bin("specado").expect("binary");
    cmd.env("NO_COLOR", "1")
        .arg("validate")
        .arg("--spec")
        .arg(&prompt_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Prompt spec is valid"));

    // Provider should validate
    let mut cmd = Command::cargo_bin("specado").expect("binary");
    cmd.env("NO_COLOR", "1")
        .arg("validate")
        .arg("--spec")
        .arg(&provider_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Provider spec is valid"));
}

#[test]
fn preview_outputs_translated_request_and_lossiness() {
    let dir = TempDir::new().expect("temp dir");
    let prompt_path = write_file(
        &dir,
        "prompt.json",
        &serde_json::to_string_pretty(&sample_prompt_json()).unwrap(),
    );
    let provider_path = write_file(
        &dir,
        "provider.yaml",
        sample_provider_yaml("https://example.com", "SPECADO_UNUSED").as_str(),
    );

    let mut cmd = Command::cargo_bin("specado").expect("binary");
    cmd.env("NO_COLOR", "1")
        .arg("preview")
        .arg("--prompt")
        .arg(&prompt_path)
        .arg("--provider")
        .arg(&provider_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("=== Translated Request ==="))
        .stdout(predicate::str::contains("=== Lossiness Report ==="));
}

#[test]
fn run_executes_against_provider() {
    let dir = TempDir::new().expect("temp dir");
    let prompt_path = write_file(
        &dir,
        "prompt.json",
        &serde_json::to_string_pretty(&sample_prompt_json()).unwrap(),
    );

    let server = MockServer::start();
    let token_env = "SPECADO_CLI_TOKEN";
    std::env::set_var(token_env, "cli-secret");

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/chat")
            .header("authorization", "Bearer cli-secret");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "data": {
                    "content": "hi from cli",
                    "finish_reason": "stop"
                }
            }));
    });

    let provider_path = write_file(
        &dir,
        "provider.yaml",
        sample_provider_yaml(&server.url("/chat"), token_env).as_str(),
    );

    let mut cmd = Command::cargo_bin("specado").expect("binary");
    cmd.env("NO_COLOR", "1")
        .arg("run")
        .arg("--prompt")
        .arg(&prompt_path)
        .arg("--provider")
        .arg(&provider_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("hi from cli"));

    mock.assert_hits(1);
    std::env::remove_var(token_env);
}

#[test]
fn ask_uses_default_provider_for_single_turn() {
    let dir = TempDir::new().expect("temp dir");

    let server = MockServer::start();
    let token_env = "SPECADO_ASK_TOKEN";
    std::env::set_var(token_env, "ask-secret");

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/chat")
            .header("authorization", "Bearer ask-secret");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "data": {
                    "content": "hi from ask",
                    "finish_reason": "stop"
                }
            }));
    });

    let provider_path = write_file(
        &dir,
        "provider.yaml",
        sample_provider_yaml(&server.url("/chat"), token_env).as_str(),
    );

    std::env::set_var("SPECADO_DEFAULT_PROVIDER", &provider_path);

    let mut cmd = Command::cargo_bin("specado").expect("binary");
    cmd.env("NO_COLOR", "1").arg("ask").arg("Hello there");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("hi from ask"));

    mock.assert_hits(1);
    std::env::remove_var("SPECADO_DEFAULT_PROVIDER");
    std::env::remove_var(token_env);
}

#[test]
fn ask_provider_and_model_flags_select_catalog_spec() {
    let dir = TempDir::new().expect("temp dir");
    let catalog_root = dir.path().join("providers");
    std::fs::create_dir_all(&catalog_root.join("flag-provider")).expect("catalog provider dir");

    let server = MockServer::start();
    let token_env = "SPECADO_FLAG_TOKEN";
    std::env::set_var(token_env, "flag-secret");

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/chat")
            .header("authorization", "Bearer flag-secret");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "data": {
                    "content": "flagged response",
                    "finish_reason": "stop"
                }
            }));
    });

    let provider_yaml = provider_yaml_with_models(
        "flag-provider",
        &["test-model", "alternate-model"],
        &server.url("/chat"),
        token_env,
    );
    let provider_spec_path = catalog_root.join("flag-provider").join("catalog.yaml");
    std::fs::write(&provider_spec_path, provider_yaml).expect("write provider spec");

    std::env::set_var("SPECADO_PROVIDERS_DIR", &catalog_root);

    let mut cmd = Command::cargo_bin("specado").expect("binary");
    cmd.env("NO_COLOR", "1")
        .arg("ask")
        .arg("Hello flags")
        .arg("--provider")
        .arg("flag-provider")
        .arg("--model")
        .arg("test-model");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("flagged response"));

    mock.assert_hits(1);
    std::env::remove_var("SPECADO_PROVIDERS_DIR");
    std::env::remove_var(token_env);
}

#[test]
fn ask_model_flag_requires_provider() {
    std::env::remove_var("SPECADO_PROVIDERS_DIR");
    std::env::remove_var("SPECADO_DEFAULT_PROVIDER");

    let mut cmd = Command::cargo_bin("specado").expect("binary");
    cmd.env("NO_COLOR", "1")
        .arg("ask")
        .arg("Hello there")
        .arg("--model")
        .arg("gpt-5");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("--model requires --provider"));
}
