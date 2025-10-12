#!/usr/bin/env bash
# Convenience wrapper that previews and optionally runs the reasoning/thinking sample prompts.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

if [[ -z "${SPECADO_BIN:-}" ]]; then
  if [[ -x "$REPO_ROOT/target/debug/specado" ]]; then
    SPECADO_BIN=("$REPO_ROOT/target/debug/specado")
  else
    SPECADO_BIN=(cargo run --quiet -p specado-cli --)
  fi
else
  # Allow the user to provide a custom command (split on spaces intentionally)
  read -r -a SPECADO_BIN <<<"$SPECADO_BIN"
fi

OPENAI_PROMPT="$REPO_ROOT/examples/prompts/openai_reasoning.json"
OPENAI_SPEC="$REPO_ROOT/crates/specado-providers/providers/openai/gpt-5/base.yaml"
ANTHROPIC_PROMPT="$REPO_ROOT/examples/prompts/anthropic_thinking.json"
ANTHROPIC_SPEC="$REPO_ROOT/crates/specado-providers/providers/anthropic/claude-4.5/sonnet.yaml"

config_env_path() {
  case "${OSTYPE:-}" in
    cygwin*|msys*|win32*)
      local base="${APPDATA:-}"
      if [[ -z "$base" ]]; then
        base="$HOME/AppData/Roaming"
      fi
      printf '%s\n' "$base/specado/.env"
      ;;
    darwin*)
      printf '%s\n' "$HOME/Library/Application Support/specado/.env"
      ;;
    *)
      printf '%s\n' "${XDG_CONFIG_HOME:-$HOME/.config}/specado/.env"
      ;;
  esac
}

trim() {
  local var="$1"
  var="${var#"${var%%[![:space:]]*}"}"
  var="${var%"${var##*[![:space:]]}"}"
  printf '%s' "$var"
}

load_env_file() {
  local file="$1"
  local override="$2"
  [[ -f "$file" ]] || return 0

  while IFS= read -r raw || [[ -n "$raw" ]]; do
    local line="${raw%%$'\r'}"
    [[ -z "$line" || "${line#"${line%%[![:space:]]*}"}" == \#* || "$line" != *"="* ]] && continue
    local key="${line%%=*}"
    local value="${line#*=}"
    key="$(trim "$key")"
    value="$(trim "$value")"
    [[ -z "$key" ]] && continue
    if [[ "$override" == "true" || -z "${!key:-}" ]]; then
      export "$key=$value"
    fi
  done < "$file"
}

load_env() {
  local global_path
  global_path="$(config_env_path)"
  load_env_file "$global_path" "false"
  load_env_file "$REPO_ROOT/.env" "true"
}

load_env

run_cli() {
  "${SPECADO_BIN[@]}" "$@"
}

command_string() {
  local args=("${SPECADO_BIN[@]}" "$@")
  printf '%q ' "${args[@]}"
}

log_step() {
  echo "--> $1"
}

print_preview() {
  printf '%s\n' "$1"
}

print_response() {
  local payload="$1"
  local start_line
  start_line=$(printf '%s\n' "$payload" | grep -n '{' | head -n1 | cut -d: -f1)
  if [[ -z "$start_line" ]]; then
    printf '%s\n' "$payload"
    return
  fi

  if (( start_line > 1 )); then
    printf '%s\n' "$payload" | head -n $((start_line-1))
  fi

  local json_block
  json_block=$(printf '%s\n' "$payload" | tail -n +$start_line)

  if command -v jq >/dev/null 2>&1; then
    local content
    content=$(echo "$json_block" | jq -r '(.content // .response_excerpt.content // empty)' 2>/dev/null)
    if [[ -n "$content" && "$content" != "null" ]]; then
      echo "--- Provider content ---"
      echo "$content"
      echo
    fi
    if parsed=$(echo "$json_block" | jq . 2>/dev/null); then
      echo "$parsed"
    else
      echo "$json_block"
    fi
  else
    echo "$json_block"
  fi
}

echo "== Preview GPT-5 reasoning payload =="
log_step "Translating prompt to provider request"
echo "Command: $(command_string preview --prompt "$OPENAI_PROMPT" --provider "$OPENAI_SPEC")"
preview_output=$(run_cli preview \
  --prompt "$OPENAI_PROMPT" \
  --provider "$OPENAI_SPEC" 2>&1) || true
print_preview "$preview_output"
echo

if [[ -n "${OPENAI_API_KEY:-}" ]]; then
  echo "== Executing GPT-5 reasoning request =="
  log_step "Dispatching request to provider"
  echo "Command: $(command_string run --prompt "$OPENAI_PROMPT" --provider "$OPENAI_SPEC" --audit-target stdout)"
  if response=$(run_cli run \
    --prompt "$OPENAI_PROMPT" \
    --provider "$OPENAI_SPEC" \
    --audit-target stdout 2>&1); then
    print_response "$response"
  else
    echo "$response"
  fi
else
  echo "Skipping GPT-5 run: set OPENAI_API_KEY to hit the live provider."
fi
echo

echo "== Preview Claude Sonnet thinking payload =="
log_step "Translating prompt to provider request"
echo "Command: $(command_string preview --prompt "$ANTHROPIC_PROMPT" --provider "$ANTHROPIC_SPEC")"
preview_output=$(run_cli preview \
  --prompt "$ANTHROPIC_PROMPT" \
  --provider "$ANTHROPIC_SPEC" 2>&1) || true
print_preview "$preview_output"
echo

if [[ -n "${ANTHROPIC_API_KEY:-}" ]]; then
  echo "== Executing Claude Sonnet thinking request =="
  log_step "Dispatching request to provider"
  echo "Command: $(command_string run --prompt "$ANTHROPIC_PROMPT" --provider "$ANTHROPIC_SPEC" --audit-target stdout)"
  if response=$(run_cli run \
    --prompt "$ANTHROPIC_PROMPT" \
    --provider "$ANTHROPIC_SPEC" \
    --audit-target stdout 2>&1); then
    print_response "$response"
  else
    echo "$response"
  fi
else
  echo "Skipping Claude Sonnet run: set ANTHROPIC_API_KEY to hit the live provider."
fi
echo

echo "Done."
