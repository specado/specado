//! Output formatting and writing utilities
//!
//! This module provides utilities for formatting and writing output
//! in various formats (JSON, YAML, human-readable) with comprehensive
//! support for TranslationResult, ValidationErrors, and progress indicators.

use crate::cli::OutputFormat;
use crate::error::Result;
use crate::logging::redaction;
use colored::Colorize;
use tracing::{debug, trace};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;
use serde_json::Value;
use specado_core::types::{TranslationResult, LossinessReport, LossinessItem, TranslationMetadata};
use specado_schemas::validation::{ValidationError, ValidationErrors};
use std::collections::HashMap;
use std::io::{self, Write, IsTerminal};
use std::time::Duration;

/// Trait for formatting output with specialized support for common types
pub trait OutputFormatter {
    /// Format a serializable value
    fn format<T: Serialize>(&self, value: &T) -> Result<String>;
    
    /// Format a TranslationResult with lossiness reporting
    fn format_translation_result(&self, result: &TranslationResult) -> Result<String>;
    
    /// Format validation errors with detailed violation reporting
    #[allow(dead_code)]
    fn format_validation_errors(&self, errors: &ValidationErrors) -> Result<String>;
    
    /// Format a single validation error
    fn format_validation_error(&self, error: &ValidationError) -> Result<String>;
    
    /// Format a lossiness report with categorization
    fn format_lossiness_report(&self, report: &LossinessReport) -> Result<String>;
}

impl OutputFormatter for OutputFormat {
    fn format<T: Serialize>(&self, value: &T) -> Result<String> {
        match self {
            OutputFormat::Json => Ok(serde_json::to_string(value)?),
            OutputFormat::JsonPretty => Ok(serde_json::to_string_pretty(value)?),
            OutputFormat::Yaml => Ok(serde_yaml::to_string(value)?),
            OutputFormat::Human => {
                // For human format, use pretty JSON as fallback
                Ok(serde_json::to_string_pretty(value)?)
            }
        }
    }
    
    fn format_translation_result(&self, result: &TranslationResult) -> Result<String> {
        match self {
            OutputFormat::Json => Ok(serde_json::to_string(result)?),
            OutputFormat::JsonPretty => Ok(serde_json::to_string_pretty(result)?),
            OutputFormat::Yaml => Ok(serde_yaml::to_string(result)?),
            OutputFormat::Human => format_translation_result_human(result),
        }
    }
    
    fn format_validation_errors(&self, errors: &ValidationErrors) -> Result<String> {
        match self {
            OutputFormat::Json => Ok(serde_json::to_string(errors)?),
            OutputFormat::JsonPretty => Ok(serde_json::to_string_pretty(errors)?),
            OutputFormat::Yaml => Ok(serde_yaml::to_string(errors)?),
            OutputFormat::Human => format_validation_errors_human(errors),
        }
    }
    
    fn format_validation_error(&self, error: &ValidationError) -> Result<String> {
        match self {
            OutputFormat::Json => Ok(serde_json::to_string(error)?),
            OutputFormat::JsonPretty => Ok(serde_json::to_string_pretty(error)?),
            OutputFormat::Yaml => Ok(serde_yaml::to_string(error)?),
            OutputFormat::Human => format_validation_error_human(error),
        }
    }
    
    fn format_lossiness_report(&self, report: &LossinessReport) -> Result<String> {
        match self {
            OutputFormat::Json => Ok(serde_json::to_string(report)?),
            OutputFormat::JsonPretty => Ok(serde_json::to_string_pretty(report)?),
            OutputFormat::Yaml => Ok(serde_yaml::to_string(report)?),
            OutputFormat::Human => format_lossiness_report_human(report),
        }
    }
}

/// Output writer that handles different output formats and colors
pub struct OutputWriter {
    format: OutputFormat,
    use_color: bool,
    show_progress: bool,
    quiet: bool,
    #[allow(dead_code)]
    verbose: u8,
    writer: Box<dyn Write>,
}

impl OutputWriter {
    /// Create a new output writer
    pub fn new(format: OutputFormat, use_color: bool, quiet: bool, verbose: u8) -> Self {
        Self {
            format,
            use_color,
            show_progress: !quiet && std::io::stdout().is_terminal(),
            quiet,
            verbose,
            writer: Box::new(io::stdout()),
        }
    }
    
    /// Create an output writer with a custom writer
    #[allow(dead_code)]
    pub fn with_writer(
        format: OutputFormat,
        use_color: bool,
        quiet: bool,
        verbose: u8,
        writer: Box<dyn Write>,
    ) -> Self {
        Self {
            format,
            use_color,
            show_progress: false, // No progress bars with custom writers
            quiet,
            verbose,
            writer,
        }
    }
    
