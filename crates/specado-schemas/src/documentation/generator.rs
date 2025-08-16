//! Main documentation generator
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::documentation::parser::{SchemaParser, PropertyInfo};
use crate::documentation::templates::Template;
use serde_json::Value;
use std::collections::HashMap;

/// Documentation generator configuration
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Include table of contents
    pub include_toc: bool,
    /// Include examples
    pub include_examples: bool,
    /// Include synthesized examples
    pub synthesize_examples: bool,
    /// Maximum depth for nested properties
    pub max_depth: usize,
    /// Include deprecated properties
    pub include_deprecated: bool,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            include_toc: true,
            include_examples: true,
            synthesize_examples: true,
            max_depth: 10,
            include_deprecated: true,
        }
    }
}

/// Result type for generator operations
pub type GeneratorResult<T> = Result<T, GeneratorError>;

/// Generator error types
#[derive(Debug)]
pub enum GeneratorError {
    InvalidSchema(String),
    ProcessingError(String),
}

impl std::fmt::Display for GeneratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeneratorError::InvalidSchema(msg) => write!(f, "Invalid schema: {}", msg),
            GeneratorError::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
        }
    }
}

impl std::error::Error for GeneratorError {}

/// Main documentation generator
pub struct DocGenerator {
    config: GeneratorConfig,
    templates: HashMap<String, String>,
}

impl DocGenerator {
    /// Create a new documentation generator
    pub fn new() -> Self {
        Self::with_config(GeneratorConfig::default())
    }

    /// Create a generator with custom configuration
    pub fn with_config(config: GeneratorConfig) -> Self {
        Self {
            config,
            templates: HashMap::new(),
        }
    }

    /// Generate documentation from a JSON Schema
    pub fn generate(&self, schema: &Value) -> GeneratorResult<String> {
        // Parse the schema
        let root_info = SchemaParser::parse(schema);
        
        let mut doc = String::new();
        
        // Generate header
        let title = schema.get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Schema Documentation");
        let description = schema.get("description")
            .and_then(|d| d.as_str());
        
        doc.push_str(&Template::header(title, description));
        
        // Generate table of contents if enabled
        if self.config.include_toc {
            let sections = self.collect_sections(&root_info);
            if !sections.is_empty() {
                doc.push_str(&Template::table_of_contents(&sections));
            }
        }
        
        // Generate overview section
        doc.push_str("## Overview\n\n");
        doc.push_str(&self.generate_overview(schema));
        
        // Generate properties documentation
        if root_info.type_info.base_type == "object" {
            // Check if there are any properties to document after filtering
            let has_props = if self.config.include_deprecated {
                !root_info.properties.is_empty()
            } else {
                root_info.properties.values().any(|p| !p.deprecated)
            };
            
            if has_props {
                doc.push_str("## Properties\n\n");
                doc.push_str(&self.generate_properties_doc(&root_info));
            }
        }
        
        // Generate complete example if enabled
        if self.config.include_examples || self.config.synthesize_examples {
            doc.push_str("## Complete Example\n\n");
            doc.push_str(&self.generate_complete_example(&root_info));
        }
        
        // Generate validation rules section
        doc.push_str("## Validation Rules\n\n");
        doc.push_str(&self.generate_validation_rules(&root_info));
        
        // Add footer
        doc.push_str(&Template::footer());
        
        Ok(doc)
    }

    /// Generate overview section
    fn generate_overview(&self, schema: &Value) -> String {
        let mut overview = String::new();
        
        // Schema version
        if let Some(schema_uri) = schema.get("$schema").and_then(|s| s.as_str()) {
            overview.push_str(&format!("**Schema Version:** `{}`\n\n", schema_uri));
        }
        
        // ID
        if let Some(id) = schema.get("$id").and_then(|i| i.as_str()) {
            overview.push_str(&format!("**ID:** `{}`\n\n", id));
        }
        
        // Type
        if let Some(type_val) = schema.get("type") {
            if let Some(type_str) = type_val.as_str() {
                overview.push_str(&format!("**Root Type:** `{}`\n\n", type_str));
            }
        }
        
        // Additional metadata
        if let Some(version) = schema.get("version").and_then(|v| v.as_str()) {
            overview.push_str(&format!("**Version:** `{}`\n\n", version));
        }
        
        overview
    }

