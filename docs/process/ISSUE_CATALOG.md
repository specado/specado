# Specado Issue Catalog

_Generated: 2025-10-06_

This catalog mirrors the current Organization Project 14 backlog (epics and tasks). Each section reproduces the issue metadata followed by the full body text.

## #1 — Epic: Foundation (Workspace & Schemas)

- **State:** open
- **Labels:** epic
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/1

### Body

## 📌 Scope
Lay the groundwork for Specado v1.0 by establishing the multi-crate workspace, shared configuration, and schema validation crates that every other component relies on.

## ✅ Exit Criteria
- [ ] `cargo test -p specado-schemas` passes on CI
- [ ] `cargo test -p specado-core` passes for the initial types/auth scaffolding
- [ ] Repository layout and shared dependencies match `SPECADO_PLAN.md`

## 📦 Linked Issues
- [ ] #17 Repository Structure & Root Workspace
- [ ] #18 specado-schemas Crate (Cargo & Validator)
- [ ] #19 Prompt Schema v1 (JSON)
- [ ] #20 Provider Schema v1 (JSON)
- [ ] #21 specado-core Cargo

## 🧭 Next Milestone
Unblock the Core Engine epic by handing off a validated workspace and schemas with passing smoke tests.


## #2 — Epic: Core Engine

- **State:** open
- **Labels:** epic
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/2

### Body

## 📌 Scope
Deliver the Specado core engine: orchestration layer, error model, authentication, translation pipeline, lossiness detection, HTTP client, and resilience features that power all bindings.

## ✅ Exit Criteria
- [ ] `cargo test --workspace` passes with lossiness and transformer suites
- [ ] `specado preview` generates a lossiness report with ≥3 distinct codes
- [ ] Public API for `specado-core` (execute/translate) is frozen and documented

## 📦 Linked Issues
- [ ] #22 Core Orchestration (specado-core/src/lib.rs)
- [ ] #23 Error Model (specado-core/src/error.rs)
- [ ] #24 Auth Handler (specado-core/src/auth.rs)
- [ ] #25 Types Barrel (specado-core/src/types/mod.rs)
- [ ] #26 Prompt Types (specado-core/src/types/prompt.rs)
- [ ] #27 Provider Types (specado-core/src/types/provider.rs)
- [ ] #28 Lossiness Types (specado-core/src/types/lossiness.rs)
- [ ] #29 Uniform Response (specado-core/src/types/response.rs)
- [ ] #30 Transformer Module Barrel (specado-core/src/transformer/mod.rs)
- [ ] #31 Translate (specado-core/src/transformer/translate.rs)
- [ ] #32 Normalize (specado-core/src/transformer/normalize.rs)
- [ ] #33 Detect: Barrel (specado-core/src/transformer/detect/mod.rs)
- [ ] #34 Detect Clamp (specado-core/src/transformer/detect/clamp.rs)
- [ ] #35 Detect Relocate (specado-core/src/transformer/detect/relocate.rs)
- [ ] #36 Detect Unsupported (specado-core/src/transformer/detect/unsupported.rs)
- [ ] #37 Detect Drops (specado-core/src/transformer/detect/drop.rs)
- [ ] #38 HTTP Client (specado-core/src/http/client.rs) & HTTP Module Shim
- [ ] #39 Circuit Breaker (specado-core/src/circuit_breaker.rs)
- [ ] #40 Retry Policy (specado-core/src/retry.rs)
- [ ] #41 Routing (Trait & Primary-Fallback)

## 🧭 Next Milestone
Enable bindings work once the core library APIs, resilience tooling, and tests are locked down and stable.


## #3 — Epic: Developer Interfaces

- **State:** open
- **Labels:** epic
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/3

### Body

## 📌 Scope
Provide first-class developer interfaces by delivering the CLI tooling plus Python and Node.js bindings built on the frozen core API.

## ✅ Exit Criteria
- [ ] `specado-cli` supports `validate`, `preview`, and `run` flows end-to-end
- [ ] `maturin develop && pytest python/tests/` passes against the Python bindings
- [ ] `cd crates/specado-node && npm run build && npm test` passes for Node bindings

## 📦 Linked Issues
- [ ] #42 CLI Cargo (crates/specado-cli/Cargo.toml) & CLI Main
- [ ] #43 Python Native Crate (crates/specado-py) & PyO3 Bindings
- [ ] #44 Python Project Config & Python High-Level API + OpenAI Compat
- [ ] #45 Node.js Bindings (napi-rs + packaging)

## 🧭 Next Milestone
Ship language integrations that consume the stable core API and unblock production hardening.


## #4 — Epic: Production Readiness

- **State:** open
- **Labels:** epic
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/4

### Body

## 📌 Scope
Harden Specado for production use by curating the built-in provider catalog and delivering observability features like hot-reload and audit logging.

## ✅ Exit Criteria
- [ ] Provider catalog covers OpenAI and Anthropic references with validated schemas
- [ ] Hot-reload integration test demonstrates live config swaps without restart
- [ ] Audit logging writes structured JSONL with redacted secrets

## 📦 Linked Issues
- [ ] #46 Provider Catalog (OpenAI, Anthropic)
- [ ] #48 Production Feature: Hot-Reload (Design & Stub)
- [ ] #49 Production Feature: Audit Logging (Design & Stub)

## 🧭 Next Milestone
Once production features are in place, focus shifts to packaging, CI, and documentation for release.


## #5 — Epic: Release Polish

- **State:** open
- **Labels:** epic
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/5

### Body

## 📌 Scope
Polish the product for release by finalising CI/CD coverage, documentation, migration guidance, and example projects.

## ✅ Exit Criteria
- [ ] CI pipeline exercises Rust, Python, and Node targets across each supported OS
- [ ] Docs include up-to-date quickstart and migration guides validated from a clean install
- [ ] Examples and golden/integration test scaffolding exist for end-to-end verification

## 📦 Linked Issues
- [ ] #47 CI/CD Workflow (GitHub Actions)
- [ ] #50 Docs, Tests, Benches, Examples Scaffolding

## 🧭 Next Milestone
After these tasks, we are ready for packaging dry-runs and a public v1.0 announcement.


## #17 — Repository Structure & Root Workspace

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/17

### Body

### 📋 Task Overview
Create the multi-crate workspace with the agreed directory layout and root `Cargo.toml`.

### 🎯 Acceptance Criteria
- [ ] Repository tree matches the structure below.
- [ ] `cargo build --workspace` discovers all members.
- [ ] Release profile set with `lto`, `strip`, single `codegen-units`.

### 📊 Technical Details
#### Implementation Approach
- Create folders and root files.
- Configure `[workspace]` members and `[workspace.dependencies]`.
- Pin minimum Rust version and shared deps.

#### 📄 File(s)
- **Repo Tree (reference)**
```text
specado/
├── Cargo.toml
├── pyproject.toml
├── crates/
│   ├── specado-core/
│   ├── specado-schemas/
│   ├── specado-providers/
│   ├── specado-cli/
│   ├── specado-py/
│   └── specado-node/
├── python/
│   └── specado/
├── tests/
│   ├── golden/
│   └── integration/
└── examples/
````

* **Root Cargo (`specado/Cargo.toml`)**

```toml
[workspace]
resolver = "2"
members = [
    "crates/specado-core",
    "crates/specado-schemas",
    "crates/specado-providers",
    "crates/specado-cli",
    "crates/specado-py",
    "crates/specado-node",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "Apache-2.0"
repository = "https://github.com/yourorg/specado"
rust-version = "1.75"

[workspace.dependencies]
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0.214", features = ["derive"] }
serde_json = "1.0.132"
serde_yaml = "0.9.34"
thiserror = "2.0.3"
async-trait = "0.1.83"
tracing = "0.1.41"
once_cell = "1.19.0"
reqwest = { version = "0.12.9", default-features = false, features = ["json", "rustls-tls"] }
jsonschema = "0.26.1"
serde_jsonpath = "0.3.3"
clap = { version = "4.5.23", features = ["derive", "env"] }
colored = "2.1.0"
specado-core = { path = "crates/specado-core" }
specado-schemas = { path = "crates/specado-schemas" }

[profile.release]
lto = true
codegen-units = 1
strip = true
```

#### Dependencies

* Related to: #18 (Schemas crate init), #21 (Core crate init)

#### API Design

```text
N/A (build system & layout)
```

### ⚠️ Risks & Considerations

* Keep crate names stable to avoid package publication issues.
* Ensure workspace resolver = 2.

### 🧪 Testing Requirements

* Build verification on all OS targets via CI (#47).

### 📚 Documentation Requirements

* [ ] Document workspace layout in `README` or `QUICKSTART.md`.

### 🔗 References

* Epic: #1 - Foundation (Workspace & Schemas)

### ⏱️ Estimates

* **Effort:** S • **Time:** 2–4h • **Complexity:** Low

## #18 — specado-schemas Crate (Cargo & Validator)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/18

### Body

### 📋 Task Overview

Create schemas crate with Cargo config and validator module exposing JSON Schema validation.

### 🎯 Acceptance Criteria

* [ ] `specado-schemas` builds.
* [ ] Validator compiles schemas once via `Lazy`.
* [ ] Friendly validation errors (joined strings).

### 📊 Technical Details

#### Implementation Approach

* Add Cargo with deps from workspace.
* Implement `SchemaValidator` with `get_validator()`.

#### 📄 File(s)

* **`crates/specado-schemas/Cargo.toml`**

```toml
[package]
name = "specado-schemas"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
serde_json.workspace = true
jsonschema.workspace = true
thiserror.workspace = true
once_cell.workspace = true
```

* **`crates/specado-schemas/src/lib.rs`**

```rust
use jsonschema::JSONSchema;
use serde_json::Value;
use thiserror::Error;
use once_cell::sync::Lazy;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Schema compilation failed: {0}")]
    Compilation(String),
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("JSON parsing failed: {0}")]
    JsonParse(#[from] serde_json::Error),
}

