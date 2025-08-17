//! Diff engine for comparing JSON values with smart comparison

use crate::{GoldenError, Result};
use colored::*;
use regex::Regex;
use serde_json::Value;
use similar::{ChangeTag, TextDiff};
use std::collections::HashSet;

/// Options for diff comparison
#[derive(Debug, Clone)]
pub struct DiffOptions {
    /// Whether to use colored output
    pub colored: bool,
    
    /// Context lines to show around changes
    pub context_lines: usize,
    
    /// Whether to normalize JSON before comparison
    pub normalize: bool,
    
    /// Tolerance for floating point comparison
    pub float_tolerance: f64,
    
    /// Whether to show full diff or just summary
    pub full_diff: bool,
    
    /// Maximum diff lines to show (0 = unlimited)
    pub max_diff_lines: usize,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            colored: true,
            context_lines: 3,
            normalize: true,
            float_tolerance: 1e-6,
            full_diff: true,
            max_diff_lines: 100,
        }
    }
}

/// Result of a diff operation
#[derive(Debug)]
pub struct DiffResult {
    /// Whether the values match
    pub matches: bool,
    
    /// Human-readable diff output
    pub diff_output: String,
    
    /// Summary of changes
    pub summary: DiffSummary,
}

/// Summary of diff changes
#[derive(Debug, Default)]
pub struct DiffSummary {
    /// Number of added lines
    pub added: usize,
    
    /// Number of removed lines
    pub removed: usize,
    
    /// Number of changed lines
    pub changed: usize,
    
    /// Paths that differ
    pub differing_paths: Vec<String>,
}

/// Engine for comparing JSON values
pub struct DiffEngine {
    options: DiffOptions,
    volatile_patterns: Vec<(String, Regex)>,
}

impl DiffEngine {
    /// Create a new diff engine
    pub fn new(options: DiffOptions) -> Self {
        Self {
            options,
            volatile_patterns: Vec::new(),
        }
    }
    
    /// Add a volatile field pattern
    pub fn add_volatile_pattern(&mut self, path: &str, pattern: &str) -> Result<()> {
        let regex = Regex::new(pattern)
            .map_err(|e| GoldenError::CorpusError(format!("Invalid regex pattern: {}", e)))?;
        
        self.volatile_patterns.push((path.to_string(), regex));
        Ok(())
    }
    
    /// Compare two JSON values
    pub fn compare(&self, expected: &Value, actual: &Value) -> DiffResult {
        // Normalize if requested
        let (expected_normalized, actual_normalized) = if self.options.normalize {
            (
                crate::snapshot::normalize_json(expected),
                crate::snapshot::normalize_json(actual),
            )
        } else {
            (expected.clone(), actual.clone())
        };
        
        // Apply volatile field masking
        let expected_masked = self.mask_volatile_fields(&expected_normalized);
        let actual_masked = self.mask_volatile_fields(&actual_normalized);
        
        // Perform structural comparison
        let structural_match = self.values_match(&expected_masked, &actual_masked);
        
        // Generate diff output
        let diff_output = if structural_match {
            String::new()
        } else {
            self.generate_diff_output(&expected_masked, &actual_masked)
        };
        
        // Collect summary
        let summary = if structural_match {
            DiffSummary::default()
        } else {
            self.collect_diff_summary(&expected_masked, &actual_masked)
        };
        
        DiffResult {
            matches: structural_match,
            diff_output,
            summary,
        }
    }
    
    /// Check if two values match structurally
    fn values_match(&self, expected: &Value, actual: &Value) -> bool {
        match (expected, actual) {
            (Value::Object(exp), Value::Object(act)) => {
                if exp.len() != act.len() {
                    return false;
                }
                
                for (key, exp_val) in exp {
                    match act.get(key) {
                        Some(act_val) => {
                            if !self.values_match(exp_val, act_val) {
                                return false;
                            }
                        }
                        None => return false,
                    }
                }
                
                true
            }
            (Value::Array(exp), Value::Array(act)) => {
                if exp.len() != act.len() {
                    return false;
                }
                
                for (exp_val, act_val) in exp.iter().zip(act.iter()) {
                    if !self.values_match(exp_val, act_val) {
                        return false;
                    }
                }
                
                true
            }
            (Value::Number(exp), Value::Number(act)) => {
                if let (Some(exp_f), Some(act_f)) = (exp.as_f64(), act.as_f64()) {
                    (exp_f - act_f).abs() <= self.options.float_tolerance
                } else {
                    exp == act
                }
            }
            (exp, act) => exp == act,
        }
    }
    
