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
        .env(token_env, "cli-secret")
        .arg("run")
        .arg("--prompt")
        .arg(&prompt_path)
        .arg("--provider")
        .arg(&provider_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("hi from cli"));

    mock.assert_hits(1);
}

#[test]
fn ask_uses_default_provider_for_single_turn() {
    let dir = TempDir::new().expect("temp dir");

    let server = MockServer::start();
    let token_env = "SPECADO_ASK_TOKEN";

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

    let mut cmd = Command::cargo_bin("specado").expect("binary");
    cmd.env("NO_COLOR", "1")
        .env(token_env, "ask-secret")
        .env("SPECADO_DEFAULT_PROVIDER", &provider_path)
        .arg("ask")
        .arg("Hello there");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("hi from ask"));

    mock.assert_hits(1);
}

#[test]
fn ask_provider_and_model_flags_select_catalog_spec() {
    let dir = TempDir::new().expect("temp dir");
    let catalog_root = dir.path().join("providers");
    std::fs::create_dir_all(catalog_root.join("flag-provider")).expect("catalog provider dir");

    let server = MockServer::start();
    let token_env = "SPECADO_FLAG_TOKEN";

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

    let mut cmd = Command::cargo_bin("specado").expect("binary");
    cmd.env("NO_COLOR", "1")
        .env(token_env, "flag-secret")
        .env("SPECADO_PROVIDERS_DIR", &catalog_root)
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
}

#[test]
fn ask_model_flag_requires_provider() {
    let mut cmd = Command::cargo_bin("specado").expect("binary");
    cmd.env("NO_COLOR", "1")
        .env_remove("SPECADO_PROVIDERS_DIR")
        .env_remove("SPECADO_DEFAULT_PROVIDER")
        .arg("ask")
        .arg("Hello there")
        .arg("--model")
        .arg("gpt-5");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("--model requires --provider"));
}

#[test]
fn ask_interactive_chat_session_handles_multiple_turns() {
    let dir = TempDir::new().expect("temp dir");

    let server = MockServer::start();
    let token_env = "SPECADO_INTERACTIVE_TOKEN";

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/chat")
            .header("authorization", "Bearer chat-secret");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "data": {
                    "content": "response from interactive mode",
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
        .env(token_env, "chat-secret")
        .arg("ask")
        .arg("--interactive")
        .arg("--provider")
        .arg(&provider_path)
        .write_stdin("Hello there\nHow are you?\n:exit\n");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Starting interactive chat"))
        .stdout(predicate::str::contains("response from interactive mode"));

    mock.assert_hits(2);
}

#[test]
fn completions_generate_scripts() {
    let mut bash_cmd = Command::cargo_bin("specado").expect("binary");
    bash_cmd.env("NO_COLOR", "1").arg("completions").arg("bash");

    bash_cmd
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());

    let mut zsh_cmd = Command::cargo_bin("specado").expect("binary");
    zsh_cmd.env("NO_COLOR", "1").arg("completions").arg("zsh");

    zsh_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("compdef"));
}

#[test]
fn ask_interactive_uses_messages_file_history() {
    let dir = TempDir::new().expect("temp dir");

    let server = MockServer::start();
    let token_env = "SPECADO_HISTORY_TOKEN";
    let provider_path = write_file(
        &dir,
        "provider.yaml",
        sample_provider_yaml(&server.url("/chat"), token_env).as_str(),
    );

    let history_path = write_file(
        &dir,
        "history.json",
        &serde_json::to_string_pretty(&json!({
            "messages": [
                {"role": "system", "content": "You are tracking a project."},
                {"role": "user", "content": "Earlier question context"},
                {"role": "assistant", "content": "Previous assistant response"}
            ]
        }))
        .unwrap(),
    );

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/chat")
            .header("authorization", "Bearer history-secret")
            .body_contains("Earlier question context")
            .body_contains("Previous assistant response")
            .body_contains("Bring me up to speed");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "data": {
                    "content": "Continuing from history",
                    "finish_reason": "stop"
                }
            }));
    });

    let mut cmd = Command::cargo_bin("specado").expect("binary");
    cmd.env("NO_COLOR", "1")
        .env(token_env, "history-secret")
        .arg("ask")
        .arg("--interactive")
        .arg("--provider")
        .arg(&provider_path)
        .arg("--messages-file")
        .arg(&history_path)
        .arg("Bring me up to speed")
        .write_stdin(":exit\n");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Continuing from history"));

    mock.assert_hits(1);
}

#[test]
fn ask_interactive_rejects_invalid_messages_file() {
    let dir = TempDir::new().expect("temp dir");

    let invalid_path = write_file(&dir, "invalid.yaml", "not: messages");

    let server = MockServer::start();
    let token_env = "SPECADO_INVALID_TOKEN";
    let mock = server.mock(|when, then| {
        when.method(POST).path("/chat");
        then.status(200).body("{}");
    });

    let provider_path = write_file(
        &dir,
        "provider.yaml",
        sample_provider_yaml(&server.url("/chat"), token_env).as_str(),
    );

    let mut cmd = Command::cargo_bin("specado").expect("binary");
    cmd.env("NO_COLOR", "1")
        .env(token_env, "invalid-secret")
        .arg("ask")
        .arg("--interactive")
        .arg("--provider")
        .arg(&provider_path)
        .arg("--messages-file")
        .arg(&invalid_path)
        .write_stdin(":exit\n");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Messages file"));

    mock.assert_hits(0);
}
