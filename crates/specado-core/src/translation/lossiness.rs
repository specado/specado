//! Lossiness tracking infrastructure for translation operations
//!
//! This module provides comprehensive lossiness tracking capabilities for translation operations.
//! It tracks all transformations, field movements, coercions, and other changes that occur
//! during the translation process, maintaining a detailed audit trail.
//!
//! # Examples
//!
//! ```
//! use specado_core::translation::lossiness::{LossinessTracker, OperationType};
//! use specado_core::StrictMode;
//! use std::collections::HashMap;
//!
//! let mut tracker = LossinessTracker::new(StrictMode::Warn);
//!
//! // Track a field transformation
//! tracker.track_transformation(
//!     "$.temperature",
//!     OperationType::TypeConversion,
//!     Some(serde_json::json!("1.5")),
//!     Some(serde_json::json!(1.5)),
//!     "Converted string to number",
//!     Some("OpenAI".to_string()),
//!     HashMap::new(),
//! );
//!
//! // Track a dropped field
//! tracker.track_dropped_field(
//!     "$.custom_field",
//!     Some(serde_json::json!("value")),
//!     "Field not supported by provider",
//!     Some("OpenAI".to_string()),
//! );
//!
//! // Get summary statistics
//! let stats = tracker.get_summary_statistics();
//! assert_eq!(stats.total_transformations, 2);
//! assert_eq!(stats.dropped_fields, 1);
//!
//! // Generate audit report
//! let report = tracker.generate_audit_report();
//! println!("{}", report);
//!
//! // Query specific transformations
//! let dropped_fields = tracker.get_dropped_fields();
//! assert_eq!(dropped_fields.len(), 1);
//! ```
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::{LossinessCode, LossinessItem, LossinessReport, LossinessSummary, Severity, StrictMode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Operation types for transformations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    /// Type conversion (e.g., string to number)
    TypeConversion,
    /// Enum value mapping
    EnumMapping,
    /// Unit conversion (e.g., temperature scales)
    UnitConversion,
    /// Value coercion to fit constraints
    Coercion,
    /// Field dropped/removed
    Dropped,
    /// Field moved to different location
    FieldMove,
    /// Default value applied
    DefaultApplied,
}

/// Record of a single transformation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationRecord {
    /// JSONPath to the affected field
    pub field_path: String,
    /// Type of operation performed
    pub operation_type: OperationType,
    /// Original value before transformation
    pub before_value: Option<Value>,
    /// Value after transformation
    pub after_value: Option<Value>,
    /// Human-readable reason for the transformation
    pub reason: String,
    /// When this transformation occurred
    pub timestamp: DateTime<Utc>,
    /// Additional context metadata
    pub metadata: HashMap<String, String>,
    /// Which provider required this transformation
    pub provider_context: Option<String>,
}

/// Performance metrics for transformations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total duration of all transformations
    pub total_duration: Duration,
    /// Duration of each transformation by field path
    pub transformation_times: HashMap<String, Duration>,
    /// Slowest transformation (field_path, duration)
    pub slowest_transformation: Option<(String, Duration)>,
}

/// Complete audit trail of transformations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrail {
    /// All transformation records
    pub records: Vec<TransformationRecord>,
    /// When tracking started
    pub start_time: DateTime<Utc>,
    /// When tracking ended (if completed)
    pub end_time: Option<DateTime<Utc>>,
    /// Total number of transformations
    pub total_transformations: usize,
    /// Number of fields that were dropped
    pub dropped_fields_count: usize,
    /// Number of value coercions
    pub coercion_count: usize,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
}

impl Default for AuditTrail {
    fn default() -> Self {
        Self {
            records: Vec::new(),
            start_time: Utc::now(),
            end_time: None,
            total_transformations: 0,
            dropped_fields_count: 0,
            coercion_count: 0,
            performance_metrics: PerformanceMetrics::default(),
        }
    }
}

