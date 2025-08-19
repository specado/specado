//! TLS/HTTPS configuration for HTTP clients
//!
//! Provides comprehensive TLS configuration including custom CA certificates,
//! certificate validation options, and development settings

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// TLS/HTTPS configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Whether to validate TLS certificates
    pub validate_certificates: bool,
    /// Custom CA certificate paths
    pub custom_ca_certs: Vec<PathBuf>,
    /// Custom CA certificate content (PEM format)
    pub custom_ca_cert_pem: Vec<String>,
    /// Client certificate path (for mutual TLS)
    pub client_cert_path: Option<PathBuf>,
    /// Client private key path (for mutual TLS)
    pub client_key_path: Option<PathBuf>,
    /// Client certificate and key content (PEM format)
    pub client_cert_pem: Option<String>,
    /// Client private key content (PEM format)
    pub client_key_pem: Option<String>,
    /// Minimum TLS version to accept
    pub min_tls_version: TlsVersion,
    /// Maximum TLS version to use
    pub max_tls_version: Option<TlsVersion>,
    /// Allowed cipher suites (empty = use defaults)
    pub allowed_ciphers: Vec<String>,
    /// SNI (Server Name Indication) hostname override
    pub sni_hostname: Option<String>,
    /// Whether to accept invalid hostnames (dangerous!)
    pub accept_invalid_hostnames: bool,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            validate_certificates: true,
            custom_ca_certs: Vec::new(),
            custom_ca_cert_pem: Vec::new(),
            client_cert_path: None,
            client_key_path: None,
            client_cert_pem: None,
            client_key_pem: None,
            min_tls_version: TlsVersion::TLS1_2,
            max_tls_version: None, // Use latest available
            allowed_ciphers: Vec::new(), // Use defaults
            sni_hostname: None,
            accept_invalid_hostnames: false,
        }
    }
}

/// TLS protocol versions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TlsVersion {
    /// TLS 1.0 (deprecated, not recommended)
    TLS1_0,
    /// TLS 1.1 (deprecated, not recommended)
    TLS1_1,
    /// TLS 1.2 (minimum recommended)
    TLS1_2,
    /// TLS 1.3 (preferred)
    TLS1_3,
}

impl TlsVersion {
    /// Convert to reqwest tls version
    /// Note: Currently not used as reqwest doesn't expose fine-grained TLS version control
    #[allow(dead_code)]
    pub fn to_reqwest_version(&self) -> u16 {
        match self {
            TlsVersion::TLS1_0 => 0x0301,
            TlsVersion::TLS1_1 => 0x0302,
            TlsVersion::TLS1_2 => 0x0303,
            TlsVersion::TLS1_3 => 0x0304,
        }
    }
}

impl TlsConfig {
    /// Create a new TLS configuration with certificate validation
    pub fn secure() -> Self {
        Self::default()
    }
    
    /// Create TLS configuration for development (allows invalid certificates)
    pub fn development() -> Self {
        Self {
            validate_certificates: false,
            accept_invalid_hostnames: true,
            min_tls_version: TlsVersion::TLS1_2,
            ..Self::default()
        }
    }
    
    /// Create TLS configuration for testing (very permissive)
    pub fn testing() -> Self {
        Self {
            validate_certificates: false,
            accept_invalid_hostnames: true,
            min_tls_version: TlsVersion::TLS1_0, // Allow old TLS for testing
            ..Self::default()
        }
    }
    
    /// Add a custom CA certificate from file path
    pub fn with_ca_cert_file(mut self, path: PathBuf) -> Self {
        self.custom_ca_certs.push(path);
        self
    }
    
    /// Add a custom CA certificate from PEM content
    pub fn with_ca_cert_pem(mut self, pem_content: String) -> Self {
        self.custom_ca_cert_pem.push(pem_content);
        self
    }
    
    /// Configure client certificate and key from files
    pub fn with_client_cert_files(mut self, cert_path: PathBuf, key_path: PathBuf) -> Self {
        self.client_cert_path = Some(cert_path);
        self.client_key_path = Some(key_path);
        self
    }
    
