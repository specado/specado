//! Schema migration hints and guidance
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::versioning::version::SchemaVersion;
use std::fmt;

/// A migration hint for upgrading between schema versions
#[derive(Debug, Clone)]
pub struct MigrationHint {
    pub from_version: SchemaVersion,
    pub to_version: SchemaVersion,
    pub change_type: ChangeType,
    pub field_path: String,
    pub description: String,
    pub action_required: String,
    pub example: Option<String>,
}

/// Type of change in a migration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    /// Field was added
    FieldAdded,
    /// Field was removed
    FieldRemoved,
    /// Field was renamed
    FieldRenamed { old_name: String, new_name: String },
    /// Field type changed
    TypeChanged { old_type: String, new_type: String },
    /// Field became required
    BecameRequired,
    /// Field became optional
    BecameOptional,
    /// Enum value added
    EnumValueAdded(String),
    /// Enum value removed
    EnumValueRemoved(String),
    /// Semantic change
    SemanticChange,
    /// Deprecation
    Deprecated { replacement: Option<String> },
}

impl fmt::Display for ChangeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChangeType::FieldAdded => write!(f, "Field added"),
            ChangeType::FieldRemoved => write!(f, "Field removed"),
            ChangeType::FieldRenamed { old_name, new_name } => {
                write!(f, "Field renamed from '{}' to '{}'", old_name, new_name)
            }
            ChangeType::TypeChanged { old_type, new_type } => {
                write!(f, "Type changed from '{}' to '{}'", old_type, new_type)
            }
            ChangeType::BecameRequired => write!(f, "Field became required"),
            ChangeType::BecameOptional => write!(f, "Field became optional"),
            ChangeType::EnumValueAdded(value) => write!(f, "Enum value '{}' added", value),
            ChangeType::EnumValueRemoved(value) => write!(f, "Enum value '{}' removed", value),
            ChangeType::SemanticChange => write!(f, "Semantic change"),
            ChangeType::Deprecated { replacement } => {
                if let Some(repl) = replacement {
                    write!(f, "Deprecated (use '{}' instead)", repl)
                } else {
                    write!(f, "Deprecated")
                }
            }
        }
    }
}

/// Registry of migration hints between schema versions
#[derive(Debug, Clone)]
pub struct MigrationRegistry {
    hints: Vec<MigrationHint>,
}

impl MigrationRegistry {
    /// Create a new migration registry
    pub fn new() -> Self {
        Self { hints: Vec::new() }
    }

    /// Add a migration hint
    pub fn add_hint(&mut self, hint: MigrationHint) {
        self.hints.push(hint);
    }

    /// Get migration hints for upgrading from one version to another
    pub fn get_hints(&self, from: &SchemaVersion, to: &SchemaVersion) -> Vec<MigrationHint> {
        self.hints
            .iter()
            .filter(|hint| {
                // Include hints that apply during migration from 'from' to 'to'
                hint.from_version >= *from && hint.to_version <= *to
            })
            .cloned()
            .collect()
    }

    /// Get breaking changes between versions
    pub fn get_breaking_changes(&self, from: &SchemaVersion, to: &SchemaVersion) -> Vec<MigrationHint> {
        self.get_hints(from, to)
            .into_iter()
            .filter(|hint| matches!(
                hint.change_type,
                ChangeType::FieldRemoved |
                ChangeType::TypeChanged { .. } |
                ChangeType::BecameRequired |
                ChangeType::EnumValueRemoved(_) |
                ChangeType::SemanticChange
            ))
            .collect()
    }

    /// Generate a migration guide as markdown
    pub fn generate_migration_guide(&self, from: &SchemaVersion, to: &SchemaVersion) -> String {
        let hints = self.get_hints(from, to);
        let breaking_changes = self.get_breaking_changes(from, to);

        let mut guide = format!("# Migration Guide: v{} to v{}\n\n", from, to);

        if breaking_changes.is_empty() {
            guide.push_str("✅ No breaking changes\n\n");
        } else {
            guide.push_str(&format!("⚠️ {} breaking change(s)\n\n", breaking_changes.len()));
            guide.push_str("## Breaking Changes\n\n");

            for hint in &breaking_changes {
                guide.push_str(&format!("### {}\n", hint.field_path));
                guide.push_str(&format!("- **Change**: {}\n", hint.change_type));
                guide.push_str(&format!("- **Description**: {}\n", hint.description));
                guide.push_str(&format!("- **Action Required**: {}\n", hint.action_required));
                
                if let Some(ref example) = hint.example {
                    guide.push_str(&format!("\n**Example**:\n```json\n{}\n```\n", example));
                }
                guide.push('\n');
            }
        }

        let non_breaking: Vec<_> = hints.iter()
            .filter(|h| !breaking_changes.iter().any(|b| b.field_path == h.field_path))
            .collect();

        if !non_breaking.is_empty() {
            guide.push_str("## Non-Breaking Changes\n\n");
            
            for hint in non_breaking {
                guide.push_str(&format!("### {}\n", hint.field_path));
                guide.push_str(&format!("- **Change**: {}\n", hint.change_type));
                guide.push_str(&format!("- **Description**: {}\n", hint.description));
                
                if let Some(ref example) = hint.example {
                    guide.push_str(&format!("\n**Example**:\n```json\n{}\n```\n", example));
                }
                guide.push('\n');
            }
        }

        guide
    }
}