    /// Get the output format
    pub fn format(&self) -> OutputFormat {
        self.format
    }
    
    /// Check if progress indicators should be shown
    #[allow(dead_code)]
    pub fn show_progress(&self) -> bool {
        self.show_progress
    }
    
    /// Write raw output
    pub fn write(&mut self, content: &str) -> Result<()> {
        write!(self.writer, "{}", content)?;
        self.writer.flush()?;
        Ok(())
    }
    
    /// Write a line of output
    pub fn writeln(&mut self, content: &str) -> Result<()> {
        writeln!(self.writer, "{}", content)?;
        self.writer.flush()?;
        Ok(())
    }
    
    /// Write an info message
    pub fn info(&mut self, message: &str) -> Result<()> {
        debug!("Output info: {}", message);
        
        if self.quiet {
            return Ok(());
        }
        
        if self.format == OutputFormat::Human {
            if self.use_color {
                self.writeln(&format!("{} {}", "ℹ".blue(), message))
            } else {
                self.writeln(&format!("INFO: {}", message))
            }
        } else {
            Ok(())
        }
    }
    
    /// Write a success message
    pub fn success(&mut self, message: &str) -> Result<()> {
        if self.quiet {
            return Ok(());
        }
        
        if self.format == OutputFormat::Human {
            if self.use_color {
                self.writeln(&message.green().to_string())
            } else {
                self.writeln(message)
            }
        } else {
            Ok(())
        }
    }
    
    /// Write a warning message
    pub fn warning(&mut self, message: &str) -> Result<()> {
        if self.format == OutputFormat::Human {
            if self.use_color {
                self.writeln(&message.yellow().to_string())
            } else {
                self.writeln(&format!("WARNING: {}", message))
            }
        } else {
            Ok(())
        }
    }
    
    /// Write an error message
    pub fn error(&mut self, message: &str) -> Result<()> {
        if self.format == OutputFormat::Human {
            if self.use_color {
                self.writeln(&message.red().to_string())
            } else {
                self.writeln(&format!("ERROR: {}", message))
            }
        } else {
            Ok(())
        }
    }
    
    /// Write a section header
    pub fn section(&mut self, title: &str) -> Result<()> {
        if self.quiet {
            return Ok(());
        }
        
        if self.format == OutputFormat::Human {
            self.writeln("")?;
            if self.use_color {
                self.writeln(&format!("═══ {} ═══", title).bright_blue().to_string())
            } else {
                self.writeln(&format!("=== {} ===", title))
            }
        } else {
            Ok(())
        }
    }
    
    /// Write data in the configured format
    pub fn data<T: Serialize>(&mut self, value: &T) -> Result<()> {
        // Create a redacted copy of the value for logging
        let mut value_json = serde_json::to_value(value)?;
        redaction::redact_json_value(&mut value_json);
        
        trace!("Outputting data: {}", 
            serde_json::to_string(&value_json).unwrap_or_else(|_| "[failed to serialize]".to_string())
        );
        
        let formatted = self.format.format(value)?;
        
        if self.format == OutputFormat::Human {
            // For human format, we might want to do additional formatting
            self.writeln(&formatted)
        } else {
            // For machine formats, write as-is
            self.write(&formatted)
        }
    }
    
    /// Write a translation result with specialized formatting
    pub fn translation_result(&mut self, result: &TranslationResult) -> Result<()> {
        let formatted = self.format.format_translation_result(result)?;
        self.writeln(&formatted)
    }
    
    /// Write validation errors with specialized formatting
    #[allow(dead_code)]
    pub fn validation_errors(&mut self, errors: &ValidationErrors) -> Result<()> {
        let formatted = self.format.format_validation_errors(errors)?;
        self.writeln(&formatted)
    }
    
    /// Write a single validation error
    pub fn validation_error(&mut self, error: &ValidationError) -> Result<()> {
        let formatted = self.format.format_validation_error(error)?;
        self.writeln(&formatted)
    }
    
    /// Write a lossiness report with specialized formatting
    pub fn lossiness_report(&mut self, report: &LossinessReport) -> Result<()> {
        let formatted = self.format.format_lossiness_report(report)?;
        self.writeln(&formatted)
    }
    
    /// Create a progress bar for long operations
    #[allow(dead_code)]
    pub fn progress_bar(&self, length: u64, message: &str) -> Option<ProgressBar> {
        if !self.show_progress {
            return None;
        }
        
        let pb = ProgressBar::new(length);
        pb.set_style(default_progress_style());
        pb.set_message(message.to_string());
        Some(pb)
    }
    
