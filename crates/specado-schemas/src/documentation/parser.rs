//! JSON Schema parser for documentation generation
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use serde_json::Value;
use std::collections::HashMap;

/// Type information extracted from schema
#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub base_type: String,
    pub format: Option<String>,
    pub array_items: Option<Box<TypeInfo>>,
    pub nullable: bool,
}

impl TypeInfo {
    /// Create type info from JSON Schema type
    pub fn from_schema(schema: &Value) -> Self {
        let base_type = if let Some(types) = schema.get("type") {
            if let Some(type_str) = types.as_str() {
                type_str.to_string()
            } else if let Some(type_array) = types.as_array() {
                // Handle nullable types ["string", "null"]
                let mut nullable = false;
                let mut base = String::new();
                for t in type_array {
                    if let Some(type_str) = t.as_str() {
                        if type_str == "null" {
                            nullable = true;
                        } else {
                            base = type_str.to_string();
                        }
                    }
                }
                return TypeInfo {
                    base_type: base,
                    format: schema.get("format").and_then(|f| f.as_str()).map(String::from),
                    array_items: None,
                    nullable,
                };
            } else {
                "unknown".to_string()
            }
        } else if schema.get("$ref").is_some() {
            "reference".to_string()
        } else if schema.get("enum").is_some() {
            "enum".to_string()
        } else if schema.get("const").is_some() {
            "const".to_string()
        } else if schema.get("anyOf").is_some() {
            "anyOf".to_string()
        } else if schema.get("oneOf").is_some() {
            "oneOf".to_string()
        } else if schema.get("allOf").is_some() {
            "allOf".to_string()
        } else {
            "unknown".to_string()
        };

        let format = schema.get("format").and_then(|f| f.as_str()).map(String::from);

        let array_items = if base_type == "array" {
            schema.get("items").map(|items| Box::new(TypeInfo::from_schema(items)))
        } else {
            None
        };

        TypeInfo {
            base_type,
            format,
            array_items,
            nullable: false,
        }
    }

    /// Get display string for the type
    pub fn display_type(&self) -> String {
        let mut result = match self.base_type.as_str() {
            "string" => {
                if let Some(ref fmt) = self.format {
                    format!("string ({})", fmt)
                } else {
                    "string".to_string()
                }
            }
            "number" => "number".to_string(),
            "integer" => "integer".to_string(),
            "boolean" => "boolean".to_string(),
            "object" => "object".to_string(),
            "array" => {
                if let Some(ref items) = self.array_items {
                    format!("array<{}>", items.display_type())
                } else {
                    "array".to_string()
                }
            }
            "enum" => "enum".to_string(),
            "const" => "const".to_string(),
            "anyOf" => "anyOf".to_string(),
            "oneOf" => "oneOf".to_string(),
            "allOf" => "allOf".to_string(),
            "reference" => "reference".to_string(),
            _ => self.base_type.clone(),
        };

        if self.nullable {
            result.push_str(" | null");
        }

        result
    }
}

/// Property information extracted from schema
#[derive(Debug, Clone)]
pub struct PropertyInfo {
    pub name: String,
    pub type_info: TypeInfo,
    pub description: Option<String>,
    pub required: bool,
    pub default: Option<Value>,
    pub enum_values: Option<Vec<Value>>,
    pub const_value: Option<Value>,
    pub examples: Vec<Value>,
    pub pattern: Option<String>,
    pub min_length: Option<u64>,
    pub max_length: Option<u64>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub min_items: Option<u64>,
    pub max_items: Option<u64>,
    pub unique_items: bool,
    pub properties: HashMap<String, PropertyInfo>,
    pub additional_properties: Option<Box<PropertyInfo>>,
    pub deprecated: bool,
    pub read_only: bool,
    pub write_only: bool,
}

