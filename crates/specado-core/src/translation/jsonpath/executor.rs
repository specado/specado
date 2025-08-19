//! JSONPath expression executor
//!
//! This module provides the main execution engine for compiled JSONPath
//! expressions with performance optimizations and comprehensive error handling.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use super::ast::*;
use super::error::*;
use super::filter::FilterExecutor;
use super::selector::{create_selector_executor, SelectionIterator, SelectorExecutor};
use crate::Result;
use serde_json::Value;

/// High-performance JSONPath expression executor
pub struct Executor {
    /// Execution context and configuration
    context: ExecutionContext,
}

/// Execution context for controlling execution behavior
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Maximum recursion depth to prevent stack overflow
    pub max_depth: usize,
    /// Maximum number of results to return (0 = unlimited)
    pub max_results: usize,
    /// Whether to short-circuit on first result
    pub short_circuit: bool,
    /// Whether to collect results eagerly or lazily
    pub eager_evaluation: bool,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            max_depth: 100,
            max_results: 0,
            short_circuit: false,
            eager_evaluation: true,
        }
    }
}

impl Executor {
    /// Create a new executor with default context
    pub fn new() -> Self {
        Self {
            context: ExecutionContext::default(),
        }
    }

    /// Create a new executor with custom context
    pub fn with_context(context: ExecutionContext) -> Self {
        Self { context }
    }

