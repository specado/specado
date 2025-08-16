//! Caching layer for schema loading performance optimization
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::loader::error::{LoaderError, LoaderResult};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Cache entry containing schema data and metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// The cached schema content
    pub content: Value,
    /// When this entry was cached
    pub cached_at: SystemTime,
    /// File modification time when cached
    pub file_mtime: SystemTime,
    /// Original file path
    pub file_path: PathBuf,
    /// Schema version if available
    pub version: Option<String>,
}

impl CacheEntry {
    /// Create a new cache entry
    pub fn new(content: Value, file_path: PathBuf, file_mtime: SystemTime) -> Self {
        let version = content
            .get("spec_version")
            .or_else(|| content.get("version"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Self {
            content,
            cached_at: SystemTime::now(),
            file_mtime,
            file_path,
            version,
        }
    }

    /// Check if this cache entry is still valid
    pub fn is_valid(&self, current_mtime: SystemTime, max_age: Option<Duration>) -> bool {
        // Check file modification time
        if current_mtime > self.file_mtime {
            return false;
        }

        // Check cache age if specified
        if let Some(max_age) = max_age {
            if let Ok(elapsed) = self.cached_at.elapsed() {
                if elapsed > max_age {
                    return false;
                }
            }
        }

        true
    }

    /// Get the age of this cache entry
    pub fn age(&self) -> Option<Duration> {
        self.cached_at.elapsed().ok()
    }
}

/// Configuration for cache behavior
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum cache size (number of entries)
    pub max_entries: usize,
    /// Maximum age for cache entries
    pub max_age: Option<Duration>,
    /// Whether to enable cache
    pub enabled: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            max_age: Some(Duration::from_secs(3600)), // 1 hour
            enabled: true,
        }
    }
}

/// In-memory cache for loaded schemas
#[derive(Debug)]
pub struct SchemaCache {
    entries: HashMap<PathBuf, CacheEntry>,
    config: CacheConfig,
    access_order: Vec<PathBuf>, // For LRU eviction
}

impl SchemaCache {
    /// Create a new schema cache with default configuration
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new schema cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            entries: HashMap::new(),
            config,
            access_order: Vec::new(),
        }
    }

    /// Get a cached entry if it exists and is valid
    pub fn get(&mut self, path: &Path) -> LoaderResult<Option<CacheEntry>> {
        if !self.config.enabled {
            return Ok(None);
        }

        let canonical_path = self.canonicalize_path(path)?;

        // First check if we have an entry
        let entry_valid = if let Some(entry) = self.entries.get(&canonical_path) {
            // Check if file still exists and get its modification time
            let current_mtime = std::fs::metadata(path)
                .map_err(|e| LoaderError::io_error(path.to_path_buf(), e))?
                .modified()
                .map_err(|e| LoaderError::io_error(path.to_path_buf(), e))?;

            entry.is_valid(current_mtime, self.config.max_age)
        } else {
            false
        };

        if entry_valid {
            // Update access order for LRU and return cloned entry
            self.update_access_order(&canonical_path);
            Ok(self.entries.get(&canonical_path).cloned())
        } else {
            // Remove invalid entry if it exists
            self.remove_path(&canonical_path);
            Ok(None)
        }
    }

    /// Cache a schema entry
    pub fn put(&mut self, path: &Path, content: Value) -> LoaderResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let canonical_path = self.canonicalize_path(path)?;

        // Get file modification time
        let file_mtime = std::fs::metadata(path)
            .map_err(|e| LoaderError::io_error(path.to_path_buf(), e))?
            .modified()
            .map_err(|e| LoaderError::io_error(path.to_path_buf(), e))?;

        let entry = CacheEntry::new(content, canonical_path.clone(), file_mtime);

        // Check if we need to evict entries
        if self.entries.len() >= self.config.max_entries {
            self.evict_lru();
        }

        // Insert the entry
        self.entries.insert(canonical_path.clone(), entry);
        self.update_access_order(&canonical_path);

        Ok(())
    }

    /// Remove a specific entry from cache
    pub fn remove(&mut self, path: &Path) -> LoaderResult<bool> {
        let canonical_path = self.canonicalize_path(path)?;
        let removed = self.entries.remove(&canonical_path).is_some();
        self.remove_from_access_order(&canonical_path);
        Ok(removed)
    }

    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let total_size = self.entries.len();
        let mut total_age = Duration::new(0, 0);
        let mut valid_entries = 0;

        for entry in self.entries.values() {
            if let Some(age) = entry.age() {
                total_age += age;
                valid_entries += 1;
            }
        }

        let average_age = if valid_entries > 0 {
            Some(total_age / valid_entries as u32)
        } else {
            None
        };

        CacheStats {
            total_entries: total_size,
            max_entries: self.config.max_entries,
            average_age,
            enabled: self.config.enabled,
        }
    }

    /// Cleanup expired entries
    pub fn cleanup_expired(&mut self) -> LoaderResult<usize> {
        if !self.config.enabled {
            return Ok(0);
        }

        let mut expired_paths = Vec::new();

        for (path, entry) in &self.entries {
            // Check if file still exists
            let current_mtime = match std::fs::metadata(&entry.file_path) {
                Ok(metadata) => metadata.modified().unwrap_or(entry.file_mtime),
                Err(_) => {
                    // File doesn't exist anymore
                    expired_paths.push(path.clone());
                    continue;
                }
            };

            if !entry.is_valid(current_mtime, self.config.max_age) {
                expired_paths.push(path.clone());
            }
        }

        let count = expired_paths.len();
        for path in expired_paths {
            self.remove_path(&path);
        }

        Ok(count)
    }

    /// Check if cache contains a path
    pub fn contains(&self, path: &Path) -> LoaderResult<bool> {
        let canonical_path = self.canonicalize_path(path)?;
        Ok(self.entries.contains_key(&canonical_path))
    }

    fn canonicalize_path(&self, path: &Path) -> LoaderResult<PathBuf> {
        path.canonicalize()
            .map_err(|e| LoaderError::io_error(path.to_path_buf(), e))
    }

    fn evict_lru(&mut self) {
        if let Some(oldest_path) = self.access_order.first().cloned() {
            self.remove_path(&oldest_path);
        }
    }

    fn update_access_order(&mut self, path: &PathBuf) {
        // Remove from current position
        self.access_order.retain(|p| p != path);
        // Add to end (most recently used)
        self.access_order.push(path.clone());
    }

    fn remove_from_access_order(&mut self, path: &PathBuf) {
        self.access_order.retain(|p| p != path);
    }

    fn remove_path(&mut self, path: &PathBuf) {
        self.entries.remove(path);
        self.remove_from_access_order(path);
    }
}