pub struct SchemaValidator {
    prompt_schema: JSONSchema,
    provider_schema: JSONSchema,
}

static VALIDATOR: Lazy<SchemaValidator> = Lazy::new(|| {
    SchemaValidator::new().expect("Failed to compile schemas")
});

pub fn get_validator() -> &'static SchemaValidator {
    &VALIDATOR
}

impl SchemaValidator {
    fn new() -> Result<Self, ValidationError> {
        let prompt_schema_json = include_str!("../schemas/prompt-spec.v1.schema.json");
        let provider_schema_json = include_str!("../schemas/provider-spec.v1.schema.json");
        
        let prompt_v: Value = serde_json::from_str(prompt_schema_json)?;
        let provider_v: Value = serde_json::from_str(provider_schema_json)?;
        
        let prompt_schema = JSONSchema::compile(&prompt_v)
            .map_err(|e| ValidationError::Compilation(e.to_string()))?;
        let provider_schema = JSONSchema::compile(&provider_v)
            .map_err(|e| ValidationError::Compilation(e.to_string()))?;
        
        Ok(Self { prompt_schema, provider_schema })
    }
    
    pub fn validate_prompt(&self, prompt: &Value) -> Result<(), ValidationError> {
        self.prompt_schema.validate(prompt).map_err(|errs| {
            ValidationError::Validation(
                errs.into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
    }
    
    pub fn validate_provider(&self, provider: &Value) -> Result<(), ValidationError> {
        self.provider_schema.validate(provider).map_err(|errs| {
            ValidationError::Validation(
                errs.into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
    }
}
```

#### Dependencies

* Uses: #19 (Prompt schema), #20 (Provider schema)

#### API Design

```rust
pub fn get_validator() -> &'static SchemaValidator;
impl SchemaValidator {
  pub fn validate_prompt(&self, v: &serde_json::Value) -> Result<(), ValidationError>;
  pub fn validate_provider(&self, v: &serde_json::Value) -> Result<(), ValidationError>;
}
```

### ⚠️ Risks & Considerations

* Embedding large schemas increases binary size slightly.

### 🧪 Testing Requirements

* Unit: load valid/invalid JSONs and assert error messages.
* Integration: used via CLI validate (#42).

### 📚 Documentation Requirements

* [ ] Short README in `specado-schemas` describing schemas and usage.

### 🔗 References

* Epic: #1 - Foundation (Workspace & Schemas)

### ⏱️ Estimates

* **Effort:** S • **Time:** 2–4h • **Complexity:** Low

## #19 — Prompt Schema v1 (JSON)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/19

### Body

### 📋 Task Overview

Define the v1 prompt schema for messages, sampling, tools, response format, and strict mode.

### 🎯 Acceptance Criteria

* [ ] JSON validates typical prompt payloads.
* [ ] `response.format` defaults to `"text"`.
* [ ] Enum casing matches code.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-schemas/schemas/prompt-spec.v1.schema.json`**

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://specado.dev/schemas/prompt-spec.v1.json",
  "type": "object",
  "required": ["version", "messages"],
  "properties": {
    "version": {
      "type": "string",
      "const": "1"
    },
    "messages": {
      "type": "array",
      "minItems": 1,
      "items": {
        "type": "object",
        "required": ["role", "content"],
        "properties": {
          "role": {
            "type": "string",
            "enum": ["system", "user", "assistant"]
          },
          "content": {
            "type": "string",
            "minLength": 1
          }
        }
      }
    },
    "sampling": {
      "type": "object",
      "properties": {
        "temperature": {"type": "number", "minimum": 0.0, "maximum": 2.0},
        "top_p": {"type": "number", "minimum": 0.0, "maximum": 1.0},
        "top_k": {"type": "integer", "minimum": 0},
        "frequency_penalty": {"type": "number", "minimum": -2.0, "maximum": 2.0},
        "presence_penalty": {"type": "number", "minimum": -2.0, "maximum": 2.0},
        "seed": {"type": "integer"}
      }
    },
    "response": {
      "type": "object",
      "properties": {
        "format": {
          "type": "string",
          "enum": ["text", "json", "json_schema"]
        },
        "json_schema": {
          "type": "object",
          "required": ["name", "schema"],
          "properties": {
            "name": {"type": "string"},
            "description": {"type": "string"},
            "schema": {"type": "object"},
            "strict": {"type": "boolean", "default": false}
          }
        }
      },
      "default": {"format": "text"}
    },
    "tools": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["name", "json_schema"],
        "properties": {
          "name": {"type": "string"},
          "description": {"type": "string"},
          "json_schema": {"type": "object"}
        }
      },
      "default": []
    },
    "tool_choice": {
      "oneOf": [
        {"type": "string", "enum": ["auto", "required"]},
        {
          "type": "object",
          "required": ["name"],
          "properties": {
            "name": {"type": "string"}
          }
        }
      ]
    },
    "strict_mode": {
      "type": "string",
      "enum": ["Strict", "Warn", "Coerce"],
      "default": "Warn"
    },
    "metadata": {
      "type": "object"
    }
  }
}
```

#### Implementation Approach

* Embed schema via `include_str!` (see #18) for fast, local validation.

#### Dependencies

* Related to: #18 (Validator), #26 (Prompt types)

#### API Design

```text
N/A (data contract)
```

### ⚠️ Risks & Considerations

* Future changes require migration doc (#50).

### 🧪 Testing Requirements

* Unit: Valid & invalid messages arrays.
* Unit: `response.format` defaulting behavior via code.

### 📚 Documentation Requirements

* [ ] Add JSON examples in `QUICKSTART.md`.

### 🔗 References

* Epic: #1 - Foundation (Workspace & Schemas)

### ⏱️ Estimates

* **Effort:** S • **Time:** 1–2h • **Complexity:** Low

## #20 — Provider Schema v1 (JSON)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/20

### Body

### 📋 Task Overview

Define v1 schema for provider configs (auth, endpoints, mappings, constraints).

### 🎯 Acceptance Criteria

* [ ] Supports bearer/apikey auth.
* [ ] Validates request/response mapping arrays.
* [ ] `supports` flags include `json_mode`, `tools`.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-schemas/schemas/provider-spec.v1.schema.json`**

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://specado.dev/schemas/provider-spec.v1.json",
  "type": "object",
  "required": ["provider", "models", "endpoints", "mappings", "constraints", "auth"],
  "properties": {
    "provider": {"type": "string"},
    "models": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id"],
        "properties": {
          "id": {"type": "string"}
        }
      }
    },
    "auth": {
      "oneOf": [
        {
          "type": "object",
          "required": ["type", "token_env"],
          "properties": {
            "type": {"const": "bearer"},
            "token_env": {"type": "string"}
          }
        },
        {
          "type": "object",
          "required": ["type", "header", "key_env"],
          "properties": {
            "type": {"const": "apikey"},
            "header": {"type": "string"},
            "key_env": {"type": "string"}
          }
        }
      ]
    },
    "endpoints": {
      "type": "object",
      "required": ["chat"],
      "properties": {
        "chat": {
          "type": "object",
          "required": ["method", "url", "headers"],
          "properties": {
            "method": {"type": "string", "enum": ["POST"]},
            "url": {"type": "string", "format": "uri"},
            "headers": {"type": "object"}
          }
        }
      }
    },
    "mappings": {
      "type": "object",
      "required": ["request", "response"],
      "properties": {
        "request": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["from", "to"],
            "properties": {
              "from": {"type": "string"},
              "to": {"type": "string"},
              "code": {"type": "string"},
              "clamp": {
                "type": "array",
                "items": {"type": "number"},
                "minItems": 2,
                "maxItems": 2
              }
            }
          }
        },
        "response": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["from", "to"],
            "properties": {
              "from": {"type": "string"},
              "to": {"type": "string"}
            }
          }
        }
      }
    },
    "constraints": {
      "type": "object",
      "required": ["supports"],
      "properties": {
        "supports": {
          "type": "object",
          "properties": {
            "json_mode": {"type": "boolean"},
            "tools": {"type": "boolean"}
          }
        }
      }
    }
  }
}
```

#### Implementation Approach

* Same embedding strategy as #19.

#### Dependencies

* Related to: #18 (Validator), #27 (Provider types)

#### API Design

```text
N/A (data contract)
```

### ⚠️ Risks & Considerations

* Mapping JSONPaths must be validated separately by runtime (#31–#37).

### 🧪 Testing Requirements

* Unit: Validate OpenAI/Anthropic YAMLs (#46).

### 📚 Documentation Requirements

* [ ] Provider authoring guide.

### 🔗 References

* Epic: #1 - Foundation (Workspace & Schemas)

### ⏱️ Estimates

* **Effort:** S • **Time:** 1–2h • **Complexity:** Low

## #21 — specado-core Cargo

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/21

### Body

### 📋 Task Overview

Initialize `specado-core` crate with dependencies and bench config.

### 🎯 Acceptance Criteria

* [ ] Crate builds.
* [ ] Criterion bench target configured.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-core/Cargo.toml`**

```toml
[package]
name = "specado-core"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
specado-schemas.workspace = true
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
serde_yaml.workspace = true
reqwest.workspace = true
thiserror.workspace = true
async-trait.workspace = true
tracing.workspace = true
serde_jsonpath.workspace = true
once_cell.workspace = true

[dev-dependencies]
tokio = { workspace = true, features = ["test-util"] }
criterion = "0.5"

[[bench]]
name = "translation"
harness = false
```

#### Implementation Approach

* Set crate deps and test/bench scaffolding.

#### Dependencies

* Uses: #17 (workspace), #18 (schemas)

#### API Design

```text
N/A
```

### ⚠️ Risks & Considerations

* None material.

### 🧪 Testing Requirements

