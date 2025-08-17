// Tests for enhanced output formatting functionality
//
// These tests verify that the new formatting capabilities work correctly
// for TranslationResult, ValidationErrors, and other specialized types.

use super::*;
use specado_core::types::{LossinessItem, LossinessSummary, TranslationMetadata};
use specado_core::{LossinessCode, Severity, StrictMode};
use specado_schemas::validation::{ValidationError, ValidationErrors, Violation};
use std::collections::HashMap;

#[test]
fn test_translation_result_formatting_human() {
    let mut summary_by_severity = HashMap::new();
    summary_by_severity.insert("Warning".to_string(), 2);
    summary_by_severity.insert("Info".to_string(), 1);

    let mut summary_by_code = HashMap::new();
    summary_by_code.insert("CLAMP".to_string(), 1);
    summary_by_code.insert("UNSUPPORTED".to_string(), 2);

    let lossiness_report = LossinessReport {
        items: vec![
            LossinessItem {
                code: LossinessCode::Clamp,
                path: "$.sampling.temperature".to_string(),
                message: "Temperature value coerced from 1.5 to 1.0".to_string(),
                severity: Severity::Warning,
                before: Some(serde_json::json!(1.5)),
                after: Some(serde_json::json!(1.0)),
            },
            LossinessItem {
                code: LossinessCode::Unsupported,
                path: "$.tools".to_string(),
                message: "Tool calls not supported by provider".to_string(),
                severity: Severity::Info,
                before: Some(serde_json::json!({"name": "test_tool"})),
                after: None,
            },
        ],
        max_severity: Severity::Warning,
        summary: LossinessSummary {
            total_items: 3,
            by_severity: summary_by_severity,
            by_code: summary_by_code,
        },
    };

    let metadata = TranslationMetadata {
        provider: "openai".to_string(),
        model: "gpt-4".to_string(),
        timestamp: "2025-01-17T10:30:00Z".to_string(),
        duration_ms: Some(150),
        strict_mode: StrictMode::Warn,
    };

    let translation_result = TranslationResult {
        provider_request_json: serde_json::json!({
            "model": "gpt-4",
            "messages": [{"role": "user", "content": "Hello"}]
        }),
        lossiness: lossiness_report,
        metadata: Some(metadata),
    };

    let result = format_translation_result_human(&translation_result);
    assert!(result.is_ok());

    let formatted = result.unwrap();
    assert!(formatted.contains("═══ Translation Result ═══"));
    assert!(formatted.contains("🔧 Translation Details:"));
    assert!(formatted.contains("Provider: openai"));
    assert!(formatted.contains("Model: gpt-4"));
    assert!(formatted.contains("Duration: 150ms"));
    assert!(formatted.contains("🔍 Lossiness Summary:"));
    assert!(formatted.contains("Total Issues: 3"));
    assert!(formatted.contains("⚠️ Warning: 2"));
    assert!(formatted.contains("ℹ️ Info: 1"));
    assert!(formatted.contains("📝 Provider Request:"));
}

#[test]
fn test_translation_result_formatting_perfect() {
    let translation_result = TranslationResult {
        provider_request_json: serde_json::json!({
            "model": "gpt-4",
            "messages": [{"role": "user", "content": "Hello"}]
        }),
        lossiness: LossinessReport {
            items: vec![],
            max_severity: Severity::Info,
            summary: LossinessSummary {
                total_items: 0,
                by_severity: HashMap::new(),
                by_code: HashMap::new(),
            },
        },
        metadata: None,
    };

    let result = format_translation_result_human(&translation_result);
    assert!(result.is_ok());

    let formatted = result.unwrap();
    assert!(formatted.contains("✅ No lossiness detected - perfect translation"));
}

#[test]
fn test_validation_errors_formatting_human() {
    let violations = vec![
        Violation {
            rule: "required".to_string(),
            expected: "property 'model_class'".to_string(),
            actual: "missing".to_string(),
        },
        Violation {
            rule: "type".to_string(),
            expected: "string".to_string(),
            actual: "number".to_string(),
        },
    ];

    let error = ValidationError::with_violations(
        "$.model_class",
        "Required property missing",
        violations,
    );

    let errors = ValidationErrors::from(vec![error]);

    let result = format_validation_errors_human(&errors);
    assert!(result.is_ok());

    let formatted = result.unwrap();
    assert!(formatted.contains("❌ Validation Failed - 1 Error(s)"));
    assert!(formatted.contains("📍 Path: $.model_class"));
    assert!(formatted.contains("💬 Message: Required property missing"));
    assert!(formatted.contains("🔍 Schema Violations:"));
    assert!(formatted.contains("• Rule: required"));
    assert!(formatted.contains("Expected: property 'model_class'"));
    assert!(formatted.contains("Actual: missing"));
}