/// Summary statistics for reporting
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SummaryStats {
    /// Total transformations performed
    pub total_transformations: usize,
    /// Transformations by operation type
    pub by_operation_type: HashMap<String, usize>,
    /// Fields affected
    pub affected_fields: usize,
    /// Fields dropped
    pub dropped_fields: usize,
    /// Average transformation time
    pub avg_transformation_time_ms: f64,
    /// Most common transformation type
    pub most_common_operation: Option<String>,
}

/// Performance report for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    /// Total time spent on transformations
    pub total_time_ms: u64,
    /// Average time per transformation
    pub avg_time_per_transformation_ms: f64,
    /// Slowest operation
    pub slowest_operation: Option<(String, u64)>,
    /// Operations by duration (field_path, duration_ms)
    pub operations_by_duration: Vec<(String, u64)>,
}

/// Tracker for recording lossiness during translation
///
/// The LossinessTracker collects all deviations, limitations, and transformations
/// that occur during the translation process. It categorizes issues by severity
/// and type, providing a comprehensive report of any differences between the
/// uniform prompt and the provider-specific request.
#[derive(Debug)]
pub struct LossinessTracker {
    items: Vec<LossinessItem>,
    strict_mode: StrictMode,
    /// Comprehensive audit trail of all transformations
    audit_trail: AuditTrail,
    /// Index for quick field lookups (field_path -> record indices)
    field_index: HashMap<String, Vec<usize>>,
    /// Timing information for performance tracking
    timing_start: Option<Instant>,
}

