//! Comprehensive logging utilities for the Specado CLI
//!
//! This module provides:
//! - Request ID generation and tracking
//! - Sensitive data redaction
//! - Performance timing spans
//! - Structured logging setup
//! - Multiple output formats (console, JSON)

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use tracing::{field, Span};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

/// Global request ID for the current session
static REQUEST_ID: OnceLock<String> = OnceLock::new();

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level filter
    pub level: String,
    /// Output format: compact, full, json
    pub format: LogFormat,
    /// Enable console output
    pub console: bool,
    /// Optional file output path
    pub file: Option<PathBuf>,
    /// Include timestamps
    pub timestamps: bool,
    /// Include thread IDs
    pub thread_ids: bool,
    /// Include file and line numbers
    pub source_location: bool,
    /// Include span events
    pub span_events: bool,
    /// Module-based filtering
    pub module_filter: Option<HashMap<String, String>>,
}

/// Log output format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogFormat {
    /// Compact format for production
    Compact,
    /// Full format with all details
    Full,
    /// JSON structured format
    Json,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Compact,
            console: true,
            file: None,
            timestamps: true,
            thread_ids: false,
            source_location: false,
            span_events: false,
            module_filter: None,
        }
    }
}

impl LoggingConfig {
    /// Create logging config from verbosity level
    pub fn from_verbosity(verbosity: u8) -> Self {
        let mut config = Self::default();
        
        match verbosity {
            0 => {
                config.level = "warn".to_string();
            }
            1 => {
                config.level = "info".to_string();
            }
            2 => {
                config.level = "debug".to_string();
                config.source_location = true;
            }
            _ => {
                config.level = "trace".to_string();
                config.format = LogFormat::Full;
                config.source_location = true;
                config.thread_ids = true;
                config.span_events = true;
            }
        }
        
        config
    }
    
    /// Merge with user config, applying environment overrides
    pub fn merge_with_env(&mut self) {
        // RUST_LOG takes precedence
        if let Ok(rust_log) = std::env::var("RUST_LOG") {
            self.level = rust_log;
        }
        
        // SPECADO_LOG_FORMAT
        if let Ok(format) = std::env::var("SPECADO_LOG_FORMAT") {
            match format.to_lowercase().as_str() {
                "compact" => self.format = LogFormat::Compact,
                "full" => self.format = LogFormat::Full,
                "json" => self.format = LogFormat::Json,
                _ => tracing::warn!("Invalid log format: {}, using default", format),
            }
        }
        
        // SPECADO_LOG_FILE
        if let Ok(file) = std::env::var("SPECADO_LOG_FILE") {
            self.file = Some(PathBuf::from(file));
        }
        
        // SPECADO_LOG_CONSOLE
        if let Ok(console) = std::env::var("SPECADO_LOG_CONSOLE") {
            self.console = console.to_lowercase() == "true" || console == "1";
        }
    }
}

/// Initialize the global logging system
pub fn init_logging(config: LoggingConfig) -> Result<()> {
    // Create environment filter
    let env_filter = create_env_filter(&config)?;
    
    // Use different subscriber based on format to avoid type conflicts
    match config.format {
        LogFormat::Compact => {
            let subscriber = tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .with_target(true)
                .with_ansi(config.console && atty::is(atty::Stream::Stderr))
                .with_thread_ids(config.thread_ids)
                .with_file(config.source_location)
                .with_line_number(config.source_location)
                .compact()
                .finish();
            
            tracing::subscriber::set_global_default(subscriber)
                .map_err(|e| Error::other(format!("Failed to initialize logging: {}", e)))?;
        }
        LogFormat::Json => {
            let subscriber = tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .with_target(true)
                .with_ansi(false) // JSON should not have ANSI codes
                .with_thread_ids(config.thread_ids)
                .with_file(config.source_location)
                .with_line_number(config.source_location)
                .json()
                .finish();
            
            tracing::subscriber::set_global_default(subscriber)
                .map_err(|e| Error::other(format!("Failed to initialize logging: {}", e)))?;
        }
        LogFormat::Full => {
            let subscriber = tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .with_target(true)
                .with_ansi(config.console && atty::is(atty::Stream::Stderr))
                .with_thread_ids(config.thread_ids)
                .with_file(config.source_location)
                .with_line_number(config.source_location)
                .finish();
            
            tracing::subscriber::set_global_default(subscriber)
                .map_err(|e| Error::other(format!("Failed to initialize logging: {}", e)))?;
        }
    }
    
    // Generate and store request ID
    let request_id = generate_request_id();
    REQUEST_ID.set(request_id.clone()).map_err(|_| {
        Error::other("Failed to set request ID - request tracking may not work correctly")
    })?;
    
    tracing::info!(
        request_id = %request_id,
        config = ?config,
        "Logging system initialized"
    );
    
    Ok(())
}

