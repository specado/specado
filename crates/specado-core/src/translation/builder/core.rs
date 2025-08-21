//! Core TranslationResultBuilder structure and basic operations
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use super::{BuilderState, ProviderRequestBuilder};
use crate::{
    LossinessReport, TranslationMetadata,
    translation::{TranslationContext, LossinessTracker},
    StrictMode,
};
use serde_json::Value;
use std::time::{Duration, Instant};

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
    pub(super) provider_request_json: Option<Value>,
    pub(super) lossiness_report: Option<LossinessReport>,
    pub(super) metadata: Option<TranslationMetadata>,
    pub(super) lossiness_tracker: Option<LossinessTracker>,
    pub(super) start_time: Option<Instant>,
    pub(super) state: BuilderState,
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

    /// Update the builder state based on current fields
    pub(super) fn update_state(&mut self) {
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

impl Default for TranslationResultBuilder {
    fn default() -> Self {
        Self::new()
    }
}