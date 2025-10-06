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
                {"role": "user", "content": "hi"}
            ]
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
}
