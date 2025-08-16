//! Schema versioning and compatibility management
//!
//! This module provides comprehensive versioning support including:
//! - Semantic version parsing and validation
//! - Compatibility range checking
//! - Migration hints between versions
//! - Version deprecation warnings
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

pub mod compatibility;
pub mod migration;
pub mod version;

pub use compatibility::{CompatibilityChecker, CompatibilityMode};
pub use migration::{MigrationHint, MigrationRegistry};
pub use version::{SchemaVersion, VersionRange};

/// Check if two schema versions are compatible
pub fn is_compatible(spec_version: &SchemaVersion, engine_version: &SchemaVersion) -> bool {
    let checker = CompatibilityChecker::default();
    checker.is_compatible(spec_version, engine_version)
}

/// Get migration hints between two versions
pub fn get_migration_hints(from: &SchemaVersion, to: &SchemaVersion) -> Vec<MigrationHint> {
    let registry = MigrationRegistry::default();
    registry.get_hints(from, to)
}