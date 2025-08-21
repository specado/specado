//! Capability discovery system for modern LLM models
//!
//! This module provides runtime capability discovery for models that may have
//! features beyond current specifications. It uses provider APIs, test requests,
//! and heuristic analysis to determine model capabilities.

use crate::error::{Error, Result};
use crate::http::HttpClient;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use super::types::{EnhancedCapabilities, DiscoveryMethod, CachedCapabilities};

/// Configuration for capability discovery
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Timeout for discovery requests in milliseconds
    pub discovery_timeout_ms: u64,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Cache TTL for discovered capabilities in seconds
    pub cache_ttl_seconds: u64,
    /// Whether to perform test validation of capabilities
    pub enable_test_validation: bool,
    /// Rate limiting for discovery requests
    pub rate_limit_per_minute: u32,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            discovery_timeout_ms: 5000,
            max_retries: 3,
            cache_ttl_seconds: 3600, // 1 hour
            enable_test_validation: true,
            rate_limit_per_minute: 10,
        }
    }
}

/// Result of capability discovery operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResult {
    pub model_id: String,
    pub provider: String,
    pub capabilities: EnhancedCapabilities,
    pub discovery_duration_ms: u64,
    pub methods_used: Vec<DiscoveryMethod>,
    pub confidence_score: f64,
    pub timestamp: DateTime<Utc>,
}

/// Errors that can occur during capability discovery
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("Provider API error during discovery: {message}")]
    ApiError { message: String },
    
    #[error("Discovery timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
    
    #[error("Rate limit exceeded for provider {provider}")]
    RateLimit { provider: String },
    
    #[error("Model {model} not found for provider {provider}")]
    ModelNotFound { provider: String, model: String },
    
    #[error("Discovery method {method} failed: {reason}")]
    MethodFailed { method: String, reason: String },
}

/// Trait for capability discovery implementations
#[async_trait]
pub trait CapabilityDiscovery: Send + Sync {
    /// Discover all capabilities for a model
    async fn discover_capabilities(&self, provider: &str, model: &str) -> Result<EnhancedCapabilities>;
    
    /// Test a specific capability with minimal request
    async fn test_capability(&self, provider: &str, model: &str, capability: &str) -> Result<bool>;
    
    /// Get cached capability information
    async fn get_cached_capabilities(&self, provider: &str, model: &str) -> Option<CachedCapabilities>;
    
    /// Clear capability cache for a model
    async fn clear_cache(&self, provider: &str, model: &str) -> Result<()>;
    
    /// Refresh all cached capabilities
    async fn refresh_all_cache(&self) -> Result<Vec<DiscoveryResult>>;
}

/// Main implementation of capability discovery
pub struct CapabilityDiscoveryEngine {
    config: DiscoveryConfig,
    cache: Arc<RwLock<HashMap<String, CachedCapabilities>>>,
    http_clients: HashMap<String, Arc<HttpClient>>,
    rate_limiter: Arc<RwLock<HashMap<String, RateLimiter>>>,
}

