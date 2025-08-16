//! Schema loading and parsing functionality
//!
//! This module provides comprehensive schema loading capabilities including:
//! - YAML and JSON parsing support
//! - Reference resolution ($ref support)
//! - Environment variable expansion
//! - In-memory caching for performance
//! - Circular reference detection
//!
//! # Example Usage
//!
//! ```rust
//! use specado_schemas::loader::SchemaLoader;
//! use std::path::Path;
//!
//! let mut loader = SchemaLoader::new();
//! let spec = loader.load_prompt_spec(Path::new("prompt.yaml"))?;
//! println!("Loaded spec: {}", serde_json::to_string_pretty(&spec)?);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

pub mod cache;
pub mod error;
pub mod parser;
pub mod resolver;
pub mod schema_loader;

pub use cache::{CacheEntry, SchemaCache};
pub use error::{LoaderError, LoaderResult};
pub use parser::{Format, SchemaParser};
pub use resolver::{ReferenceResolver, ResolverContext};
pub use schema_loader::SchemaLoader;