impl Default for SchemaCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub max_entries: usize,
    pub average_age: Option<Duration>,
    pub enabled: bool,
}

impl CacheStats {
    /// Calculate cache utilization as a percentage
    pub fn utilization(&self) -> f64 {
        if self.max_entries == 0 {
            0.0
        } else {
            (self.total_entries as f64 / self.max_entries as f64) * 100.0
        }
    }

    /// Check if cache is nearly full
    pub fn is_nearly_full(&self) -> bool {
        self.utilization() > 90.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_cache_entry_creation() {
        let content = json!({"spec_version": "1.0", "id": "test"});
        let path = PathBuf::from("test.yaml");
        let mtime = SystemTime::now();

        let entry = CacheEntry::new(content.clone(), path.clone(), mtime);

        assert_eq!(entry.content, content);
        assert_eq!(entry.file_path, path);
        assert_eq!(entry.version, Some("1.0".to_string()));
    }

    #[test]
    fn test_cache_entry_validity() {
        let content = json!({"id": "test"});
        let path = PathBuf::from("test.yaml");
        let mtime = SystemTime::now();

        let entry = CacheEntry::new(content, path, mtime);

        // Valid with same mtime
        assert!(entry.is_valid(mtime, None));

        // Invalid with newer mtime
        let newer_mtime = mtime + Duration::from_secs(1);
        assert!(!entry.is_valid(newer_mtime, None));

        // Invalid with age limit - wait a bit to ensure the entry is old enough
        std::thread::sleep(Duration::from_millis(2));
        assert!(!entry.is_valid(mtime, Some(Duration::from_millis(1))));
    }

    #[test]
    fn test_schema_cache_operations() -> LoaderResult<()> {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.yaml");
        fs::write(&file_path, "id: test\nversion: 1.0")?;

        let mut cache = SchemaCache::new();
        let content = json!({"id": "test", "version": "1.0"});

        // Initially empty
        assert!(cache.get(&file_path)?.is_none());

        // Put and get
        cache.put(&file_path, content.clone())?;
        let cached = cache.get(&file_path)?;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().content, content);

        // Contains check
        assert!(cache.contains(&file_path)?);

        // Remove
        assert!(cache.remove(&file_path)?);
        assert!(cache.get(&file_path)?.is_none());

        Ok(())
    }

    #[test]
    fn test_cache_lru_eviction() -> LoaderResult<()> {
        let dir = tempdir().unwrap();
        let mut cache = SchemaCache::with_config(CacheConfig {
            max_entries: 2,
            max_age: None,
            enabled: true,
        });

        // Create test files
        let file1 = dir.path().join("test1.yaml");
        let file2 = dir.path().join("test2.yaml");
        let file3 = dir.path().join("test3.yaml");

        for file in [&file1, &file2, &file3] {
            fs::write(file, "id: test")?;
        }

        // Fill cache to capacity
        cache.put(&file1, json!({"id": "test1"}))?;
        cache.put(&file2, json!({"id": "test2"}))?;

        // Both should be cached
        assert!(cache.contains(&file1)?);
        assert!(cache.contains(&file2)?);

        // Adding third should evict first (LRU)
        cache.put(&file3, json!({"id": "test3"}))?;
        assert!(!cache.contains(&file1)?);
        assert!(cache.contains(&file2)?);
        assert!(cache.contains(&file3)?);

        Ok(())
    }

    #[test]
    fn test_cache_stats() {
        let cache = SchemaCache::new();
        let stats = cache.stats();

        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.max_entries, 1000);
        assert!(stats.enabled);
        assert_eq!(stats.utilization(), 0.0);
        assert!(!stats.is_nearly_full());
    }

    #[test]
    fn test_disabled_cache() -> LoaderResult<()> {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.yaml");
        fs::write(&file_path, "id: test")?;

        let mut cache = SchemaCache::with_config(CacheConfig {
            max_entries: 100,
            max_age: None,
            enabled: false,
        });

        let content = json!({"id": "test"});

        // Operations should succeed but do nothing
        cache.put(&file_path, content.clone())?;
        assert!(cache.get(&file_path)?.is_none());
        assert!(!cache.contains(&file_path)?);

        Ok(())
    }
}