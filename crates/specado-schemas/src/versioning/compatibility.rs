//! Version compatibility checking
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::versioning::version::{SchemaVersion, VersionRange};
use std::collections::HashMap;

/// Compatibility checking mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityMode {
    /// Strict compatibility - exact version match
    Strict,
    /// Forward compatible - newer minor/patch versions allowed
    Forward,
    /// Backward compatible - older minor/patch versions allowed
    Backward,
    /// Flexible - any version in the same major version
    Flexible,
}

/// Version compatibility checker
#[derive(Debug, Clone)]
pub struct CompatibilityChecker {
    mode: CompatibilityMode,
    supported_ranges: HashMap<String, VersionRange>,
}

impl CompatibilityChecker {
    /// Create a new compatibility checker
    pub fn new(mode: CompatibilityMode) -> Self {
        Self {
            mode,
            supported_ranges: HashMap::new(),
        }
    }

    /// Add a supported version range for a specific schema type
    pub fn add_supported_range(&mut self, schema_type: &str, range: VersionRange) {
        self.supported_ranges.insert(schema_type.to_string(), range);
    }

    /// Check if two versions are compatible
    pub fn is_compatible(&self, spec_version: &SchemaVersion, engine_version: &SchemaVersion) -> bool {
        match self.mode {
            CompatibilityMode::Strict => spec_version == engine_version,
            CompatibilityMode::Forward => {
                // Engine can be newer than spec (forward compatible)
                spec_version.major == engine_version.major &&
                (spec_version.minor < engine_version.minor ||
                 (spec_version.minor == engine_version.minor && spec_version.patch <= engine_version.patch))
            }
            CompatibilityMode::Backward => {
                // Engine can be older than spec (backward compatible)
                spec_version.major == engine_version.major &&
                (spec_version.minor > engine_version.minor ||
                 (spec_version.minor == engine_version.minor && spec_version.patch >= engine_version.patch))
            }
            CompatibilityMode::Flexible => {
                // Same major version is compatible
                spec_version.major == engine_version.major
            }
        }
    }

    /// Check if a version is supported for a specific schema type
    pub fn is_supported(&self, schema_type: &str, version: &SchemaVersion) -> bool {
        if let Some(range) = self.supported_ranges.get(schema_type) {
            range.matches(version)
        } else {
            // If no specific range is defined, use general compatibility
            true
        }
    }

    /// Get compatibility warnings for a version pair
    pub fn get_warnings(&self, spec_version: &SchemaVersion, engine_version: &SchemaVersion) -> Vec<String> {
        let mut warnings = Vec::new();

        // Major version mismatch
        if spec_version.major != engine_version.major {
            warnings.push(format!(
                "Major version mismatch: spec uses v{}, engine supports v{}",
                spec_version.major, engine_version.major
            ));
        }

        // Pre-release warning
        if spec_version.is_pre_release() {
            warnings.push("Using pre-release version - may contain unstable features".to_string());
        }

        // Minor version ahead warning
        if spec_version.major == engine_version.major && spec_version.minor > engine_version.minor {
            warnings.push(format!(
                "Spec uses newer minor version ({}) than engine supports ({})",
                spec_version.minor, engine_version.minor
            ));
        }

        warnings
    }

    /// Determine if an upgrade is breaking
    pub fn is_breaking_change(from: &SchemaVersion, to: &SchemaVersion) -> bool {
        // Major version changes are always breaking
        if from.major != to.major {
            return true;
        }

        // For 0.x versions, minor changes are breaking
        if from.major == 0 && from.minor != to.minor {
            return true;
        }

        false
    }

    /// Get the compatibility mode
    pub fn mode(&self) -> CompatibilityMode {
        self.mode
    }
}

impl Default for CompatibilityChecker {
    fn default() -> Self {
        Self::new(CompatibilityMode::Forward)
    }
}

/// Check compatibility between schema versions and engine versions
pub fn check_compatibility(
    schema_version: &str,
    engine_version: &str,
    mode: CompatibilityMode,
) -> Result<bool, String> {
    let schema_ver = SchemaVersion::parse(schema_version)
        .map_err(|e| format!("Invalid schema version: {}", e))?;
    let engine_ver = SchemaVersion::parse(engine_version)
        .map_err(|e| format!("Invalid engine version: {}", e))?;

    let checker = CompatibilityChecker::new(mode);
    Ok(checker.is_compatible(&schema_ver, &engine_ver))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strict_compatibility() {
        let checker = CompatibilityChecker::new(CompatibilityMode::Strict);
        let v1 = SchemaVersion::new(1, 2, 3);
        let v2 = SchemaVersion::new(1, 2, 3);
        let v3 = SchemaVersion::new(1, 2, 4);

        assert!(checker.is_compatible(&v1, &v2));
        assert!(!checker.is_compatible(&v1, &v3));
    }

    #[test]
    fn test_forward_compatibility() {
        let checker = CompatibilityChecker::new(CompatibilityMode::Forward);
        let spec = SchemaVersion::new(1, 2, 0);
        let engine_newer = SchemaVersion::new(1, 3, 0);
        let engine_older = SchemaVersion::new(1, 1, 0);
        let engine_different_major = SchemaVersion::new(2, 0, 0);

        assert!(checker.is_compatible(&spec, &engine_newer));
        assert!(!checker.is_compatible(&spec, &engine_older));
        assert!(!checker.is_compatible(&spec, &engine_different_major));
    }

    #[test]
    fn test_breaking_changes() {
        let v1_0_0 = SchemaVersion::new(1, 0, 0);
        let v1_1_0 = SchemaVersion::new(1, 1, 0);
        let v2_0_0 = SchemaVersion::new(2, 0, 0);
        let v0_1_0 = SchemaVersion::new(0, 1, 0);
        let v0_2_0 = SchemaVersion::new(0, 2, 0);

        assert!(!CompatibilityChecker::is_breaking_change(&v1_0_0, &v1_1_0));
        assert!(CompatibilityChecker::is_breaking_change(&v1_0_0, &v2_0_0));
        assert!(CompatibilityChecker::is_breaking_change(&v0_1_0, &v0_2_0));
    }

    #[test]
    fn test_compatibility_warnings() {
        let checker = CompatibilityChecker::default();
        let spec = SchemaVersion::new(2, 3, 0);
        let engine = SchemaVersion::new(1, 5, 0);

        let warnings = checker.get_warnings(&spec, &engine);
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("Major version mismatch"));
    }

    #[test]
    fn test_supported_ranges() {
        let mut checker = CompatibilityChecker::default();
        checker.add_supported_range("prompt-spec", VersionRange::parse("^1.0.0").unwrap());

        let v1_0_0 = SchemaVersion::new(1, 0, 0);
        let v1_5_0 = SchemaVersion::new(1, 5, 0);
        let v2_0_0 = SchemaVersion::new(2, 0, 0);

        assert!(checker.is_supported("prompt-spec", &v1_0_0));
        assert!(checker.is_supported("prompt-spec", &v1_5_0));
        assert!(!checker.is_supported("prompt-spec", &v2_0_0));
    }
}