/// Create environment filter based on configuration
fn create_env_filter(config: &LoggingConfig) -> Result<EnvFilter> {
    let mut filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));
    
    // Apply module-specific filters
    if let Some(module_filters) = &config.module_filter {
        for (module, level) in module_filters {
            filter = filter.add_directive(
                format!("{}={}", module, level)
                    .parse()
                    .map_err(|e| Error::other(format!("Invalid filter directive: {}", e)))?,
            );
        }
    }
    
    Ok(filter)
}

/// Generate a unique request ID for this session
pub fn generate_request_id() -> String {
    format!("req_{}", Uuid::new_v4().simple())
}

/// Get the current request ID
pub fn current_request_id() -> Option<&'static str> {
    REQUEST_ID.get().map(|s| s.as_str())
}

/// Create a span with request ID and timing
pub fn create_operation_span(operation: &str, details: Option<&str>) -> Span {
    let span = tracing::info_span!(
        "operation",
        operation = operation,
        request_id = current_request_id().unwrap_or("unknown"),
        details = details.unwrap_or(""),
        duration_ms = field::Empty,
    );
    
    span
}

/// Record operation duration in the current span
pub fn record_duration(span: &Span, start_time: std::time::Instant) {
    let duration = start_time.elapsed();
    span.record("duration_ms", duration.as_millis() as u64);
}

/// Sensitive data redaction utilities
pub mod redaction {
    use regex::Regex;
    use std::sync::OnceLock;
    
    static API_KEY_REGEX: OnceLock<Regex> = OnceLock::new();
    static TOKEN_REGEX: OnceLock<Regex> = OnceLock::new();
    static PASSWORD_REGEX: OnceLock<Regex> = OnceLock::new();
    
    /// Initialize redaction patterns
    fn init_patterns() {
        API_KEY_REGEX.get_or_init(|| {
            Regex::new(r#"(?i)(api[_-]?key|apikey)[=:\s]+['"]?([a-zA-Z0-9_-]{10,})['"]?"#)
                .unwrap()
        });
        
        TOKEN_REGEX.get_or_init(|| {
            Regex::new(r#"(?i)(token|bearer)[=:\s]+['"]?([a-zA-Z0-9_.-]{10,})['"]?"#)
                .unwrap()
        });
        
        PASSWORD_REGEX.get_or_init(|| {
            Regex::new(r#"(?i)(password|passwd|pwd)[=:\s]+['"]?([^\s'\"]{3,})['"]?"#)
                .unwrap()
        });
    }
    
    /// Redact sensitive information from a string
    pub fn redact_sensitive(input: &str) -> String {
        init_patterns();
        
        let mut result = input.to_string();
        
        // Redact API keys
        if let Some(regex) = API_KEY_REGEX.get() {
            result = regex.replace_all(&result, "$1=***").to_string();
        }
        
        // Redact tokens
        if let Some(regex) = TOKEN_REGEX.get() {
            result = regex.replace_all(&result, "$1=***").to_string();
        }
        
        // Redact passwords
        if let Some(regex) = PASSWORD_REGEX.get() {
            result = regex.replace_all(&result, "$1=***").to_string();
        }
        
        result
    }
    
    /// Redact sensitive information from JSON values
    pub fn redact_json_value(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, val) in map.iter_mut() {
                    if is_sensitive_key(key) {
                        *val = serde_json::Value::String("***".to_string());
                    } else {
                        redact_json_value(val);
                    }
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr.iter_mut() {
                    redact_json_value(item);
                }
            }
            serde_json::Value::String(s) => {
                *s = redact_sensitive(s);
            }
            _ => {}
        }
    }
    
    /// Check if a JSON key contains sensitive information
    fn is_sensitive_key(key: &str) -> bool {
        let key_lower = key.to_lowercase();
        key_lower.contains("key")
            || key_lower.contains("token")
            || key_lower.contains("password")
            || key_lower.contains("passwd")
            || key_lower.contains("secret")
            || key_lower.contains("credential")
            || key_lower.contains("auth")
    }
}

/// Performance timing utilities
pub mod timing {
    use std::time::Instant;
    use tracing::Span;
    
