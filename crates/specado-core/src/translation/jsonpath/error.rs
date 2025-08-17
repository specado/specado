//! Error types for JSONPath operations
//!
//! This module provides comprehensive error handling for JSONPath parsing
//! and execution with detailed context and recovery suggestions.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use thiserror::Error;
use std::fmt;

/// Comprehensive JSONPath error types
#[derive(Error, Debug, Clone)]
pub enum JSONPathError {
    /// Parse errors during JSONPath expression parsing
    #[error("Parse error at position {position}: {message}")]
    Parse {
        message: String,
        position: usize,
        input: String,
        context: Option<String>,
    },

    /// Path validation errors
    #[error("Invalid path: {message}")]
    InvalidPath {
        message: String,
        path: String,
        suggestion: Option<String>,
    },

    /// Index out of bounds errors
    #[error("Index out of bounds: {index} in array of length {length}")]
    IndexOutOfBounds {
        index: i64,
        length: usize,
        path: String,
    },

    /// Type mismatch errors
    #[error("Type mismatch: expected {expected}, found {found} at {path}")]
    TypeMismatch {
        expected: String,
        found: String,
        path: String,
    },

    /// Filter evaluation errors
    #[error("Filter evaluation failed: {message}")]
    FilterEvaluation {
        message: String,
        filter: String,
        context: Option<String>,
    },

    /// Function execution errors
    #[error("Function error: {function}() - {message}")]
    Function {
        function: String,
        message: String,
        args: Vec<String>,
    },

    /// Expression compilation errors
    #[error("Compilation error: {message}")]
    Compilation {
        message: String,
        expression: String,
    },

    /// Runtime execution errors
    #[error("Execution error: {message}")]
    Execution {
        message: String,
        path: String,
        value_type: Option<String>,
    },

    /// Optimization errors
    #[error("Optimization error: {message}")]
    Optimization {
        message: String,
        optimization: String,
    },

    /// Unsupported feature errors
    #[error("Unsupported feature: {feature}")]
    Unsupported {
        feature: String,
        alternative: Option<String>,
    },

    /// Syntax errors with detailed position information
    #[error("Syntax error: {message}")]
    Syntax {
        message: String,
        position: usize,
        input: String,
        expected: Vec<String>,
        found: String,
    },

    /// Semantic errors in expressions
    #[error("Semantic error: {message}")]
    Semantic {
        message: String,
        expression: String,
        context: String,
    },
}

/// Position information for error reporting
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    /// Zero-based byte offset
    pub offset: usize,
    /// One-based line number
    pub line: usize,
    /// One-based column number
    pub column: usize,
}

/// Context information for error recovery
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// The input being parsed
    pub input: String,
    /// Current position in input
    pub position: Position,
    /// Expected tokens at this position
    pub expected: Vec<String>,
    /// Actually found token
    pub found: String,
    /// Additional context message
    pub message: Option<String>,
}