    /// Create a spinner for indeterminate progress
    pub fn spinner(&self, message: &str) -> Option<ProgressBar> {
        if !self.show_progress {
            return None;
        }
        
        let pb = ProgressBar::new_spinner();
        pb.set_style(default_spinner_style());
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        Some(pb)
    }
    
    /// Get verbosity level
    #[allow(dead_code)]
    pub fn verbosity(&self) -> u8 {
        self.verbose
    }
    
    /// Check if verbose output should be shown
    #[allow(dead_code)]
    pub fn is_verbose(&self) -> bool {
        self.verbose > 0
    }
    
    /// Write debug information if verbose mode is enabled
    #[allow(dead_code)]
    pub fn debug(&mut self, message: &str) -> Result<()> {
        if self.verbose > 0 && self.format == OutputFormat::Human {
            if self.use_color {
                self.writeln(&format!("{} {}", "DEBUG:".dimmed(), message.dimmed()))
            } else {
                self.writeln(&format!("DEBUG: {}", message))
            }
        } else {
            Ok(())
        }
    }
    
    /// Write trace information if high verbosity is enabled
    #[allow(dead_code)]
    pub fn trace(&mut self, message: &str) -> Result<()> {
        if self.verbose > 1 && self.format == OutputFormat::Human {
            if self.use_color {
                self.writeln(&format!("{} {}", "TRACE:".dimmed(), message.dimmed()))
            } else {
                self.writeln(&format!("TRACE: {}", message))
            }
        } else {
            Ok(())
        }
    }
    
    /// Write a table (for human format)
    #[allow(dead_code)]
    pub fn table(&mut self, headers: &[&str], rows: Vec<Vec<String>>) -> Result<()> {
        if self.quiet || self.format != OutputFormat::Human {
            return Ok(());
        }
        
        // Calculate column widths
        let mut widths = headers.iter().map(|h| h.len()).collect::<Vec<_>>();
        for row in &rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }
        
        // Print header
        let header_row = headers
            .iter()
            .enumerate()
            .map(|(i, h)| format!("{:width$}", h, width = widths[i]))
            .collect::<Vec<_>>()
            .join(" │ ");
        
        if self.use_color {
            self.writeln(&header_row.bold().to_string())?;
        } else {
            self.writeln(&header_row)?;
        }
        
        // Print separator
        let separator = widths
            .iter()
            .map(|w| "─".repeat(*w))
            .collect::<Vec<_>>()
            .join("─┼─");
        self.writeln(&separator)?;
        
