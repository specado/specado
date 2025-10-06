use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AuthScheme {
    Bearer { token_env: String },
    ApiKey { header: String, key_env: String },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AuthError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Unsupported auth scheme: {0}")]
    UnsupportedScheme(String),
    #[error("Failed to inject authentication headers: {0}")]
    HeaderInjection(String),
}
