use jsonschema::{validator_for, Validator};
use once_cell::sync::Lazy;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Schema compilation failed: {0}")]
    Compilation(String),
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("JSON parsing failed: {0}")]
    JsonParse(#[from] serde_json::Error),
}

pub struct SchemaValidator {
    prompt_schema: Validator,
    provider_schema: Validator,
}

static VALIDATOR: Lazy<SchemaValidator> =
    Lazy::new(|| SchemaValidator::new().expect("Failed to compile schemas"));

/// Get the singleton schema validator compiled at startup.
pub fn get_validator() -> &'static SchemaValidator {
    &VALIDATOR
}

impl SchemaValidator {
    fn new() -> Result<Self, ValidationError> {
        let prompt_schema_json = include_str!("../schemas/prompt-spec.v1.schema.json");
        let provider_schema_json = include_str!("../schemas/provider-spec.v1.schema.json");

        let prompt_v: Value = serde_json::from_str(prompt_schema_json)?;
        let provider_v: Value = serde_json::from_str(provider_schema_json)?;

        let prompt_schema =
            validator_for(&prompt_v).map_err(|e| ValidationError::Compilation(e.to_string()))?;
        let provider_schema =
            validator_for(&provider_v).map_err(|e| ValidationError::Compilation(e.to_string()))?;

        Ok(Self {
            prompt_schema,
            provider_schema,
        })
    }

    pub fn validate_prompt(&self, prompt: &Value) -> Result<(), ValidationError> {
        match self.prompt_schema.validate(prompt) {
            Ok(()) => Ok(()),
            Err(_) => {
                let joined = self
                    .prompt_schema
                    .iter_errors(prompt)
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                Err(ValidationError::Validation(joined))
            }
        }
    }

    pub fn validate_provider(&self, provider: &Value) -> Result<(), ValidationError> {
        match self.provider_schema.validate(provider) {
            Ok(()) => Ok(()),
            Err(_) => {
                let joined = self
                    .provider_schema
                    .iter_errors(provider)
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                Err(ValidationError::Validation(joined))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validates_prompt_schema() {
        let validator = get_validator();
        let prompt = json!({
            "version": "1",
            "messages": [
                {"role": "system", "content": "You are helpful."},
                {"role": "user", "content": "Summarize this article."}
            ],
            "sampling": {
                "temperature": 0.7,
                "top_p": 0.9,
                "top_k": 40
            },
            "response": {
                "format": "json_schema",
                "json_schema": {
                    "name": "summary",
                    "schema": {"type": "object"}
                }
            },
            "tools": [
                {
                    "name": "fetch_url",
                    "json_schema": {"type": "object"}
                }
            ],
            "tool_choice": "auto",
            "strict_mode": "Warn"
        });
        assert!(validator.validate_prompt(&prompt).is_ok());
    }

    #[test]
    fn invalid_prompt_reports_errors() {
        let validator = get_validator();
        let prompt = json!({
            "version": "1",
            "messages": []
        });

        let err = validator.validate_prompt(&prompt).unwrap_err();
        assert!(matches!(err, ValidationError::Validation(_)));
    }

    #[test]
    fn prompt_schema_defaults_response_to_text() {
        let schema_json: Value =
            serde_json::from_str(include_str!("../schemas/prompt-spec.v1.schema.json"))
                .expect("schema should parse");

        let default_format = schema_json
            .pointer("/properties/response/default/format")
            .and_then(Value::as_str);

        assert_eq!(default_format, Some("text"));
    }

    #[test]
    fn validates_provider_schema() {
        let validator = get_validator();
        let provider = json!({
            "provider": "openai",
            "models": [
                {"id": "gpt-4o"}
            ],
            "auth": {
                "type": "bearer",
                "token_env": "OPENAI_API_KEY"
            },
            "endpoints": {
                "chat": {
                    "method": "POST",
                    "url": "https://api.openai.com/v1/chat/completions",
                    "headers": {
                        "content-type": "application/json"
                    }
                }
            },
            "mappings": {
                "request": [
                    {"from": "prompt.messages", "to": "body.messages"}
                ],
                "response": [
                    {"from": "body.choices[0].message", "to": "prompt.response"}
                ]
            },
            "constraints": {
                "supports": {
                    "json_mode": true,
                    "tools": true
                }
            }
        });

        assert!(validator.validate_provider(&provider).is_ok());
    }

    #[test]
    fn invalid_provider_requires_auth_fields() {
        let validator = get_validator();
        let provider = json!({
            "provider": "openai",
            "models": [
                {"id": "gpt-4o"}
            ],
            "auth": {
                "type": "bearer"
            },
            "endpoints": {
                "chat": {
                    "method": "POST",
                    "url": "https://api.openai.com/v1/chat/completions",
                    "headers": {}
                }
            },
            "mappings": {
                "request": [],
                "response": []
            },
            "constraints": {
                "supports": {
                    "json_mode": true,
                    "tools": false
                }
            }
        });

        let err = validator.validate_provider(&provider).unwrap_err();
        assert!(matches!(err, ValidationError::Validation(_)));
    }
}