        // Print rows
        for row in rows {
            let row_str = row
                .iter()
                .enumerate()
                .map(|(i, cell)| {
                    if i < widths.len() {
                        format!("{:width$}", cell, width = widths[i])
                    } else {
                        cell.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(" │ ");
            self.writeln(&row_str)?;
        }
        
        Ok(())
    }
}

/// Helper function to create a progress bar style
#[allow(dead_code)]
pub fn default_progress_style() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("#>-")
}

/// Helper function to create a spinner style
pub fn default_spinner_style() -> ProgressStyle {
    ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap()
}

/// Format a TranslationResult for human reading
fn format_translation_result_human(result: &TranslationResult) -> Result<String> {
    let mut output = String::new();
    
    // Translation summary
    output.push_str("═══ Translation Result ═══\n\n");
    
    // Metadata section
    if let Some(metadata) = &result.metadata {
        output.push_str(&format_translation_metadata_human(metadata)?);
        output.push('\n');
    }
    
    // Lossiness summary
    if !result.lossiness.items.is_empty() {
        output.push_str("🔍 Lossiness Summary:\n");
        output.push_str(&format!("  Total Issues: {}\n", result.lossiness.summary.total_items));
        output.push_str(&format!("  Max Severity: {:?}\n", result.lossiness.max_severity));
        
        // Breakdown by severity
        for (severity, count) in &result.lossiness.summary.by_severity {
            let icon = match severity.as_str() {
                "Error" => "❌",
                "Warning" => "⚠️",
                "Info" => "ℹ️",
                _ => "•",
            };
            output.push_str(&format!("  {} {}: {}\n", icon, severity, count));
        }
        output.push('\n');
    } else {
        output.push_str("✅ No lossiness detected - perfect translation\n\n");
    }
    
    // Provider request (formatted nicely)
    output.push_str("📝 Provider Request:\n");
    output.push_str(&serde_json::to_string_pretty(&result.provider_request_json)?);
    output.push('\n');
    
    Ok(output)
}

/// Format translation metadata for human reading
fn format_translation_metadata_human(metadata: &TranslationMetadata) -> Result<String> {
    let mut output = String::new();
    
    output.push_str("🔧 Translation Details:\n");
    output.push_str(&format!("  Provider: {}\n", metadata.provider));
    output.push_str(&format!("  Model: {}\n", metadata.model));
    output.push_str(&format!("  Strict Mode: {:?}\n", metadata.strict_mode));
    output.push_str(&format!("  Timestamp: {}\n", metadata.timestamp));
    
    if let Some(duration) = metadata.duration_ms {
        output.push_str(&format!("  Duration: {}ms\n", duration));
    }
    
    Ok(output)
}

/// Format validation errors for human reading
#[allow(dead_code)]
fn format_validation_errors_human(errors: &ValidationErrors) -> Result<String> {
    let mut output = String::new();
    
    output.push_str(&format!("❌ Validation Failed - {} Error(s)\n\n", errors.len()));
    
    for (i, error) in errors.errors.iter().enumerate() {
        output.push_str(&format!("{}. {}\n", i + 1, format_validation_error_human(error)?));
    }
    
    Ok(output)
}

/// Format a single validation error for human reading
fn format_validation_error_human(error: &ValidationError) -> Result<String> {
    let mut output = String::new();
    
    output.push_str(&format!("📍 Path: {}\n", error.path));
    output.push_str(&format!("💬 Message: {}\n", error.message));
    
    if !error.schema_violations.is_empty() {
        output.push_str("🔍 Schema Violations:\n");
        
        for violation in &error.schema_violations {
            output.push_str(&format!("  • Rule: {}\n", violation.rule));
            output.push_str(&format!("    Expected: {}\n", violation.expected));
            output.push_str(&format!("    Actual: {}\n", violation.actual));
            output.push('\n');
        }
    }
    
    Ok(output)
}

/// Format lossiness report for human reading
fn format_lossiness_report_human(report: &LossinessReport) -> Result<String> {
    let mut output = String::new();
    
    if report.items.is_empty() {
        output.push_str("✅ No lossiness detected\n");
        return Ok(output);
    }
    
    output.push_str(&format!("🔍 Lossiness Report - {} Issue(s)\n\n", report.summary.total_items));
    
    // Summary by severity
    output.push_str("📊 Summary by Severity:\n");
    for (severity, count) in &report.summary.by_severity {
        let icon = match severity.as_str() {
            "Error" => "❌",
            "Warning" => "⚠️", 
            "Info" => "ℹ️",
            _ => "•",
        };
        output.push_str(&format!("  {} {}: {}\n", icon, severity, count));
    }
    output.push('\n');
    
    // Summary by code
    output.push_str("📋 Summary by Type:\n");
    for (code, count) in &report.summary.by_code {
        output.push_str(&format!("  • {}: {}\n", code, count));
    }
    output.push('\n');
    
    // Group issues by severity for better readability
    let mut by_severity: HashMap<String, Vec<&LossinessItem>> = HashMap::new();
    for item in &report.items {
        by_severity
            .entry(format!("{:?}", item.severity))
            .or_default()
            .push(item);
    }
    
    // Display issues grouped by severity
    for severity in ["Error", "Warning", "Info"] {
        if let Some(items) = by_severity.get(severity) {
            let icon = match severity {
                "Error" => "❌",
                "Warning" => "⚠️",
                "Info" => "ℹ️",
                _ => "•",
            };
            
            output.push_str(&format!("{} {} Issues:\n", icon, severity));
            
            for item in items {
                output.push_str(&format!("  📍 Path: {}\n", item.path));
                output.push_str(&format!("  🏷️  Code: {:?}\n", item.code));
                output.push_str(&format!("  💬 Message: {}\n", item.message));
                
                if let Some(before) = &item.before {
                    output.push_str("  📥 Before:\n");
                    output.push_str(&format!("    {}\n", format_value_compact(before)));
                }
                
                if let Some(after) = &item.after {
                    output.push_str("  📤 After:\n");
                    output.push_str(&format!("    {}\n", format_value_compact(after)));
                }
                
                output.push('\n');
            }
        }
    }
    
    Ok(output)
}

/// Format a JSON value in a compact, human-readable way
fn format_value_compact(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{}\"", s),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(arr) => {
            if arr.len() <= 3 {
                format!("[{}]", arr.iter()
                    .map(format_value_compact)
                    .collect::<Vec<_>>()
                    .join(", "))
            } else {
                format!("[{} items]", arr.len())
            }
        }
        Value::Object(obj) => {
            if obj.len() <= 2 {
                let items: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("{}: {}", k, format_value_compact(v)))
                    .collect();
                format!("{{{}}}", items.join(", "))
            } else {
                format!("{{{} fields}}", obj.len())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    include!("output/tests.rs");
}