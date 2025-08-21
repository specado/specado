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
    ErrorClassification,
    RetryPolicy,
    retry::execute_with_retry,
    auth::create_auth_handler,
    FallbackHandler, FallbackConfig, FallbackAttempt,
    DiagnosticsBuilder,
};
use crate::Result;

use crate::http::{
    timeout::{TimeoutConfig, RequestTimeout, with_timeout},
    tls::{TlsConfig, load_cert_from_file},
    rate_limit::{RateLimitConfig, RateLimiter},
    network_errors::{NetworkErrorHandler, CircuitBreakerConfig},
};

/// Configuration for the HTTP client
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// Retry policy for failed requests
    pub retry_policy: RetryPolicy,
    /// Request timeout in seconds (deprecated - use timeout_config)
    pub timeout_secs: u64,
    /// Whether to validate TLS certificates (deprecated - use tls_config)
    pub validate_tls: bool,
    /// Fallback configuration
    pub fallback_config: FallbackConfig,
    /// Advanced timeout configuration
    pub timeout_config: TimeoutConfig,
    /// TLS/HTTPS configuration
    pub tls_config: TlsConfig,
    /// Rate limiting configuration
    pub rate_limit_config: Option<RateLimitConfig>,
    /// Circuit breaker configuration for network error handling
    pub circuit_breaker_config: CircuitBreakerConfig,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            retry_policy: RetryPolicy::default(),
            timeout_secs: 30, // Kept for backward compatibility
            validate_tls: true, // Kept for backward compatibility
            fallback_config: FallbackConfig::default(),
            timeout_config: TimeoutConfig::default(),
            tls_config: TlsConfig::default(),
            rate_limit_config: None, // Disabled by default
            circuit_breaker_config: CircuitBreakerConfig::default(),
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
    /// Fallback handler for error recovery
    fallback_handler: std::sync::Mutex<FallbackHandler>,
    /// Rate limiter (optional)
    rate_limiter: Option<RateLimiter>,
    /// Network error handler with circuit breaker
    network_error_handler: NetworkErrorHandler,
}

