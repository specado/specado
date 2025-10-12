#!/usr/bin/env bash
# Convenience wrapper that previews and optionally runs the reasoning/thinking sample prompts.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

SPECADO_BIN="${SPECADO_BIN:-}"
if [[ -z "${SPECADO_BIN}" ]]; then
  if [[ -x "$REPO_ROOT/target/debug/specado" ]]; then
    SPECADO_BIN="$REPO_ROOT/target/debug/specado"
  else
    SPECADO_BIN="cargo run --quiet -p specado-cli --"
  fi
fi

OPENAI_PROMPT="$REPO_ROOT/examples/prompts/openai_reasoning.json"
OPENAI_SPEC="$REPO_ROOT/crates/specado-providers/providers/openai/gpt-5/base.yaml"

ANTHROPIC_PROMPT="$REPO_ROOT/examples/prompts/anthropic_thinking.json"
ANTHROPIC_SPEC="$REPO_ROOT/crates/specado-providers/providers/anthropic/claude-4.5/sonnet.yaml"

run_cli() {
  # shellcheck disable=SC2086
  eval "$SPECADO_BIN" "$@"
}

echo "== Preview GPT-5 reasoning payload =="
run_cli preview \
  --prompt "$OPENAI_PROMPT" \
  --provider "$OPENAI_SPEC"
echo

if [[ -n "${OPENAI_API_KEY:-}" ]]; then
  echo "== Executing GPT-5 reasoning request =="
  run_cli run \
    --prompt "$OPENAI_PROMPT" \
    --provider "$OPENAI_SPEC" \
    --audit-target stdout
else
  echo "Skipping GPT-5 run: set OPENAI_API_KEY to hit the live provider."
fi
echo

echo "== Preview Claude Sonnet thinking payload =="
run_cli preview \
  --prompt "$ANTHROPIC_PROMPT" \
  --provider "$ANTHROPIC_SPEC"
echo

if [[ -n "${ANTHROPIC_API_KEY:-}" ]]; then
  echo "== Executing Claude Sonnet thinking request =="
  run_cli run \
    --prompt "$ANTHROPIC_PROMPT" \
    --provider "$ANTHROPIC_SPEC" \
    --audit-target stdout
else
  echo "Skipping Claude Sonnet run: set ANTHROPIC_API_KEY to hit the live provider."
fi
echo

echo "Done."
