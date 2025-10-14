<p align="center">
  <img src="docs/assets/specado-logo.svg" alt="Specado logo" width="260" />
</p>

<h1 align="center">From Fragile Scripts to Bulletproof Specs</h1>

<p align="center">
  <a href="https://github.com/specado/specado/actions/workflows/ci.yml">
    <img alt="CI status" src="https://github.com/specado/specado/actions/workflows/ci.yml/badge.svg">
  </a>
  <a href="https://github.com/specado/specado/blob/master/LICENSE">
    <img alt="License" src="https://img.shields.io/badge/license-Apache--2.0-0A0A0A?label=license">
  </a>
  <img alt="Crates.io" src="https://img.shields.io/badge/cargo-0.2.2-3B82F6?logo=rust">
  <img alt="npm" src="https://img.shields.io/badge/npm-0.2.2-CB3837?logo=npm">
  <img alt="PyPI" src="https://img.shields.io/badge/pypi-0.2.2-3776AB?logo=pypi">
</p>

---

Specado replaces fragile prompt scripts with a spec-first workflow. Define your prompt, sampling knobs, and provider routing once in a `spec.yaml`, validate it, and run that same spec from the CLI, Python, Node.js, or Rust. A single resolver now lives inside the core, so every surface—CLI or binding—shares the same behaviour without re-implementing name → provider-path logic.

## Supported surfaces

| Surface | Package | What you get |
| --- | --- | --- |
| CLI | `specado-cli-temp` | `ask`, `validate`, `preview`, `run`, and shell completions. |
| Python | `specado` | Bundled providers, `Client.complete`, async interoperability. |
| Node.js | `specado` | Native N-API client with TypeScript definitions. |
| Rust | `specado` | Hybrid API (`execute`, `execute_from_path`, `ExecuteOptions`). |

> The provider catalog ships with every package. Override it only when you need custom specs.

## Installation

- **CLI**
  ```bash
  cargo install specado-cli-temp
  ```
- **Python**
  ```bash
  pip install specado
  ```
- **Node.js**
  ```bash
  npm install specado
  ```
- **Rust core**
  ```toml
  [dependencies]
  specado = "0.2.2"
  tokio = { version = "1", features = ["full"] }
  ```

### Credentials & provider catalogs

Specado never prompts for secrets. Every surface reads `SPECADO_*` variables from:

1. `~/.config/specado/.env` (Linux), `~/Library/Application Support/specado/.env` (macOS), or `%AppData%\specado\.env` (Windows).
2. A project-local `.env`.
3. The process environment.

Create the global file once and lock it down:

```bash
mkdir -p ~/.config/specado
cat <<'EOF' > ~/.config/specado/.env
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=anthropic-...
# Point at your own catalog (optional)
# SPECADO_PROVIDERS_DIR=/absolute/path/to/providers
# Pin a default provider for the CLI and bindings
# SPECADO_DEFAULT_PROVIDER=crates/specado-providers/providers/openai/gpt-5/base.yaml
EOF
chmod 600 ~/.config/specado/.env
```

- `SPECADO_PROVIDERS_DIR` lets you ship a custom provider tree. Bindings pass this path automatically when you use `providers_dir=…`.
- `SPECADO_DEFAULT_PROVIDER` is an escape hatch when you want the CLI or bindings to fall back to a specific spec.

Add a `.env` next to a project if you need per-repo overrides; exported environment variables always take precedence.

## CLI at a glance

```bash
# Validate schema + provider capabilities
specado validate --spec spec.yaml

# Preview translated payload + lossiness report
specado preview --prompt spec.yaml --provider openai

# Execute with friendly provider / model names
specado ask "Qualify this inbound lead." --provider openai --model gpt-5

# Interactive chat loop with history
specado ask --interactive --provider openai

# Generate shell completions
specado completions bash > /usr/local/share/bash-completion/completions/specado
```

See `specado --help` for the full matrix of flags (`--messages-file`, `--reason`, `--watch`, audit logging, etc.).

## Specs that travel everywhere

### PromptSpec essentials

| Field | Required | Purpose |
| --- | --- | --- |
| `version` | Required | Schema version (currently `"1"`). |
| `messages[]` | Required | Ordered turns (`system` / `user` / `assistant`). |
| `sampling` | Optional | Deterministic knobs (`temperature`, `top_p`, `seed`, …). |
| `response` | Optional | Output contract (`text`, `json`, or `json_schema`). |
| `tools` / `tool_choice` | Optional | Provider-agnostic tool definitions and selection. |
| `strict_mode` | Optional | `Warn` (default), `Strict`, or `Coerce` mismatch behaviour. |
| `metadata` | Optional | Free-form hints for adapters (routing, tracing, etc.). |

