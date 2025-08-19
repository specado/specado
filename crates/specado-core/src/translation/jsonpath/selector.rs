//! Selector implementations for JSONPath traversal
//!
//! This module provides efficient implementations for each type of JSONPath
//! selector with minimal allocations and iterator-based traversal.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use super::ast::*;
use super::error::*;
use crate::Result;
use serde_json::Value;
use std::borrow::Cow;

/// Selection result - a reference to a value in the JSON structure
pub type SelectionResult<'a> = &'a Value;

/// Selection iterator for lazy evaluation
pub struct SelectionIterator<'a> {
    values: Vec<&'a Value>,
    index: usize,
}

impl<'a> SelectionIterator<'a> {
    /// Create a new selection iterator
    pub fn new(values: Vec<&'a Value>) -> Self {
        Self { values, index: 0 }
    }

    /// Create an empty iterator
    pub fn empty() -> Self {
        Self {
            values: Vec::new(),
            index: 0,
        }
    }

    /// Create a single-value iterator
    pub fn single(value: &'a Value) -> Self {
        Self {
            values: vec![value],
            index: 0,
        }
    }

    /// Collect all remaining values
    pub fn collect(self) -> Vec<&'a Value> {
        self.values
    }

    /// Check if iterator is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Get the count of values
    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl<'a> Iterator for SelectionIterator<'a> {
    type Item = &'a Value;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.values.len() {
            let value = self.values[self.index];
            self.index += 1;
            Some(value)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.values.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for SelectionIterator<'a> {}

/// Trait for selector execution
pub trait SelectorExecutor {
    /// Execute this selector on a set of input values
    fn execute<'a>(&self, inputs: SelectionIterator<'a>) -> Result<SelectionIterator<'a>>;
}

/// Root selector implementation
pub struct RootSelector;

impl SelectorExecutor for RootSelector {
    fn execute<'a>(&self, inputs: SelectionIterator<'a>) -> Result<SelectionIterator<'a>> {
        // Root selector is a no-op, just pass through
        Ok(inputs)
    }
}

/// Property selector implementation
pub struct PropertySelector {
    pub key: Cow<'static, str>,
}

impl PropertySelector {
    pub fn new(key: impl Into<Cow<'static, str>>) -> Self {
        Self { key: key.into() }
    }
}

impl SelectorExecutor for PropertySelector {
    fn execute<'a>(&self, inputs: SelectionIterator<'a>) -> Result<SelectionIterator<'a>> {
        let mut results = Vec::new();

        for value in inputs {
            if let Some(obj) = value.as_object() {
                if let Some(child) = obj.get(self.key.as_ref()) {
                    results.push(child);
                }
            }
        }

        Ok(SelectionIterator::new(results))
    }
}

/// Array index selector implementation
pub struct IndexSelector {
    pub index: ArrayIndex,
}

impl IndexSelector {
    pub fn new(index: ArrayIndex) -> Self {
        Self { index }
    }

    pub fn positive(index: usize) -> Self {
        Self {
            index: ArrayIndex::Positive(index),
        }
    }

    pub fn negative(index: usize) -> Self {
        Self {
            index: ArrayIndex::Negative(index),
        }
    }
}

impl SelectorExecutor for IndexSelector {
    fn execute<'a>(&self, inputs: SelectionIterator<'a>) -> Result<SelectionIterator<'a>> {
        let mut results = Vec::new();

        for value in inputs {
            if let Some(array) = value.as_array() {
                let effective_index = match self.index {
                    ArrayIndex::Positive(idx) => {
                        if idx < array.len() {
                            Some(idx)
                        } else {
                            None
                        }
                    }
                    ArrayIndex::Negative(idx) => {
                        if idx <= array.len() && idx > 0 {
                            Some(array.len() - idx)
                        } else {
                            None
                        }
                    }
                };

                if let Some(idx) = effective_index {
                    results.push(&array[idx]);
                }
            }
        }

        Ok(SelectionIterator::new(results))
    }
}

/// Array slice selector implementation
pub struct SliceSelector {
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub step: i64,
}

impl SliceSelector {
    pub fn new(start: Option<i64>, end: Option<i64>, step: i64) -> Self {
        Self { start, end, step }
    }

    /// Calculate effective slice bounds for an array
    fn calculate_bounds(&self, length: usize) -> (usize, usize, bool) {
        let len = length as i64;
        
        let start = match self.start {
            Some(s) if s < 0 => (len + s).max(0) as usize,
            Some(s) => (s as usize).min(length),
            None => if self.step > 0 { 0 } else { length.saturating_sub(1) },
        };

        let end = match self.end {
            Some(e) if e < 0 => (len + e).max(0) as usize,
            Some(e) => (e as usize).min(length),
            None => if self.step > 0 { length } else { 0 },
        };

        let reverse = self.step < 0;
        (start, end, reverse)
    }
}

