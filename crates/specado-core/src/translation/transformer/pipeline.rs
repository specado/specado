//! Transformation pipeline implementation
//!
//! This module contains the core transformation pipeline logic that applies
//! transformation rules to JSON data structures using JSONPath for field selection.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::Result;
use super::super::jsonpath::JSONPath;
use super::super::lossiness::{LossinessTracker, OperationType};
use super::super::TranslationContext;
use super::types::{
    TransformationError, TransformationType, ValueType, ConversionFormula, 
    Condition, TransformationContext, TransformationDirection, TransformationRule
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
                transformer(value, context)
            }
        };
        
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
    #[allow(clippy::only_used_in_recursion)]
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
                Ok(found_values.contains(&value))
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
        matches!((&rule.direction, direction), 
            (TransformationDirection::Bidirectional, _) | 
            (TransformationDirection::Forward, TransformationDirection::Forward) | 
            (TransformationDirection::Reverse, TransformationDirection::Reverse)
        )
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

impl Default for TransformationPipeline {
    fn default() -> Self {
        Self::new()
    }
}