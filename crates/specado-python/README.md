# Specado Python Bindings

[![PyPI version](https://badge.fury.io/py/specado.svg)](https://badge.fury.io/py/specado)
[![Python versions](https://img.shields.io/pypi/pyversions/specado.svg)](https://pypi.org/project/specado/)
[![License](https://img.shields.io/pypi/l/specado.svg)](https://github.com/specado/specado/blob/main/LICENSE)

Python bindings for [Specado](https://www.specado.com) - Universal LLM Specification Language.

## Overview

Specado provides a universal specification language for Large Language Models (LLMs), enabling seamless translation between different provider APIs. This Python package offers idiomatic Python bindings with full type hints, async support, and comprehensive error handling.

## Features

- 🔄 **Universal Translation**: Convert prompts between different LLM providers
- 📝 **Schema Validation**: Validate prompt and provider specifications
- 🚀 **Async Support**: Both synchronous and asynchronous execution
- 🔒 **Type Safety**: Complete type hints with mypy compatibility  
- ⚡ **High Performance**: Built with Rust for optimal speed
- 🛡️ **Error Handling**: Comprehensive exception types and context
- 🐍 **Pythonic API**: Idiomatic Python design patterns

## Installation

```bash
pip install specado
```

### Requirements

- Python 3.8+
- No additional dependencies required

### Development Installation

```bash
# Clone the repository
git clone https://github.com/specado/specado.git
cd specado/crates/specado-python

# Install in development mode
pip install -e ".[dev]"

# Run tests
pytest
```

## Quick Start

### Basic Usage

```python
import specado

# Create a prompt specification
prompt = specado.PromptSpec(
    model_class="Chat",
    messages=[
        specado.Message("system", "You are a helpful assistant."),
        specado.Message("user", "What's the weather like?")
    ],
    strict_mode="warn"
)

# Load a provider specification (OpenAI example)
provider_spec = specado.ProviderSpec.from_dict({
    "spec_version": "1.0.0",
    "provider": {
        "name": "openai",
        "base_url": "https://api.openai.com",
        "headers": {"Authorization": "Bearer YOUR_API_KEY"}
    },
    "models": [...] # Model specifications
})

# Translate the prompt to provider-specific format
result = specado.translate(
    prompt=prompt,
    provider_spec=provider_spec,
    model_id="gpt-4",
    mode="standard"
)

# Execute the request (async)
response = await specado.run(
    request=result.provider_request_json,
    provider_spec=provider_spec,
    timeout=30
)

print(response.content)
```

### Synchronous Usage

```python
import specado

# For synchronous operations, use run_sync instead
response = specado.run_sync(
    request=result.provider_request_json,
    provider_spec=provider_spec,
    timeout=30
)

print(f"Model: {response.model}")
print(f"Content: {response.content}")
print(f"Finish Reason: {response.finish_reason}")
```

### Validation

```python
import specado

# Validate a prompt specification
prompt_dict = {
    "model_class": "Chat",
    "messages": [
        {"role": "user", "content": "Hello, world!"}
    ],
    "strict_mode": "warn"
}

validation_result = specado.validate(prompt_dict, "prompt")

if validation_result.is_valid:
    print("✅ Prompt is valid")
else:
    print("❌ Validation errors:")
    for error in validation_result.errors:
        print(f"  - {error}")
```

### Advanced Features

#### Complex Prompts with Tools

```python
import specado

# Define a tool for function calling
weather_tool = specado.Tool(
    name="get_weather",
    description="Get current weather for a location",
    json_schema={
        "type": "object",
        "properties": {
            "location": {
                "type": "string",
                "description": "City name"
            },
            "units": {
                "type": "string",
                "enum": ["celsius", "fahrenheit"],
                "default": "celsius"
            }
        },
        "required": ["location"]
    }
)

# Create a complex prompt with tools and sampling parameters
complex_prompt = specado.PromptSpec(
    model_class="Chat",
    messages=[
        specado.Message("system", "You are a weather assistant."),
        specado.Message("user", "What's the weather in San Francisco?")
    ],
    tools=[weather_tool],
    tool_choice="auto",
    sampling=specado.SamplingParams(
        temperature=0.7,
        top_p=0.9,
        frequency_penalty=0.1
    ),
    limits=specado.Limits(
        max_output_tokens=1000,
        reasoning_tokens=500
    ),
    strict_mode="error"
)
```

#### Error Handling

```python
import specado
from specado import SpecadoError, TranslationError, ValidationError, ProviderError, TimeoutError

try:
    result = specado.translate(prompt, provider_spec, "gpt-4")
    response = await specado.run(result.provider_request_json, provider_spec)
    
except ValidationError as e:
    print(f"Validation failed: {e}")
except TranslationError as e:
    print(f"Translation failed: {e}")
except ProviderError as e:
    print(f"Provider error: {e}")
except TimeoutError as e:
    print(f"Request timed out: {e}")
except SpecadoError as e:
    print(f"General Specado error: {e}")
```

#### Working with Dictionaries

```python
import specado

# You can work with dictionaries directly
prompt_dict = {
    "model_class": "Chat",
    "messages": [
        {"role": "user", "content": "Hello!"}
    ],
    "strict_mode": "warn"
}

provider_dict = {
    "spec_version": "1.0.0",
    "provider": {"name": "openai", "base_url": "https://api.openai.com", "headers": {}},
    "models": [...]
}

# Create objects from dictionaries
prompt = specado.PromptSpec.from_dict(prompt_dict)
provider = specado.ProviderSpec.from_dict(provider_dict)

# Convert objects back to dictionaries
prompt_dict = prompt.to_dict()
provider_dict = provider.to_dict()
```

## API Reference

### Core Functions

#### `translate(prompt, provider_spec, model_id, mode="standard")`

Translate a prompt to a provider-specific request.

**Parameters:**
- `prompt` (PromptSpec): The prompt specification to translate
- `provider_spec` (ProviderSpec): The provider specification
- `model_id` (str): The model identifier to use
- `mode` (str): Translation mode ("standard" or "strict")

**Returns:**
- `TranslationResult`: The translated provider request with lossiness information

**Raises:**
- `TranslationError`: If translation fails
- `ValidationError`: If input validation fails
- `ProviderError`: If provider or model is not found

#### `validate(spec, schema_type)`

Validate a specification against its schema.

**Parameters:**
- `spec` (Any): The specification to validate (PromptSpec, ProviderSpec, or dict)
- `schema_type` (Literal["prompt", "provider"]): The type of schema to validate against

**Returns:**
- `ValidationResult`: Validation result with errors if any

**Raises:**
- `ValidationError`: If schema type is invalid or validation setup fails

#### `run(request, provider_spec, timeout=None)` (async)

Run a provider request asynchronously.

**Parameters:**
- `request` (dict): The provider request to execute
- `provider_spec` (ProviderSpec): The provider specification
- `timeout` (Optional[int]): Timeout in seconds (None for default)

**Returns:**
- `UniformResponse`: The normalized response from the provider

**Raises:**
- `ProviderError`: If the provider request fails
- `TimeoutError`: If the request times out
- `NetworkError`: If there are network issues

#### `run_sync(request, provider_spec, timeout=None)`

Run a provider request synchronously.

**Parameters:** Same as `run()`
**Returns:** Same as `run()`
**Raises:** Same as `run()`

### Type Classes

#### `PromptSpec`

Represents a universal prompt specification.

**Constructor:**
```python
PromptSpec(
    model_class: str,
    messages: List[Message],
    tools: Optional[List[Tool]] = None,
    tool_choice: Optional[ToolChoice] = None,
    response_format: Optional[ResponseFormat] = None,
    sampling: Optional[SamplingParams] = None,
    limits: Optional[Limits] = None,
    media: Optional[MediaConfig] = None,
    strict_mode: Literal["warn", "error"] = "warn"
)
```

**Methods:**
- `to_dict() -> Dict[str, Any]`: Convert to dictionary
- `from_dict(data: Dict[str, Any]) -> PromptSpec`: Create from dictionary

#### `ProviderSpec`

Represents a provider specification.

**Constructor:**
```python
ProviderSpec(
    spec_version: str,
    provider: ProviderInfo,
    models: List[ModelSpec]
)
```

**Methods:**
- `to_dict() -> Dict[str, Any]`: Convert to dictionary
- `from_dict(data: Dict[str, Any]) -> ProviderSpec`: Create from dictionary

#### `Message`

Represents a conversation message.

**Constructor:**
```python
Message(
    role: Literal["system", "user", "assistant"],
    content: str,
    name: Optional[str] = None,
    metadata: Optional[Dict[str, Any]] = None
)
```

#### `Tool`

Represents a tool definition for function calling.

**Constructor:**
```python
Tool(
    name: str,
    json_schema: Dict[str, Any],
    description: Optional[str] = None
)
```

### Response Types

#### `TranslationResult`

Result of a translation operation.

**Properties:**
- `provider_request_json: Dict[str, Any]`: Provider-specific request JSON
- `has_lossiness: bool`: Whether the translation has any lossiness

#### `UniformResponse`

Normalized response from a provider.

**Properties:**
- `model: str`: Model identifier
- `content: str`: Response content
- `finish_reason: str`: Reason for finishing
- `tool_calls: Optional[List[ToolCall]]`: Tool calls if any

#### `ValidationResult`

Result of a validation operation.

**Properties:**
- `is_valid: bool`: Whether the validation passed
- `errors: List[str]`: List of validation errors

### Exception Types

- `SpecadoError`: Base exception for all Specado errors
- `TranslationError`: Error during translation
- `ValidationError`: Error during validation
- `ProviderError`: Error from provider
- `TimeoutError`: Timeout error

## Performance

The Python bindings are built with Rust and optimized for performance:

- **Translation**: ~1-5ms per operation
- **Validation**: ~0.1-1ms per operation  
- **Serialization**: ~0.5-2ms for typical payloads
- **Memory**: Minimal overhead with efficient FFI layer

## Type Hints

This package provides complete type hints and is fully compatible with mypy:

```bash
# Type check your code
mypy your_code.py

# The package includes py.typed marker for type checker discovery
```

## Testing

```bash
# Install test dependencies
pip install -e ".[test]"

# Run all tests
pytest

# Run with coverage
pytest --cov=specado

# Run specific test categories
pytest -m "not slow"  # Skip slow tests
pytest -m benchmark   # Only benchmark tests
```

## Contributing

We welcome contributions! Please see our [Contributing Guide](https://github.com/specado/specado/blob/main/CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone repository
git clone https://github.com/specado/specado.git
cd specado/crates/specado-python

# Install development dependencies
pip install -e ".[dev]"

# Install pre-commit hooks
pre-commit install

# Run tests
pytest

# Format code
black src tests
ruff check src tests --fix

# Type check
mypy src
```

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](https://github.com/specado/specado/blob/main/LICENSE) file for details.

## Links

- **Website**: [specado.com](https://www.specado.com)
- **Documentation**: [docs.specado.com](https://docs.specado.com)
- **GitHub**: [github.com/specado/specado](https://github.com/specado/specado)
- **PyPI**: [pypi.org/project/specado](https://pypi.org/project/specado/)
- **Issues**: [github.com/specado/specado/issues](https://github.com/specado/specado/issues)

## Support

- 📧 **Email**: [support@specado.com](mailto:support@specado.com)
- 💬 **Issues**: [GitHub Issues](https://github.com/specado/specado/issues)
- 📖 **Documentation**: [docs.specado.com](https://docs.specado.com)