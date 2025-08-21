# Story 1.1.1: Pre-validated Model Specifications - Architecture

## Executive Summary

This document defines the **spec-driven architecture** for implementing pre-validated model specifications in Specado, enabling support for modern LLM models while maintaining system functionality. The architecture extends existing `ModelSpec` types incrementally and uses **capability inference** rather than hardcoded model detection.

## Architecture Goals

1. **Zero Configuration**: Users can start generating text immediately without provider setup
2. **Spec-Driven**: Capabilities inferred from ModelSpec structure, no hardcoded model names
3. **Extensibility**: New models automatically get capabilities based on their specifications
4. **Validation**: Multi-level validation with graceful fallback for unknown parameters
5. **Performance**: Fast loading with intelligent caching and minimal runtime overhead
6. **Backward Compatibility**: Full compatibility with existing codebase and specifications

## Current Model Support (First Pass)

### OpenAI
- **gpt-5**: Latest flagship model with advanced reasoning (existing spec enhanced)
- **gpt-5-mini**: Efficient model for common use cases (existing spec enhanced)
- **gpt-5-nano**: Ultra-fast model for simple tasks (new spec to be created)

### Anthropic  
- **claude-4-sonnet**: Balanced performance and speed (created)
- **claude-opus-4.1**: Maximum capability model (existing)
- **claude-sonnet-4**: Balanced model (existing)

*Google and Meta models deferred to later phase for focused delivery*

## System Architecture (Minimal Extensions to Existing System)

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Application Layer (Unchanged)                   │
├─────────────────────────────────────────────────────────────────────┤
│                    Existing Core Types (Extended)                   │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐      │
│  │  ProviderSpec   │ │   ModelSpec+    │ │CapabilityDetector│     │
│  │  (unchanged)    │ │ (+ capabilities)│ │ (spec-driven)   │      │
│  └─────────────────┘ └─────────────────┘ └─────────────────┘      │
├─────────────────────────────────────────────────────────────────────┤
│              Existing Storage (providers/ directory)               │
│           ┌─────────────────┐ ┌─────────────────┐                  │
│           │   JSON Specs    │ │specado-schemas  │                  │
│           │ (existing path) │ │  (unchanged)    │                  │
│           └─────────────────┘ └─────────────────┘                  │
└─────────────────────────────────────────────────────────────────────┘
```

## Spec-Driven Capability System

### Core Principle: Infer Capabilities from ModelSpec Structure

**No Hardcoded Model Names** - Capabilities are determined by analyzing the ModelSpec fields:

```rust
impl CapabilityDetector {
    /// Extract capabilities from ModelSpec structure (no hardcoding)
    pub fn extract_capabilities_from_spec(model_spec: &ModelSpec) -> Capabilities {
        let mut capabilities = Capabilities::default();
        
        // Infer from input_modes
        capabilities.vision = model_spec.input_modes.images.unwrap_or(false);
        
        // Infer from tooling config  
        capabilities.function_calling = model_spec.tooling.tools_supported;
        capabilities.streaming = model_spec.endpoints.streaming_chat_completion.is_some();
        
        // Infer from parameters (reasoning/thinking parameters)
        if let Some(params_obj) = model_spec.parameters.as_object() {
            capabilities.reasoning = params_obj.contains_key("reasoning_depth")
                .or(params_obj.contains_key("thinking_budget"))
                .or(params_obj.contains_key("reasoning_mode"))
                .then_some(true);
                
            capabilities.extended_context = Self::detect_extended_context(params_obj);
        }
        
        capabilities
    }
    
