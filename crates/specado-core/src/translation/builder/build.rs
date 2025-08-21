//! Build operations and factory methods for TranslationResultBuilder
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use super::{BuilderError, BuilderState, TranslationResultBuilder};
use crate::{
    LossinessReport, LossinessSummary, Severity, TranslationResult,
    translation::TranslationContext,
};
use serde_json::Value;
use std::collections::HashMap;

impl TranslationResultBuilder {
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
}