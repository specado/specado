//! Constraint and provider-specific validation logic
//!
//! This module contains validation functions for provider-specific constraints,
//! mutually exclusive fields, and other cross-field validation rules.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::Result;
use super::{ValidationError, ValidationSeverity};
use super::super::TranslationContext;

/// Validate provider-specific constraints
pub fn validate_provider_constraints(context: &TranslationContext) -> Result<Vec<ValidationError>> {
    let errors = Vec::new();
    
    // Check for unknown top-level fields if provider forbids them
    if context.model_spec.constraints.forbid_unknown_top_level_fields {
        // This would require introspection of the JSON structure
        // For now, we'll add a placeholder that could be implemented
        // when we have more detailed provider specifications
    }
    
    Ok(errors)
}

/// Validate mutually exclusive field combinations
pub fn validate_mutually_exclusive_fields(context: &TranslationContext) -> Result<Vec<ValidationError>> {
    let mut errors = Vec::new();
    
    for exclusive_group in &context.model_spec.constraints.mutually_exclusive {
        let present_fields: Vec<_> = exclusive_group.iter()
            .filter(|field| is_field_present(context, field))
            .collect();
        
        if present_fields.len() > 1 {
            let present_str = present_fields.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
            let expected_str = exclusive_group.join(", ");
            
            errors.push(ValidationError {
                field_path: "constraint_violation".to_string(),
                message: format!(
                    "Mutually exclusive fields detected: {}",
                    present_str
                ),
                expected: Some(format!("Only one of: {}", expected_str)),
                actual: Some(format!("Present: {}", present_str)),
                severity: ValidationSeverity::Error,
            });
        }
    }
    
    Ok(errors)
}

/// Check if a field is present in the prompt spec
fn is_field_present(context: &TranslationContext, field_path: &str) -> bool {
    match field_path {
        "tools" => context.prompt_spec.tools.is_some(),
        "tool_choice" => context.prompt_spec.tool_choice.is_some(),
        "response_format" => context.prompt_spec.response_format.is_some(),
        "sampling" => context.prompt_spec.sampling.is_some(),
        "limits" => context.prompt_spec.limits.is_some(),
        "media" => context.prompt_spec.media.is_some(),
        _ => false, // For more complex paths, we'd need JSONPath evaluation
    }
}