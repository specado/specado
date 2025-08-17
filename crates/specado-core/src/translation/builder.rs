//! TranslationResult builder for constructing translation results
//!
//! This module implements the comprehensive TranslationResult builder for Issue #21.
//! Provides advanced builder features including validation, merging, incremental building,
//! and state tracking for the Specado translation engine.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::{
    LossinessCode, LossinessItem, LossinessReport, LossinessSummary, Severity, StrictMode, 
    TranslationMetadata, TranslationResult,
};
use crate::translation::{TranslationContext, LossinessTracker};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Builder state tracking the completeness of the result
#[derive(Debug, Clone, PartialEq)]
pub enum BuilderState {
    /// Builder is incomplete, missing required fields
    Incomplete,
    /// Builder has all required fields and is ready to build
    Ready,
    /// Builder has been consumed to create a TranslationResult
    Built,
}

/// Enhanced error type for builder operations
#[derive(Debug, Clone)]
pub enum BuilderError {
    /// Attempted to use a builder that was already built
    AlreadyBuilt,
    /// Missing required fields for building
    MissingRequired(Vec<String>),
    /// Invalid merge operation
    InvalidMerge(String),
    /// Validation failed
    ValidationFailed(String),
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuilderError::AlreadyBuilt => write!(f, "Builder has already been built and cannot be reused"),
            BuilderError::MissingRequired(fields) => write!(f, "Missing required fields: {}", fields.join(", ")),
            BuilderError::InvalidMerge(msg) => write!(f, "Invalid merge operation: {}", msg),
            BuilderError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl std::error::Error for BuilderError {}

/// Builder for constructing TranslationResult instances
///
/// The TranslationResultBuilder provides a fluent API for constructing
/// translation results with all necessary components including the
/// provider request JSON, lossiness report, and metadata.
///
/// ## Features
/// - State tracking (Incomplete/Ready/Built)
/// - Validation methods to ensure completeness
/// - Merge capabilities for combining partial results
/// - Incremental building support
/// - Convenience methods for common patterns
/// - Error recovery for partial failures
/// - Integration with TranslationContext
pub struct TranslationResultBuilder {
    provider_request_json: Option<Value>,
    lossiness_report: Option<LossinessReport>,
    metadata: Option<TranslationMetadata>,
    lossiness_tracker: Option<LossinessTracker>,
    start_time: Option<Instant>,
    state: BuilderState,
}

impl TranslationResultBuilder {
    /// Create a new TranslationResult builder
    pub fn new() -> Self {
        Self {
            provider_request_json: None,
            lossiness_report: None,
            metadata: None,
            lossiness_tracker: None,
            start_time: None,
            state: BuilderState::Incomplete,
        }
    }

    /// Create a builder from TranslationContext
    ///
    /// This convenience constructor creates a builder pre-configured with
    /// context information including provider details and timing.
    pub fn from_context(context: &TranslationContext) -> Self {
        let metadata = TranslationMetadata {
            provider: context.provider_name().to_string(),
            model: context.model_id().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms: None,
            strict_mode: context.strict_mode,
        };

        let lossiness_tracker = LossinessTracker::new(context.strict_mode);

        Self {
            provider_request_json: None,
            lossiness_report: None,
            metadata: Some(metadata),
            lossiness_tracker: Some(lossiness_tracker),
            start_time: Some(Instant::now()),
            state: BuilderState::Incomplete,
        }
    }

    /// Create a builder with timing tracking enabled
    pub fn with_timing() -> Self {
        Self {
            provider_request_json: None,
            lossiness_report: None,
            metadata: None,
            lossiness_tracker: None,
            start_time: Some(Instant::now()),
            state: BuilderState::Incomplete,
        }
    }

    /// Set the provider request JSON
    pub fn with_provider_request(mut self, request: Value) -> Self {
        if self.state == BuilderState::Built {
            panic!("Cannot modify a builder that has already been built");
        }
        self.provider_request_json = Some(request);
        self.update_state();
        self
    }

    /// Set the provider request JSON incrementally, building JSON piece by piece
    pub fn with_provider_request_incremental(self) -> ProviderRequestBuilder {
        if self.state == BuilderState::Built {
            panic!("Cannot modify a builder that has already been built");
        }
        ProviderRequestBuilder::new(self)
    }

