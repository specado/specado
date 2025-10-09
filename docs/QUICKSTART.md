# Specado Quickstart

This guide walks through using Specado from the command line, Python, and Node.js. It assumes you are working from a checkout of this repository and references the sample assets that ship in `examples/`.

## Prerequisites
- Rust 1.75 or newer with Cargo in your PATH
- Python 3.9+ with a working `pip` (for bindings)
- Node.js 18, 20, or 22 with `npm`
- OpenAI and Anthropic API keys exported as environment variables when executing against live providers (`OPENAI_API_KEY`, `ANTHROPIC_API_KEY`)

## 1. Clone and build the workspace
```sh
# fetch sources
 git clone https://github.com/yourorg/specado.git
 cd specado

# compile all crates once to prime the build cache
 cargo build --workspace
```

## 2. CLI walkthrough
The CLI lives in `crates/specado-cli` and exposes `validate`, `preview`, and `run` subcommands. The examples below reference the sample prompt and provider specs added under `examples/`.

```sh
# build the CLI binary
cargo build -p specado-cli

# validate a provider or prompt spec
./target/debug/specado validate --spec crates/specado-providers/providers/openai/gpt-5-family.yaml
./target/debug/specado validate --spec examples/prompts/basic_chat.json

# preview the translated payload and lossiness report (no network call)
./target/debug/specado preview \
  --prompt examples/prompts/basic_chat.json \
  --provider crates/specado-providers/providers/openai/gpt-5-family.yaml

# execute a prompt against the provider (requires API credentials)
OPENAI_API_KEY=sk-your-key \
./target/debug/specado run \
  --prompt examples/prompts/basic_chat.json \
  --provider crates/specado-providers/providers/openai/gpt-5-family.yaml \
  --audit-target stdout
```

The helper script `examples/cli_preview.sh` wraps the preview command and documents the expected environment when you need a quick smoke test.

> **Model metadata**
> The sample prompt `examples/prompts/basic_chat.json` includes metadata that targets OpenAI's GPT-5 Responses API (`openai_model`, reasoning effort, verbosity, max output tokens) and Anthropic's Claude Sonnet 4.5 (`anthropic_model`, thinking configuration, and max tokens). Tweak these fields to match the models and limits available to your account.

## 3. Python quickstart
The Python bindings are published from `crates/specado-py` and surfaced to the Python package in `python/specado`. Use `maturin` to build the native extension in-place, then run the sample program in `examples/python_basic.py`.

```sh
# install maturin if you have not already
pip install maturin

# build and install the extension in editable mode
maturin develop -m crates/specado-py/Cargo.toml

# run the sample which instantiates the high-level client
OPENAI_API_KEY=sk-your-key \
python examples/python_basic.py \
  --provider crates/specado-providers/providers/openai/gpt-5-family.yaml \
  --prompt examples/prompts/basic_chat.json
```

The script demonstrates both the native `Client` and the OpenAI compatibility layer. Swap the provider path or prompt spec as needed for Anthropic fixtures.

## 4. Node.js quickstart
The Node binding under `crates/specado-node` is built with napi-rs. After installing dependencies and compiling the native module, you can execute `examples/node_basic.mjs`.

```sh
# install dependencies and build
(cd crates/specado-node && npm install && npm run build)

# run the example using the transpiled JavaScript entrypoint
OPENAI_API_KEY=sk-your-key \
node examples/node_basic.mjs \
  --provider crates/specado-providers/providers/openai/gpt-5-family.yaml \
  --prompt examples/prompts/basic_chat.json
```

By default the sample logs the normalized response. Pass `--audit` to enable the audit logging stub described in the script.

## 5. Validate the environment
Before contributing changes upstream, run the standard quality gates:
```sh
cargo fmt
cargo clippy -- -D warnings
cargo test --workspace
pytest
(cd crates/specado-node && npm test)
```

These commands are also captured in `docs/process/SPECADO_PLAN.md` and enforced in CI. With the quickstart complete you can proceed to the migration notes in `docs/MIGRATION.md` and the testing guides under `tests/`.
