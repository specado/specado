use crate::auth::AuthError;
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
    Auth(#[from] AuthError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderErrorKind {
    RateLimit,
    Timeout,
    InvalidRequest,
    AuthenticationFailed,
    ServerError,
    Unknown,
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthError;

    #[test]
    fn wraps_io_error() {
        let err = std::io::Error::other("disk failed");
        let wrapped = Error::from(err);
        matches!(wrapped, Error::Io(_))
            .then_some(())
            .expect("io errors map to Error::Io");
    }

    #[test]
    fn wraps_json_error() {
        let err = serde_json::from_str::<serde_json::Value>("not-json").unwrap_err();
        let wrapped = Error::from(err);
        assert!(matches!(wrapped, Error::Json(_)));
    }

    #[test]
    fn auth_error_converts() {
        let auth_err = AuthError::MissingEnvVar("OPENAI_API_KEY".into());
        let wrapped = Error::from(auth_err);
        assert!(matches!(wrapped, Error::Auth(_)));
    }

    #[test]
    fn provider_error_kind_debug() {
        let kind = ProviderErrorKind::RateLimit;
        assert_eq!(format!("{:?}", kind), "RateLimit");
    }
}
