//! Enhanced error diagnostics and reporting
//!
//! This module provides detailed error diagnostics with actionable suggestions
//! and clear formatting for user-friendly error messages.

use crate::http::{HttpError, ErrorClassification};
use crate::http::fallback::FallbackAttempt;
use colored::Colorize;
use std::fmt;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Diagnostic information for an error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDiagnostics {
    /// The original error
    pub error: String,
    
    /// Error classification
    pub classification: String,
    
    /// Contextual information
    pub context: ErrorContext,
    
    /// Recovery attempts made
    pub recovery_attempts: Vec<RecoveryAttempt>,
    
    /// Suggested actions for the user
    pub suggested_actions: Vec<String>,
    
    /// Links to documentation or resources
    pub help_links: Vec<HelpLink>,
    
    /// Additional diagnostic data
    pub metadata: HashMap<String, String>,
}

/// Context information about when the error occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub endpoint: Option<String>,
    pub timestamp: String,
    pub request_id: Option<String>,
}

/// Information about a recovery attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAttempt {
    pub strategy: String,
    pub result: String,
    pub duration_ms: u64,
}

/// Help link with description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpLink {
    pub title: String,
    pub url: String,
}

impl ErrorDiagnostics {
    /// Create new diagnostics from an HTTP error
    pub fn from_http_error(error: &HttpError) -> Self {
        let classification = format!("{:?}", error.classification());
        let suggested_actions = Self::suggest_actions_for_error(error);
        let help_links = Self::get_help_links_for_error(error);
        
        Self {
            error: error.to_string(),
            classification,
            context: ErrorContext {
                provider: None,
                model: None,
                endpoint: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
                request_id: None,
            },
            recovery_attempts: Vec::new(),
            suggested_actions,
            help_links,
            metadata: HashMap::new(),
        }
    }
    
    /// Add context information
    pub fn with_context(mut self, provider: &str, model: &str, endpoint: &str) -> Self {
        self.context.provider = Some(provider.to_string());
        self.context.model = Some(model.to_string());
        self.context.endpoint = Some(endpoint.to_string());
        self
    }
    
