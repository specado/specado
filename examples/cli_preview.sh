#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SPECADO_BIN="${SPECADO_BIN:-$ROOT_DIR/target/debug/specado}"
PROMPT_SPEC="${PROMPT_SPEC:-$ROOT_DIR/examples/prompts/basic_chat.json}"
PROVIDER_SPEC="${PROVIDER_SPEC:-$ROOT_DIR/crates/specado-providers/providers/openai/gpt-5.yaml}"

if [ ! -x "$SPECADO_BIN" ]; then
  echo "specado binary not found at $SPECADO_BIN" >&2
  echo "Hint: run 'cargo build -p specado-cli' first." >&2
  exit 1
fi

exec "$SPECADO_BIN" preview \
  --prompt "$PROMPT_SPEC" \
  --provider "$PROVIDER_SPEC" \
  "$@"
