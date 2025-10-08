# Examples Overview

This directory contains runnable samples that mirror the quickstart in `docs/QUICKSTART.md` and support Issue #50 (Docs, Tests, Benches, Examples Scaffolding).

## Assets
- `prompts/basic_chat.json` — Minimal chat prompt shared by all examples.
- `cli_preview.sh` — Helper script that invokes `specado preview` with the sample assets.
- `python_basic.py` — Python binding example using both the native client and the OpenAI compatibility shim.
- `node_basic.mjs` — Node.js binding example that exercises the napi-based client.

## Usage
Follow the instructions in the quickstart to build the CLI, Python extension, and Node module. Each script accepts optional flags to point at different specs, enable audit logging, or turn on watch plumbing.

These samples are intentionally lightweight; add new fixtures alongside them when demonstrating additional providers or prompt types.