    /// A timer that automatically logs duration when dropped
    pub struct Timer {
        start: Instant,
        span: Span,
        operation: String,
    }
    
    impl Timer {
        pub fn new(operation: &str) -> Self {
            let span = super::create_operation_span(operation, None);
            
            Self {
                start: Instant::now(),
                span,
                operation: operation.to_string(),
            }
        }
        
        pub fn with_details(operation: &str, details: &str) -> Self {
            let span = super::create_operation_span(operation, Some(details));
            
            Self {
                start: Instant::now(),
                span,
                operation: operation.to_string(),
            }
        }
        
        /// Get elapsed time without finishing the timer
        pub fn elapsed(&self) -> std::time::Duration {
            self.start.elapsed()
        }
        
        /// Finish the timer and log the duration
        pub fn finish(self) {
            let duration = self.start.elapsed();
            self.span.record("duration_ms", duration.as_millis() as u64);
            
            tracing::info!(
                operation = %self.operation,
                duration_ms = duration.as_millis(),
                "Operation completed"
            );
        }
    }
    
    impl Drop for Timer {
        fn drop(&mut self) {
            let duration = self.start.elapsed();
            self.span.record("duration_ms", duration.as_millis() as u64);
            
            tracing::debug!(
                operation = %self.operation,
                duration_ms = duration.as_millis(),
                "Operation completed (auto-timed)"
            );
        }
    }
}

/// Macro for creating instrumented functions with automatic redaction
#[macro_export]
macro_rules! instrument_with_redaction {
    ($func:ident, $($skip:ident),*) => {
        #[tracing::instrument(skip($($skip),*))]
        $func
    };
}

/// Helper macro for logging with request ID
#[macro_export]
macro_rules! log_with_request_id {
    ($level:ident, $($arg:tt)*) => {
        if let Some(request_id) = $crate::logging::current_request_id() {
            tracing::$level!(request_id = request_id, $($arg)*);
        } else {
            tracing::$level!($($arg)*);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_redaction() {
        let input = "api_key=sk-1234567890abcdef token=bearer_xyz password=secret123";
        let redacted = redaction::redact_sensitive(input);
        assert!(redacted.contains("api_key=***"));
        assert!(redacted.contains("token=***"));
        assert!(redacted.contains("password=***"));
        assert!(!redacted.contains("sk-1234567890abcdef"));
        assert!(!redacted.contains("bearer_xyz"));
        assert!(!redacted.contains("secret123"));
    }
    
    #[test]
    fn test_json_redaction() {
        let mut value = serde_json::json!({
            "api_key": "sk-1234567890abcdef",
            "model": "gpt-4",
            "headers": {
                "authorization": "Bearer token123"
            }
        });
        
        redaction::redact_json_value(&mut value);
        
        assert_eq!(value["api_key"], "***");
        assert_eq!(value["model"], "gpt-4");
        assert_eq!(value["headers"]["authorization"], "***");
    }
    
    #[test]
    fn test_logging_config_from_verbosity() {
        let config = LoggingConfig::from_verbosity(0);
        assert_eq!(config.level, "warn");
        assert!(!config.source_location);
        
        let config = LoggingConfig::from_verbosity(2);
        assert_eq!(config.level, "debug");
        assert!(config.source_location);
        
        let config = LoggingConfig::from_verbosity(3);
        assert_eq!(config.level, "trace");
        assert!(config.thread_ids);
        assert!(config.span_events);
    }
}