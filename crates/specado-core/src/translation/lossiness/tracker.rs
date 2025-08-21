//! Core lossiness tracker implementation
//!
//! This module contains the main `LossinessTracker` struct and its implementation,
//! providing comprehensive tracking of transformations and lossiness items.

use crate::translation::lossiness::types::TransformationRecord;
use crate::translation::lossiness::reporting::{
    audit_trail_to_json, generate_audit_report, generate_performance_report, generate_summary_statistics
};
use crate::translation::lossiness::statistics::{PerformanceReport, SummaryStats};
use crate::translation::lossiness::types::{AuditTrail, OperationType};
use crate::{LossinessCode, LossinessItem, LossinessReport, LossinessSummary, Severity, StrictMode};
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};

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
    #[allow(clippy::too_many_arguments)]
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
    #[allow(clippy::too_many_arguments)]
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
        let record = TransformationRecord::new(
            field_path,
            operation_type.clone(),
            before_value,
            after_value,
            reason,
            provider_context,
            metadata,
        );

        let record_index = self.audit_trail.records.len();
        self.audit_trail.records.push(record);
        
        // Update field index
        self.field_index
            .entry(field_path.to_string())
            .or_default()
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
        let record = TransformationRecord::new_dropped_field(
            field_path,
            original_value,
            reason,
            provider_context,
        );
        
        let record_index = self.audit_trail.records.len();
        self.audit_trail.records.push(record);
        
        // Update field index
        self.field_index
            .entry(field_path.to_string())
            .or_default()
            .push(record_index);

        // Update counters
        self.audit_trail.total_transformations += 1;
        self.audit_trail.dropped_fields_count += 1;
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
        let record = TransformationRecord::new_coercion(
            field_path,
            original_value,
            coerced_value,
            reason,
            provider_context,
        );
        
        let record_index = self.audit_trail.records.len();
        self.audit_trail.records.push(record);
        
        // Update field index
        self.field_index
            .entry(field_path.to_string())
            .or_default()
            .push(record_index);

        // Update counters
        self.audit_trail.total_transformations += 1;
        self.audit_trail.coercion_count += 1;
    }

    /// Track when a default value is applied
    pub fn track_default_applied(
        &mut self,
        field_path: &str,
        default_value: Option<Value>,
        reason: &str,
        provider_context: Option<String>,
    ) {
        let record = TransformationRecord::new_default_applied(
            field_path,
            default_value,
            reason,
            provider_context,
        );
        
        let record_index = self.audit_trail.records.len();
        self.audit_trail.records.push(record);
        
        // Update field index
        self.field_index
            .entry(field_path.to_string())
            .or_default()
            .push(record_index);

        // Update counters
        self.audit_trail.total_transformations += 1;
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
        crate::translation::lossiness::operations::get_dropped_fields(&self.audit_trail.records)
    }

    /// Get all coercions
    pub fn get_coercions(&self) -> Vec<&TransformationRecord> {
        crate::translation::lossiness::operations::get_coercions(&self.audit_trail.records)
    }

    /// Get transformations by operation type
    pub fn get_transformations_by_type(&self, op_type: OperationType) -> Vec<&TransformationRecord> {
        crate::translation::lossiness::operations::filter_by_operation_type(&self.audit_trail.records, op_type)
    }

    /// Check if a field has been changed/transformed
    pub fn has_field_changed(&self, path: &str) -> bool {
        self.field_index.contains_key(path)
    }

    /// Generate a human-readable audit report
    pub fn generate_audit_report(&self) -> String {
        let stats = self.get_summary_statistics();
        generate_audit_report(&self.audit_trail, &stats)
    }

    /// Convert audit trail to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        audit_trail_to_json(&self.audit_trail)
    }

    /// Get summary statistics
    pub fn get_summary_statistics(&self) -> SummaryStats {
        generate_summary_statistics(&self.audit_trail)
    }

    /// Get performance report
    pub fn get_performance_report(&self) -> PerformanceReport {
        generate_performance_report(&self.audit_trail)
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