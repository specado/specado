//! Snapshot management for golden tests

use crate::{GoldenError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// A test snapshot containing expected output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Name of the test
    pub name: String,
    
    /// Test metadata
    pub metadata: SnapshotMetadata,
    
    /// The actual snapshot content
    pub content: Value,
    
    /// Fields to ignore during comparison
    #[serde(default)]
    pub ignore_fields: Vec<String>,
    
    /// Fields that may vary (use regex matching)
    #[serde(default)]
    pub volatile_fields: Vec<VolatileField>,
}

/// Metadata about a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// Version of the snapshot format
    pub version: String,
    
    /// When the snapshot was created
    pub created_at: String,
    
    /// When the snapshot was last updated
    pub updated_at: String,
    
    /// Description of what this tests
    pub description: Option<String>,
    
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A field that may change between runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatileField {
    /// JSONPath to the field
    pub path: String,
    
    /// Regular expression the field must match
    pub pattern: String,
}

/// Manages reading and writing snapshots
pub struct SnapshotManager {
    snapshot_dir: PathBuf,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub fn new(snapshot_dir: impl AsRef<Path>) -> Self {
        Self {
            snapshot_dir: snapshot_dir.as_ref().to_path_buf(),
        }
    }
    
    /// Load a snapshot from disk
    pub fn load(&self, name: &str) -> Result<Snapshot> {
        let path = self.snapshot_path(name);
        
        if !path.exists() {
            return Err(GoldenError::CorpusError(format!(
                "Snapshot '{}' not found at {:?}",
                name, path
            )));
        }
        
        let content = fs::read_to_string(&path)?;
        let snapshot: Snapshot = serde_json::from_str(&content)?;
        
        Ok(snapshot)
    }
    
    /// Save a snapshot to disk
    pub fn save(&self, snapshot: &Snapshot) -> Result<()> {
        let path = self.snapshot_path(&snapshot.name);
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Pretty-print the JSON for readability
        let content = serde_json::to_string_pretty(snapshot)?;
        fs::write(&path, content)?;
        
        Ok(())
    }
    
    /// Update an existing snapshot
    pub fn update(&self, name: &str, new_content: Value) -> Result<()> {
        let mut snapshot = self.load(name)?;
        
        // Update content and metadata
        snapshot.content = new_content;
        snapshot.metadata.updated_at = Utc::now().to_rfc3339();
        
        self.save(&snapshot)
    }
    
    /// Create a new snapshot
    pub fn create(
        &self,
        name: &str,
        content: Value,
        description: Option<String>,
    ) -> Result<Snapshot> {
        let now = Utc::now().to_rfc3339();
        
        let snapshot = Snapshot {
            name: name.to_string(),
            metadata: SnapshotMetadata {
                version: "1.0.0".to_string(),
                created_at: now.clone(),
                updated_at: now,
                description,
                tags: Vec::new(),
            },
            content,
            ignore_fields: Vec::new(),
            volatile_fields: Vec::new(),
        };
        
        self.save(&snapshot)?;
        Ok(snapshot)
    }
    
    /// Check if a snapshot exists
    pub fn exists(&self, name: &str) -> bool {
        self.snapshot_path(name).exists()
    }
    
    /// List all snapshots
    pub fn list(&self) -> Result<Vec<String>> {
        let mut snapshots = Vec::new();
        
        if !self.snapshot_dir.exists() {
            return Ok(snapshots);
        }
        
        for entry in fs::read_dir(&self.snapshot_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    snapshots.push(stem.to_string());
                }
            }
        }
        
        snapshots.sort();
        Ok(snapshots)
    }
    
    /// Delete a snapshot
    pub fn delete(&self, name: &str) -> Result<()> {
        let path = self.snapshot_path(name);
        
        if path.exists() {
            fs::remove_file(path)?;
        }
        
        Ok(())
    }
    
    /// Get the path for a snapshot
    fn snapshot_path(&self, name: &str) -> PathBuf {
        let filename = if name.ends_with(".json") {
            name.to_string()
        } else {
            format!("{}.json", name)
        };
        
        self.snapshot_dir.join(filename)
    }
    
    /// Create a backup of a snapshot before updating
    pub fn backup(&self, name: &str) -> Result<()> {
        let source = self.snapshot_path(name);
        
        if !source.exists() {
            return Ok(());
        }
        
        let backup_name = format!("{}.backup.{}", name, Utc::now().timestamp());
        let backup_path = self.snapshot_path(&backup_name);
        
        fs::copy(source, backup_path)?;
        Ok(())
    }
}

