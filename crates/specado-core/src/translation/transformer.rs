//! Field transformation system for translating values between uniform and provider formats
//!
//! This module provides a comprehensive field transformation system that uses the
//! JSONPath engine to locate fields and apply transformations such as type conversions,
//! enum mappings, unit conversions, and conditional transformations.
//!
//! # Examples
//!
//! ## Basic Type Conversion
//!
//! ```
//! use specado_core::{TransformationPipeline, TransformationRuleBuilder, TransformationDirection};
//! use specado_core::translation::transformer::built_in;
//! use serde_json::json;
//!
//! let mut pipeline = TransformationPipeline::new();
//!
//! // Convert string temperature to number
//! let temp_rule = TransformationRuleBuilder::new("temp_convert", "$.temperature")
//!     .transformation(built_in::string_to_number())
//!     .build()
//!     .unwrap();
//!
//! pipeline = pipeline.add_rule(temp_rule);
//!
//! let input = json!({
//!     "temperature": "0.7"
//! });
//!
//! // Note: This example requires a TranslationContext for the transform call
//! // See the tests for complete usage examples
//! ```
//!
//! ## Model Mapping
//!
//! ```
//! use specado_core::translation::transformer::built_in;
//! use serde_json::json;
//!
//! // Map OpenAI model names to Anthropic equivalents
//! let model_transform = built_in::openai_to_anthropic_models();
//! ```
//!
//! ## Temperature Scaling
//!
//! ```
//! use specado_core::translation::transformer::built_in;
//!
//! // Convert temperature from 0-2 range (OpenAI) to 0-1 range (Anthropic)
//! let temp_transform = built_in::temperature_0_2_to_0_1();
//! ```
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::Result;
use super::jsonpath::{JSONPath, JSONPathError};
use super::lossiness::{LossinessTracker, OperationType};
use super::TranslationContext;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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

/// A pipeline of transformation rules
#[derive(Debug, Clone)]
pub struct TransformationPipeline {
    /// Ordered list of transformation rules
    rules: Vec<TransformationRule>,
    /// Compiled JSONPath expressions cache
    path_cache: HashMap<String, JSONPath>,
    /// Optional lossiness tracker for monitoring transformations
    lossiness_tracker: Option<Arc<Mutex<LossinessTracker>>>,
}

