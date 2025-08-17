//! Filter expression evaluation for JSONPath
//!
//! This module provides efficient evaluation of filter expressions with
//! short-circuit evaluation and comprehensive type handling.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use super::ast::*;
use super::error::*;
use super::selector::{SelectionIterator, SelectorExecutor};
use crate::Result;
use serde_json::Value;
use std::cmp::Ordering;

/// Filter executor for evaluating filter expressions
pub struct FilterExecutor {
    pub filter: CompiledFilter,
}

impl FilterExecutor {
    pub fn new(filter: CompiledFilter) -> Self {
        Self { filter }
    }
}

impl SelectorExecutor for FilterExecutor {
    fn execute<'a>(&self, inputs: SelectionIterator<'a>) -> Result<SelectionIterator<'a>> {
        let mut results = Vec::new();
        let context = FilterContext::new();

        for value in inputs {
            let filter_result = self.evaluate_filter(&self.filter, value, value, &context)?;
            if is_truthy(&filter_result) {
                results.push(value);
            }
        }

        Ok(SelectionIterator::new(results))
    }
}

impl FilterExecutor {
    /// Evaluate a filter expression
    pub fn evaluate_filter(
        &self,
        filter: &CompiledFilter,
        current: &Value,
        root: &Value,
        context: &FilterContext,
    ) -> Result<Value> {
        match filter {
            CompiledFilter::Current => Ok(current.clone()),
            CompiledFilter::Root => Ok(root.clone()),
            CompiledFilter::Property { base, property } => {
                let base_value = self.evaluate_filter(base, current, root, context)?;
                self.get_property(&base_value, property)
            }
            CompiledFilter::Index { base, index } => {
                let base_value = self.evaluate_filter(base, current, root, context)?;
                self.get_index(&base_value, *index)
            }
            CompiledFilter::Literal(literal) => Ok(self.literal_to_value(literal)),
            CompiledFilter::Binary { left, operator, right } => {
                self.evaluate_binary(left, *operator, right, current, root, context)
            }
            CompiledFilter::Unary { operator, operand } => {
                self.evaluate_unary(*operator, operand, current, root, context)
            }
            CompiledFilter::Function { function, args } => {
                self.evaluate_function(function, args, current, root, context)
            }
            CompiledFilter::Exists { path } => {
                let path_result = self.evaluate_filter(path, current, root, context);
                Ok(Value::Bool(path_result.is_ok()))
            }
        }
    }

    /// Evaluate binary operation with short-circuit evaluation
    fn evaluate_binary(
        &self,
        left: &CompiledFilter,
        operator: BinaryOperator,
        right: &CompiledFilter,
        current: &Value,
        root: &Value,
        context: &FilterContext,
    ) -> Result<Value> {
        match operator {
            BinaryOperator::And => {
                let left_val = self.evaluate_filter(left, current, root, context)?;
                if !is_truthy(&left_val) {
                    return Ok(Value::Bool(false));
                }
                let right_val = self.evaluate_filter(right, current, root, context)?;
                Ok(Value::Bool(is_truthy(&right_val)))
            }
            BinaryOperator::Or => {
                let left_val = self.evaluate_filter(left, current, root, context)?;
                if is_truthy(&left_val) {
                    return Ok(Value::Bool(true));
                }
                let right_val = self.evaluate_filter(right, current, root, context)?;
                Ok(Value::Bool(is_truthy(&right_val)))
            }
            _ => {
                let left_val = self.evaluate_filter(left, current, root, context)?;
                let right_val = self.evaluate_filter(right, current, root, context)?;
                self.apply_binary_operator(&left_val, operator, &right_val)
            }
        }
    }

