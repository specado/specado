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
pub fn openai_to_anthropic_models() -> TransformationType {
    let mut mappings = HashMap::new();
    mappings.insert("gpt-5".to_string(), "claude-opus-4-1-20250805".to_string());
    mappings.insert("gpt-5-mini".to_string(), "claude-3-sonnet-20240229".to_string());
    mappings.insert("gpt-3.5-turbo".to_string(), "claude-3-haiku-20240307".to_string());

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