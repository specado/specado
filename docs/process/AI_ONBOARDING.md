# AI Assistant Onboarding Guide

This guide explains how automation agents (Codex, Claude Code, Gemini Code Assist, etc.) should work inside the Specado repository.

You must always analyze the Github Issues to determine the current state of the project and other critical context about where to start your work, continue your work, or otherwise tackle the project and associated requests.

## 1. GitHub Workflow
- Read `docs/process/GITHUB_TRACKING.md` before running any `gh` commands; it contains the canonical checklist, quoting rules (single quotes), and hygiene requirements when creating, working on, updating, or closing issues.
- The github repo is `specado/specado` and the organization is `specado`
- Maintain GitHub sub-issue links: epics #1–#5 parent their child issues. If re-scoping work, update the sub-issue relationships via `addSubIssue`/`removeSubIssue` mutations and adjust project statuses in Organization Project 14.

## 2. Language-Specific Code Rules
- **Rust (`specado-core`, `specado-schemas`, etc.)**: Adhere to the workspace toolchain (Rust 1.75+). Prefer idiomatic async patterns with `tokio`, use `thiserror` for error types, and keep transformer modules modular (one file per lossiness code). All public API changes must pass `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test --workspace`.
- **Python (`crates/specado-py`, `python/specado/`)**: Use PyO3 bindings exposed through `python/specado`. Keep user-facing imports stable (`from specado import Client`). Run `maturin develop` followed by `pytest` for validation.
- **Node.js (`crates/specado-node`)**: Bindings are implemented with `napi-rs`. Ensure TypeScript definitions stay in sync with the Rust exports (`npm run build` generates artifacts). Use `npm test` across the matrix defined in CI.
- For all languages, prioritize deterministic snapshot tests and update golden files only with clear justification.

## 3. CLI & Package Usage
- The CLI command is `specado`. Invoke workflows such as `specado validate`, `specado preview`, and `specado run` once the binary is built.
- Language imports use the plain package name `specado`: `pip install specado`, `npm install specado`, and `import specado`/`require('specado')`. Do not create variant names or scoped packages.

## 4. Critical Practices
- Keep Org Project 14 item statuses up to date (Todo/In Progress/Done) whenever you touch an issue.
- When closing issues, include evidence (links to commits, test results) and update any dependent issues or epics.
- Honour the repository’s ASCII-only convention unless the file already uses non-ASCII characters.
- If you detect unsanctioned file changes mid-session, pause and surface the situation to the maintainer before continuing.
