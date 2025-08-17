//! Lossiness tracking infrastructure for translation operations
//!
//! This module will implement comprehensive lossiness tracking in issue #18.
//! Currently provides a minimal placeholder implementation.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::{LossinessCode, LossinessItem, LossinessReport, LossinessSummary, Severity, StrictMode};
use std::collections::HashMap;

/// Tracker for recording lossiness during translation
///
/// The LossinessTracker collects all deviations, limitations, and transformations
/// that occur during the translation process. It categorizes issues by severity
/// and type, providing a comprehensive report of any differences between the
/// uniform prompt and the provider-specific request.
pub struct LossinessTracker {
    items: Vec<LossinessItem>,
    strict_mode: StrictMode,
}

impl LossinessTracker {
    /// Create a new lossiness tracker
    pub fn new(strict_mode: StrictMode) -> Self {
        Self {
            items: Vec::new(),
            strict_mode,
        }
    }

    /// Add a clamped value lossiness item
    pub fn add_clamped(
        &mut self,
        path: &str,
        message: &str,
        original: Option<serde_json::Value>,
        clamped: Option<serde_json::Value>,
    ) {
        self.items.push(LossinessItem {
            code: LossinessCode::Clamp,
            path: path.to_string(),
            message: message.to_string(),
            severity: self.determine_severity(LossinessCode::Clamp),
            before: original,
            after: clamped,
        });
    }

    /// Add a dropped field lossiness item
    pub fn add_dropped(
        &mut self,
        path: &str,
        message: &str,
        dropped_value: Option<serde_json::Value>,
    ) {
        self.items.push(LossinessItem {
            code: LossinessCode::Drop,
            path: path.to_string(),
            message: message.to_string(),
            severity: self.determine_severity(LossinessCode::Drop),
            before: dropped_value,
            after: None,
        });
    }

    /// Add an emulated feature lossiness item
    pub fn add_emulated(
        &mut self,
        path: &str,
        message: &str,
        original: Option<serde_json::Value>,
    ) {
        self.items.push(LossinessItem {
            code: LossinessCode::Emulate,
            path: path.to_string(),
            message: message.to_string(),
            severity: self.determine_severity(LossinessCode::Emulate),
            before: original,
            after: None,
        });
    }

    /// Add a conflict resolution lossiness item
    pub fn add_conflict(
        &mut self,
        path: &str,
        message: &str,
        conflicting_values: Option<serde_json::Value>,
        resolved_value: Option<serde_json::Value>,
    ) {
        self.items.push(LossinessItem {
            code: LossinessCode::Conflict,
            path: path.to_string(),
            message: message.to_string(),
            severity: self.determine_severity(LossinessCode::Conflict),
            before: conflicting_values,
            after: resolved_value,
        });
    }

    /// Add a relocated field lossiness item
    pub fn add_relocated(
        &mut self,
        original_path: &str,
        new_path: &str,
        value: Option<serde_json::Value>,
    ) {
        self.items.push(LossinessItem {
            code: LossinessCode::Relocate,
            path: original_path.to_string(),
            message: format!("Field relocated from '{}' to '{}'", original_path, new_path),
            severity: self.determine_severity(LossinessCode::Relocate),
            before: value.clone(),
            after: value,
        });
    }

    /// Add an unsupported feature lossiness item
    pub fn add_unsupported(
        &mut self,
        path: &str,
        message: &str,
        unsupported_value: Option<serde_json::Value>,
    ) {
        self.items.push(LossinessItem {
            code: LossinessCode::Unsupported,
            path: path.to_string(),
            message: message.to_string(),
            severity: self.determine_severity(LossinessCode::Unsupported),
            before: unsupported_value,
            after: None,
        });
    }

    /// Add a mapping fallback lossiness item
    pub fn add_map_fallback(
        &mut self,
        path: &str,
        message: &str,
        original: Option<serde_json::Value>,
        fallback: Option<serde_json::Value>,
    ) {
        self.items.push(LossinessItem {
            code: LossinessCode::MapFallback,
            path: path.to_string(),
            message: message.to_string(),
            severity: self.determine_severity(LossinessCode::MapFallback),
            before: original,
            after: fallback,
        });
    }

    /// Add a performance impact lossiness item
    pub fn add_performance_impact(
        &mut self,
        path: &str,
        message: &str,
        affected_value: Option<serde_json::Value>,
    ) {
        self.items.push(LossinessItem {
            code: LossinessCode::PerformanceImpact,
            path: path.to_string(),
            message: message.to_string(),
            severity: self.determine_severity(LossinessCode::PerformanceImpact),
            before: affected_value,
            after: None,
        });
    }

    /// Add a custom lossiness item
    pub fn add_item(&mut self, item: LossinessItem) {
        self.items.push(item);
    }

    /// Determine severity based on lossiness code and strict mode
    fn determine_severity(&self, code: LossinessCode) -> Severity {
        match (code, self.strict_mode) {
            // Unsupported features are always critical since they cannot be emulated or coerced
            (LossinessCode::Unsupported, _) => Severity::Critical,
            
            // Drops are errors in strict mode
            (LossinessCode::Drop, StrictMode::Strict) => Severity::Error,
            (LossinessCode::Drop, _) => Severity::Warning,
            
            // Conflicts are errors in strict mode
            (LossinessCode::Conflict, StrictMode::Strict) => Severity::Error,
            (LossinessCode::Conflict, _) => Severity::Warning,
            
            // Clamps are warnings in strict mode
            (LossinessCode::Clamp, StrictMode::Strict) => Severity::Warning,
            (LossinessCode::Clamp, _) => Severity::Info,
            
            // Emulations are warnings
            (LossinessCode::Emulate, _) => Severity::Warning,
            
            // Relocations are info
            (LossinessCode::Relocate, _) => Severity::Info,
            
            // Map fallbacks are warnings
            (LossinessCode::MapFallback, _) => Severity::Warning,
            
            // Performance impacts are warnings
            (LossinessCode::PerformanceImpact, _) => Severity::Warning,
        }
    }

