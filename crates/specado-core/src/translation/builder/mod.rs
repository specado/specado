//! TranslationResult builder for constructing translation results
//!
//! This module implements the comprehensive TranslationResult builder for Issue #21.
//! Provides advanced builder features including validation, merging, incremental building,
//! and state tracking for the Specado translation engine.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

mod types;
mod core;
mod lossiness;
mod validation;
mod build;
mod request;
mod extensions;

// Re-export public types and builders
pub use types::{BuilderError, BuilderState};
pub use core::TranslationResultBuilder;
pub use request::ProviderRequestBuilder;

#[cfg(test)]
mod tests;