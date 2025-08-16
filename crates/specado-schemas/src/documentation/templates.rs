//! Markdown templates for documentation generation
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::documentation::parser::PropertyInfo;
use serde_json::Value;
use std::collections::HashMap;

/// Template type for different documentation sections
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateType {
    Header,
    TableOfContents,
    Overview,
    Property,
    Example,
    Constraints,
    EnumValues,
    Footer,
}

/// Template for generating markdown documentation
pub struct Template;

impl Template {
    /// Generate header section
    pub fn header(title: &str, description: Option<&str>) -> String {
        let mut result = format!("# {}\n\n", title);
        
        if let Some(desc) = description {
            result.push_str(&format!("{}\n\n", desc));
        }
        
        result.push_str("---\n\n");
        result
    }

    /// Generate table of contents
    pub fn table_of_contents(sections: &[(&str, usize)]) -> String {
        let mut result = String::from("## Table of Contents\n\n");
        
        for (title, level) in sections {
            let indent = "  ".repeat(*level - 1);
            let anchor = title.to_lowercase().replace(' ', "-").replace('.', "");
            result.push_str(&format!("{}- [{}](#{})\n", indent, title, anchor));
        }
        
        result.push_str("\n");
        result
    }

    /// Generate property documentation
    pub fn property(prop: &PropertyInfo, level: usize) -> String {
        let mut result = String::new();
        let heading = "#".repeat(level.min(6));
        
        // Property heading
        result.push_str(&format!("{} `{}`\n\n", heading, prop.name));
        
        // Type badge
        let type_display = prop.type_info.display_type();
        result.push_str(&format!("**Type:** `{}`", type_display));
        
        if prop.required {
            result.push_str(" **[Required]**");
        }
        
        if prop.deprecated {
            result.push_str(" **[Deprecated]**");
        }
        
        result.push_str("\n\n");
        
        // Description
        if let Some(ref desc) = prop.description {
            result.push_str(&format!("{}\n\n", desc));
        }
        
        // Default value
        if let Some(ref default) = prop.default {
            result.push_str(&format!("**Default:** `{}`\n\n", 
                serde_json::to_string(default).unwrap_or_else(|_| "null".to_string())));
        }
        
        // Enum values
        if let Some(ref values) = prop.enum_values {
            result.push_str(&Self::enum_values(values));
        }
        
        // Const value
        if let Some(ref const_val) = prop.const_value {
            result.push_str(&format!("**Constant Value:** `{}`\n\n", 
                serde_json::to_string(const_val).unwrap_or_else(|_| "null".to_string())));
        }
        
        // Constraints
        let constraints = prop.get_constraints();
        if !constraints.is_empty() {
            result.push_str(&Self::constraints(&constraints));
        }
        
        // Examples
        if !prop.examples.is_empty() {
            result.push_str(&Self::examples(&prop.examples));
        }
        
        // Nested properties for objects
        if !prop.properties.is_empty() {
            result.push_str(&format!("{}# Properties\n\n", "#".repeat((level + 1).min(6))));
            
            // Sort properties by name for consistent output
            let mut sorted_props: Vec<_> = prop.properties.values().collect();
            sorted_props.sort_by_key(|p| &p.name);
            
            for nested_prop in sorted_props {
                result.push_str(&Self::property(nested_prop, level + 2));
            }
        }
        
        // Additional properties
        if let Some(ref additional) = prop.additional_properties {
            result.push_str(&format!("{}# Additional Properties\n\n", "#".repeat((level + 1).min(6))));
            result.push_str("Any additional properties must conform to:\n\n");
            result.push_str(&Self::property(additional, level + 2));
        }
        
        result
    }

    /// Generate enum values documentation
    pub fn enum_values(values: &[Value]) -> String {
        let mut result = String::from("**Allowed Values:**\n\n");
        
        for value in values {
            let value_str = match value {
                Value::String(s) => format!("\"{}\"", s),
                _ => serde_json::to_string(value).unwrap_or_else(|_| "null".to_string()),
            };
            result.push_str(&format!("- `{}`\n", value_str));
        }
        
        result.push_str("\n");
        result
    }

    /// Generate constraints documentation
    pub fn constraints(constraints: &[String]) -> String {
        let mut result = String::from("**Constraints:**\n\n");
        
        for constraint in constraints {
            result.push_str(&format!("- {}\n", constraint));
        }
        
        result.push_str("\n");
        result
    }

    /// Generate examples section
    pub fn examples(examples: &[Value]) -> String {
        let mut result = String::from("**Examples:**\n\n");
        
        for (i, example) in examples.iter().enumerate() {
            if examples.len() > 1 {
                result.push_str(&format!("Example {}:\n", i + 1));
            }
            
            result.push_str("```json\n");
            result.push_str(&serde_json::to_string_pretty(example).unwrap_or_else(|_| "null".to_string()));
            result.push_str("\n```\n\n");
        }
        
        result
    }

    /// Generate a complete example for a property based on its constraints
    pub fn synthesize_example(prop: &PropertyInfo) -> Value {
        Self::synthesize_example_with_config(prop, true)
    }
    
