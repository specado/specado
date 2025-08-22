# Provider Specification Validation Rules

This document outlines the loader and linter validation rules implemented for the Provider Specification schema.

## Validation Modes

The validation system supports three modes:

- **Basic**: JSON Schema validation only
- **Partial**: Schema + selected custom rules (development mode) 
- **Strict**: Schema + all custom rules (production mode)

## JSON Schema Validation

All provider specifications must conform to the base JSON Schema defined in `schemas/provider-spec.schema.json`. This includes:

### Required Fields
- `spec_version`: Must be "1.1.0"
- `provider`: Provider configuration object
- `models`: Array of model definitions (can be empty)

### Provider Object Requirements
- `name`: Provider identifier string
- `base_url`: Valid URL for API endpoint
- `auth`: Authentication configuration (if not using templated headers)

### Model Object Requirements
- `id`: Unique model identifier
- `endpoints`: Endpoint configuration object
- Each endpoint must specify valid `method`, `path`, and `protocol`

## Custom Business Logic Rules

The following custom validation rules are enforced beyond basic JSON Schema validation:

### 1. JSONPath Validation
**Rule**: All JSONPath expressions are evaluated at runtime and should be syntactically valid
**Applies to**:
- `mappings.paths.*` - Parameter mapping paths
- `response_normalization.*_path` - Response extraction paths
- Stream event selector `type_path` values

**Flexible Syntax**:
```json
// Both forms are supported - automatic root resolution
"type_path": "object"          // Simple path
"type_path": "$.object"        // Explicit root
"content_path": "choices[0].message.content"  // Array indexing
```

### 2. Environment Variable Reference Format
**Rule**: Environment variable references must use `${ENV:VARIABLE_NAME}` format
**Applies to**:
- `provider.auth.value_template`
- Any templated configuration values

**Examples**:
```json
// Valid
"value_template": "${ENV:OPENAI_API_KEY}"

// Invalid - wrong format
"value_template": "$OPENAI_API_KEY"
```

### 3. Input Mode Compatibility
**Rule**: Model input modes must be compatible with model family constraints
**Examples**:
- Chat models cannot have image input mode
- Text-only models cannot support multimodal inputs

### 4. Capability-Dependent Configuration
**Rule**: Certain configuration sections are only valid when corresponding capabilities are enabled

#### RAG Configuration
```json
// Only valid when capabilities.supports_rag is true
"rag_config": { ... }
```

#### Conversation Management
```json
// Only valid when capabilities.supports_conversation_persistence is true  
"conversation_management": { ... }
```

#### Tool Choice Modes
```json
// When capabilities.supports_tools is true, tool_choice_modes must include "auto"
"tooling": {
  "tool_choice_modes": ["auto", "any", "tool"]
}
```

### 5. Protocol Security Consistency
**Rule**: Endpoint protocols must match base URL security level
- If `base_url` uses HTTPS/WSS, endpoints must use `https`/`wss`
- If `base_url` uses HTTP/WS, endpoints must use `http`/`ws`

**Example Validation Error**:
```json
{
  "provider": {
    "base_url": "https://api.example.com"
  },
  "models": [{
    "endpoints": {
      "chat_completion": {
        "protocol": "http"  // ERROR: Doesn't match HTTPS base_url
      }
    }
  }]
}
```

### 6. Stream Event Type Validation
**Rule**: Stream event routes must use standardized emit types
**Valid emit types**: `start`, `delta`, `tool`, `stop`, `error`, `custom`

**Stream Route Structure**:
```json
"routes": [
  {
    "when": "content_block_delta",
    "emit": "delta",           // Must be valid emit type
    "text_path": "$.delta.text"
  }
]
```

### 7. Authentication Configuration
**Rule**: Either structured `auth` object OR templated headers required (not both)

**Valid Authentication Patterns**:
```json
// Option 1: Structured auth object
"provider": {
  "auth": {
    "type": "bearer",
    "header_name": "Authorization", 
    "value_template": "Bearer ${ENV:API_KEY}"
  }
}

// Option 2: Templated header
"provider": {
  "headers": {
    "Authorization": "Bearer ${ENV:API_KEY}"
  }
}
```

### 8. Extensions and Custom Channels
**Rule**: When using `emit: "custom"`, extensions object is required
**Rule**: Channel extensions must be meaningful (reasoning, usage, debug, etc.)

```json
{
  "when": "thinking_block_delta",
  "emit": "delta", 
  "text_path": "delta.text",
  "extensions": {
    "channel": "reasoning"  // Required for specialized channels
  }
}
```

## Validation Configuration

### Batch Validation Options
- `fail_fast`: Stop on first error vs collect all errors
- `max_errors`: Limit number of errors collected (0 = unlimited)
- `mode`: Basic/Partial/Strict validation level

### Usage Examples

```rust
use specado_schemas::validation::{ValidationConfig, validate_provider_specs_batch};

// Strict validation for production
let config = ValidationConfig::strict();

// Development validation with error limiting  
let config = ValidationConfig::partial()
    .with_fail_fast()
    .with_max_errors(10);

// Basic schema-only validation
let config = ValidationConfig::basic();
```

## Error Messages

Validation errors include:
- **JSONPath**: Exact location using JSONPath notation (e.g., `$.models[0].endpoints.protocol`)
- **Description**: Human-readable explanation of the validation failure
- **Context**: Additional information about the validation rule that failed

## Integration Points

### Schema Loader
- Validates during `load_schema()` when `validate_basic_structure: true`
- Supports environment variable expansion with validation
- Caching layer preserves validation results

### Linter Integration  
- Batch validation for multiple specification files
- Configurable error reporting and collection
- IDE integration via language server protocol

## Performance Considerations

- Schema compilation is cached for repeated validations
- JSONPath expressions are pre-validated and cached
- Batch operations optimize validator creation overhead
- Custom rule evaluation scales O(n) with document size