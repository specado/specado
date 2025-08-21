//! Field transformation system for translating values between uniform and provider formats
//!
//! This module provides a comprehensive field transformation system that uses the
//! JSONPath engine to locate fields and apply transformations such as type conversions,
//! enum mappings, unit conversions, and conditional transformations.
//!
//! # Module Organization
//!
//! The transformation system is organized into the following modules:
//!
//! - [`types`] - Core types, enums, and error definitions
//! - [`pipeline`] - Main transformation pipeline implementation
//! - [`builder`] - Fluent builder API for creating transformation rules
//! - [`built_in`] - Pre-configured transformations for common use cases
//! - [`tests`] - Comprehensive test suite
//!
//! # Examples
//!
//! ## Basic Type Conversion
//!
//! ```
//! use specado_core::{TransformationPipeline, TransformationRuleBuilder, TransformationDirection};
//! use specado_core::translation::transformer::built_in;
//! use serde_json::json;
//!
//! let mut pipeline = TransformationPipeline::new();
//!
//! // Convert string temperature to number
//! let temp_rule = TransformationRuleBuilder::new("temp_convert", "$.temperature")
//!     .transformation(built_in::string_to_number())
//!     .build()
//!     .unwrap();
//!
//! pipeline = pipeline.add_rule(temp_rule);
//!
//! let input = json!({
//!     "temperature": "0.7"
//! });
//!
//! // Note: This example requires a TranslationContext for the transform call
//! // See the tests for complete usage examples
//! ```
//!
//! ## Model Mapping
//!
//! ```
//! use specado_core::translation::transformer::built_in;
//! use serde_json::json;
//!
//! // Map OpenAI model names to Anthropic equivalents
//! let model_transform = built_in::openai_to_anthropic_models();
//! ```
//!
//! ## Temperature Scaling
//!
//! ```
//! use specado_core::translation::transformer::built_in;
//!
//! // Convert temperature from 0-2 range (OpenAI) to 0-1 range (Anthropic)
//! let temp_transform = built_in::temperature_0_2_to_0_1();
//! ```
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

// Core types and error definitions
pub mod types;

// Main transformation pipeline
pub mod pipeline;

// Rule builder API
pub mod builder;

// Pre-configured transformations
pub mod built_in;

// Test module
#[cfg(test)]
mod tests;

// Re-export main public types and functions for convenience
pub use types::{
    TransformationError, TransformationType, ValueType, ConversionFormula,
    Condition, TransformerFunction, TransformationContext, 
    TransformationDirection, TransformationRule
};

pub use pipeline::TransformationPipeline;
pub use builder::TransformationRuleBuilder;

// Re-export built-in transformers - no need to re-export the module itself
// Users can access as transformer::built_in::function_name()