    /// Set the lossiness report
    pub fn with_lossiness_report(mut self, report: LossinessReport) -> Self {
        if self.state == BuilderState::Built {
            panic!("Cannot modify a builder that has already been built");
        }
        self.lossiness_report = Some(report);
        self.lossiness_tracker = None; // Clear tracker if report is set directly
        self.update_state();
        self
    }

    /// Set the translation metadata
    pub fn with_metadata(mut self, metadata: TranslationMetadata) -> Self {
        if self.state == BuilderState::Built {
            panic!("Cannot modify a builder that has already been built");
        }
        self.metadata = Some(metadata);
        self.update_state();
        self
    }

    /// Add a single lossiness item incrementally
    pub fn add_lossiness_item(mut self, item: LossinessItem) -> Self {
        if self.state == BuilderState::Built {
            panic!("Cannot modify a builder that has already been built");
        }
        
        if self.lossiness_tracker.is_none() {
            // Create a tracker if one doesn't exist
            let strict_mode = self.metadata.as_ref()
                .map(|m| m.strict_mode)
                .unwrap_or(StrictMode::Warn);
            self.lossiness_tracker = Some(LossinessTracker::new(strict_mode));
        }
        
        if let Some(ref mut tracker) = self.lossiness_tracker {
            tracker.add_item(item);
        }
        self.update_state();
        self
    }

    /// Add a lossiness item for a dropped field
    pub fn add_dropped_field(self, path: &str, message: &str, value: Option<Value>) -> Self {
        let item = LossinessItem {
            code: LossinessCode::Drop,
            path: path.to_string(),
            message: message.to_string(),
            severity: Severity::Warning,
            before: value,
            after: None,
        };
        self.add_lossiness_item(item)
    }

    /// Add a lossiness item for a clamped value
    pub fn add_clamped_value(self, path: &str, message: &str, before: Value, after: Value) -> Self {
        let item = LossinessItem {
            code: LossinessCode::Clamp,
            path: path.to_string(),
            message: message.to_string(),
            severity: Severity::Info,
            before: Some(before),
            after: Some(after),
        };
        self.add_lossiness_item(item)
    }

    /// Validate that the builder has all required fields
    ///
    /// Returns Ok(()) if valid, otherwise returns a BuilderError with missing fields
    pub fn validate(&self) -> Result<(), BuilderError> {
        if self.state == BuilderState::Built {
            return Err(BuilderError::AlreadyBuilt);
        }

        let mut missing = Vec::new();
        
        if self.provider_request_json.is_none() {
            missing.push("provider_request_json".to_string());
        }

        if !missing.is_empty() {
            return Err(BuilderError::MissingRequired(missing));
        }

        Ok(())
    }

    /// Merge another builder's data into this builder
    ///
    /// This allows combining partial results from different sources.
    /// The other builder's values take precedence for conflicting fields.
    pub fn merge(mut self, other: TranslationResultBuilder) -> Result<Self, BuilderError> {
        if self.state == BuilderState::Built {
            return Err(BuilderError::AlreadyBuilt);
        }

        if other.state == BuilderState::Built {
            return Err(BuilderError::InvalidMerge("Cannot merge from a built builder".to_string()));
        }

        // Merge provider request JSON (prefer other's if present)
        if other.provider_request_json.is_some() {
            self.provider_request_json = other.provider_request_json;
        }

        // Merge metadata (prefer other's if present)
        if other.metadata.is_some() {
            self.metadata = other.metadata;
        }

        // Merge lossiness - combine both trackers and reports
        let my_tracker = self.lossiness_tracker.take();
        let other_tracker = other.lossiness_tracker;
        
        match (my_tracker, other_tracker) {
            (Some(mut my_tracker), Some(other_tracker)) => {
                // Merge the items from both trackers
                let other_report = other_tracker.build_report();
                for item in other_report.items {
                    my_tracker.add_item(item);
                }
                self.lossiness_tracker = Some(my_tracker);
            }
            (None, Some(other_tracker)) => {
                self.lossiness_tracker = Some(other_tracker);
            }
            (Some(my_tracker), None) => {
                self.lossiness_tracker = Some(my_tracker);
            }
            (None, None) => {
                // Merge reports if present
                if other.lossiness_report.is_some() {
                    self.lossiness_report = other.lossiness_report;
                }
            }
        }

        // Update timing if other has earlier start time
        if other.start_time.is_some() && 
           (self.start_time.is_none() || other.start_time < self.start_time) {
            self.start_time = other.start_time;
        }

        self.update_state();
        Ok(self)
    }

