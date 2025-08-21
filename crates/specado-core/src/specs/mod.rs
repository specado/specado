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

/// Check if a model uses the modern specification system (based on capabilities field)
pub fn is_modern_model(model_spec: &crate::types::ModelSpec) -> bool {
    model_spec.capabilities.is_some() || CapabilityDetector::has_enhanced_capabilities(model_spec)
}

// TODO: Implement enhanced validation when registry types are available
// /// Validate a prompt specification against enhanced model capabilities
// pub async fn validate_enhanced_prompt(
//     registry: &EnhancedSpecRegistry,
//     provider: &str,
//     model: &str,
//     prompt_spec: &crate::PromptSpec,
// ) -> Result<ValidationResult, ValidationError> {
//     // Implementation will be added when registry types are defined
//     unimplemented!("Enhanced validation will be implemented in future task")
// }

#[cfg(test)]
mod tests {
    // TODO: Add tests for modern model detection when we have sample ModelSpec instances
    // #[test]
    // fn test_modern_model_detection() {
    //     // Tests will be implemented when we create sample ModelSpec instances
    // }
}