impl PropertyInfo {
    /// Create property info from schema
    pub fn from_schema(name: String, schema: &Value, required: bool) -> Self {
        let type_info = TypeInfo::from_schema(schema);
        let description = schema.get("description").and_then(|d| d.as_str()).map(String::from);
        let default = schema.get("default").cloned();
        let enum_values = schema.get("enum").and_then(|e| e.as_array()).cloned();
        let const_value = schema.get("const").cloned();
        
        let examples = if let Some(ex) = schema.get("examples") {
            if let Some(arr) = ex.as_array() {
                arr.clone()
            } else {
                vec![ex.clone()]
            }
        } else if let Some(ex) = schema.get("example") {
            vec![ex.clone()]
        } else {
            Vec::new()
        };

        let pattern = schema.get("pattern").and_then(|p| p.as_str()).map(String::from);
        let min_length = schema.get("minLength").and_then(|m| m.as_u64());
        let max_length = schema.get("maxLength").and_then(|m| m.as_u64());
        let minimum = schema.get("minimum").and_then(|m| m.as_f64());
        let maximum = schema.get("maximum").and_then(|m| m.as_f64());
        let min_items = schema.get("minItems").and_then(|m| m.as_u64());
        let max_items = schema.get("maxItems").and_then(|m| m.as_u64());
        let unique_items = schema.get("uniqueItems").and_then(|u| u.as_bool()).unwrap_or(false);
        
        let deprecated = schema.get("deprecated").and_then(|d| d.as_bool()).unwrap_or(false);
        let read_only = schema.get("readOnly").and_then(|r| r.as_bool()).unwrap_or(false);
        let write_only = schema.get("writeOnly").and_then(|w| w.as_bool()).unwrap_or(false);

        // Parse nested properties for objects
        let properties = if type_info.base_type == "object" {
            if let Some(props) = schema.get("properties") {
                if let Some(props_obj) = props.as_object() {
                    let required_fields = schema.get("required")
                        .and_then(|r| r.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str())
                                .map(String::from)
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();

                    props_obj.iter()
                        .map(|(key, value)| {
                            let is_required = required_fields.contains(key);
                            (key.clone(), PropertyInfo::from_schema(key.clone(), value, is_required))
                        })
                        .collect()
                } else {
                    HashMap::new()
                }
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };

        let additional_properties = if type_info.base_type == "object" {
            schema.get("additionalProperties").and_then(|ap| {
                if ap.is_object() {
                    Some(Box::new(PropertyInfo::from_schema("*".to_string(), ap, false)))
                } else {
                    None
                }
            })
        } else {
            None
        };

        PropertyInfo {
            name,
            type_info,
            description,
            required,
            default,
            enum_values,
            const_value,
            examples,
            pattern,
            min_length,
            max_length,
            minimum,
            maximum,
            min_items,
            max_items,
            unique_items,
            properties,
            additional_properties,
            deprecated,
            read_only,
            write_only,
        }
    }

    /// Get all constraints as a formatted string
    pub fn get_constraints(&self) -> Vec<String> {
        let mut constraints = Vec::new();

        if self.required {
            constraints.push("Required".to_string());
        }
        
        if self.deprecated {
            constraints.push("⚠️ Deprecated".to_string());
        }
        
        if self.read_only {
            constraints.push("Read-only".to_string());
        }
        
        if self.write_only {
            constraints.push("Write-only".to_string());
        }

        if let Some(ref pattern) = self.pattern {
            constraints.push(format!("Pattern: `{}`", pattern));
        }

        if let Some(min) = self.min_length {
            constraints.push(format!("Min length: {}", min));
        }

        if let Some(max) = self.max_length {
            constraints.push(format!("Max length: {}", max));
        }

        if let Some(min) = self.minimum {
            constraints.push(format!("Min: {}", min));
        }

        if let Some(max) = self.maximum {
            constraints.push(format!("Max: {}", max));
        }

        if let Some(min) = self.min_items {
            constraints.push(format!("Min items: {}", min));
        }

        if let Some(max) = self.max_items {
            constraints.push(format!("Max items: {}", max));
        }

        if self.unique_items {
            constraints.push("Unique items".to_string());
        }

        constraints
    }
}

/// Schema parser for extracting documentation information
pub struct SchemaParser;

impl SchemaParser {
    /// Parse a JSON Schema and extract documentation information
    pub fn parse(schema: &Value) -> PropertyInfo {
        let name = schema.get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Schema")
            .to_string();
        
        PropertyInfo::from_schema(name, schema, false)
    }

    /// Extract all properties recursively
    pub fn extract_all_properties(info: &PropertyInfo) -> Vec<&PropertyInfo> {
        let mut result = vec![info];
        
        for prop in info.properties.values() {
            result.extend(Self::extract_all_properties(prop));
        }
        
        if let Some(ref additional) = info.additional_properties {
            result.extend(Self::extract_all_properties(additional));
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_type_info_from_schema() {
        let schema = json!({
            "type": "string",
            "format": "email"
        });
        
        let type_info = TypeInfo::from_schema(&schema);
        assert_eq!(type_info.base_type, "string");
        assert_eq!(type_info.format, Some("email".to_string()));
        assert_eq!(type_info.display_type(), "string (email)");
    }

    #[test]
    fn test_array_type_info() {
        let schema = json!({
            "type": "array",
            "items": {
                "type": "number"
            }
        });
        
        let type_info = TypeInfo::from_schema(&schema);
        assert_eq!(type_info.base_type, "array");
        assert!(type_info.array_items.is_some());
        assert_eq!(type_info.display_type(), "array<number>");
    }

    #[test]
    fn test_property_info_with_constraints() {
        let schema = json!({
            "type": "string",
            "description": "User email",
            "format": "email",
            "minLength": 5,
            "maxLength": 100,
            "pattern": "^[^@]+@[^@]+$",
            "examples": ["user@example.com"]
        });
        
        let prop = PropertyInfo::from_schema("email".to_string(), &schema, true);
        assert_eq!(prop.name, "email");
        assert_eq!(prop.description, Some("User email".to_string()));
        assert_eq!(prop.min_length, Some(5));
        assert_eq!(prop.max_length, Some(100));
        assert!(!prop.examples.is_empty());
        
        let constraints = prop.get_constraints();
        assert!(constraints.contains(&"Required".to_string()));
        assert!(constraints.contains(&"Min length: 5".to_string()));
        assert!(constraints.contains(&"Max length: 100".to_string()));
    }

    #[test]
    fn test_enum_property() {
        let schema = json!({
            "type": "string",
            "enum": ["chat", "completion", "embedding"],
            "description": "Model type"
        });
        
        let prop = PropertyInfo::from_schema("model_type".to_string(), &schema, false);
        assert!(prop.enum_values.is_some());
        assert_eq!(prop.enum_values.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_object_properties() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "User name"
                },
                "age": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 150
                }
            },
            "required": ["name"]
        });
        
        let prop = PropertyInfo::from_schema("user".to_string(), &schema, false);
        assert_eq!(prop.properties.len(), 2);
        assert!(prop.properties.get("name").unwrap().required);
        assert!(!prop.properties.get("age").unwrap().required);
    }
}