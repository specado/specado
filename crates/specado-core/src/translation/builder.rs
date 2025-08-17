//! TranslationResult builder for constructing translation results
//!
//! This module will implement the comprehensive TranslationResult builder in issue #21.
//! Currently provides a minimal placeholder implementation.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::{
    LossinessReport, LossinessSummary, Severity, TranslationMetadata, TranslationResult,
};
use serde_json::Value;
use std::collections::HashMap;

/// Builder for constructing TranslationResult instances
///
/// The TranslationResultBuilder provides a fluent API for constructing
/// translation results with all necessary components including the
/// provider request JSON, lossiness report, and metadata.
pub struct TranslationResultBuilder {
    provider_request_json: Option<Value>,
    lossiness_report: Option<LossinessReport>,
    metadata: Option<TranslationMetadata>,
}

impl TranslationResultBuilder {
    /// Create a new TranslationResult builder
    pub fn new() -> Self {
        Self {
            provider_request_json: None,
            lossiness_report: None,
            metadata: None,
        }
    }

    /// Set the provider request JSON
    pub fn with_provider_request(mut self, request: Value) -> Self {
        self.provider_request_json = Some(request);
        self
    }

    /// Set the lossiness report
    pub fn with_lossiness_report(mut self, report: LossinessReport) -> Self {
        self.lossiness_report = Some(report);
        self
    }

    /// Set the translation metadata
    pub fn with_metadata(mut self, metadata: TranslationMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Build the TranslationResult
    ///
    /// This method constructs the final TranslationResult. If no lossiness
    /// report is provided, it creates an empty one. The provider request JSON
    /// must be provided.
    pub fn build(self) -> TranslationResult {
        let provider_request_json = self
            .provider_request_json
            .unwrap_or_else(|| serde_json::json!({}));

        let lossiness = self.lossiness_report.unwrap_or_else(|| {
            // Create an empty lossiness report
            LossinessReport {
                items: Vec::new(),
                max_severity: Severity::Info,
                summary: LossinessSummary {
                    total_items: 0,
                    by_severity: HashMap::new(),
                    by_code: HashMap::new(),
                },
            }
        });

        TranslationResult {
            provider_request_json,
            lossiness,
            metadata: self.metadata,
        }
    }

    /// Create a successful translation result with no lossiness
    pub fn success(request: Value) -> TranslationResult {
        Self::new()
            .with_provider_request(request)
            .with_lossiness_report(LossinessReport {
                items: Vec::new(),
                max_severity: Severity::Info,
                summary: LossinessSummary {
                    total_items: 0,
                    by_severity: HashMap::new(),
                    by_code: HashMap::new(),
                },
            })
            .build()
    }

    /// Check if the builder has all required fields
    pub fn is_complete(&self) -> bool {
        self.provider_request_json.is_some()
    }

    /// Get a reference to the provider request JSON if set
    pub fn provider_request(&self) -> Option<&Value> {
        self.provider_request_json.as_ref()
    }

    /// Get a reference to the lossiness report if set
    pub fn lossiness(&self) -> Option<&LossinessReport> {
        self.lossiness_report.as_ref()
    }

    /// Get a reference to the metadata if set
    pub fn metadata(&self) -> Option<&TranslationMetadata> {
        self.metadata.as_ref()
    }
}

impl Default for TranslationResultBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension methods for TranslationResult
impl TranslationResult {
    /// Create a new TranslationResult using the builder
    pub fn builder() -> TranslationResultBuilder {
        TranslationResultBuilder::new()
    }

    /// Check if the translation has any lossiness
    pub fn has_lossiness(&self) -> bool {
        self.lossiness.items.len() > 0
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LossinessCode, LossinessItem, StrictMode};

    #[test]
    fn test_builder_basic() {
        let request = serde_json::json!({
            "model": "test-model",
            "messages": []
        });

        let result = TranslationResultBuilder::new()
            .with_provider_request(request.clone())
            .build();

        assert_eq!(result.provider_request_json, request);
        assert!(!result.has_lossiness());
        assert!(result.metadata.is_none());
    }

