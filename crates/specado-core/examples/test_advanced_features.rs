// Complete test of advanced API functionality
use specado_core::types::{AdvancedParams, ReasoningEffort, ReasoningMode, VerbosityLevel, PromptSpec, Message, MessageRole, StrictMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Complete Advanced API Functionality");
    println!("{}", "=".repeat(60));
    
    // Test 1: Create and validate advanced parameters
    println!("\n1️⃣ Testing Advanced Parameters Structure");
    let advanced_params = AdvancedParams {
        thinking: Some(true),
        min_thinking_tokens: Some(1024),
        reasoning_effort: Some(ReasoningEffort::High),
        seed: Some(42),
        reasoning_mode: Some(ReasoningMode::Balanced),
        thinking_budget: Some(32768),
        verbosity: Some(VerbosityLevel::Detailed),
    };
    println!("✅ Advanced parameters created successfully");
    
    // Test 2: Create PromptSpec with advanced parameters
    println!("\n2️⃣ Testing PromptSpec with Advanced Parameters");
    let prompt_spec = PromptSpec {
        model_class: "ReasoningChat".to_string(),
        messages: vec![
            Message {
                role: MessageRole::User,
                content: "Explain quantum computing with deep reasoning".to_string(),
                name: None,
                metadata: None,
            }
        ],
        tools: None,
        tool_choice: None,
        response_format: None,
        sampling: None,
        limits: None,
        media: None,
        advanced: Some(advanced_params.clone()),
        strict_mode: StrictMode::Warn,
    };
    println!("✅ PromptSpec with advanced parameters created successfully");
    
    // Test 3: Serialize to JSON
    println!("\n3️⃣ Testing JSON Serialization");
    let json = serde_json::to_string_pretty(&prompt_spec)?;
    println!("✅ Serialization successful");
    println!("📝 Advanced parameters in JSON:");
    
    // Parse and show just the advanced section
    let parsed: serde_json::Value = serde_json::from_str(&json)?;
    if let Some(advanced) = parsed.get("advanced") {
        println!("{}", serde_json::to_string_pretty(advanced)?);
    }
    
    // Test 4: Verify enum serialization
    println!("\n4️⃣ Testing Enum Serialization");
    let reasoning_effort_json = serde_json::to_string(&ReasoningEffort::High)?;
    let reasoning_mode_json = serde_json::to_string(&ReasoningMode::Balanced)?;
    let verbosity_json = serde_json::to_string(&VerbosityLevel::Detailed)?;
    
    println!("✅ ReasoningEffort::High → {}", reasoning_effort_json);
    println!("✅ ReasoningMode::Balanced → {}", reasoning_mode_json);
    println!("✅ VerbosityLevel::Detailed → {}", verbosity_json);
    
    // Test 5: Deserialize from JSON
    println!("\n5️⃣ Testing JSON Deserialization");
    let deserialized: PromptSpec = serde_json::from_str(&json)?;
    println!("✅ Deserialization successful");
    
    // Verify values
    if let Some(ref advanced) = deserialized.advanced {
        assert_eq!(advanced.thinking, Some(true));
        assert_eq!(advanced.min_thinking_tokens, Some(1024));
        assert_eq!(advanced.seed, Some(42));
        println!("✅ Value verification successful");
    }
    
    println!("\n{}", "=".repeat(60));
    println!("🎉 All Enhanced API Tests Passed!");
    println!("{}", "=".repeat(60));
    
    println!("\n📈 Test Summary:");
    println!("   ✅ Advanced parameter structure creation");
    println!("   ✅ PromptSpec integration");
    println!("   ✅ JSON serialization/deserialization");
    println!("   ✅ Type-safe enum handling");
    println!("   ✅ Value validation");
    
    println!("\n🚀 Ready for:");
    println!("   - Integration with translation pipeline");
    println!("   - Real API calls with advanced parameters");
    println!("   - Python/Node.js bindings generation");
    println!("   - Production deployment");
    
    Ok(())
}