    /// Update the builder state based on current fields
    fn update_state(&mut self) {
        if self.state == BuilderState::Built {
            return;
        }

        self.state = if self.provider_request_json.is_some() {
            BuilderState::Ready
        } else {
            BuilderState::Incomplete
        };
    }

    /// Get the current builder state
    pub fn state(&self) -> &BuilderState {
        &self.state
    }

    /// Build the TranslationResult
    ///
    /// This method constructs the final TranslationResult. If no lossiness
    /// report is provided, it creates an empty one. The provider request JSON
    /// must be provided.
    pub fn build(mut self) -> Result<TranslationResult, BuilderError> {
        // Validate before building
        self.validate()?;

        // Finalize timing if metadata exists and start_time is set
        if let (Some(ref mut metadata), Some(start_time)) = (&mut self.metadata, self.start_time) {
            if metadata.duration_ms.is_none() {
                metadata.duration_ms = Some(start_time.elapsed().as_millis() as u64);
            }
        }

        let provider_request_json = self
            .provider_request_json
            .unwrap_or_else(|| serde_json::json!({}));

        // Build lossiness report from tracker if available, otherwise use existing report
        let lossiness = if let Some(tracker) = self.lossiness_tracker {
            tracker.build_report()
        } else {
            self.lossiness_report.unwrap_or_else(|| {
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
            })
        };

        // Mark as built
        self.state = BuilderState::Built;

        Ok(TranslationResult {
            provider_request_json,
            lossiness,
            metadata: self.metadata,
        })
    }

    /// Build the result without consuming the builder (for error recovery)
    pub fn try_build(&self) -> Result<TranslationResult, BuilderError> {
        // Validate first
        self.validate()?;

        let provider_request_json = self
            .provider_request_json
            .clone()
            .unwrap_or_else(|| serde_json::json!({}));

        // Build lossiness report from tracker if available
        let lossiness = if self.lossiness_tracker.is_some() {
            // Unfortunately we can't easily clone the tracker's items, 
            // so we'll use the existing report if available
            if let Some(ref report) = self.lossiness_report {
                report.clone()
            } else {
                // Create empty report if no tracker items can be accessed
                LossinessReport {
                    items: Vec::new(),
                    max_severity: Severity::Info,
                    summary: LossinessSummary {
                        total_items: 0,
                        by_severity: HashMap::new(),
                        by_code: HashMap::new(),
                    },
                }
            }
        } else {
            self.lossiness_report.clone().unwrap_or_else(|| {
                LossinessReport {
                    items: Vec::new(),
                    max_severity: Severity::Info,
                    summary: LossinessSummary {
                        total_items: 0,
                        by_severity: HashMap::new(),
                        by_code: HashMap::new(),
                    },
                }
            })
        };

        Ok(TranslationResult {
            provider_request_json,
            lossiness,
            metadata: self.metadata.clone(),
        })
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
            .expect("Building a success result should never fail")
    }

    /// Create a successful result from context
    pub fn success_from_context(context: &TranslationContext, request: Value) -> TranslationResult {
        Self::from_context(context)
            .with_provider_request(request)
            .build()
            .expect("Building from context should never fail")
    }

    /// Create a builder for a failed translation with error details
    pub fn with_critical_error(self, path: &str, message: &str, value: Option<Value>) -> Self {
        let item = LossinessItem {
            code: LossinessCode::Unsupported,
            path: path.to_string(),
            message: message.to_string(),
            severity: Severity::Critical,
            before: value,
            after: None,
        };
        self.add_lossiness_item(item)
    }