impl LossinessTracker {
    /// Create a new lossiness tracker
    pub fn new(strict_mode: StrictMode) -> Self {
        Self {
            items: Vec::new(),
            strict_mode,
            audit_trail: AuditTrail::default(),
            field_index: HashMap::new(),
            timing_start: Some(Instant::now()),
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

    /// Track a transformation operation with timing
    pub fn track_transformation_with_timing<F, R>(
        &mut self,
        field_path: &str,
        operation_type: OperationType,
        before_value: Option<Value>,
        after_value: Option<Value>,
        reason: &str,
        provider_context: Option<String>,
        metadata: HashMap<String, String>,
        operation: F,
    ) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();
        
        // Update performance metrics
        self.audit_trail.performance_metrics.transformation_times
            .insert(field_path.to_string(), duration);
            
        // Update slowest if this is slower
        match &self.audit_trail.performance_metrics.slowest_transformation {
            Some((_, current_slowest)) if duration > *current_slowest => {
                self.audit_trail.performance_metrics.slowest_transformation = 
                    Some((field_path.to_string(), duration));
            }
            None => {
                self.audit_trail.performance_metrics.slowest_transformation = 
                    Some((field_path.to_string(), duration));
            }
            _ => {} // Current slowest is still slower
        }
        
        // Record the transformation
        self.track_transformation(
            field_path,
            operation_type,
            before_value,
            after_value,
            reason,
            provider_context,
            metadata,
        );
        
        result
    }

    /// Track a transformation operation
    pub fn track_transformation(
        &mut self,
        field_path: &str,
        operation_type: OperationType,
        before_value: Option<Value>,
        after_value: Option<Value>,
        reason: &str,
        provider_context: Option<String>,
        metadata: HashMap<String, String>,
    ) {
        let record = TransformationRecord {
            field_path: field_path.to_string(),
            operation_type: operation_type.clone(),
            before_value,
            after_value,
            reason: reason.to_string(),
            timestamp: Utc::now(),
            metadata,
            provider_context,
        };

        let record_index = self.audit_trail.records.len();
        self.audit_trail.records.push(record);
        
        // Update field index
        self.field_index
            .entry(field_path.to_string())
            .or_insert_with(Vec::new)
            .push(record_index);

        // Update counters
        self.audit_trail.total_transformations += 1;
        match operation_type {
            OperationType::Dropped => self.audit_trail.dropped_fields_count += 1,
            OperationType::Coercion => self.audit_trail.coercion_count += 1,
            _ => {}
        }
    }

    /// Track a dropped field
    pub fn track_dropped_field(
        &mut self,
        field_path: &str,
        original_value: Option<Value>,
        reason: &str,
        provider_context: Option<String>,
    ) {
        let mut metadata = HashMap::new();
        metadata.insert("action".to_string(), "field_dropped".to_string());
        
        self.track_transformation(
            field_path,
            OperationType::Dropped,
            original_value,
            None,
            reason,
            provider_context,
            metadata,
        );
    }

    /// Track a value coercion
    pub fn track_coercion(
        &mut self,
        field_path: &str,
        original_value: Option<Value>,
        coerced_value: Option<Value>,
        reason: &str,
        provider_context: Option<String>,
    ) {
        let mut metadata = HashMap::new();
        metadata.insert("action".to_string(), "value_coerced".to_string());
        
        self.track_transformation(
            field_path,
            OperationType::Coercion,
            original_value,
            coerced_value,
            reason,
            provider_context,
            metadata,
        );
    }

    /// Track when a default value is applied
    pub fn track_default_applied(
        &mut self,
        field_path: &str,
        default_value: Option<Value>,
        reason: &str,
        provider_context: Option<String>,
    ) {
        let mut metadata = HashMap::new();
        metadata.insert("action".to_string(), "default_applied".to_string());
        
        self.track_transformation(
            field_path,
            OperationType::DefaultApplied,
            None,
            default_value,
            reason,
            provider_context,
            metadata,
        );
    }

    /// Get transformation history for a specific field
    pub fn get_field_history(&self, path: &str) -> Vec<&TransformationRecord> {
        self.field_index
            .get(path)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&i| self.audit_trail.records.get(i))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all dropped fields
    pub fn get_dropped_fields(&self) -> Vec<&TransformationRecord> {
        self.audit_trail
            .records
            .iter()
            .filter(|record| record.operation_type == OperationType::Dropped)
            .collect()
    }

    /// Get all coercions
    pub fn get_coercions(&self) -> Vec<&TransformationRecord> {
        self.audit_trail
            .records
            .iter()
            .filter(|record| record.operation_type == OperationType::Coercion)
            .collect()
    }

    /// Get transformations by operation type
    pub fn get_transformations_by_type(&self, op_type: OperationType) -> Vec<&TransformationRecord> {
        self.audit_trail
            .records
            .iter()
            .filter(|record| record.operation_type == op_type)
            .collect()
    }

    /// Check if a field has been changed/transformed
    pub fn has_field_changed(&self, path: &str) -> bool {
        self.field_index.contains_key(path)
    }

    /// Generate a human-readable audit report
    pub fn generate_audit_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== Transformation Audit Report ===\n\n");
        
        let stats = self.get_summary_statistics();
        report.push_str(&format!("Total Transformations: {}\n", stats.total_transformations));
        report.push_str(&format!("Affected Fields: {}\n", stats.affected_fields));
        report.push_str(&format!("Dropped Fields: {}\n", stats.dropped_fields));
        
        if let Some(ref common_op) = stats.most_common_operation {
            report.push_str(&format!("Most Common Operation: {}\n", common_op));
        }
        
        report.push_str(&format!("Average Transformation Time: {:.2}ms\n\n", stats.avg_transformation_time_ms));

        // Group by operation type
        for (op_type, count) in &stats.by_operation_type {
            report.push_str(&format!("--- {} Operations ({}) ---\n", op_type, count));
            
            let matching_records: Vec<_> = self.audit_trail
                .records
                .iter()
                .filter(|r| format!("{:?}", r.operation_type) == *op_type)
                .collect();
                
            for record in matching_records.iter().take(5) { // Show first 5 of each type
                report.push_str(&format!(
                    "  • {}: {} -> {}\n    Reason: {}\n",
                    record.field_path,
                    record.before_value.as_ref()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "None".to_string()),
                    record.after_value.as_ref()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "None".to_string()),
                    record.reason
                ));
            }
            if matching_records.len() > 5 {
                report.push_str(&format!("    ... and {} more\n", matching_records.len() - 5));
            }
            report.push('\n');
        }

        report
    }

    /// Convert audit trail to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.audit_trail)
    }

    /// Get summary statistics
    pub fn get_summary_statistics(&self) -> SummaryStats {
        let mut by_operation_type = HashMap::new();
        let mut affected_fields = std::collections::HashSet::new();

        for record in &self.audit_trail.records {
            let op_type = format!("{:?}", record.operation_type);
            *by_operation_type.entry(op_type).or_insert(0) += 1;
            affected_fields.insert(record.field_path.clone());
        }

        // Calculate average time if we have timing data
        let avg_transformation_time_ms = if self.audit_trail.total_transformations > 0 {
            self.audit_trail.performance_metrics.total_duration.as_millis() as f64
                / self.audit_trail.total_transformations as f64
        } else {
            0.0
        };

        let most_common_operation = by_operation_type
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(op_type, _)| op_type.clone());

        SummaryStats {
            total_transformations: self.audit_trail.total_transformations,
            by_operation_type,
            affected_fields: affected_fields.len(),
            dropped_fields: self.audit_trail.dropped_fields_count,
            avg_transformation_time_ms,
            most_common_operation,
        }
    }

    /// Get performance report
    pub fn get_performance_report(&self) -> PerformanceReport {
        let total_time_ms = self.audit_trail.performance_metrics.total_duration.as_millis() as u64;
        
        let avg_time_per_transformation_ms = if self.audit_trail.total_transformations > 0 {
            total_time_ms as f64 / self.audit_trail.total_transformations as f64
        } else {
            0.0
        };

        let slowest_operation = self.audit_trail.performance_metrics.slowest_transformation
            .as_ref()
            .map(|(path, duration)| (path.clone(), duration.as_millis() as u64));

        let mut operations_by_duration: Vec<_> = self.audit_trail.performance_metrics.transformation_times
            .iter()
            .map(|(path, duration)| (path.clone(), duration.as_millis() as u64))
            .collect();
        operations_by_duration.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by duration descending

        PerformanceReport {
            total_time_ms,
            avg_time_per_transformation_ms,
            slowest_operation,
            operations_by_duration,
        }
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
    pub fn build_report(mut self) -> LossinessReport {
        // Finalize the audit trail
        self.audit_trail.end_time = Some(Utc::now());
        
        // Update performance metrics if we have timing data
        if let Some(start_time) = self.timing_start {
            let total_duration = start_time.elapsed();
            self.audit_trail.performance_metrics.total_duration = total_duration;
            
            // Find slowest transformation (simplified - in real implementation would track individual timings)
            if !self.audit_trail.records.is_empty() {
                let avg_per_transformation = total_duration / self.audit_trail.records.len() as u32;
                // For now, just mark the first transformation as potentially slowest
                if let Some(first_record) = self.audit_trail.records.first() {
                    self.audit_trail.performance_metrics.slowest_transformation = 
                        Some((first_record.field_path.clone(), avg_per_transformation));
                }
            }
        }
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
    
    /// Update the after value of the last transformation record
    pub fn update_last_transformation_after_value(&mut self, after_value: Option<Value>) {
        if let Some(last_record) = self.audit_trail.records.last_mut() {
            last_record.after_value = after_value;
        }
    }
    
    /// Update performance metrics for a specific transformation
    pub fn update_transformation_timing(&mut self, field_path: &str, duration: Duration) {
        self.audit_trail.performance_metrics.transformation_times
            .insert(field_path.to_string(), duration);
            
        // Update slowest if this is slower
        match &self.audit_trail.performance_metrics.slowest_transformation {
            Some((_, current_slowest)) if duration > *current_slowest => {
                self.audit_trail.performance_metrics.slowest_transformation = 
                    Some((field_path.to_string(), duration));
            }
            None => {
                self.audit_trail.performance_metrics.slowest_transformation = 
                    Some((field_path.to_string(), duration));
            }
            _ => {} // Current slowest is still slower
        }
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

    #[test]
    fn test_track_transformation() {
        let mut tracker = LossinessTracker::new(StrictMode::Warn);
        let mut metadata = HashMap::new();
        metadata.insert("test_key".to_string(), "test_value".to_string());
        
        tracker.track_transformation(
            "$.temperature",
            OperationType::TypeConversion,
            Some(serde_json::json!("98.6")),
            Some(serde_json::json!(98.6)),
            "Converted string to number",
            Some("OpenAI".to_string()),
            metadata,
        );

        assert_eq!(tracker.audit_trail.total_transformations, 1);
        assert_eq!(tracker.audit_trail.records.len(), 1);
        
        let record = &tracker.audit_trail.records[0];
        assert_eq!(record.field_path, "$.temperature");
        assert_eq!(record.operation_type, OperationType::TypeConversion);
        assert_eq!(record.reason, "Converted string to number");
        assert_eq!(record.provider_context, Some("OpenAI".to_string()));
        assert_eq!(record.metadata.get("test_key"), Some(&"test_value".to_string()));
    }

    #[test]
    fn test_track_dropped_field() {
        let mut tracker = LossinessTracker::new(StrictMode::Warn);
        
        tracker.track_dropped_field(
            "$.custom_field",
            Some(serde_json::json!("custom_value")),
            "Field not supported by provider",
            Some("Claude".to_string()),
        );

        assert_eq!(tracker.audit_trail.dropped_fields_count, 1);
        assert_eq!(tracker.audit_trail.total_transformations, 1);
        
        let dropped_fields = tracker.get_dropped_fields();
        assert_eq!(dropped_fields.len(), 1);
        assert_eq!(dropped_fields[0].field_path, "$.custom_field");
        assert_eq!(dropped_fields[0].operation_type, OperationType::Dropped);
    }

    #[test]
    fn test_track_coercion() {
        let mut tracker = LossinessTracker::new(StrictMode::Warn);
        
        tracker.track_coercion(
            "$.max_tokens",
            Some(serde_json::json!(5000)),
            Some(serde_json::json!(4096)),
            "Clamped to provider maximum",
            Some("OpenAI".to_string()),
        );

        assert_eq!(tracker.audit_trail.coercion_count, 1);
        
        let coercions = tracker.get_coercions();
        assert_eq!(coercions.len(), 1);
        assert_eq!(coercions[0].field_path, "$.max_tokens");
        assert_eq!(coercions[0].before_value, Some(serde_json::json!(5000)));
        assert_eq!(coercions[0].after_value, Some(serde_json::json!(4096)));
    }

    #[test]
    fn test_track_default_applied() {
        let mut tracker = LossinessTracker::new(StrictMode::Warn);
        
        tracker.track_default_applied(
            "$.temperature",
            Some(serde_json::json!(1.0)),
            "Applied default temperature",
            Some("Anthropic".to_string()),
        );

        let records = tracker.get_transformations_by_type(OperationType::DefaultApplied);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].field_path, "$.temperature");
        assert_eq!(records[0].after_value, Some(serde_json::json!(1.0)));
    }

    #[test]
    fn test_field_index_and_history() {
        let mut tracker = LossinessTracker::new(StrictMode::Warn);
        
        // Track multiple transformations on the same field
        tracker.track_transformation(
            "$.temperature",
            OperationType::TypeConversion,
            Some(serde_json::json!("1.5")),
            Some(serde_json::json!(1.5)),
            "String to number",
            None,
            HashMap::new(),
        );
        
        tracker.track_transformation(
            "$.temperature",
            OperationType::Coercion,
            Some(serde_json::json!(1.5)),
            Some(serde_json::json!(1.0)),
            "Clamped to valid range",
            None,
            HashMap::new(),
        );

        // Test field change detection
        assert!(tracker.has_field_changed("$.temperature"));
        assert!(!tracker.has_field_changed("$.nonexistent"));

        // Test field history
        let history = tracker.get_field_history("$.temperature");
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].operation_type, OperationType::TypeConversion);
        assert_eq!(history[1].operation_type, OperationType::Coercion);
    }

    #[test]
    fn test_get_transformations_by_type() {
        let mut tracker = LossinessTracker::new(StrictMode::Warn);
        
        tracker.track_transformation(
            "$.field1",
            OperationType::TypeConversion,
            None,
            None,
            "Test conversion 1",
            None,
            HashMap::new(),
        );
        
        tracker.track_transformation(
            "$.field2",
            OperationType::TypeConversion,
            None,
            None,
            "Test conversion 2",
            None,
            HashMap::new(),
        );
        
        tracker.track_transformation(
            "$.field3",
            OperationType::EnumMapping,
            None,
            None,
            "Test enum mapping",
            None,
            HashMap::new(),
        );

        let type_conversions = tracker.get_transformations_by_type(OperationType::TypeConversion);
        assert_eq!(type_conversions.len(), 2);
        
        let enum_mappings = tracker.get_transformations_by_type(OperationType::EnumMapping);
        assert_eq!(enum_mappings.len(), 1);
        
        let unit_conversions = tracker.get_transformations_by_type(OperationType::UnitConversion);
        assert_eq!(unit_conversions.len(), 0);
    }

    #[test]
    fn test_summary_statistics() {
        let mut tracker = LossinessTracker::new(StrictMode::Warn);
        
        tracker.track_transformation(
            "$.field1",
            OperationType::TypeConversion,
            None,
            None,
            "Test",
            None,
            HashMap::new(),
        );
        
        tracker.track_transformation(
            "$.field2",
            OperationType::TypeConversion,
            None,
            None,
            "Test",
            None,
            HashMap::new(),
        );
        
        tracker.track_dropped_field("$.field3", None, "Test drop", None);

        let stats = tracker.get_summary_statistics();
        assert_eq!(stats.total_transformations, 3);
        assert_eq!(stats.affected_fields, 3);
        assert_eq!(stats.dropped_fields, 1);
        assert_eq!(stats.by_operation_type.get("TypeConversion"), Some(&2));
        assert_eq!(stats.by_operation_type.get("Dropped"), Some(&1));
        assert_eq!(stats.most_common_operation, Some("TypeConversion".to_string()));
    }

    #[test]
    fn test_audit_report_generation() {
        let mut tracker = LossinessTracker::new(StrictMode::Warn);
        
        tracker.track_transformation(
            "$.temperature",
            OperationType::TypeConversion,
            Some(serde_json::json!("1.0")),
            Some(serde_json::json!(1.0)),
            "Convert string to number",
            None,
            HashMap::new(),
        );

        let report = tracker.generate_audit_report();
        assert!(report.contains("=== Transformation Audit Report ==="));
        assert!(report.contains("Total Transformations: 1"));
        assert!(report.contains("Affected Fields: 1"));
        assert!(report.contains("TypeConversion Operations"));
        assert!(report.contains("$.temperature"));
        assert!(report.contains("Convert string to number"));
    }

    #[test]
    fn test_json_serialization() {
        let mut tracker = LossinessTracker::new(StrictMode::Warn);
        
        tracker.track_transformation(
            "$.test_field",
            OperationType::Coercion,
            Some(serde_json::json!(100)),
            Some(serde_json::json!(50)),
            "Test coercion",
            Some("test_provider".to_string()),
            HashMap::new(),
        );

        let json_result = tracker.to_json();
        assert!(json_result.is_ok());
        
        let json_str = json_result.unwrap();
        assert!(json_str.contains("test_field"));
        assert!(json_str.contains("test_provider"));
        assert!(json_str.contains("Test coercion"));
    }

    #[test]
    fn test_performance_report() {
        let mut tracker = LossinessTracker::new(StrictMode::Warn);
        
        // Simulate some transformations
        tracker.track_transformation(
            "$.field1",
            OperationType::TypeConversion,
            None,
            None,
            "Test",
            None,
            HashMap::new(),
        );

        let report = tracker.get_performance_report();
        assert_eq!(report.total_time_ms, 0); // No actual timing recorded in these tests
        assert_eq!(report.avg_time_per_transformation_ms, 0.0);
        assert_eq!(report.slowest_operation, None);
        assert_eq!(report.operations_by_duration.len(), 0);
    }

    #[test]
    fn test_track_transformation_with_timing() {
        let mut tracker = LossinessTracker::new(StrictMode::Warn);
        
        // Test the timing wrapper
        let result = tracker.track_transformation_with_timing(
            "$.test_field",
            OperationType::TypeConversion,
            Some(serde_json::json!("test")),
            Some(serde_json::json!(42)),
            "Test with timing",
            None,
            HashMap::new(),
            || {
                // Simulate some work
                std::thread::sleep(std::time::Duration::from_millis(1));
                "operation_result"
            }
        );

        assert_eq!(result, "operation_result");
        assert_eq!(tracker.audit_trail.total_transformations, 1);
        
        // Check that timing was recorded
        assert!(tracker.audit_trail.performance_metrics.transformation_times
            .contains_key("$.test_field"));
        assert!(tracker.audit_trail.performance_metrics.slowest_transformation.is_some());
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that existing functionality still works
        let mut tracker = LossinessTracker::new(StrictMode::Strict);
        
        // Use existing methods
        tracker.add_clamped("temp", "msg", None, None);
        tracker.add_dropped("field", "msg", None);
        tracker.add_unsupported("feature", "msg", None);
        
        // Build report should still work
        let report = tracker.build_report();
        assert_eq!(report.items.len(), 3);
        assert!(report.max_severity >= Severity::Warning);
        
        // Existing query methods should still work
        assert_eq!(report.summary.total_items, 3);
        assert!(report.summary.by_code.contains_key("Clamp"));
        assert!(report.summary.by_code.contains_key("Drop"));
        assert!(report.summary.by_code.contains_key("Unsupported"));
    }

    #[test]
    fn test_operation_type_serialization() {
        let op_types = vec![
            OperationType::TypeConversion,
            OperationType::EnumMapping,
            OperationType::UnitConversion,
            OperationType::Coercion,
            OperationType::Dropped,
            OperationType::FieldMove,
            OperationType::DefaultApplied,
        ];

        for op_type in op_types {
            let json = serde_json::to_string(&op_type).unwrap();
            let deserialized: OperationType = serde_json::from_str(&json).unwrap();
            assert_eq!(op_type, deserialized);
        }
    }

    #[test]
    fn test_complex_audit_trail() {
        let mut tracker = LossinessTracker::new(StrictMode::Warn);
        
        // Simulate a complex transformation scenario
        tracker.track_transformation(
            "$.messages[0].content",
            OperationType::TypeConversion,
            Some(serde_json::json!({"text": "Hello"})),
            Some(serde_json::json!("Hello")),
            "Converted object to string",
            Some("OpenAI".to_string()),
            HashMap::new(),
        );
        
        tracker.track_dropped_field(
            "$.messages[0].metadata",
            Some(serde_json::json!({"custom": "data"})),
            "Metadata not supported",
            Some("OpenAI".to_string()),
        );
        
        tracker.track_coercion(
            "$.max_tokens",
            Some(serde_json::json!(5000)),
            Some(serde_json::json!(4096)),
            "Exceeded provider limit",
            Some("OpenAI".to_string()),
        );
        
        tracker.track_default_applied(
            "$.temperature",
            Some(serde_json::json!(1.0)),
            "Missing temperature, applied default",
            Some("OpenAI".to_string()),
        );

        // Verify comprehensive tracking
        assert_eq!(tracker.audit_trail.total_transformations, 4);
        assert_eq!(tracker.audit_trail.dropped_fields_count, 1);
        assert_eq!(tracker.audit_trail.coercion_count, 1);
        
        let stats = tracker.get_summary_statistics();
        assert_eq!(stats.total_transformations, 4);
        assert_eq!(stats.affected_fields, 4);
        assert_eq!(stats.dropped_fields, 1);
        
        // Test audit report
        let report = tracker.generate_audit_report();
        assert!(report.contains("Total Transformations: 4"));
        assert!(report.contains("Affected Fields: 4"));
        assert!(report.contains("Dropped Fields: 1"));
    }
}
