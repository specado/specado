//! Validation demonstration example
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use serde_json::json;
use specado_schemas::{
    create_prompt_spec_validator, create_provider_spec_validator,
    SchemaValidator, ValidationMode, ValidationContext
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Specado Schema Validation Demo ===\n");

    // Demo PromptSpec validation
    demo_prompt_spec_validation()?;
    
    println!();
    
    // Demo ProviderSpec validation  
    demo_provider_spec_validation()?;

    Ok(())
}

fn demo_prompt_spec_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- PromptSpec Validation Demo ---");
    
    let validator = create_prompt_spec_validator()?;
    
    // Valid PromptSpec
    let valid_spec = json!({
        "spec_version": "1.0",
        "id": "demo-123",
        "model_class": "Chat", 
        "messages": [
            {"role": "user", "content": "Hello, world!"}
        ]
    });
    
    println!("✅ Validating valid PromptSpec:");
    match validator.validate(&valid_spec) {
        Ok(_) => println!("   Valid!"),
        Err(e) => println!("   Error: {}", e),
    }
    
    // Invalid PromptSpec - missing required field
    let invalid_spec = json!({
        "spec_version": "1.0",
        "id": "demo-invalid",
        "model_class": "Chat"
        // Missing required 'messages' field
    });
    
    println!("\n❌ Validating invalid PromptSpec (missing messages):");
    match validator.validate(&invalid_spec) {
        Ok(_) => println!("   Unexpectedly valid!"),
        Err(e) => println!("   Error: {}", e),
    }
    
    // Invalid business rule - tool_choice without tools
    let business_rule_violation = json!({
        "spec_version": "1.0", 
        "id": "demo-business-rule",
        "model_class": "Chat",
        "messages": [{"role": "user", "content": "Test"}],
        "tool_choice": "auto"
        // Missing 'tools' array required by business rule
    });
    
    println!("\n⚠️  Validating business rule violation (tool_choice without tools):");
    match validator.validate(&business_rule_violation) {
        Ok(_) => println!("   Unexpectedly valid!"),
        Err(e) => println!("   Error: {}", e),
    }
    
    // Test different validation modes
    println!("\n🔍 Testing validation modes:");
    let context = ValidationContext::new(ValidationMode::Basic);
    match validator.validate_with_context(&business_rule_violation, &context) {
        Ok(_) => println!("   Basic mode: Valid (only basic structure checked)"),
        Err(e) => println!("   Basic mode error: {}", e),
    }
    
    Ok(())
}

fn demo_provider_spec_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- ProviderSpec Validation Demo ---");
    
    let validator = create_provider_spec_validator()?;
    
    // Valid ProviderSpec
    let valid_spec = json!({
        "spec_version": "1.0",
        "provider_id": "demo-provider",
        "base_url": "https://api.demo.com",
        "authentication": {
            "type": "api_key",
            "env_var": "${ENV:DEMO_API_KEY}"
        },
        "capabilities": {
            "supports_tools": true,
            "supports_rag": false,
            "supports_streaming": true,
            "model_families": ["chat"]
        }
    });
    
    println!("✅ Validating valid ProviderSpec:");
    match validator.validate(&valid_spec) {
        Ok(_) => println!("   Valid!"),
        Err(e) => println!("   Error: {}", e),
    }
    
    // Invalid ProviderSpec - missing required field
    let invalid_spec = json!({
        "spec_version": "1.0",
        "provider_id": "demo-invalid"
        // Missing required fields: base_url, authentication, capabilities
    });
    
    println!("\n❌ Validating invalid ProviderSpec (missing required fields):");
    match validator.validate(&invalid_spec) {
        Ok(_) => println!("   Unexpectedly valid!"),
        Err(e) => println!("   Error: {}", e),
    }
    
    // Invalid environment variable format
    let invalid_env_var = json!({
        "spec_version": "1.0",
        "provider_id": "demo-env",
        "base_url": "https://api.demo.com",
        "authentication": {
            "type": "api_key",
            "env_var": "${env:invalid_format}"  // Should be ${ENV:VARIABLE_NAME}
        },
        "capabilities": {
            "supports_tools": false,
            "supports_rag": false,
            "supports_streaming": true,
            "model_families": ["chat"]
        }
    });
    
    println!("\n⚠️  Validating invalid environment variable format:");
    match validator.validate(&invalid_env_var) {
        Ok(_) => println!("   Unexpectedly valid!"),
        Err(e) => println!("   Error: {}", e),
    }
    
    Ok(())
}