#[test]
fn test_lossiness_report_formatting_human() {
    let mut summary_by_severity = HashMap::new();
    summary_by_severity.insert("Error".to_string(), 1);
    summary_by_severity.insert("Warning".to_string(), 1);

    let mut summary_by_code = HashMap::new();
    summary_by_code.insert("UNSUPPORTED".to_string(), 1);
    summary_by_code.insert("CLAMP".to_string(), 1);

    let report = LossinessReport {
        items: vec![
            LossinessItem {
                code: LossinessCode::Unsupported,
                path: "$.tools".to_string(),
                message: "Tool calls not supported".to_string(),
                severity: Severity::Error,
                before: Some(serde_json::json!({"tools": ["function1"]})),
                after: None,
            },
            LossinessItem {
                code: LossinessCode::Clamp,
                path: "$.temperature".to_string(),
                message: "Temperature coerced to valid range".to_string(),
                severity: Severity::Warning,
                before: Some(serde_json::json!(2.5)),
                after: Some(serde_json::json!(2.0)),
            },
        ],
        max_severity: Severity::Error,
        summary: LossinessSummary {
            total_items: 2,
            by_severity: summary_by_severity,
            by_code: summary_by_code,
        },
    };

    let result = format_lossiness_report_human(&report);
    assert!(result.is_ok());

    let formatted = result.unwrap();
    assert!(formatted.contains("🔍 Lossiness Report - 2 Issue(s)"));
    assert!(formatted.contains("📊 Summary by Severity:"));
    assert!(formatted.contains("❌ Error: 1"));
    assert!(formatted.contains("⚠️ Warning: 1"));
    assert!(formatted.contains("📋 Summary by Type:"));
    assert!(formatted.contains("• UNSUPPORTED: 1"));
    assert!(formatted.contains("• CLAMP: 1"));
    assert!(formatted.contains("❌ Error Issues:"));
    assert!(formatted.contains("⚠️ Warning Issues:"));
    assert!(formatted.contains("📍 Path: $.tools"));
    assert!(formatted.contains("🏷️  Code: Unsupported"));
    assert!(formatted.contains("💬 Message: Tool calls not supported"));
    assert!(formatted.contains("📥 Before:"));
    assert!(formatted.contains("📤 After:"));
}

#[test]
fn test_lossiness_report_empty() {
    let report = LossinessReport {
        items: vec![],
        max_severity: Severity::Info,
        summary: LossinessSummary {
            total_items: 0,
            by_severity: HashMap::new(),
            by_code: HashMap::new(),
        },
    };

    let result = format_lossiness_report_human(&report);
    assert!(result.is_ok());

    let formatted = result.unwrap();
    assert!(formatted.contains("✅ No lossiness detected"));
}

#[test]
fn test_format_value_compact() {
    // Test string
    assert_eq!(format_value_compact(&serde_json::json!("hello")), "\"hello\"");

    // Test number
    assert_eq!(format_value_compact(&serde_json::json!(42)), "42");

    // Test boolean
    assert_eq!(format_value_compact(&serde_json::json!(true)), "true");

    // Test null
    assert_eq!(format_value_compact(&serde_json::json!(null)), "null");

    // Test small array
    assert_eq!(
        format_value_compact(&serde_json::json!([1, 2, 3])),
        "[1, 2, 3]"
    );

    // Test large array
    assert_eq!(
        format_value_compact(&serde_json::json!([1, 2, 3, 4, 5])),
        "[5 items]"
    );

    // Test small object
    assert_eq!(
        format_value_compact(&serde_json::json!({"a": 1, "b": 2})),
        "{a: 1, b: 2}"
    );

    // Test large object
    assert_eq!(
        format_value_compact(&serde_json::json!({"a": 1, "b": 2, "c": 3})),
        "{3 fields}"
    );
}

#[test]
fn test_output_writer_creation() {
    let writer = OutputWriter::new(OutputFormat::Human, true, false, 1);
    assert_eq!(writer.format(), OutputFormat::Human);
    assert_eq!(writer.verbosity(), 1);
    assert!(writer.is_verbose());
}

#[test]
fn test_output_formatter_trait() {
    let formatter = OutputFormat::Human;
    
    // Test basic formatting
    let simple_data = serde_json::json!({"test": "value"});
    let result = formatter.format(&simple_data);
    assert!(result.is_ok());

    // Test translation result formatting
    let translation_result = TranslationResult {
        provider_request_json: serde_json::json!({"model": "test"}),
        lossiness: LossinessReport {
            items: vec![],
            max_severity: Severity::Info,
            summary: LossinessSummary {
                total_items: 0,
                by_severity: HashMap::new(),
                by_code: HashMap::new(),
            },
        },
        metadata: None,
    };

    let result = formatter.format_translation_result(&translation_result);
    assert!(result.is_ok());
}