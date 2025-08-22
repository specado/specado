# Provider Spec Schema v1.1 - Simplified Structure

## Overview

This document explains the simplified provider specification schema that introduces a "core + extensions" pattern to reduce complexity while preserving all existing functionality.

## Key Changes

### 1. Core + Extensions Pattern

The schema now distinguishes between **core fields** (actively used by the runtime) and **extended fields** (provider-specific or experimental features). Extended fields are moved to `extensions` objects at various levels:

- **Provider level**: `provider.extensions` for provider-wide experimental features
- **Tooling level**: `tooling.extensions` for additional tool capabilities  
- **Constraints level**: `constraints.extensions` for additional constraint configurations
- **Mappings level**: `mappings.extensions` for transformations and advanced mappings
- **Model level**: `extensions` for model-specific experimental features

### 2. Optional Authentication

The `provider.auth` object is now **optional**, addressing usability issues where authentication is handled externally or through other means.

```json
{
  "provider": {
    "name": "openai",
    "base_url": "https://api.openai.com",
    "headers": {
      "Authorization": "Bearer ${ENV:OPENAI_API_KEY}"
    }
    // auth is now optional
  }
}
```

### 3. Relaxed Validation

Several overly strict validations have been removed:

- **JSONPath patterns**: No longer restricted to basic patterns, allowing complex expressions like `$.choices[0].message.content`
- **Strategy enums**: `json_output.strategy` now accepts any string, not just predefined values
- **Parameter flexibility**: More flexible parameter definitions to support provider variations

### 4. Runtime-Aligned Structure

The schema now closely mirrors the Rust runtime structs, ensuring every required field is actually used:

#### Core Tooling Fields (Required)
- `tools_supported`
- `parallel_tool_calls_default`  
- `can_disable_parallel_tool_calls`
- `disable_switch` (optional)

#### Extended Tooling Fields (Moved to Extensions)
- `tool_types`
- `tool_choice_modes`
- `strict_tools_support`
- `context_free_grammars`
- `preambles_supported`
- `custom_tools`

## Extensions Philosophy

The `extensions` object serves as an "escape hatch" for:

1. **Experimental features** not yet standardized
2. **Provider-specific capabilities** that don't fit the common model
3. **Future functionality** without breaking existing schemas
4. **Migration path** for deprecated fields

### Important Notes about Extensions

- **Known anchors validated**: Common extension patterns like reasoning, transformations, rate_limits are validated via $defs; unknown keys are accepted
- **Flexibility**: Supports arbitrary nested structures via additionalProperties: true
- **Forward compatibility**: New features can be added without schema changes
- **Backwards compatibility**: Existing functionality preserved during migration

## Migration Examples

### Before (v1.0)
```json
{
  "tooling": {
    "tools_supported": true,
    "parallel_tool_calls_default": true,
    "can_disable_parallel_tool_calls": true,
    "tool_types": ["function", "custom"],
    "tool_choice_modes": ["auto", "none", "required"],
    "strict_tools_support": true
  }
}
```

### After (v1.1)
```json
{
  "tooling": {
    "tools_supported": true,
    "parallel_tool_calls_default": true,
    "can_disable_parallel_tool_calls": true,
    "extensions": {
      "tool_types": ["function", "custom"],
      "tool_choice_modes": ["auto", "none", "required"],
      "strict_tools_support": true
    }
  }
}
```

## Complete Example

See `providers/examples/minimal-openai.json` for a complete example demonstrating:

- Optional authentication with extension fallback
- Core vs extended tooling capabilities
- Flexible JSONPath expressions
- Extensions at multiple levels
- Provider-specific experimental features

## Benefits

1. **Reduced complexity**: Core schema focuses on runtime-required fields
2. **Better validation**: No false negatives from overly strict patterns
3. **Forward compatibility**: Easy to add new features via extensions
4. **Cleaner specs**: Core functionality is immediately visible
5. **Flexibility preserved**: All existing power available through extensions

## Validation Strategy

- **Core fields**: Strictly validated against the schema
- **Extensions**: Known anchors (e.g., reasoning, transformations, rate_limits) are validated via $defs; unknown keys are accepted
- **Optional fields**: Only validated if present
- **Runtime alignment**: Required fields match actual code usage

This approach ensures the schema serves as both a contract (for core functionality) and a flexible container (for extensions and experimentation).

### Extensions Philosophy

Extensions provide structured flexibility through a "light contract" approach:

- **Known anchors**: Common extension patterns like `reasoning`, `conversation_management`, `transformations` are typed and validated
- **Escape hatch preserved**: Unknown keys are accepted via `additionalProperties: true`
- **Provider-specific features**: Custom capabilities can be added without schema changes

## Canonical Strategy Values

The following strategy values are officially supported and validated:

### JSON Output Strategies
- `"response_format"` - Native JSON parameter (OpenAI-style)
- `"system_prompt"` - JSON output via system instructions (Anthropic-style)
- `"structured_output"` - Provider-specific structured response
- `"schema_guided"` - Schema-based output formatting

### Authentication Types
- `"bearer"` - Bearer token authentication
- `"api-key"` - API key header authentication  
- `"basic"` - Basic HTTP authentication
- `"custom"` - Custom authentication method

### Stream Protocols
- `"sse"` - Server-Sent Events
- `"websocket"` - WebSocket connection

## JSONPath Expression Guidelines

All `*_path` properties accept JSONPath expressions with the following requirements:

### Syntax Requirements
- JSONPath expressions with automatic root resolution: `path.to.field` or `$.path.to.field`
- Array indexing supported: `choices[0].message.content`
- Wildcard selectors allowed: `content[*].text`
- Filter expressions supported: `content[?(@.type == 'text')]`

### Common Patterns
```json
{
  "response_normalization": {
    "content_path": "choices[0].message.content",
    "tool_calls_path": "choices[0].message.tool_calls",
    "finish_reason_path": "choices[0].finish_reason",
    "usage_path": "usage"
  },
  "stream": {
    "event_selector": {
      "type_path": "choices[0].delta.type",
      "routes": [
        {
          "when": "content",
          "emit": "delta",
          "text_path": "choices[0].delta.content"
        }
      ]
    }
  }
}
```

### Validation Notes
- JSONPath expressions are flexible and support both simple paths and complex expressions
- Validation occurs during runtime when the paths are actually evaluated
- Complex expressions are cached for performance during evaluation
- Provider-specific path structures are supported through extensions

### Best Practices
1. **Use specific paths**: Prefer `choices[0].content` over `choices[*].content`
2. **Validate with sample data**: Test JSONPath expressions against real API responses
3. **Document custom paths**: Include comments or documentation for complex expressions
4. **Handle missing fields**: Consider optional paths for fields that may not always be present

## Stream Event Customization

The schema supports a `"custom"` emit type for specialized stream events that require additional metadata:

### Custom Emit Type Usage
```json
{
  "routes": [
    {
      "when": "thinking_block_delta",
      "emit": "custom",
      "text_path": "delta.text",
      "extensions": {
        "channel": "reasoning",
        "event_type": "thinking",
        "metadata": {
          "confidence": "high",
          "step_type": "analysis"
        }
      }
    }
  ]
}
```

### Extension Requirements
- **Required for custom emit**: When `emit: "custom"`, the `extensions` object must be present
- **Channel specification**: Use `extensions.channel` to categorize the event (e.g., "reasoning", "usage", "debug")
- **Event metadata**: Additional properties in `extensions` provide context for downstream processing
- **Downstream handling**: Applications can route custom events based on channel and metadata