impl TransformationPipeline {
    /// Create a new transformation pipeline
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            path_cache: HashMap::new(),
            lossiness_tracker: None,
        }
    }

    /// Create a pipeline with a lossiness tracker
    pub fn with_lossiness_tracker(mut self, tracker: Arc<Mutex<LossinessTracker>>) -> Self {
        self.lossiness_tracker = Some(tracker);
        self
    }

    /// Add a transformation rule to the pipeline
    pub fn add_rule(mut self, rule: TransformationRule) -> Self {
        self.rules.push(rule);
        // Sort by priority (highest first)
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        self
    }

    /// Add multiple rules to the pipeline
    pub fn add_rules<I>(mut self, rules: I) -> Self 
    where
        I: IntoIterator<Item = TransformationRule>,
    {
        for rule in rules {
            self.rules.push(rule);
        }
        // Sort by priority (highest first)
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        self
    }

    /// Apply all transformations in the pipeline
    pub fn transform(
        &mut self,
        source_data: &Value,
        direction: TransformationDirection,
        translation_context: &TranslationContext,
    ) -> Result<Value> {
        let mut result = source_data.clone();
        let mut transformation_context = TransformationContext {
            source_data: source_data.clone(),
            target_data: result.clone(),
            translation_context: translation_context.clone(),
            current_path: String::new(),
            metadata: HashMap::new(),
        };

        // Apply rules in priority order
        // Clone rules to avoid borrowing issues
        let rules = self.rules.clone();
        for rule in rules {
            // Check if rule applies to this direction
            if !self.rule_applies_to_direction(&rule, &direction) {
                continue;
            }

            match self.apply_rule(&rule, &mut result, &mut transformation_context) {
                Ok(_) => {},
                Err(e) => {
                    // Track the failure
                    self.track_failed_transformation(&rule, &e, &transformation_context, None);
                    
                    if rule.optional {
                        // Log warning and continue with optional rules
                        log::warn!("Optional transformation rule '{}' failed: {}", rule.id, e);
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        Ok(result)
    }

    /// Apply a single transformation rule
    fn apply_rule(
        &mut self,
        rule: &TransformationRule,
        data: &mut Value,
        context: &mut TransformationContext,
    ) -> Result<()> {
        // Parse and cache JSONPath for source
        let source_jsonpath = self.get_or_compile_path(&rule.source_path)?;
        
        // Find values at source path and collect them to avoid borrow conflicts
        let source_values: Vec<Value> = source_jsonpath.execute_owned(data)?;
        
        if source_values.is_empty() {
            // No values found - handle based on transformation type
            return self.handle_missing_source(rule, data, context);
        }

        // Apply transformation to each found value
        for value in source_values {
            context.current_path = rule.source_path.clone();
            
            // Capture before value for tracking
            let before_value = value.clone();
            
            // Apply transformation and track timing if tracker is available
            let start_time = std::time::Instant::now();
            let transformed_value = self.apply_transformation(&rule.transformation, &value, context)?;
            let duration = start_time.elapsed();
            
            // Track the transformation if tracker is available
            if let Some(tracker) = &self.lossiness_tracker {
                let provider_context = context.translation_context.provider_spec.provider.name.clone();
                let operation_type = self.transformation_to_operation_type(&rule.transformation);
                let reason = self.transformation_reason(rule, &rule.transformation);
                let metadata = self.build_transformation_metadata(rule, context);
                
                if let Ok(mut t) = tracker.lock() {
                    // Record transformation with timing
                    t.track_transformation(
                        &rule.source_path,
                        operation_type,
                        Some(before_value.clone()),
                        Some(transformed_value.clone()),
                        &reason,
                        Some(provider_context),
                        metadata,
                    );
                    
                    // Update performance metrics
                    t.update_transformation_timing(&rule.source_path, duration);
                }
            }

            // Determine target path
            let target_path = rule.target_path.as_ref().unwrap_or(&rule.source_path);
            
            // Set transformed value at target path
            self.set_value_at_path(target_path, transformed_value.clone(), data)?;
            
            // Note: transformation tracking is already complete above with both before and after values
        }

        Ok(())
    }

    /// Apply a specific transformation type
    fn apply_transformation(
        &mut self,
        transformation: &TransformationType,
        value: &Value,
        context: &TransformationContext,
    ) -> Result<Value> {
        let result = match transformation {
            TransformationType::TypeConversion { from, to } => {
                self.convert_type(value, from, to, &context.current_path)
            }
            TransformationType::EnumMapping { mappings, default } => {
                self.map_enum(value, mappings, default, &context.current_path)
            }
            TransformationType::UnitConversion { from_unit, to_unit, formula } => {
                self.convert_unit(value, from_unit, to_unit, formula, &context.current_path)
            }
            TransformationType::FieldRename { .. } => {
                // Field rename is handled by target_path, just return value
                Ok(value.clone())
            }
            TransformationType::DefaultValue { value: default_value } => {
                if value.is_null() {
                    Ok(default_value.clone())
                } else {
                    Ok(value.clone())
                }
            }
            TransformationType::Conditional { condition, if_true, if_false } => {
                if self.evaluate_condition(condition, &context.source_data, context)? {
                    self.apply_transformation(if_true, value, context)
                } else if let Some(false_transform) = if_false {
                    self.apply_transformation(false_transform, value, context)
                } else {
                    Ok(value.clone())
                }
            }
            TransformationType::Custom { transformer, .. } => {
                transformer(value, context).map_err(Into::into)
            }
        };
        
        // Handle specific error cases for tracking
        if let Err(ref _error) = result {
            // Track specific transformation errors if we have a tracker
            // This is handled at a higher level now
        }
        
        result
    }

    /// Convert between value types
    fn convert_type(
        &self,
        value: &Value,
        from: &ValueType,
        to: &ValueType,
        path: &str,
    ) -> Result<Value> {
        if from == to {
            return Ok(value.clone());
        }

        let result = match (from, to) {
            (ValueType::String, ValueType::Number) => {
                if let Some(s) = value.as_str() {
                    s.parse::<f64>()
                        .map(|n| Value::Number(serde_json::Number::from_f64(n).unwrap_or_else(|| serde_json::Number::from(0))))
                        .map_err(|_| TransformationError::TypeConversion {
                            from: "string".to_string(),
                            to: "number".to_string(),
                            value: s.to_string(),
                            path: path.to_string(),
                        })?
                } else {
                    return Err(TransformationError::TypeConversion {
                        from: format!("{:?}", value),
                        to: "number".to_string(),
                        value: value.to_string(),
                        path: path.to_string(),
                    }.into());
                }
            }
            (ValueType::String, ValueType::Boolean) => {
                if let Some(s) = value.as_str() {
                    match s.to_lowercase().as_str() {
                        "true" | "yes" | "1" | "on" => Value::Bool(true),
                        "false" | "no" | "0" | "off" => Value::Bool(false),
                        _ => return Err(TransformationError::TypeConversion {
                            from: "string".to_string(),
                            to: "boolean".to_string(),
                            value: s.to_string(),
                            path: path.to_string(),
                        }.into()),
                    }
                } else {
                    return Err(TransformationError::TypeConversion {
                        from: format!("{:?}", value),
                        to: "boolean".to_string(),
                        value: value.to_string(),
                        path: path.to_string(),
                    }.into());
                }
            }
            (ValueType::Number, ValueType::String) => {
                if let Some(n) = value.as_f64() {
                    Value::String(n.to_string())
                } else if let Some(n) = value.as_i64() {
                    Value::String(n.to_string())
                } else if let Some(n) = value.as_u64() {
                    Value::String(n.to_string())
                } else {
                    return Err(TransformationError::TypeConversion {
                        from: format!("{:?}", value),
                        to: "string".to_string(),
                        value: value.to_string(),
                        path: path.to_string(),
                    }.into());
                }
            }
            (ValueType::Boolean, ValueType::String) => {
                if let Some(b) = value.as_bool() {
                    Value::String(b.to_string())
                } else {
                    return Err(TransformationError::TypeConversion {
                        from: format!("{:?}", value),
                        to: "string".to_string(),
                        value: value.to_string(),
                        path: path.to_string(),
                    }.into());
                }
            }
            // Add more conversion patterns as needed
            _ => {
                return Err(TransformationError::TypeConversion {
                    from: format!("{:?}", from),
                    to: format!("{:?}", to),
                    value: value.to_string(),
                    path: path.to_string(),
                }.into());
            }
        };

        Ok(result)
    }

    /// Map enum values using lookup table
    fn map_enum(
        &self,
        value: &Value,
        mappings: &HashMap<String, String>,
        default: &Option<String>,
        path: &str,
    ) -> Result<Value> {
        let string_value = if let Some(s) = value.as_str() {
            s
        } else {
            return Err(TransformationError::EnumMapping {
                value: value.to_string(),
                path: path.to_string(),
                available_mappings: mappings.keys().cloned().collect(),
            }.into());
        };

        if let Some(mapped_value) = mappings.get(string_value) {
            Ok(Value::String(mapped_value.clone()))
        } else if let Some(default_value) = default {
            Ok(Value::String(default_value.clone()))
        } else {
            Err(TransformationError::EnumMapping {
                value: string_value.to_string(),
                path: path.to_string(),
                available_mappings: mappings.keys().cloned().collect(),
            }.into())
        }
    }

    /// Convert between units
    fn convert_unit(
        &self,
        value: &Value,
        from_unit: &str,
        to_unit: &str,
        formula: &ConversionFormula,
        path: &str,
    ) -> Result<Value> {
        let numeric_value = if let Some(n) = value.as_f64() {
            n
        } else {
            return Err(TransformationError::UnitConversion {
                message: "Value is not a number".to_string(),
                path: path.to_string(),
                from_unit: from_unit.to_string(),
                to_unit: to_unit.to_string(),
            }.into());
        };

        let converted_value = match formula {
            ConversionFormula::Linear { scale, offset } => {
                numeric_value * scale + offset
            }
            ConversionFormula::Custom(formula_name) => {
                // Handle known conversion formulas
                match (from_unit, to_unit, formula_name.as_str()) {
                    ("celsius", "fahrenheit", "c_to_f") => numeric_value * 9.0 / 5.0 + 32.0,
                    ("fahrenheit", "celsius", "f_to_c") => (numeric_value - 32.0) * 5.0 / 9.0,
                    ("kelvin", "celsius", "k_to_c") => numeric_value - 273.15,
                    ("celsius", "kelvin", "c_to_k") => numeric_value + 273.15,
                    _ => {
                        return Err(TransformationError::UnitConversion {
                            message: format!("Unknown conversion formula: {}", formula_name),
                            path: path.to_string(),
                            from_unit: from_unit.to_string(),
                            to_unit: to_unit.to_string(),
                        }.into());
                    }
                }
            }
        };

        Ok(Value::Number(
            serde_json::Number::from_f64(converted_value)
                .unwrap_or_else(|| serde_json::Number::from(0))
        ))
    }

    /// Evaluate a condition
    fn evaluate_condition(
        &mut self,
        condition: &Condition,
        data: &Value,
        context: &TransformationContext,
    ) -> Result<bool> {
        match condition {
            Condition::Equals { path, value } => {
                let jsonpath = self.get_or_compile_path(path)?;
                let found_values = jsonpath.execute(data)?;
                Ok(found_values.iter().any(|v| *v == value))
            }
            Condition::Exists { path } => {
                let jsonpath = self.get_or_compile_path(path)?;
                let found_values = jsonpath.execute(data)?;
                Ok(!found_values.is_empty())
            }
            Condition::Matches { path, pattern } => {
                let jsonpath = self.get_or_compile_path(path)?;
                let found_values = jsonpath.execute(data)?;
                let regex = regex::Regex::new(pattern).map_err(|e| {
                    TransformationError::ConditionEvaluation {
                        message: format!("Invalid regex pattern: {}", e),
                        path: path.clone(),
                        condition: format!("matches {}", pattern),
                    }
                })?;
                Ok(found_values.iter().any(|v| {
                    if let Some(s) = v.as_str() {
                        regex.is_match(s)
                    } else {
                        false
                    }
                }))
            }
            Condition::And(conditions) => {
                for cond in conditions {
                    if !self.evaluate_condition(cond, data, context)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Condition::Or(conditions) => {
                for cond in conditions {
                    if self.evaluate_condition(cond, data, context)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Condition::Not(condition) => {
                Ok(!self.evaluate_condition(condition, data, context)?)
            }
        }
    }

    /// Handle missing source values
    fn handle_missing_source(
        &self,
        rule: &TransformationRule,
        data: &mut Value,
        context: &TransformationContext,
    ) -> Result<()> {
        // Only handle DefaultValue transformation for missing sources
        if let TransformationType::DefaultValue { value } = &rule.transformation {
            let target_path = rule.target_path.as_ref().unwrap_or(&rule.source_path);
            self.set_value_at_path(target_path, value.clone(), data)?;
            
            // Track default value application
            if let Some(tracker) = &self.lossiness_tracker {
                let provider_context = context.translation_context.provider_spec.provider.name.clone();
                let reason = format!(
                    "Default value applied for missing field: {}",
                    rule.source_path
                );
                
                if let Ok(mut t) = tracker.lock() {
                    t.track_default_applied(
                        &rule.source_path,
                        Some(value.clone()),
                        &reason,
                        Some(provider_context),
                    );
                }
            }
        } else {
            // Track conditional transformation that was skipped
            if let Some(tracker) = &self.lossiness_tracker {
                let provider_context = context.translation_context.provider_spec.provider.name.clone();
                let reason = format!(
                    "Transformation skipped - no value found at path: {}",
                    rule.source_path
                );
                
                if let Ok(mut t) = tracker.lock() {
                    t.track_transformation(
                        &rule.source_path,
                        self.transformation_to_operation_type(&rule.transformation),
                        None,
                        None,
                        &reason,
                        Some(provider_context),
                        self.build_transformation_metadata(rule, context),
                    );
                }
            }
        }
        Ok(())
    }

    /// Get or compile a JSONPath expression
    fn get_or_compile_path(&mut self, path: &str) -> Result<&JSONPath> {
        if !self.path_cache.contains_key(path) {
            let jsonpath = JSONPath::parse(path)?;
            self.path_cache.insert(path.to_string(), jsonpath);
        }
        Ok(&self.path_cache[path])
    }

    /// Set a value at a JSONPath location
    fn set_value_at_path(&self, path: &str, value: Value, data: &mut Value) -> Result<()> {
        // Simple implementation for dot notation paths
        // In a full implementation, this would support complex JSONPath assignments
        if path == "$" {
            *data = value;
            return Ok(());
        }

        let path = path.trim_start_matches("$.");
        let parts: Vec<&str> = path.split('.').collect();

        if parts.is_empty() {
            return Ok(());
        }

        // Ensure data is an object
        if !data.is_object() {
            *data = Value::Object(serde_json::Map::new());
        }

        let mut current = data;
        let last_index = parts.len() - 1;

        for (i, part) in parts.iter().enumerate() {
            if i == last_index {
                // Set the final value
                if let Some(obj) = current.as_object_mut() {
                    obj.insert(part.to_string(), value);
                }
                return Ok(());
            } else {
                // Navigate or create intermediate objects
                let obj = current.as_object_mut().ok_or_else(|| {
                    crate::Error::Translation {
                        message: "Expected object for path navigation".to_string(),
                        context: Some("Field Transformation".to_string()),
                    }
                })?;
                current = obj
                    .entry(part.to_string())
                    .or_insert_with(|| Value::Object(serde_json::Map::new()));
            }
        }

        Ok(())
    }

    /// Check if a rule applies to the given direction
    fn rule_applies_to_direction(&self, rule: &TransformationRule, direction: &TransformationDirection) -> bool {
        match (&rule.direction, direction) {
            (TransformationDirection::Bidirectional, _) => true,
            (TransformationDirection::Forward, TransformationDirection::Forward) => true,
            (TransformationDirection::Reverse, TransformationDirection::Reverse) => true,
            _ => false,
        }
    }

    /// Get the number of rules in the pipeline
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.path_cache.len(), self.path_cache.capacity())
    }

    /// Clear the path cache
    pub fn clear_cache(&mut self) {
        self.path_cache.clear();
    }
    
    /// Convert transformation type to operation type for tracking
    fn transformation_to_operation_type(&self, transformation: &TransformationType) -> OperationType {
        match transformation {
            TransformationType::TypeConversion { .. } => OperationType::TypeConversion,
            TransformationType::EnumMapping { .. } => OperationType::EnumMapping,
            TransformationType::UnitConversion { .. } => OperationType::UnitConversion,
            TransformationType::FieldRename { .. } => OperationType::FieldMove,
            TransformationType::DefaultValue { .. } => OperationType::DefaultApplied,
            TransformationType::Conditional { .. } => OperationType::TypeConversion, // Generic for conditionals
            TransformationType::Custom { .. } => OperationType::TypeConversion, // Generic for custom
        }
    }
    
    /// Generate descriptive reason for transformation
    fn transformation_reason(&self, rule: &TransformationRule, transformation: &TransformationType) -> String {
        let rule_name = rule.id.clone();
        match transformation {
            TransformationType::TypeConversion { from, to } => {
                format!("Rule '{}': Convert {:?} to {:?}", rule_name, from, to)
            }
            TransformationType::EnumMapping { mappings, default } => {
                let default_info = default.as_ref()
                    .map(|d| format!(" (default: {})", d))
                    .unwrap_or_default();
                format!("Rule '{}': Enum mapping with {} options{}", rule_name, mappings.len(), default_info)
            }
            TransformationType::UnitConversion { from_unit, to_unit, .. } => {
                format!("Rule '{}': Convert {} to {}", rule_name, from_unit, to_unit)
            }
            TransformationType::FieldRename { new_name } => {
                format!("Rule '{}': Rename field to {}", rule_name, new_name)
            }
            TransformationType::DefaultValue { value } => {
                format!("Rule '{}': Apply default value: {}", rule_name, value)
            }
            TransformationType::Conditional { .. } => {
                format!("Rule '{}': Conditional transformation", rule_name)
            }
            TransformationType::Custom { name, .. } => {
                format!("Rule '{}': Custom transformation '{}'", rule_name, name)
            }
        }
    }
    
    /// Build metadata for transformation tracking
    fn build_transformation_metadata(&self, rule: &TransformationRule, context: &TransformationContext) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("rule_id".to_string(), rule.id.clone());
        metadata.insert("rule_priority".to_string(), rule.priority.to_string());
        metadata.insert("rule_optional".to_string(), rule.optional.to_string());
        metadata.insert("rule_direction".to_string(), format!("{:?}", rule.direction));
        metadata.insert("source_path".to_string(), rule.source_path.clone());
        
        if let Some(target_path) = &rule.target_path {
            metadata.insert("target_path".to_string(), target_path.clone());
        }
        
        metadata.insert("current_path".to_string(), context.current_path.clone());
        
        // Add any additional context metadata
        for (key, value) in &context.metadata {
            metadata.insert(
                format!("context_{}", key),
                value.as_str().unwrap_or(&value.to_string()).to_string()
            );
        }
        
        metadata
    }
}

impl Default for TransformationPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for handling failed transformations with tracking
impl TransformationPipeline {
    /// Track a failed transformation
    fn track_failed_transformation(
        &self,
        rule: &TransformationRule,
        error: &crate::Error,
        context: &TransformationContext,
        original_value: Option<Value>,
    ) {
        if let Some(tracker) = &self.lossiness_tracker {
            let provider_context = context.translation_context.provider_spec.provider.name.clone();
            let reason = if rule.optional {
                format!("Optional transformation failed: {}", error)
            } else {
                format!("Transformation failed: {}", error)
            };
            
            if let Ok(mut t) = tracker.lock() {
                t.track_transformation(
                    &rule.source_path,
                    self.transformation_to_operation_type(&rule.transformation),
                    original_value,
                    None,
                    &reason,
                    Some(provider_context),
                    self.build_transformation_metadata(rule, context),
                );
            }
        }
    }
}

/// Builder for creating transformation rules
pub struct TransformationRuleBuilder {
    id: String,
    source_path: String,
    target_path: Option<String>,
    transformation: Option<TransformationType>,
    direction: TransformationDirection,
    priority: i32,
    optional: bool,
    tracker: Option<Arc<Mutex<LossinessTracker>>>,
}

impl TransformationRuleBuilder {
    /// Create a new rule builder
    pub fn new(id: impl Into<String>, source_path: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            source_path: source_path.into(),
            target_path: None,
            transformation: None,
            direction: TransformationDirection::Forward,
            priority: 0,
            optional: false,
            tracker: None,
        }
    }

    /// Set the target path
    pub fn target_path(mut self, path: impl Into<String>) -> Self {
        self.target_path = Some(path.into());
        self
    }

    /// Set the transformation type
    pub fn transformation(mut self, transformation: TransformationType) -> Self {
        self.transformation = Some(transformation);
        self
    }

    /// Set the direction
    pub fn direction(mut self, direction: TransformationDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Set the priority
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Make the rule optional
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
    
    /// Add a lossiness tracker to this rule
    pub fn with_tracker(mut self, tracker: Arc<Mutex<LossinessTracker>>) -> Self {
        self.tracker = Some(tracker);
        self
    }

    /// Build the transformation rule
    pub fn build(self) -> Result<TransformationRule> {
        let transformation = self.transformation.ok_or_else(|| {
            TransformationError::Configuration {
                message: "Transformation type is required".to_string(),
                rule_id: Some(self.id.clone()),
            }
        })?;

        Ok(TransformationRule {
            id: self.id,
            source_path: self.source_path,
            target_path: self.target_path,
            transformation,
            direction: self.direction,
            priority: self.priority,
            optional: self.optional,
        })
    }
}

/// Built-in transformers for common operations
pub mod built_in {
    use super::*;

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PromptSpec, ProviderSpec, ModelSpec, StrictMode};
    use serde_json::json;

    fn create_test_context() -> TranslationContext {
        // Create minimal test context - details don't matter for transformer tests
        let prompt_spec = PromptSpec {
            model_class: "Chat".to_string(),
            messages: vec![],
            tools: None,
            tool_choice: None,
            response_format: None,
            sampling: None,
            limits: None,
            media: None,
            strict_mode: StrictMode::Warn,
        };

        let provider_spec = ProviderSpec {
            spec_version: "1.0.0".to_string(),
            provider: crate::ProviderInfo {
                name: "test".to_string(),
                base_url: "https://api.test.com".to_string(),
                headers: std::collections::HashMap::new(),
            },
            models: vec![],
        };

        let model_spec = ModelSpec {
            id: "test-model".to_string(),
            aliases: None,
            family: "test".to_string(),
            endpoints: crate::Endpoints {
                chat_completion: crate::EndpointConfig {
                    method: "POST".to_string(),
                    path: "/chat".to_string(),
                    protocol: "https".to_string(),
                    query: None,
                    headers: None,
                },
                streaming_chat_completion: crate::EndpointConfig {
                    method: "POST".to_string(),
                    path: "/chat".to_string(),
                    protocol: "https".to_string(),
                    query: None,
                    headers: None,
                },
            },
            input_modes: crate::InputModes {
                messages: true,
                single_text: false,
                images: false,
            },
            tooling: crate::ToolingConfig {
                tools_supported: true,
                parallel_tool_calls_default: false,
                can_disable_parallel_tool_calls: false,
                disable_switch: None,
            },
            json_output: crate::JsonOutputConfig {
                native_param: true,
                strategy: "native".to_string(),
            },
            parameters: json!({}),
            constraints: crate::Constraints {
                system_prompt_location: "first".to_string(),
                forbid_unknown_top_level_fields: false,
                mutually_exclusive: vec![],
                resolution_preferences: vec![],
                limits: crate::ConstraintLimits {
                    max_tool_schema_bytes: 100000,
                    max_system_prompt_bytes: 10000,
                },
            },
            mappings: crate::Mappings {
                paths: std::collections::HashMap::new(),
                flags: std::collections::HashMap::new(),
            },
            response_normalization: crate::ResponseNormalization {
                sync: crate::SyncNormalization {
                    content_path: "content".to_string(),
                    finish_reason_path: "finish".to_string(),
                    finish_reason_map: std::collections::HashMap::new(),
                },
                stream: crate::StreamNormalization {
                    protocol: "sse".to_string(),
                    event_selector: crate::EventSelector {
                        type_path: "type".to_string(),
                        routes: vec![],
                    },
                },
            },
        };

        TranslationContext::new(prompt_spec, provider_spec, model_spec, StrictMode::Warn)
    }

    #[test]
    fn test_pipeline_creation() {
        let pipeline = TransformationPipeline::new();
        assert_eq!(pipeline.rule_count(), 0);
    }

    #[test]
    fn test_type_conversion() {
        let mut pipeline = TransformationPipeline::new();
        let context = create_test_context();

        let rule = TransformationRuleBuilder::new("string_to_num", "$.temperature")
            .transformation(built_in::string_to_number())
            .build()
            .unwrap();

        pipeline = pipeline.add_rule(rule);

        let input = json!({
            "temperature": "0.7"
        });

        let result = pipeline.transform(&input, TransformationDirection::Forward, &context).unwrap();
        assert_eq!(result["temperature"], json!(0.7));
    }

    #[test]
    fn test_enum_mapping() {
        let mut pipeline = TransformationPipeline::new();
        let context = create_test_context();

        let rule = TransformationRuleBuilder::new("model_mapping", "$.model")
            .transformation(built_in::openai_to_anthropic_models())
            .build()
            .unwrap();

        pipeline = pipeline.add_rule(rule);

        let input = json!({
            "model": "gpt-4"
        });

        let result = pipeline.transform(&input, TransformationDirection::Forward, &context).unwrap();
        assert_eq!(result["model"], json!("claude-3-opus-20240229"));
    }

    #[test]
    fn test_unit_conversion() {
        let mut pipeline = TransformationPipeline::new();
        let context = create_test_context();

        let rule = TransformationRuleBuilder::new("temp_scale", "$.temperature")
            .transformation(built_in::temperature_0_2_to_0_1())
            .build()
            .unwrap();

        pipeline = pipeline.add_rule(rule);

        let input = json!({
            "temperature": 1.0
        });

        let result = pipeline.transform(&input, TransformationDirection::Forward, &context).unwrap();
        assert_eq!(result["temperature"], json!(0.5));
    }

    #[test]
    fn test_default_value() {
        let mut pipeline = TransformationPipeline::new();
        let context = create_test_context();

        let rule = TransformationRuleBuilder::new("default_temp", "$.temperature")
            .transformation(built_in::default_value(json!(0.7)))
            .build()
            .unwrap();

        pipeline = pipeline.add_rule(rule);

        let input = json!({
            "model": "gpt-4"
        });

        let result = pipeline.transform(&input, TransformationDirection::Forward, &context).unwrap();
        assert_eq!(result["temperature"], json!(0.7));
    }

    #[test]
    fn test_field_rename() {
        let mut pipeline = TransformationPipeline::new();
        let context = create_test_context();

        let rule = TransformationRuleBuilder::new("rename_temp", "$.temperature")
            .target_path("$.temp")
            .transformation(built_in::rename_field("temp"))
            .build()
            .unwrap();

        pipeline = pipeline.add_rule(rule);

        let input = json!({
            "temperature": 0.7
        });

        let result = pipeline.transform(&input, TransformationDirection::Forward, &context).unwrap();
        assert_eq!(result["temp"], json!(0.7));
    }

    #[test]
    fn test_conditional_transformation() {
        let mut pipeline = TransformationPipeline::new();
        let context = create_test_context();

        let condition = Condition::Equals {
            path: "$.provider".to_string(),
            value: json!("openai"),
        };

        let transformation = TransformationType::Conditional {
            condition,
            if_true: Box::new(built_in::openai_to_anthropic_models()),
            if_false: None,
        };

        let rule = TransformationRuleBuilder::new("conditional_model", "$.model")
            .transformation(transformation)
            .build()
            .unwrap();

        pipeline = pipeline.add_rule(rule);

        let input = json!({
            "provider": "openai",
            "model": "gpt-4"
        });

        let result = pipeline.transform(&input, TransformationDirection::Forward, &context).unwrap();
        assert_eq!(result["model"], json!("claude-3-opus-20240229"));
    }

    #[test]
    fn test_rule_priority() {
        let mut pipeline = TransformationPipeline::new();
        let context = create_test_context();

        let rule1 = TransformationRuleBuilder::new("low_priority", "$.value")
            .transformation(built_in::default_value(json!("low")))
            .priority(1)
            .build()
            .unwrap();

        let rule2 = TransformationRuleBuilder::new("high_priority", "$.value")
            .transformation(built_in::default_value(json!("high")))
            .priority(10)
            .build()
            .unwrap();

        pipeline = pipeline.add_rule(rule1).add_rule(rule2);

        let input = json!({});

        let result = pipeline.transform(&input, TransformationDirection::Forward, &context).unwrap();
        // Higher priority rule should execute first and set the value
        assert_eq!(result["value"], json!("high"));
    }

    #[test]
    fn test_transformation_direction() {
        let mut pipeline = TransformationPipeline::new();
        let context = create_test_context();

        let forward_rule = TransformationRuleBuilder::new("forward_only", "$.forward")
            .transformation(built_in::default_value(json!("forward")))
            .direction(TransformationDirection::Forward)
            .build()
            .unwrap();

        let reverse_rule = TransformationRuleBuilder::new("reverse_only", "$.reverse")
            .transformation(built_in::default_value(json!("reverse")))
            .direction(TransformationDirection::Reverse)
            .build()
            .unwrap();

        pipeline = pipeline.add_rule(forward_rule).add_rule(reverse_rule);

        let input = json!({});

        // Test forward direction
        let forward_result = pipeline.transform(&input, TransformationDirection::Forward, &context).unwrap();
        assert_eq!(forward_result["forward"], json!("forward"));
        assert!(forward_result.get("reverse").is_none());

        // Test reverse direction
        let reverse_result = pipeline.transform(&input, TransformationDirection::Reverse, &context).unwrap();
        assert_eq!(reverse_result["reverse"], json!("reverse"));
        assert!(reverse_result.get("forward").is_none());
    }

    #[test]
    fn test_optional_rule_failure() {
        let mut pipeline = TransformationPipeline::new();
        let context = create_test_context();

        // Create a rule that will fail (trying to convert non-numeric string to number)
        let rule = TransformationRuleBuilder::new("failing_rule", "$.text")
            .transformation(built_in::string_to_number())
            .optional()
            .build()
            .unwrap();

        pipeline = pipeline.add_rule(rule);

        let input = json!({
            "text": "not a number"
        });

        // Should not fail because rule is optional
        let result = pipeline.transform(&input, TransformationDirection::Forward, &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_lossiness_tracking_integration() {
        use crate::StrictMode;
        use super::super::lossiness::LossinessTracker;
        use std::sync::{Arc, Mutex};
        
        let mut pipeline = TransformationPipeline::new();
        let context = create_test_context();
        
        // Create a lossiness tracker
        let tracker = Arc::new(Mutex::new(LossinessTracker::new(StrictMode::Warn)));
        pipeline = pipeline.with_lossiness_tracker(tracker.clone());
        
        // Add a transformation rule
        let rule = TransformationRuleBuilder::new("string_to_num_tracked", "$.temperature")
            .transformation(built_in::string_to_number())
            .build()
            .unwrap();
            
        pipeline = pipeline.add_rule(rule);
        
        let input = json!({
            "temperature": "0.7"
        });
        
        let result = pipeline.transform(&input, TransformationDirection::Forward, &context).unwrap();
        assert_eq!(result["temperature"], json!(0.7));
        
        // Verify tracking occurred
        let tracker_guard = tracker.lock().unwrap();
        let stats = tracker_guard.get_summary_statistics();
        assert_eq!(stats.total_transformations, 1);
        assert_eq!(stats.by_operation_type.get("TypeConversion"), Some(&1));
        
        // Check the recorded transformation
        let records = tracker_guard.get_transformations_by_type(OperationType::TypeConversion);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].field_path, "$.temperature");
        assert_eq!(records[0].before_value, Some(json!("0.7")));
        assert_eq!(records[0].after_value, Some(json!(0.7)));
    }
    
    #[test]
    fn test_default_value_tracking() {
        use crate::StrictMode;
        use super::super::lossiness::LossinessTracker;
        use std::sync::{Arc, Mutex};
        
        let mut pipeline = TransformationPipeline::new();
        let context = create_test_context();
        
        let tracker = Arc::new(Mutex::new(LossinessTracker::new(StrictMode::Warn)));
        pipeline = pipeline.with_lossiness_tracker(tracker.clone());
        
        let rule = TransformationRuleBuilder::new("default_temp_tracked", "$.temperature")
            .transformation(built_in::default_value(json!(0.7)))
            .build()
            .unwrap();
            
        pipeline = pipeline.add_rule(rule);
        
        let input = json!({
            "model": "gpt-4"
        });
        
        let result = pipeline.transform(&input, TransformationDirection::Forward, &context).unwrap();
        assert_eq!(result["temperature"], json!(0.7));
        
        // Verify default tracking
        let tracker_guard = tracker.lock().unwrap();
        let defaults = tracker_guard.get_transformations_by_type(OperationType::DefaultApplied);
        assert_eq!(defaults.len(), 1);
        assert_eq!(defaults[0].field_path, "$.temperature");
        assert_eq!(defaults[0].after_value, Some(json!(0.7)));
    }
    
    #[test]
    fn test_failed_transformation_tracking() {
        use crate::StrictMode;
        use super::super::lossiness::LossinessTracker;
        use std::sync::{Arc, Mutex};
        
        let mut pipeline = TransformationPipeline::new();
        let context = create_test_context();
        
        let tracker = Arc::new(Mutex::new(LossinessTracker::new(StrictMode::Warn)));
        pipeline = pipeline.with_lossiness_tracker(tracker.clone());
        
        // Create a rule that will fail but is optional
        let rule = TransformationRuleBuilder::new("failing_rule_tracked", "$.text")
            .transformation(built_in::string_to_number())
            .optional()
            .build()
            .unwrap();
            
        pipeline = pipeline.add_rule(rule);
        
        let input = json!({
            "text": "not a number"
        });
        
        // Should not fail because rule is optional
        let result = pipeline.transform(&input, TransformationDirection::Forward, &context);
        assert!(result.is_ok());
        
        // Verify failure was tracked
        let tracker_guard = tracker.lock().unwrap();
        let stats = tracker_guard.get_summary_statistics();
        assert!(stats.total_transformations > 0); // At least the failed attempt should be tracked
    }
    
    #[test]
    fn test_enum_mapping_tracking() {
        use crate::StrictMode;
        use super::super::lossiness::LossinessTracker;
        use std::sync::{Arc, Mutex};
        
        let mut pipeline = TransformationPipeline::new();
        let context = create_test_context();
        
        let tracker = Arc::new(Mutex::new(LossinessTracker::new(StrictMode::Warn)));
        pipeline = pipeline.with_lossiness_tracker(tracker.clone());
        
        let rule = TransformationRuleBuilder::new("model_mapping_tracked", "$.model")
            .transformation(built_in::openai_to_anthropic_models())
            .build()
            .unwrap();
            
        pipeline = pipeline.add_rule(rule);
        
        let input = json!({
            "model": "gpt-4"
        });
        
        let result = pipeline.transform(&input, TransformationDirection::Forward, &context).unwrap();
        assert_eq!(result["model"], json!("claude-3-opus-20240229"));
        
        // Verify enum mapping was tracked
        let tracker_guard = tracker.lock().unwrap();
        let mappings = tracker_guard.get_transformations_by_type(OperationType::EnumMapping);
        assert_eq!(mappings.len(), 1);
        assert_eq!(mappings[0].field_path, "$.model");
        assert_eq!(mappings[0].before_value, Some(json!("gpt-4")));
        assert_eq!(mappings[0].after_value, Some(json!("claude-3-opus-20240229")));
    }
    
    #[test]
    fn test_unit_conversion_tracking() {
        use crate::StrictMode;
        use super::super::lossiness::LossinessTracker;
        use std::sync::{Arc, Mutex};
        
        let mut pipeline = TransformationPipeline::new();
        let context = create_test_context();
        
        let tracker = Arc::new(Mutex::new(LossinessTracker::new(StrictMode::Warn)));
        pipeline = pipeline.with_lossiness_tracker(tracker.clone());
        
        let rule = TransformationRuleBuilder::new("temp_scale_tracked", "$.temperature")
            .transformation(built_in::temperature_0_2_to_0_1())
            .build()
            .unwrap();
            
        pipeline = pipeline.add_rule(rule);
        
        let input = json!({
            "temperature": 1.0
        });
        
        let result = pipeline.transform(&input, TransformationDirection::Forward, &context).unwrap();
        assert_eq!(result["temperature"], json!(0.5));
        
        // Verify unit conversion was tracked
        let tracker_guard = tracker.lock().unwrap();
        let conversions = tracker_guard.get_transformations_by_type(OperationType::UnitConversion);
        assert_eq!(conversions.len(), 1);
        assert_eq!(conversions[0].field_path, "$.temperature");
        assert_eq!(conversions[0].before_value, Some(json!(1.0)));
        assert_eq!(conversions[0].after_value, Some(json!(0.5)));
    }
    
    #[test]
    fn test_field_rename_tracking() {
        use crate::StrictMode;
        use super::super::lossiness::LossinessTracker;
        use std::sync::{Arc, Mutex};
        
        let mut pipeline = TransformationPipeline::new();
        let context = create_test_context();
        
        let tracker = Arc::new(Mutex::new(LossinessTracker::new(StrictMode::Warn)));
        pipeline = pipeline.with_lossiness_tracker(tracker.clone());
        
        let rule = TransformationRuleBuilder::new("rename_temp_tracked", "$.temperature")
            .target_path("$.temp")
            .transformation(built_in::rename_field("temp"))
            .build()
            .unwrap();
            
        pipeline = pipeline.add_rule(rule);
        
        let input = json!({
            "temperature": 0.7
        });
        
        let result = pipeline.transform(&input, TransformationDirection::Forward, &context).unwrap();
        assert_eq!(result["temp"], json!(0.7));
        
        // Verify field rename was tracked
        let tracker_guard = tracker.lock().unwrap();
        let moves = tracker_guard.get_transformations_by_type(OperationType::FieldMove);
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].field_path, "$.temperature");
    }
    
    #[test]
    fn test_comprehensive_transformation_pipeline() {
        let mut pipeline = TransformationPipeline::new();
        let context = create_test_context();

        // Create a complex transformation pipeline that demonstrates multiple features
        
        // 1. Model mapping (highest priority)
        let model_rule = TransformationRuleBuilder::new("model_mapping", "$.model")
            .transformation(built_in::openai_to_anthropic_models())
            .priority(100)
            .build()
            .unwrap();

        // 2. Temperature scaling
        let temp_rule = TransformationRuleBuilder::new("temp_scaling", "$.temperature")
            .transformation(built_in::temperature_0_2_to_0_1())
            .priority(50)
            .build()
            .unwrap();

        // 3. Convert string max_tokens to number
        let max_tokens_rule = TransformationRuleBuilder::new("max_tokens_convert", "$.max_tokens")
            .transformation(built_in::string_to_number())
            .priority(40)
            .optional()
            .build()
            .unwrap();

        // 4. Add default values
        let default_temp_rule = TransformationRuleBuilder::new("default_temp", "$.temperature")
            .transformation(built_in::default_value(json!(0.7)))
            .priority(10)
            .build()
            .unwrap();

        // 5. Rename messages to conversation
        let rename_rule = TransformationRuleBuilder::new("rename_messages", "$.messages")
            .target_path("$.conversation")
            .transformation(built_in::rename_field("conversation"))
            .priority(30)
            .build()
            .unwrap();

        pipeline = pipeline
            .add_rule(model_rule)
            .add_rule(temp_rule)
            .add_rule(max_tokens_rule)
            .add_rule(default_temp_rule)
            .add_rule(rename_rule);

        let input = json!({
            "model": "gpt-4",
            "temperature": 1.6,
            "max_tokens": "1000",
            "messages": [
                {"role": "user", "content": "Hello"}
            ]
        });

        let result = pipeline.transform(&input, TransformationDirection::Forward, &context).unwrap();

        // Verify transformations were applied correctly
        assert_eq!(result["model"], json!("claude-3-opus-20240229"));
        assert_eq!(result["temperature"], json!(0.8)); // 1.6 * 0.5 = 0.8
        assert_eq!(result["max_tokens"], json!(1000.0));
        assert_eq!(result["conversation"], json!([{"role": "user", "content": "Hello"}]));
        
        // Note: The current rename implementation copies to target path but doesn't remove source
        // Both fields should exist - this is copy behavior, not move behavior
        assert!(result.get("messages").is_some());
        assert!(result.get("conversation").is_some());
        assert_eq!(result["messages"], result["conversation"]);
    }
    
    #[test]
    fn test_comprehensive_pipeline_with_tracking() {
        use crate::StrictMode;
        use super::super::lossiness::LossinessTracker;
        use std::sync::{Arc, Mutex};
        
        let mut pipeline = TransformationPipeline::new();
        let context = create_test_context();
        
        // Create a tracker and add it to the pipeline
        let tracker = Arc::new(Mutex::new(LossinessTracker::new(StrictMode::Warn)));
        pipeline = pipeline.with_lossiness_tracker(tracker.clone());
        
        // Create the same comprehensive pipeline as the existing test
        let model_rule = TransformationRuleBuilder::new("model_mapping", "$.model")
            .transformation(built_in::openai_to_anthropic_models())
            .priority(100)
            .build()
            .unwrap();

        let temp_rule = TransformationRuleBuilder::new("temp_scaling", "$.temperature")
            .transformation(built_in::temperature_0_2_to_0_1())
            .priority(50)
            .build()
            .unwrap();

        let max_tokens_rule = TransformationRuleBuilder::new("max_tokens_convert", "$.max_tokens")
            .transformation(built_in::string_to_number())
            .priority(40)
            .optional()
            .build()
            .unwrap();

        let default_temp_rule = TransformationRuleBuilder::new("default_temp", "$.temperature")
            .transformation(built_in::default_value(json!(0.7)))
            .priority(10)
            .build()
            .unwrap();

        let rename_rule = TransformationRuleBuilder::new("rename_messages", "$.messages")
            .target_path("$.conversation")
            .transformation(built_in::rename_field("conversation"))
            .priority(30)
            .build()
            .unwrap();

        pipeline = pipeline
            .add_rule(model_rule)
            .add_rule(temp_rule)
            .add_rule(max_tokens_rule)
            .add_rule(default_temp_rule)
            .add_rule(rename_rule);

        let input = json!({
            "model": "gpt-4",
            "temperature": 1.6,
            "max_tokens": "1000",
            "messages": [
                {"role": "user", "content": "Hello"}
            ]
        });

        let result = pipeline.transform(&input, TransformationDirection::Forward, &context).unwrap();

        // Verify transformations were applied correctly
        assert_eq!(result["model"], json!("claude-3-opus-20240229"));
        assert_eq!(result["temperature"], json!(0.8)); // 1.6 * 0.5 = 0.8
        assert_eq!(result["max_tokens"], json!(1000.0));
        assert_eq!(result["conversation"], json!([{"role": "user", "content": "Hello"}]));
        
        // Verify tracking occurred for all transformations
        let tracker_guard = tracker.lock().unwrap();
        let stats = tracker_guard.get_summary_statistics();
        
        // Should have tracked: model mapping, temp scaling, max_tokens conversion, rename
        // Note: default temp rule doesn't apply since temperature already exists
        assert!(stats.total_transformations >= 4);
        
        // Verify specific operation types were recorded
        assert!(stats.by_operation_type.contains_key("EnumMapping")); // model mapping
        assert!(stats.by_operation_type.contains_key("UnitConversion")); // temp scaling
        assert!(stats.by_operation_type.contains_key("TypeConversion")); // max_tokens
        assert!(stats.by_operation_type.contains_key("FieldMove")); // rename
        
        // Generate and verify audit report
        let report = tracker_guard.generate_audit_report();
        assert!(report.contains("Transformation Audit Report"));
        assert!(report.contains("$.model"));
        assert!(report.contains("$.temperature"));
        assert!(report.contains("$.max_tokens"));
    }

    #[test]
    fn test_jsonpath_integration() {
        let mut pipeline = TransformationPipeline::new();
        let context = create_test_context();

        // Test complex JSONPath expressions with transformations
        let nested_rule = TransformationRuleBuilder::new("nested_transform", "$.config.sampling.temperature")
            .transformation(built_in::temperature_0_2_to_0_1())
            .build()
            .unwrap();

        // Use a simpler path for now - array indexing in set_value_at_path needs more work
        let temp_rule = TransformationRuleBuilder::new("temp_convert", "$.temp_string")
            .transformation(built_in::string_to_number())
            .build()
            .unwrap();

        pipeline = pipeline.add_rule(nested_rule).add_rule(temp_rule);

        let input = json!({
            "config": {
                "sampling": {
                    "temperature": 1.4
                }
            },
            "temp_string": "42.5"
        });

        let result = pipeline.transform(&input, TransformationDirection::Forward, &context).unwrap();

        assert_eq!(result["config"]["sampling"]["temperature"], json!(0.7)); // 1.4 * 0.5
        assert_eq!(result["temp_string"], json!(42.5)); // String "42.5" converted to number
    }

    #[test]
    fn test_bidirectional_transformations() {
        let mut pipeline = TransformationPipeline::new();
        let context = create_test_context();

        // Create a bidirectional transformation
        let temp_rule = TransformationRuleBuilder::new("temp_bidirectional", "$.temperature")
            .transformation(built_in::temperature_0_2_to_0_1())
            .direction(TransformationDirection::Bidirectional)
            .build()
            .unwrap();

        pipeline = pipeline.add_rule(temp_rule);

        let input = json!({
            "temperature": 1.0
        });

        // Test forward direction (should scale down)
        let forward_result = pipeline.transform(&input, TransformationDirection::Forward, &context).unwrap();
        assert_eq!(forward_result["temperature"], json!(0.5));

        // Test reverse direction (should also scale down - this is a simple test)
        let reverse_result = pipeline.transform(&input, TransformationDirection::Reverse, &context).unwrap();
        assert_eq!(reverse_result["temperature"], json!(0.5));
    }
}