    /// Detect extended context from max_tokens values >100k
    fn detect_extended_context(params: &Map<String, Value>) -> Option<bool> {
        for key in ["max_tokens", "max_output_tokens", "context_window"] {
            if let Some(value) = params.get(key) {
                if let Some(max_val) = value.as_u64() {
                    if max_val > 100_000 { return Some(true); }
                }
            }
        }
        None
    }
}
```

### Capability Detection Examples

**Example 1: OpenAI GPT-5 Model**
```json
// providers/openai/gpt-5.json (existing structure + inferred capabilities)
{
  "input_modes": { "images": true },              // → vision: true
  "tooling": { "tools_supported": true },         // → function_calling: true
  "endpoints": { "streaming_chat_completion": {} }, // → streaming: true
  "parameters": {
    "max_tokens": { "maximum": 200000 },          // → extended_context: true
    "reasoning_depth": { "enum": ["shallow", "deep"] } // → reasoning: true
  }
}
// Result: Capabilities automatically inferred from spec structure
```

**Example 2: Anthropic Claude Model**
```json
// providers/anthropic/claude-4-sonnet.json (spec-driven detection)
{
  "input_modes": { "images": true, "documents": true },
  "tooling": { "tools_supported": true },
  "parameters": {
    "thinking_budget": { "maximum": 65536 },      // → reasoning: true
    "max_tokens": { "maximum": 200000 }           // → extended_context: true
  }
}
// Result: Capabilities inferred without hardcoding "claude-4" anywhere
```
    
    /// Performance characteristics
    pub performance: PerformanceSpec,
}
```

### ParameterSpec

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSpec {
    /// Parameter type
    pub param_type: ParameterType,
    
    /// Default value
    pub default: Option<serde_json::Value>,
    
    /// Validation rules
    pub validation: ValidationRules,
    
    /// Parameter description
    pub description: String,
    
    /// Whether parameter is required
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    Float { min: Option<f64>, max: Option<f64> },
    Integer { min: Option<i64>, max: Option<i64> },
    String { max_length: Option<usize> },
    Boolean,
    Array { item_type: Box<ParameterType>, max_items: Option<usize> },
    Object { schema: serde_json::Value },
}
```

### CapabilitySet

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySet {
    /// Text generation capability
    pub text_generation: bool,
    
    /// Vision/image analysis capability
    pub vision: bool,
    
    /// Function calling capability  
    pub function_calling: bool,
    
    /// Streaming response capability
    pub streaming: bool,
    
    /// Reasoning capability (chain-of-thought)
    pub reasoning: bool,
    
    /// Code generation/execution capability
    pub code_execution: bool,
    
    /// Custom capabilities for extensibility
    pub custom: HashMap<String, bool>,
}
```

## File Structure

```
/
├── providers/                          # Specification files (existing structure)
│   ├── openai/
│   │   ├── gpt-5.json                 # GPT-5 specification (existing)
│   │   ├── gpt-5-mini.json            # GPT-5 Mini specification (existing)
│   │   └── gpt-5-nano.json            # GPT-5 Nano specification (to be created)
│   ├── anthropic/
│   │   ├── claude-4-sonnet.json       # Claude 4 Sonnet (created)
│   │   ├── claude-opus-4.1.json       # Claude Opus 4.1 (existing)
│   │   └── claude-sonnet-4.json       # Claude Sonnet 4 (existing)
│   └── google/
│       ├── gemini-pro.json            # Gemini Pro (to be created)
│       └── gemini-flash.json          # Gemini Flash (to be created)
crates/specado-core/src/
├── provider_discovery/                 # Enhanced discovery (existing module)
│   ├── mod.rs                         # Extended for capability discovery
│   └── error.rs                       # Discovery-specific errors
├── specs/                             # New enhanced specs module
│   ├── mod.rs                         # Capability extensions to ModelSpec
│   └── capabilities.rs                # Capability types and detection
└── tests/
    ├── fixtures/                      # Test specification files
    └── integration/                   # Provider API integration tests
```

## Specification File Format

### Provider Specification (JSON - matches existing format)

