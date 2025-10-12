#!/usr/bin/env python3
"""Specado Python demo covering reasoning (OpenAI) and thinking (Anthropic).

Build the native extension first via ``maturin develop -m crates/specado-py/Cargo.toml``.
By default the script points at the richer demo prompts that ship with ``examples/``.
Use ``--scenario`` to switch between GPT-5 reasoning and Claude Sonnet thinking flows.
"""

from __future__ import annotations

import argparse
import json
import os
import pathlib
import sys
from typing import Any, Dict

try:  # Optional dependency used when prompt files are written in YAML.
    import yaml  # type: ignore
except ModuleNotFoundError:  # pragma: no cover - best-effort import for the example.
    yaml = None  # type: ignore

from specado import Client, PromptSpec
from specado import Message as PromptMessage
from specado.compat.openai import OpenAI


SCENARIOS = {
    "openai-reasoning": {
        "provider": "crates/specado-providers/providers/openai/gpt-5/base.yaml",
        "prompt": "examples/prompts/openai_reasoning.json",
        "api_key": "OPENAI_API_KEY",
        "description": "GPT-5 reasoning controls",
    },
    "anthropic-thinking": {
        "provider": "crates/specado-providers/providers/anthropic/claude-4.5/sonnet.yaml",
        "prompt": "examples/prompts/anthropic_thinking.json",
        "api_key": "ANTHROPIC_API_KEY",
        "description": "Claude Sonnet thinking mode",
    },
}


def config_env_path() -> pathlib.Path:
    if sys.platform == "win32":
        base = os.environ.get("APPDATA")
        if base is None:
            base = pathlib.Path.home() / "AppData" / "Roaming"
        else:
            base = pathlib.Path(base)
        return base / "specado" / ".env"

    if sys.platform == "darwin":
        return pathlib.Path.home() / "Library" / "Application Support" / "specado" / ".env"

    base = pathlib.Path(os.environ.get("XDG_CONFIG_HOME", pathlib.Path.home() / ".config"))
    return base / "specado" / ".env"


def load_env_file(path: pathlib.Path, override: bool) -> None:
    try:
        data = path.read_text().splitlines()
    except FileNotFoundError:
        return
    except OSError as exc:  # pragma: no cover - best effort logging
        print(f"warning: unable to read {path}: {exc}", file=sys.stderr)
        return

    for raw_line in data:
        line = raw_line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        key = key.strip()
        value = value.strip()
        if not key:
            continue
        if override or key not in os.environ:
            os.environ[key] = value


def load_env() -> None:
    load_env_file(config_env_path(), override=False)
    load_env_file(pathlib.Path(".env"), override=True)


def load_prompt(path: pathlib.Path) -> Dict[str, Any]:
    data = path.read_text()
    suffix = path.suffix.lower()

    if suffix in {".yaml", ".yml"}:
        if yaml is None:
            raise SystemExit(
                "Install PyYAML (pip install pyyaml) to consume YAML prompts; "
                "alternatively supply the JSON version."
            )
        return yaml.safe_load(data)

    return json.loads(data)


def ensure_api_key(var_name: str) -> None:
    if var_name in os.environ:
        return
    print(
        f"warning: {var_name} is not set. The provider call will likely fail.",
        file=sys.stderr,
    )


def run_native_client(args: argparse.Namespace, prompt_payload: Dict[str, Any]) -> Dict[str, Any]:
    audit_config = {"target": "stdout", "redact": args.audit_redact or []} if args.audit else None
    client = Client(
        args.provider,
        watch=args.watch,
        audit_config=audit_config,
    )
    return client.complete(prompt_payload)


def run_openai_compat(args: argparse.Namespace, prompt_payload: Dict[str, Any]) -> Dict[str, Any]:
    client = OpenAI(args.provider)
    prompt = PromptSpec(
        version=prompt_payload.get("version", "1"),
        messages=[
            PromptMessage(role=message["role"], content=message["content"])
            for message in prompt_payload["messages"]
        ],
        strict_mode=prompt_payload.get("strict_mode", "Warn"),
    )
    completion = client.chat.completions.create(
        model=prompt_payload.get("model", "gpt-5"),
        messages=[
            {"role": message.role, "content": message.content}
            for message in prompt.messages
        ],
        temperature=prompt_payload.get("sampling", {}).get("temperature"),
    )
    choice = completion.choices[0]
    return {
        "content": choice.message.content,
        "finish_reason": choice.finish_reason,
    }


def main() -> None:
    load_env()
    parser = argparse.ArgumentParser(description="Run the Specado Python demo")
    parser.add_argument(
        "--scenario",
        choices=sorted(SCENARIOS),
        default="openai-reasoning",
        help="Select which demo prompt to execute",
    )
    parser.add_argument("--provider")
    parser.add_argument("--prompt")
    parser.add_argument("--watch", action="store_true", help="Enable experimental watch plumbing")
    parser.add_argument("--audit", action="store_true", help="Send audit logs to stdout")
    parser.add_argument(
        "--audit-redact",
        nargs="*",
        help="Additional case-insensitive redaction patterns for audit logging",
    )
    parser.add_argument(
        "--openai-compat",
        action="store_true",
        help="Route the prompt through the OpenAI compatibility shim",
    )
    args = parser.parse_args()

    scenario = SCENARIOS[args.scenario]
    provider_path = args.provider or scenario["provider"]
    prompt_path_value = args.prompt or scenario["prompt"]

    args.provider = provider_path
    args.prompt = prompt_path_value

    prompt_path = pathlib.Path(prompt_path_value)
    prompt_payload = load_prompt(prompt_path)

    ensure_api_key(scenario["api_key"])

    if args.openai_compat:
        if args.scenario != "openai-reasoning":
            raise SystemExit("OpenAI compatibility mode only works with the GPT-5 scenario.")
        response = run_openai_compat(args, prompt_payload)
    else:
        response = run_native_client(args, prompt_payload)

    print(json.dumps(response, indent=2))


    print(
        f"\nCompleted scenario '{args.scenario}' ({scenario['description']}) using provider {provider_path} and prompt {prompt_path}",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
