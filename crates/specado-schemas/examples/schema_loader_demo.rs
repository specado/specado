//! Schema Loader Demonstration
//!
//! This example shows how to use the SchemaLoader with both YAML and JSON files,
//! including reference resolution and environment variable expansion.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use specado_schemas::{
    create_schema_loader, SchemaLoader, LoaderResult,
    loader::{
        cache::CacheConfig,
        schema_loader::LoaderConfig,
    },
};
use serde_json::json;
use std::env;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn main() -> LoaderResult<()> {
    println!("🚀 Schema Loader Demonstration");
    println!("================================\n");

    // Create a temporary directory for our examples
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Set up environment variable for testing
    env::set_var("DEMO_API_KEY", "demo-secret-key-12345");
    env::set_var("DEMO_BASE_URL", "https://api.example.com/v1");
    env::set_var("USER_NAME", "Demo User");

    // Create example schema files
    create_example_files(&base_path)?;

    // Demonstrate basic loading
    demo_basic_loading(&base_path)?;

    // Demonstrate reference resolution
    demo_reference_resolution(&base_path)?;

    // Demonstrate environment variable expansion
    demo_environment_expansion(&base_path)?;

    // Demonstrate caching
    demo_caching(&base_path)?;

    // Demonstrate batch loading
    demo_batch_loading(&base_path)?;

    // Demonstrate metadata extraction
    demo_metadata_extraction(&base_path)?;

    // Demonstrate error handling
    demo_error_handling(&base_path)?;

    println!("\n✅ All demonstrations completed successfully!");

    Ok(())
}

