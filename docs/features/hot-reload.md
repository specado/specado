# Hot Reload Design Stub

This document captures the MVP design for Specado's hot-reload support. The goal is to let long-running processes detect provider spec changes without restart while keeping the current synchronous execution path as the source of truth.

## Current Execution Path
- `specado_core::execute` loads provider specs from disk on each call via `hot_reload::global_cache().load_or_read`. The cache is a lightweight `RwLock<HashMap<PathBuf, CachedProvider>>` that stores the latest parsed `ProviderSpec`.
- CLI (`specado run/preview`), Python (`specado.Client`), and Node (`new Client()`) hold only file paths and rebuild state on every invocation. No watcher, timer, or async task is started today.

## Proposed Topology
- `HotReloadConfig` now tracks `enable`, `watch_paths`, and `debounce_ms`. The CLI exposes `--watch` and optional `--watch-provider-dir` flags that push configuration into `hot_reload::set_global_config`, while bindings persist the flag for future use.
- A future implementation will call `start_hot_reload(config, cache)` (behind the `hot-reload` feature) to spawn a Tokio task running `notify` watchers. For the stub we ship only the handle type so later work can add Drop semantics and graceful shutdown.
- Provider specs resolve through `ProviderCache::load_or_read`. Once watchers land, cache updates will flow through a centralized `Arc<RwLock<...>>` that the watcher mutates after validating replacements with `specado_schemas::get_validator()`.

## Error Recovery & Telemetry
- Cache reads continue to surface `Error::Config` immediately so callers behave exactly as before when files disappear or fail to parse.
- When the watcher is implemented it should log soft failures via `tracing::warn` and retain the last good spec. The debounce duration is stored in the config to allow jitter tuning per binding.

## Binding Integration Surface
- CLI: `specado run --watch --watch-provider-dir providers/` stores the config and prints an experimental warning. The current binary still performs a one-shot execution.
- Python: `Client(provider_path, watch=True)` toggles an internal flag so the native layer knows to opt in once hot reload is live.
- Node: `new Client(path, { watch: { enable: true, paths: [...] } })` records the same settings and leaves implementation to the future feature gate.

## Follow-up Work
- Implement `start_hot_reload` with the `notify` crate, wiring file events into cache updates.
- Plumb watcher lifecycle into CLI (Tokio runtime), Python (shared runtime), and Node (Tokio via napi) with shutdown hooks.
- Add integration tests that mutate temp specs and assert that `execute` observes the change when hot reload is enabled.