    #[test]
    fn test_builder_with_metadata() {
        let metadata = TranslationMetadata {
            provider: "test-provider".to_string(),
            model: "test-model".to_string(),
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            duration_ms: Some(100),
            strict_mode: StrictMode::Warn,
        };

        let result = TranslationResultBuilder::new()
            .with_provider_request(serde_json::json!({}))
            .with_metadata(metadata.clone())
            .build();

        assert!(result.metadata.is_some());
        assert_eq!(result.provider_name(), Some("test-provider"));
        assert_eq!(result.model_name(), Some("test-model"));
        assert_eq!(result.duration_ms(), Some(100));
    }

    #[test]
    fn test_builder_with_lossiness() {
        let lossiness = LossinessReport {
            items: vec![LossinessItem {
                code: LossinessCode::Drop,
                path: "test_field".to_string(),
                message: "Field dropped".to_string(),
                severity: Severity::Warning,
                before: Some(serde_json::json!("value")),
                after: None,
            }],
            max_severity: Severity::Warning,
            summary: LossinessSummary {
                total_items: 1,
                by_severity: {
                    let mut map = HashMap::new();
                    map.insert("warning".to_string(), 1);
                    map
                },
                by_code: {
                    let mut map = HashMap::new();
                    map.insert("Drop".to_string(), 1);
                    map
                },
            },
        };

        let result = TranslationResultBuilder::new()
            .with_provider_request(serde_json::json!({}))
            .with_lossiness_report(lossiness)
            .build();

        assert!(result.has_lossiness());
        assert!(result.has_warnings());
        assert!(!result.has_errors());
        assert!(!result.has_critical_issues());
    }

    #[test]
    fn test_success_helper() {
        let request = serde_json::json!({
            "model": "test-model",
            "messages": []
        });

        let result = TranslationResultBuilder::success(request.clone());

        assert_eq!(result.provider_request_json, request);
        assert!(!result.has_lossiness());
        assert_eq!(result.lossiness.max_severity, Severity::Info);
    }

    #[test]
    fn test_builder_completeness() {
        let builder = TranslationResultBuilder::new();
        assert!(!builder.is_complete());

        let builder = builder.with_provider_request(serde_json::json!({}));
        assert!(builder.is_complete());
    }

    #[test]
    fn test_builder_getters() {
        let request = serde_json::json!({"test": "value"});
        let metadata = TranslationMetadata {
            provider: "test".to_string(),
            model: "model".to_string(),
            timestamp: "now".to_string(),
            duration_ms: None,
            strict_mode: StrictMode::Strict,
        };

        let builder = TranslationResultBuilder::new()
            .with_provider_request(request.clone())
            .with_metadata(metadata.clone());

        assert_eq!(builder.provider_request(), Some(&request));
        assert_eq!(builder.metadata(), Some(&metadata));
        assert_eq!(builder.lossiness(), None);
    }

    #[test]
    fn test_result_severity_checks() {
        // Test critical severity
        let mut result = TranslationResultBuilder::success(serde_json::json!({}));
        result.lossiness.max_severity = Severity::Critical;
        assert!(result.has_critical_issues());
        assert!(result.has_errors());
        assert!(result.has_warnings());

        // Test error severity
        result.lossiness.max_severity = Severity::Error;
        assert!(!result.has_critical_issues());
        assert!(result.has_errors());
        assert!(result.has_warnings());

        // Test warning severity
        result.lossiness.max_severity = Severity::Warning;
        assert!(!result.has_critical_issues());
        assert!(!result.has_errors());
        assert!(result.has_warnings());

        // Test info severity
        result.lossiness.max_severity = Severity::Info;
        assert!(!result.has_critical_issues());
        assert!(!result.has_errors());
        assert!(!result.has_warnings());
    }
}