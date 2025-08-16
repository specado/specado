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
pub mod types;

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

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Placeholder translate function (to be implemented in issue #9)
pub fn translate(
    _prompt_spec: &PromptSpec,
    _provider_spec: &ProviderSpec,
    _model_id: &str,
    _mode: StrictMode,
) -> Result<TranslationResult> {
    Err(Error::Unsupported {
        message: "Translation not yet implemented (see issue #9)".to_string(),
        feature: Some("translate".to_string()),
    })
}

/// Placeholder run function (to be implemented in L2)
pub async fn run(_provider_request_json: &serde_json::Value) -> Result<UniformResponse> {
    Err(Error::Unsupported {
        message: "Run functionality not yet implemented (L2)".to_string(),
        feature: Some("run".to_string()),
    })
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