* Build via CI (#47).

### 📚 Documentation Requirements

* [ ] crate-level README.

### 🔗 References

* Epic: #1 - Foundation (Workspace & Schemas)

### ⏱️ Estimates

* **Effort:** XS • **Time:** 0.5–1h • **Complexity:** Low

## #22 — Core Orchestration (specado-core/src/lib.rs)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/22

### Body

### 📋 Task Overview

Implement the top-level `execute` and `translate` exports and module wiring.

### 🎯 Acceptance Criteria

* [ ] `execute` reads provider spec, validates auth, sends request, normalizes.
* [ ] `translate` delegates to transformer.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-core/src/lib.rs`**

```rust
pub mod types;
pub mod auth;
pub mod transformer;
pub mod router;
pub mod http;
pub mod circuit_breaker;
pub mod retry;
pub mod error;

pub use types::*;
pub use auth::{AuthScheme, AuthHandler, AuthError};
pub use error::{Error, Result};

pub async fn execute(prompt: PromptSpec, provider_path: &str) -> Result<UniformResponse> {
    let provider_content = std::fs::read_to_string(provider_path)
        .map_err(|e| Error::Config(format!("Failed to read provider spec: {}", e)))?;
    
    let provider_spec: ProviderSpec = serde_yaml::from_str(&provider_content)
        .map_err(|e| Error::Config(format!("Failed to parse provider spec: {}", e)))?;
    
    let auth_handler = AuthHandler::new(provider_spec.auth.clone());
    auth_handler.validate()?;
    
    let (translated, lossiness) = transformer::translate(&prompt, &provider_spec)?;
    
    if prompt.strict_mode == types::StrictMode::Strict && lossiness.is_lossy {
        return Err(Error::StrictModeViolation);
    }
    
    let mut headers = provider_spec.endpoints.chat.headers.clone();
    auth_handler.inject_headers(&mut headers)?;
    
    let mut header_map = reqwest::header::HeaderMap::new();
    for (k, v) in headers {
        let name = reqwest::header::HeaderName::from_bytes(k.as_bytes())
            .map_err(|e| Error::Config(format!("Invalid header name '{}': {}", k, e)))?;
        let value = reqwest::header::HeaderValue::from_str(&v)
            .map_err(|e| Error::Config(format!("Invalid header value for '{}': {}", k, e)))?;
        header_map.insert(name, value);
    }
    
    let client = http::get_client();
    let response = client
        .post(&provider_spec.endpoints.chat.url)
        .headers(header_map)
        .json(&translated)
        .send()
        .await
        .map_err(Error::Http)?;
    
    let raw_response: serde_json::Value = response.json().await
        .map_err(Error::Http)?;
    
    let mut uniform_response = transformer::normalize(raw_response, &provider_spec)?;
    uniform_response.extensions.lossiness = lossiness;
    
    Ok(uniform_response)
}

pub fn translate(prompt: &PromptSpec, provider: &ProviderSpec) -> Result<(serde_json::Value, types::LossinessReport)> {
    transformer::translate(prompt, provider)
}
```

#### Implementation Approach

* Strict-mode violation check prior to HTTP call.

#### Dependencies

* Uses: #24 (Auth), #38 (HTTP), #31–#32 (Transformer), #23 (Error)

#### API Design

```rust
pub async fn execute(prompt: PromptSpec, provider_path: &str) -> Result<UniformResponse>;
pub fn translate(prompt: &PromptSpec, provider: &ProviderSpec) -> Result<(serde_json::Value, LossinessReport)>;
```

### ⚠️ Risks & Considerations

* Provider spec deserialization errors surfaced as Config.
* Header name/value validation strictness.

### 🧪 Testing Requirements

* Integration: mock HTTP server for 2xx/4xx/5xx/429/timeouts.
* E2E: CLI `run` happy path.

### 📚 Documentation Requirements

* [ ] Mention strict mode behavior.

### 🔗 References

* Epic: #2 - Core Engine

### ⏱️ Estimates

* **Effort:** M • **Time:** 4–6h • **Complexity:** Medium

## #23 — Error Model (specado-core/src/error.rs)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/23

### Body

### 📋 Task Overview

Provide cohesive error enumeration for config/transform/http/auth, etc.

### 🎯 Acceptance Criteria

* [ ] `Error` variants cover main failure modes.
* [ ] `Result<T>` alias defined.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-core/src/error.rs`**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),
    
    #[error("Provider error ({provider}): {kind:?}")]
    Provider {
        provider: String,
        kind: ProviderErrorKind,
    },
    
    #[error("Transform error: {0}")]
    Transform(String),
    
    #[error("Strict mode violation")]
    StrictModeViolation,
    
    #[error("Circuit breaker is open")]
    CircuitBreakerOpen,
    
    #[error("Circuit breaker is half-open, rejecting request")]
    CircuitBreakerHalfOpen,
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Authentication error: {0}")]
    Auth(#[from] crate::auth::AuthError),
}

#[derive(Debug, Clone)]
pub enum ProviderErrorKind {
    RateLimit,
    Timeout,
    InvalidRequest,
    AuthenticationFailed,
    ServerError,
    Unknown,
}

pub type Result<T> = std::result::Result<T, Error>;
```

#### Implementation Approach

* Forward common external errors via `#[from]`.

#### Dependencies

* Related: #24, #39–#40.

#### API Design

```rust
pub enum Error { ... }
pub type Result<T> = std::result::Result<T, Error>;
```

### ⚠️ Risks & Considerations

* Avoid leaking provider secrets via error strings.

### 🧪 Testing Requirements

* Unit: map external errors to variants.
* Integration: error surfacing from `execute`.

### 📚 Documentation Requirements

* [ ] Error handling section in docs.

### 🔗 References

* Epic: #2 - Core Engine

### ⏱️ Estimates

* **Effort:** S • **Time:** 1–2h • **Complexity:** Low

## #24 — Auth Handler (specado-core/src/auth.rs)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/24

### Body

### 📋 Task Overview

Implement auth schemes: bearer, apikey (custom header), and custom headers with `${ENV:VAR}` expansion.

### 🎯 Acceptance Criteria

* [ ] Missing env returns `MissingEnvVar`.
* [ ] Headers injected for all schemes.
* [ ] `validate()` checks all required envs/placeholders.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-core/src/auth.rs`**

```rust
use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Invalid auth configuration: {0}")]
    InvalidConfig(String),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AuthScheme {
    Bearer { token_env: String },
    #[serde(rename = "apikey")]
    ApiKey { header: String, key_env: String },
    Custom { headers: HashMap<String, String> },
}

pub struct AuthHandler {
    scheme: AuthScheme,
}

impl AuthHandler {
    pub fn new(scheme: AuthScheme) -> Self {
        Self { scheme }
    }
    
    fn expand_env_var(template: &str) -> Result<String, AuthError> {
        if let Some(stripped) = template.strip_prefix("${ENV:").and_then(|s| s.strip_suffix('}')) {
            std::env::var(stripped)
                .map_err(|_| AuthError::MissingEnvVar(stripped.to_string()))
        } else {
            Ok(template.to_string())
        }
    }
    
    pub fn inject_headers(&self, headers: &mut HashMap<String, String>) -> Result<(), AuthError> {
        match &self.scheme {
            AuthScheme::Bearer { token_env } => {
                let token = std::env::var(token_env)
                    .map_err(|_| AuthError::MissingEnvVar(token_env.clone()))?;
                headers.insert("Authorization".to_string(), format!("Bearer {}", token));
            }
            AuthScheme::ApiKey { header, key_env } => {
                let key = std::env::var(key_env)
                    .map_err(|_| AuthError::MissingEnvVar(key_env.clone()))?;
                headers.insert(header.clone(), key);
            }
            AuthScheme::Custom { headers: custom } => {
                for (key, value_template) in custom {
                    let value = Self::expand_env_var(value_template)?;
                    headers.insert(key.clone(), value);
                }
            }
        }
        Ok(())
    }
    
    pub fn validate(&self) -> Result<(), AuthError> {
        match &self.scheme {
            AuthScheme::Bearer { token_env } => {
                std::env::var(token_env)
                    .map_err(|_| AuthError::MissingEnvVar(token_env.clone()))?;
            }
            AuthScheme::ApiKey { key_env, .. } => {
                std::env::var(key_env)
                    .map_err(|_| AuthError::MissingEnvVar(key_env.clone()))?;
            }
            AuthScheme::Custom { headers } => {
                for (_, value_template) in headers {
                    Self::expand_env_var(value_template)?;
                }
            }
        }
        Ok(())
    }
}
```

#### Implementation Approach

* Avoid storing tokens in memory beyond header injection.

#### Dependencies

* Uses: #27 (ProviderSpec.auth)

#### API Design

```rust
pub enum AuthScheme { Bearer{token_env}, ApiKey{header,key_env}, Custom{headers} }
pub struct AuthHandler;
impl AuthHandler { pub fn inject_headers(&self, headers: &mut HashMap<String,String>) -> Result<(),AuthError>; }
```

### ⚠️ Risks & Considerations

* Ensure redaction in audit logs later (#48).

### 🧪 Testing Requirements

* Unit: missing env, custom `${ENV:VAR}` resolution.
* Integration: request contains correct header.

### 📚 Documentation Requirements

* [ ] Auth examples per provider.

### 🔗 References

* Epic: #2 - Core Engine

### ⏱️ Estimates

* **Effort:** S • **Time:** 2–3h • **Complexity:** Low

## #25 — Types Barrel (specado-core/src/types/mod.rs)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/25

### Body

### 📋 Task Overview

Re-export core types from submodules.

### 🎯 Acceptance Criteria

* [ ] Public re-export compiles and shortens imports.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-core/src/types/mod.rs`**