    /// Apply binary operator to two values
    fn apply_binary_operator(
        &self,
        left: &Value,
        operator: BinaryOperator,
        right: &Value,
    ) -> Result<Value> {
        match operator {
            BinaryOperator::Equal => Ok(Value::Bool(values_equal(left, right))),
            BinaryOperator::NotEqual => Ok(Value::Bool(!values_equal(left, right))),
            BinaryOperator::LessThan => self.compare_values(left, right, |ord| ord == Ordering::Less),
            BinaryOperator::LessThanOrEqual => {
                self.compare_values(left, right, |ord| ord != Ordering::Greater)
            }
            BinaryOperator::GreaterThan => {
                self.compare_values(left, right, |ord| ord == Ordering::Greater)
            }
            BinaryOperator::GreaterThanOrEqual => {
                self.compare_values(left, right, |ord| ord != Ordering::Less)
            }
            BinaryOperator::RegexMatch => self.regex_match(left, right),
            BinaryOperator::In => self.value_in(left, right),
            BinaryOperator::And | BinaryOperator::Or => {
                // These should be handled in evaluate_binary for short-circuit evaluation
                unreachable!("And/Or operators should be handled separately")
            }
        }
    }

    /// Compare two values with the given comparison function
    fn compare_values<F>(&self, left: &Value, right: &Value, compare: F) -> Result<Value>
    where
        F: Fn(Ordering) -> bool,
    {
        let ordering = self.value_ordering(left, right)?;
        Ok(Value::Bool(compare(ordering)))
    }

