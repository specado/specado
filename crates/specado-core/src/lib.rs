//! Specado Core - Translation engine for spec-driven LLM prompt compilation
//!
//! This crate provides the core functionality for translating uniform prompts
//! to provider-specific requests and normalizing responses back to a uniform format.
//!
//! # Main Components
//!
//! - **Error Handling**: Comprehensive error types using `thiserror` and `anyhow`
//! - **Core Types**: Data structures for prompts, providers, and translation results
//! - **Translation Engine**: Convert uniform prompts to provider-specific formats
//! - **Response Normalization**: Convert provider responses to uniform format
//!
//! # Example
//!
//! ```no_run
//! use specado_core::{Result, PromptSpec, ProviderSpec, StrictMode};
//!
//! fn example() -> Result<()> {
//!     // Translation functionality will be implemented in future issues
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod http;
pub mod llm;
pub mod provider_discovery;
pub mod response;
pub mod specs;
pub mod translation;
pub mod types;

#[cfg(test)]
pub mod proptest_strategies;

// Re-export main types for convenience
pub use error::{Error, Result, Severity, StrictMode};
pub use types::{
    // Core specifications
    PromptSpec, ProviderSpec,
    
    // Message types
    Message, MessageRole,
    
    // Tool types
    Tool, ToolChoice, ToolCall,
    
    // Configuration types
    ResponseFormat, SamplingParams, Limits, MediaConfig,
    
    // Provider types
    ProviderInfo, ModelSpec, Endpoints, EndpointConfig,
    InputModes, ToolingConfig, JsonOutputConfig,
    Constraints, ConstraintLimits, Mappings,
    ResponseNormalization, SyncNormalization, StreamNormalization,
    EventSelector, EventRoute,
    
    // Translation types
    TranslationResult, LossinessReport, LossinessItem, LossinessSummary,
    TranslationMetadata,
    
    // Response types
    UniformResponse, FinishReason,
    
    // Stream types
    StreamHandle,
};

// Re-export error enums
pub use error::LossinessCode;

// Re-export high-level LLM interface
pub use llm::{LLM, GenerationMode};

// Re-export response extensions
pub use response::{ResponseExt, TokenUsage, ToolCallInfo};

// Re-export specs types
pub use specs::{Capabilities, CapabilityDetector};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Main translation function that converts a PromptSpec to provider-specific format
///
/// This function is the primary public API for the translation engine. It takes
/// a validated `PromptSpec` and converts it to a provider-specific JSON format
/// based on the `ProviderSpec` configuration.
///
/// See `translation::translate` for full documentation.
pub use translation::{
    translate, StrictnessAction, StrictnessPolicy, PolicyResult,
    TransformationPipeline, TransformationRule, TransformationRuleBuilder,
    TransformationType, TransformationDirection, TransformationError,
    ValueType, ConversionFormula, Condition,
};

/// Execute a provider request and return a normalized response
///
/// This function sends the compiled provider request to the provider's API
/// and normalizes the response to the UniformResponse format.
///
/// # Arguments
/// * `provider_request` - The compiled provider request containing:
///   - `provider_spec`: The provider specification
///   - `model_id`: The model to use
///   - `request_body`: The request payload
///
/// # Returns
/// A normalized UniformResponse or an error
pub async fn run(provider_request: &serde_json::Value) -> Result<UniformResponse> {
    // Extract components from the provider request
    let provider_spec_json = provider_request.get("provider_spec")
        .ok_or_else(|| Error::Validation {
            field: "provider_spec".to_string(),
            message: "Missing provider_spec in request".to_string(),
            expected: Some("ProviderSpec object".to_string()),
        })?;
    
    let model_id = provider_request.get("model_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::Validation {
            field: "model_id".to_string(),
            message: "Missing or invalid model_id".to_string(),
            expected: Some("String model ID".to_string()),
        })?;
    
    let request_body = provider_request.get("request_body")
        .ok_or_else(|| Error::Validation {
            field: "request_body".to_string(),
            message: "Missing request_body".to_string(),
            expected: Some("Request payload object".to_string()),
        })?;
    
    // Parse the provider spec
    let provider_spec: ProviderSpec = serde_json::from_value(provider_spec_json.clone())
        .map_err(|e| Error::Json {
            message: format!("Failed to parse provider_spec: {}", e),
            source: e,
        })?;
    
    // Create HTTP client
    let client = http::HttpClient::with_default_config(provider_spec)?;
    
    // Get the model spec
    let model = client.get_model(model_id)
        .ok_or_else(|| Error::Provider {
            provider: client.provider_spec().provider.name.clone(),
            message: format!("Model '{}' not found", model_id),
            source: None,
        })?;
    
    // Execute the request
    let response = client.execute_chat_completion(model, request_body.clone()).await?;
    
    // Normalize the response using the model's response_normalization config
    let normalized = http::normalize_response(&response, model, model_id)?;
    
    Ok(normalized)
}

/// Placeholder stream function (to be implemented in L3)
pub async fn stream(_provider_request_json: &serde_json::Value) -> Result<StreamHandle> {
    Err(Error::Unsupported {
        message: "Stream functionality not yet implemented (L3)".to_string(),
        feature: Some("stream".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_error_creation() {
        let err = Error::SchemaValidation {
            message: "Test error".to_string(),
            source: None,
        };
        assert!(err.to_string().contains("Test error"));
    }

    #[test]
    fn test_strict_mode_equality() {
        assert_eq!(StrictMode::Strict, StrictMode::Strict);
        assert_ne!(StrictMode::Strict, StrictMode::Warn);
    }
}