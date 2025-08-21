//! Reporting and formatting utilities for lossiness tracking
//!
//! This module provides functionality for generating human-readable reports
//! and converting tracking data to various formats.

use crate::translation::lossiness::statistics::{PerformanceReport, SummaryStats};
use crate::translation::lossiness::types::AuditTrail;
use std::collections::HashMap;

/// Generate a human-readable audit report from audit trail data
pub fn generate_audit_report(audit_trail: &AuditTrail, stats: &SummaryStats) -> String {
    let mut report = String::new();
    
    report.push_str("=== Transformation Audit Report ===\n\n");
    
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
        
        let matching_records: Vec<_> = audit_trail
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

/// Generate summary statistics from audit trail data
pub fn generate_summary_statistics(audit_trail: &AuditTrail) -> SummaryStats {
    let mut by_operation_type = HashMap::new();
    let mut affected_fields = std::collections::HashSet::new();

    for record in &audit_trail.records {
        let op_type = format!("{:?}", record.operation_type);
        *by_operation_type.entry(op_type).or_insert(0) += 1;
        affected_fields.insert(record.field_path.clone());
    }

    // Calculate average time if we have timing data
    let avg_transformation_time_ms = if audit_trail.total_transformations > 0 {
        audit_trail.performance_metrics.total_duration.as_millis() as f64
            / audit_trail.total_transformations as f64
    } else {
        0.0
    };

    let most_common_operation = by_operation_type
        .iter()
        .max_by_key(|(_, &count)| count)
        .map(|(op_type, _)| op_type.clone());

    SummaryStats {
        total_transformations: audit_trail.total_transformations,
        by_operation_type,
        affected_fields: affected_fields.len(),
        dropped_fields: audit_trail.dropped_fields_count,
        avg_transformation_time_ms,
        most_common_operation,
    }
}

/// Generate performance report from audit trail data
pub fn generate_performance_report(audit_trail: &AuditTrail) -> PerformanceReport {
    let total_time_ms = audit_trail.performance_metrics.total_duration.as_millis() as u64;
    
    let avg_time_per_transformation_ms = if audit_trail.total_transformations > 0 {
        total_time_ms as f64 / audit_trail.total_transformations as f64
    } else {
        0.0
    };

    let slowest_operation = audit_trail.performance_metrics.slowest_transformation
        .as_ref()
        .map(|(path, duration)| (path.clone(), duration.as_millis() as u64));

    let mut operations_by_duration: Vec<_> = audit_trail.performance_metrics.transformation_times
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

/// Convert audit trail to JSON
pub fn audit_trail_to_json(audit_trail: &AuditTrail) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(audit_trail)
}