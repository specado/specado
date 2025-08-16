//! Schema documentation generation
//!
//! This module provides automatic documentation generation from JSON Schema definitions,
//! creating comprehensive markdown documentation with examples and constraint descriptions.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

pub mod generator;
pub mod parser;
pub mod templates;

pub use generator::{DocGenerator, GeneratorConfig, GeneratorResult};
pub use parser::{SchemaParser, PropertyInfo, TypeInfo};
pub use templates::{Template, TemplateType};

/// Create a documentation generator with default configuration
pub fn create_doc_generator() -> DocGenerator {
    DocGenerator::new()
}

/// Generate documentation from a JSON Schema
pub fn generate_docs(schema: &serde_json::Value) -> GeneratorResult<String> {
    let generator = create_doc_generator();
    generator.generate(schema)
}