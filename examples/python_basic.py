#!/usr/bin/env python3
"""Minimal Specado Python example for Issue #50.

The script loads the sample prompt in ``examples/prompts/basic_chat.json`` and executes
it against the provider spec supplied on the command line. Build the native extension
via ``maturin develop -m crates/specado-py/Cargo.toml`` before running this file.
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


def ensure_api_key() -> None:
    if "OPENAI_API_KEY" not in os.environ:
        print(
            "warning: OPENAI_API_KEY is not set. The provider call will likely fail.",
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
        messages=[
            PromptMessage(role=message["role"], content=message["content"])
            for message in prompt_payload["messages"]
        ]
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
    parser = argparse.ArgumentParser(description="Run the Specado Python example")
    parser.add_argument(
        "--provider",
        default="crates/specado-providers/providers/openai/gpt-5-family.yaml",
    )
    parser.add_argument("--prompt", default="examples/prompts/basic_chat.json")
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

    prompt_path = pathlib.Path(args.prompt)
    prompt_payload = load_prompt(prompt_path)

    ensure_api_key()

    if args.openai_compat:
        response = run_openai_compat(args, prompt_payload)
    else:
        response = run_native_client(args, prompt_payload)

    print(json.dumps(response, indent=2))


if __name__ == "__main__":
    main()
