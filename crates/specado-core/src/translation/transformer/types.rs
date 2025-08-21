//! Core types for field transformation system
//!
//! This module defines the fundamental types used throughout the transformation system,
//! including error types, transformation types, value types, and context structures.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::Result;
use super::super::jsonpath::JSONPathError;
use super::super::TranslationContext;
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during field transformations
#[derive(Error, Debug, Clone)]
pub enum TransformationError {
    /// Type conversion failed
    #[error("Type conversion failed: cannot convert {from} to {to} for value: {value}")]
    TypeConversion {
        from: String,
        to: String,
        value: String,
        path: String,
    },

    /// Enum mapping not found
    #[error("Enum mapping not found: {value} at {path}")]
    EnumMapping {
        value: String,
        path: String,
        available_mappings: Vec<String>,
    },

    /// Unit conversion failed
    #[error("Unit conversion failed: {message} at {path}")]
    UnitConversion {
        message: String,
        path: String,
        from_unit: String,
        to_unit: String,
    },

    /// Condition evaluation failed
    #[error("Condition evaluation failed: {message} at {path}")]
    ConditionEvaluation {
        message: String,
        path: String,
        condition: String,
    },

    /// JSONPath error during transformation
    #[error("JSONPath error during transformation: {source}")]
    JSONPath {
        #[from]
        source: JSONPathError,
    },

    /// Invalid transformation configuration
    #[error("Invalid transformation configuration: {message}")]
    Configuration {
        message: String,
        rule_id: Option<String>,
    },
}

impl From<TransformationError> for crate::Error {
    fn from(err: TransformationError) -> Self {
        crate::Error::Translation {
            message: err.to_string(),
            context: Some("Field Transformation".to_string()),
        }
    }
}

/// Supported value types for conversions
#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Null,
}

/// Conversion formulas for unit transformations
#[derive(Debug, Clone, PartialEq)]
pub enum ConversionFormula {
    /// Linear formula: output = input * scale + offset
    Linear { scale: f64, offset: f64 },
    /// Custom function for complex conversions
    Custom(String),
}

/// Condition for conditional transformations
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    /// Check if field equals a value
    Equals { path: String, value: Value },
    /// Check if field exists
    Exists { path: String },
    /// Check if field matches a pattern
    Matches { path: String, pattern: String },
    /// Logical AND of conditions
    And(Vec<Condition>),
    /// Logical OR of conditions
    Or(Vec<Condition>),
    /// Logical NOT of a condition
    Not(Box<Condition>),
}

/// A function that can transform a JSON value
pub type TransformerFunction = fn(&Value, &TransformationContext) -> Result<Value>;

/// Type of transformation to apply
#[derive(Debug, Clone)]
pub enum TransformationType {
    /// Convert between primitive types (string, number, boolean)
    TypeConversion {
        from: ValueType,
        to: ValueType,
    },
    /// Map enum values using a lookup table
    EnumMapping {
        mappings: HashMap<String, String>,
        default: Option<String>,
    },
    /// Convert between units (e.g., temperature scaling)
    UnitConversion {
        from_unit: String,
        to_unit: String,
        formula: ConversionFormula,
    },
    /// Rename or restructure fields
    FieldRename {
        new_name: String,
    },
    /// Inject default values if field is missing
    DefaultValue {
        value: Value,
    },
    /// Apply transformation conditionally
    Conditional {
        condition: Condition,
        if_true: Box<TransformationType>,
        if_false: Option<Box<TransformationType>>,
    },
    /// Custom transformation using a closure
    Custom {
        name: String,
        transformer: TransformerFunction,
    },
}

impl PartialEq for TransformationType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::TypeConversion { from: f1, to: t1 }, Self::TypeConversion { from: f2, to: t2 }) => {
                f1 == f2 && t1 == t2
            }
            (Self::EnumMapping { mappings: m1, default: d1 }, Self::EnumMapping { mappings: m2, default: d2 }) => {
                m1 == m2 && d1 == d2
            }
            (Self::UnitConversion { from_unit: f1, to_unit: t1, formula: fo1 }, 
             Self::UnitConversion { from_unit: f2, to_unit: t2, formula: fo2 }) => {
                f1 == f2 && t1 == t2 && fo1 == fo2
            }
            (Self::FieldRename { new_name: n1 }, Self::FieldRename { new_name: n2 }) => {
                n1 == n2
            }
            (Self::DefaultValue { value: v1 }, Self::DefaultValue { value: v2 }) => {
                v1 == v2
            }
            (Self::Conditional { condition: c1, if_true: t1, if_false: f1 }, 
             Self::Conditional { condition: c2, if_true: t2, if_false: f2 }) => {
                c1 == c2 && t1 == t2 && f1 == f2
            }
            (Self::Custom { name: n1, .. }, Self::Custom { name: n2, .. }) => {
                // Compare only by name for function pointers
                n1 == n2
            }
            _ => false,
        }
    }
}

/// Context information available to transformers
#[derive(Debug, Clone)]
pub struct TransformationContext {
    /// Original data being transformed
    pub source_data: Value,
    /// Target data being built
    pub target_data: Value,
    /// Translation context with provider info
    pub translation_context: TranslationContext,
    /// Path where transformation is being applied
    pub current_path: String,
    /// Additional metadata
    pub metadata: HashMap<String, Value>,
}

/// Direction of transformation
#[derive(Debug, Clone, PartialEq)]
pub enum TransformationDirection {
    /// Uniform to provider (forward)
    Forward,
    /// Provider to uniform (reverse)
    Reverse,
    /// Both directions
    Bidirectional,
}

/// A single transformation rule
#[derive(Debug, Clone)]
pub struct TransformationRule {
    /// Unique identifier for this rule
    pub id: String,
    /// JSONPath to select source fields
    pub source_path: String,
    /// JSONPath to place transformed fields (optional, defaults to source_path)
    pub target_path: Option<String>,
    /// Type of transformation to apply
    pub transformation: TransformationType,
    /// Direction of transformation (forward, reverse, or both)
    pub direction: TransformationDirection,
    /// Priority for rule ordering (higher numbers execute first)
    pub priority: i32,
    /// Whether this rule is optional (continues on error)
    pub optional: bool,
}