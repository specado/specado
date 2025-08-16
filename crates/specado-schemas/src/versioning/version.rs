//! Semantic version parsing and validation
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

/// Semantic version representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
    pub build_metadata: Option<String>,
}

impl SchemaVersion {
    /// Create a new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
            build_metadata: None,
        }
    }

    /// Parse a version string
    pub fn parse(version_str: &str) -> Result<Self, VersionError> {
        // Remove 'v' prefix if present
        let version_str = version_str.strip_prefix('v').unwrap_or(version_str);
        
        // Split on '+' for build metadata
        let (version_part, build_metadata) = if let Some(plus_pos) = version_str.find('+') {
            (
                &version_str[..plus_pos],
                Some(version_str[plus_pos + 1..].to_string()),
            )
        } else {
            (version_str, None)
        };
        
        // Split on '-' for pre-release
        let (version_part, pre_release) = if let Some(dash_pos) = version_part.find('-') {
            (
                &version_part[..dash_pos],
                Some(version_part[dash_pos + 1..].to_string()),
            )
        } else {
            (version_part, None)
        };
        
        // Parse the version numbers
        let parts: Vec<&str> = version_part.split('.').collect();
        if parts.len() != 3 {
            return Err(VersionError::InvalidFormat(format!(
                "Expected format X.Y.Z, got: {}",
                version_str
            )));
        }
        
        let major = parts[0]
            .parse()
            .map_err(|_| VersionError::InvalidFormat(format!("Invalid major version: {}", parts[0])))?;
        let minor = parts[1]
            .parse()
            .map_err(|_| VersionError::InvalidFormat(format!("Invalid minor version: {}", parts[1])))?;
        let patch = parts[2]
            .parse()
            .map_err(|_| VersionError::InvalidFormat(format!("Invalid patch version: {}", parts[2])))?;
        
        Ok(Self {
            major,
            minor,
            patch,
            pre_release,
            build_metadata,
        })
    }

    /// Check if this is a pre-release version
    pub fn is_pre_release(&self) -> bool {
        self.pre_release.is_some()
    }

    /// Check if this version satisfies a version range
    pub fn satisfies(&self, range: &VersionRange) -> bool {
        range.matches(self)
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pre) = self.pre_release {
            write!(f, "-{}", pre)?;
        }
        if let Some(ref build) = self.build_metadata {
            write!(f, "+{}", build)?;
        }
        Ok(())
    }
}

impl FromStr for SchemaVersion {
    type Err = VersionError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl PartialOrd for SchemaVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SchemaVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => match self.patch.cmp(&other.patch) {
                    Ordering::Equal => {
                        // Pre-release versions have lower precedence
                        match (&self.pre_release, &other.pre_release) {
                            (None, None) => Ordering::Equal,
                            (None, Some(_)) => Ordering::Greater,
                            (Some(_), None) => Ordering::Less,
                            (Some(a), Some(b)) => a.cmp(b),
                        }
                    }
                    ord => ord,
                },
                ord => ord,
            },
            ord => ord,
        }
    }
}

/// Version range specification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionRange {
    /// Exact version match
    Exact(SchemaVersion),
    /// Caret range (^X.Y.Z) - compatible with specified version
    Caret(SchemaVersion),
    /// Tilde range (~X.Y.Z) - approximately equivalent
    Tilde(SchemaVersion),
    /// Greater than or equal
    GreaterOrEqual(SchemaVersion),
    /// Less than
    LessThan(SchemaVersion),
    /// Any version
    Any,
}

impl VersionRange {
    /// Parse a version range string
    pub fn parse(range_str: &str) -> Result<Self, VersionError> {
        let range_str = range_str.trim();
        
        if range_str == "*" || range_str.is_empty() {
            return Ok(VersionRange::Any);
        }
        
        if let Some(version_str) = range_str.strip_prefix('^') {
            let version = SchemaVersion::parse(version_str)?;
            return Ok(VersionRange::Caret(version));
        }
        
        if let Some(version_str) = range_str.strip_prefix('~') {
            let version = SchemaVersion::parse(version_str)?;
            return Ok(VersionRange::Tilde(version));
        }
        
        if let Some(version_str) = range_str.strip_prefix(">=") {
            let version = SchemaVersion::parse(version_str)?;
            return Ok(VersionRange::GreaterOrEqual(version));
        }
        
        if let Some(version_str) = range_str.strip_prefix('<') {
            let version = SchemaVersion::parse(version_str)?;
            return Ok(VersionRange::LessThan(version));
        }
        
        // Try to parse as exact version
        let version = SchemaVersion::parse(range_str)?;
        Ok(VersionRange::Exact(version))
    }