    /// Configure client certificate and key from PEM content
    pub fn with_client_cert_pem(mut self, cert_pem: String, key_pem: String) -> Self {
        self.client_cert_pem = Some(cert_pem);
        self.client_key_pem = Some(key_pem);
        self
    }
    
    /// Set minimum TLS version
    pub fn with_min_tls_version(mut self, version: TlsVersion) -> Self {
        self.min_tls_version = version;
        self
    }
    
    /// Set maximum TLS version
    pub fn with_max_tls_version(mut self, version: TlsVersion) -> Self {
        self.max_tls_version = Some(version);
        self
    }
    
    /// Override SNI hostname
    pub fn with_sni_hostname(mut self, hostname: String) -> Self {
        self.sni_hostname = Some(hostname);
        self
    }
    
    /// Validate the TLS configuration
    pub fn validate(&self) -> Result<(), TlsConfigError> {
        // Check that if max version is set, it's >= min version
        if let Some(max_version) = self.max_tls_version {
            if max_version < self.min_tls_version {
                return Err(TlsConfigError::InvalidVersionRange(
                    self.min_tls_version,
                    max_version,
                ));
            }
        }
        
        // Check that client cert and key are both provided or both missing (file path version)
        match (&self.client_cert_path, &self.client_key_path) {
            (Some(_), None) | (None, Some(_)) => {
                return Err(TlsConfigError::IncompleteClientCertFiles);
            }
            _ => {}
        }
        
        // Check that client cert and key are both provided or both missing (PEM version)
        match (&self.client_cert_pem, &self.client_key_pem) {
            (Some(_), None) | (None, Some(_)) => {
                return Err(TlsConfigError::IncompleteClientCertPem);
            }
            _ => {}
        }
        
        // Check for conflicting client cert configurations
        if self.client_cert_path.is_some() && self.client_cert_pem.is_some() {
            return Err(TlsConfigError::ConflictingClientCertConfig);
        }
        
        // Validate CA certificate files exist
        for ca_path in &self.custom_ca_certs {
            if !ca_path.exists() {
                return Err(TlsConfigError::CaCertFileNotFound(ca_path.clone()));
            }
        }
        
        // Validate client certificate files exist
        if let Some(cert_path) = &self.client_cert_path {
            if !cert_path.exists() {
                return Err(TlsConfigError::ClientCertFileNotFound(cert_path.clone()));
            }
        }
        
        if let Some(key_path) = &self.client_key_path {
            if !key_path.exists() {
                return Err(TlsConfigError::ClientKeyFileNotFound(key_path.clone()));
            }
        }
        
        Ok(())
    }
    
    /// Check if this configuration requires client certificates
    pub fn has_client_cert(&self) -> bool {
        self.client_cert_path.is_some() || self.client_cert_pem.is_some()
    }
    
    /// Check if this configuration has custom CA certificates
    pub fn has_custom_ca_certs(&self) -> bool {
        !self.custom_ca_certs.is_empty() || !self.custom_ca_cert_pem.is_empty()
    }
}

/// TLS configuration errors
#[derive(Debug, thiserror::Error)]
pub enum TlsConfigError {
    #[error("Invalid TLS version range: min {0:?} > max {1:?}")]
    InvalidVersionRange(TlsVersion, TlsVersion),
    
    #[error("Incomplete client certificate configuration (files): both cert and key paths must be provided")]
    IncompleteClientCertFiles,
    
    #[error("Incomplete client certificate configuration (PEM): both cert and key content must be provided")]
    IncompleteClientCertPem,
    
    #[error("Conflicting client certificate configuration: cannot specify both file paths and PEM content")]
    ConflictingClientCertConfig,
    
    #[error("CA certificate file not found: {0:?}")]
    CaCertFileNotFound(PathBuf),
    