    /// Generate properties documentation
    fn generate_properties_doc(&self, root_info: &PropertyInfo) -> String {
        let mut doc = String::new();
        
        // Filter and sort properties
        let mut props: Vec<_> = root_info.properties.values()
            .filter(|prop| self.config.include_deprecated || !prop.deprecated)
            .collect();
        
        // Sort: required first, then alphabetically
        props.sort_by(|a, b| {
            match (a.required, b.required) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
        
        for prop in props {
            doc.push_str(&Template::property(prop, 3));
        }
        
        // Document additional properties if present
        if let Some(ref additional) = root_info.additional_properties {
            doc.push_str("### Additional Properties\n\n");
            doc.push_str("Any additional properties must conform to:\n\n");
            doc.push_str(&Template::property(additional, 4));
        }
        
        doc
    }

    /// Generate complete example
    fn generate_complete_example(&self, root_info: &PropertyInfo) -> String {
        let mut doc = String::new();
        
        // Use provided examples if available
        if !root_info.examples.is_empty() {
            for (i, example) in root_info.examples.iter().enumerate() {
                if root_info.examples.len() > 1 {
                    doc.push_str(&format!("### Example {}\n\n", i + 1));
                }
                doc.push_str("```json\n");
                doc.push_str(&serde_json::to_string_pretty(example).unwrap_or_else(|_| "null".to_string()));
                doc.push_str("\n```\n\n");
            }
        } else if self.config.synthesize_examples {
            // Synthesize an example
            let example = Template::synthesize_example_with_config(root_info, self.config.include_deprecated);
            doc.push_str("```json\n");
            doc.push_str(&serde_json::to_string_pretty(&example).unwrap_or_else(|_| "null".to_string()));
            doc.push_str("\n```\n\n");
        }
        
        doc
    }

    /// Generate validation rules section
    fn generate_validation_rules(&self, root_info: &PropertyInfo) -> String {
        let mut doc = String::new();
        let mut rules = Vec::new();
        
        // Collect all validation rules
        self.collect_validation_rules(root_info, "", &mut rules);
        
        if rules.is_empty() {
            doc.push_str("No additional validation rules defined.\n\n");
        } else {
            for (path, rule) in rules {
                doc.push_str(&format!("- **{}**: {}\n", path, rule));
            }
            doc.push_str("\n");
        }
        
        doc
    }

    /// Recursively collect validation rules
    fn collect_validation_rules(&self, prop: &PropertyInfo, path: &str, rules: &mut Vec<(String, String)>) {
        let current_path = if path.is_empty() {
            prop.name.clone()
        } else {
            format!("{}.{}", path, prop.name)
        };
        
        // Pattern validation
        if let Some(ref pattern) = prop.pattern {
            rules.push((current_path.clone(), format!("Must match pattern `{}`", pattern)));
        }
        
        // String length validation
        if prop.min_length.is_some() || prop.max_length.is_some() {
            let rule = match (prop.min_length, prop.max_length) {
                (Some(min), Some(max)) => format!("Length must be between {} and {} characters", min, max),
                (Some(min), None) => format!("Minimum length: {} characters", min),
                (None, Some(max)) => format!("Maximum length: {} characters", max),
                _ => String::new(),
            };
            if !rule.is_empty() {
                rules.push((current_path.clone(), rule));
            }
        }
        
        // Numeric range validation
        if prop.minimum.is_some() || prop.maximum.is_some() {
            let rule = match (prop.minimum, prop.maximum) {
                (Some(min), Some(max)) => format!("Value must be between {} and {}", min, max),
                (Some(min), None) => format!("Minimum value: {}", min),
                (None, Some(max)) => format!("Maximum value: {}", max),
                _ => String::new(),
            };
            if !rule.is_empty() {
                rules.push((current_path.clone(), rule));
            }
        }
        
        // Array validation
        if prop.min_items.is_some() || prop.max_items.is_some() {
            let rule = match (prop.min_items, prop.max_items) {
                (Some(min), Some(max)) => format!("Array must contain between {} and {} items", min, max),
                (Some(min), None) => format!("Minimum {} items required", min),
                (None, Some(max)) => format!("Maximum {} items allowed", max),
                _ => String::new(),
            };
            if !rule.is_empty() {
                rules.push((current_path.clone(), rule));
            }
        }
        
        if prop.unique_items {
            rules.push((current_path.clone(), "Array items must be unique".to_string()));
        }
        
        // Enum validation
        if prop.enum_values.is_some() {
            rules.push((current_path.clone(), "Value must be one of the allowed values".to_string()));
        }
        
        // Const validation
        if prop.const_value.is_some() {
            rules.push((current_path.clone(), "Value must match the constant value".to_string()));
        }
        
        // Recurse into nested properties
        for nested_prop in prop.properties.values() {
            self.collect_validation_rules(nested_prop, &current_path, rules);
        }
    }

    /// Collect sections for table of contents
    fn collect_sections(&self, root_info: &PropertyInfo) -> Vec<(&'static str, usize)> {
        let mut sections = vec![
            ("Overview", 1),
        ];
        
        if root_info.type_info.base_type == "object" {
            // Check if there are any properties to document after filtering
            let has_props = if self.config.include_deprecated {
                !root_info.properties.is_empty()
            } else {
                root_info.properties.values().any(|p| !p.deprecated)
            };
            
            if has_props {
                sections.push(("Properties", 1));
            }
        }
        
        if self.config.include_examples || self.config.synthesize_examples {
            sections.push(("Complete Example", 1));
        }
        
        sections.push(("Validation Rules", 1));
        
        sections
    }

    /// Add a custom template
    pub fn add_template(&mut self, name: String, template: String) {
        self.templates.insert(name, template);
    }

    /// Get the current configuration
    pub fn config(&self) -> &GeneratorConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: GeneratorConfig) {
        self.config = config;
    }
}

impl Default for DocGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_documentation_generation() {
        let schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "Test Schema",
            "description": "A test schema for documentation",
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "The name field"
                },
                "age": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 150
                }
            },
            "required": ["name"]
        });
        
        let generator = DocGenerator::new();
        let doc = generator.generate(&schema).unwrap();
        
        assert!(doc.contains("# Test Schema"));
        assert!(doc.contains("A test schema for documentation"));
        assert!(doc.contains("## Properties"));
        assert!(doc.contains("`name`"));
        assert!(doc.contains("**[Required]**"));
        assert!(doc.contains("`age`"));
    }

    #[test]
    fn test_enum_documentation() {
        let schema = json!({
            "title": "Enum Schema",
            "type": "object",
            "properties": {
                "status": {
                    "type": "string",
                    "enum": ["active", "inactive", "pending"],
                    "description": "Current status"
                }
            }
        });
        
        let generator = DocGenerator::new();
        let doc = generator.generate(&schema).unwrap();
        
        assert!(doc.contains("**Allowed Values:**"));
        assert!(doc.contains("`\"active\"`"));
        assert!(doc.contains("`\"inactive\"`"));
        assert!(doc.contains("`\"pending\"`"));
    }

    #[test]
    fn test_validation_rules_generation() {
        let schema = json!({
            "title": "Validated Schema",
            "type": "object",
            "properties": {
                "email": {
                    "type": "string",
                    "format": "email",
                    "pattern": "^[^@]+@[^@]+$"
                },
                "items": {
                    "type": "array",
                    "minItems": 1,
                    "maxItems": 10,
                    "uniqueItems": true
                }
            }
        });
        
        let generator = DocGenerator::new();
        let doc = generator.generate(&schema).unwrap();
        
        assert!(doc.contains("## Validation Rules"));
        assert!(doc.contains("Must match pattern"));
        assert!(doc.contains("Array must contain between"));
        assert!(doc.contains("Array items must be unique"));
    }

    #[test]
    fn test_example_synthesis() {
        let schema = json!({
            "title": "Example Schema",
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "format": "uuid"
                },
                "created": {
                    "type": "string",
                    "format": "date-time"
                },
                "count": {
                    "type": "integer",
                    "minimum": 0,
                    "default": 0
                }
            },
            "required": ["id", "created"]
        });
        
        let generator = DocGenerator::new();
        let doc = generator.generate(&schema).unwrap();
        
        assert!(doc.contains("## Complete Example"));
        assert!(doc.contains("```json"));
    }

    #[test]
    fn test_config_options() {
        let schema = json!({
            "title": "Config Test",
            "type": "object",
            "properties": {
                "deprecated_field": {
                    "type": "string",
                    "deprecated": true
                }
            }
        });
        
        let mut config = GeneratorConfig::default();
        config.include_deprecated = false;
        config.include_toc = false;
        
        let generator = DocGenerator::with_config(config);
        let doc = generator.generate(&schema).unwrap();
        
        // Debug print to see what's being generated
        if doc.contains("deprecated_field") {
            eprintln!("Generated doc contains deprecated_field:\n{}", doc);
        }
        
        assert!(!doc.contains("Table of Contents"));
        assert!(!doc.contains("deprecated_field"));
    }
}