    /// Generate human-readable diff output
    fn generate_diff_output(&self, expected: &Value, actual: &Value) -> String {
        let expected_str = serde_json::to_string_pretty(expected).unwrap();
        let actual_str = serde_json::to_string_pretty(actual).unwrap();
        
        let text_diff = TextDiff::from_lines(&expected_str, &actual_str);
        let mut output = String::new();
        
        if self.options.colored {
            output.push_str(&"=== Diff Output ===\n".bold().to_string());
        } else {
            output.push_str("=== Diff Output ===\n");
        }
        
        let mut line_count = 0;
        
        for change in text_diff.iter_all_changes() {
            if self.options.max_diff_lines > 0 && line_count >= self.options.max_diff_lines {
                output.push_str("... (diff truncated) ...\n");
                break;
            }
            
            let line = match change.tag() {
                ChangeTag::Delete => {
                    if self.options.colored {
                        format!("{}{}", "-".red(), change.to_string().red())
                    } else {
                        format!("-{}", change)
                    }
                }
                ChangeTag::Insert => {
                    if self.options.colored {
                        format!("{}{}", "+".green(), change.to_string().green())
                    } else {
                        format!("+{}", change)
                    }
                }
                ChangeTag::Equal => {
                    if self.options.full_diff || line_count < self.options.context_lines {
                        format!(" {}", change)
                    } else {
                        continue;
                    }
                }
            };
            
            output.push_str(&line);
            line_count += 1;
        }
        
        output
    }
    
