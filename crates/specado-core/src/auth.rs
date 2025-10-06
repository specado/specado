use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AuthScheme {
    Bearer {
        token_env: String,
    },
    #[serde(rename = "apikey")]
    ApiKey {
        header: String,
        key_env: String,
    },
    Custom {
        headers: HashMap<String, String>,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AuthError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Invalid auth configuration: {0}")]
    InvalidConfig(String),
}

pub struct AuthHandler {
    scheme: AuthScheme,
}

impl AuthHandler {
    pub fn new(scheme: AuthScheme) -> Self {
        Self { scheme }
    }

    pub fn scheme(&self) -> &AuthScheme {
        &self.scheme
    }

    pub fn validate(&self) -> Result<(), AuthError> {
        match &self.scheme {
            AuthScheme::Bearer { token_env } => {
                Self::require_env(token_env)?;
            }
            AuthScheme::ApiKey { key_env, .. } => {
                Self::require_env(key_env)?;
            }
            AuthScheme::Custom { headers } => {
                for value in headers.values() {
                    Self::expand_env_var(value)?;
                }
            }
        }
        Ok(())
    }

    pub fn inject_headers(&self, headers: &mut HashMap<String, String>) -> Result<(), AuthError> {
        match &self.scheme {
            AuthScheme::Bearer { token_env } => {
                let token = Self::require_env(token_env)?;
                headers.insert("Authorization".to_string(), format!("Bearer {}", token));
            }
            AuthScheme::ApiKey { header, key_env } => {
                let key = Self::require_env(key_env)?;
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

    fn require_env(var: &str) -> Result<String, AuthError> {
        std::env::var(var).map_err(|_| AuthError::MissingEnvVar(var.to_string()))
    }

    fn expand_env_var(template: &str) -> Result<String, AuthError> {
        if let Some(stripped) = template
            .strip_prefix("${ENV:")
            .and_then(|s| s.strip_suffix('}'))
        {
            Self::require_env(stripped)
        } else if template.contains("${ENV:") {
            Err(AuthError::InvalidConfig(template.to_string()))
        } else {
            Ok(template.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unset(name: &str) {
        std::env::remove_var(name);
    }

    #[test]
    fn bearer_missing_env_returns_error() {
        let var = "SPECADO_TEST_TOKEN";
        unset(var);
        let handler = AuthHandler::new(AuthScheme::Bearer {
            token_env: var.to_string(),
        });
        let mut headers = HashMap::new();
        let err = handler.inject_headers(&mut headers).unwrap_err();
        assert!(matches!(err, AuthError::MissingEnvVar(m) if m == var));
    }

    #[test]
    fn injects_bearer_header() {
        let var = "SPECADO_TEST_TOKEN_OK";
        std::env::set_var(var, "token-value");
        let handler = AuthHandler::new(AuthScheme::Bearer {
            token_env: var.to_string(),
        });
        let mut headers = HashMap::new();
        handler.inject_headers(&mut headers).unwrap();
        assert_eq!(headers.get("Authorization").unwrap(), "Bearer token-value");
        unset(var);
    }

    #[test]
    fn custom_env_expansion() {
        let key = "SPECADO_CUSTOM_KEY";
        std::env::set_var(key, "123");
        let handler = AuthHandler::new(AuthScheme::Custom {
            headers: HashMap::from([(
                "X-Api-Key".to_string(),
                "${ENV:SPECADO_CUSTOM_KEY}".to_string(),
            )]),
        });
        let mut headers = HashMap::new();
        handler.inject_headers(&mut headers).unwrap();
        assert_eq!(headers["X-Api-Key"], "123");
        unset(key);
    }

    #[test]
    fn validate_checks_custom_placeholders() {
        let handler = AuthHandler::new(AuthScheme::Custom {
            headers: HashMap::from([("X-Thing".to_string(), "${ENV:SPECADO_MISSING}".to_string())]),
        });
        let err = handler.validate().unwrap_err();
        assert!(matches!(err, AuthError::MissingEnvVar(var) if var == "SPECADO_MISSING"));
    }

    #[test]
    fn invalid_placeholder_returns_invalid_config() {
        let handler = AuthHandler::new(AuthScheme::Custom {
            headers: HashMap::from([("X-Thing".to_string(), "${ENV:UNTERMINATED".to_string())]),
        });
        let err = handler.validate().unwrap_err();
        assert!(matches!(err, AuthError::InvalidConfig(_)));
    }
}
