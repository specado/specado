//! Example demonstrating the high-level LLM API
//! 
//! Run with: cargo run --example high_level_api

use specado_core::{LLM, GenerationMode, Message, ResponseExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Simple generation with GPT-5
    println!("=== Simple Generation Example ===\n");
    
    let llm = LLM::new("gpt-5-mini")?;
    let response = llm.generate(
        "Write a haiku about rust programming",
        GenerationMode::Creative,
        Some(100)
    ).await?;
    
    println!("Response: {}", response.text());
    println!("Tokens used: {:?}\n", response.usage());
    
    // Example 2: Chat with Claude
    println!("=== Chat Example ===\n");
    
    let llm = LLM::new("claude-opus-4.1")?;
    let messages = vec![
        Message::system("You are a helpful assistant"),
        Message::user("What are the benefits of Rust?"),
    ];
    
    let response = llm.chat(
        messages,
        GenerationMode::Balanced,
        Some(200)
    ).await?;
    
    println!("Response: {}", response.text());
    
    // Example 3: Using different generation modes
    println!("=== Generation Modes Example ===\n");
    
    let llm = LLM::new("gpt-5")?;
    
    // Precise mode for factual content
    let precise_response = llm.generate(
        "List 3 key features of Rust",
        GenerationMode::Precise,
        Some(150)
    ).await?;
    
    println!("Precise mode response: {}", precise_response.text());
    
    // Creative mode for imaginative content
    let creative_response = llm.generate(
        "Imagine a world where computers program themselves",
        GenerationMode::Creative,
        Some(200)
    ).await?;
    
    println!("Creative mode response: {}", creative_response.text());
    
    Ok(())
}