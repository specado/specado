# **Product Requirements Document (PRD): Specado v1.0**

**Status:** Final  
**Date:** 2025-08-16  
**Version:** 1.0.0

---

## Executive Summary

Specado is a spec-driven library and CLI that compiles uniform LLM prompts into provider-native requests, normalizes responses bidirectionally, and provides transparent lossiness reports. It enables seamless multi-provider workflows without hardcoded adapters—all provider behavior is defined in declarative specification files.

---

## 1) One-Line Summary

**Specado** compiles a uniform prompt into a provider-native request using a declarative **ProviderSpec**, then normalizes the provider's response back to a uniform shape—emitting a detailed **LossinessReport** that explains every drop, clamp, relocation, conflict, or emulation.

---

## 2) Goals & Non-Goals

### Goals (v1.0)

* **Spec-Driven Translation & Normalization**  
  All provider behavior lives in **data** (JSON/YAML specs). The engine contains **no hardcoded provider adapters**.

* **Strict Contracts**  
  **PromptSpec** (uniform request) and **ProviderSpec** (capabilities + mappings) are defined by JSON Schema (draft 2020-12).

* **Lossiness Visibility**  
  A formal **LossinessReport** captures deviations with codes, before/after, and severities. User-controlled policies: **Strict | Warn | Coerce**.

* **Bidirectional I/O**  
  Translate uniform requests **to** provider payloads; normalize provider responses/streams **from** providers to **UniformResponse** / unified **Stream events**.

* **Binding-Ready Design**  
  Core in **Rust** with a JSON in/out FFI boundary; first-class **Node.js** and **Python** bindings. Optional **WASM** for translate/preview/normalize (no HTTP).

* **CLI First**  
  A powerful CLI to validate, preview, run, stream, and compare providers.

### Non-Goals (v1.0)

* No orchestration, agent loops, retrievers, background queues, or evaluation frameworks.
* No UI beyond the CLI.
* No auto-scraping provider docs; **ProviderSpec is the source of truth**.
* **No cost estimation or budget enforcement** inside the engine.

---

## 3) Personas & Key Use Cases

### Personas

* **Application Engineer**  
  Swap providers (e.g., OpenAI → Anthropic) by changing a model ID and receive a clear **lossiness report** on differences.

* **Platform / Infra Team**  
  Enforce org policies (disable features, pin versions) by managing a central repository of **ProviderSpec** files.

* **Prompt / Data Engineer**  
  Guarantee structured outputs across models, even when emulation is required (tools or prompt-only), with transparent reporting.

### Core Use Cases

1. **Preview Translation** offline and see lossiness before any network calls.
2. **Run Sync** and get a fully **normalized** response.
3. **Stream** with **normalized** start/delta/tool/stop events + cancellation.
4. **Matrix Compare** multiple providers/models and produce a diff/report.
5. **Tools & Structured Outputs** with native support or spec-driven emulation.

---

## 4) System Overview

Specado is a **stateless** translation/normalization engine:

```
1) Uniform Prompt (JSON) --schema validate--> UniformRequest (PromptSpec)

2) Provider Spec (YAML/JSON) --schema validate--> ProviderModel (ProviderSpec)

3) UniformRequest + ProviderModel
   -> Pre-validate (ranges, mutual exclusivity, limits, capability checks)
   -> Map fields via JSONPath to build ProviderRequest
   -> Emulate/drop/relocate/clamp per rules and strictness policy
   -> TranslationResult { provider_request_json, lossiness }

4) (Optional) Execute HTTP/Stream --> Raw ProviderResponse/Events

5) Normalize via ProviderSpec (sync + stream JSONPaths/maps)
   -> UniformResponse (sync) OR normalized events (start|delta|tool|stop)
```

**All behavior (request mapping, response parsing, streaming) is data-driven by ProviderSpec.**

---

## 5) Public Surfaces

### 5.1 Rust Core Library (`specado`)

