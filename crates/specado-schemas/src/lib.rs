//! Specado Schemas - JSON Schema definitions and validators
//!
//! This crate provides JSON Schema draft 2020-12 definitions and comprehensive
//! validation for:
//! - **PromptSpec**: Uniform request format for LLM provider interactions
//! - **ProviderSpec**: Provider capabilities and mapping configurations
//!
//! ## Features
//!
//! - **Schema Validation**: JSON Schema draft 2020-12 compliance checking
//! - **Custom Business Rules**: Domain-specific validation logic
//! - **Multiple Validation Modes**: Basic, Partial, and Strict validation
//! - **Batch Processing**: Efficient validation of multiple documents
//! - **Detailed Error Reporting**: Rich error context and violation details
//!
//! ## Quick Start
//!
//! ```rust
//! use specado_schemas::{create_prompt_spec_validator, SchemaValidator};
//! use serde_json::json;
//!
//! // Create a validator
//! let validator = create_prompt_spec_validator().unwrap();
//!
//! // Validate a PromptSpec
//! let spec = json!({
//!     "spec_version": "1.0",
//!     "id": "example-123",
//!     "model_class": "Chat",
//!     "messages": [
//!         {"role": "user", "content": "Hello, world!"}
//!     ]
//! });
//!
//! match validator.validate(&spec) {
//!     Ok(_) => println!("Valid PromptSpec!"),
//!     Err(e) => println!("Validation error: {}", e),
//! }
//! ```
//!
//! ## Validation Modes
//!
//! - **Basic**: JSON Schema validation only - fast for basic compliance
//! - **Partial**: Schema + selected custom rules - good for development
//! - **Strict**: Schema + all custom rules - recommended for production
//!
//! ## Custom Rules
//!
//! ### PromptSpec Rules
//! - `tool_choice` requires non-empty `tools` array
//! - `reasoning_tokens` only valid for "ReasoningChat" model class
//! - `rag` configuration only valid for "RAGChat" model class
//! - Media input constraints based on model class compatibility
//! - Conversation message reference validation
//! - Strict mode prevents unknown fields
//!
//! ### ProviderSpec Rules
//! - JSONPath expressions in mappings must be valid syntax
//! - Environment variable references must use `${ENV:VAR}` format
//! - Input modes must be compatible with model family constraints
//! - Response normalization paths must be valid JSONPath expressions
//! - Tooling configuration requires tool support capability
//! - RAG/conversation configs require corresponding capability flags
//! - Endpoint protocols must match base_url scheme
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

pub mod validation;

// Re-export commonly used types for convenience
pub use validation::{
    ValidationError, ValidationErrors, ValidationResult, ValidationMode,
    SchemaValidator, ValidationContext, ValidationHelpers,
    PromptSpecValidator, ProviderSpecValidator,
    create_prompt_spec_validator, create_provider_spec_validator,
    ValidationConfig, validate_prompt_specs_batch, validate_provider_specs_batch,
};