impl SelectorExecutor for SliceSelector {
    fn execute<'a>(&self, inputs: SelectionIterator<'a>) -> Result<SelectionIterator<'a>> {
        if self.step == 0 {
            return Err(JSONPathError::execution(
                "Slice step cannot be zero",
                "slice",
                Some("array".to_string()),
            ).into());
        }

        let mut results = Vec::new();

        for value in inputs {
            if let Some(array) = value.as_array() {
                let (start, end, reverse) = self.calculate_bounds(array.len());
                
                if reverse {
                    // Negative step - iterate backwards
                    let step = (-self.step) as usize;
                    let mut i = start;
                    while i > end {
                        results.push(&array[i]);
                        if i < step {
                            break;
                        }
                        i -= step;
                    }
                } else {
                    // Positive step - iterate forwards
                    let step = self.step as usize;
                    let mut i = start;
                    while i < end {
                        results.push(&array[i]);
                        i += step;
                    }
                }
            }
        }

        Ok(SelectionIterator::new(results))
    }
}

/// Wildcard selector implementation
pub struct WildcardSelector;

impl SelectorExecutor for WildcardSelector {
    fn execute<'a>(&self, inputs: SelectionIterator<'a>) -> Result<SelectionIterator<'a>> {
        let mut results = Vec::new();

        for value in inputs {
            match value {
                Value::Object(obj) => {
                    for child in obj.values() {
                        results.push(child);
                    }
                }
                Value::Array(array) => {
                    for child in array {
                        results.push(child);
                    }
                }
                _ => {
                    // Wildcards don't match primitive values
                }
            }
        }

        Ok(SelectionIterator::new(results))
    }
}

/// Recursive descent selector implementation
pub struct RecursiveDescentSelector {
    pub target: Option<Cow<'static, str>>,
}

impl Default for RecursiveDescentSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl RecursiveDescentSelector {
    pub fn new() -> Self {
        Self { target: None }
    }

    pub fn with_target(target: impl Into<Cow<'static, str>>) -> Self {
        Self {
            target: Some(target.into()),
        }
    }

    /// Recursively collect all values matching the criteria
    fn collect_recursive<'a>(&self, value: &'a Value, results: &mut Vec<&'a Value>) {
        match value {
            Value::Object(obj) => {
                if let Some(ref target_key) = self.target {
                    // Looking for specific property
                    if let Some(target_value) = obj.get(target_key.as_ref()) {
                        results.push(target_value);
                    }
                    // Continue recursion into all children
                    for child in obj.values() {
                        self.collect_recursive(child, results);
                    }
                } else {
                    // Collect all descendants
                    for child in obj.values() {
                        results.push(child);
                        self.collect_recursive(child, results);
                    }
                }
            }
            Value::Array(array) => {
                for child in array {
                    if self.target.is_none() {
                        results.push(child);
                    }
                    self.collect_recursive(child, results);
                }
            }
            _ => {
                // Primitive values don't have children
            }
        }
    }
}

impl SelectorExecutor for RecursiveDescentSelector {
    fn execute<'a>(&self, inputs: SelectionIterator<'a>) -> Result<SelectionIterator<'a>> {
        let mut results = Vec::new();

        for value in inputs {
            self.collect_recursive(value, &mut results);
        }

        Ok(SelectionIterator::new(results))
    }
}

/// Union selector implementation
pub struct UnionSelector {
    pub selectors: Vec<Box<dyn SelectorExecutor>>,
}

impl UnionSelector {
    pub fn new(selectors: Vec<Box<dyn SelectorExecutor>>) -> Self {
        Self { selectors }
    }
}

impl SelectorExecutor for UnionSelector {
    fn execute<'a>(&self, inputs: SelectionIterator<'a>) -> Result<SelectionIterator<'a>> {
        let mut results = Vec::new();
        let input_values: Vec<&'a Value> = inputs.collect();

        for selector in &self.selectors {
            let input_iter = SelectionIterator::new(input_values.clone());
            let selector_results = selector.execute(input_iter)?;
            results.extend(selector_results.collect());
        }

        // Remove duplicates while preserving order
        // Note: This is a simple implementation that doesn't distinguish between 
        // different references to the same value. For most use cases, this is acceptable.
        let mut unique_results = Vec::new();
        for result in results {
            if !unique_results.contains(&result) {
                unique_results.push(result);
            }
        }

        Ok(SelectionIterator::new(unique_results))
    }
}