```json
// providers/openai/gpt-5.json (existing format extended)
{
  "spec_version": "1.1.0",
  "provider": {
    "name": "openai",
    "base_url": "https://api.openai.com/v1",
    "headers": {
      "Authorization": "Bearer ${OPENAI_API_KEY}"
    }
  },
  "models": [
    {
      "id": "gpt-5",
      "aliases": ["gpt-5-thinking"],
      "family": "gpt",
      "description": "Most capable OpenAI model with advanced reasoning",
    
    parameters:
      temperature:
        type: float
        min: 0.0
        max: 2.0
        default: 1.0
        description: Controls randomness in response generation
        required: false
        
      max_tokens:
        type: integer
        min: 1
        max: 200000
        default: null
        description: Maximum tokens in response
        required: false
        
      top_p:
        type: float
        min: 0.0
        max: 1.0
        default: 1.0
        description: Nucleus sampling parameter
        required: false
    
    capabilities:
      text_generation: true
      vision: true
      function_calling: true
      streaming: true
      reasoning: true
      code_execution: false
      
    limits:
      context_window: 200000
      max_output_tokens: 32000
      max_function_calls: 10
      
    performance:
      latency_p50_ms: 800
      latency_p95_ms: 2000
      throughput_tokens_per_second: 150

  gpt-5-mini:
    id: gpt-5-mini
    display_name: GPT-5 Mini
    description: Efficient OpenAI model optimized for common use cases
    # ... similar structure with different values
    
  gpt-5-nano:
    id: gpt-5-nano  
    display_name: GPT-5 Nano
    description: Ultra-fast OpenAI model for simple tasks
    # ... similar structure with different values
```

## API Design

### Registry Interface

```rust
impl SpecRegistry {
    /// Initialize registry with all provider specifications
    pub fn new() -> Result<Self, SpecError> {
        // Load all provider specs from disk
        // Validate against JSON schemas
        // Build in-memory registry
    }
    
    /// Get provider specification by name
    pub fn get_provider(&self, name: &str) -> Option<&ProviderSpec> {
        self.providers.get(name)
    }
    
    /// Get model specification
    pub fn get_model(&self, provider: &str, model: &str) -> Option<&ModelSpec> {
        self.get_provider(provider)?
            .models.get(model)
    }
    
    /// List all available providers
    pub fn list_providers(&self) -> Vec<&str> {
        self.providers.keys().collect()
    }
    
    /// List models for a provider
    pub fn list_models(&self, provider: &str) -> Vec<&str> {
        self.get_provider(provider)
            .map(|p| p.models.keys().collect())
            .unwrap_or_default()
    }
    
    /// Validate parameters against model spec
    pub fn validate_parameters(
        &self,
        provider: &str,
        model: &str,
        params: &serde_json::Value,
    ) -> Result<(), ValidationError> {
        // Implementation for parameter validation
    }
    
    /// Hot reload specifications from disk
    pub fn reload(&mut self) -> Result<(), SpecError> {
        // Re-read spec files and update registry
    }
}
```

### Validation Engine

```rust
impl SpecValidator {
    /// Validate a provider specification
    pub fn validate_provider_spec(spec: &ProviderSpec) -> Result<(), ValidationError> {
        // JSON schema validation
        // Custom business logic validation
        // Cross-reference validation
    }
    
    /// Validate runtime parameters
    pub fn validate_parameters(
        model_spec: &ModelSpec,
        params: &serde_json::Value,
    ) -> Result<(), ValidationError> {
        // Type checking
        // Range validation  
        // Required parameter checking
        // Custom validation rules
    }
    
    /// Validate capability compatibility
    pub fn validate_capabilities(
        requested: &[String],
        available: &CapabilitySet,
    ) -> Result<(), ValidationError> {
        // Check if model supports requested capabilities
    }
}
```

## Implementation Strategy

### Phase 1: Foundation (Week 1)
1. **Core Types**: Define all Rust types and traits
2. **File Format**: Design YAML specification format  
3. **Schema**: Create JSON Schema for validation
4. **OpenAI Spec**: Implement gpt-5, gpt-5-mini, gpt-5-nano specifications

### Phase 2: Expansion (Week 1-2)
1. **Anthropic Spec**: Add claude-4-sonnet, claude-41-opus
2. **Validation Engine**: Implement comprehensive validation
3. **Registry**: Build registry with caching and hot-reload
4. **Testing**: Unit and integration tests

### Phase 3: Completion (Week 2)
1. **Google/Meta Specs**: Add Gemini and Llama specifications
2. **Error Handling**: Comprehensive error types and messages
3. **Documentation**: API documentation and usage examples
4. **Performance**: Optimize loading and validation performance