```rust
pub mod prompt;
pub mod provider;
pub mod lossiness;
pub mod response;

pub use prompt::{PromptSpec, Message, MessageRole, SamplingConfig, ResponseConfig, ResponseFormat, StrictMode};
pub use provider::{ProviderSpec, ModelConfig, Endpoints, EndpointConfig, Mappings, RequestMapping, ResponseMapping, Constraints, SupportFlags};
pub use lossiness::{LossinessReport, LossinessEntry, LossinessCode, LossinessLevel};
pub use response::{UniformResponse, FinishReason, Usage, Extensions};
```

#### Dependencies

* Uses: #26–#29

### 🧪 Testing Requirements

* Compile-time sanity via CI.

### 🔗 References

* Epic: #2 - Core Engine

### ⏱️ Estimates

* **Effort:** XS • **Time:** 0.5h • **Complexity:** Low

## #26 — Prompt Types (specado-core/src/types/prompt.rs)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/26

### Body

### 📋 Task Overview

Define `PromptSpec` and related enums/structs matching the schema.

### 🎯 Acceptance Criteria

* [ ] Enum casing aligns with schema.
* [ ] `ResponseConfig` defaults to text.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-core/src/types/prompt.rs`**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSpec {
    pub version: String,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub sampling: SamplingConfig,
    #[serde(default)]
    pub response: ResponseConfig,
    #[serde(default)]
    pub tools: Vec<Tool>,
    #[serde(default)]
    pub tool_choice: Option<ToolChoice>,
    #[serde(default)]
    pub strict_mode: StrictMode,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SamplingConfig {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<u32>,
    pub frequency_penalty: Option<f64>,
    pub presence_penalty: Option<f64>,
    pub seed: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseConfig {
    #[serde(default = "default_format")]
    pub format: ResponseFormat,
    pub json_schema: Option<JsonSchema>,
}

fn default_format() -> ResponseFormat {
    ResponseFormat::Text
}

impl Default for ResponseConfig {
    fn default() -> Self {
        Self {
            format: ResponseFormat::Text,
            json_schema: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormat {
    Text,
    Json,
    JsonSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchema {
    pub name: String,
    pub description: Option<String>,
    pub schema: serde_json::Value,
    #[serde(default)]
    pub strict: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    pub json_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    String(ToolChoiceString),
    Object { name: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceString {
    Auto,
    Required,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum StrictMode {
    Strict,
    #[default]
    Warn,
    Coerce,
}
```

#### Dependencies

* Related to: #19 (Prompt schema)

### 🧪 Testing Requirements

* Unit: serde round-trips.
* Unit: default format behavior.

### 🔗 References

* Epic: #2 - Core Engine

### ⏱️ Estimates

* **Effort:** S • **Time:** 2–3h • **Complexity:** Low

## #27 — Provider Types (specado-core/src/types/provider.rs)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/27

### Body

### 📋 Task Overview

Define `ProviderSpec` and related config types.

### 🎯 Acceptance Criteria

