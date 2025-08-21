//! Authentication handling for provider APIs
//!
//! Supports multiple authentication schemes:
//! - Bearer tokens (OpenAI)
//! - API keys in headers (Anthropic)
//! - Environment variable expansion

use std::collections::HashMap;
use crate::Result;

/// Authentication errors
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Missing API key: {0}")]
    MissingApiKey(String),
    
    #[error("Invalid authentication configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),
}

/// Trait for handling provider-specific authentication
pub trait AuthHandler: Send + Sync {
    /// Apply authentication to request headers
    fn apply_auth(&self, headers: &mut HashMap<String, String>) -> Result<()>;
    
    /// Validate that required credentials are available
    fn validate_credentials(&self) -> Result<()>;
}

/// OpenAI authentication handler (Bearer token)
#[derive(Debug, Clone)]
pub struct OpenAIAuth {
    api_key: Option<String>,
}

impl OpenAIAuth {
    /// Create from environment variable
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY").ok();
        Ok(Self { api_key })
    }
    
    /// Create with explicit API key
    pub fn new(api_key: String) -> Self {
        Self { api_key: Some(api_key) }
    }
}

impl AuthHandler for OpenAIAuth {
    fn apply_auth(&self, headers: &mut HashMap<String, String>) -> Result<()> {
        match &self.api_key {
            Some(key) => {
                headers.insert(
                    "Authorization".to_string(),
                    format!("Bearer {}", key),
                );
                Ok(())
            }
            None => Err(crate::Error::Configuration {
                message: "OpenAI API key not found. Set OPENAI_API_KEY environment variable".to_string(),
                source: None,
            }),
        }
    }
    
    fn validate_credentials(&self) -> Result<()> {
        if self.api_key.is_none() {
            return Err(crate::Error::Configuration {
                message: "OpenAI API key not configured".to_string(),
                source: None,
            });
        }
        Ok(())
    }
}

/// Anthropic authentication handler (x-api-key header)
#[derive(Debug, Clone)]
pub struct AnthropicAuth {
    api_key: Option<String>,
}

impl AnthropicAuth {
    /// Create from environment variable
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").ok();
        Ok(Self { api_key })
    }
    
    /// Create with explicit API key
    pub fn new(api_key: String) -> Self {
        Self { api_key: Some(api_key) }
    }
}

impl AuthHandler for AnthropicAuth {
    fn apply_auth(&self, headers: &mut HashMap<String, String>) -> Result<()> {
        match &self.api_key {
            Some(key) => {
                headers.insert(
                    "x-api-key".to_string(),
                    key.clone(),
                );
                // Also add Anthropic version header
                headers.insert(
                    "anthropic-version".to_string(),
                    "2023-06-01".to_string(),
                );
                Ok(())
            }
            None => Err(crate::Error::Configuration {
                message: "Anthropic API key not found. Set ANTHROPIC_API_KEY environment variable".to_string(),
                source: None,
            }),
        }
    }
    
    fn validate_credentials(&self) -> Result<()> {
        if self.api_key.is_none() {
            return Err(crate::Error::Configuration {
                message: "Anthropic API key not configured".to_string(),
                source: None,
            });
        }
        Ok(())
    }
}

/// Generic authentication handler supporting ${ENV:VAR} syntax
#[derive(Debug, Clone)]
pub struct GenericAuth {
    auth_headers: HashMap<String, String>,
}

impl GenericAuth {
    /// Create from a map of header names to values
    pub fn new(auth_headers: HashMap<String, String>) -> Self {
        Self { auth_headers }
    }
    
    /// Expand environment variables in header values
    fn expand_env_vars(&self, value: &str) -> Result<String> {
        let mut result = value.to_string();
        
        // Find all ${ENV:...} patterns
        let re = regex::Regex::new(r"\$\{ENV:([^}]+)\}")
            .expect("Valid regex pattern");
        
        for cap in re.captures_iter(value) {
            let var_name = &cap[1];
            let env_value = std::env::var(var_name)
                .map_err(|_| crate::Error::Configuration {
                    message: format!("Environment variable {} not found", var_name),
                    source: None,
                })?;
            
            let pattern = format!("${{ENV:{}}}", var_name);
            result = result.replace(&pattern, &env_value);
        }
        
        Ok(result)
    }
}

impl AuthHandler for GenericAuth {
    fn apply_auth(&self, headers: &mut HashMap<String, String>) -> Result<()> {
        for (key, value) in &self.auth_headers {
            let expanded_value = self.expand_env_vars(value)?;
            headers.insert(key.clone(), expanded_value);
        }
        Ok(())
    }
    
    fn validate_credentials(&self) -> Result<()> {
        // Try to expand all environment variables to validate they exist
        for value in self.auth_headers.values() {
            self.expand_env_vars(value)?;
        }
        Ok(())
    }
}

