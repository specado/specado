//! HTTP request builder for provider API requests
//!
//! Constructs HTTP requests from ProviderSpec endpoint configurations

use std::collections::HashMap;
use reqwest::{Method, Url};
use serde_json::Value;
use crate::types::{EndpointConfig, ProviderSpec, ModelSpec};
use crate::Result;

/// Builder for constructing HTTP requests from provider specifications
#[derive(Debug, Clone)]
pub struct RequestBuilder {
    base_url: String,
    headers: HashMap<String, String>,
}

impl RequestBuilder {
    /// Create a new RequestBuilder from a ProviderSpec
    pub fn new(provider_spec: &ProviderSpec) -> Self {
        Self {
            base_url: provider_spec.provider.base_url.clone(),
            headers: provider_spec.provider.headers.clone(),
        }
    }

    /// Build a request for a specific endpoint
    pub fn build_request(
        &self,
        endpoint: &EndpointConfig,
        _model: &ModelSpec,
        body: Option<Value>,
    ) -> Result<reqwest::Request> {
        // Construct full URL
        let url = self.build_url(endpoint)?;
        
        // Determine HTTP method
        let method = self.parse_method(&endpoint.method)?;
        
        // Build base request
        let mut request_builder = reqwest::Client::new()
            .request(method, url);
        
        // Add headers from provider spec
        for (key, value) in &self.headers {
            let expanded_value = self.expand_env_vars(value)?;
            request_builder = request_builder.header(key, expanded_value);
        }
        
        // Add endpoint-specific headers
        if let Some(headers) = &endpoint.headers {
            for (key, value) in headers {
                let expanded_value = self.expand_env_vars(value)?;
                request_builder = request_builder.header(key, expanded_value);
            }
        }
        
        // Set Content-Type for JSON bodies
        if body.is_some() {
            request_builder = request_builder.header("Content-Type", "application/json");
        }
        
        // Add query parameters
        if let Some(query) = &endpoint.query {
            for (key, value) in query {
                request_builder = request_builder.query(&[(key, value)]);
            }
        }
        
        // Add body if present
        if let Some(body) = body {
            request_builder = request_builder.json(&body);
        }
        
        // Build and return the request
        request_builder.build()
            .map_err(|e| crate::Error::HttpRequest {
                message: format!("Failed to build request: {}", e),
                source: Some(Box::new(e)),
            })
    }
    
    /// Build the full URL from base URL and endpoint path
    fn build_url(&self, endpoint: &EndpointConfig) -> Result<Url> {
        let base = Url::parse(&self.base_url)
            .map_err(|e| crate::Error::HttpRequest {
                message: format!("Invalid base URL: {}", self.base_url),
                source: Some(Box::new(e)),
            })?;
        
        base.join(&endpoint.path)
            .map_err(|e| crate::Error::HttpRequest {
                message: format!("Failed to join path: {}", endpoint.path),
                source: Some(Box::new(e)),
            })
    }
    
    /// Parse HTTP method from string
    fn parse_method(&self, method_str: &str) -> Result<Method> {
        match method_str.to_uppercase().as_str() {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "DELETE" => Ok(Method::DELETE),
            "PATCH" => Ok(Method::PATCH),
            "HEAD" => Ok(Method::HEAD),
            _ => Err(crate::Error::HttpRequest {
                message: format!("Unsupported HTTP method: {}", method_str),
                source: None,
            }),
        }
    }
    
    /// Expand environment variables in the format ${ENV:VAR_NAME}
    pub fn expand_env_vars(&self, value: &str) -> Result<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ProviderInfo;
    
    fn create_test_provider_spec() -> ProviderSpec {
        ProviderSpec {
            spec_version: "1.0.0".to_string(),
            provider: ProviderInfo {
                name: "test-provider".to_string(),
                base_url: "https://api.example.com".to_string(),
                headers: HashMap::from([
                    ("Authorization".to_string(), "Bearer ${ENV:TEST_API_KEY}".to_string()),
                ]),
            },
            models: vec![],
        }
    }
    
    fn create_test_endpoint() -> EndpointConfig {
        EndpointConfig {
            method: "POST".to_string(),
            path: "/v1/chat/completions".to_string(),
            protocol: "http".to_string(),
            query: Some(HashMap::from([
                ("stream".to_string(), "false".to_string()),
            ])),
            headers: Some(HashMap::from([
                ("X-Custom-Header".to_string(), "test-value".to_string()),
            ])),
        }
    }
    
    #[test]
    fn test_request_builder_creation() {
        let spec = create_test_provider_spec();
        let builder = RequestBuilder::new(&spec);
        
        assert_eq!(builder.base_url, "https://api.example.com");
        assert!(builder.headers.contains_key("Authorization"));
    }
    
    #[test]
    fn test_method_parsing() {
        let spec = create_test_provider_spec();
        let builder = RequestBuilder::new(&spec);
        
        assert_eq!(builder.parse_method("GET").unwrap(), Method::GET);
        assert_eq!(builder.parse_method("post").unwrap(), Method::POST);
        assert_eq!(builder.parse_method("PUT").unwrap(), Method::PUT);
        
        assert!(builder.parse_method("INVALID").is_err());
    }
    
    #[test]
    fn test_url_building() {
        let spec = create_test_provider_spec();
        let builder = RequestBuilder::new(&spec);
        let endpoint = create_test_endpoint();
        
        let url = builder.build_url(&endpoint).unwrap();
        assert_eq!(url.as_str(), "https://api.example.com/v1/chat/completions");
    }
    
    #[test]
    fn test_env_var_expansion() {
        std::env::set_var("TEST_VAR", "test_value");
        
        let spec = create_test_provider_spec();
        let builder = RequestBuilder::new(&spec);
        
        let result = builder.expand_env_vars("Bearer ${ENV:TEST_VAR}").unwrap();
        assert_eq!(result, "Bearer test_value");
        
        // Test missing env var
        let result = builder.expand_env_vars("${ENV:MISSING_VAR}");
        assert!(result.is_err());
        
        std::env::remove_var("TEST_VAR");
    }
}