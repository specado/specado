# Examples Overview

This directory contains runnable samples that mirror the quickstart in `docs/QUICKSTART.md` and support Issue #50 (Docs, Tests, Benches, Examples Scaffolding).

## Assets
- `prompts/basic_chat.json` — Minimal chat prompt shared by all examples.
- `prompts/openai_reasoning.json` — GPT-5 prompt that exercises reasoning controls.
- `prompts/anthropic_thinking.json` — Claude Sonnet prompt with thinking enabled.
- `cli_preview.sh` — Minimal wrapper around `specado preview` for ad-hoc checks.
- `cli_demo.sh` — Runs the reasoning/thinking prompts via `specado preview`/`run`.
- `python_basic.py` — Python demo supporting `--scenario {openai-reasoning,anthropic-thinking}` plus OpenAI compatibility mode.
- `node_basic.mjs` — Node.js demo with the same scenarios and optional audit/watch toggles.

## Usage
Follow the instructions in the quickstart to build the CLI, Python extension, and Node module. Each script accepts optional flags to point at different specs, enable audit logging, or turn on watch plumbing.

### CLI

Ensure the CLI binary is built (`cargo build -p specado-cli`). Then:

```sh
./examples/cli_demo.sh
```

The script previews both demo prompts and executes live calls when `OPENAI_API_KEY`/`ANTHROPIC_API_KEY` are present. Set `SPECADO_BIN` to point at a custom binary if you prefer not to use `cargo run`.

### Python

After `maturin develop -m crates/specado-py/Cargo.toml`, invoke:

```sh
python examples/python_basic.py --scenario openai-reasoning
python examples/python_basic.py --scenario anthropic-thinking
```

Add `--openai-compat` on the first command to exercise the compatibility shim.

### Node.js

Build the napi module (`(cd crates/specado-node && npm install && npm run build)`) and run:

```sh
node examples/node_basic.mjs --scenario openai-reasoning
node examples/node_basic.mjs --scenario anthropic-thinking
```

Use `--audit` or `--watch` to toggle optional features.

When adapting a sample for a new provider or model, consult `docs/PROVIDER_SPEC.md` for the spec format and metadata keys that need to be supplied in the prompt.

These samples are intentionally lightweight; add new fixtures alongside them when demonstrating additional providers or prompt types.