```rust
pub enum StrictMode { Strict, Warn, Coerce }

#[derive(Serialize, Deserialize)]
pub struct Lossiness {
  pub code: String,          // "Clamp" | "Drop" | "Emulate" | "Conflict" | "Relocate" | "Unsupported" | "MapFallback" | "PerformanceImpact"
  pub path: String,          // e.g., "sampling.top_p"
  pub message: String,       // human-readable detail
  pub before: Option<serde_json::Value>,
  pub after:  Option<serde_json::Value>,
  pub severity: String       // "info" | "warn" | "error"
}

pub struct LossinessReport { pub items: Vec<Lossiness> }

pub struct TranslationResult {
  pub provider_request_json: serde_json::Value,
  pub lossiness: LossinessReport,
}

pub fn translate(
  prompt_json: &str,
  provider_spec_json: &str,
  model_id: &str,
  mode: StrictMode
) -> Result<TranslationResult, SpecadoError>;

// Execute sync and normalize to UniformResponse
pub fn run(provider_request_json: &str) -> Result<UniformResponse, SpecadoError>;

// Start streaming; consume via next_event()
pub fn stream(provider_request_json: &str) -> Result<StreamHandle, SpecadoError>;
pub fn next_event(h: &mut StreamHandle) -> Option<serde_json::Value>;
```

**UniformResponse (canonical):**

```json
{
  "model": "string",
  "content": "string",
  "finish_reason": "stop | length | tool_call | end_conversation | other",
  "tool_calls": [ { "name": "string", "arguments": { } } ],  // Optional field
  "raw_metadata": { }
}
```

### 5.2 Command-Line Interface (`specado`)

```bash
# Validate schemas & preview translation (no HTTP)
specado validate --prompt prompt.json --provider providers/openai/gpt-5.yaml --model gpt-5
specado preview  --prompt prompt.json --provider providers/anthropic/claude-opus-4-1.yaml --model claude-opus-4-1-20250805 --strict warn

# Execute requests and get normalized sync output
specado run     --prompt prompt.json --provider providers/openai/gpt-5.yaml --model gpt-5 --out response.json

# Stream normalized events (NDJSON: start|delta|tool|stop)
specado stream  --prompt prompt.json --provider providers/anthropic/claude-opus-4-1.yaml --model claude-opus-4-1-20250805

# Compare providers/models and generate a report
specado matrix  --prompt prompt.json --models openai/gpt-5 anthropic/claude-opus-4-1-20250805 --report matrix.md

# Diff two normalized responses (structure + basic semantics)
specado diff --left openai-response.json --right anthropic-response.json --out diff.md
```

**Discovery:** `--model <provider>/<model>` resolves to (1) explicit path if given, else (2) `./providers/<provider>/<model>.yaml`, else (3) `~/.config/specado/providers/<provider>/<model>.yaml`.

### 5.3 Bindings

* **Node.js (napi-rs)**: `translate`, `run`, `stream` (async iterable), JSON in/out only.
* **Python (pyo3)**: `translate`, `run`, `stream` (generator).
* **WASM (optional in v1.0)**: `translate`, `preview`, `normalize(response_json)`; no HTTP.

---

## 6) Authoritative Schemas (v1.0)

> JSON Schema **draft 2020-12**. These are canonical contracts the engine validates.  
> Full schemas are provided as separate files: `/schemas/prompt-spec.schema.json` and `/schemas/provider-spec.schema.json`
>
> **Important:** When discrepancies exist between PRD descriptions and the actual JSON Schemas, the schemas in `/schemas/` are authoritative. Update this document during spec refinement phases to maintain alignment.

### 6.1 `PromptSpec` (uniform request; highlights)

* **Fields:**
  - `model_class` ("Chat" | "ReasoningChat")
  - `messages` (array of `{role, content}`)
  - `tools?` (array of `{name, description?, json_schema}`)
  - `tool_choice?` (`"auto"` | `"required"` | `{ "name": string }`)
  - `response_format?` (`"text"` | `"json_object"` | `{ "json_schema": {...}, "strict?": true }`)
  - `sampling?` (`temperature?`, `top_p?`, `top_k?`, …)
  - `limits?` (`max_output_tokens?`, `reasoning_tokens?`)
  - `media?` (`input_images?`, `input_audio?`, `output_audio?`)
  - `strict_mode` (**"Strict" | "Warn" | "Coerce"**)

* **Rules:** Required fields validated; unsupported combinations flagged before translation.

**See:** `/schemas/prompt-spec.schema.json` for complete specification

### 6.2 `ProviderSpec` (capabilities + mappings; highlights)