    /// Generate a complete example with configuration for deprecated fields
    pub fn synthesize_example_with_config(prop: &PropertyInfo, include_deprecated: bool) -> Value {
        match prop.type_info.base_type.as_str() {
            "string" => {
                if let Some(ref const_val) = prop.const_value {
                    const_val.clone()
                } else if let Some(ref values) = prop.enum_values {
                    values.first().cloned().unwrap_or(Value::String("example".to_string()))
                } else if let Some(ref default) = prop.default {
                    default.clone()
                } else if !prop.examples.is_empty() {
                    prop.examples[0].clone()
                } else {
                    match prop.type_info.format.as_deref() {
                        Some("email") => Value::String("user@example.com".to_string()),
                        Some("uri") => Value::String("https://example.com".to_string()),
                        Some("date") => Value::String("2025-01-31".to_string()),
                        Some("date-time") => Value::String("2025-01-31T12:00:00Z".to_string()),
                        Some("uuid") => Value::String("550e8400-e29b-41d4-a716-446655440000".to_string()),
                        _ => Value::String(format!("example_{}", prop.name)),
                    }
                }
            }
            "number" => {
                if let Some(ref default) = prop.default {
                    default.clone()
                } else if let Some(min) = prop.minimum {
                    Value::Number(serde_json::Number::from_f64(min).unwrap())
                } else {
                    Value::Number(serde_json::Number::from(0))
                }
            }
            "integer" => {
                if let Some(ref default) = prop.default {
                    default.clone()
                } else if let Some(min) = prop.minimum {
                    Value::Number(serde_json::Number::from_f64(min).unwrap())
                } else {
                    Value::Number(serde_json::Number::from(0))
                }
            }
            "boolean" => {
                prop.default.clone().unwrap_or(Value::Bool(false))
            }
            "array" => {
                if let Some(ref default) = prop.default {
                    default.clone()
                } else if !prop.examples.is_empty() {
                    prop.examples[0].clone()
                } else if let Some(ref items_type) = prop.type_info.array_items {
                    // Create example array with one item
                    let item_prop = PropertyInfo {
                        name: "item".to_string(),
                        type_info: items_type.as_ref().clone(),
                        description: None,
                        required: false,
                        default: None,
                        enum_values: None,
                        const_value: None,
                        examples: Vec::new(),
                        pattern: None,
                        min_length: None,
                        max_length: None,
                        minimum: None,
                        maximum: None,
                        min_items: None,
                        max_items: None,
                        unique_items: false,
                        properties: HashMap::new(),
                        additional_properties: None,
                        deprecated: false,
                        read_only: false,
                        write_only: false,
                    };
                    Value::Array(vec![Self::synthesize_example_with_config(&item_prop, include_deprecated)])
                } else {
                    Value::Array(Vec::new())
                }
            }
            "object" => {
                if let Some(ref default) = prop.default {
                    default.clone()
                } else if !prop.examples.is_empty() {
                    prop.examples[0].clone()
                } else {
                    let mut obj = serde_json::Map::new();
                    
                    // Add all required properties first (excluding deprecated if configured)
                    for (key, nested_prop) in &prop.properties {
                        if nested_prop.required && (include_deprecated || !nested_prop.deprecated) {
                            obj.insert(key.clone(), Self::synthesize_example_with_config(nested_prop, include_deprecated));
                        }
                    }
                    
                    // Add some optional properties for completeness (up to 3)
                    let mut optional_count = 0;
                    for (key, nested_prop) in &prop.properties {
                        if !nested_prop.required && optional_count < 3 && (include_deprecated || !nested_prop.deprecated) {
                            obj.insert(key.clone(), Self::synthesize_example_with_config(nested_prop, include_deprecated));
                            optional_count += 1;
                        }
                    }
                    
                    Value::Object(obj)
                }
            }
            _ => Value::Null,
        }
    }

    /// Generate footer section
    pub fn footer() -> String {
        "\n---\n\n*Generated by Specado Schema Documentation Generator*\n".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documentation::parser::TypeInfo;
    use std::collections::HashMap;
    use serde_json::json;

    #[test]
    fn test_header_generation() {
        let header = Template::header("Test Schema", Some("This is a test schema"));
        assert!(header.contains("# Test Schema"));
        assert!(header.contains("This is a test schema"));
    }

    #[test]
    fn test_table_of_contents() {
        let sections = vec![
            ("Overview", 1),
            ("Properties", 1),
            ("name", 2),
            ("age", 2),
            ("Examples", 1),
        ];
        
        let toc = Template::table_of_contents(&sections);
        assert!(toc.contains("[Overview](#overview)"));
        assert!(toc.contains("  - [name](#name)"));
        assert!(toc.contains("  - [age](#age)"));
    }

    #[test]
    fn test_enum_values_generation() {
        let values = vec![
            json!("chat"),
            json!("completion"),
            json!("embedding"),
        ];
        
        let doc = Template::enum_values(&values);
        assert!(doc.contains("- `\"chat\"`"));
        assert!(doc.contains("- `\"completion\"`"));
        assert!(doc.contains("- `\"embedding\"`"));
    }

    #[test]
    fn test_example_synthesis() {
        let prop = PropertyInfo {
            name: "email".to_string(),
            type_info: TypeInfo {
                base_type: "string".to_string(),
                format: Some("email".to_string()),
                array_items: None,
                nullable: false,
            },
            description: None,
            required: true,
            default: None,
            enum_values: None,
            const_value: None,
            examples: Vec::new(),
            pattern: None,
            min_length: None,
            max_length: None,
            minimum: None,
            maximum: None,
            min_items: None,
            max_items: None,
            unique_items: false,
            properties: HashMap::new(),
            additional_properties: None,
            deprecated: false,
            read_only: false,
            write_only: false,
        };
        
        let example = Template::synthesize_example(&prop);
        assert_eq!(example, json!("user@example.com"));
    }
}