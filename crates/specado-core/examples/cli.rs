// Simple CLI for any LLM model using Specado
// Usage: cargo run --example cli [model]
// Example: cargo run --example cli gpt-5

use specado_core::types::{PromptSpec, ProviderSpec};
use specado_core::translation::translate;
use dotenv::dotenv;
use reqwest::Client;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();
    
    // Get model from command line (default: gpt-5)
    let args: Vec<String> = env::args().collect();
    let model = args.get(1).map(|s| s.as_str()).unwrap_or("gpt-5");
    
    // Determine provider configuration based on model
    let (provider_name, api_key_env, api_url) = match model {
        m if m.starts_with("gpt") || m.starts_with("o1") => 
            ("openai", "OPENAI_API_KEY", "https://api.openai.com/v1/chat/completions"),
        m if m.starts_with("claude") => 
            ("anthropic", "ANTHROPIC_API_KEY", "https://api.anthropic.com/v1/messages"),
        _ => ("openai", "OPENAI_API_KEY", "https://api.openai.com/v1/chat/completions"),
    };
    
    // Get API key
    let api_key = env::var(api_key_env)?;
    
    // Get user prompt
    println!("Prompt: ");
    let mut prompt = String::new();
    std::io::stdin().read_line(&mut prompt)?;
    
    // Load provider specification
    let spec_path = format!("providers/{}/{}.json", provider_name, model);
    let spec_json = std::fs::read_to_string(&spec_path)?;
    let provider_spec: ProviderSpec = serde_json::from_str(&spec_json)?;
    
    // Create prompt with just the message - everything else uses defaults
    let prompt_spec = PromptSpec::new(prompt.trim());
    
    // Translate to provider format and make API call
    let request = translate(&prompt_spec, &provider_spec, model, Default::default())?;
    
    let response = Client::new()
        .post(api_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request.provider_request_json)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    
    // Extract and display response (handles both OpenAI and Anthropic formats)
    let content = response["choices"][0]["message"]["content"]
        .as_str()
        .or_else(|| response["content"][0]["text"].as_str())
        .unwrap_or("No response");
    
    println!("\n{}", content);
    
    Ok(())
}