/// Normalize JSON for comparison
pub fn normalize_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut normalized = serde_json::Map::new();
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by_key(|(k, _)| k.as_str());
            
            for (key, val) in entries {
                normalized.insert(key.clone(), normalize_json(val));
            }
            
            Value::Object(normalized)
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(normalize_json).collect())
        }
        Value::String(s) => {
            // Trim whitespace for comparison
            Value::String(s.trim().to_string())
        }
        Value::Number(n) => {
            // Round floating point numbers to avoid precision issues
            if let Some(f) = n.as_f64() {
                let rounded = (f * 1000000.0).round() / 1000000.0;
                serde_json::Number::from_f64(rounded)
                    .map(Value::Number)
                    .unwrap_or(value.clone())
            } else {
                value.clone()
            }
        }
        _ => value.clone(),
    }
}

/// Apply ignore fields to a JSON value
pub fn apply_ignores(value: &mut Value, ignore_fields: &[String]) {
    for field_path in ignore_fields {
        remove_field_by_path(value, field_path);
    }
}

/// Remove a field by JSONPath-like notation
fn remove_field_by_path(value: &mut Value, path: &str) {
    let parts: Vec<&str> = path.split('.').filter(|s| !s.is_empty()).collect();
    
    if parts.is_empty() {
        return;
    }
    
    remove_field_recursive(value, &parts);
}

fn remove_field_recursive(value: &mut Value, path_parts: &[&str]) {
    if path_parts.is_empty() {
        return;
    }
    
    let (first, rest) = path_parts.split_first().unwrap();
    
    match value {
        Value::Object(map) => {
            if rest.is_empty() {
                // Remove the field
                map.remove(*first);
            } else if let Some(next_value) = map.get_mut(*first) {
                // Continue traversing
                remove_field_recursive(next_value, rest);
            }
        }
        Value::Array(arr) => {
            // Apply to all array elements
            for item in arr {
                remove_field_recursive(item, path_parts);
            }
        }
        _ => {}
    }
}

use chrono::Utc;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;
    
    #[test]
    fn test_snapshot_manager_create_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SnapshotManager::new(temp_dir.path());
        
        let content = json!({
            "model": "gpt-4",
            "messages": [{"role": "user", "content": "Hello"}]
        });
        
        let snapshot = manager
            .create("test", content.clone(), Some("Test snapshot".to_string()))
            .unwrap();
        
        assert_eq!(snapshot.name, "test");
        assert_eq!(snapshot.content, content);
        
        let loaded = manager.load("test").unwrap();
        assert_eq!(loaded.name, "test");
        assert_eq!(loaded.content, content);
    }
    
    #[test]
    fn test_normalize_json() {
        let input = json!({
            "b": 2,
            "a": 1,
            "c": {
                "d": 3.14159265359,
                "e": "  spaced  "
            }
        });
        
        let normalized = normalize_json(&input);
        
        // Check that keys are sorted
        let obj = normalized.as_object().unwrap();
        let keys: Vec<_> = obj.keys().collect();
        assert_eq!(keys, vec!["a", "b", "c"]);
        
        // Check that nested values are normalized
        let c_val = &obj["c"];
        assert!(c_val.is_object());
        
        // Check float rounding
        let d_val = c_val["d"].as_f64().unwrap();
        assert!((d_val - 3.141593).abs() < 0.000001);
        
        // Check string trimming
        assert_eq!(c_val["e"].as_str().unwrap(), "spaced");
    }
    
    #[test]
    fn test_apply_ignores() {
        let mut value = json!({
            "model": "gpt-4",
            "timestamp": "2025-01-01T00:00:00Z",
            "nested": {
                "keep": "this",
                "remove": "that"
            }
        });
        
        let ignore_fields = vec![
            "timestamp".to_string(),
            "nested.remove".to_string(),
        ];
        
        apply_ignores(&mut value, &ignore_fields);
        
        assert!(!value.as_object().unwrap().contains_key("timestamp"));
        assert!(value["nested"]["keep"].is_string());
        assert!(!value["nested"].as_object().unwrap().contains_key("remove"));
    }
}