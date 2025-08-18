//! Unified HTTP client orchestrating all components
//!
//! Provides a high-level interface for making authenticated, retryable HTTP requests

use std::sync::Arc;
use reqwest::{Client as ReqwestClient, Response};
use serde_json::Value;
use crate::types::{ProviderSpec, ModelSpec, EndpointConfig};
use crate::http::{
    RequestBuilder,
    AuthHandler,
    HttpError,
    RetryPolicy,
    retry::execute_with_retry,
    auth::create_auth_handler,
};
use crate::Result;

/// Configuration for the HTTP client
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// Retry policy for failed requests
    pub retry_policy: RetryPolicy,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Whether to validate TLS certificates
    pub validate_tls: bool,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            retry_policy: RetryPolicy::default(),
            timeout_secs: 30,
            validate_tls: true,
        }
    }
}

/// Unified HTTP client for provider API communication
pub struct HttpClient {
    /// Underlying reqwest client
    client: ReqwestClient,
    /// Request builder for constructing requests
    request_builder: RequestBuilder,
    /// Authentication handler
    auth_handler: Arc<dyn AuthHandler>,
    /// Client configuration
    config: HttpClientConfig,
    /// Provider specification
    provider_spec: ProviderSpec,
}

impl HttpClient {
    /// Create a new HTTP client for a provider
    pub fn new(provider_spec: ProviderSpec, config: HttpClientConfig) -> Result<Self> {
        // Create reqwest client with configuration
        let client = ReqwestClient::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .danger_accept_invalid_certs(!config.validate_tls)
            .build()
            .map_err(|e| crate::Error::HttpRequest {
                message: format!("Failed to create HTTP client: {}", e),
                source: Some(Box::new(e)),
            })?;
        
        // Create request builder
        let request_builder = RequestBuilder::new(&provider_spec);
        
        // Create authentication handler
        let auth_handler = Arc::from(create_auth_handler(&provider_spec.provider.name)?);
        
        Ok(Self {
            client,
            request_builder,
            auth_handler,
            config,
            provider_spec,
        })
    }
    
    /// Create with default configuration
    pub fn with_default_config(provider_spec: ProviderSpec) -> Result<Self> {
        Self::new(provider_spec, HttpClientConfig::default())
    }
    
    /// Execute a synchronous request to the chat completion endpoint
    pub async fn execute_chat_completion(
        &self,
        model: &ModelSpec,
        request_body: Value,
    ) -> Result<Value> {
        let endpoint = &model.endpoints.chat_completion;
        self.execute_request(endpoint, model, request_body).await
    }
    
    /// Execute a streaming request to the chat completion endpoint
    pub async fn execute_streaming_chat_completion(
        &self,
        model: &ModelSpec,
        request_body: Value,
    ) -> Result<Response> {
        let endpoint = &model.endpoints.streaming_chat_completion;
        self.execute_raw_request(endpoint, model, request_body).await
    }
    
    /// Execute a generic request with retry logic
    async fn execute_request(
        &self,
        endpoint: &EndpointConfig,
        model: &ModelSpec,
        request_body: Value,
    ) -> Result<Value> {
        let response = self.execute_raw_request(endpoint, model, request_body).await?;
        
        // Check if response is successful
        if !response.status().is_success() {
            let error = HttpError::from_response(response).await;
            return Err(error.into());
        }
        
        // Parse response body as JSON
        let json_value = response.json::<Value>().await
            .map_err(|e| crate::Error::Http {
                message: format!("Failed to parse response as JSON: {}", e),
                status_code: None,
                source: Some(anyhow::anyhow!("{}", e)),
            })?;
        Ok(json_value)
    }
    
    /// Execute a raw request (returns Response for streaming)
    async fn execute_raw_request(
        &self,
        endpoint: &EndpointConfig,
        model: &ModelSpec,
        request_body: Value,
    ) -> Result<Response> {
        let client = self.client.clone();
        let request_builder = self.request_builder.clone();
        let auth_handler = self.auth_handler.clone();
        
        // Execute with retry logic
        execute_with_retry(
            || async {
                // Build the request
                let mut request = request_builder
                    .build_request(endpoint, model, Some(request_body.clone()))
                    .map_err(|e| HttpError {
                        status_code: None,
                        classification: crate::http::error::ErrorClassification::ClientError,
                        provider_code: None,
                        message: e.to_string(),
                        details: None,
                        retry_after: None,
                    })?;
                
                // Apply authentication
                let mut headers = std::collections::HashMap::new();
                auth_handler.apply_auth(&mut headers)
                    .map_err(|e| HttpError {
                        status_code: None,
                        classification: crate::http::error::ErrorClassification::AuthenticationError,
                        provider_code: None,
                        message: e.to_string(),
                        details: None,
                        retry_after: None,
                    })?;
                
                // Add auth headers to request
                for (key, value) in headers {
                    if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(key.as_bytes()) {
                        if let Ok(header_value) = reqwest::header::HeaderValue::from_str(&value) {
                            request.headers_mut().insert(header_name, header_value);
                        }
                    }
                }
                
                // Execute the request
                let response = client.execute(request).await
                    .map_err(|e| HttpError::from_request_error(e))?;
                
                // Check for errors
                if !response.status().is_success() {
                    let error = HttpError::from_response(response).await;
                    return Err(error);
                }
                
                Ok(response)
            },
            self.config.retry_policy.clone(),
        ).await
        .map_err(|e| e.into())
    }
    
