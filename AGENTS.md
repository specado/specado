# Repository Guidelines

## Project Structure & Module Organization
- Root `Cargo.toml` defines the Rust workspace; core logic lives in `crates/specado-core`, schemas in `crates/specado-schemas`, and provider integrations under `crates/specado-providers`.
- The CLI is in `crates/specado-cli` and produces the `specado` binary; Node and Python bindings sit in `crates/specado-node` and `crates/specado-py` with companion packages in `python/`.
- Integration and golden tests live in `tests/integration` and `tests/golden`; documentation and process notes reside in `docs/`.
- Sample workflows and fixtures are under `examples/`; keep new assets grouped there.

## Build, Test, and Development Commands
- `cargo build --workspace` builds all Rust crates; `cargo build -p specado-cli` emits the CLI binary.
- `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test --workspace` are mandatory before pushing.
- Python bindings: `maturin develop -m crates/specado-py/Cargo.toml` followed by `pytest` from `python/`.
- Node bindings: run `npm install` then `npm run build && npm test` inside `crates/specado-node`.
- The CLI entrypoints (`specado validate`, `specado preview`, `specado run`) assume the CLI has been built via Cargo.

## Coding Style & Naming Conventions
- Enforce `rustfmt` defaults and prefer idiomatic async (`tokio`) patterns; error types use `thiserror`.
- Python modules mirror the Rust namespace (`specado.*`) and must remain ASCII; lint with `ruff` if available.
- Node code follows the existing `napi-rs` structure; keep TypeScript definitions in sync when exports change.
- Use snake_case for files, CamelCase for Rust types, and avoid introducing non-ASCII unless the file already uses it.

## Testing Guidelines
- Favor deterministic snapshots; update `tests/golden` only when behavior intentionally changes and document the reason.
- Name Rust tests after the feature (`mod_name_case_scenario`); Python tests live under `python/tests` with `test_*.py`.
- Run `cargo test --workspace`, `pytest`, and `npm test` for affected surfaces; capture evidence when closing issues.

## Commit & Pull Request Guidelines
- Follow Conventional Commits (`feat(provider): add cache layer`) and reference the issue (`Closes #123`) in the message.
- Pull requests must include a concise summary, list of changes, testing results, and linked issues per `docs/process/GITHUB_TRACKING.md`.
- Keep Org Project 14 statuses in sync and update epic relationships when scope shifts; never mention internal planning docs in GitHub comments.

## Agent-Specific Notes
- Review open GitHub issues and project status before starting work; if network access is unavailable, confirm the latest context with the maintainer.
- Preserve existing automation by keeping golden files, fixtures, and binding APIs stable unless explicitly tasked to change them.
- Treat new `--watch` and `--audit-*` flags as experimental: they configure core stubs but do not yet start watchers or emit logs without follow-up feature work.
