//! Comprehensive tests for the lossiness tracking module
//!
//! This module contains all the tests from the original lossiness.rs file,
//! ensuring that the refactored modules maintain all existing functionality.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::{LossinessCode, Severity, StrictMode};
    use std::collections::HashMap;

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

        let stats = tracker.get_summary_statistics();
        assert_eq!(stats.total_transformations, 1);
        
        let field_history = tracker.get_field_history("$.temperature");
        assert_eq!(field_history.len(), 1);
        
        let record = &field_history[0];
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

        let stats = tracker.get_summary_statistics();
        assert_eq!(stats.dropped_fields, 1);
        assert_eq!(stats.total_transformations, 1);
        
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

        let stats = tracker.get_summary_statistics();
        assert_eq!(stats.total_transformations, 1);
        
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
        let stats = tracker.get_summary_statistics();
        assert_eq!(stats.total_transformations, 1);
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