impl JSONPathError {
    /// Create a parse error with position and context
    pub fn parse(message: impl Into<String>, position: usize, input: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
            position,
            input: input.into(),
            context: None,
        }
    }

    /// Create a parse error with additional context
    pub fn parse_with_context(
        message: impl Into<String>,
        position: usize,
        input: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self::Parse {
            message: message.into(),
            position,
            input: input.into(),
            context: Some(context.into()),
        }
    }

    /// Create a syntax error with detailed information
    pub fn syntax(
        message: impl Into<String>,
        position: usize,
        input: impl Into<String>,
        expected: Vec<String>,
        found: impl Into<String>,
    ) -> Self {
        Self::Syntax {
            message: message.into(),
            position,
            input: input.into(),
            expected,
            found: found.into(),
        }
    }

    /// Create an invalid path error with suggestion
    pub fn invalid_path(
        message: impl Into<String>,
        path: impl Into<String>,
        suggestion: Option<String>,
    ) -> Self {
        Self::InvalidPath {
            message: message.into(),
            path: path.into(),
            suggestion,
        }
    }

    /// Create an index out of bounds error
    pub fn index_out_of_bounds(index: i64, length: usize, path: impl Into<String>) -> Self {
        Self::IndexOutOfBounds {
            index,
            length,
            path: path.into(),
        }
    }

    /// Create a type mismatch error
    pub fn type_mismatch(
        expected: impl Into<String>,
        found: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self::TypeMismatch {
            expected: expected.into(),
            found: found.into(),
            path: path.into(),
        }
    }

    /// Create a filter evaluation error
    pub fn filter_evaluation(
        message: impl Into<String>,
        filter: impl Into<String>,
        context: Option<String>,
    ) -> Self {
        Self::FilterEvaluation {
            message: message.into(),
            filter: filter.into(),
            context,
        }
    }

    /// Create a function error
    pub fn function(
        function: impl Into<String>,
        message: impl Into<String>,
        args: Vec<String>,
    ) -> Self {
        Self::Function {
            function: function.into(),
            message: message.into(),
            args,
        }
    }

    /// Create a compilation error
    pub fn compilation(
        message: impl Into<String>,
        expression: impl Into<String>,
    ) -> Self {
        Self::Compilation {
            message: message.into(),
            expression: expression.into(),
        }
    }

    /// Create an execution error
    pub fn execution(
        message: impl Into<String>,
        path: impl Into<String>,
        value_type: Option<String>,
    ) -> Self {
        Self::Execution {
            message: message.into(),
            path: path.into(),
            value_type,
        }
    }

    /// Create an unsupported feature error
    pub fn unsupported(feature: impl Into<String>, alternative: Option<String>) -> Self {
        Self::Unsupported {
            feature: feature.into(),
            alternative,
        }
    }

    /// Get the error message with formatted context
    pub fn detailed_message(&self) -> String {
        match self {
            Self::Parse { message, position, input, context } => {
                let mut result = format!("Parse error at position {}: {}", position, message);
                if let Some(ctx) = context {
                    result.push_str(&format!("\nContext: {}", ctx));
                }
                if !input.is_empty() {
                    result.push_str(&format!("\nInput: {}", input));
                    if *position < input.len() {
                        result.push_str(&format!(
                            "\n       {}^",
                            " ".repeat(*position)
                        ));
                    }
                }
                result
            }
            Self::Syntax { message, position, input, expected, found } => {
                let mut result = format!("Syntax error at position {}: {}", position, message);
                result.push_str(&format!("\nExpected one of: {}", expected.join(", ")));
                result.push_str(&format!("\nFound: {}", found));
                if !input.is_empty() && *position < input.len() {
                    result.push_str(&format!("\nInput: {}", input));
                    result.push_str(&format!(
                        "\n       {}^",
                        " ".repeat(*position)
                    ));
                }
                result
            }
            Self::InvalidPath { message, path, suggestion } => {
                let mut result = format!("Invalid path '{}': {}", path, message);
                if let Some(suggestion) = suggestion {
                    result.push_str(&format!("\nSuggestion: {}", suggestion));
                }
                result
            }
            _ => self.to_string(),
        }
    }

    /// Get recovery suggestions for this error
    pub fn recovery_suggestions(&self) -> Vec<String> {
        match self {
            Self::Parse { .. } => vec![
                "Check for unclosed brackets or quotes".to_string(),
                "Verify JSONPath syntax is correct".to_string(),
            ],
            Self::InvalidPath { suggestion, .. } => {
                if let Some(s) = suggestion {
                    vec![s.clone()]
                } else {
                    vec!["Check path syntax and property names".to_string()]
                }
            }
            Self::IndexOutOfBounds { index, length, .. } => {
                if *index < 0 {
                    vec![format!("Use positive index 0-{}", length.saturating_sub(1))]
                } else {
                    vec![format!("Use index in range 0-{}", length.saturating_sub(1))]
                }
            }
            Self::TypeMismatch { expected, .. } => {
                vec![format!("Ensure the value is of type {}", expected)]
            }
            Self::FilterEvaluation { .. } => vec![
                "Check filter syntax and property references".to_string(),
                "Ensure all referenced properties exist".to_string(),
            ],
            Self::Function { function, .. } => {
                vec![format!("Check {} function arguments and usage", function)]
            }
            Self::Unsupported { alternative, .. } => {
                if let Some(alt) = alternative {
                    vec![format!("Use {} instead", alt)]
                } else {
                    vec!["This feature is not currently supported".to_string()]
                }
            }
            _ => vec!["Check JSONPath expression syntax".to_string()],
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(self, 
            Self::Parse { .. } | 
            Self::InvalidPath { .. } | 
            Self::Syntax { .. }
        )
    }

    /// Get the error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Parse { .. } | Self::Syntax { .. } => ErrorSeverity::Error,
            Self::InvalidPath { .. } => ErrorSeverity::Warning,
            Self::IndexOutOfBounds { .. } => ErrorSeverity::Warning,
            Self::TypeMismatch { .. } => ErrorSeverity::Warning,
            Self::FilterEvaluation { .. } => ErrorSeverity::Error,
            Self::Function { .. } => ErrorSeverity::Error,
            Self::Compilation { .. } => ErrorSeverity::Error,
            Self::Execution { .. } => ErrorSeverity::Warning,
            Self::Optimization { .. } => ErrorSeverity::Info,
            Self::Unsupported { .. } => ErrorSeverity::Error,
            Self::Semantic { .. } => ErrorSeverity::Error,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Informational message
    Info,
    /// Warning - operation may continue with degraded functionality
    Warning,
    /// Error - operation cannot continue
    Error,
    /// Critical - system integrity at risk
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Info => write!(f, "INFO"),
            ErrorSeverity::Warning => write!(f, "WARNING"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl Position {
    /// Create a new position
    pub fn new(offset: usize, line: usize, column: usize) -> Self {
        Self { offset, line, column }
    }

    /// Create position from offset in input
    pub fn from_offset(input: &str, offset: usize) -> Self {
        let mut line = 1;
        let mut column = 1;
        
        for (i, ch) in input.char_indices() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        
        Self { offset, line, column }
    }
}

impl ErrorContext {
    /// Create new error context
    pub fn new(
        input: String,
        position: Position,
        expected: Vec<String>,
        found: String,
    ) -> Self {
        Self {
            input,
            position,
            expected,
            found,
            message: None,
        }
    }

    /// Add a context message
    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    /// Format the error context for display
    pub fn format(&self) -> String {
        let mut result = String::new();
        
        if let Some(msg) = &self.message {
            result.push_str(&format!("Context: {}\n", msg));
        }
        
        result.push_str(&format!(
            "At line {}, column {}: Expected {}, found {}",
            self.position.line,
            self.position.column,
            self.expected.join(" or "),
            self.found
        ));
        
        if !self.input.is_empty() {
            result.push_str(&format!("\n{}", self.input));
            if self.position.offset < self.input.len() {
                result.push_str(&format!(
                    "\n{}^",
                    " ".repeat(self.position.column.saturating_sub(1))
                ));
            }
        }
        
        result
    }
}

// Convert JSONPathError to the main Error type
impl From<JSONPathError> for crate::Error {
    fn from(err: JSONPathError) -> Self {
        crate::Error::Translation {
            message: err.to_string(),
            context: Some("JSONPath".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_creation() {
        let err = JSONPathError::parse("Invalid token", 5, "$.test[");
        match err {
            JSONPathError::Parse { message, position, input, .. } => {
                assert_eq!(message, "Invalid token");
                assert_eq!(position, 5);
                assert_eq!(input, "$.test[");
            }
            _ => panic!("Expected parse error"),
        }
    }

    #[test]
    fn test_detailed_message() {
        let err = JSONPathError::syntax(
            "Unexpected token",
            3,
            "$.{test",
            vec!["property name".to_string()],
            "{"
        );
        
        let detailed = err.detailed_message();
        assert!(detailed.contains("Syntax error"));
        assert!(detailed.contains("position 3"));
        assert!(detailed.contains("Expected one of: property name"));
        assert!(detailed.contains("Found: {"));
    }

    #[test]
    fn test_recovery_suggestions() {
        let err = JSONPathError::index_out_of_bounds(5, 3, "$.array[5]");
        let suggestions = err.recovery_suggestions();
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].contains("0-2"));
    }

    #[test]
    fn test_error_severity() {
        let parse_err = JSONPathError::parse("test", 0, "");
        assert_eq!(parse_err.severity(), ErrorSeverity::Error);
        
        let index_err = JSONPathError::index_out_of_bounds(1, 0, "");
        assert_eq!(index_err.severity(), ErrorSeverity::Warning);
    }

    #[test]
    fn test_position_from_offset() {
        let input = "line1\nline2\nline3";
        let pos = Position::from_offset(input, 8); // 'i' in "line2"
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 3);
        assert_eq!(pos.offset, 8);
    }
}