    /// Add recovery attempts from fallback handler
    pub fn with_recovery_attempts(mut self, attempts: &[FallbackAttempt]) -> Self {
        self.recovery_attempts = attempts.iter().map(|a| RecoveryAttempt {
            strategy: a.strategy.clone(),
            result: if a.success { 
                "Success".to_string() 
            } else { 
                a.error.as_ref().unwrap_or(&"Failed".to_string()).clone()
            },
            duration_ms: a.duration.as_millis() as u64,
        }).collect();
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Suggest actions based on error type
    fn suggest_actions_for_error(error: &HttpError) -> Vec<String> {
        match error.classification() {
            ErrorClassification::AuthenticationError => vec![
                "Check that your API key is correctly set in the environment".to_string(),
                "Verify the API key format (e.g., OpenAI keys start with 'sk-')".to_string(),
                "Ensure the API key has not expired or been revoked".to_string(),
                "Try regenerating a new API key from your provider's dashboard".to_string(),
            ],
            ErrorClassification::RateLimitError => vec![
                "Wait a moment before retrying the request".to_string(),
                "Consider implementing request throttling".to_string(),
                "Check your API usage limits and quotas".to_string(),
                "Upgrade your API plan if you need higher limits".to_string(),
            ],
            ErrorClassification::NetworkError => vec![
                "Check your internet connection".to_string(),
                "Verify the API endpoint URL is correct".to_string(),
                "Check if the provider's API service is operational".to_string(),
                "Try using a different network or VPN if blocked".to_string(),
            ],
            ErrorClassification::ClientError => vec![
                "Review the request format and parameters".to_string(),
                "Check that all required fields are present".to_string(),
                "Verify the model ID is correct and available".to_string(),
                "Ensure request body follows the provider's API schema".to_string(),
            ],
            ErrorClassification::ServerError => vec![
                "The provider's service may be experiencing issues".to_string(),
                "Try again in a few moments".to_string(),
                "Check the provider's status page for outages".to_string(),
                "Consider using an alternative model or provider".to_string(),
            ],
            ErrorClassification::TimeoutError => vec![
                "Request timed out - the server took too long to respond".to_string(),
                "Try increasing timeout values in client configuration".to_string(),
                "Check network connectivity and latency".to_string(),
                "Consider using a different endpoint or region".to_string(),
            ],
            ErrorClassification::TlsError => vec![
                "TLS/SSL handshake failed".to_string(),
                "Check certificate validity and chain".to_string(),
                "Verify TLS configuration and supported versions".to_string(),
                "Ensure system time and date are correct".to_string(),
            ],
            ErrorClassification::DnsError => vec![
                "DNS resolution failed - cannot resolve hostname".to_string(),
                "Check DNS server configuration".to_string(),
                "Verify the API endpoint hostname is correct".to_string(),
                "Try using a different DNS resolver or network".to_string(),
            ],
            ErrorClassification::ConnectionError => vec![
                "Failed to establish connection to the server".to_string(),
                "Check firewall and network proxy settings".to_string(),
                "Verify the server is accepting connections".to_string(),
                "Try connecting from a different network".to_string(),
            ],
            ErrorClassification::CircuitBreakerOpen => vec![
                "Circuit breaker is open - too many recent failures".to_string(),
                "Wait for the circuit breaker recovery period".to_string(),
                "Check server health and recent error patterns".to_string(),
                "Consider using alternative endpoints".to_string(),
            ],
            ErrorClassification::Unknown => vec![
                "An unknown error occurred".to_string(),
                "Check the error details for more information".to_string(),
                "Try the request again or contact support".to_string(),
            ],
        }
    }
    
    /// Get relevant help links
    fn get_help_links_for_error(error: &HttpError) -> Vec<HelpLink> {
        let mut links = Vec::new();
        
        // Add provider-specific links based on error context
        match error.classification() {
            ErrorClassification::AuthenticationError => {
                links.push(HelpLink {
                    title: "OpenAI API Keys".to_string(),
                    url: "https://platform.openai.com/api-keys".to_string(),
                });
                links.push(HelpLink {
                    title: "Anthropic API Keys".to_string(),
                    url: "https://console.anthropic.com/settings/keys".to_string(),
                });
            }
            ErrorClassification::RateLimitError => {
                links.push(HelpLink {
                    title: "OpenAI Rate Limits".to_string(),
                    url: "https://platform.openai.com/docs/guides/rate-limits".to_string(),
                });
            }
            _ => {}
        }
        
        // Always include Specado docs
        links.push(HelpLink {
            title: "Specado Documentation".to_string(),
            url: "https://docs.specado.com/troubleshooting".to_string(),
        });
        
        links
    }
    
    /// Format as a user-friendly error message
    pub fn format_display(&self, use_color: bool) -> String {
        let mut output = String::new();
        
        // Header
        let header = format!("\n{}\n ERROR: {} \n{}\n",
            "═".repeat(50),
            self.classification,
            "═".repeat(50)
        );
        
        if use_color {
            output.push_str(&header.red().bold().to_string());
        } else {
            output.push_str(&header);
        }
        
        // What happened
        output.push_str(&format!("\n{} What happened:\n", 
            if use_color { "✗".red().to_string() } else { "✗".to_string() }
        ));
        output.push_str(&format!("  {}\n", self.error));
        
        // Context
        if self.context.provider.is_some() || self.context.model.is_some() {
            output.push_str(&format!("\n{} Context:\n", 
"📍"
            ));
            
            if let Some(ref provider) = self.context.provider {
                output.push_str(&format!("  • Provider: {}\n", provider));
            }
            if let Some(ref model) = self.context.model {
                output.push_str(&format!("  • Model: {}\n", model));
            }
            if let Some(ref endpoint) = self.context.endpoint {
                output.push_str(&format!("  • Endpoint: {}\n", endpoint));
            }
        }
        
        // Recovery attempts
        if !self.recovery_attempts.is_empty() {
            output.push_str(&format!("\n{} Recovery attempted:\n", 
"🔄"
            ));
            
            for (i, attempt) in self.recovery_attempts.iter().enumerate() {
                let status = if attempt.result == "Success" {
                    if use_color {
                        "✓".green().to_string()
                    } else {
                        "✓".to_string()
                    }
                } else if use_color {
                    "✗".red().to_string()
                } else {
                    "✗".to_string()
                };
                
                output.push_str(&format!("  {} Attempt {}: {} - {} ({}ms)\n",
                    status,
                    i + 1,
                    attempt.strategy,
                    attempt.result,
                    attempt.duration_ms
                ));
            }
        }
        
        // Suggested actions
        if !self.suggested_actions.is_empty() {
            output.push_str(&format!("\n{} Suggested actions:\n", 
"💡"
            ));
            
            for (i, action) in self.suggested_actions.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", i + 1, action));
            }
        }
        
        // Help links
        if !self.help_links.is_empty() {
            output.push_str(&format!("\n{} More information:\n", 
"📚"
            ));
            
            for link in &self.help_links {
                let formatted_link = if use_color {
                    format!("  • {}: {}", link.title, link.url.blue().underline())
                } else {
                    format!("  • {}: {}", link.title, link.url)
                };
                output.push_str(&format!("{}\n", formatted_link));
            }
        }
        
        // Footer
        let footer = format!("\n{}\n", "═".repeat(50));
        if use_color {
            output.push_str(&footer.red().bold().to_string());
        } else {
            output.push_str(&footer);
        }
        
        output
    }
}

