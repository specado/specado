# Audit Logging Design Stub

This document outlines the initial design for structured audit logging in Specado. The focus is to wire configuration, redaction, and logging stubs so that future work can enable full JSONL emission without disturbing the current MVP runtime.

## Execution Touch Points
- `specado_core::execute` accepts an optional `audit::AuditContext` when the `audit-logging` feature is enabled. This context wraps the start time, correlation ID, redaction helpers, and optional logger.
- On success the context emits an `AuditEvent` containing the provider, model, latency, lossiness report, redacted request payload, and a response excerpt (`serde_json` snapshot of `UniformResponse`).
- Failures (config, HTTP, strict mode, provider errors) record an `AuditEvent` with `status = error` and the error category derived from `Error`.

## Config Surface
- Core exposes `AuditConfig { target, redact }` and `AuditTarget::{Stdout, File}`. CLI flags `--audit-target`, `--audit-file`, and `--audit-redact` map to this struct, and the CLI prints an experimental warning when enabled.
- Python: `Client(..., audit_config={"target": "stdout", "redact": [...]})` is parsed into `AuditConfig` and stored for each invocation.
- Node: `new Client(path, { audit: { target: 'stdout' | { file: 'audit.jsonl' }, redact: [...] } })` follows the same schema.

## Redaction & Correlation
- Default regex patterns mask keys matching `authorization`, `token`, `secret`, and `api[-_]?key`. Callers can append additional case-insensitive patterns.
- Each `AuditContext` generates a UUIDv4 correlation ID and uses `time::OffsetDateTime` to stamp events in RFC3339 format.

## I/O Strategy
- `JsonlAuditLogger` writes synchronously either to `stdout` or to an append-only file. Errors during writes are surfaced via `tracing::warn` but do not fail the request.
- The module is behind the `audit-logging` feature flag so the code paths are present but inert unless dependants opt in.

## Next Steps
- Move to buffered, async logging (e.g., `tokio::fs::File`) and add rotation/retention policies.
- Propagate correlation IDs through bindings so downstream services can reuse them when making chained calls.
- Expand tests to cover file logging in integration scenarios and to assert redaction on nested data structures.