/// Create a selector executor from a compiled selector
pub fn create_selector_executor(selector: &CompiledSelector) -> Box<dyn SelectorExecutor> {
    match selector {
        CompiledSelector::Root => Box::new(RootSelector),
        CompiledSelector::Property { key, .. } => {
            Box::new(PropertySelector::new(key.clone()))
        }
        CompiledSelector::Index { index } => {
            Box::new(IndexSelector::new(index.clone()))
        }
        CompiledSelector::Slice { start, end, step } => {
            Box::new(SliceSelector::new(*start, *end, *step))
        }
        CompiledSelector::Wildcard => Box::new(WildcardSelector),
        CompiledSelector::RecursiveDescent { target } => {
            if let Some(target_key) = target {
                Box::new(RecursiveDescentSelector::with_target(target_key.clone()))
            } else {
                Box::new(RecursiveDescentSelector::new())
            }
        }
        CompiledSelector::Union { selectors } => {
            let executor_selectors: Vec<Box<dyn SelectorExecutor>> = selectors
                .iter()
                .map(create_selector_executor)
                .collect();
            Box::new(UnionSelector::new(executor_selectors))
        }
        CompiledSelector::Filter { .. } => {
            // Filter execution is handled separately in the filter module
            panic!("Filter selectors should be handled by FilterExecutor")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_property_selector() {
        let data = json!({
            "store": {
                "book": "value"
            }
        });

        let selector = PropertySelector::new("store");
        let input = SelectionIterator::single(&data);
        let results = selector.execute(input).unwrap();
        let collected: Vec<_> = results.collect();
        
        assert_eq!(collected.len(), 1);
        assert!(collected[0].is_object());
    }

    #[test]
    fn test_index_selector_positive() {
        let data = json!(["first", "second", "third"]);

        let selector = IndexSelector::positive(1);
        let input = SelectionIterator::single(&data);
        let results = selector.execute(input).unwrap();
        let collected: Vec<_> = results.collect();
        
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0], &json!("second"));
    }

    #[test]
    fn test_index_selector_negative() {
        let data = json!(["first", "second", "third"]);

        let selector = IndexSelector::negative(1);
        let input = SelectionIterator::single(&data);
        let results = selector.execute(input).unwrap();
        let collected: Vec<_> = results.collect();
        
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0], &json!("third"));
    }

    #[test]
    fn test_slice_selector() {
        let data = json!(["a", "b", "c", "d", "e"]);

        let selector = SliceSelector::new(Some(1), Some(4), 1);
        let input = SelectionIterator::single(&data);
        let results = selector.execute(input).unwrap();
        let collected: Vec<_> = results.collect();
        
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], &json!("b"));
        assert_eq!(collected[1], &json!("c"));
        assert_eq!(collected[2], &json!("d"));
    }

    #[test]
    fn test_slice_selector_step() {
        let data = json!(["a", "b", "c", "d", "e"]);

        let selector = SliceSelector::new(Some(0), None, 2);
        let input = SelectionIterator::single(&data);
        let results = selector.execute(input).unwrap();
        let collected: Vec<_> = results.collect();
        
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], &json!("a"));
        assert_eq!(collected[1], &json!("c"));
        assert_eq!(collected[2], &json!("e"));
    }

    #[test]
    fn test_wildcard_selector_object() {
        let data = json!({
            "a": 1,
            "b": 2,
            "c": 3
        });

        let selector = WildcardSelector;
        let input = SelectionIterator::single(&data);
        let results = selector.execute(input).unwrap();
        let collected: Vec<_> = results.collect();
        
        assert_eq!(collected.len(), 3);
    }

    #[test]
    fn test_wildcard_selector_array() {
        let data = json!([1, 2, 3]);

        let selector = WildcardSelector;
        let input = SelectionIterator::single(&data);
        let results = selector.execute(input).unwrap();
        let collected: Vec<_> = results.collect();
        
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], &json!(1));
        assert_eq!(collected[1], &json!(2));
        assert_eq!(collected[2], &json!(3));
    }

    #[test]
    fn test_recursive_descent_selector() {
        let data = json!({
            "store": {
                "book": [
                    {"author": "Author 1"},
                    {"author": "Author 2"}
                ],
                "author": "Store Author"
            }
        });

        let selector = RecursiveDescentSelector::with_target("author");
        let input = SelectionIterator::single(&data);
        let results = selector.execute(input).unwrap();
        let collected: Vec<_> = results.collect();
        
        assert_eq!(collected.len(), 3);
    }

    #[test]
    fn test_recursive_descent_all() {
        let data = json!({
            "a": {
                "b": 1,
                "c": 2
            }
        });

        let selector = RecursiveDescentSelector::new();
        let input = SelectionIterator::single(&data);
        let results = selector.execute(input).unwrap();
        let collected: Vec<_> = results.collect();
        
        // Should collect the object, then its properties
        assert!(collected.len() >= 3);
    }

    #[test]
    fn test_selection_iterator() {
        let data = json!([1, 2, 3]);
        let values = vec![&data[0], &data[1], &data[2]];
        let mut iter = SelectionIterator::new(values);
        
        assert_eq!(iter.len(), 3);
        assert!(!iter.is_empty());
        
        assert_eq!(iter.next(), Some(&json!(1)));
        assert_eq!(iter.next(), Some(&json!(2)));
        assert_eq!(iter.next(), Some(&json!(3)));
        assert_eq!(iter.next(), None);
    }
}