* **Top-Level:**
  - `spec_version` (semver)
  - `provider` (`name`, `base_url`, `headers`)
  - `models[]`

* **Per Model:**
  - `id`, `aliases?`, `family`
  - `endpoints`
    - `chat_completion`: `{method, path, protocol: "http"}`
    - `streaming_chat_completion`: `{method, path, protocol: "sse"|"chunked", query?, headers?}`
  - `input_modes`: `{messages, single_text, images}`
  - `tooling`: Tools support, parallel control, emulation settings
  - `json_output`: Native support and strategy
  - `parameters`: Standard knobs + `feature_flags`
  - `constraints`: Exclusivity rules, limits, preferences
  - `mappings`: JSONPath mappings from uniform to provider
  - `response_normalization`: Sync and stream parsing rules

**See:** `/schemas/provider-spec.schema.json` for complete specification

### 6.3 Version Compatibility

| Engine Version | PromptSpec | ProviderSpec |
|---------------|------------|--------------|
| 1.0.x         | 1.0-1.1    | 1.0-1.1      |

---

## 7) Engine Behavior & Lossiness Model

1. **Validate** PromptSpec & ProviderSpec JSON against schemas.
2. **Pre-Validate** against model constraints:
   * Ranges (e.g., temperature, top_p)
   * Conflicts (`mutually_exclusive`) with `resolution_preferences`
   * Capability mismatches (e.g., tools unsupported)
   * Size constraints (`limits`) on system prompt, tool schemas
3. **Map** uniform keys to provider payload via `mappings.paths` (create missing parents).
4. **Transform & Emulate** (depending on provider flags & `json_output.strategy`):
   * **Relocate** system prompt (e.g., Anthropic top-level).
   * **Emulate** structured JSON via tools or prompt when native unsupported.
   * **Serialize tool calls** engine-side if parallel disable not supported (when `emulate_serialization: true`).
5. **LossinessReport** (one record per deviation):
   
   **Codes:**
   * `Clamp` – value clamped into supported range
   * `Drop` – unsupported field removed
   * `Emulate` – behavior achieved via non-native mechanism
   * `Conflict` – mutually exclusive; resolved via preference
   * `Relocate` – field moved (e.g., system message)
   * `Unsupported` – requested capability not available
   * `MapFallback` – alternate mapping used
   * `PerformanceImpact` – likely quality/latency risk (e.g., very large schema)

6. **Normalization** (sync & stream) via `response_normalization` JSONPaths and mapping tables.

**Strictness Policies**

* **Strict:** violation → error (fail fast).
* **Warn:** proceed with Drop/Clamp/Emulate/Relocate, record warnings.
* **Coerce:** like Warn, but clamp numeric values automatically.

---

## 8) Delivery Plan — The Pyramid (5 Levels)

> Each level ends with **Spec Refinement** and **Docs**. All levels are **independently demonstrable**.

### **L1 — Contracts & Preview (offline)**

* **Ships:** PromptSpec & ProviderSpec schemas; loader/validator; translation preview; real lossiness.
* **Acceptance:**
  * Valid preview for OpenAI & Anthropic.
  * Anthropic **system** relocates to top-level with `Relocate`.
  * Conflicts (e.g., `temperature` vs `top_p` if prohibited) produce `Conflict` + resolution per preferences.

### **L2 — Sync End-to-End (execute + normalize, sync)**

* **Ships:** HTTP client (method/path/query/headers), retries/backoff; sync **response_normalization** (content, tool calls, finish reason, error extraction).
* **Acceptance:**
  * `specado run` returns **UniformResponse** for both providers.

### **L3 — Streaming (Lite)**

* **Ships:** Raw SSE/stream passthrough + cancellation (handle drop).
* **Acceptance:**
  * `specado stream --raw` prints provider events; cancellation works.

### **L4 — Streaming (Normalized)**

* **Ships:** Spec-driven stream normalization (routes → `start|delta|tool|stop`).
* **Acceptance:**
  * Normalized events with correct ordering for OpenAI & Anthropic; cancellation works.

### **L5 — Tools, Structured Outputs, Feature Flags & DX**