* [ ] Matches provider schema.
* [ ] `auth` uses `AuthScheme`.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-core/src/types/provider.rs`**

```rust
use serde::Deserialize;
use std::collections::HashMap;
use crate::auth::AuthScheme;

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderSpec {
    pub provider: String,
    pub models: Vec<ModelConfig>,
    pub endpoints: Endpoints,
    pub mappings: Mappings,
    pub constraints: Constraints,
    pub auth: AuthScheme,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelConfig {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Endpoints {
    pub chat: EndpointConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EndpointConfig {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Mappings {
    pub request: Vec<RequestMapping>,
    pub response: Vec<ResponseMapping>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequestMapping {
    pub from: String,
    pub to: String,
    pub code: Option<String>,
    pub clamp: Option<[f64; 2]>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResponseMapping {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Constraints {
    pub supports: SupportFlags,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SupportFlags {
    pub json_mode: bool,
    pub tools: bool,
}
```

#### Dependencies

* Uses: #20 (Provider schema), #24 (Auth)

### 🧪 Testing Requirements

* Unit: serde deserialization from YAML.

### 🔗 References

* Epic: #2 - Core Engine

### ⏱️ Estimates

* **Effort:** S • **Time:** 2h • **Complexity:** Low

## #28 — Lossiness Types (specado-core/src/types/lossiness.rs)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/28

### Body

### 📋 Task Overview

Implement lossiness report types and helpers.

### 🎯 Acceptance Criteria

* [ ] Add entries toggles `is_lossy`.
* [ ] Omissions list maintained.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-core/src/types/lossiness.rs`**

```rust
use serde::{Deserialize, Serialize};
use super::StrictMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LossinessReport {
    pub is_lossy: bool,
    pub strict_mode: StrictMode,
    pub entries: Vec<LossinessEntry>,
    pub omissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LossinessEntry {
    pub code: LossinessCode,
    pub level: LossinessLevel,
    pub path: String,
    pub reason: String,
    pub suggested_fix: Option<String>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LossinessCode {
    Clamp,
    Drop,
    Emulate,
    Conflict,
    Relocate,
    Unsupported,
    MapFallback,
    PerformanceImpact,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LossinessLevel {
    Info,
    Warn,
    Error,
}

impl LossinessReport {
    pub fn new(strict_mode: StrictMode) -> Self {
        Self {
            is_lossy: false,
            strict_mode,
            entries: Vec::new(),
            omissions: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: LossinessEntry) {
        self.is_lossy = true;
        self.entries.push(entry);
    }

    pub fn add_omission(&mut self, path: String) {
        self.omissions.push(path);
    }
}
```

#### Dependencies

* Used by: #31–#37 and #22

### 🧪 Testing Requirements

* Unit: `add_entry` sets `is_lossy = true`.

### 🔗 References

* Epic: #2 - Core Engine

### ⏱️ Estimates

* **Effort:** XS • **Time:** 1h • **Complexity:** Low

## #29 — Uniform Response (specado-core/src/types/response.rs)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/29

### Body

### 📋 Task Overview

Define provider-normalized response structure.

### 🎯 Acceptance Criteria

* [ ] Finish reasons mapped snake_case.
* [ ] Extensions carry `LossinessReport`.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-core/src/types/response.rs`**

```rust
use serde::{Deserialize, Serialize};
use super::LossinessReport;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniformResponse {
    pub content: String,
    pub tool_calls: Vec<serde_json::Value>,
    pub finish_reason: FinishReason,
    pub model: String,
    pub provider_used: String,
    pub usage: Option<Usage>,
    pub extensions: Extensions,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ToolCall,
    ContentFilter,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extensions {
    pub lossiness: LossinessReport,
}
```

#### Dependencies

* Used by: #22, #32

### 🧪 Testing Requirements

* Unit: serde (de)serialization.

### 🔗 References

* Epic: #2 - Core Engine

### ⏱️ Estimates

* **Effort:** XS • **Time:** 1h • **Complexity:** Low

## #30 — Transformer Module Barrel (specado-core/src/transformer/mod.rs)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/30

### Body

### 📋 Task Overview

Expose `translate` and `normalize` modules.

### 🎯 Acceptance Criteria

* [ ] Public re-exports compile.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-core/src/transformer/mod.rs`**

```rust
pub mod translate;
pub mod normalize;
pub mod detect;

pub use translate::translate;
pub use normalize::normalize;
```

#### Dependencies

* Uses: #31, #32, #33–#37

### 🧪 Testing Requirements

* Compile-time sanity.

### 🔗 References

* Epic: #2 - Core Engine

### ⏱️ Estimates

* **Effort:** XS • **Time:** 0.5h • **Complexity:** Low

## #31 — Translate (specado-core/src/transformer/translate.rs)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/31

### Body

### 📋 Task Overview

Map `PromptSpec` to provider request JSON using JSONPath and record lossiness.

### 🎯 Acceptance Criteria

* [ ] Applies `clamp` where declared.
* [ ] Records `Relocate`, `Unsupported`, `Drop` signals.
* [ ] Builds nested JSON at target paths.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-core/src/transformer/translate.rs`**

```rust
use crate::types::{PromptSpec, ProviderSpec, LossinessReport};
use crate::error::{Error, Result};
use serde_json::{json, Value};
use serde_jsonpath::JsonPath;

pub fn translate(prompt: &PromptSpec, provider: &ProviderSpec) -> Result<(Value, LossinessReport)> {
    let mut report = LossinessReport::new(prompt.strict_mode);
    let mut payload = json!({});
    
    let prompt_value = serde_json::to_value(prompt)
        .map_err(|e| Error::Transform(format!("Failed to serialize prompt: {}", e)))?;
    
    for mapping in &provider.mappings.request {
        let path = JsonPath::parse(&mapping.from)
            .map_err(|e| Error::Transform(format!("Invalid JSONPath '{}': {}", mapping.from, e)))?;
        
        let matches = path.query(&prompt_value).all();
        
        let value = match matches.len() {
            0 => continue,
            1 => matches[0].clone(),
            _ => Value::Array(matches.iter().map(|v| (*v).clone()).collect()),
        };
        
        let processed_value = if let Some(clamp_range) = mapping.clamp {
            if let Some(num) = value.as_f64() {
                json!(super::detect::clamp_value(num, clamp_range, mapping.from.clone(), &mut report))
            } else {
                value
            }
        } else {
            value
        };
        
        set_value_at_path(&mut payload, &mapping.to, processed_value)?;
        
        if let Some(code) = &mapping.code {
            if code == "Relocate" {
                super::detect::detect_relocate(prompt, provider, &mut report);
            }
        }
    }
    
    super::detect::detect_unsupported(prompt, provider, &mut report);
    super::detect::detect_drops(prompt, provider, &mut report);
    
    Ok((payload, report))
}

fn set_value_at_path(target: &mut Value, path: &str, value: Value) -> Result<()> {
    let parts: Vec<&str> = path.trim_start_matches("$.").split('.').collect();
    
    let mut current = target;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            if let Some(obj) = current.as_object_mut() {
                obj.insert(part.to_string(), value);
            }
            break;
        }
        
        if !current.get(part).is_some() {
            if let Some(obj) = current.as_object_mut() {
                obj.insert(part.to_string(), json!({}));
            }
        }
        
        current = current.get_mut(part).ok_or_else(|| {
            Error::Transform(format!("Failed to navigate to path: {}", path))
        })?;
    }
    
    Ok(())
}
```

#### Dependencies

* Uses: #33–#37 detectors, #28 lossiness

### 🧪 Testing Requirements

* Unit: clamp range, relocate detection, omissions.
* Golden: payload snapshots per provider.

### 🔗 References

* Epic: #2 - Core Engine

### ⏱️ Estimates

* **Effort:** M • **Time:** 5–8h • **Complexity:** Medium

## #32 — Normalize (specado-core/src/transformer/normalize.rs)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/32

### Body

### 📋 Task Overview

Extract `content` and `finish_reason` from raw provider JSON using response mappings.

### 🎯 Acceptance Criteria

* [ ] Maps finish reasons via helper.
* [ ] Uses provider model and name for metadata.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-core/src/transformer/normalize.rs`**

```rust
use crate::types::{ProviderSpec, UniformResponse, FinishReason, Extensions, LossinessReport, StrictMode};
use crate::error::Result;
use serde_json::Value;
use serde_jsonpath::JsonPath;

pub fn normalize(raw: Value, provider: &ProviderSpec) -> Result<UniformResponse> {
    let mut content = String::new();
    let mut finish_reason = FinishReason::Stop;

    for mapping in &provider.mappings.response {
        let path = JsonPath::parse(&mapping.from)
            .map_err(|e| crate::error::Error::Transform(format!("Invalid JSONPath: {}", e)))?;
        
        let matches = path.query(&raw).all();
        
        let value = match matches.len() {
            0 => continue,
            1 => matches[0].clone(),
            _ => Value::Array(matches.iter().map(|v| (*v).clone()).collect()),
        };
        
        match mapping.to.as_str() {
            "content" => {
                content = value.as_str().unwrap_or("").to_string();
            }
            "finish_reason" => {
                if let Some(reason_str) = value.as_str() {
                    finish_reason = map_finish_reason(reason_str);
                }
            }
            _ => {}
        }
    }

    Ok(UniformResponse {
        content,
        tool_calls: vec![],
        finish_reason,
        model: provider.models[0].id.clone(),
        provider_used: provider.provider.clone(),
        usage: None,
        extensions: Extensions {
            lossiness: LossinessReport::new(StrictMode::Warn),
        },
    })
}

fn map_finish_reason(raw: &str) -> FinishReason {
    match raw {
        "stop" | "end_turn" => FinishReason::Stop,
        "length" | "max_tokens" => FinishReason::Length,
        "tool_calls" | "tool_use" => FinishReason::ToolCall,
        "content_filter" => FinishReason::ContentFilter,
        _ => FinishReason::Error,
    }
}
```

#### Dependencies

* Uses: #29, #27

### 🧪 Testing Requirements

* Unit: JSONPath extraction.
* Unit: reason mapping matrix.

### 🔗 References

* Epic: #2 - Core Engine

### ⏱️ Estimates

* **Effort:** S • **Time:** 2–3h • **Complexity:** Low

## #33 — Detect: Barrel (specado-core/src/transformer/detect/mod.rs)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/33

### Body

### 📋 Task Overview

Expose detector functions.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-core/src/transformer/detect/mod.rs`**

```rust
pub mod clamp;
pub mod relocate;
pub mod unsupported;
pub mod drop;

pub use clamp::clamp_value;
pub use relocate::detect_relocate;
pub use unsupported::detect_unsupported;
pub use drop::detect_drops;
```

### 🎯 Acceptance Criteria

* [ ] Re-exports compile.

### Dependencies

* Uses: #34–#37

### 🧪 Testing Requirements

* Compile test only here.

### 🔗 References

* Epic: #2 - Core Engine

### ⏱️ Estimates

* **Effort:** XS • **Time:** 0.5h • **Complexity:** Low

## #34 — Detect Clamp (specado-core/src/transformer/detect/clamp.rs)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/34

### Body

### 📋 Task Overview

Clamp numeric values and record lossiness entries.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-core/src/transformer/detect/clamp.rs`**

```rust
use crate::types::{LossinessReport, LossinessEntry, LossinessCode, LossinessLevel};
use serde_json::json;

pub fn clamp_value(value: f64, range: [f64; 2], path: String, report: &mut LossinessReport) -> f64 {
    let [min, max] = range;
    if value < min {
        report.add_entry(LossinessEntry {
            code: LossinessCode::Clamp,
            level: LossinessLevel::Warn,
            path,
            reason: format!("Value {} below minimum {}", value, min),
            suggested_fix: Some(format!("Use value >= {}", min)),
            details: Some(json!({"requested": value, "clamped_to": min})),
        });
        min
    } else if value > max {
        report.add_entry(LossinessEntry {
            code: LossinessCode::Clamp,
            level: LossinessLevel::Warn,
            path,
            reason: format!("Value {} above maximum {}", value, max),
            suggested_fix: Some(format!("Use value <= {}", max)),
            details: Some(json!({"requested": value, "clamped_to": max})),
        });
        max
    } else {
        value
    }
}
```

### 🎯 Acceptance Criteria

* [ ] Entry created when clamp occurs.

### Dependencies

* Related: #28

### 🧪 Testing Requirements

* Unit: less-than-min, greater-than-max, in-range.

### 🔗 References

* Epic: #2 - Core Engine

### ⏱️ Estimates

* **Effort:** XS • **Time:** 1h • **Complexity:** Low

## #35 — Detect Relocate (specado-core/src/transformer/detect/relocate.rs)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/35

### Body

### 📋 Task Overview

Record info lossiness when system prompt is relocated.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-core/src/transformer/detect/relocate.rs`**

```rust
use crate::types::{PromptSpec, ProviderSpec, MessageRole, LossinessReport, LossinessEntry, LossinessCode, LossinessLevel};
use serde_json::json;

pub fn detect_relocate(
    prompt: &PromptSpec,
    provider: &ProviderSpec,
    report: &mut LossinessReport
) {
    let has_system = prompt.messages.iter().any(|m| m.role == MessageRole::System);
    
    if has_system {
        for mapping in &provider.mappings.request {
            if mapping.code.as_deref() == Some("Relocate") {
                report.add_entry(LossinessEntry {
                    code: LossinessCode::Relocate,
                    level: LossinessLevel::Info,
                    path: "messages[0]".to_string(),
                    reason: "System message relocated to provider-specific location".to_string(),
                    suggested_fix: None,
                    details: Some(json!({
                        "from": mapping.from,
                        "to": mapping.to
                    })),
                });
                break;
            }
        }
    }
}
```

### 🎯 Acceptance Criteria

* [ ] Entry added only when applicable.

### Dependencies

* Uses: #26 (Prompt), #27 (Provider)

### 🧪 Testing Requirements

* Unit: with and without system message.

### 🔗 References

* Epic: #2 - Core Engine

### ⏱️ Estimates

* **Effort:** XS • **Time:** 1h • **Complexity:** Low

## #36 — Detect Unsupported (specado-core/src/transformer/detect/unsupported.rs)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/36

### Body

### 📋 Task Overview

Warn/error when provider lacks JSON mode or tools support.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-core/src/transformer/detect/unsupported.rs`**

```rust
use crate::types::{PromptSpec, ProviderSpec, ResponseFormat, LossinessReport, LossinessEntry, LossinessCode, LossinessLevel};
use serde_json::json;

pub fn detect_unsupported(
    prompt: &PromptSpec,
    provider: &ProviderSpec,
    report: &mut LossinessReport
) {
    if prompt.response.format == ResponseFormat::Json && !provider.constraints.supports.json_mode {
        report.add_entry(LossinessEntry {
            code: LossinessCode::Unsupported,
            level: LossinessLevel::Warn,
            path: "response.format".to_string(),
            reason: "Provider does not support native JSON mode".to_string(),
            suggested_fix: Some("Use a provider with native JSON support or accept emulation".to_string()),
            details: Some(json!({"requested": "json", "supported": false})),
        });
    }

    if !prompt.tools.is_empty() && !provider.constraints.supports.tools {
        report.add_entry(LossinessEntry {
            code: LossinessCode::Unsupported,
            level: LossinessLevel::Error,
            path: "tools".to_string(),
            reason: "Provider does not support tools".to_string(),
            suggested_fix: Some("Remove tools or use a different provider".to_string()),
            details: Some(json!({"tool_count": prompt.tools.len()})),
        });
    }
}
```

### 🎯 Acceptance Criteria

* [ ] JSON mode & tools conditions handled.

### Dependencies

* Uses: #26, #27

### 🧪 Testing Requirements

* Unit: both flags false/true, tools count > 0.

### 🔗 References

* Epic: #2 - Core Engine

### ⏱️ Estimates

* **Effort:** XS • **Time:** 1h • **Complexity:** Low

## #37 — Detect Drops (specado-core/src/transformer/detect/drop.rs)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/37

### Body

### 📋 Task Overview

Record when parameters (e.g., `top_k`) aren’t mapped to provider.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-core/src/transformer/detect/drop.rs`**

```rust
use crate::types::{PromptSpec, ProviderSpec, LossinessReport, LossinessEntry, LossinessCode, LossinessLevel};
use serde_json::json;

pub fn detect_drops(
    prompt: &PromptSpec,
    provider: &ProviderSpec,
    report: &mut LossinessReport
) {
    if let Some(top_k) = prompt.sampling.top_k {
        let has_top_k_mapping = provider.mappings.request.iter()
            .any(|m| m.from.contains("top_k"));
        
        if !has_top_k_mapping {
            report.add_entry(LossinessEntry {
                code: LossinessCode::Drop,
                level: LossinessLevel::Warn,
                path: "sampling.top_k".to_string(),
                reason: "Parameter not supported by provider".to_string(),
                suggested_fix: Some("Remove top_k or use a provider that supports it".to_string()),
                details: Some(json!({"requested": top_k})),
            });
            report.add_omission("$.sampling.top_k".to_string());
        }
    }
}
```

### 🎯 Acceptance Criteria

* [ ] Omissions recorded in report.

### Dependencies

* Uses: #28

### 🧪 Testing Requirements

* Unit: present/absent mapping cases.

### 🔗 References

* Epic: #2 - Core Engine

### ⏱️ Estimates

* **Effort:** XS • **Time:** 1h • **Complexity:** Low

## #38 — HTTP Client (specado-core/src/http/client.rs) & HTTP Module Shim

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/38

### Body

### 📋 Task Overview

Provide pooled `reqwest::Client` via `once_cell`. Expose `get_client()`.

### 🎯 Acceptance Criteria

* [ ] Singleton client with timeouts and pool tuning.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-core/src/http/client.rs`**

```rust
use once_cell::sync::Lazy;
use reqwest::Client;
use std::time::Duration;

pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(60))
        .pool_max_idle_per_host(10)
        .connect_timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client")
});

pub fn get_client() -> &'static Client {
    &HTTP_CLIENT
}
```

* **`crates/specado-core/src/http/mod.rs`**

```rust
pub mod client;
pub use client::get_client;
```

#### Dependencies

* Used by: #22

### 🧪 Testing Requirements

* Integration: multiple calls reuse same client (observable via instrumentation).

### 🔗 References

* Epic: #2 - Core Engine

### ⏱️ Estimates

* **Effort:** XS • **Time:** 1h • **Complexity:** Low

## #39 — Circuit Breaker (specado-core/src/circuit_breaker.rs)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/39

### Body

### 📋 Task Overview

Implement Closed/Open/Half-Open states with thresholds/timeouts.

### 🎯 Acceptance Criteria

* [ ] Open after threshold failures.
* [ ] Half-open allows limited test requests.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-core/src/circuit_breaker.rs`**

```rust
use tokio::sync::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::error::{Error, Result};

pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_threshold: usize,
    timeout: Duration,
    half_open_max_requests: usize,
}

enum CircuitState {
    Closed { failure_count: usize },
    Open { opened_at: Instant },
    HalfOpen { success_count: usize, failure_count: usize },
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed { failure_count: 0 })),
            failure_threshold: 5,
            timeout: Duration::from_secs(30),
            half_open_max_requests: 3,
        }
    }

    pub async fn call<F, T>(&self, f: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        let mut state = self.state.lock().await;
        
        match *state {
            CircuitState::Open { opened_at } => {
                if opened_at.elapsed() > self.timeout {
                    *state = CircuitState::HalfOpen { success_count: 0, failure_count: 0 };
                } else {
                    return Err(Error::CircuitBreakerOpen);
                }
            }
            CircuitState::HalfOpen { success_count, .. } => {
                if success_count >= self.half_open_max_requests {
                    return Err(Error::CircuitBreakerHalfOpen);
                }
            }
            _ => {}
        }
        
        drop(state);
        
        match f.await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(e) => {
                self.on_failure().await;
                Err(e)
            }
        }
    }

    async fn on_success(&self) {
        let mut state = self.state.lock().await;
        match *state {
            CircuitState::HalfOpen { success_count, .. } => {
                if success_count + 1 >= self.half_open_max_requests {
                    *state = CircuitState::Closed { failure_count: 0 };
                } else {
                    if let CircuitState::HalfOpen { ref mut success_count, .. } = *state {
                        *success_count += 1;
                    }
                }
            }
            CircuitState::Closed { .. } => {
                *state = CircuitState::Closed { failure_count: 0 };
            }
            _ => {}
        }
    }

    async fn on_failure(&self) {
        let mut state = self.state.lock().await;
        match *state {
            CircuitState::Closed { failure_count } => {
                if failure_count + 1 >= self.failure_threshold {
                    *state = CircuitState::Open { opened_at: Instant::now() };
                } else {
                    *state = CircuitState::Closed { failure_count: failure_count + 1 };
                }
            }
            CircuitState::HalfOpen { .. } => {
                *state = CircuitState::Open { opened_at: Instant::now() };
            }
            _ => {}
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}
```

#### Dependencies

* Related: #23

### 🧪 Testing Requirements

* Unit: state transitions.

### 🔗 References

* Epic: #2 - Core Engine

### ⏱️ Estimates

* **Effort:** M • **Time:** 4–6h • **Complexity:** Medium

## #40 — Retry Policy (specado-core/src/retry.rs)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/40

### Body

### 📋 Task Overview

Backoff retries with cap and attempt limit.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-core/src/retry.rs`**

```rust
use std::time::Duration;
use crate::error::Result;

pub struct RetryPolicy {
    max_attempts: usize,
    base_delay: Duration,
    max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
        }
    }
}

impl RetryPolicy {
    pub async fn execute<F, T>(&self, mut f: F) -> Result<T>
    where
        F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>,
    {
        let mut attempt = 0;
        loop {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) if attempt + 1 >= self.max_attempts => return Err(e),
                Err(_) => {
                    attempt += 1;
                    let delay = self.base_delay * 2u32.pow(attempt as u32);
                    let delay = delay.min(self.max_delay);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
}
```

### 🎯 Acceptance Criteria

* [ ] Exponential backoff with cap.

#### Dependencies

* Related: #22 (future use)

### 🧪 Testing Requirements

* Unit: attempt counting and delay capping (mock/simulate).

### 🔗 References

* Epic: #2 - Core Engine

### ⏱️ Estimates

* **Effort:** S • **Time:** 2–3h • **Complexity:** Low

## #41 — Routing (Trait & Primary-Fallback)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/41

### Body

### 📋 Task Overview

Define routing trait and a basic PrimaryFallback router.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-core/src/router/traits.rs`**

```rust
use async_trait::async_trait;
use crate::types::{PromptSpec, UniformResponse};
use crate::error::Result;

#[async_trait]
pub trait Router: Send + Sync {
    async fn route(&self, prompt: PromptSpec) -> Result<UniformResponse>;
}
```

* **`crates/specado-core/src/router/primary_fallback.rs`**

```rust
use async_trait::async_trait;
use crate::router::traits::Router;
use crate::types::{PromptSpec, UniformResponse};
use crate::error::Result;

pub struct PrimaryFallbackRouter {
    primary: String,
    fallbacks: Vec<String>,
}

impl PrimaryFallbackRouter {
    pub fn new(primary: String, fallbacks: Vec<String>) -> Self {
        Self { primary, fallbacks }
    }
}

#[async_trait]
impl Router for PrimaryFallbackRouter {
    async fn route(&self, prompt: PromptSpec) -> Result<UniformResponse> {
        crate::execute(prompt, &self.primary).await
    }
}
```

* **`crates/specado-core/src/router/mod.rs`**

```rust
pub mod traits;
pub mod primary_fallback;

pub use traits::Router;
pub use primary_fallback::PrimaryFallbackRouter;
```

### 🎯 Acceptance Criteria

* [ ] `route()` compiles and calls `execute`.

#### Dependencies

* Uses: #22

### 🧪 Testing Requirements

* Unit: simple call-through test using mock provider.

### 🔗 References

* Epic: #2 - Core Engine

### ⏱️ Estimates

* **Effort:** S • **Time:** 2–3h • **Complexity:** Low

## #42 — CLI Cargo (crates/specado-cli/Cargo.toml) & CLI Main

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/42

### Body

### 📋 Task Overview

Implement `specado` CLI with `validate`, `preview`, `run`.

### 🎯 Acceptance Criteria

* [ ] `validate` auto-detects prompt vs provider.
* [ ] `preview` prints translated JSON + lossiness.
* [ ] `run` prints uniform response JSON.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-cli/Cargo.toml`**

```toml
[package]
name = "specado-cli"
version.workspace = true
edition.workspace = true
license.workspace = true

[[bin]]
name = "specado"
path = "src/main.rs"

[dependencies]
specado-core.workspace = true
specado-schemas.workspace = true
clap.workspace = true
colored.workspace = true
tokio.workspace = true
serde_json.workspace = true
serde_yaml.workspace = true
anyhow = "1.0"
```

* **`crates/specado-cli/src/main.rs`**

```rust
use clap::{Parser, Subcommand};
use colored::*;
use specado_core::{PromptSpec, ProviderSpec};
use specado_schemas;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "specado")]
#[command(version, about = "Spec-driven LLM abstraction", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Validate {
        #[arg(long)]
        spec: PathBuf,
    },
    Preview {
        #[arg(long)]
        prompt: PathBuf,
        #[arg(long)]
        provider: PathBuf,
    },
    Run {
        #[arg(long)]
        prompt: PathBuf,
        #[arg(long)]
        provider: PathBuf,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    let result = match cli.command {
        Commands::Validate { spec } => validate_command(spec).await,
        Commands::Preview { prompt, provider } => preview_command(prompt, provider).await,
        Commands::Run { prompt, provider } => run_command(prompt, provider).await,
    };
    
    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }
}

async fn validate_command(spec_path: PathBuf) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(&spec_path)?;
    
    let ext = spec_path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let spec_value: serde_json::Value = if matches!(ext, "yaml" | "yml") {
        serde_yaml::from_str(&content)?
    } else {
        serde_json::from_str(&content)?
    };
    
    let validator = specado_schemas::get_validator();
    
    let looks_provider = spec_value.get("provider").is_some() 
        || spec_value.get("endpoints").is_some()
        || spec_value.get("auth").is_some();
    
    if looks_provider {
        validator.validate_provider(&spec_value)?;
        println!("{} Provider spec is valid", "✓".green().bold());
    } else {
        validator.validate_prompt(&spec_value)?;
        println!("{} Prompt spec is valid", "✓".green().bold());
    }
    
    Ok(())
}

async fn preview_command(prompt_path: PathBuf, provider_path: PathBuf) -> anyhow::Result<()> {
    let prompt_content = std::fs::read_to_string(&prompt_path)?;
    let prompt_spec: PromptSpec = serde_json::from_str(&prompt_content)?;
    
    let provider_content = std::fs::read_to_string(&provider_path)?;
    let provider_spec: ProviderSpec = serde_yaml::from_str(&provider_content)?;
    
    let (translated, lossiness) = specado_core::translate(&prompt_spec, &provider_spec)?;
    
    println!("{}", "=== Translated Request ===".cyan().bold());
    println!("{}", serde_json::to_string_pretty(&translated)?);
    println!();
    
    println!("{}", "=== Lossiness Report ===".yellow().bold());
    if lossiness.is_lossy {
        for entry in &lossiness.entries {
            let level_str = match entry.level {
                specado_core::LossinessLevel::Info => "INFO".blue(),
                specado_core::LossinessLevel::Warn => "WARN".yellow(),
                specado_core::LossinessLevel::Error => "ERROR".red(),
            };
            println!("{} {:?}: {}", level_str, entry.code, entry.reason);
        }
    } else {
        println!("{} No lossiness detected", "✓".green().bold());
    }
    
    Ok(())
}

async fn run_command(prompt_path: PathBuf, provider_path: PathBuf) -> anyhow::Result<()> {
    let prompt_content = std::fs::read_to_string(&prompt_path)?;
    let prompt_spec: PromptSpec = serde_json::from_str(&prompt_content)?;
    
    let response = specado_core::execute(prompt_spec, provider_path.to_str().unwrap()).await?;
    
    println!("{}", serde_json::to_string_pretty(&response)?);
    
    Ok(())
}
```

#### Dependencies

* Uses: #18–#20, #22, #31–#32

### 🧪 Testing Requirements

* E2E: run all commands with sample files.

### 🔗 References

* Epic: #3 - Developer Interfaces

### ⏱️ Estimates

* **Effort:** M • **Time:** 4–6h • **Complexity:** Medium

## #43 — Python Native Crate (crates/specado-py) & PyO3 Bindings

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/43

### Body

### 📋 Task Overview

Expose `Client.complete()` to Python via PyO3 with embedded Tokio runtime.

### 🎯 Acceptance Criteria

* [ ] Build via `maturin develop`.
* [ ] `Client(provider_path).complete(dict)` returns JSON dict.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-py/Cargo.toml`**

```toml
[package]
name = "specado-py"
version.workspace = true
edition.workspace = true
license.workspace = true

[lib]
name = "specado"
crate-type = ["cdylib"]

[dependencies]
specado-core.workspace = true
pyo3 = { version = "0.22.6", features = ["extension-module", "abi3-py39"] }
tokio.workspace = true
serde_json.workspace = true
pythonize = "0.22.0"
once_cell.workspace = true
```

* **`crates/specado-py/src/lib.rs`**

```rust
use pyo3::prelude::*;
use pyo3::types::PyDict;
use specado_core::{PromptSpec, UniformResponse};
use once_cell::sync::Lazy;

static RUNTIME: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
});

#[pyclass]
struct Client {
    provider_path: String,
}

#[pymethods]
impl Client {
    #[new]
    fn new(provider_path: String) -> PyResult<Self> {
        Ok(Self { provider_path })
    }

    fn complete(&self, py: Python, prompt: &Bound<'_, PyDict>) -> PyResult<PyObject> {
        let prompt_json: serde_json::Value = pythonize::depythonize(prompt)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        
        let prompt_spec: PromptSpec = serde_json::from_value(prompt_json)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        
        let provider_path = self.provider_path.clone();
        
        let response: UniformResponse = py.allow_threads(|| {
            RUNTIME.block_on(async {
                specado_core::execute(prompt_spec, &provider_path).await
            })
        })
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        
        let result_json = serde_json::to_value(&response)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        
        pythonize::pythonize(py, &result_json)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }
}

#[pymodule]
fn specado(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Client>()?;
    Ok(())
}
```

#### Dependencies

* Uses: #22

### 🧪 Testing Requirements

* Integration: Python pytest calls `Client.complete`.

### 🔗 References

* Epic: #3 - Developer Interfaces

### ⏱️ Estimates

* **Effort:** M • **Time:** 4–6h • **Complexity:** Medium

## #44 — Python Project Config & Python High-Level API + OpenAI Compat

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/44

### Body

### 📋 Task Overview

Ship `pyproject.toml`, Python wrapper, and a minimal OpenAI-compat shim.

### 🎯 Acceptance Criteria

* [ ] `pip install -e .` via `maturin develop`.
* [ ] `OpenAI(...).chat.completions.create(...)` path returns message.

### 📊 Technical Details

#### 📄 File(s)

* **`pyproject.toml`**

```toml
[build-system]
requires = ["maturin>=1.7,<2.0"]
build-backend = "maturin"

[project]
name = "specado"
version = "0.1.0"
description = "Spec-driven LLM abstraction library"
requires-python = ">=3.9"
license = {text = "Apache-2.0"}
classifiers = [
    "Programming Language :: Rust",
    "Programming Language :: Python :: 3.9",
    "Programming Language :: Python :: 3.10",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
]

[tool.maturin]
python-source = "python"
module-name = "specado._native"
manifest-path = "crates/specado-py/Cargo.toml"
features = ["pyo3/extension-module"]
```

* **`python/specado/__init__.py`**

```python
from typing import Optional, Dict, Any, List
from dataclasses import dataclass
from ._native import Client as _NativeClient

__version__ = "0.1.0"

@dataclass
class Message:
    role: str
    content: str

@dataclass
class PromptSpec:
    messages: List[Message]
    sampling: Optional[Dict[str, float]] = None
    strict_mode: str = "Warn"
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            "version": "1",
            "messages": [{"role": m.role, "content": m.content} for m in self.messages],
            "sampling": self.sampling or {},
            "strict_mode": self.strict_mode,
        }

