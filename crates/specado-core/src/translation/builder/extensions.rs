//! Extension methods for TranslationResult
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::{Severity, TranslationResult};
use super::TranslationResultBuilder;

/// Extension methods for TranslationResult
impl TranslationResult {
    /// Create a new TranslationResult using the builder
    pub fn builder() -> TranslationResultBuilder {
        TranslationResultBuilder::new()
    }

    /// Check if the translation has critical issues
    pub fn has_critical_issues(&self) -> bool {
        self.lossiness.max_severity == Severity::Critical
    }

    /// Check if the translation has errors
    pub fn has_errors(&self) -> bool {
        self.lossiness.max_severity >= Severity::Error
    }

    /// Check if the translation has warnings
    pub fn has_warnings(&self) -> bool {
        self.lossiness.max_severity >= Severity::Warning
    }

    /// Get the provider name from metadata
    pub fn provider_name(&self) -> Option<&str> {
        self.metadata.as_ref().map(|m| m.provider.as_str())
    }

    /// Get the model name from metadata
    pub fn model_name(&self) -> Option<&str> {
        self.metadata.as_ref().map(|m| m.model.as_str())
    }

    /// Get the translation duration in milliseconds
    pub fn duration_ms(&self) -> Option<u64> {
        self.metadata.as_ref().and_then(|m| m.duration_ms)
    }
}