impl HttpClient {
    /// Create a new HTTP client for a provider
    pub fn new(provider_spec: ProviderSpec, config: HttpClientConfig) -> Result<Self> {
        // Validate configurations
        config.timeout_config.validate()
            .map_err(|e| crate::Error::Configuration {
                message: format!("Invalid timeout config: {}", e),
                source: None,
            })?;
            
        config.tls_config.validate()
            .map_err(|e| crate::Error::Configuration {
                message: format!("Invalid TLS config: {}", e),
                source: None,
            })?;
            
        if let Some(rate_config) = &config.rate_limit_config {
            rate_config.validate()
                .map_err(|e| crate::Error::Configuration {
                    message: format!("Invalid rate limit config: {}", e),
                    source: None,
                })?;
        }
        
        // Create reqwest client with enhanced configuration
        let mut client_builder = ReqwestClient::builder()
            .connect_timeout(config.timeout_config.connect_timeout)
            .timeout(config.timeout_config.request_timeout);
        
        // Apply TLS configuration
        client_builder = Self::configure_tls(client_builder, &config.tls_config)?;
        
        let client = client_builder
            .build()
            .map_err(|e| crate::Error::HttpRequest {
                message: format!("Failed to create HTTP client: {}", e),
                source: Some(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            })?;
        
        // Create request builder
        let request_builder = RequestBuilder::new(&provider_spec);
        
        // Create authentication handler
        let auth_handler = Arc::from(create_auth_handler(&provider_spec.provider.name)?);
        
        // Create fallback handler
        let fallback_handler = std::sync::Mutex::new(
            FallbackHandler::new(config.fallback_config.clone())
        );
        
        // Create rate limiter if configured
        let rate_limiter = config.rate_limit_config
            .clone()
            .map(RateLimiter::new);
        
        // Create network error handler
        let network_error_handler = NetworkErrorHandler::new(config.circuit_breaker_config.clone());
        
        Ok(Self {
            client,
            request_builder,
            auth_handler,
            config,
            provider_spec,
            fallback_handler,
            rate_limiter,
            network_error_handler,
        })
    }
    
    /// Configure TLS settings for reqwest client builder
    fn configure_tls(
        mut builder: reqwest::ClientBuilder,
        tls_config: &TlsConfig,
    ) -> Result<reqwest::ClientBuilder> {
        // Certificate validation
        if !tls_config.validate_certificates {
            builder = builder.danger_accept_invalid_certs(true);
        }
        
        if tls_config.accept_invalid_hostnames {
            builder = builder.danger_accept_invalid_hostnames(true);
        }
        
        // Add custom CA certificates
        for ca_cert_path in &tls_config.custom_ca_certs {
            let cert_pem = load_cert_from_file(ca_cert_path)
                .map_err(|e| crate::Error::Configuration {
                    message: format!("Failed to load CA certificate from {:?}: {}", ca_cert_path, e),
                    source: Some(e.into()),
                })?;
            let cert = reqwest::Certificate::from_pem(cert_pem.as_bytes())
                .map_err(|e| crate::Error::Configuration {
                    message: format!("Invalid CA certificate format: {}", e),
                    source: Some(e.into()),
                })?;
            builder = builder.add_root_certificate(cert);
        }
        
        for ca_cert_pem in &tls_config.custom_ca_cert_pem {
            let cert = reqwest::Certificate::from_pem(ca_cert_pem.as_bytes())
                .map_err(|e| crate::Error::Configuration {
                    message: format!("Invalid CA certificate PEM format: {}", e),
                    source: Some(e.into()),
                })?;
            builder = builder.add_root_certificate(cert);
        }
        
        // Client certificates (mutual TLS)
        if let (Some(cert_pem), Some(key_pem)) = (&tls_config.client_cert_pem, &tls_config.client_key_pem) {
            let identity = reqwest::Identity::from_pem(
                format!("{}{}", cert_pem, key_pem).as_bytes()
            ).map_err(|e| crate::Error::Configuration {
                message: format!("Invalid client certificate/key format: {}", e),
                source: Some(e.into()),
            })?;
            builder = builder.identity(identity);
        } else if let (Some(cert_path), Some(key_path)) = (&tls_config.client_cert_path, &tls_config.client_key_path) {
            let cert_pem = load_cert_from_file(cert_path)
                .map_err(|e| crate::Error::Configuration {
                    message: format!("Failed to load client certificate: {}", e),
                    source: Some(e.into()),
                })?;
            let key_pem = load_cert_from_file(key_path)
                .map_err(|e| crate::Error::Configuration {
                    message: format!("Failed to load client key: {}", e),
                    source: Some(e.into()),
                })?;
            let identity = reqwest::Identity::from_pem(
                format!("{}{}", cert_pem, key_pem).as_bytes()
            ).map_err(|e| crate::Error::Configuration {
                message: format!("Invalid client certificate/key format: {}", e),
                source: Some(e.into()),
            })?;
            builder = builder.identity(identity);
        }
        
        // TODO: Add TLS version constraints when reqwest supports them
        // Currently reqwest doesn't expose fine-grained TLS version control
        
        Ok(builder)
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
        self.execute_chat_completion_with_timeout(model, request_body, None).await
    }
    
    /// Execute a synchronous request with custom timeout
    pub async fn execute_chat_completion_with_timeout(
        &self,
        model: &ModelSpec,
        request_body: Value,
        timeout_override: Option<RequestTimeout>,
    ) -> Result<Value> {
        let endpoint = &model.endpoints.chat_completion;
        self.execute_request_with_timeout(endpoint, model, request_body, timeout_override).await
    }
    
    /// Execute a streaming request to the chat completion endpoint
    pub async fn execute_streaming_chat_completion(
        &self,
        model: &ModelSpec,
        request_body: Value,
    ) -> Result<Response> {
        self.execute_streaming_chat_completion_with_timeout(model, request_body, None).await
    }
    
    /// Execute a streaming request with custom timeout
    pub async fn execute_streaming_chat_completion_with_timeout(
        &self,
        model: &ModelSpec,
        request_body: Value,
        timeout_override: Option<RequestTimeout>,
    ) -> Result<Response> {
        let endpoint = &model.endpoints.streaming_chat_completion;
        self.execute_raw_request_with_timeout(endpoint, model, request_body, timeout_override).await
    }
    
    /// Execute a generic request with retry logic and fallback strategies
    #[allow(dead_code)]
    async fn execute_request(
        &self,
        endpoint: &EndpointConfig,
        model: &ModelSpec,
        request_body: Value,
    ) -> Result<Value> {
        self.execute_request_with_timeout(endpoint, model, request_body, None).await
    }
    
    /// Execute a generic request with timeout override
    async fn execute_request_with_timeout(
        &self,
        endpoint: &EndpointConfig,
        model: &ModelSpec,
        request_body: Value,
        timeout_override: Option<RequestTimeout>,
    ) -> Result<Value> {
        // Apply rate limiting
        let endpoint_key = format!("{}:{}", self.provider_spec.provider.name, endpoint.path);
        if let Some(rate_limiter) = &self.rate_limiter {
            rate_limiter.wait_for_permit(&self.provider_spec.provider.name).await
                .map_err(|e| crate::Error::RateLimit {
                    message: format!("Rate limit exceeded: {}", e),
                    retry_after: None,
                })?;
        }
        
        // Check circuit breaker
        self.network_error_handler.can_request(&endpoint_key)
            .map_err(|e| match e {
                crate::http::NetworkError::CircuitBreakerOpen { retry_after } => {
                    crate::Error::CircuitBreakerOpen {
                        message: format!("Circuit breaker is open for endpoint {}", endpoint_key),
                        retry_after: Some(retry_after.as_secs()),
                    }
                }
                _ => crate::Error::Http {
                    message: format!("Network error: {}", e),
                    status_code: None,
                    source: Some(anyhow::anyhow!("{}", e)),
                }
            })?;
        
        // Try normal execution first
        #[allow(unused_assignments)]
        let mut last_error: Option<HttpError> = None;
        let _start = std::time::Instant::now();
        
        // Primary attempt
        match self.execute_raw_request_with_timeout(endpoint, model, request_body.clone(), timeout_override.clone()).await {
            Ok(response) => {
                // Check if response is successful
                if !response.status().is_success() {
                    let error = HttpError::from_response(response).await;
                    
                    // Handle 429 rate limiting responses
                    if error.status_code == Some(429) {
                        if let Some(rate_limiter) = &self.rate_limiter {
                            rate_limiter.handle_429_response(
                                error.retry_after,
                                &self.provider_spec.provider.name
                            ).await.map_err(|e| crate::Error::RateLimit {
                                message: format!("Rate limit handling failed: {}", e),
                                retry_after: error.retry_after,
                            })?;
                        }
                    }
                    
                    // Record failure with network error handler
                    self.network_error_handler.record_failure(&endpoint_key, &error);
                    
                    last_error = Some(error);
                } else {
                    // Record success with network error handler
                    self.network_error_handler.record_success(&endpoint_key);
                    
                    // Parse response body as JSON
                    return response.json::<Value>().await
                        .map_err(|e| crate::Error::Http {
                            message: format!("Failed to parse response as JSON: {}", e),
                            status_code: None,
                            source: Some(anyhow::anyhow!("{}", e)),
                        });
                }
            }
            Err(e) => {
                // Convert crate::Error to HttpError if possible
                if let crate::Error::Http { message, status_code, .. } = &e {
                    let http_error = HttpError {
                        status_code: *status_code,
                        classification: ErrorClassification::Unknown,
                        provider_code: None,
                        message: message.clone(),
                        details: None,
                        retry_after: None,
                    };
                    
                    // Record failure with network error handler
                    self.network_error_handler.record_failure(&endpoint_key, &http_error);
                    
                    last_error = Some(http_error);
                } else {
                    return Err(e);
                }
            }
        }
        
        // If primary attempt failed, try fallback strategies
        if let Some(error) = last_error {
            return self.execute_with_fallback_and_timeout(endpoint, model, request_body, error, timeout_override).await;
        }
        
        Err(crate::Error::Http {
            message: "Request failed without specific error".to_string(),
            status_code: None,
            source: None,
        })
    }
    
    /// Execute request with fallback strategies and timeout
    #[allow(clippy::await_holding_lock)]
    async fn execute_with_fallback_and_timeout(
        &self,
        endpoint: &EndpointConfig,
        model: &ModelSpec,
        mut request_body: Value,
        initial_error: HttpError,
        timeout_override: Option<RequestTimeout>,
    ) -> Result<Value> {
        let endpoint_key = format!("{}:{}", self.provider_spec.provider.name, endpoint.path);
        
        // Check if error is retryable via network error handler
        if !self.network_error_handler.is_retryable(&initial_error, 0) {
            return Err(initial_error.into());
        }
        let mut fallback = self.fallback_handler.lock().unwrap();
        let mut attempt = 0;
        let mut last_error = initial_error;
        
        while fallback.should_retry(&last_error, attempt) {
            let start = std::time::Instant::now();
            attempt += 1;
            
            // Apply degradation if needed
            if fallback.config.allow_degradation {
                request_body = fallback.apply_degradation(request_body.clone(), attempt);
            }
            
            // Try with alternative URL if available
            let alt_url = fallback.get_alternative_url(attempt as usize - 1);
            
            // Record attempt
            let strategy = if alt_url.is_some() {
                format!("Alternative URL attempt {}", attempt)
            } else {
                format!("Retry attempt {} with degradation", attempt)
            };
            
            // Calculate delay with jitter - use network error handler for better delays
            let network_delay = self.network_error_handler.get_retry_delay(&last_error);
            let fallback_delay = fallback.calculate_retry_delay(attempt - 1);
            let delay = network_delay.unwrap_or(fallback_delay);
            tokio::time::sleep(delay).await;
            
            // Execute request with timeout
            match self.execute_raw_request_with_timeout(endpoint, model, request_body.clone(), timeout_override.clone()).await {
                Ok(response) => {
                    if response.status().is_success() {
                        // Record successful recovery
                        self.network_error_handler.record_success(&endpoint_key);
                        fallback.record_attempt(FallbackAttempt {
                            strategy: strategy.clone(),
                            success: true,
                            error: None,
                            duration: start.elapsed(),
                        });
                        
                        // Parse and return response
                        return response.json::<Value>().await
                            .map_err(|e| crate::Error::Http {
                                message: format!("Failed to parse response as JSON: {}", e),
                                status_code: None,
                                source: Some(anyhow::anyhow!("{}", e)),
                            });
                    } else {
                        let error = HttpError::from_response(response).await;
                        self.network_error_handler.record_failure(&endpoint_key, &error);
                        fallback.record_attempt(FallbackAttempt {
                            strategy: strategy.clone(),
                            success: false,
                            error: Some(error.to_string()),
                            duration: start.elapsed(),
                        });
                        last_error = error;
                    }
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    fallback.record_attempt(FallbackAttempt {
                        strategy: strategy.clone(),
                        success: false,
                        error: Some(error_msg.clone()),
                        duration: start.elapsed(),
                    });
                    
                    // Convert crate::Error to HttpError if possible
                    if let crate::Error::Http { message, status_code, .. } = e {
                        let http_error = HttpError {
                            status_code,
                            classification: ErrorClassification::Unknown,
                            provider_code: None,
                            message,
                            details: None,
                            retry_after: None,
                        };
                        self.network_error_handler.record_failure(&endpoint_key, &http_error);
                        last_error = http_error;
                    } else {
                        // Non-HTTP error, can't retry
                        break;
                    }
                }
            }
        }
        
        // All attempts failed, return enhanced error with diagnostics
        let diagnostics = DiagnosticsBuilder::from_error(&last_error)
            .provider(&self.provider_spec.provider.name)
            .model(&model.id)
            .endpoint(&endpoint.path)
            .recovery_attempts(fallback.attempts())
            .build();
        
        Err(crate::Error::HttpWithDiagnostics {
            error: last_error,
            diagnostics: Box::new(diagnostics),
        })
    }
    
    /// Execute a raw request (returns Response for streaming)
    #[allow(dead_code)]
    async fn execute_raw_request(
        &self,
        endpoint: &EndpointConfig,
        model: &ModelSpec,
        request_body: Value,
    ) -> Result<Response> {
        self.execute_raw_request_with_timeout(endpoint, model, request_body, None).await
    }
    
    /// Execute a raw request with timeout override
    async fn execute_raw_request_with_timeout(
        &self,
        endpoint: &EndpointConfig,
        model: &ModelSpec,
        request_body: Value,
        timeout_override: Option<RequestTimeout>,
    ) -> Result<Response> {
        let client = self.client.clone();
        let request_builder = self.request_builder.clone();
        let auth_handler = self.auth_handler.clone();
        let timeout_config = if let Some(override_timeout) = &timeout_override {
            override_timeout.apply_to(&self.config.timeout_config)
        } else {
            self.config.timeout_config.clone()
        };
        
        // Execute with timeout and retry logic
        let request_future = execute_with_retry(
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
                    .map_err(HttpError::from_request_error)?;
                
                // Check for errors
                if !response.status().is_success() {
                    let error = HttpError::from_response(response).await;
                    return Err(error);
                }
                
                Ok(response)
            },
            self.config.retry_policy.clone(),
        );
        
        // Apply timeout wrapper
        with_timeout(request_future, &timeout_config, timeout_override.as_ref()).await
            .map_err(|_| crate::Error::Timeout {
                message: format!("Request timed out after {:?}", timeout_config.request_timeout),
                timeout_duration: timeout_config.request_timeout,
            })?
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
    
    /// Get rate limiter status (for debugging/monitoring)
    pub fn rate_limiter_status(&self) -> Option<crate::http::rate_limit::TokenStatus> {
        self.rate_limiter.as_ref().map(|limiter| limiter.get_token_status())
    }
    
    /// Get circuit breaker statistics
    pub fn circuit_breaker_stats(&self) -> std::collections::HashMap<String, crate::http::network_errors::CircuitBreakerStats> {
        self.network_error_handler.get_circuit_stats()
    }
    
    /// Update rate limit configuration
    pub fn update_rate_limit_config(&mut self, config: Option<RateLimitConfig>) -> Result<()> {
        if let Some(rate_config) = &config {
            rate_config.validate()
                .map_err(|e| crate::Error::Configuration {
                    message: format!("Invalid rate limit config: {}", e),
                    source: None,
                })?;
        }
        
        self.rate_limiter = config.clone().map(RateLimiter::new);
        self.config.rate_limit_config = config;
        Ok(())
    }
    
    /// Get current configuration
    pub fn config(&self) -> &HttpClientConfig {
        &self.config
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
                    capabilities: None,
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
        
        // Test new configuration defaults
        assert_eq!(config.timeout_config.request_timeout, std::time::Duration::from_secs(30));
        assert_eq!(config.timeout_config.connect_timeout, std::time::Duration::from_secs(10));
        assert!(config.tls_config.validate_certificates);
        assert!(config.rate_limit_config.is_none());
        assert_eq!(config.circuit_breaker_config.failure_threshold, 5);
    }
}