class Client:
    def __init__(self, provider_path: str):
        self._client = _NativeClient(provider_path)
    
    def complete(self, prompt: PromptSpec) -> Dict[str, Any]:
        return self._client.complete(prompt.to_dict())

__all__ = ["Client", "PromptSpec", "Message"]
```

* **`python/specado/compat/__init__.py`**

```python
from .openai import OpenAI

__all__ = ["OpenAI"]
```

* **`python/specado/compat/openai.py`**

```python
from typing import List, Dict, Any, Optional
from .. import Client as SpecadoClient, PromptSpec, Message as SpecadoMessage

class Message:
    def __init__(self, role: str, content: str):
        self.role = role
        self.content = content

class Choice:
    def __init__(self, message: Message, finish_reason: str):
        self.message = message
        self.finish_reason = finish_reason

class ChatCompletion:
    def __init__(self, choices: List[Choice]):
        self.choices = choices

class ChatCompletions:
    def __init__(self, client: SpecadoClient):
        self._client = client
    
    def create(
        self,
        model: str,
        messages: List[Dict[str, str]],
        temperature: Optional[float] = None,
        **kwargs
    ) -> ChatCompletion:
        spec = PromptSpec(
            messages=[SpecadoMessage(role=m["role"], content=m["content"]) for m in messages],
            sampling={"temperature": temperature} if temperature else None
        )
        
        response = self._client.complete(spec)
        
        message = Message(role="assistant", content=response["content"])
        choice = Choice(message=message, finish_reason=response["finish_reason"])
        
        return ChatCompletion(choices=[choice])

