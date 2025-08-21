//! Enhanced capabilities for modern LLM models
//!
//! This module provides minimal extensions to the existing ModelSpec type
//! to support capability discovery for modern models. It builds on the existing
//! provider_discovery module and maintains full backward compatibility.

pub mod capabilities;

// Re-export main types
pub use capabilities::{
    Capabilities, CachedCapabilities, CapabilityDetector, DiscoveryFlags
};

/// Current specification version for capability extensions
pub const ENHANCED_SPEC_VERSION: &str = "1.1.0";

/// Check if a model uses the modern specification system
pub fn is_modern_model(provider: &str, model: &str) -> bool {
    matches!((provider, model), 
        ("openai", model_id) if model_id.starts_with("gpt-5") |
        ("anthropic", model_id) if model_id.starts_with("claude-4")
    )
}

/// Validate a prompt specification against enhanced model capabilities
pub async fn validate_enhanced_prompt(
    registry: &EnhancedSpecRegistry,
    provider: &str,
    model: &str,
    prompt_spec: &crate::PromptSpec,
) -> Result<ValidationResult, ValidationError> {
    // Get model specification
    let model_spec = registry.get_model_spec(provider, model).await?;
    
    // Discover current capabilities if needed
    let capabilities = if is_modern_model(provider, model) {
        Some(registry.discover_capabilities(provider, model).await?)
    } else {
        None
    };
    
    // Run enhanced validation
    let validation_engine = registry.get_validation_engine();
    validation_engine.validate_comprehensive(model_spec, prompt_spec, capabilities.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_modern_model_detection() {
        assert!(is_modern_model("openai", "gpt-5"));
        assert!(is_modern_model("openai", "gpt-5-mini"));
        assert!(is_modern_model("openai", "gpt-5-nano"));
        assert!(is_modern_model("anthropic", "claude-4-sonnet"));
        assert!(is_modern_model("anthropic", "claude-41-opus"));
        
        assert!(!is_modern_model("openai", "gpt-4"));
        assert!(!is_modern_model("anthropic", "claude-3-sonnet"));
    }
    
    #[tokio::test]
    async fn test_enhanced_spec_initialization() {
        let result = initialize_enhanced_specs(None, None).await;
        assert!(result.is_ok());
    }
}