impl fmt::Display for ErrorDiagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_display(false))
    }
}

/// Builder for ErrorDiagnostics
pub struct DiagnosticsBuilder {
    diagnostics: ErrorDiagnostics,
}

impl DiagnosticsBuilder {
    /// Create a new builder from an error
    pub fn from_error(error: &HttpError) -> Self {
        Self {
            diagnostics: ErrorDiagnostics::from_http_error(error),
        }
    }
    
    /// Set provider context
    pub fn provider(mut self, provider: &str) -> Self {
        self.diagnostics.context.provider = Some(provider.to_string());
        self
    }
    
    /// Set model context
    pub fn model(mut self, model: &str) -> Self {
        self.diagnostics.context.model = Some(model.to_string());
        self
    }
    
    /// Set endpoint context
    pub fn endpoint(mut self, endpoint: &str) -> Self {
        self.diagnostics.context.endpoint = Some(endpoint.to_string());
        self
    }
    
    /// Set request ID
    pub fn request_id(mut self, id: &str) -> Self {
        self.diagnostics.context.request_id = Some(id.to_string());
        self
    }
    
    /// Add recovery attempts
    pub fn recovery_attempts(mut self, attempts: &[FallbackAttempt]) -> Self {
        self.diagnostics = self.diagnostics.with_recovery_attempts(attempts);
        self
    }
    
    /// Add custom suggested action
    pub fn suggest(mut self, action: &str) -> Self {
        self.diagnostics.suggested_actions.push(action.to_string());
        self
    }
    
    /// Add metadata
    pub fn metadata(mut self, key: &str, value: &str) -> Self {
        self.diagnostics.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Build the diagnostics
    pub fn build(self) -> ErrorDiagnostics {
        self.diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_diagnostics_creation() {
        let error = HttpError {
            status_code: Some(401),
            classification: ErrorClassification::AuthenticationError,
            provider_code: None,
            message: "Invalid API key".to_string(),
            details: None,
            retry_after: None,
        };
        
        let diagnostics = ErrorDiagnostics::from_http_error(&error)
            .with_context("openai", "gpt-4", "https://api.openai.com/v1/chat/completions");
        
        assert_eq!(diagnostics.classification, "AuthenticationError");
        assert_eq!(diagnostics.context.provider, Some("openai".to_string()));
        assert_eq!(diagnostics.context.model, Some("gpt-4".to_string()));
        assert!(!diagnostics.suggested_actions.is_empty());
    }
    
    #[test]
    fn test_diagnostics_builder() {
        let error = HttpError {
            status_code: Some(429),
            classification: ErrorClassification::RateLimitError,
            provider_code: None,
            message: "Rate limit exceeded".to_string(),
            details: None,
            retry_after: Some(60),
        };
        
        let diagnostics = DiagnosticsBuilder::from_error(&error)
            .provider("anthropic")
            .model("claude-3")
            .endpoint("https://api.anthropic.com/v1/messages")
            .request_id("req-123")
            .suggest("Consider implementing request batching")
            .metadata("requests_per_minute", "100")
            .build();
        
        assert_eq!(diagnostics.context.provider, Some("anthropic".to_string()));
        assert_eq!(diagnostics.context.request_id, Some("req-123".to_string()));
        assert!(diagnostics.suggested_actions.contains(&"Consider implementing request batching".to_string()));
        assert_eq!(diagnostics.metadata.get("requests_per_minute"), Some(&"100".to_string()));
    }
    
    #[test]
    fn test_format_display() {
        let error = HttpError {
            status_code: None,
            classification: ErrorClassification::NetworkError,
            provider_code: None,
            message: "Connection timeout".to_string(),
            details: None,
            retry_after: None,
        };
        
        let diagnostics = ErrorDiagnostics::from_http_error(&error)
            .with_context("openai", "gpt-4", "https://api.openai.com/v1/chat/completions");
        
        let display = diagnostics.format_display(false);
        
        assert!(display.contains("ERROR: NetworkError"));
        assert!(display.contains("Connection timeout"));
        assert!(display.contains("Provider: openai"));
        assert!(display.contains("Model: gpt-4"));
        assert!(display.contains("Suggested actions:"));
    }
}