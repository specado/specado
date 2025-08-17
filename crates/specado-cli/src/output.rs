//! Output formatting and writing utilities
//!
//! This module provides utilities for formatting and writing output
//! in various formats (JSON, YAML, human-readable).

use crate::cli::OutputFormat;
use crate::error::Result;
use colored::Colorize;
use indicatif::ProgressStyle;
use serde::Serialize;
use std::io::{self, Write};

/// Trait for formatting output
pub trait OutputFormatter {
    /// Format a serializable value
    fn format<T: Serialize>(&self, value: &T) -> Result<String>;
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
}

/// Output writer that handles different output formats and colors
pub struct OutputWriter {
    format: OutputFormat,
    use_color: bool,
    show_progress: bool,
    quiet: bool,
    writer: Box<dyn Write>,
}

impl OutputWriter {
    /// Create a new output writer
    pub fn new(format: OutputFormat, use_color: bool, quiet: bool) -> Self {
        Self {
            format,
            use_color,
            show_progress: !quiet && atty::is(atty::Stream::Stdout),
            quiet,
            writer: Box::new(io::stdout()),
        }
    }
    
    /// Create an output writer with a custom writer
    pub fn with_writer(
        format: OutputFormat,
        use_color: bool,
        quiet: bool,
        writer: Box<dyn Write>,
    ) -> Self {
        Self {
            format,
            use_color,
            show_progress: false, // No progress bars with custom writers
            quiet,
            writer,
        }
    }
    
    /// Get the output format
    pub fn format(&self) -> OutputFormat {
        self.format
    }
    
    /// Check if progress indicators should be shown
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
        let formatted = self.format.format(value)?;
        
        if self.format == OutputFormat::Human {
            // For human format, we might want to do additional formatting
            self.writeln(&formatted)
        } else {
            // For machine formats, write as-is
            self.write(&formatted)
        }
    }
    
    /// Write a table (for human format)
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