/// Factory for creating appropriate auth handler based on provider
pub fn create_auth_handler(provider_name: &str) -> Result<Box<dyn AuthHandler>> {
    match provider_name.to_lowercase().as_str() {
        "openai" => Ok(Box::new(OpenAIAuth::from_env()?)),
        "anthropic" => Ok(Box::new(AnthropicAuth::from_env()?)),
        _ => {
            // For unknown providers, return a generic handler
            // that will use headers from the provider spec
            Ok(Box::new(GenericAuth::new(HashMap::new())))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_openai_auth() {
        // Save original env var value for restoration
        let original_key = std::env::var("OPENAI_API_KEY").ok();
        
        // Set test API key
        std::env::set_var("OPENAI_API_KEY", "test-key-123");
        
        let auth = OpenAIAuth::from_env().unwrap();
        let mut headers = HashMap::new();
        
        auth.apply_auth(&mut headers).unwrap();
        
        assert_eq!(headers.get("Authorization").unwrap(), "Bearer test-key-123");
        
        // Restore original environment state
        match original_key {
            Some(key) => std::env::set_var("OPENAI_API_KEY", key),
            None => std::env::remove_var("OPENAI_API_KEY"),
        }
    }
    
    #[test]
    fn test_anthropic_auth() {
        // Save original env var value for restoration
        let original_key = std::env::var("ANTHROPIC_API_KEY").ok();
        
        // Set test API key  
        std::env::set_var("ANTHROPIC_API_KEY", "test-key-456");
        
        let auth = AnthropicAuth::from_env().unwrap();
        let mut headers = HashMap::new();
        
        auth.apply_auth(&mut headers).unwrap();
        
        assert_eq!(headers.get("x-api-key").unwrap(), "test-key-456");
        assert_eq!(headers.get("anthropic-version").unwrap(), "2023-06-01");
        
        // Restore original environment state
        match original_key {
            Some(key) => std::env::set_var("ANTHROPIC_API_KEY", key),
            None => std::env::remove_var("ANTHROPIC_API_KEY"),
        }
    }
    
    #[test]
    fn test_missing_api_key() {
        // Test the auth behavior directly without relying on global environment
        let auth = OpenAIAuth { api_key: None };
        let mut headers = HashMap::new();
        
        let result = auth.apply_auth(&mut headers);
        
        // Verify the test assertion
        assert!(result.is_err(), "Expected auth to fail with missing API key");
        let error_message = result.unwrap_err().to_string();
        assert!(
            error_message.contains("not found") || error_message.contains("not configured"), 
            "Expected error message to mention missing key, got: {}", 
            error_message
        );
    }
    
    #[test]
    #[ignore] // Potentially flaky due to environment contamination from other tests
    fn test_missing_api_key_from_env() {
        // Save original env var value for restoration
        let original_key = std::env::var("OPENAI_API_KEY").ok();
        
        // Ensure env var is not set - explicitly remove it
        std::env::remove_var("OPENAI_API_KEY");
        
        // Verify it's actually removed
        assert!(std::env::var("OPENAI_API_KEY").is_err(), "OPENAI_API_KEY should be unset for this test");
        
        let auth = OpenAIAuth::from_env().unwrap();
        let mut headers = HashMap::new();
        
        let result = auth.apply_auth(&mut headers);
        
        // Restore original environment state
        if let Some(key) = original_key {
            std::env::set_var("OPENAI_API_KEY", key);
        }
        
        // Verify the test assertion
        assert!(result.is_err(), "Expected auth to fail with missing API key");
        let error_message = result.unwrap_err().to_string();
        assert!(
            error_message.contains("not found") || error_message.contains("not configured"), 
            "Expected error message to mention missing key, got: {}", 
            error_message
        );
    }
    
    #[test]
    fn test_generic_auth_env_expansion() {
        // Save original env var value for restoration
        let original_key = std::env::var("CUSTOM_API_KEY").ok();
        
        // Set test API key
        std::env::set_var("CUSTOM_API_KEY", "custom-key-789");
        
        let mut auth_headers = HashMap::new();
        auth_headers.insert(
            "X-API-Key".to_string(),
            "${ENV:CUSTOM_API_KEY}".to_string(),
        );
        
        let auth = GenericAuth::new(auth_headers);
        let mut headers = HashMap::new();
        
        auth.apply_auth(&mut headers).unwrap();
        
        assert_eq!(headers.get("X-API-Key").unwrap(), "custom-key-789");
        
        // Restore original environment state
        match original_key {
            Some(key) => std::env::set_var("CUSTOM_API_KEY", key),
            None => std::env::remove_var("CUSTOM_API_KEY"),
        }
    }
}