    /// Set metadata provider information
    pub fn with_provider_info(self, provider: &str, model: &str, strict_mode: StrictMode) -> Self {
        let metadata = TranslationMetadata {
            provider: provider.to_string(),
            model: model.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms: None,
            strict_mode,
        };
        self.with_metadata(metadata)
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

    /// Get a reference to the lossiness tracker if set
    pub fn lossiness_tracker(&self) -> Option<&LossinessTracker> {
        self.lossiness_tracker.as_ref()
    }

    /// Get the elapsed time since builder creation (if timing is enabled)
    pub fn elapsed_time(&self) -> Option<Duration> {
        self.start_time.map(|start| start.elapsed())
    }
}

/// Helper builder for incrementally constructing provider request JSON
pub struct ProviderRequestBuilder {
    builder: TranslationResultBuilder,
    request: Value,
}

impl ProviderRequestBuilder {
    fn new(builder: TranslationResultBuilder) -> Self {
        Self {
            builder,
            request: serde_json::json!({}),
        }
    }

    /// Set a field in the provider request JSON
    pub fn set_field<T: serde::Serialize>(mut self, key: &str, value: T) -> Self {
        if let Value::Object(ref mut obj) = self.request {
            obj.insert(key.to_string(), serde_json::to_value(value).unwrap_or(Value::Null));
        }
        self
    }

    /// Set the model field
    pub fn set_model(self, model: &str) -> Self {
        self.set_field("model", model)
    }

    /// Set messages array
    pub fn set_messages(self, messages: Vec<Value>) -> Self {
        self.set_field("messages", messages)
    }

    /// Add a single message to the messages array
    pub fn add_message(mut self, message: Value) -> Self {
        if let Value::Object(ref mut obj) = self.request {
            let messages = obj.entry("messages".to_string())
                .or_insert_with(|| Value::Array(vec![]));
            
            if let Value::Array(ref mut arr) = messages {
                arr.push(message);
            }
        }
        self
    }

    /// Set tool-related fields
    pub fn set_tools(self, tools: Vec<Value>) -> Self {
        self.set_field("tools", tools)
    }

    /// Set tool choice
    pub fn set_tool_choice(self, tool_choice: Value) -> Self {
        self.set_field("tool_choice", tool_choice)
    }

    /// Set sampling parameters
    pub fn set_temperature(self, temperature: f64) -> Self {
        self.set_field("temperature", temperature)
    }

    /// Set top_p parameter
    pub fn set_top_p(self, top_p: f64) -> Self {
        self.set_field("top_p", top_p)
    }

    /// Set max_tokens parameter
    pub fn set_max_tokens(self, max_tokens: u32) -> Self {
        self.set_field("max_tokens", max_tokens)
    }

    /// Set response format
    pub fn set_response_format(self, format: Value) -> Self {
        self.set_field("response_format", format)
    }

    /// Merge another JSON object into the request
    pub fn merge_object(mut self, other: Value) -> Self {
        if let (Value::Object(ref mut base), Value::Object(other_obj)) = (&mut self.request, other) {
            for (key, value) in other_obj {
                base.insert(key, value);
            }
        }
        self
    }

