# Specado v1.0 Issue Index

_Updated: 2025-10-06_

This document mirrors the canonical GitHub epics and their tasks. All planning details now live with the issues themselves; use this index to jump between epics, child issues, and the implementation plan.

## Epic Overview

- [#1](https://github.com/specado/specado/issues/1) **Epic: Foundation (Workspace & Schemas)** — Foundation (Workspace & Schemas). Source: SPECADO_PLAN.md Week 1: Foundation.
- [#2](https://github.com/specado/specado/issues/2) **Epic: Core Engine** — Core Engine. Source: SPECADO_PLAN.md Weeks 2-3: Core Features & API Freeze.
- [#3](https://github.com/specado/specado/issues/3) **Epic: Developer Interfaces** — Developer Interfaces. Source: SPECADO_PLAN.md Weeks 4-5: Bindings & Resilience.
- [#4](https://github.com/specado/specado/issues/4) **Epic: Production Readiness** — Production Readiness. Source: SPECADO_PLAN.md Weeks 6-7: Production Features.
- [#5](https://github.com/specado/specado/issues/5) **Epic: Release Polish** — Release Polish. Source: SPECADO_PLAN.md Week 8: Release Polish.

### Epic #1 — Epic: Foundation (Workspace & Schemas)

_Plan reference: SPECADO_PLAN.md Week 1: Foundation._

- [#17](https://github.com/specado/specado/issues/17) Repository Structure & Root Workspace
- [#18](https://github.com/specado/specado/issues/18) specado-schemas Crate (Cargo & Validator)
- [#19](https://github.com/specado/specado/issues/19) Prompt Schema v1 (JSON)
- [#20](https://github.com/specado/specado/issues/20) Provider Schema v1 (JSON)
- [#21](https://github.com/specado/specado/issues/21) specado-core Cargo

### Epic #2 — Epic: Core Engine

_Plan reference: SPECADO_PLAN.md Weeks 2-3: Core Features & API Freeze._

- [#22](https://github.com/specado/specado/issues/22) Core Orchestration (specado-core/src/lib.rs)
- [#23](https://github.com/specado/specado/issues/23) Error Model (specado-core/src/error.rs)
- [#24](https://github.com/specado/specado/issues/24) Auth Handler (specado-core/src/auth.rs)
- [#25](https://github.com/specado/specado/issues/25) Types Barrel (specado-core/src/types/mod.rs)
- [#26](https://github.com/specado/specado/issues/26) Prompt Types (specado-core/src/types/prompt.rs)
- [#27](https://github.com/specado/specado/issues/27) Provider Types (specado-core/src/types/provider.rs)
- [#28](https://github.com/specado/specado/issues/28) Lossiness Types (specado-core/src/types/lossiness.rs)
- [#29](https://github.com/specado/specado/issues/29) Uniform Response (specado-core/src/types/response.rs)
- [#30](https://github.com/specado/specado/issues/30) Transformer Module Barrel (specado-core/src/transformer/mod.rs)
- [#31](https://github.com/specado/specado/issues/31) Translate (specado-core/src/transformer/translate.rs)
- [#32](https://github.com/specado/specado/issues/32) Normalize (specado-core/src/transformer/normalize.rs)
- [#33](https://github.com/specado/specado/issues/33) Detect: Barrel (specado-core/src/transformer/detect/mod.rs)
- [#34](https://github.com/specado/specado/issues/34) Detect Clamp (specado-core/src/transformer/detect/clamp.rs)
- [#35](https://github.com/specado/specado/issues/35) Detect Relocate (specado-core/src/transformer/detect/relocate.rs)
- [#36](https://github.com/specado/specado/issues/36) Detect Unsupported (specado-core/src/transformer/detect/unsupported.rs)
- [#37](https://github.com/specado/specado/issues/37) Detect Drops (specado-core/src/transformer/detect/drop.rs)
- [#38](https://github.com/specado/specado/issues/38) HTTP Client (specado-core/src/http/client.rs) & HTTP Module Shim
- [#39](https://github.com/specado/specado/issues/39) Circuit Breaker (specado-core/src/circuit_breaker.rs)
- [#40](https://github.com/specado/specado/issues/40) Retry Policy (specado-core/src/retry.rs)
- [#41](https://github.com/specado/specado/issues/41) Routing (Trait & Primary-Fallback)

### Epic #3 — Epic: Developer Interfaces

_Plan reference: SPECADO_PLAN.md Weeks 4-5: Bindings & Resilience._

- [#42](https://github.com/specado/specado/issues/42) CLI Cargo (crates/specado-cli/Cargo.toml) & CLI Main
- [#43](https://github.com/specado/specado/issues/43) Python Native Crate (crates/specado-py) & PyO3 Bindings
- [#44](https://github.com/specado/specado/issues/44) Python Project Config & Python High-Level API + OpenAI Compat
- [#45](https://github.com/specado/specado/issues/45) Node.js Bindings (napi-rs + packaging)

### Epic #4 — Epic: Production Readiness

_Plan reference: SPECADO_PLAN.md Weeks 6-7: Production Features._

- [#46](https://github.com/specado/specado/issues/46) Provider Catalog (OpenAI, Anthropic)
- [#48](https://github.com/specado/specado/issues/48) Production Feature: Hot-Reload (Design & Stub)
- [#49](https://github.com/specado/specado/issues/49) Production Feature: Audit Logging (Design & Stub)

### Epic #5 — Epic: Release Polish

_Plan reference: SPECADO_PLAN.md Week 8: Release Polish._

- [#47](https://github.com/specado/specado/issues/47) CI/CD Workflow (GitHub Actions)
- [#50](https://github.com/specado/specado/issues/50) Docs, Tests, Benches, Examples Scaffolding