class Chat:
    def __init__(self, client: SpecadoClient):
        self.completions = ChatCompletions(client)

class OpenAI:
    def __init__(self, provider_path: str = "providers/openai/gpt-4.yaml"):
        self._client = SpecadoClient(provider_path)
        self.chat = Chat(self._client)
```

#### Dependencies

* Uses: #43

### 🧪 Testing Requirements

* Integration: Python OpenAI-compat smoke test.

### 🔗 References

* Epic: #3 - Developer Interfaces

### ⏱️ Estimates

* **Effort:** S • **Time:** 2–3h • **Complexity:** Low

## #45 — Node.js Bindings (napi-rs + packaging)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/45

### Body

### 📋 Task Overview

Expose `Client.complete(prompt)` to Node; package for npm with cross-target artifacts.

### 🎯 Acceptance Criteria

* [ ] `napi build --platform` succeeds.
* [ ] TS types available.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-node/Cargo.toml`**

```toml
[package]
name = "specado-node"
version.workspace = true
edition.workspace = true
license.workspace = true

[lib]
crate-type = ["cdylib"]

[dependencies]
specado-core.workspace = true
napi = { version = "2.16.11", features = ["async", "serde-json", "tokio_rt"] }
napi-derive = "2.16.8"
tokio.workspace = true
serde_json.workspace = true
```

* **`crates/specado-node/src/lib.rs`**

```rust
#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use specado_core::{PromptSpec, UniformResponse};

#[napi]
pub struct Client {
    provider_path: String,
}

#[napi]
impl Client {
    #[napi(constructor)]
    pub fn new(provider_path: String) -> Result<Self> {
        Ok(Self { provider_path })
    }

    #[napi]
    pub async fn complete(&self, prompt: serde_json::Value) -> Result<serde_json::Value> {
        let prompt_spec: PromptSpec = serde_json::from_value(prompt)
            .map_err(|e| Error::from_reason(format!("Invalid prompt: {}", e)))?;
        
        let response: UniformResponse = specado_core::execute(prompt_spec, &self.provider_path)
            .await
            .map_err(|e| Error::from_reason(format!("Execution failed: {}", e)))?;
        
        serde_json::to_value(&response)
            .map_err(|e| Error::from_reason(format!("Serialization failed: {}", e)))
    }
}
```

* **`crates/specado-node/package.json`**