Authoritative references live in [`crates/specado-core/src/types/prompt.rs`](crates/specado-core/src/types/prompt.rs) and [`crates/specado-schemas/schemas/prompt-spec.v1.schema.json`](crates/specado-schemas/schemas/prompt-spec.v1.schema.json).

### Provider specs

Provider definitions reside in [`crates/specado-providers/providers/`](crates/specado-providers/providers/). Each provider folder contains:

- `provider`, `interface`, and `contract_version`.
- `models[]` and `capabilities` to advertise supported surfaces.
- `auth` blocks (env-var style).
- `endpoints` with `mappings` for prompt/response translation.
- `constraints` (lossiness hints, unsupported parameters).

Write your own provider YAML, point bindings at its parent directory, and the resolver will discover it automatically.

## Using specs (and when you don’t need one)

### 1. CLI workflow

`spec.yaml`

```yaml
version: "1"
metadata:
  name: lead-qualifier
messages:
  - role: system
    content: |
      Qualify this lead.
      Reply with QUALIFIED or NOT_QUALIFIED plus a one-line reason.
  - role: user
    content: "We're evaluating enterprise automation for 2,500 SDRs."
sampling:
  temperature: 0.6
  seed: 23
response:
  format: text
strict_mode: Warn
```

Run it:

```bash
export OPENAI_API_KEY=sk-...
specado validate --spec spec.yaml
specado preview --prompt spec.yaml --provider openai
specado ask "Summarize the latest lead and recommend next steps." \
  --provider openai \
  --model gpt-5
```

- The CLI respects the credential loading order described in [Credentials & provider catalogs](#credentials--provider-catalogs); export keys or place them in `.env`.
- `specado run` accepts the absolute path to a provider spec when you want explicit control.

### 2. Python binding

```python
from specado import Client, Message, PromptSpec

prompt = PromptSpec(
    messages=[
        Message(role="system", content="Qualify the lead in a single sentence."),
        Message(role="user", content="We run a 500 seat contact centre and need LLM automation."),
    ],
    sampling={"temperature": 0.4, "seed": 7},
)

client = Client("openai", model="gpt-5")
result = client.complete(prompt)
print(result["content"])
```

- Providers bundle with the wheel; override `providers_dir` for custom catalogs.
- `complete` accepts `PromptSpec` or a raw dict that matches the schema.

### 3. Node.js binding

```ts
import { Client } from "specado";

const prompt = {
  version: "1" as const,
  messages: [
    { role: "system", content: "Qualify the lead and return QUALIFIED or NOT_QUALIFIED." },
    { role: "user", content: "We're shortlisting enterprise automation partners." },
  ],
  sampling: { temperature: 0.5, seed: 11 },
};

async function main() {
  const client = new Client("openai", { model: "gpt-5" });
  const result = await client.complete(prompt);
  console.log(result.content);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
```

### 4. Rust core – spec path vs. in-memory prompt

```rust
use specado::{
    execute, execute_from_path, ExecuteOptions, Message, MessageRole, PromptSpec, Result, SamplingConfig,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Use friendly provider + model (resolver handles discovery)
    let prompt = PromptSpec {
        version: "1".into(),
        messages: vec![
            Message { role: MessageRole::System, content: "Return QUALIFIED or NOT_QUALIFIED.".into() },
            Message { role: MessageRole::User, content: "We have an enterprise GTM org of 2,000 reps.".into() },
        ],
        sampling: SamplingConfig { temperature: Some(0.5), seed: Some(13), ..Default::default() },
        ..Default::default()
    };

    let friendly = execute(
        prompt.clone(),
        "openai",
        ExecuteOptions::for_model("gpt-5"),
        None,
    )
    .await?;
    println!("Friendly resolver ⇒ {}", friendly.content);

    // Run against an explicit provider spec path (no resolver step)
    let direct = execute_from_path(
        prompt,
        "crates/specado-providers/providers/openai/gpt-5/base.yaml",
        None,
    )
    .await?;
    println!("Explicit path ⇒ {}", direct.content);

    Ok(())
}
```

## Contributing

We love pragmatic contributions. Before opening a PR:

1. Check the [open issues](https://github.com/specado/specado/issues) or start a discussion.
2. Run the suite: `cargo test --workspace`, `source python/.venv/bin/activate && python -m pytest`, `npm test`.
3. Follow [Conventional Commits](docs/process/CONTRIBUTING.md) for commit messages.

Every merge triggers automated packaging across crates.io, PyPI, and npm so the CLI, bindings, and core stay in sync.

---

Ready for more? Browse the [`docs/`](docs/) folder for provider authoring guides, golden tests, and integration patterns, or use the curated template in [`docs/README.registry.md`](docs/README.registry.md) when publishing to registries.
