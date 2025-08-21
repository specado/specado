//! Statistics and reporting structures for lossiness tracking
//!
//! This module provides data structures for generating summaries and performance reports
//! from lossiness tracking data.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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