impl Default for MigrationRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        
        // Add some example migrations for PromptSpec
        registry.add_hint(MigrationHint {
            from_version: SchemaVersion::new(1, 0, 0),
            to_version: SchemaVersion::new(1, 1, 0),
            change_type: ChangeType::FieldAdded,
            field_path: "$.conversation".to_string(),
            description: "Added conversation management support".to_string(),
            action_required: "No action required - field is optional".to_string(),
            example: Some(r#"{
  "conversation": {
    "parent_message_id": "msg-123",
    "conversation_id": "conv-456"
  }
}"#.to_string()),
        });

        registry.add_hint(MigrationHint {
            from_version: SchemaVersion::new(1, 1, 0),
            to_version: SchemaVersion::new(2, 0, 0),
            change_type: ChangeType::FieldRenamed {
                old_name: "max_tokens".to_string(),
                new_name: "limits.max_tokens".to_string(),
            },
            field_path: "$.max_tokens".to_string(),
            description: "Token limits moved to dedicated limits object".to_string(),
            action_required: "Move max_tokens field into limits object".to_string(),
            example: Some(r#"// Before:
{
  "max_tokens": 1000
}

// After:
{
  "limits": {
    "max_tokens": 1000
  }
}"#.to_string()),
        });

        registry.add_hint(MigrationHint {
            from_version: SchemaVersion::new(1, 0, 0),
            to_version: SchemaVersion::new(1, 2, 0),
            change_type: ChangeType::Deprecated {
                replacement: Some("sampling.temperature".to_string()),
            },
            field_path: "$.temperature".to_string(),
            description: "Temperature moved to sampling configuration".to_string(),
            action_required: "Move temperature to sampling.temperature".to_string(),
            example: None,
        });

        registry
    }
}

/// Check if a migration is needed between versions
pub fn needs_migration(from: &SchemaVersion, to: &SchemaVersion) -> bool {
    from != to
}

/// Get a summary of changes between versions
pub fn get_migration_summary(from: &SchemaVersion, to: &SchemaVersion) -> String {
    let registry = MigrationRegistry::default();
    let hints = registry.get_hints(from, to);
    let breaking = registry.get_breaking_changes(from, to);
    
    format!(
        "Migration from v{} to v{}: {} total changes ({} breaking)",
        from,
        to,
        hints.len(),
        breaking.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_hints() {
        let mut registry = MigrationRegistry::new();
        
        registry.add_hint(MigrationHint {
            from_version: SchemaVersion::new(1, 0, 0),
            to_version: SchemaVersion::new(1, 1, 0),
            change_type: ChangeType::FieldAdded,
            field_path: "$.newField".to_string(),
            description: "Added new field".to_string(),
            action_required: "None".to_string(),
            example: None,
        });

        registry.add_hint(MigrationHint {
            from_version: SchemaVersion::new(1, 1, 0),
            to_version: SchemaVersion::new(2, 0, 0),
            change_type: ChangeType::FieldRemoved,
            field_path: "$.oldField".to_string(),
            description: "Removed deprecated field".to_string(),
            action_required: "Remove field from spec".to_string(),
            example: None,
        });

        let v1_0 = SchemaVersion::new(1, 0, 0);
        let v1_1 = SchemaVersion::new(1, 1, 0);
        let v2_0 = SchemaVersion::new(2, 0, 0);

        // Get hints for 1.0 to 1.1
        let hints = registry.get_hints(&v1_0, &v1_1);
        assert_eq!(hints.len(), 1);
        assert!(matches!(hints[0].change_type, ChangeType::FieldAdded));

        // Get hints for 1.0 to 2.0
        let hints = registry.get_hints(&v1_0, &v2_0);
        assert_eq!(hints.len(), 2);

        // Get breaking changes
        let breaking = registry.get_breaking_changes(&v1_0, &v2_0);
        assert_eq!(breaking.len(), 1);
        assert!(matches!(breaking[0].change_type, ChangeType::FieldRemoved));
    }

    #[test]
    fn test_change_type_display() {
        let change = ChangeType::FieldRenamed {
            old_name: "foo".to_string(),
            new_name: "bar".to_string(),
        };
        assert_eq!(change.to_string(), "Field renamed from 'foo' to 'bar'");

        let change = ChangeType::Deprecated {
            replacement: Some("newField".to_string()),
        };
        assert_eq!(change.to_string(), "Deprecated (use 'newField' instead)");
    }

    #[test]
    fn test_migration_guide_generation() {
        let registry = MigrationRegistry::default();
        let v1_0 = SchemaVersion::new(1, 0, 0);
        let v2_0 = SchemaVersion::new(2, 0, 0);

        let guide = registry.generate_migration_guide(&v1_0, &v2_0);
        assert!(guide.contains("Migration Guide"));
        assert!(guide.contains("v1.0.0 to v2.0.0"));
    }
}