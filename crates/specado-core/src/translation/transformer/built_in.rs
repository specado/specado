//! Built-in transformers for common operations
//!
//! This module provides pre-configured transformations for common use cases
//! such as type conversions, model mappings, and unit conversions.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use super::types::{TransformationType, ValueType, ConversionFormula};
use serde_json::Value;
use std::collections::HashMap;

/// Create a string to number conversion
pub fn string_to_number() -> TransformationType {
    TransformationType::TypeConversion {
        from: ValueType::String,
        to: ValueType::Number,
    }
}

/// Create a number to string conversion
pub fn number_to_string() -> TransformationType {
    TransformationType::TypeConversion {
        from: ValueType::Number,
        to: ValueType::String,
    }
}

/// Create a string to boolean conversion
pub fn string_to_boolean() -> TransformationType {
    TransformationType::TypeConversion {
        from: ValueType::String,
        to: ValueType::Boolean,
    }
}

/// Create a boolean to string conversion
pub fn boolean_to_string() -> TransformationType {
    TransformationType::TypeConversion {
        from: ValueType::Boolean,
        to: ValueType::String,
    }
}

/// Create an OpenAI to Anthropic model mapping
/// 
/// Note: This function creates hardcoded model mappings and should be avoided in favor
/// of spec-driven model selection. Consider using provider discovery and capability
/// matching instead of these static mappings.
#[deprecated(note = "Use spec-driven model selection instead of hardcoded mappings")]
pub fn openai_to_anthropic_models() -> TransformationType {
    // These mappings are deprecated in favor of spec-driven model selection
    // based on capabilities and provider specifications
    // Only include mappings for backwards compatibility in test/development scenarios
    let mappings = {
        #[cfg(any(test, feature = "legacy-mappings"))]
        {
            let mut m = HashMap::new();
            m.insert("gpt-5".to_string(), "claude-opus-4-1-20250805".to_string());
            m.insert("gpt-5-mini".to_string(), "claude-3-sonnet-20240229".to_string());
            m.insert("gpt-3.5-turbo".to_string(), "claude-3-haiku-20240307".to_string());
            m
        }
        #[cfg(not(any(test, feature = "legacy-mappings")))]
        HashMap::new()
    };

    TransformationType::EnumMapping {
        mappings,
        default: Some("claude-3-haiku-20240307".to_string()),
    }
}

/// Create a temperature converter (0-2 to 0-1 range)
pub fn temperature_0_2_to_0_1() -> TransformationType {
    TransformationType::UnitConversion {
        from_unit: "openai_temp".to_string(),
        to_unit: "anthropic_temp".to_string(),
        formula: ConversionFormula::Linear { scale: 0.5, offset: 0.0 },
    }
}

/// Create a temperature converter (0-1 to 0-2 range)
pub fn temperature_0_1_to_0_2() -> TransformationType {
    TransformationType::UnitConversion {
        from_unit: "anthropic_temp".to_string(),
        to_unit: "openai_temp".to_string(),
        formula: ConversionFormula::Linear { scale: 2.0, offset: 0.0 },
    }
}

/// Create a default value injection
pub fn default_value(value: Value) -> TransformationType {
    TransformationType::DefaultValue { value }
}

/// Create a field rename transformation
pub fn rename_field(new_name: impl Into<String>) -> TransformationType {
    TransformationType::FieldRename {
        new_name: new_name.into(),
    }
}