//! Lossiness tracking infrastructure for translation operations
//!
//! This module provides comprehensive lossiness tracking capabilities for translation operations.
//! It tracks all transformations, field movements, coercions, and other changes that occur
//! during the translation process, maintaining a detailed audit trail.
//!
//! # Examples
//!
//! ```
//! use specado_core::translation::lossiness::{LossinessTracker, OperationType};
//! use specado_core::StrictMode;
//! use std::collections::HashMap;
//!
//! let mut tracker = LossinessTracker::new(StrictMode::Warn);
//!
//! // Track a field transformation
//! tracker.track_transformation(
//!     "$.temperature",
//!     OperationType::TypeConversion,
//!     Some(serde_json::json!("1.5")),
//!     Some(serde_json::json!(1.5)),
//!     "Converted string to number",
//!     Some("OpenAI".to_string()),
//!     HashMap::new(),
//! );
//!
//! // Track a dropped field
//! tracker.track_dropped_field(
//!     "$.custom_field",
//!     Some(serde_json::json!("value")),
//!     "Field not supported by provider",
//!     Some("OpenAI".to_string()),
//! );
//!
//! // Get summary statistics
//! let stats = tracker.get_summary_statistics();
//! assert_eq!(stats.total_transformations, 2);
//! assert_eq!(stats.dropped_fields, 1);
//!
//! // Generate audit report
//! let report = tracker.generate_audit_report();
//! println!("{}", report);
//!
//! // Query specific transformations
//! let dropped_fields = tracker.get_dropped_fields();
//! assert_eq!(dropped_fields.len(), 1);
//! ```
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

pub mod operations;
pub mod reporting;
pub mod statistics;
pub mod tracker;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export the main types and structs for backward compatibility
pub use operations::*;
pub use reporting::*;
pub use statistics::*;
pub use tracker::LossinessTracker;
pub use types::*;