    /// Execute a compiled JSONPath expression against data
    pub fn execute<'a>(
        &self,
        expression: &CompiledExpression,
        data: &'a Value,
    ) -> Result<Vec<&'a Value>> {
        // Quick optimization for simple property access
        if expression.is_simple_property() {
            return self.execute_simple_property(expression, data);
        }

        // Full execution path
        let mut current_results = SelectionIterator::single(data);
        for (depth, selector) in expression.selectors.iter().enumerate() {
            if depth >= self.context.max_depth {
                return Err(JSONPathError::execution(
                    "Maximum recursion depth exceeded",
                    "execution",
                    Some("depth limit".to_string()),
                ).into());
            }

            current_results = self.execute_selector(selector, current_results, depth)?;

            // Short-circuit if we have no results
            if current_results.is_empty() && !self.context.short_circuit {
                break;
            }

            // Limit results if configured
            if self.context.max_results > 0 && current_results.len() > self.context.max_results {
                let limited_values: Vec<_> = current_results
                    .collect()
                    .into_iter()
                    .take(self.context.max_results)
                    .collect();
                current_results = SelectionIterator::new(limited_values);
            }
        }

        Ok(current_results.collect())
    }

    /// Execute a single selector
    fn execute_selector<'a>(
        &self,
        selector: &CompiledSelector,
        inputs: SelectionIterator<'a>,
        depth: usize,
    ) -> Result<SelectionIterator<'a>> {
        match selector {
            CompiledSelector::Filter { filter } => {
                let filter_executor = FilterExecutor::new(filter.clone());
                filter_executor.execute(inputs)
            }
            CompiledSelector::RecursiveDescent { .. } => {
                // Recursive descent needs special depth tracking
                if depth >= self.context.max_depth - 10 {
                    return Err(JSONPathError::execution(
                        "Approaching maximum recursion depth in recursive descent",
                        "recursive_descent",
                        Some("depth warning".to_string()),
                    ).into());
                }
                let executor = create_selector_executor(selector);
                executor.execute(inputs)
            }
            _ => {
                let executor = create_selector_executor(selector);
                executor.execute(inputs)
            }
        }
    }

    /// Optimized execution for simple property access
    fn execute_simple_property<'a>(
        &self,
        expression: &CompiledExpression,
        data: &'a Value,
    ) -> Result<Vec<&'a Value>> {
        if let Some(property_name) = expression.simple_property_name() {
            if let Some(obj) = data.as_object() {
                if let Some(value) = obj.get(property_name) {
                    return Ok(vec![value]);
                }
            }
        }
        Ok(vec![])
    }

    /// Execute with streaming results (for very large datasets)
    pub fn execute_streaming<'a>(
        &self,
        expression: &CompiledExpression,
        data: &'a Value,
    ) -> Result<ExecutionStream<'a>> {
        let results = self.execute(expression, data)?;
        Ok(ExecutionStream::new(results))
    }

    /// Execute and return only the first result
    pub fn execute_first<'a>(
        &self,
        expression: &CompiledExpression,
        data: &'a Value,
    ) -> Result<Option<&'a Value>> {
        let executor = Self::with_context(ExecutionContext {
            short_circuit: true,
            max_results: 1,
            ..self.context.clone()
        });
        
        let results = executor.execute(expression, data)?;
        Ok(results.into_iter().next())
    }

    /// Execute and check if any results exist
    pub fn exists(&self, expression: &CompiledExpression, data: &Value) -> Result<bool> {
        let executor = Self::with_context(ExecutionContext {
            short_circuit: true,
            max_results: 1,
            ..self.context.clone()
        });
        
        let results = executor.execute(expression, data)?;
        Ok(!results.is_empty())
    }

    /// Execute and count results without materializing them
    pub fn count(&self, expression: &CompiledExpression, data: &Value) -> Result<usize> {
        let results = self.execute(expression, data)?;
        Ok(results.len())
    }

    /// Execute with detailed performance metrics
    pub fn execute_with_metrics<'a>(
        &self,
        expression: &CompiledExpression,
        data: &'a Value,
    ) -> Result<(Vec<&'a Value>, ExecutionMetrics)> {
        let start_time = std::time::Instant::now();
        let mut metrics = ExecutionMetrics::new();

        metrics.expression_complexity = expression.metadata.complexity;
        metrics.selectors_count = expression.selectors.len();

        let results = self.execute(expression, data)?;

        metrics.execution_time = start_time.elapsed();
        metrics.results_count = results.len();
        metrics.max_depth_reached = self.calculate_max_depth_reached(expression);

        Ok((results, metrics))
    }

    /// Calculate the maximum depth reached during execution
    fn calculate_max_depth_reached(&self, expression: &CompiledExpression) -> usize {
        let mut max_depth = 0;
        for selector in &expression.selectors {
            match selector {
                CompiledSelector::RecursiveDescent { .. } => max_depth += 10, // Estimate
                _ => max_depth += 1,
            }
        }
        max_depth
    }

    /// Validate an expression before execution
    pub fn validate_expression(expression: &CompiledExpression) -> Result<()> {
        // Check for obvious issues
        if expression.selectors.is_empty() {
            return Err(JSONPathError::compilation(
                "Expression has no selectors",
                "empty expression",
            ).into());
        }

        // Validate each selector
        for (i, selector) in expression.selectors.iter().enumerate() {
            Self::validate_selector(selector, i)?;
        }

        // Check for potential performance issues
        if expression.metadata.complexity > 0.8 {
            // This is just a warning, not an error
            log::warn!("High complexity JSONPath expression detected: {}", expression.metadata.complexity);
        }

        Ok(())
    }

    /// Validate a single selector
    fn validate_selector(selector: &CompiledSelector, position: usize) -> Result<()> {
        match selector {
            CompiledSelector::Root => {
                if position != 0 {
                    return Err(JSONPathError::compilation(
                        "Root selector can only appear at the beginning",
                        format!("selector at position {}", position),
                    ).into());
                }
            }
            CompiledSelector::Slice { start, end, step } => {
                if *step == 0 {
                    return Err(JSONPathError::compilation(
                        "Slice step cannot be zero",
                        format!("slice at position {}", position),
                    ).into());
                }
                if let (Some(s), Some(e)) = (start, end) {
                    if *step > 0 && s >= e {
                        return Err(JSONPathError::compilation(
                            "Invalid slice bounds with positive step",
                            format!("slice [{}:{}:{}] at position {}", s, e, step, position),
                        ).into());
                    }
                }
            }
            CompiledSelector::Union { selectors } => {
                if selectors.is_empty() {
                    return Err(JSONPathError::compilation(
                        "Union selector cannot be empty",
                        format!("union at position {}", position),
                    ).into());
                }
            }
            _ => {} // Other selectors are always valid
        }
        Ok(())
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

/// Streaming execution results for memory-efficient processing
pub struct ExecutionStream<'a> {
    results: std::vec::IntoIter<&'a Value>,
}

impl<'a> ExecutionStream<'a> {
    fn new(results: Vec<&'a Value>) -> Self {
        Self {
            results: results.into_iter(),
        }
    }

    /// Collect remaining results
    pub fn collect(self) -> Vec<&'a Value> {
        self.results.collect()
    }

    /// Take up to n results
    pub fn take(self, n: usize) -> impl Iterator<Item = &'a Value> {
        self.results.take(n)
    }

    /// Filter results with a predicate
    pub fn filter<F>(self, predicate: F) -> impl Iterator<Item = &'a Value>
    where
        F: FnMut(&&'a Value) -> bool,
    {
        self.results.filter(predicate)
    }
}