* **Ships:**
  * Tools (native where supported; spec-driven serialization/emulation otherwise).
  * Structured outputs (OpenAI: native; Anthropic: tools or prompt per `json_output.strategy`, with post-validation of sizes).
  * Feature flags (e.g., `reasoning`, `personality`, `end_conversation_supported`).
  * CLI DX: provider discovery, `matrix`, `diff`.
  * **Bindings**: Node & Python; **WASM** optional (translate/preview/normalize).
* **Acceptance:**
  * Strict JSON via native or emulated flow as requested;
  * Matrix & diff produce useful reports;
  * Bindings pass the shared golden corpus.

---

## 9) Testing & Quality

* **Golden Tests**  
  Corpus of prompts → snapshot **provider_request_json** and **LossinessReport**, plus normalized sync responses and normalized stream event sequences.

* **Property Tests**  
  Post-translation numeric knobs always within provider ranges.

* **Fuzzing**  
  JSONPath mapping/normalization (missing parents, invalid selectors), out-of-order/missing stream events.

* **Binding Parity**  
  Node/Python bindings run the same corpus as Rust.

---

## 10) Non-Functional Requirements

* **Performance**: Translation/normalization adds negligible overhead (target sub-millisecond on typical prompts; non-binding guidance).
* **Stability**: All provider variability is isolated to specs; engine is generic.
* **Portability**: Linux/macOS/Windows; no provider-specific binaries.
* **Observability**: Optional debug logs (redact secrets); include request IDs and event counts.
* **Security & Privacy**:
  * No logging of prompt content by default.
  * Never log API keys; support `${ENV:VAR}` in headers.
  * TLS required for HTTP endpoints.

### 10.1 Required Environment Variables

The following environment variables must be set for provider authentication:
* `OPENAI_API_KEY` - Required for OpenAI provider specs
* `ANTHROPIC_API_KEY` - Required for Anthropic provider specs

---

## 11) Packaging, Versioning & Structure

### Crate Layout

```
specado/
  crates/
    specado-core        # engine: translate, execute, stream, normalize (spec-driven)
    specado-schemas     # JSON Schemas; optional Rust types for compile-time alignment
    specado-providers   # DATA-ONLY curated specs (OpenAI, Anthropic)
    specado-cli         # CLI
    specado-ffi         # C-ABI for Node/Python
    specado-wasm        # (optional) translate/preview/normalize
```

### Repository Structure

```
specado/
├── docs/
│   ├── PRD.md                    # This document
│   └── CODES.md                  # Lossiness code definitions
├── schemas/
│   ├── prompt-spec.schema.json   # The PromptSpec JSON Schema
│   └── provider-spec.schema.json # The ProviderSpec JSON Schema
├── providers/                    # Provider spec files (YAML/JSON)
│   ├── openai/
│   │   └── gpt-5.yaml
│   └── anthropic/
│       └── claude-opus-4-1.yaml
└── crates/
    └── ...                       # As defined above
```

### Spec Versioning

`spec_version: "1.1.0"` (semver). Engine accepts compatible ranges (e.g., ^1.1.x) with validation.

---

## 12) Success Metrics

### Technical Metrics

- Zero provider-specific code in engine
- 100% behavior defined in specs
- < 100ms translation overhead
- Streaming latency < 50ms per event

### User Metrics

- Single configuration change to switch providers
- Complete lossiness visibility
- Consistent API across all bindings

---

## 13) Definition of Done (overall)

* All **five levels** demonstrably usable via CLI.
* **Spec-only** behavior: adding a provider/model is editing a spec file.
* OpenAI (GPT-5) and Anthropic (Claude Opus 4.1) pass acceptance at each level.
* Test suite includes **golden**, **property**, **fuzzing**, and **binding parity**.
* **CODES.md** documents lossiness codes & semantics.

---

## 14) Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Provider drift (payloads/events change) | Breaking changes | Specs are easy to update; golden tests + fuzzing catch breaks early |
| Emulation scope creep | Complexity explosion | Keep emulation minimal (serialization, structured JSON via tools/prompt), document clearly in lossiness |
| Binding/runtime differences | Inconsistent behavior | JSON in/out boundary + handle-based streaming keeps parity across languages |

---

## 15) Decision Log

1. **Spec-driven vs. code adapters:** Chose pure spec-driven for maintainability
2. **Rust core:** Performance and memory safety for streaming
3. **JSON Schema validation:** Industry standard, good tooling
4. **JSONPath for mappings:** Simple, well-understood, sufficient for 95% of cases
5. **5-level pyramid delivery:** Each level independently valuable

