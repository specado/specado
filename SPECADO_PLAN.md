# Specado v1.0 Implementation Specification

**Status:** Production-Ready  
**Date:** January 2025  
**Rust Version:** 1.75+

## Package Versions

### Rust Dependencies

```toml
tokio = "1.40"
serde = { version = "1.0.214", features = ["derive"] }
serde_json = "1.0.132"
serde_yaml = "0.9.34"
reqwest = { version = "0.12.9", features = ["json", "rustls-tls"] }
clap = { version = "4.5.23", features = ["derive"] }
thiserror = "2.0.3"
async-trait = "0.1.83"
tracing = "0.1.41"
jsonschema = "0.26.1"
serde_jsonpath = "0.3.3"
once_cell = "1.19.0"
```

### Python Bindings

```toml
pyo3 = { version = "0.22.6", features = ["extension-module", "abi3-py39"] }
pythonize = "0.22.0"
```

### Node.js Bindings

```toml
napi = { version = "2.16.11", features = ["async", "serde-json", "tokio_rt"] }
napi-derive = "2.16.8"
```

## Repository Structure

```
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
```

## Workspace Root

**File:** `Cargo.toml`

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

## Crate: specado-schemas

**File:** `crates/specado-schemas/Cargo.toml`

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

**File:** `crates/specado-schemas/schemas/prompt-spec.v1.schema.json`

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

**File:** `crates/specado-schemas/schemas/provider-spec.v1.schema.json`

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

**File:** `crates/specado-schemas/src/lib.rs`

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

## Crate: specado-core

**File:** `crates/specado-core/Cargo.toml`

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

**File:** `crates/specado-core/src/lib.rs`

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

**File:** `crates/specado-core/src/error.rs`

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

**File:** `crates/specado-core/src/auth.rs`

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

**File:** `crates/specado-core/src/types/mod.rs`

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

**File:** `crates/specado-core/src/types/prompt.rs`

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

**File:** `crates/specado-core/src/types/provider.rs`

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

**File:** `crates/specado-core/src/types/lossiness.rs`

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

**File:** `crates/specado-core/src/types/response.rs`

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

**File:** `crates/specado-core/src/transformer/mod.rs`

```rust
pub mod translate;
pub mod normalize;
pub mod detect;

pub use translate::translate;
pub use normalize::normalize;
```

**File:** `crates/specado-core/src/transformer/translate.rs`

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

**File:** `crates/specado-core/src/transformer/normalize.rs`

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

**File:** `crates/specado-core/src/transformer/detect/mod.rs`

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

**File:** `crates/specado-core/src/transformer/detect/clamp.rs`

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

**File:** `crates/specado-core/src/transformer/detect/relocate.rs`

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

**File:** `crates/specado-core/src/transformer/detect/unsupported.rs`

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

**File:** `crates/specado-core/src/transformer/detect/drop.rs`

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

**File:** `crates/specado-core/src/circuit_breaker.rs`

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

**File:** `crates/specado-core/src/retry.rs`

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

**File:** `crates/specado-core/src/router/mod.rs`

```rust
pub mod traits;
pub mod primary_fallback;

pub use traits::Router;
pub use primary_fallback::PrimaryFallbackRouter;
```

**File:** `crates/specado-core/src/router/traits.rs`

```rust
use async_trait::async_trait;
use crate::types::{PromptSpec, UniformResponse};
use crate::error::Result;

#[async_trait]
pub trait Router: Send + Sync {
    async fn route(&self, prompt: PromptSpec) -> Result<UniformResponse>;
}
```

**File:** `crates/specado-core/src/router/primary_fallback.rs`

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

**File:** `crates/specado-core/src/http/client.rs`

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

## Crate: specado-cli

**File:** `crates/specado-cli/Cargo.toml`

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

**File:** `crates/specado-cli/src/main.rs`

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

## Crate: specado-py

**File:** `crates/specado-py/Cargo.toml`

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

**File:** `crates/specado-py/src/lib.rs`

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

**File:** `pyproject.toml`

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

**File:** `python/specado/__init__.py`

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

**File:** `python/specado/compat/__init__.py`

```python
from .openai import OpenAI

__all__ = ["OpenAI"]
```

**File:** `python/specado/compat/openai.py`

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

## Crate: specado-node

**File:** `crates/specado-node/Cargo.toml`

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

**File:** `crates/specado-node/src/lib.rs`

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

**File:** `crates/specado-node/package.json`

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

**File:** `crates/specado-node/index.js`

```javascript
const { loadBinding } = require('@napi-rs/cli');
module.exports = loadBinding(__dirname, 'specado', 'specado');
```

**File:** `crates/specado-node/index.d.ts`

```javascript
// Minimal type surface for consumers.
// CommonJS: const { Client } = require('specado')
// ESM: import { Client } from 'specado'

export class Client {
  constructor(providerPath: string);
  complete(prompt: unknown): Promise<unknown>;
}
```

## Provider Specs

**File:** `crates/specado-providers/providers/openai/gpt-4.yaml`

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

**File:** `crates/specado-providers/providers/anthropic/claude-3-opus.yaml`

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

## CI/CD

**File:** `.github/workflows/ci.yml`

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

## Feature Scope Definitions (v1)

### **Hot-Reload Scope (v1)**

  - **Functionality:** The library will monitor the directory containing provider specs for filesystem changes.
  - **Scope:** This applies only to **`ProviderSpec`** YAML files. Changes to schemas or other configurations are not covered in v1.
  - **Behavior:** Upon detecting a change to a loaded provider spec, the library will atomically swap the old configuration for the new one in memory. Subsequent requests will use the new spec.
  - **Interface:** This will be an opt-in feature enabled during `Client` initialization in the language bindings. It will not be exposed via the CLI.