impl<'a> Iterator for ExecutionStream<'a> {
    type Item = &'a Value;

    fn next(&mut self) -> Option<Self::Item> {
        self.results.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.results.size_hint()
    }
}

/// Execution performance metrics
#[derive(Debug, Clone)]
pub struct ExecutionMetrics {
    /// Time taken to execute the expression
    pub execution_time: std::time::Duration,
    /// Number of selectors in the expression
    pub selectors_count: usize,
    /// Complexity score of the expression
    pub expression_complexity: f32,
    /// Number of results found
    pub results_count: usize,
    /// Maximum depth reached during execution
    pub max_depth_reached: usize,
    /// Whether execution was short-circuited
    pub short_circuited: bool,
}

impl ExecutionMetrics {
    fn new() -> Self {
        Self {
            execution_time: std::time::Duration::from_nanos(0),
            selectors_count: 0,
            expression_complexity: 0.0,
            results_count: 0,
            max_depth_reached: 0,
            short_circuited: false,
        }
    }

    /// Get execution time in milliseconds
    pub fn execution_time_ms(&self) -> f64 {
        self.execution_time.as_secs_f64() * 1000.0
    }

    /// Get execution time in microseconds
    pub fn execution_time_us(&self) -> f64 {
        self.execution_time.as_secs_f64() * 1_000_000.0
    }

    /// Calculate results per millisecond
    pub fn results_per_ms(&self) -> f64 {
        if self.execution_time.as_millis() == 0 {
            self.results_count as f64
        } else {
            self.results_count as f64 / self.execution_time_ms()
        }
    }