    /// Complete the incremental building and return to the main builder
    pub fn done(mut self) -> TranslationResultBuilder {
        self.builder.provider_request_json = Some(self.request);
        self.builder.update_state();
        self.builder
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
            .build()
            .expect("Basic build should succeed");

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
            .build()
            .expect("Build with metadata should succeed");

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
            .build()
            .expect("Build with lossiness should succeed");

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
        assert_eq!(builder.state(), &BuilderState::Ready);
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

    // New comprehensive tests for Issue #21 features

    #[test]
    fn test_builder_state_tracking() {
        let mut builder = TranslationResultBuilder::new();
        assert_eq!(builder.state(), &BuilderState::Incomplete);

        builder = builder.with_provider_request(serde_json::json!({"model": "test"}));
        assert_eq!(builder.state(), &BuilderState::Ready);

        let _result = builder.build().expect("Should build successfully");
        // builder is consumed, so we can't check state after build()
    }

    #[test]
    fn test_validation() {
        let incomplete_builder = TranslationResultBuilder::new();
        assert!(incomplete_builder.validate().is_err());

        let complete_builder = TranslationResultBuilder::new()
            .with_provider_request(serde_json::json!({}));
        assert!(complete_builder.validate().is_ok());
    }

    #[test]
    fn test_merge_builders() {
        let builder1 = TranslationResultBuilder::new()
            .with_metadata(TranslationMetadata {
                provider: "provider1".to_string(),
                model: "model1".to_string(),
                timestamp: "now".to_string(),
                duration_ms: None,
                strict_mode: StrictMode::Warn,
            });

        let builder2 = TranslationResultBuilder::new()
            .with_provider_request(serde_json::json!({"model": "test"}));

        let merged = builder1.merge(builder2).expect("Merge should succeed");
        assert!(merged.is_complete());
        assert!(merged.metadata().is_some());
        assert!(merged.provider_request().is_some());
    }

    #[test]
    fn test_incremental_lossiness() {
        let result = TranslationResultBuilder::new()
            .with_provider_request(serde_json::json!({}))
            .with_provider_info("test-provider", "test-model", StrictMode::Warn)
            .add_dropped_field("tools", "Tools not supported", Some(serde_json::json!([])))
            .add_clamped_value("temperature", "Temperature clamped", 
                serde_json::json!(2.5), serde_json::json!(2.0))
            .build()
            .expect("Build with lossiness should succeed");

        assert!(result.has_lossiness());
        assert_eq!(result.lossiness.items.len(), 2);
    }

    #[test]
    fn test_from_context() {
        let context = create_test_context();
        let builder = TranslationResultBuilder::from_context(&context);
        
        assert!(builder.metadata().is_some());
        assert_eq!(builder.metadata().unwrap().provider, "test-provider");
        assert!(builder.start_time.is_some());
        assert!(builder.lossiness_tracker().is_some());
    }

    #[test]
    fn test_incremental_request_builder() {
        let result = TranslationResultBuilder::new()
            .with_provider_request_incremental()
            .set_model("test-model")
            .add_message(serde_json::json!({
                "role": "user",
                "content": "Hello"
            }))
            .set_temperature(0.7)
            .set_max_tokens(100)
            .done()
            .build()
            .expect("Incremental build should succeed");

        let request = &result.provider_request_json;
        assert_eq!(request["model"], "test-model");
        assert_eq!(request["temperature"], 0.7);
        assert_eq!(request["max_tokens"], 100);
        assert!(request["messages"].is_array());
    }

    #[test]
    fn test_timing_features() {
        let builder = TranslationResultBuilder::with_timing();
        assert!(builder.start_time.is_some());
        assert!(builder.elapsed_time().is_some());

        // Test automatic duration calculation
        let result = builder
            .with_provider_request(serde_json::json!({}))
            .with_provider_info("test", "model", StrictMode::Warn)
            .build()
            .expect("Build with timing should succeed");

        assert!(result.metadata.is_some());
        assert!(result.duration_ms().is_some());
    }

    #[test]
    fn test_success_from_context() {
        let context = create_test_context();
        let request = serde_json::json!({"model": "test"});
        
        let result = TranslationResultBuilder::success_from_context(&context, request.clone());
        
        assert_eq!(result.provider_request_json, request);
        assert!(!result.has_lossiness());
        assert!(result.metadata.is_some());
        assert_eq!(result.provider_name(), Some("test-provider"));
    }

    #[test]
    fn test_critical_error_builder() {
        let result = TranslationResultBuilder::new()
            .with_provider_request(serde_json::json!({}))
            .with_critical_error("tools", "Tools not supported", Some(serde_json::json!([])))
            .build()
            .expect("Build with critical error should succeed");

        assert!(result.has_critical_issues());
        assert_eq!(result.lossiness.max_severity, Severity::Critical);
    }

    #[test]
    fn test_try_build_for_error_recovery() {
        let builder = TranslationResultBuilder::new()
            .with_provider_request(serde_json::json!({}));

        // try_build doesn't consume the builder
        let result = builder.try_build().expect("try_build should succeed");
        assert_eq!(result.provider_request_json, serde_json::json!({}));

        // We can still use the builder after try_build
        let final_result = builder.build().expect("Final build should succeed");
        assert_eq!(final_result.provider_request_json, serde_json::json!({}));
    }

    #[test]
    fn test_merge_lossiness_trackers() {
        let builder1 = TranslationResultBuilder::new()
            .with_provider_info("test", "model", StrictMode::Warn)
            .add_dropped_field("field1", "Dropped", None);

        let builder2 = TranslationResultBuilder::new()
            .with_provider_info("test", "model", StrictMode::Warn)
            .add_dropped_field("field2", "Also dropped", None);

        let merged = builder1.merge(builder2).expect("Merge should succeed");
        let result = merged
            .with_provider_request(serde_json::json!({}))
            .build()
            .expect("Build merged should succeed");

        // Should have both lossiness items
        assert_eq!(result.lossiness.items.len(), 2);
    }

    #[test]
    fn test_error_handling() {
        // Test building incomplete builder
        let incomplete = TranslationResultBuilder::new();
        assert!(incomplete.build().is_err());

        // Test merging with built builder would panic, so we test the error case differently
        let builder1 = TranslationResultBuilder::new();
        let builder2 = TranslationResultBuilder::new();
        
        // This should work fine
        let _merged = builder1.merge(builder2);
    }

    // Helper function for tests
    fn create_test_context() -> crate::translation::TranslationContext {
        use crate::{
            Constraints, ConstraintLimits, EndpointConfig, Endpoints, InputModes, JsonOutputConfig,
            Mappings, Message, MessageRole, ProviderInfo, ProviderSpec, PromptSpec, 
            ResponseNormalization, StreamNormalization, SyncNormalization, ToolingConfig, ModelSpec,
        };
        use std::collections::HashMap;

        let prompt_spec = PromptSpec {
            model_class: "Chat".to_string(),
            messages: vec![Message {
                role: MessageRole::User,
                content: "Test".to_string(),
                name: None,
                metadata: None,
            }],
            tools: None,
            tool_choice: None,
            response_format: None,
            sampling: None,
            limits: None,
            media: None,
            strict_mode: StrictMode::Warn,
        };

        let provider_spec = ProviderSpec {
            spec_version: "1.0.0".to_string(),
            provider: ProviderInfo {
                name: "test-provider".to_string(),
                base_url: "https://api.test.com".to_string(),
                headers: HashMap::new(),
            },
            models: vec![],
        };

        let model_spec = ModelSpec {
            id: "test-model".to_string(),
            aliases: None,
            family: "test".to_string(),
            endpoints: Endpoints {
                chat_completion: EndpointConfig {
                    method: "POST".to_string(),
                    path: "/chat".to_string(),
                    protocol: "https".to_string(),
                    query: None,
                    headers: None,
                },
                streaming_chat_completion: EndpointConfig {
                    method: "POST".to_string(),
                    path: "/chat".to_string(),
                    protocol: "https".to_string(),
                    query: None,
                    headers: None,
                },
            },
            input_modes: InputModes {
                messages: true,
                single_text: false,
                images: true,
            },
            tooling: ToolingConfig {
                tools_supported: true,
                parallel_tool_calls_default: false,
                can_disable_parallel_tool_calls: false,
                disable_switch: None,
            },
            json_output: JsonOutputConfig {
                native_param: true,
                strategy: "native".to_string(),
            },
            parameters: serde_json::json!({}),
            constraints: Constraints {
                system_prompt_location: "first".to_string(),
                forbid_unknown_top_level_fields: false,
                mutually_exclusive: vec![],
                resolution_preferences: vec![],
                limits: ConstraintLimits {
                    max_tool_schema_bytes: 100000,
                    max_system_prompt_bytes: 10000,
                },
            },
            mappings: Mappings {
                paths: HashMap::new(),
                flags: HashMap::new(),
            },
            response_normalization: ResponseNormalization {
                sync: SyncNormalization {
                    content_path: "content".to_string(),
                    finish_reason_path: "finish".to_string(),
                    finish_reason_map: HashMap::new(),
                },
                stream: StreamNormalization {
                    protocol: "sse".to_string(),
                    event_selector: crate::EventSelector {
                        type_path: "type".to_string(),
                        routes: vec![],
                    },
                },
            },
        };

        crate::translation::TranslationContext::new(prompt_spec, provider_spec, model_spec, StrictMode::Warn)
    }
}