### **Audit Logging Scope (v1)**

  - **Goal:** To provide a machine-readable, structured log of all executed requests for observability.
  - **Format:** Newline-delimited JSON (**JSONL**).
  - **Target:** Can be configured to output to **`stdout`** or a specified **file path**.
  - **Core Fields:** Each log entry will contain a `timestamp`, a unique `correlation_id`, the `provider_used`, a summary of the `lossiness_report` (e.g., `is_lossy: true`, `codes: ["Relocate", "Clamp"]`), latency, and the outcome (`success` or `error`).
  - **Redaction:** API keys and any fields in headers matching "Authorization" or "Token" will be redacted by default.

## Expanded Testing Strategy

### **Resilience & Negative-Path Testing**

Before the language bindings are finalized, the core library will be hardened with tests covering the following failure modes:

  - **Configuration Errors:** Test behavior with malformed `ProviderSpec` YAML/JSON, missing spec files, and invalid file paths.
  - **Authentication Errors:** Verify that the `AuthHandler` produces clear `MissingEnvVar` errors when required API keys are not set in the environment.
  - **API Errors:** Use a mock HTTP server (e.g., `wiremock`) to simulate provider API failures, including:
      - `4xx` client errors (e.g., invalid requests).
      - `5xx` server errors.
      - Rate-limit responses (`429 Too Many Requests`).
      - Network timeouts.
  - **Validation Errors:** Test library behavior when provided with an invalid `PromptSpec` that fails schema validation.

### **Lossiness Report Testing**

The "Lossiness Report" feature will be validated with a dedicated test suite to prevent regressions:

  - **Unit Tests:** Each individual lossiness code (`Clamp`, `Drop`, `Relocate`, etc.) will have a dedicated unit test. Each test will use a minimal `PromptSpec` and `ProviderSpec` designed specifically to trigger that code, asserting that the correct `LossinessEntry` is generated.
  - **Golden Snapshot Tests:** For each supported provider (OpenAI, Anthropic), a canonical `PromptSpec` will be translated. The resulting provider payload **and** the full `LossinessReport` will be saved as snapshot files. CI will fail if these snapshots change unexpectedly, immediately flagging any regressions in the translation or lossiness logic.

## Implementation Timeline

### **Week 1: Foundation**

  - Set up workspace structure.
  - Implement JSON Schema validation.
  - Define all core types (`PromptSpec`, `ProviderSpec`, etc.).
  - Implement the authentication handler (`auth.rs`).
  - **Exit Criteria:**
      - ✅ `cargo test -p specado-schemas` passes.
      - ✅ `cargo test -p specado-core` (for types and auth handler) passes.

### **Week 2-3: Core Features & API Freeze**

  - Complete lossiness detection logic for all 8 codes.
  - Implement a robust HTTP client using the shared `once_cell` instance.
  - Implement the response normalization logic.
  - Fully implement `specado-cli` with `validate`, `preview`, and `run` commands.
  - Write initial golden snapshot tests for lossiness reports.
  - **Exit Criteria:**
      - ✅ `cargo test --workspace` passes, including new lossiness unit tests.
      - ✅ `specado preview` successfully generates a lossiness report with at least 3 distinct codes.
      - ✅ **Core API Freeze:** Public functions (`execute`, `translate`) and data structures in `specado-core` are stabilized. A `CHANGELOG.md` is initiated. **Bindings work can now begin against a stable target.**

### **Week 4-5: Bindings & Resilience**

  - Implement Python bindings (`PyO3`) against the frozen core API.
  - Implement Node.js bindings (`napi-rs`) against the frozen core API.
  - Implement integration tests for both Python and Node.js bindings.
  - Implement resilience and negative-path tests for the core library (see Expanded Testing Strategy).
  - **Exit Criteria:**
      - ✅ The core library successfully passes all new resilience tests (e.g., handles missing env vars gracefully).
      - ✅ `maturin develop && pytest python/tests/` passes.
      - ✅ `cd crates/specado-node && npm run build && npm test` passes.

### **Week 6-7: Production Features**

  - Implement **Hot-Reload** functionality as defined in "Feature Scope Definitions."
  - Implement **Audit Logging** as defined in "Feature Scope Definitions."
  - Implement performance benchmarks and integrate them into the CI pipeline.
  - Draft initial user documentation (`QUICKSTART.md`, `MIGRATION.md`).
  - **Exit Criteria:**
      - ✅ An integration test proves a running application reloads a changed `ProviderSpec` without restarting.
      - ✅ Audit logs are successfully written to a file in JSONL format, capturing a `run` command with a correlation ID and redacted secrets (verified by a unit test).
      - ✅ Performance benchmarks (`cargo bench`) run successfully in CI and meet the defined budget (p50 ≤ 10ms).

### **Week 8: Release Polish**

  - Finalize packaging for PyPI and npm.
  - Create and validate complete example applications.
  - Finalize all documentation and perform a "fresh install" test of the library.
  - Finalize the CI/CD workflow for automated publishing.
  - **Exit Criteria:**
      - ✅ A packaging **"dry-run"** succeeds: the Python package installs from a local wheel, and the Node.js package installs from a local tarball.
      - ✅ The `QUICKSTART.md` guide is validated by following it from a clean environment, with the process taking under 10 minutes.
      - ✅ The full CI/CD pipeline, including `clippy`, `fmt`, and all language tests, passes cleanly on the `main` branch.