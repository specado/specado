//! Core types and enums for lossiness tracking
//!
//! This module defines the fundamental types used throughout the lossiness tracking system,
//! including operation types and transformation records.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

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