    /// Build the final lossiness report
    pub fn build_report(self) -> LossinessReport {
        // Calculate max severity
        let max_severity = self
            .items
            .iter()
            .map(|item| item.severity)
            .max()
            .unwrap_or(Severity::Info);

        // Build summary statistics
        let mut by_severity = HashMap::new();
        let mut by_code = HashMap::new();

        for item in &self.items {
            *by_severity
                .entry(item.severity.to_string())
                .or_insert(0) += 1;
            *by_code
                .entry(item.code.to_string())
                .or_insert(0) += 1;
        }

        let summary = LossinessSummary {
            total_items: self.items.len(),
            by_severity,
            by_code,
        };

        LossinessReport {
            items: self.items,
            max_severity,
            summary,
        }
    }

    /// Check if there are any critical issues
    pub fn has_critical_issues(&self) -> bool {
        self.items
            .iter()
            .any(|item| item.severity == Severity::Critical)
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.items
            .iter()
            .any(|item| item.severity >= Severity::Error)
    }

    /// Get the current number of items
    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_creation() {
        let tracker = LossinessTracker::new(StrictMode::Strict);
        assert_eq!(tracker.item_count(), 0);
        assert!(!tracker.has_errors());
        assert!(!tracker.has_critical_issues());
    }

    #[test]
    fn test_add_clamped() {
        let mut tracker = LossinessTracker::new(StrictMode::Warn);
        tracker.add_clamped(
            "temperature",
            "Temperature clamped to provider range",
            Some(serde_json::json!(2.5)),
            Some(serde_json::json!(2.0)),
        );
        
        assert_eq!(tracker.item_count(), 1);
        let report = tracker.build_report();
        assert_eq!(report.items[0].code, LossinessCode::Clamp);
        assert_eq!(report.items[0].severity, Severity::Info);
    }

    #[test]
    fn test_add_unsupported_strict() {
        let mut tracker = LossinessTracker::new(StrictMode::Strict);
        tracker.add_unsupported(
            "tools",
            "Tools not supported by provider",
            Some(serde_json::json!([])),
        );
        
        assert!(tracker.has_critical_issues());
        let report = tracker.build_report();
        assert_eq!(report.items[0].severity, Severity::Critical);
    }

    #[test]
    fn test_add_dropped_strict() {
        let mut tracker = LossinessTracker::new(StrictMode::Strict);
        tracker.add_dropped(
            "custom_field",
            "Custom field not supported",
            Some(serde_json::json!("value")),
        );
        
        assert!(tracker.has_errors());
        let report = tracker.build_report();
        assert_eq!(report.items[0].severity, Severity::Error);
    }

    #[test]
    fn test_build_report_summary() {
        let mut tracker = LossinessTracker::new(StrictMode::Warn);
        
        tracker.add_clamped("field1", "Clamped", None, None);
        tracker.add_dropped("field2", "Dropped", None);
        tracker.add_emulated("field3", "Emulated", None);
        tracker.add_clamped("field4", "Clamped", None, None);
        
        let report = tracker.build_report();
        
        assert_eq!(report.summary.total_items, 4);
        assert_eq!(report.summary.by_code.get("Clamp"), Some(&2));
        assert_eq!(report.summary.by_code.get("Drop"), Some(&1));
        assert_eq!(report.summary.by_code.get("Emulate"), Some(&1));
        assert_eq!(report.max_severity, Severity::Warning);
    }

    #[test]
    fn test_severity_determination() {
        // Test strict mode severities
        let mut tracker = LossinessTracker::new(StrictMode::Strict);
        tracker.add_unsupported("f1", "msg", None);
        tracker.add_dropped("f2", "msg", None);
        tracker.add_conflict("f3", "msg", None, None);
        tracker.add_clamped("f4", "msg", None, None);
        
        let report = tracker.build_report();
        assert_eq!(report.items[0].severity, Severity::Critical); // Unsupported
        assert_eq!(report.items[1].severity, Severity::Error);    // Drop
        assert_eq!(report.items[2].severity, Severity::Error);    // Conflict
        assert_eq!(report.items[3].severity, Severity::Warning);  // Clamp
        
        // Test warn mode severities
        let mut tracker = LossinessTracker::new(StrictMode::Warn);
        tracker.add_unsupported("f1", "msg", None);
        tracker.add_dropped("f2", "msg", None);
        tracker.add_conflict("f3", "msg", None, None);
        tracker.add_clamped("f4", "msg", None, None);
        
        let report = tracker.build_report();
        assert_eq!(report.items[0].severity, Severity::Critical); // Unsupported
        assert_eq!(report.items[1].severity, Severity::Warning); // Drop
        assert_eq!(report.items[2].severity, Severity::Warning); // Conflict
        assert_eq!(report.items[3].severity, Severity::Info);    // Clamp
    }
}