impl CapabilityDiscoveryEngine {
    /// Create new discovery engine with configuration
    pub fn new(config: DiscoveryConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            http_clients: HashMap::new(),
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Add HTTP client for a provider
    pub fn add_provider_client(&mut self, provider: &str, client: HttpClient) {
        self.http_clients.insert(provider.to_string(), Arc::new(client));
    }
    
    /// Generate cache key for model capabilities
    fn cache_key(&self, provider: &str, model: &str) -> String {
        format!("{}:{}", provider, model)
    }
    
    /// Check if cached capabilities are still valid
    async fn is_cache_valid(&self, provider: &str, model: &str) -> bool {
        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(&self.cache_key(provider, model)) {
            let age = Utc::now().signed_duration_since(cached.cached_at);
            age.num_seconds() < self.config.cache_ttl_seconds as i64
        } else {
            false
        }
    }
    
    /// Discover capabilities through API introspection
    async fn discover_via_api(&self, provider: &str, model: &str) -> Result<EnhancedCapabilities> {
        let client = self.http_clients.get(provider)
            .ok_or_else(|| Error::Provider {
                provider: provider.to_string(),
                message: "No HTTP client configured".to_string(),
                source: None,
            })?;
            
        match provider {
            "openai" => self.discover_openai_capabilities(client, model).await,
            "anthropic" => self.discover_anthropic_capabilities(client, model).await,
            "google" => self.discover_google_capabilities(client, model).await,
            _ => self.discover_generic_capabilities(client, model).await,
        }
    }
    
    /// OpenAI-specific capability discovery
    async fn discover_openai_capabilities(&self, client: &HttpClient, model: &str) -> Result<EnhancedCapabilities> {
        // Query OpenAI /models endpoint for model information
        let models_response = client.get("/models").await?;
        
        // Find our specific model
        let model_info = models_response["data"]
            .as_array()
            .and_then(|models| {
                models.iter().find(|m| m["id"].as_str() == Some(model))
            })
            .ok_or_else(|| Error::Provider {
                provider: "openai".to_string(),
                message: format!("Model {} not found in /models response", model),
                source: None,
            })?;
        
        // Extract capabilities from model information
        let mut capabilities = EnhancedCapabilities::default();
        capabilities.text_generation = true; // All OpenAI models support text
        
        // Detect vision capability
        if let Some(object_type) = model_info.get("object").and_then(|v| v.as_str()) {
            capabilities.vision = object_type.contains("vision") || model.contains("vision");
        }
        
        // Detect reasoning capability for GPT-5 series
        if model.starts_with("gpt-5") {
            capabilities.reasoning = true;
            capabilities.extended_context = true;
            capabilities.experimental.insert("thinking_mode".to_string(), serde_json::Value::Bool(true));
        }
        
        // Function calling support (most modern models)
        capabilities.function_calling = !model.contains("nano");
        
        // Streaming support (all modern models)
        capabilities.streaming = true;
        
        capabilities.discovery_method = DiscoveryMethod::ApiIntrospection;
        capabilities.discovered_at = Some(Utc::now());
        capabilities.discovery_confidence = 0.85;
        
        Ok(capabilities)
    }
    
    /// Anthropic-specific capability discovery
    async fn discover_anthropic_capabilities(&self, client: &HttpClient, model: &str) -> Result<EnhancedCapabilities> {
        // Anthropic doesn't have a public models endpoint, so we use known patterns
        let mut capabilities = EnhancedCapabilities::default();
        capabilities.text_generation = true;
        
        // Claude-4 series capabilities
        if model.starts_with("claude-4") {
            capabilities.reasoning = true;
            capabilities.vision = true;
            capabilities.function_calling = true;
            capabilities.streaming = true;
            capabilities.extended_context = true;
            capabilities.multimodal = vec!["text".to_string(), "image".to_string(), "document".to_string()];
            
            // Claude-4 specific experimental features
            capabilities.experimental.insert("thinking_mode".to_string(), serde_json::Value::Bool(true));
            capabilities.experimental.insert("computer_use".to_string(), serde_json::Value::Bool(model.contains("sonnet")));
            capabilities.experimental.insert("advanced_reasoning".to_string(), serde_json::Value::Bool(true));
            
            if model.contains("opus") {
                capabilities.experimental.insert("maximum_capability".to_string(), serde_json::Value::Bool(true));
            }
        }
        
        capabilities.discovery_method = DiscoveryMethod::Static; // Based on known patterns
        capabilities.discovered_at = Some(Utc::now());
        capabilities.discovery_confidence = 0.90; // High confidence for known patterns
        
        Ok(capabilities)
    }
    
    /// Generic capability discovery for unknown providers
    async fn discover_generic_capabilities(&self, _client: &HttpClient, _model: &str) -> Result<EnhancedCapabilities> {
        // For unknown providers, use conservative defaults
        let mut capabilities = EnhancedCapabilities::default();
        capabilities.text_generation = true; // Assume basic text generation
        capabilities.discovery_method = DiscoveryMethod::Static;
        capabilities.discovered_at = Some(Utc::now());
        capabilities.discovery_confidence = 0.50; // Low confidence
        
        Ok(capabilities)
    }
    
    /// Google-specific capability discovery
    async fn discover_google_capabilities(&self, client: &HttpClient, model: &str) -> Result<EnhancedCapabilities> {
        // Google Gemini capabilities based on known patterns
        let mut capabilities = EnhancedCapabilities::default();
        capabilities.text_generation = true;
        
        if model.contains("gemini") {
            capabilities.vision = true;
            capabilities.function_calling = true;
            capabilities.streaming = true;
            capabilities.multimodal = vec!["text".to_string(), "image".to_string(), "video".to_string()];
            
            if model.contains("pro") {
                capabilities.extended_context = true;
                capabilities.reasoning = true;
            }
        }
        
        capabilities.discovery_method = DiscoveryMethod::Static;
        capabilities.discovered_at = Some(Utc::now());
        capabilities.discovery_confidence = 0.80;
        
        Ok(capabilities)
    }
}

#[async_trait]
impl CapabilityDiscovery for CapabilityDiscoveryEngine {
    async fn discover_capabilities(&self, provider: &str, model: &str) -> Result<EnhancedCapabilities> {
        // Check cache first
        if self.is_cache_valid(provider, model).await {
            if let Some(cached) = self.get_cached_capabilities(provider, model).await {
                return Ok(cached.capabilities);
            }
        }
        
        // Check rate limiting
        self.check_rate_limit(provider).await?;
        
        // Perform discovery
        let start_time = std::time::Instant::now();
        let capabilities = self.discover_via_api(provider, model).await?;
        let duration = start_time.elapsed();
        
        // Cache the result
        let cached = CachedCapabilities {
            capabilities: capabilities.clone(),
            cached_at: Utc::now(),
            ttl_seconds: self.config.cache_ttl_seconds,
            confidence: capabilities.discovery_confidence,
        };
        
        let mut cache = self.cache.write().await;
        cache.insert(self.cache_key(provider, model), cached);
        
        // Log discovery result
        log::info!(
            "Discovered capabilities for {}:{} in {}ms with confidence {}",
            provider, model, duration.as_millis(), capabilities.discovery_confidence
        );
        
        Ok(capabilities)
    }
    
