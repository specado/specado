# Specado – Package README Template

<p align="right"><a href="https://github.com/specado/specado">github.com/specado/specado</a></p>

This template keeps the language-specific README files in sync with the core project. Copy the relevant slice into each registry-specific README (crates.io, PyPI, npm) and adjust badges/links as needed. Every example reflects the hybrid resolver introduced in `specado-core`, showing both spec-driven and explicit-path execution. All surfaces read credentials from `~/.config/specado/.env` (macOS: `~/Library/Application Support/specado/.env`, Windows: `%AppData%\specado\.env`), a project-local `.env`, or process environment variables.

---

## Rust CLI (`specado-cli-temp`)

### Installation

```bash
cargo install specado-cli-temp
```

### Usage

```bash
# Validate a spec before shipping it
specado validate --spec spec.yaml

# Preview the translated provider payload plus lossiness report
specado preview --prompt spec.yaml --provider openai

# Ask the provider directly (friendly names or explicit paths both work)
specado ask \
  "Summarize our new enterprise lead and recommend next steps." \
  --provider openai \
  --model gpt-5

# Execute against an explicit provider YAML (skip friendly lookup)
specado run --prompt spec.yaml --provider crates/specado-providers/providers/openai/gpt-5/base.yaml
```

- Populate `OPENAI_API_KEY`/`ANTHROPIC_API_KEY` in `~/.config/specado/.env` (or the platform equivalent) or export them before running.
- Key commands (see `specado --help` for flags): `ask`, `validate`, `preview`, `run`, and `completions`.

---

## Python (`specado`)

### Installation

```bash
pip install specado
```

### Usage

```python
from specado import Client, Message, PromptSpec

prompt = PromptSpec(
    messages=[
        Message(role="system", content="You qualify inbound leads."),
        Message(role="user", content="I run a 2,000 seat sales org and need automation help."),
    ],
    sampling={"temperature": 0.4, "seed": 7},
)

client = Client("openai", model="gpt-5")
result = client.complete(prompt)

print(result["content"])

# Bypass the resolver with an explicit provider catalog
direct = Client("crates/specado-providers/providers/openai/gpt-5/base.yaml")
print(direct.complete(prompt)["content"])
```

- `Client(provider, *, model=None, providers_dir=None, watch=None, audit_config=None)` accepts friendly names or explicit paths.
- Providers are bundled with the wheel. Override `providers_dir` to point to your own catalog.
- `complete` accepts either the `PromptSpec` helper or a raw prompt dictionary.

---

## Node.js (`specado`)

### Installation

```bash
npm install specado
```

### Usage

```ts
import { Client } from "specado";

const prompt = {
  version: "1" as const,
  messages: [
    { role: "system", content: "You qualify inbound leads." },
    { role: "user", content: "We're evaluating your enterprise platform." },
  ],
  sampling: { temperature: 0.6, seed: 11 },
  response: { format: "text" as const },
};

async function main() {
  const client = new Client("openai", { model: "gpt-5" });
  const result = await client.complete(prompt);
  console.log(result.content);

  // Explicit provider spec path (skip resolver and bundled catalog)
  const direct = new Client("crates/specado-providers/providers/openai/gpt-5/base.yaml");
  const directResult = await direct.complete(prompt);
  console.log(directResult.content);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
```

The constructor accepts either a friendly provider name or a spec path. Optional fields on `ClientOptions` include `model`, `providersDir`, `watch`, and `audit`.

---

## Rust Core (`specado`)

### Installation

```toml
[dependencies]
specado = "0.2.0"
tokio = { version = "1", features = ["full"] }
```

### Usage

```rust
use specado::{
    execute, execute_from_path, ExecuteOptions, Message, MessageRole, PromptSpec, Result, SamplingConfig, StrictMode,
};

#[tokio::main]
async fn main() -> Result<()> {
    let prompt = PromptSpec {
        version: "1".into(),
        messages: vec![
            Message { role: MessageRole::System, content: "Respond with QUALIFIED or NOT_QUALIFIED.".into() },
            Message { role: MessageRole::User, content: "We're expanding our enterprise GTM team." .into() },
        ],
        sampling: SamplingConfig {
            temperature: Some(0.5),
            seed: Some(13),
            ..Default::default()
        },
        strict_mode: StrictMode::Warn,
        ..Default::default()
    };

    let response = execute(
        prompt,
        "openai",
        ExecuteOptions::for_model("gpt-5"),
        None, // optional AuditContext when audit-logging is enabled
    )
    .await?;

    println!("{}", response.content);

    // Or run against an explicit provider spec path
    let direct = execute_from_path(
        PromptSpec {
            version: "1".into(),
            messages: vec![
                Message { role: MessageRole::System, content: "Respond with QUALIFIED or NOT_QUALIFIED.".into() },
                Message { role: MessageRole::User, content: "We're expanding our enterprise GTM team.".into() },
            ],
            sampling: SamplingConfig { temperature: Some(0.5), seed: Some(19), ..Default::default() },
            strict_mode: StrictMode::Warn,
            ..Default::default()
        },
        "crates/specado-providers/providers/openai/gpt-5/base.yaml",
        None,
    )
    .await?;

    println!("{}", direct.content);
    Ok(())
}
```

- Use `execute` for friendly provider names (with optional `ExecuteOptions`).
- Use `execute_from_path` when you already have a concrete spec on disk.
- `ExecuteOptions::with_providers_dir` lets you point at a custom catalog; bindings use the same helper internally.

---

## PromptSpec Reference (All Packages)

| Field | Type | Notes |
| --- | --- | --- |
| `version` | `"1"` | Schema version. |
| `messages[]` | `{ role: "system" \| "user" \| "assistant", content: string }` | Ordered conversation turns. |
| `sampling` | `{ temperature?, top_p?, top_k?, frequency_penalty?, presence_penalty?, seed? }` | Optional stochastic controls, all provider-neutral. |
| `response` | `{ format: "text" \| "json" \| "json_schema", json_schema? }` | Output contract and optional JSON schema. |
| `tools[]` | `{ name, description?, json_schema }` | Provider-agnostic function/tool declarations. |
| `tool_choice` | `"auto"` \| `"required"` \| `{ name }` | Selects which tool, when providers support it. |
| `strict_mode` | `"Strict"` \| `"Warn"` (default) \| `"Coerce"` | Controls model vs. prompt mismatches. |
| `metadata` | object | Free-form hints for adapters (routing, tracing, etc.). |

For a full specification, consult:

- [`crates/specado-core/src/types/prompt.rs`](../crates/specado-core/src/types/prompt.rs)
- [`crates/specado-schemas/schemas/prompt-spec.v1.schema.json`](../crates/specado-schemas/schemas/prompt-spec.v1.schema.json)

Keep the high-level README and this template in lockstep so every published artifact tells the same story.