    /// Get ordering between two values
    fn value_ordering(&self, left: &Value, right: &Value) -> Result<Ordering> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => {
                let a_f64 = a.as_f64().unwrap_or(0.0);
                let b_f64 = b.as_f64().unwrap_or(0.0);
                Ok(a_f64.partial_cmp(&b_f64).unwrap_or(Ordering::Equal))
            }
            (Value::String(a), Value::String(b)) => Ok(a.cmp(b)),
            (Value::Bool(a), Value::Bool(b)) => Ok(a.cmp(b)),
            (Value::Null, Value::Null) => Ok(Ordering::Equal),
            _ => Err(JSONPathError::type_mismatch(
                "comparable types",
                format!("{} and {}", value_type_name(left), value_type_name(right)),
                "filter comparison",
            ).into()),
        }
    }

    /// Perform regex matching
    fn regex_match(&self, left: &Value, right: &Value) -> Result<Value> {
        let text = match left {
            Value::String(s) => s,
            _ => {
                return Err(JSONPathError::type_mismatch(
                    "string",
                    value_type_name(left),
                    "regex match left operand",
                ).into());
            }
        };

        let pattern = match right {
            Value::String(s) => s,
            _ => {
                return Err(JSONPathError::type_mismatch(
                    "string",
                    value_type_name(right),
                    "regex match right operand",
                ).into());
            }
        };

        // Simple pattern matching (full regex would require additional dependencies)
        // For now, implement basic wildcard matching
        let matches = if pattern.contains('*') {
            simple_wildcard_match(text, pattern)
        } else {
            text.contains(pattern)
        };

        Ok(Value::Bool(matches))
    }

    /// Check if left value is contained in right value
    fn value_in(&self, left: &Value, right: &Value) -> Result<Value> {
        match right {
            Value::Array(array) => {
                let found = array.iter().any(|item| values_equal(left, item));
                Ok(Value::Bool(found))
            }
            Value::String(s) => {
                if let Value::String(substr) = left {
                    Ok(Value::Bool(s.contains(substr)))
                } else {
                    Ok(Value::Bool(false))
                }
            }
            Value::Object(obj) => {
                if let Value::String(key) = left {
                    Ok(Value::Bool(obj.contains_key(key)))
                } else {
                    Ok(Value::Bool(false))
                }
            }
            _ => Ok(Value::Bool(false)),
        }
    }

    /// Evaluate unary operation
    fn evaluate_unary(
        &self,
        operator: UnaryOperator,
        operand: &CompiledFilter,
        current: &Value,
        root: &Value,
        context: &FilterContext,
    ) -> Result<Value> {
        let operand_val = self.evaluate_filter(operand, current, root, context)?;

        match operator {
            UnaryOperator::Not => Ok(Value::Bool(!is_truthy(&operand_val))),
            UnaryOperator::Negate => match operand_val {
                Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        Ok(Value::Number(serde_json::Number::from(-i)))
                    } else if let Some(f) = n.as_f64() {
                        Ok(Value::Number(
                            serde_json::Number::from_f64(-f).unwrap_or(serde_json::Number::from(0)),
                        ))
                    } else {
                        Ok(Value::Number(serde_json::Number::from(0)))
                    }
                }
                _ => Err(JSONPathError::type_mismatch(
                    "number",
                    value_type_name(&operand_val),
                    "negation operator",
                ).into()),
            },
        }
    }

    /// Evaluate function call
    fn evaluate_function(
        &self,
        function: &FilterFunction,
        args: &[CompiledFilter],
        current: &Value,
        root: &Value,
        context: &FilterContext,
    ) -> Result<Value> {
        let mut evaluated_args = Vec::new();
        for arg in args {
            evaluated_args.push(self.evaluate_filter(arg, current, root, context)?);
        }

        match function {
            FilterFunction::Length | FilterFunction::Size => {
                if evaluated_args.is_empty() {
                    self.get_length(current)
                } else {
                    self.get_length(&evaluated_args[0])
                }
            }
            FilterFunction::Type => {
                let target = if evaluated_args.is_empty() {
                    current
                } else {
                    &evaluated_args[0]
                };
                Ok(Value::String(value_type_name(target).to_string()))
            }
            FilterFunction::Keys => {
                let target = if evaluated_args.is_empty() {
                    current
                } else {
                    &evaluated_args[0]
                };
                self.get_keys(target)
            }
            FilterFunction::Values => {
                let target = if evaluated_args.is_empty() {
                    current
                } else {
                    &evaluated_args[0]
                };
                self.get_values(target)
            }
            FilterFunction::Custom { implementation, .. } => {
                implementation(&evaluated_args).map_err(Into::into)
            }
        }
    }

    /// Get property from value
    fn get_property(&self, value: &Value, property: &str) -> Result<Value> {
        match value {
            Value::Object(obj) => {
                obj.get(property)
                    .cloned()
                    .ok_or_else(|| {
                        JSONPathError::execution(
                            format!("Property '{}' not found", property),
                            format!("@.{}", property),
                            Some("object".to_string()),
                        ).into()
                    })
            }
            _ => Err(JSONPathError::type_mismatch(
                "object",
                value_type_name(value),
                format!("property access '{}'", property),
            ).into()),
        }
    }

    /// Get index from value
    fn get_index(&self, value: &Value, index: i64) -> Result<Value> {
        match value {
            Value::Array(array) => {
                let effective_index = if index < 0 {
                    let len = array.len() as i64;
                    if -index <= len {
                        (len + index) as usize
                    } else {
                        return Err(JSONPathError::index_out_of_bounds(
                            index,
                            array.len(),
                            format!("@[{}]", index),
                        ).into());
                    }
                } else {
                    index as usize
                };

                array
                    .get(effective_index)
                    .cloned()
                    .ok_or_else(|| {
                        JSONPathError::index_out_of_bounds(
                            index,
                            array.len(),
                            format!("@[{}]", index),
                        ).into()
                    })
            }
            _ => Err(JSONPathError::type_mismatch(
                "array",
                value_type_name(value),
                format!("index access [{}]", index),
            ).into()),
        }
    }

    /// Get length of value
    fn get_length(&self, value: &Value) -> Result<Value> {
        let length = match value {
            Value::Array(array) => array.len(),
            Value::Object(obj) => obj.len(),
            Value::String(s) => s.chars().count(),
            _ => {
                return Err(JSONPathError::type_mismatch(
                    "array, object, or string",
                    value_type_name(value),
                    "length() function",
                ).into());
            }
        };
        Ok(Value::Number(serde_json::Number::from(length)))
    }

    /// Get keys of an object
    fn get_keys(&self, value: &Value) -> Result<Value> {
        match value {
            Value::Object(obj) => {
                let keys: Vec<Value> = obj.keys().map(|k| Value::String(k.clone())).collect();
                Ok(Value::Array(keys))
            }
            _ => Err(JSONPathError::type_mismatch(
                "object",
                value_type_name(value),
                "keys() function",
            ).into()),
        }
    }

    /// Get values of an object or array
    fn get_values(&self, value: &Value) -> Result<Value> {
        match value {
            Value::Object(obj) => {
                let values: Vec<Value> = obj.values().cloned().collect();
                Ok(Value::Array(values))
            }
            Value::Array(array) => Ok(Value::Array(array.clone())),
            _ => Err(JSONPathError::type_mismatch(
                "object or array",
                value_type_name(value),
                "values() function",
            ).into()),
        }
    }

    /// Convert literal to JSON value
    fn literal_to_value(&self, literal: &FilterLiteral) -> Value {
        match literal {
            FilterLiteral::String(s) => Value::String(s.clone()),
            FilterLiteral::Number(n) => {
                Value::Number(serde_json::Number::from_f64(*n).unwrap_or(serde_json::Number::from(0)))
            }
            FilterLiteral::Boolean(b) => Value::Bool(*b),
            FilterLiteral::Null => Value::Null,
        }
    }
}