---

## 16) Post-v1.0 Roadmap

* **v1.1:** Support for additional providers (e.g., Google Gemini, Cohere).
* **v1.2:** Support for additional modalities and logical operations (e.g., image generation, embeddings).
* **v2.0:** Introduce policy-as-code for more granular control over lossiness handling and a pluggable transformation DSL.

---

## Appendix A — Seed Provider Specs (illustrative)

> Field names/paths are representative; finalize JSONPaths to match your current provider examples when authoring the actual specs.

### A.1 OpenAI — GPT-5 (Responses API default)

`providers/openai/gpt-5.yaml`

```yaml
spec_version: "1.1.0"
provider:
  name: openai
  base_url: "https://api.openai.com"
  headers: { Authorization: "Bearer ${ENV:OPENAI_API_KEY}" }

models:
  - id: "gpt-5"
    aliases: ["gpt-5-chat-latest"]
    family: "gpt-5"
    endpoints:
      chat_completion:           { method: POST, path: "/v1/responses", protocol: "http" }
      streaming_chat_completion: { method: POST, path: "/v1/responses", protocol: "sse", query: { stream: "true" } }
    input_modes: { messages: true, single_text: true, images: true }
    tooling:
      tools_supported: true
      parallel_tool_calls_default: true
      can_disable_parallel_tool_calls: true
      disable_switch: { path: "$.parallel_tool_calls", value: false }
    json_output: { native_param: true, strategy: "json_schema" }
    parameters:
      temperature:       { supported: true, mapping: "temperature",       range: { min: 0.0, max: 2.0 } }
      top_p:             { supported: true, mapping: "top_p",             range: { min: 0.0, max: 1.0 } }
      max_output_tokens: { supported: true, mapping: "max_output_tokens" }
      stop:              { supported: true, mapping: "stop" }
      feature_flags:
        reasoning:   { supported: true, mapping: "reasoning.effort", kind: "enum",    enum: ["low","medium","high"] }
        personality: { supported: true, mapping: "personality.warmth", kind: "numeric", default: 0.5 }
    constraints:
      system_prompt_location: "message_role"
      forbid_unknown_top_level_fields: false
      mutually_exclusive: []
      resolution_preferences: []
      limits:
        max_tool_schema_bytes: 200000
        max_system_prompt_bytes: 32000
    mappings:
      paths:
        "messages":                 "$.input"
        "sampling.temperature":     "$.temperature"
        "sampling.top_p":           "$.top_p"
        "limits.max_output_tokens": "$.max_output_tokens"
        "response_format":          "$.response_format"
      flags:
        json_mode: "native"
        emulation_strategy: "auto"
    response_normalization:
      sync:
        content_path: "$.output[0].content"
        finish_reason_path: "$.finish_reason"
        finish_reason_map:
          stop: "stop"
          length: "length"
          tool_call: "tool_call"
          end: "end_conversation"
      stream:
        protocol: "sse"
        event_selector:
          type_path: "$.type"
          routes:
            - { when: "response.started",    emit: "start" }
            - { when: "response.delta",      emit: "delta", text_path: "$.delta" }
            - { when: "tool.call",           emit: "tool",  name_path: "$.name", args_path: "$.arguments" }
            - { when: "response.completed",  emit: "stop" }
```

> Optional alternative spec (Chat Completions) can live in `providers/openai/gpt-5.chat.yaml` using `choices[0]` paths.

### A.2 Anthropic — Claude Opus 4.1 (Messages API)

`providers/anthropic/claude-opus-4-1.yaml`