    /// Check if a version matches this range
    pub fn matches(&self, version: &SchemaVersion) -> bool {
        match self {
            VersionRange::Any => true,
            VersionRange::Exact(v) => version == v,
            VersionRange::Caret(v) => {
                // Compatible with version v
                // Major must match, minor/patch can be greater
                if v.major == 0 {
                    // 0.x.y versions are special - minor changes are breaking
                    if v.minor == 0 {
                        // 0.0.x - only patch can change
                        version.major == 0 && version.minor == 0 && version.patch >= v.patch
                    } else {
                        // 0.x.y - minor must match, patch can be greater
                        version.major == 0 && version.minor == v.minor && version.patch >= v.patch
                    }
                } else {
                    // Normal semver - major must match, minor/patch can be greater
                    version.major == v.major && 
                    (version.minor > v.minor || 
                     (version.minor == v.minor && version.patch >= v.patch))
                }
            }
            VersionRange::Tilde(v) => {
                // Approximately equivalent - patch level changes allowed
                version.major == v.major && 
                version.minor == v.minor && 
                version.patch >= v.patch
            }
            VersionRange::GreaterOrEqual(v) => version >= v,
            VersionRange::LessThan(v) => version < v,
        }
    }
}

impl fmt::Display for VersionRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionRange::Any => write!(f, "*"),
            VersionRange::Exact(v) => write!(f, "{}", v),
            VersionRange::Caret(v) => write!(f, "^{}", v),
            VersionRange::Tilde(v) => write!(f, "~{}", v),
            VersionRange::GreaterOrEqual(v) => write!(f, ">={}", v),
            VersionRange::LessThan(v) => write!(f, "<{}", v),
        }
    }
}

/// Version parsing error
#[derive(Debug, Clone)]
pub enum VersionError {
    InvalidFormat(String),
    InvalidRange(String),
}

impl fmt::Display for VersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionError::InvalidFormat(msg) => write!(f, "Invalid version format: {}", msg),
            VersionError::InvalidRange(msg) => write!(f, "Invalid version range: {}", msg),
        }
    }
}

impl std::error::Error for VersionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let v = SchemaVersion::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert_eq!(v.pre_release, None);
        assert_eq!(v.build_metadata, None);

        let v = SchemaVersion::parse("v2.0.0-alpha").unwrap();
        assert_eq!(v.major, 2);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
        assert_eq!(v.pre_release, Some("alpha".to_string()));

        let v = SchemaVersion::parse("1.0.0+build123").unwrap();
        assert_eq!(v.build_metadata, Some("build123".to_string()));

        let v = SchemaVersion::parse("3.1.4-beta.2+exp.sha.5114f85").unwrap();
        assert_eq!(v.major, 3);
        assert_eq!(v.minor, 1);
        assert_eq!(v.patch, 4);
        assert_eq!(v.pre_release, Some("beta.2".to_string()));
        assert_eq!(v.build_metadata, Some("exp.sha.5114f85".to_string()));
    }

    #[test]
    fn test_version_comparison() {
        let v1 = SchemaVersion::new(1, 0, 0);
        let v2 = SchemaVersion::new(2, 0, 0);
        let v3 = SchemaVersion::new(1, 1, 0);
        let v4 = SchemaVersion::new(1, 0, 1);

        assert!(v1 < v2);
        assert!(v1 < v3);
        assert!(v1 < v4);
        assert!(v4 < v3);
        assert!(v3 < v2);
    }

    #[test]
    fn test_range_matching() {
        let v1_0_0 = SchemaVersion::new(1, 0, 0);
        let v1_2_3 = SchemaVersion::new(1, 2, 3);
        let v1_2_4 = SchemaVersion::new(1, 2, 4);
        let v1_3_0 = SchemaVersion::new(1, 3, 0);
        let v2_0_0 = SchemaVersion::new(2, 0, 0);

        // Caret range tests
        let range = VersionRange::Caret(v1_2_3.clone());
        assert!(range.matches(&v1_2_3));
        assert!(range.matches(&v1_2_4));
        assert!(range.matches(&v1_3_0));
        assert!(!range.matches(&v2_0_0));
        assert!(!range.matches(&v1_0_0));

        // Tilde range tests
        let range = VersionRange::Tilde(v1_2_3.clone());
        assert!(range.matches(&v1_2_3));
        assert!(range.matches(&v1_2_4));
        assert!(!range.matches(&v1_3_0));
        assert!(!range.matches(&v2_0_0));
    }

    #[test]
    fn test_range_parsing() {
        assert!(matches!(VersionRange::parse("*").unwrap(), VersionRange::Any));
        assert!(matches!(VersionRange::parse("^1.2.3").unwrap(), VersionRange::Caret(_)));
        assert!(matches!(VersionRange::parse("~1.2.3").unwrap(), VersionRange::Tilde(_)));
        assert!(matches!(VersionRange::parse(">=1.0.0").unwrap(), VersionRange::GreaterOrEqual(_)));
        assert!(matches!(VersionRange::parse("<2.0.0").unwrap(), VersionRange::LessThan(_)));
        assert!(matches!(VersionRange::parse("1.2.3").unwrap(), VersionRange::Exact(_)));
    }
}