//! High-performance JSONPath mapping engine for field translation
//!
//! This module provides a comprehensive JSONPath implementation for mapping
//! data between uniform and provider-specific formats with minimal allocations
//! and maximum performance.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

pub mod ast;
pub mod error;
pub mod executor;
pub mod filter;
pub mod optimizer;
pub mod parser;
pub mod selector;

pub use ast::{Expression, CompiledExpression, ExpressionMetadata, Selector, CompiledSelector, ArrayIndex};
pub use error::{JSONPathError, ErrorSeverity};
pub use executor::{Executor, ExecutionContext, ExecutionMetrics};
pub use filter::{FilterExecutor, FilterContext};
pub use optimizer::{Optimizer, OptimizerConfig};
pub use parser::Parser;

use crate::Result;
use serde_json::Value;

/// High-level JSONPath expression for convenient API usage
#[derive(Debug, Clone)]
pub struct JSONPath {
    expression: CompiledExpression,
}

impl JSONPath {
    /// Parse and compile a JSONPath expression
    pub fn parse(path: &str) -> Result<Self> {
        let expression = Parser::new(path)?.parse()?;
        let compiled = Optimizer::new().optimize(expression)?;
        Ok(Self {
            expression: compiled,
        })
    }

    /// Execute the JSONPath against the given data
    pub fn execute<'a>(&self, data: &'a Value) -> Result<Vec<&'a Value>> {
        Executor::new().execute(&self.expression, data)
    }

    /// Execute and return the first match, if any
    pub fn execute_single<'a>(&self, data: &'a Value) -> Result<Option<&'a Value>> {
        let results = self.execute(data)?;
        Ok(results.into_iter().next())
    }

    /// Execute and collect results into owned values
    pub fn execute_owned(&self, data: &Value) -> Result<Vec<Value>> {
        let results = self.execute(data)?;
        Ok(results.into_iter().cloned().collect())
    }

    /// Check if the path exists in the data
    pub fn exists(&self, data: &Value) -> Result<bool> {
        let results = self.execute(data)?;
        Ok(!results.is_empty())
    }

    /// Get the raw expression for inspection
    pub fn expression(&self) -> &CompiledExpression {
        &self.expression
    }
}

impl std::fmt::Display for JSONPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expression)
    }
}

/// Convenient functions for direct JSONPath operations
pub fn select<'a>(path: &str, data: &'a Value) -> Result<Vec<&'a Value>> {
    let jsonpath = JSONPath::parse(path)?;
    jsonpath.execute(data)
}

pub fn select_single<'a>(path: &str, data: &'a Value) -> Result<Option<&'a Value>> {
    let jsonpath = JSONPath::parse(path)?;
    jsonpath.execute_single(data)
}

pub fn exists(path: &str, data: &Value) -> Result<bool> {
    let jsonpath = JSONPath::parse(path)?;
    jsonpath.exists(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_dot_notation() {
        let data = json!({
            "store": {
                "book": [
                    {"title": "Book 1", "price": 10.99},
                    {"title": "Book 2", "price": 8.95}
                ]
            }
        });

        let results = select("$.store.book", &data).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_array());
    }

    #[test]
    fn test_bracket_notation() {
        let data = json!({
            "store": {
                "book": [
                    {"title": "Book 1", "price": 10.99},
                    {"title": "Book 2", "price": 8.95}
                ]
            }
        });

        let results = select("$['store']['book']", &data).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_array());
    }

    #[test]
    fn test_array_index() {
        let data = json!({
            "books": [
                {"title": "Book 1"},
                {"title": "Book 2"},
                {"title": "Book 3"}
            ]
        });

        let results = select("$.books[0]", &data).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["title"], "Book 1");
    }

    #[test]
    fn test_wildcard() {
        let data = json!({
            "store": {
                "book": {"title": "Book"},
                "movie": {"title": "Movie"}
            }
        });

        let results = select("$.store.*", &data).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_recursive_descent() {
        let data = json!({
            "store": {
                "book": [
                    {"author": "Author 1"},
                    {"author": "Author 2"}
                ],
                "author": "Store Author"
            }
        });

        let results = select("$..author", &data).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_exists() {
        let data = json!({
            "store": {
                "book": {"title": "Book"}
            }
        });

        assert!(exists("$.store.book", &data).unwrap());
        assert!(!exists("$.store.movie", &data).unwrap());
    }

    #[test]
    fn test_select_single() {
        let data = json!({
            "title": "Test Book"
        });

        let result = select_single("$.title", &data).unwrap();
        assert_eq!(result, Some(&json!("Test Book")));

        let result = select_single("$.missing", &data).unwrap();
        assert_eq!(result, None);
    }
}