## Migration Strategy

### Coexistence Approach
- New spec system runs alongside existing code initially
- Feature flag controls which system is used
- Gradual migration of components to use new specs
- Comprehensive testing ensures no regression

### Migration Steps
1. **Parallel Development**: Build new system without affecting existing code
2. **Validation Phase**: Run both systems in parallel, compare results
3. **Gradual Rollout**: Start using new specs for new features
4. **Full Migration**: Replace all old specification code
5. **Cleanup**: Remove feature flags and old code

## Testing Strategy

### Unit Tests
- Type serialization/deserialization
- Parameter validation logic
- Registry operations
- Error handling

### Integration Tests
- Real provider API compatibility
- Specification loading from files
- End-to-end validation workflows
- Performance benchmarks

### Test Data
- Valid specification files for all providers
- Invalid specifications for error testing
- Edge case parameter values
- Real API response samples

## Performance Considerations

### Loading Performance
- **Target**: < 100ms for full registry initialization
- **Strategy**: Lazy loading of specifications
- **Caching**: In-memory caching with invalidation
- **Optimization**: Pre-compiled validation schemas

### Runtime Performance
- **Target**: < 1ms for parameter validation
- **Strategy**: Optimized validation algorithms
- **Caching**: Cache validation results for repeated calls
- **Memory**: Efficient data structures

### Memory Usage
- **Target**: < 10MB for all specifications
- **Strategy**: Compact data representations
- **Optimization**: Share common data structures
- **Monitoring**: Memory usage tracking

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum SpecError {
    #[error("Provider '{0}' not found")]
    ProviderNotFound(String),
    
    #[error("Model '{model}' not found for provider '{provider}'")]
    ModelNotFound { provider: String, model: String },
    
    #[error("Specification file error: {0}")]
    FileError(#[from] std::io::Error),
    
    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),
    
    #[error("JSON schema validation error: {0}")]
    SchemaError(String),
    
    #[error("Parameter validation error: {0}")]
    ValidationError(#[from] ValidationError),
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Parameter '{param}' is required but not provided")]
    RequiredParameter { param: String },
    
    #[error("Parameter '{param}' value {value} is outside valid range [{min}, {max}]")]
    OutOfRange { param: String, value: f64, min: f64, max: f64 },
    
    #[error("Parameter '{param}' has invalid type, expected {expected}")]
    InvalidType { param: String, expected: String },
    
    #[error("Model does not support capability '{capability}'")]
    UnsupportedCapability { capability: String },
}
```

## Security Considerations

### Specification Security
- **Validation**: All specs validated against schemas before loading
- **Sandboxing**: Specification loading isolated from application logic
- **Integrity**: Checksums or signatures for specification files
- **Access Control**: Read-only access to specification files

### API Security
- **Input Validation**: All API parameters validated before use
- **Error Messages**: Error messages don't leak sensitive information
- **Logging**: Security-relevant events logged appropriately
- **Rate Limiting**: Protection against spec-based DoS attacks

## Monitoring and Observability

### Metrics
- Specification load time and success rate
- Parameter validation performance
- Registry cache hit/miss rates
- Provider/model usage statistics

### Logging
- Specification loading events
- Validation failures with context
- Performance warnings
- Configuration changes

### Health Checks
- Registry initialization status
- Specification file integrity
- Cache performance metrics
- Provider availability validation

## Future Extensions

### Dynamic Specifications
- Runtime specification updates from provider APIs
- Automatic discovery of new models
- Version compatibility management
- A/B testing for specification changes

### Advanced Validation
- Cross-parameter validation rules
- Machine learning-based parameter optimization
- Cost optimization recommendations
- Performance prediction based on parameters

### Community Integration
- User-contributed provider specifications
- Specification marketplace
- Community validation and testing
- Collaborative specification maintenance

## Conclusion

This architecture provides a robust, extensible foundation for LLM provider specifications that enables Specado's smart defaults while maintaining flexibility for advanced use cases. The design balances simplicity for users with comprehensive functionality for the system, supporting both current requirements and future growth.