    #[error("Client certificate file not found: {0:?}")]
    ClientCertFileNotFound(PathBuf),
    
    #[error("Client key file not found: {0:?}")]
    ClientKeyFileNotFound(PathBuf),
}

/// Helper function to load certificate from file
pub fn load_cert_from_file(path: &PathBuf) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

/// Helper function to validate PEM format
pub fn validate_pem_format(pem_content: &str) -> bool {
    pem_content.contains("-----BEGIN") && pem_content.contains("-----END")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_tls_config_default() {
        let config = TlsConfig::default();
        assert!(config.validate_certificates);
        assert_eq!(config.min_tls_version, TlsVersion::TLS1_2);
        assert!(config.custom_ca_certs.is_empty());
        assert!(!config.accept_invalid_hostnames);
    }
    
    #[test]
    fn test_tls_config_presets() {
        let secure = TlsConfig::secure();
        assert!(secure.validate_certificates);
        assert!(!secure.accept_invalid_hostnames);
        
        let dev = TlsConfig::development();
        assert!(!dev.validate_certificates);
        assert!(dev.accept_invalid_hostnames);
        
        let testing = TlsConfig::testing();
        assert!(!testing.validate_certificates);
        assert_eq!(testing.min_tls_version, TlsVersion::TLS1_0);
    }
    
    #[test]
    fn test_tls_config_builder() {
        let config = TlsConfig::secure()
            .with_ca_cert_pem("-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----".to_string())
            .with_min_tls_version(TlsVersion::TLS1_3)
            .with_sni_hostname("example.com".to_string());
        
        assert_eq!(config.custom_ca_cert_pem.len(), 1);
        assert_eq!(config.min_tls_version, TlsVersion::TLS1_3);
        assert_eq!(config.sni_hostname, Some("example.com".to_string()));
    }
    
    #[test]
    fn test_tls_version_ordering() {
        assert!(TlsVersion::TLS1_0 < TlsVersion::TLS1_1);
        assert!(TlsVersion::TLS1_1 < TlsVersion::TLS1_2);
        assert!(TlsVersion::TLS1_2 < TlsVersion::TLS1_3);
    }
    
    #[test]
    fn test_tls_config_validation() {
        // Valid config should pass
        let config = TlsConfig::default();
        assert!(config.validate().is_ok());
        
        // Invalid version range should fail
        let mut config = TlsConfig::default();
        config.min_tls_version = TlsVersion::TLS1_3;
        config.max_tls_version = Some(TlsVersion::TLS1_2);
        assert!(config.validate().is_err());
        
        // Incomplete client cert files should fail
        let mut config = TlsConfig::default();
        config.client_cert_path = Some(PathBuf::from("cert.pem"));
        assert!(config.validate().is_err());
        
        // Incomplete client cert PEM should fail
        let mut config = TlsConfig::default();
        config.client_cert_pem = Some("cert content".to_string());
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_pem_validation() {
        let valid_pem = "-----BEGIN CERTIFICATE-----\nMIIC...\n-----END CERTIFICATE-----";
        assert!(validate_pem_format(valid_pem));
        
        let invalid_pem = "not a pem file";
        assert!(!validate_pem_format(invalid_pem));
    }
    
    #[test]
    fn test_config_predicates() {
        let mut config = TlsConfig::default();
        assert!(!config.has_client_cert());
        assert!(!config.has_custom_ca_certs());
        
        config.client_cert_pem = Some("cert".to_string());
        config.client_key_pem = Some("key".to_string());
        assert!(config.has_client_cert());
        
        config.custom_ca_cert_pem.push("ca cert".to_string());
        assert!(config.has_custom_ca_certs());
    }
    
    #[test]
    fn test_load_cert_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let cert_content = "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----";
        std::io::Write::write_all(&mut temp_file, cert_content.as_bytes()).unwrap();
        
        let loaded = load_cert_from_file(&temp_file.path().to_path_buf()).unwrap();
        assert_eq!(loaded, cert_content);
    }
}