fn create_example_files(base_path: &Path) -> LoaderResult<()> {
    println!("📁 Creating example schema files...\n");

    // Create a basic PromptSpec in YAML
    let prompt_spec = r#"# Example PromptSpec
spec_version: "1.0"
id: "greeting-prompt"
model_class: "Chat"
description: "A simple greeting prompt"
messages:
  - role: "system"
    content: "You are a helpful assistant."
  - role: "user"
    content: "Say hello to ${ENV:USER_NAME}!"
parameters:
  temperature: 0.7
  max_tokens: 100
"#;
    fs::write(base_path.join("prompt.yaml"), prompt_spec).unwrap();

    // Create a ProviderSpec in JSON
    let provider_spec = json!({
        "spec_version": "1.0",
        "provider_name": "example-provider",
        "base_url": "${ENV:DEMO_BASE_URL}",
        "authentication": {
            "type": "bearer",
            "api_key": "${ENV:DEMO_API_KEY}"
        },
        "capabilities": {
            "chat": true,
            "streaming": true,
            "tools": false
        },
        "model_mappings": {
            "$ref": "models.yaml#/definitions/models"
        }
    });
    fs::write(
        base_path.join("provider.json"),
        serde_json::to_string_pretty(&provider_spec).unwrap(),
    ).unwrap();

    // Create a referenced models file
    let models_spec = r#"# Model definitions
spec_version: "1.0"
definitions:
  models:
    "gpt-4":
      family: "openai"
      input_modes: ["text"]
      max_tokens: 8192
    "gpt-3.5-turbo":
      family: "openai"
      input_modes: ["text"]
      max_tokens: 4096
"#;
    fs::write(base_path.join("models.yaml"), models_spec).unwrap();

    // Create a schema with circular reference (for error demo)
    let circular_a = r#"
type: object
properties:
  b_ref:
    $ref: "circular_b.yaml"
"#;
    fs::write(base_path.join("circular_a.yaml"), circular_a).unwrap();

    let circular_b = r#"
type: object
properties:
  a_ref:
    $ref: "circular_a.yaml"
"#;
    fs::write(base_path.join("circular_b.yaml"), circular_b).unwrap();

    // Create an invalid schema for error demo
    let invalid_schema = r#"
this is not valid YAML: {
  "nor valid JSON"
"#;
    fs::write(base_path.join("invalid.yaml"), invalid_schema).unwrap();

    println!("   ✅ Created example files:");
    println!("   - prompt.yaml (PromptSpec with env vars)");
    println!("   - provider.json (ProviderSpec with references)");
    println!("   - models.yaml (Referenced definitions)");
    println!("   - circular_*.yaml (For error demonstration)");
    println!("   - invalid.yaml (For error demonstration)");

    Ok(())
}

fn demo_basic_loading(base_path: &Path) -> LoaderResult<()> {
    println!("\n🔄 Basic Schema Loading");
    println!("======================");

    let mut loader = create_schema_loader();

    // Load YAML PromptSpec
    println!("\n📋 Loading PromptSpec from YAML...");
    let prompt_path = base_path.join("prompt.yaml");
    let prompt_spec = loader.load_prompt_spec(&prompt_path)?;
    
    println!("   ID: {}", prompt_spec["id"]);
    println!("   Model Class: {}", prompt_spec["model_class"]);
    println!("   Description: {}", prompt_spec["description"]);

    // Load JSON ProviderSpec
    println!("\n🏭 Loading ProviderSpec from JSON...");
    let provider_path = base_path.join("provider.json");
    let provider_spec = loader.load_provider_spec(&provider_path)?;
    
    println!("   Provider: {}", provider_spec["provider_name"]);
    println!("   Base URL: {}", provider_spec["base_url"]);
    println!("   Has Chat Capability: {}", provider_spec["capabilities"]["chat"]);

    Ok(())
}

fn demo_reference_resolution(base_path: &Path) -> LoaderResult<()> {
    println!("\n🔗 Reference Resolution");
    println!("======================");

    let mut loader = create_schema_loader();
    let provider_path = base_path.join("provider.json");
    
    println!("\n📖 Loading ProviderSpec with $ref resolution...");
    let provider_spec = loader.load_schema(&provider_path)?;
    
    // The model_mappings should now contain the resolved content
    if let Some(models) = provider_spec.get("model_mappings") {
        println!("   ✅ Reference resolved successfully!");
        if let Some(gpt4) = models.get("gpt-4") {
            println!("   GPT-4 Max Tokens: {}", gpt4["max_tokens"]);
            println!("   GPT-4 Family: {}", gpt4["family"]);
        }
    } else {
        println!("   ❌ Reference resolution failed");
    }

    Ok(())
}

fn demo_environment_expansion(base_path: &Path) -> LoaderResult<()> {
    println!("\n🌍 Environment Variable Expansion");
    println!("=================================");

    // Set a test environment variable
    env::set_var("USER_NAME", "Alice");

    let mut loader = create_schema_loader();
    
    println!("\n🔧 Loading schemas with environment variable expansion...");
    
    // Load PromptSpec with env vars
    let prompt_path = base_path.join("prompt.yaml");
    let prompt_spec = loader.load_schema(&prompt_path)?;
    
    println!("   Original: Say hello to ${{ENV:USER_NAME}}!");
    if let Some(content) = prompt_spec["messages"][1]["content"].as_str() {
        println!("   Expanded: {}", content);
    }

    // Load ProviderSpec with env vars
    let provider_path = base_path.join("provider.json");
    let provider_spec = loader.load_schema(&provider_path)?;
    
    println!("\n   Provider Authentication:");
    println!("   Base URL: {}", provider_spec["base_url"]);
    println!("   API Key: {}", provider_spec["authentication"]["api_key"]);

    Ok(())
}

fn demo_caching(base_path: &Path) -> LoaderResult<()> {
    println!("\n💾 Caching Demonstration");
    println!("========================");

    let mut loader = create_schema_loader();
    let prompt_path = base_path.join("prompt.yaml");

    println!("\n📊 Cache statistics before loading:");
    let stats = loader.cache_stats();
    println!("   Entries: {}/{}", stats.total_entries, stats.max_entries);
    println!("   Utilization: {:.1}%", stats.utilization());

    // First load (should parse and cache)
    println!("\n🔄 First load (should parse file)...");
    let start = std::time::Instant::now();
    let _spec1 = loader.load_schema(&prompt_path)?;
    let first_duration = start.elapsed();
    println!("   Duration: {:?}", first_duration);
    println!("   Cached: {}", loader.is_cached(&prompt_path)?);

    // Second load (should use cache)
    println!("\n⚡ Second load (should use cache)...");
    let start = std::time::Instant::now();
    let _spec2 = loader.load_schema(&prompt_path)?;
    let second_duration = start.elapsed();
    println!("   Duration: {:?}", second_duration);
    
    if second_duration < first_duration {
        println!("   ✅ Cache speedup: {:.1}x faster!", 
                first_duration.as_nanos() as f64 / second_duration.as_nanos() as f64);
    }

    // Cache statistics after loading
    println!("\n📊 Cache statistics after loading:");
    let stats = loader.cache_stats();
    println!("   Entries: {}/{}", stats.total_entries, stats.max_entries);
    println!("   Utilization: {:.1}%", stats.utilization());

    Ok(())
}

fn demo_batch_loading(base_path: &Path) -> LoaderResult<()> {
    println!("\n📦 Batch Loading");
    println!("===============");

    let mut loader = create_schema_loader();
    
    println!("\n🔄 Loading multiple schemas in batch...");
    let file_paths = vec![
        base_path.join("prompt.yaml"),
        base_path.join("provider.json"),
        base_path.join("models.yaml"),
    ];
    let paths: Vec<&Path> = file_paths.iter().map(|p| p.as_path()).collect();

    let start = std::time::Instant::now();
    let results = loader.load_schemas_batch(&paths)?;
    let duration = start.elapsed();

    println!("   ✅ Loaded {} schemas in {:?}", results.len(), duration);
    
    for (path, schema) in &results {
        let filename = path.file_name().unwrap().to_str().unwrap();
        if let Some(id) = schema.get("id").and_then(|v| v.as_str()) {
            println!("   - {}: {}", filename, id);
        } else if let Some(provider) = schema.get("provider_name").and_then(|v| v.as_str()) {
            println!("   - {}: {}", filename, provider);
        } else {
            println!("   - {}: (definitions file)", filename);
        }
    }

    Ok(())
}

fn demo_metadata_extraction(base_path: &Path) -> LoaderResult<()> {
    println!("\n📋 Metadata Extraction");
    println!("======================");

    let mut loader = create_schema_loader();
    
    println!("\n📖 Extracting metadata from schemas...");
    
    let schema_files = ["prompt.yaml", "provider.json", "models.yaml"];
    
    for filename in &schema_files {
        let path = base_path.join(filename);
        let metadata = loader.get_schema_metadata(&path)?;
        
        println!("\n   📄 {}:", filename);
        println!("      Format: {:?}", metadata.format);
        println!("      Version: {:?}", metadata.version);
        println!("      Type: {:?}", metadata.schema_type);
        if let Some(size) = metadata.size_bytes {
            println!("      Size: {} bytes", size);
        }
        if let Some(modified) = metadata.last_modified {
            println!("      Modified: {:?}", modified);
        }
    }

    Ok(())
}

fn demo_error_handling(base_path: &Path) -> LoaderResult<()> {
    println!("\n⚠️  Error Handling");
    println!("==================");

    let mut loader = create_schema_loader();

    // Test parsing error
    println!("\n🔴 Testing parsing error...");
    let invalid_path = base_path.join("invalid.yaml");
    match loader.load_schema(&invalid_path) {
        Ok(_) => println!("   ❌ Expected parsing error but got success"),
        Err(e) => {
            println!("   ✅ Caught parsing error: {}", e);
            if let Some(path) = e.path() {
                println!("      File: {}", path.display());
            }
        }
    }

    // Test circular reference error
    println!("\n🔄 Testing circular reference error...");
    let circular_path = base_path.join("circular_a.yaml");
    match loader.load_schema(&circular_path) {
        Ok(_) => println!("   ❌ Expected circular reference error but got success"),
        Err(e) => {
            println!("   ✅ Caught circular reference error: {}", e);
        }
    }

    // Test missing file error
    println!("\n📁 Testing missing file error...");
    let missing_path = base_path.join("nonexistent.yaml");
    match loader.load_schema(&missing_path) {
        Ok(_) => println!("   ❌ Expected file not found error but got success"),
        Err(e) => {
            println!("   ✅ Caught file error: {}", e);
        }
    }

    // Test environment variable error
    println!("\n🌍 Testing environment variable error...");
    let env_error_content = r#"
spec_version: "1.0"
api_key: "${ENV:NONEXISTENT_VAR}"
"#;
    let env_error_path = base_path.join("env_error.yaml");
    fs::write(&env_error_path, env_error_content).unwrap();
    
    match loader.load_schema(&env_error_path) {
        Ok(_) => println!("   ❌ Expected environment variable error but got success"),
        Err(e) => {
            println!("   ✅ Caught environment variable error: {}", e);
        }
    }

    Ok(())
}

/// Create a custom loader configuration for demonstration
#[allow(dead_code)]
fn create_custom_loader() -> SchemaLoader {
    let cache_config = CacheConfig {
        max_entries: 50,
        max_age: Some(std::time::Duration::from_secs(300)), // 5 minutes
        enabled: true,
    };

    let loader_config = LoaderConfig {
        cache: cache_config,
        max_resolution_depth: 5,
        allow_env_expansion: true,
        validate_basic_structure: true,
        auto_resolve_refs: true,
        base_dir: None,
    };

    SchemaLoader::with_config(loader_config)
}