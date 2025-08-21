//! Operation tracking and management for lossiness tracking
//!
//! This module provides functionality for tracking specific types of operations
//! such as dropped fields, coercions, and defaults.

use crate::translation::lossiness::types::{OperationType, TransformationRecord};
use serde_json::Value;
use std::collections::HashMap;

impl TransformationRecord {
    /// Create a new transformation record for a dropped field
    pub fn new_dropped_field(
        field_path: &str,
        original_value: Option<Value>,
        reason: &str,
        provider_context: Option<String>,
    ) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("action".to_string(), "field_dropped".to_string());
        
        Self {
            field_path: field_path.to_string(),
            operation_type: OperationType::Dropped,
            before_value: original_value,
            after_value: None,
            reason: reason.to_string(),
            timestamp: chrono::Utc::now(),
            metadata,
            provider_context,
        }
    }

    /// Create a new transformation record for a value coercion
    pub fn new_coercion(
        field_path: &str,
        original_value: Option<Value>,
        coerced_value: Option<Value>,
        reason: &str,
        provider_context: Option<String>,
    ) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("action".to_string(), "value_coerced".to_string());
        
        Self {
            field_path: field_path.to_string(),
            operation_type: OperationType::Coercion,
            before_value: original_value,
            after_value: coerced_value,
            reason: reason.to_string(),
            timestamp: chrono::Utc::now(),
            metadata,
            provider_context,
        }
    }

    /// Create a new transformation record for a default value application
    pub fn new_default_applied(
        field_path: &str,
        default_value: Option<Value>,
        reason: &str,
        provider_context: Option<String>,
    ) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("action".to_string(), "default_applied".to_string());
        
        Self {
            field_path: field_path.to_string(),
            operation_type: OperationType::DefaultApplied,
            before_value: None,
            after_value: default_value,
            reason: reason.to_string(),
            timestamp: chrono::Utc::now(),
            metadata,
            provider_context,
        }
    }

    /// Create a generic transformation record
    pub fn new(
        field_path: &str,
        operation_type: OperationType,
        before_value: Option<Value>,
        after_value: Option<Value>,
        reason: &str,
        provider_context: Option<String>,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            field_path: field_path.to_string(),
            operation_type,
            before_value,
            after_value,
            reason: reason.to_string(),
            timestamp: chrono::Utc::now(),
            metadata,
            provider_context,
        }
    }
}

/// Helper functions for filtering operations
pub fn filter_by_operation_type(
    records: &[TransformationRecord], 
    op_type: OperationType
) -> Vec<&TransformationRecord> {
    records
        .iter()
        .filter(|record| record.operation_type == op_type)
        .collect()
}

/// Get all dropped fields from a collection of records
pub fn get_dropped_fields(records: &[TransformationRecord]) -> Vec<&TransformationRecord> {
    filter_by_operation_type(records, OperationType::Dropped)
}

/// Get all coercions from a collection of records
pub fn get_coercions(records: &[TransformationRecord]) -> Vec<&TransformationRecord> {
    filter_by_operation_type(records, OperationType::Coercion)
}