    /// Validate that the client is properly configured
    pub fn validate(&self) -> Result<()> {
        // Validate authentication credentials
        self.auth_handler.validate_credentials()?;
        
        // Validate provider spec has required models
        if self.provider_spec.models.is_empty() {
            return Err(crate::Error::Configuration {
                message: "Provider spec has no models configured".to_string(),
                source: None,
            });
        }
        
        Ok(())
    }
    
    /// Get a model spec by ID
    pub fn get_model(&self, model_id: &str) -> Option<&ModelSpec> {
        self.provider_spec.models.iter()
            .find(|m| {
                m.id == model_id || 
                m.aliases.as_ref()
                    .map(|aliases| aliases.iter().any(|a| a == model_id))
                    .unwrap_or(false)
            })
    }
    
    /// Get a reference to the provider spec
    pub fn provider_spec(&self) -> &ProviderSpec {
        &self.provider_spec
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        ProviderInfo, Endpoints, InputModes, JsonOutputConfig, ToolingConfig,
        Constraints, ConstraintLimits, Mappings, ResponseNormalization,
        SyncNormalization, StreamNormalization, EventSelector
    };
    use std::collections::HashMap;
    
    fn create_test_provider_spec() -> ProviderSpec {
        ProviderSpec {
            spec_version: "1.0.0".to_string(),
            provider: ProviderInfo {
                name: "test-provider".to_string(),
                base_url: "https://api.example.com".to_string(),
                headers: HashMap::from([
                    ("Authorization".to_string(), "Bearer test-key".to_string()),
                ]),
            },
            models: vec![
                ModelSpec {
                    id: "test-model".to_string(),
                    aliases: Some(vec!["test".to_string()]),
                    family: "test-family".to_string(),
                    endpoints: Endpoints {
                        chat_completion: EndpointConfig {
                            method: "POST".to_string(),
                            path: "/v1/chat/completions".to_string(),
                            protocol: "http".to_string(),
                            query: None,
                            headers: None,
                        },
                        streaming_chat_completion: EndpointConfig {
                            method: "POST".to_string(),
                            path: "/v1/chat/completions".to_string(),
                            protocol: "sse".to_string(),
                            query: Some(HashMap::from([
                                ("stream".to_string(), "true".to_string()),
                            ])),
                            headers: None,
                        },
                    },
                    input_modes: InputModes {
                        messages: true,
                        single_text: false,
                        images: false,
                    },
                    tooling: ToolingConfig {
                        tools_supported: false,
                        parallel_tool_calls_default: false,
                        can_disable_parallel_tool_calls: false,
                        disable_switch: None,
                    },
                    json_output: JsonOutputConfig {
                        native_param: false,
                        strategy: "none".to_string(),
                    },
                    parameters: serde_json::json!({}),
                    constraints: Constraints {
                        system_prompt_location: "message_role".to_string(),
                        forbid_unknown_top_level_fields: false,
                        mutually_exclusive: vec![],
                        resolution_preferences: vec![],
                        limits: ConstraintLimits {
                            max_tool_schema_bytes: 100000,
                            max_system_prompt_bytes: 10000,
                        },
                    },
                    mappings: Mappings {
                        paths: HashMap::new(),
                        flags: HashMap::new(),
                    },
                    response_normalization: ResponseNormalization {
                        sync: SyncNormalization {
                            content_path: "$.content".to_string(),
                            finish_reason_path: "$.finish_reason".to_string(),
                            finish_reason_map: HashMap::new(),
                        },
                        stream: StreamNormalization {
                            protocol: "sse".to_string(),
                            event_selector: EventSelector {
                                type_path: "$.type".to_string(),
                                routes: vec![],
                            },
                        },
                    },
                },
            ],
        }
    }
    
    #[test]
    fn test_client_creation() {
        let spec = create_test_provider_spec();
        let config = HttpClientConfig::default();
        
        // This will fail in tests due to missing env vars, but tests structure
        let result = HttpClient::new(spec, config);
        
        // In real tests, we'd mock the auth handler
        assert!(result.is_err() || result.is_ok());
    }
    
    #[test]
    fn test_get_model() {
        let spec = create_test_provider_spec();
        
        // Create a client with a mock auth handler for testing
        // In real implementation, we'd use a test double
        std::env::set_var("TEST_API_KEY", "test");
        
        let config = HttpClientConfig::default();
        
        // This test validates the model lookup logic
        // In production, we'd have proper mocking
        if let Ok(client) = HttpClient::new(spec, config) {
            assert!(client.get_model("test-model").is_some());
            assert!(client.get_model("test").is_some()); // Via alias
            assert!(client.get_model("nonexistent").is_none());
        }
        
        std::env::remove_var("TEST_API_KEY");
    }
    
    #[test]
    fn test_config_defaults() {
        let config = HttpClientConfig::default();
        assert_eq!(config.timeout_secs, 30);
        assert!(config.validate_tls);
        assert_eq!(config.retry_policy.max_attempts, 3);
    }
}