    /// Get a performance rating (0.0 = slow, 1.0 = fast)
    pub fn performance_rating(&self) -> f64 {
        let time_score = if self.execution_time_ms() < 1.0 {
            1.0
        } else if self.execution_time_ms() < 10.0 {
            0.8
        } else if self.execution_time_ms() < 100.0 {
            0.6
        } else {
            0.3
        };

        let complexity_score = 1.0 - self.expression_complexity as f64;
        
        (time_score + complexity_score) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translation::jsonpath::{CompiledExpression, CompiledSelector, ArrayIndex, ExpressionMetadata};
    use serde_json::json;

    #[test]
    fn test_executor_simple_property() {
        let data = json!({
            "store": {
                "book": "value"
            }
        });

        let expression = CompiledExpression::new(
            vec![
                CompiledSelector::Root,
                CompiledSelector::Property {
                    key: "store".into(),
                    is_dynamic: false,
                },
            ],
            ExpressionMetadata::default(),
        );

        let executor = Executor::new();
        let results = executor.execute(&expression, &data).unwrap();
        
        assert_eq!(results.len(), 1);
        assert!(results[0].is_object());
    }

    #[test]
    fn test_executor_array_index() {
        let data = json!(["first", "second", "third"]);

        let expression = CompiledExpression::new(
            vec![
                CompiledSelector::Root,
                CompiledSelector::Index {
                    index: ArrayIndex::Positive(1),
                },
            ],
            ExpressionMetadata::default(),
        );

        let executor = Executor::new();
        let results = executor.execute(&expression, &data).unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], &json!("second"));
    }

    #[test]
    fn test_executor_wildcard() {
        let data = json!({
            "a": 1,
            "b": 2,
            "c": 3
        });

        let expression = CompiledExpression::new(
            vec![
                CompiledSelector::Root,
                CompiledSelector::Wildcard,
            ],
            ExpressionMetadata::default(),
        );

        let executor = Executor::new();
        let results = executor.execute(&expression, &data).unwrap();
        
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_executor_slice() {
        let data = json!(["a", "b", "c", "d", "e"]);

        let expression = CompiledExpression::new(
            vec![
                CompiledSelector::Root,
                CompiledSelector::Slice {
                    start: Some(1),
                    end: Some(4),
                    step: 1,
                },
            ],
            ExpressionMetadata::default(),
        );

        let executor = Executor::new();
        let results = executor.execute(&expression, &data).unwrap();
        
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], &json!("b"));
        assert_eq!(results[1], &json!("c"));
        assert_eq!(results[2], &json!("d"));
    }

    #[test]
    fn test_executor_exists() {
        let data = json!({
            "store": {
                "book": "value"
            }
        });

        let expression = CompiledExpression::new(
            vec![
                CompiledSelector::Root,
                CompiledSelector::Property {
                    key: "store".into(),
                    is_dynamic: false,
                },
            ],
            ExpressionMetadata::default(),
        );

        let executor = Executor::new();
        assert!(executor.exists(&expression, &data).unwrap());

        let missing_expression = CompiledExpression::new(
            vec![
                CompiledSelector::Root,
                CompiledSelector::Property {
                    key: "missing".into(),
                    is_dynamic: false,
                },
            ],
            ExpressionMetadata::default(),
        );

        assert!(!executor.exists(&missing_expression, &data).unwrap());
    }

    #[test]
    fn test_executor_first() {
        let data = json!([1, 2, 3]);

        let expression = CompiledExpression::new(
            vec![
                CompiledSelector::Root,
                CompiledSelector::Wildcard,
            ],
            ExpressionMetadata::default(),
        );

        let executor = Executor::new();
        let result = executor.execute_first(&expression, &data).unwrap();
        
        assert!(result.is_some());
        assert_eq!(result.unwrap(), &json!(1));
    }

    #[test]
    fn test_executor_count() {
        let data = json!([1, 2, 3, 4, 5]);

        let expression = CompiledExpression::new(
            vec![
                CompiledSelector::Root,
                CompiledSelector::Wildcard,
            ],
            ExpressionMetadata::default(),
        );

        let executor = Executor::new();
        let count = executor.count(&expression, &data).unwrap();
        
        assert_eq!(count, 5);
    }

    #[test]
    fn test_executor_with_metrics() {
        let data = json!({
            "items": [1, 2, 3, 4, 5]
        });

        let expression = CompiledExpression::new(
            vec![
                CompiledSelector::Root,
                CompiledSelector::Property {
                    key: "items".into(),
                    is_dynamic: false,
                },
                CompiledSelector::Wildcard,
            ],
            ExpressionMetadata {
                complexity: 0.3,
                ..Default::default()
            },
        );

        let executor = Executor::new();
        let (results, metrics) = executor.execute_with_metrics(&expression, &data).unwrap();
        
        assert_eq!(results.len(), 5);
        assert_eq!(metrics.results_count, 5);
        assert_eq!(metrics.selectors_count, 3);
        assert_eq!(metrics.expression_complexity, 0.3);
        assert!(metrics.execution_time.as_nanos() > 0);
    }

    #[test]
    fn test_execution_context_limits() {
        let data = json!([1, 2, 3, 4, 5]);

        let expression = CompiledExpression::new(
            vec![
                CompiledSelector::Root,
                CompiledSelector::Wildcard,
            ],
            ExpressionMetadata::default(),
        );

        let context = ExecutionContext {
            max_results: 3,
            ..Default::default()
        };

        let executor = Executor::with_context(context);
        let results = executor.execute(&expression, &data).unwrap();
        
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_execution_stream() {
        let data = json!([1, 2, 3, 4, 5]);

        let expression = CompiledExpression::new(
            vec![
                CompiledSelector::Root,
                CompiledSelector::Wildcard,
            ],
            ExpressionMetadata::default(),
        );

        let executor = Executor::new();
        let stream = executor.execute_streaming(&expression, &data).unwrap();
        
        let collected: Vec<_> = stream.take(3).collect();
        assert_eq!(collected.len(), 3);
    }

    #[test]
    fn test_validate_expression() {
        // Valid expression
        let valid_expression = CompiledExpression::new(
            vec![
                CompiledSelector::Root,
                CompiledSelector::Property {
                    key: "test".into(),
                    is_dynamic: false,
                },
            ],
            ExpressionMetadata::default(),
        );
        assert!(Executor::validate_expression(&valid_expression).is_ok());

        // Invalid expression - empty selectors
        let empty_expression = CompiledExpression::new(
            vec![],
            ExpressionMetadata::default(),
        );
        assert!(Executor::validate_expression(&empty_expression).is_err());

        // Invalid expression - zero step slice
        let invalid_slice = CompiledExpression::new(
            vec![
                CompiledSelector::Root,
                CompiledSelector::Slice {
                    start: Some(0),
                    end: Some(5),
                    step: 0,
                },
            ],
            ExpressionMetadata::default(),
        );
        assert!(Executor::validate_expression(&invalid_slice).is_err());
    }
}
