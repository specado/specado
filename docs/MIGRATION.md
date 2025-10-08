# Specado Migration Guide (Draft)

The Specado API is stabilising as we drive toward v1.0. This document captures the policies and checklists that will guide future breaking changes. It is intentionally lightweight for now and will expand alongside the CHANGELOG once public releases begin.

## Versioning policy
- Specado follows [Semantic Versioning](https://semver.org/).
- Until we tag `v1.0.0`, minor version bumps (e.g. `0.2.x`) may include breaking changes but must be called out explicitly in release notes.
- After `v1.0.0`, breaking changes require a major version increment across the Rust crates, Python package, and Node package.
- The workspace keeps crate versions aligned via `[workspace.package]` in `Cargo.toml`; when bumping versions, update the Python `__version__` and Node package manifest in the same change.

## Breaking change checklist
When a change requires consumers to update code or configuration, ensure the following steps are complete:

1. Document the change in `CHANGELOG.md` with upgrade instructions and cross-reference the relevant GitHub issues.
2. Update API reference docs, code samples in `examples/`, and any affected golden snapshots.
3. Revise `docs/QUICKSTART.md` to reflect new CLI flags or language binding behaviour.
4. Provide migration snippets or compatibility shims in the language bindings where reasonable (e.g., `specado.compat`).
5. Add regression tests to cover the new behaviour and keep previous fixtures where they help demonstrate changes.

## Current assumptions
- Latest verified toolchain versions and required environment variables are tracked in `docs/QUICKSTART.md`.
- Production features such as hot-reload and audit logging are experimental; interface changes to these areas should flag the `experimental` label in GitHub.
- Provider specifications under `crates/specado-providers/providers/` define the canonical shapes that examples should mirror.

## Next steps
- Create `CHANGELOG.md` at the v1.0.0 freeze and link it from this guide.
- Flesh out language-specific migration playbooks (Python, Node) once their APIs stabilise.
- Document data migration requirements for persisted audit logs when the format is finalised.

_Last updated: October 2025. Maintained as part of Issue #50 (Docs, Tests, Benches, Examples Scaffolding)._ 