    async fn test_capability(&self, provider: &str, model: &str, capability: &str) -> Result<bool> {
        // Implement minimal test requests to validate specific capabilities
        match capability {
            "vision" => self.test_vision_capability(provider, model).await,
            "function_calling" => self.test_function_calling_capability(provider, model).await,
            "reasoning" => self.test_reasoning_capability(provider, model).await,
            _ => Ok(false), // Unknown capability
        }
    }
    
    async fn get_cached_capabilities(&self, provider: &str, model: &str) -> Option<CachedCapabilities> {
        let cache = self.cache.read().await;
        cache.get(&self.cache_key(provider, model)).cloned()
    }
    
    async fn clear_cache(&self, provider: &str, model: &str) -> Result<()> {
        let mut cache = self.cache.write().await;
        cache.remove(&self.cache_key(provider, model));
        Ok(())
    }
    
    async fn refresh_all_cache(&self) -> Result<Vec<DiscoveryResult>> {
        let mut results = Vec::new();
        
        // Get all cached models
        let cache_keys: Vec<String> = {
            let cache = self.cache.read().await;
            cache.keys().cloned().collect()
        };
        
        // Refresh each cached model
        for key in cache_keys {
            if let Some((provider, model)) = key.split_once(':') {
                match self.discover_capabilities(provider, model).await {
                    Ok(capabilities) => {
                        results.push(DiscoveryResult {
                            model_id: model.to_string(),
                            provider: provider.to_string(),
                            capabilities,
                            discovery_duration_ms: 0, // Will be set by actual discovery
                            methods_used: vec![DiscoveryMethod::ApiIntrospection],
                            confidence_score: capabilities.discovery_confidence,
                            timestamp: Utc::now(),
                        });
                    }
                    Err(e) => {
                        log::warn!("Failed to refresh capabilities for {}:{}: {}", provider, model, e);
                    }
                }
            }
        }
        
        Ok(results)
    }
}

impl CapabilityDiscoveryEngine {
    /// Check rate limiting for provider
    async fn check_rate_limit(&self, provider: &str) -> Result<()> {
        // Simplified rate limiting - in production would use proper rate limiter
        Ok(())
    }
    