```json
{
  "name": "specado",
  "version": "0.1.0",
  "main": "index.js",
  "types": "index.d.ts",
  "files": [
    "index.js",
    "index.d.ts",
    "npm/**"
  ],
  "napi": {
    "name": "specado",
    "triples": {
      "defaults": true,
      "additional": [
        "aarch64-apple-darwin",
        "aarch64-unknown-linux-gnu",
        "x86_64-unknown-linux-musl"
      ]
    }
  },
  "scripts": {
    "build": "napi build --platform --release",
    "build:debug": "napi build --platform",
    "prepublishOnly": "napi prepublish -t npm",
    "test": "node --test",
    "artifacts": "napi artifacts"
  },
  "dependencies": {
    "@napi-rs/cli": "^2.18.4"
  },
  "devDependencies": {
    "@types/node": "^22.10.2"
  },
  "engines": {
    "node": ">= 18"
  }
}
```

* **`crates/specado-node/index.js`**

```javascript
const { loadBinding } = require('@napi-rs/cli');
module.exports = loadBinding(__dirname, 'specado', 'specado');
```

* **`crates/specado-node/index.d.ts`**

```javascript
// Minimal type surface for consumers.
// CommonJS: const { Client } = require('specado')
// ESM: import { Client } from 'specado'

export class Client {
  constructor(providerPath: string);
  complete(prompt: unknown): Promise<unknown>;
}
```

#### Dependencies

* Uses: #22

### 🧪 Testing Requirements

* Integration: Node test runner smoke test.

### 🔗 References

* Epic: #3 - Developer Interfaces

### ⏱️ Estimates

* **Effort:** M • **Time:** 4–6h • **Complexity:** Medium

## #46 — Provider Catalog (OpenAI, Anthropic)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/46

### Body

### 📋 Task Overview

Provide initial provider YAMLs validating against schema and working with translation/normalize.

### 🎯 Acceptance Criteria

* [ ] OpenAI supports messages, temperature, top_p.
* [ ] Anthropic relocates system and clamps temperature.

### 📊 Technical Details

#### 📄 File(s)

* **`crates/specado-providers/providers/openai/gpt-4.yaml`**

```yaml
provider: openai
models:
  - id: gpt-4-turbo

auth:
  type: bearer
  token_env: OPENAI_API_KEY

endpoints:
  chat:
    method: POST
    url: https://api.openai.com/v1/chat/completions
    headers:
      Content-Type: application/json

mappings:
  request:
    - from: "$.messages"
      to: "$.messages"
    - from: "$.sampling.temperature"
      to: "$.temperature"
    - from: "$.sampling.top_p"
      to: "$.top_p"
  
  response:
    - from: "$.choices[0].message.content"
      to: "content"
    - from: "$.choices[0].finish_reason"
      to: "finish_reason"

constraints:
  supports:
    json_mode: true
    tools: true
```

* **`crates/specado-providers/providers/anthropic/claude-3-opus.yaml`**

```yaml
provider: anthropic
models:
  - id: claude-3-opus-20240229

auth:
  type: apikey
  header: x-api-key
  key_env: ANTHROPIC_API_KEY

endpoints:
  chat:
    method: POST
    url: https://api.anthropic.com/v1/messages
    headers:
      Content-Type: application/json
      anthropic-version: "2023-06-01"

mappings:
  request:
    - from: "$.messages[?(@.role=='system')].content"
      to: "$.system"
      code: Relocate
    - from: "$.messages[?(@.role!='system')]"
      to: "$.messages"
    - from: "$.sampling.temperature"
      to: "$.temperature"
      clamp: [0.0, 1.0]
      code: Clamp
  
  response:
    - from: "$.content[0].text"
      to: "content"
    - from: "$.stop_reason"
      to: "finish_reason"

constraints:
  supports:
    json_mode: false
    tools: true
```

#### Dependencies

* Uses: #20 schema, #31–#32

### 🧪 Testing Requirements

* Golden snapshots for both providers.

### 🔗 References

* Epic: #4 - Production Readiness

### ⏱️ Estimates

* **Effort:** S • **Time:** 2–3h • **Complexity:** Low

## #47 — CI/CD Workflow (GitHub Actions)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/47

### Body

### 📋 Task Overview

Set up multi-OS CI for Rust, Python, Node; cache Cargo; run tests, clippy, fmt.

### 🎯 Acceptance Criteria

* [ ] All jobs pass on main.
* [ ] Matrix covers OS and Python/Node versions.

### 📊 Technical Details

#### 📄 File(s)

* **`.github/workflows/ci.yml`**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  test-rust:
    name: Test Rust on ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      
      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Run tests
        run: cargo test --workspace --all-features
        env:
          OPENAI_API_KEY: test-key
          ANTHROPIC_API_KEY: test-key
      
      - name: Run clippy
        run: cargo clippy --workspace --all-targets -- -D warnings
      
      - name: Check formatting
        run: cargo fmt --all -- --check

  test-python:
    name: Test Python ${{ matrix.python }} on ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        python: ["3.9", "3.10", "3.11", "3.12"]
    steps:
      - uses: actions/checkout@v4
      
      - uses: actions/setup-python@v5
        with:
          python-version: ${{ matrix.python }}
      
      - name: Install maturin
        run: pip install maturin pytest
      
      - name: Build and test
        run: |
          maturin develop
          pytest python/tests/
        env:
          OPENAI_API_KEY: test-key
          ANTHROPIC_API_KEY: test-key

  test-node:
    name: Test Node ${{ matrix.node }} on ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        node: [18, 20, 22]
    steps:
      - uses: actions/checkout@v4
      
      - uses: actions/setup-node@v4
        with:
          node-version: ${{ matrix.node }}
      
      - name: Install dependencies
        working-directory: crates/specado-node
        run: npm install
      
      - name: Build
        working-directory: crates/specado-node
        run: npm run build
        env:
          OPENAI_API_KEY: test-key
          ANTHROPIC_API_KEY: test-key
      
      - name: Test
        working-directory: crates/specado-node
        run: npm test
```

#### Dependencies

* Uses: all buildable crates.

### 🧪 Testing Requirements

* CI is the test.

### 🔗 References

* Epic: #5 - Release Polish

### ⏱️ Estimates

* **Effort:** S • **Time:** 2–3h • **Complexity:** Low

## #48 — Production Feature: Hot-Reload (Design & Stub)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/48

### Body

### 📋 Task Overview

Add design and basic stubs for watching provider spec dirs and atomically swapping loaded configs.

### 🎯 Acceptance Criteria

* [ ] Design doc committed (under `/docs`).
* [ ] Stub feature flag in core and binding toggles.

### 📊 Technical Details

#### Implementation Approach

* Use a watcher (e.g., `notify`) in a background task keyed by provider path.
* Maintain an `Arc<RwLock<ProviderSpecCache>>`.

#### Dependencies

* Related to: #22 (execute), #44/#45 (bindings toggles)

#### API Design

```rust
// Sketch
pub struct ProviderCache { /* path -> ProviderSpec */ }
impl ProviderCache {
  pub fn enable_watch(path: &str) -> Result<()>;
  pub fn get(&self, path: &str) -> Arc<ProviderSpec>;
}
```

### ⚠️ Risks & Considerations

* Atomic swap to avoid race; validate before swap.

### 🧪 Testing Requirements

* Integration: change YAML on disk; subsequent call uses new config.

### 📚 Documentation Requirements

* [ ] `/docs/hot-reload.md`

### 🔗 References

* Epic: #4 - Production Readiness

### ⏱️ Estimates

* **Effort:** L • **Time:** 10–16h • **Complexity:** Medium-High

## #49 — Production Feature: Audit Logging (Design & Stub)

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/49

### Body

### 📋 Task Overview

Structured JSONL logs with redaction, correlation IDs, latency, outcome, lossiness summary.

### 🎯 Acceptance Criteria

* [ ] Design doc committed.
* [ ] Config for stdout/file path; redaction list defaulted.

### 📊 Technical Details

#### Implementation Approach

* Wrap `execute` with timing and correlation ID.
* Redact headers matching `Authorization|Token`.

#### Dependencies

* Uses: #22, #24

#### API Design

```rust
// Sketch
pub struct AuditConfig { pub target: AuditTarget, pub redact: Vec<Regex> }
pub fn with_audit(config: AuditConfig) -> ClientDecorator;
```

### ⚠️ Risks & Considerations

* PII handling; file rotation if needed (out of scope v1).

### 🧪 Testing Requirements

* Unit: redaction works.
* Integration: JSONL line contains expected fields.

### 📚 Documentation Requirements

* [ ] `/docs/audit-logging.md`

### 🔗 References

* Epic: #4 - Production Readiness

### ⏱️ Estimates

* **Effort:** M • **Time:** 6–10h • **Complexity:** Medium

## #50 — Docs, Tests, Benches, Examples Scaffolding

- **State:** open
- **Labels:** task
- **Assignees:** —
- **URL:** https://github.com/specado/specado/issues/50

### Body

### 📋 Task Overview

Create initial docs and test scaffolding directories.

### 🎯 Acceptance Criteria

* [ ] `/docs/QUICKSTART.md`, `/docs/MIGRATION.md` stubs.
* [ ] `tests/golden/` and `tests/integration/` exist.
* [ ] `examples/` exists.

### 📊 Technical Details

#### Implementation Approach

* Provide minimal examples and test placeholders.

#### Dependencies

* Related to: all functional issues.

#### API Design

```text
N/A
```

### ⚠️ Risks & Considerations

* Keep examples minimal but runnable.

### 🧪 Testing Requirements

* CI executes test placeholders (can be no-op initially).

### 📚 Documentation Requirements

* [ ] Author basic quickstart flow.

### 🔗 References

* Epic: #5 - Release Polish

### ⏱️ Estimates

* **Effort:** S • **Time:** 2–4h • **Complexity:** Low