```yaml
spec_version: "1.1.0"
provider:
  name: anthropic
  base_url: "https://api.anthropic.com"
  headers:
    x-api-key: "${ENV:ANTHROPIC_API_KEY}"
    anthropic-version: "2023-06-01"

models:
  - id: "claude-opus-4-1-20250805"
    aliases: ["claude-opus-4-1","opus-4.1"]
    family: "opus-4.1"
    endpoints:
      chat_completion:           { method: POST, path: "/v1/messages", protocol: "http" }
      streaming_chat_completion: { method: POST, path: "/v1/messages", protocol: "sse", query: { stream: "true" } }
    input_modes: { messages: true, single_text: false, images: true }
    tooling:
      tools_supported: true
      parallel_tool_calls_default: true
      can_disable_parallel_tool_calls: true
      disable_switch: { path: "$.tool_choice.disable_parallel_tool_use", value: true }
    json_output: { native_param: false, strategy: "tools" }
    parameters:
      temperature:       { supported: true, mapping: "temperature",  range: { min: 0.0, max: 1.0 } }
      top_p:             { supported: true, mapping: "top_p",        range: { min: 0.0, max: 1.0 } }
      max_output_tokens: { supported: true, mapping: "max_tokens" }
      stop:              { supported: true, mapping: "stop_sequences" }
      feature_flags:
        reasoning:                  { supported: true, mapping: "thinking", kind: "boolean", default: false }
        end_conversation_supported: { supported: true, mapping: "end_conversation", kind: "boolean", default: false }
    constraints:
      system_prompt_location: "top_level"
      forbid_unknown_top_level_fields: true
      mutually_exclusive: [["sampling.temperature","sampling.top_p"]]
      resolution_preferences: ["sampling.temperature","sampling.top_p"]
      limits:
        max_tool_schema_bytes: 180000
        max_system_prompt_bytes: 30000
    mappings:
      paths:
        "system":                   "$.system"
        "messages":                 "$.messages"
        "sampling.temperature":     "$.temperature"
        "sampling.top_p":           "$.top_p"
        "limits.max_output_tokens": "$.max_tokens"
      flags:
        json_mode: "emulate_tools_or_prompt"
    response_normalization:
      sync:
        content_path: "$.content[-1].text"
        finish_reason_path: "$.stop_reason"
        finish_reason_map:
          end_turn: "stop"
          max_tokens: "length"
          tool_use: "tool_call"
          end: "end_conversation"
      stream:
        protocol: "sse"
        event_selector:
          type_path: "$.type"
          routes:
            - { when: "message_start",        emit: "start" }
            - { when: "content_block_delta",  emit: "delta", text_path: "$.delta" }
            - { when: "tool_use",             emit: "tool",  name_path: "$.name", args_path: "$.input" }
            - { when: "message_stop",         emit: "stop" }
```

---

## Appendix B — Lossiness Codes (CODES.md summary)

* **Clamp**: Numeric value out of range was adjusted (e.g., `temperature` 2.5 → 2.0).
* **Drop**: Field unsupported; removed (e.g., `top_k` dropped).
* **Emulate**: Achieved behavior via non-native means (e.g., JSON via tools).
* **Conflict**: Mutually exclusive keys; resolved per `resolution_preferences`.
* **Relocate**: Field moved (e.g., system message → top-level `system`).
* **Unsupported**: Capability not available; operation continues depending on strictness.
* **MapFallback**: Primary mapping missing; used fallback path.
* **PerformanceImpact**: Likely quality/latency risk (e.g., oversized JSON schema).

**Example Record**

```json
{
  "code": "Conflict",
  "path": "sampling.top_p",
  "message": "temperature and top_p are mutually exclusive; preferred temperature.",
  "before": 0.9,
  "after": null,
  "severity": "warn"
}
```

---

## Appendix C — Example Uniform Prompt

```json
{
  "model_class": "Chat",
  "messages": [
    {"role": "System", "content": "You are a helpful assistant."},
    {"role": "User", "content": "What is the capital of France?"}
  ],
  "sampling": {
    "temperature": 0.7,
    "top_p": 0.9
  },
  "limits": {
    "max_output_tokens": 1000
  },
  "response_format": "text",
  "strict_mode": "Warn"
}
```

---

## Appendix D — Schema Files

The complete JSON Schema definitions for both `PromptSpec` and `ProviderSpec` are maintained as separate files in the repository:

- `/schemas/prompt-spec.schema.json` - Defines the uniform request format
- `/schemas/provider-spec.schema.json` - Defines provider capabilities and mappings

These schemas serve as the authoritative contracts that the engine validates against and should be versioned alongside the codebase.

---

**Document Control**
- Author: Specado Team
- Review: Platform Engineering
- Approval: Product Management
- Distribution: Public
- Version: 1.0.0
- Date: 2025-08-16