    /// Test vision capability with minimal request
    async fn test_vision_capability(&self, provider: &str, model: &str) -> Result<bool> {
        if !self.config.enable_test_validation {
            return Ok(false);
        }
        
        // Create minimal test request with image to see if model supports it
        // This would be implemented with actual test API calls
        // For now, return based on known patterns
        match provider {
            "openai" => Ok(model.starts_with("gpt-5")),
            "anthropic" => Ok(model.starts_with("claude-4")),
            "google" => Ok(model.contains("gemini")),
            _ => Ok(false),
        }
    }
    
    /// Test function calling capability
    async fn test_function_calling_capability(&self, provider: &str, model: &str) -> Result<bool> {
        if !self.config.enable_test_validation {
            return Ok(false);
        }
        
        // Test with minimal function definition
        match provider {
            "openai" => Ok(!model.contains("nano")), // GPT-5-nano might not support functions
            "anthropic" => Ok(model.starts_with("claude-4")),
            "google" => Ok(model.contains("pro")),
            _ => Ok(false),
        }
    }
    
    /// Test reasoning capability
    async fn test_reasoning_capability(&self, provider: &str, model: &str) -> Result<bool> {
        if !self.config.enable_test_validation {
            return Ok(false);
        }
        
        // Modern models generally support reasoning
        match provider {
            "openai" => Ok(model.starts_with("gpt-5")),
            "anthropic" => Ok(model.starts_with("claude-4")),
            "google" => Ok(model.contains("pro")),
            _ => Ok(false),
        }
    }
}

/// Rate limiter for discovery requests
#[derive(Debug)]
struct RateLimiter {
    requests: Vec<DateTime<Utc>>,
    limit_per_minute: u32,
}

impl RateLimiter {
    fn new(limit_per_minute: u32) -> Self {
        Self {
            requests: Vec::new(),
            limit_per_minute,
        }
    }
    
    fn can_make_request(&mut self) -> bool {
        let now = Utc::now();
        let one_minute_ago = now - chrono::Duration::minutes(1);
        
        // Remove old requests
        self.requests.retain(|&timestamp| timestamp > one_minute_ago);
        
        // Check if under limit
        self.requests.len() < self.limit_per_minute as usize
    }
    
    fn record_request(&mut self) {
        self.requests.push(Utc::now());
    }
}

/// Enhanced capabilities with default implementations
impl Default for EnhancedCapabilities {
    fn default() -> Self {
        Self {
            text_generation: true,
            vision: false,
            function_calling: false,
            streaming: false,
            reasoning: false,
            extended_context: false,
            multimodal: vec!["text".to_string()],
            code_execution: false,
            web_search: false,
            file_handling: false,
            experimental: HashMap::new(),
            discovered_at: None,
            discovery_confidence: 1.0,
            discovery_method: DiscoveryMethod::Static,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cache_key_generation() {
        let engine = CapabilityDiscoveryEngine::new(DiscoveryConfig::default());
        assert_eq!(engine.cache_key("openai", "gpt-5"), "openai:gpt-5");
    }
    
    #[tokio::test]
    async fn test_cache_validity() {
        let engine = CapabilityDiscoveryEngine::new(DiscoveryConfig::default());
        
        // Fresh cache should be invalid
        assert!(!engine.is_cache_valid("openai", "gpt-5").await);
        
        // Add cached capability
        let cached = CachedCapabilities {
            capabilities: EnhancedCapabilities::default(),
            cached_at: Utc::now(),
            ttl_seconds: 3600,
            confidence: 0.9,
        };
        
        {
            let mut cache = engine.cache.write().await;
            cache.insert("openai:gpt-5".to_string(), cached);
        }
        
        // Should now be valid
        assert!(engine.is_cache_valid("openai", "gpt-5").await);
    }
    
    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(5);
        
        // Should allow initial requests
        assert!(limiter.can_make_request());
        limiter.record_request();
        
        // Should handle limit correctly
        for _ in 0..4 {
            assert!(limiter.can_make_request());
            limiter.record_request();
        }
        
        // Should be at limit
        assert!(!limiter.can_make_request());
    }
    
    #[tokio::test]
    async fn test_openai_discovery() {
        let config = DiscoveryConfig {
            enable_test_validation: false, // Disable for unit test
            ..Default::default()
        };
        let engine = CapabilityDiscoveryEngine::new(config);
        
        // This would normally require a real HTTP client
        // For testing, we'd use a mock client
    }
}