    /// Collect summary of differences
    fn collect_diff_summary(&self, expected: &Value, actual: &Value) -> DiffSummary {
        let mut summary = DiffSummary::default();
        
        self.collect_diff_paths(expected, actual, String::new(), &mut summary.differing_paths);
        
        // Count line differences
        let expected_str = serde_json::to_string_pretty(expected).unwrap();
        let actual_str = serde_json::to_string_pretty(actual).unwrap();
        let text_diff = TextDiff::from_lines(&expected_str, &actual_str);
        
        for change in text_diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Delete => summary.removed += 1,
                ChangeTag::Insert => summary.added += 1,
                ChangeTag::Equal => {}
            }
        }
        
        summary.changed = summary.differing_paths.len();
        summary
    }
    
    /// Recursively collect paths that differ
    fn collect_diff_paths(&self, expected: &Value, actual: &Value, path: String, paths: &mut Vec<String>) {
        match (expected, actual) {
            (Value::Object(exp), Value::Object(act)) => {
                let all_keys: HashSet<_> = exp.keys().chain(act.keys()).collect();
                
                for key in all_keys {
                    let new_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    
                    match (exp.get(key), act.get(key)) {
                        (Some(exp_val), Some(act_val)) => {
                            if !self.values_match(exp_val, act_val) {
                                self.collect_diff_paths(exp_val, act_val, new_path, paths);
                            }
                        }
                        (Some(_), None) => paths.push(format!("{} (missing in actual)", new_path)),
                        (None, Some(_)) => paths.push(format!("{} (extra in actual)", new_path)),
                        (None, None) => {}
                    }
                }
            }
            (Value::Array(exp), Value::Array(act)) => {
                for (i, (exp_val, act_val)) in exp.iter().zip(act.iter()).enumerate() {
                    let new_path = format!("{}[{}]", path, i);
                    if !self.values_match(exp_val, act_val) {
                        self.collect_diff_paths(exp_val, act_val, new_path, paths);
                    }
                }
                
                if exp.len() != act.len() {
                    paths.push(format!("{} (array length mismatch: {} vs {})", path, exp.len(), act.len()));
                }
            }
            _ => {
                if !self.values_match(expected, actual) {
                    paths.push(path);
                }
            }
        }
    }
    
    /// Mask volatile fields in a value
    fn mask_volatile_fields(&self, value: &Value) -> Value {
        let mut masked = value.clone();
        
        for (path, pattern) in &self.volatile_patterns {
            self.mask_field_by_pattern(&mut masked, path, pattern);
        }
        
        masked
    }
    
    /// Mask a specific field if it matches the pattern
    fn mask_field_by_pattern(&self, value: &mut Value, path: &str, pattern: &Regex) {
        let parts: Vec<&str> = path.split('.').filter(|s| !s.is_empty()).collect();
        
        if parts.is_empty() {
            return;
        }
        
        self.mask_field_recursive(value, &parts, pattern);
    }
    
    fn mask_field_recursive(&self, value: &mut Value, path_parts: &[&str], pattern: &Regex) {
        if path_parts.is_empty() {
            return;
        }
        
        let (first, rest) = path_parts.split_first().unwrap();
        
        match value {
            Value::Object(map) => {
                if rest.is_empty() {
                    // Check and mask the field if it matches the pattern
                    if let Some(field_value) = map.get_mut(*first) {
                        if let Value::String(s) = field_value {
                            if pattern.is_match(s) {
                                *field_value = Value::String("***MASKED***".to_string());
                            }
                        }
                    }
                } else if let Some(next_value) = map.get_mut(*first) {
                    // Continue traversing
                    self.mask_field_recursive(next_value, rest, pattern);
                }
            }
            Value::Array(arr) => {
                // Apply to all array elements
                for item in arr {
                    self.mask_field_recursive(item, path_parts, pattern);
                }
            }
            _ => {}
        }
    }
    
    /// Create a simple text diff for error messages
    pub fn simple_diff(&self, expected: &str, actual: &str) -> String {
        let diff = TextDiff::from_lines(expected, actual);
        let mut output = String::new();
        
        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            output.push_str(&format!("{}{}", sign, change));
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_values_match_exact() {
        let engine = DiffEngine::new(DiffOptions::default());
        
        let val1 = json!({"a": 1, "b": "test"});
        let val2 = json!({"a": 1, "b": "test"});
        
        assert!(engine.values_match(&val1, &val2));
    }
    
    #[test]
    fn test_values_match_float_tolerance() {
        let engine = DiffEngine::new(DiffOptions {
            float_tolerance: 0.001,
            ..Default::default()
        });
        
        let val1 = json!({"pi": 3.14159});
        let val2 = json!({"pi": 3.14160});
        
        assert!(engine.values_match(&val1, &val2));
    }
    
    #[test]
    fn test_values_mismatch() {
        let engine = DiffEngine::new(DiffOptions::default());
        
        let val1 = json!({"a": 1});
        let val2 = json!({"a": 2});
        
        assert!(!engine.values_match(&val1, &val2));
    }
    
    #[test]
    fn test_diff_summary() {
        let engine = DiffEngine::new(DiffOptions::default());
        
        let expected = json!({
            "model": "gpt-4",
            "temperature": 0.7,
            "messages": []
        });
        
        let actual = json!({
            "model": "gpt-3.5",
            "temperature": 0.7,
            "messages": [],
            "extra": "field"
        });
        
        let result = engine.compare(&expected, &actual);
        
        assert!(!result.matches);
        assert!(result.summary.differing_paths.len() > 0);
    }
    
    #[test]
    fn test_volatile_field_masking() {
        let mut engine = DiffEngine::new(DiffOptions::default());
        engine.add_volatile_pattern("timestamp", r"^\d{4}-\d{2}-\d{2}").unwrap();
        
        let val1 = json!({"timestamp": "2025-01-01", "data": "test"});
        let val2 = json!({"timestamp": "2025-01-02", "data": "test"});
        
        let masked1 = engine.mask_volatile_fields(&val1);
        let masked2 = engine.mask_volatile_fields(&val2);
        
        assert_eq!(masked1["timestamp"], "***MASKED***");
        assert_eq!(masked2["timestamp"], "***MASKED***");
    }
}