/// Context for filter evaluation
pub struct FilterContext {
    // Additional variables or state can be added here in the future
}

impl FilterContext {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for FilterContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a value is truthy in filter context
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
        Value::String(s) => !s.is_empty(),
        Value::Array(arr) => !arr.is_empty(),
        Value::Object(obj) => !obj.is_empty(),
    }
}

/// Check if two values are equal
fn values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => {
            // Handle numeric comparison carefully
            match (a.as_i64(), b.as_i64()) {
                (Some(ai), Some(bi)) => ai == bi,
                _ => {
                    let af = a.as_f64().unwrap_or(0.0);
                    let bf = b.as_f64().unwrap_or(0.0);
                    (af - bf).abs() < f64::EPSILON
                }
            }
        }
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Array(a), Value::Array(b)) => a == b,
        (Value::Object(a), Value::Object(b)) => a == b,
        _ => false,
    }
}

/// Get the type name of a value
fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Simple wildcard matching (supports * only)
fn simple_wildcard_match(text: &str, pattern: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    
    if parts.len() == 1 {
        // No wildcards, exact match
        return text == pattern;
    }
    
    let mut text_pos = 0;
    
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        
        if i == 0 {
            // First part must match start
            if !text[text_pos..].starts_with(part) {
                return false;
            }
            text_pos += part.len();
        } else if i == parts.len() - 1 {
            // Last part must match end
            if !text[text_pos..].ends_with(part) {
                return false;
            }
        } else {
            // Middle part must be found
            if let Some(pos) = text[text_pos..].find(part) {
                text_pos += pos + part.len();
            } else {
                return false;
            }
        }
    }
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_is_truthy() {
        assert!(!is_truthy(&Value::Null));
        assert!(!is_truthy(&Value::Bool(false)));
        assert!(is_truthy(&Value::Bool(true)));
        assert!(!is_truthy(&Value::Number(serde_json::Number::from(0))));
        assert!(is_truthy(&Value::Number(serde_json::Number::from(1))));
        assert!(!is_truthy(&Value::String("".to_string())));
        assert!(is_truthy(&Value::String("test".to_string())));
        assert!(!is_truthy(&Value::Array(vec![])));
        assert!(is_truthy(&Value::Array(vec![Value::Null])));
        assert!(!is_truthy(&json!({})));
        assert!(is_truthy(&json!({"a": 1})));
    }

    #[test]
    fn test_values_equal() {
        assert!(values_equal(&Value::Null, &Value::Null));
        assert!(values_equal(&Value::Bool(true), &Value::Bool(true)));
        assert!(!values_equal(&Value::Bool(true), &Value::Bool(false)));
        assert!(values_equal(
            &Value::Number(serde_json::Number::from(42)),
            &Value::Number(serde_json::Number::from(42))
        ));
        assert!(values_equal(
            &Value::String("test".to_string()),
            &Value::String("test".to_string())
        ));
        assert!(!values_equal(&Value::String("a".to_string()), &Value::String("b".to_string())));
    }

    #[test]
    fn test_value_type_name() {
        assert_eq!(value_type_name(&Value::Null), "null");
        assert_eq!(value_type_name(&Value::Bool(true)), "boolean");
        assert_eq!(value_type_name(&Value::Number(serde_json::Number::from(1))), "number");
        assert_eq!(value_type_name(&Value::String("test".to_string())), "string");
        assert_eq!(value_type_name(&Value::Array(vec![])), "array");
        assert_eq!(value_type_name(&json!({})), "object");
    }

    #[test]
    fn test_simple_wildcard_match() {
        assert!(simple_wildcard_match("hello world", "hello*"));
        assert!(simple_wildcard_match("hello world", "*world"));
        assert!(simple_wildcard_match("hello world", "hello*world"));
        assert!(simple_wildcard_match("hello world", "*"));
        assert!(!simple_wildcard_match("hello world", "hi*"));
        assert!(!simple_wildcard_match("hello world", "*universe"));
    }

    #[test]
    fn test_filter_executor_simple() {
        let data = json!([
            {"price": 8.95},
            {"price": 12.99},
            {"price": 8.99}
        ]);

        // Create a filter: @.price < 10
        let filter = CompiledFilter::Binary {
            left: Box::new(CompiledFilter::Property {
                base: Box::new(CompiledFilter::Current),
                property: "price".into(),
            }),
            operator: BinaryOperator::LessThan,
            right: Box::new(CompiledFilter::Literal(FilterLiteral::Number(10.0))),
        };

        let executor = FilterExecutor::new(filter);
        let input = SelectionIterator::new(vec![&data[0], &data[1], &data[2]]);
        let results = executor.execute(input).unwrap();
        let collected: Vec<_> = results.collect();

        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0]["price"], json!(8.95));
        assert_eq!(collected[1]["price"], json!(8.99));
    }

    #[test]
    fn test_filter_functions() {
        let data = json!({"items": [1, 2, 3]});

        let filter_executor = FilterExecutor::new(CompiledFilter::Current);
        let context = FilterContext::new();

        // Test length function
        let length_result = filter_executor
            .evaluate_function(
                &FilterFunction::Length,
                &[CompiledFilter::Property {
                    base: Box::new(CompiledFilter::Current),
                    property: "items".into(),
                }],
                &data,
                &data,
                &context,
            )
            .unwrap();
        assert_eq!(length_result, json!(3));

        // Test type function
        let type_result = filter_executor
            .evaluate_function(
                &FilterFunction::Type,
                &[CompiledFilter::Current],
                &data,
                &data,
                &context,
            )
            .unwrap();
        assert_eq!(type_result, json!("object"));
    }

    #[test]
    fn test_binary_operators() {
        let executor = FilterExecutor::new(CompiledFilter::Current);
        
        // Test equality
        let result = executor.apply_binary_operator(
            &json!(42),
            BinaryOperator::Equal,
            &json!(42),
        ).unwrap();
        assert_eq!(result, json!(true));

        // Test inequality
        let result = executor.apply_binary_operator(
            &json!(42),
            BinaryOperator::NotEqual,
            &json!(24),
        ).unwrap();
        assert_eq!(result, json!(true));

        // Test comparison
        let result = executor.apply_binary_operator(
            &json!(10),
            BinaryOperator::LessThan,
            &json!(20),
        ).unwrap();
        assert_eq!(result, json!(true));
    }
}