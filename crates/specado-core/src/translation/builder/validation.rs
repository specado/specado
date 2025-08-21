//! Validation and merge operations for TranslationResultBuilder
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use super::{BuilderError, BuilderState, TranslationResultBuilder};

impl TranslationResultBuilder {
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
}