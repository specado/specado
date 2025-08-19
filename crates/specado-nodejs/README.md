# Specado Node.js Bindings

High-performance Node.js bindings for [Specado](https://www.specado.com), the universal LLM prompt translation and execution library.

## Features

- **Universal Prompt Translation**: Convert between different LLM provider formats
- **Provider Validation**: Validate prompt and provider specifications
- **Async Execution**: Execute provider requests with full async/await support
- **TypeScript Support**: Comprehensive TypeScript definitions included
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **High Performance**: Built with Rust and NAPI-RS for optimal performance

## Installation

```bash
npm install @specado/nodejs
```

## Quick Start

```typescript
import { translate, validate, run, SchemaType } from '@specado/nodejs';

// Define a prompt
const prompt = {
  model_class: "Chat",
  messages: [
    { role: "user", content: "Hello, world!" }
  ],
  strict_mode: "standard"
};

// Validate the prompt
const validation = validate(prompt, SchemaType.Prompt);
if (!validation.valid) {
  console.error('Validation errors:', validation.errors);
  process.exit(1);
}

// Define provider specification
const providerSpec = {
  name: "openai",
  version: "1.0.0",
  base_url: "https://api.openai.com/v1",
  auth: {
    auth_type: "bearer",
    header: "Authorization",
    env_var: "OPENAI_API_KEY"
  },
  models: [
    {
      id: "gpt-4",
      name: "GPT-4",
      capabilities: ["chat", "tools"],
      context_size: 128000,
      max_output: 4096
    }
  ],
  mappings: {
    request: {
      model: "{{ model_id }}",
      messages: "{{ messages }}"
    },
    response: {
      content: "{{ choices[0].message.content }}"
    }
  }
};

// Translate prompt to provider format
const translation = translate(prompt, providerSpec, "gpt-4");
console.log('Translated request:', translation.request);

// Execute the request
const response = await run({
  provider: providerSpec,
  request: translation.request,
  credentials: { api_key: process.env.OPENAI_API_KEY }
}, providerSpec, {
  timeout_seconds: 30
});

console.log('Response:', response.content);
console.log('Usage:', response.usage);
```

## API Reference

### Core Functions

#### `translate(prompt, providerSpec, modelId, options?)`

Translates a universal prompt to a provider-specific request format.

**Parameters:**
- `prompt` (PromptSpec): The universal prompt specification
- `providerSpec` (ProviderSpec): Target provider specification
- `modelId` (string): Target model identifier
- `options` (TranslateOptions, optional): Translation options

**Returns:** `TranslateResult`

#### `validate(spec, schemaType)`

Validates a specification against its schema.

**Parameters:**
- `spec` (any): The specification to validate
- `schemaType` (SchemaType): Type of schema (Prompt or Provider)

**Returns:** `ValidationResult`

#### `run(request, providerSpec, options?)`

Executes a provider request asynchronously.

**Parameters:**
- `request` (ProviderRequest): The request to execute
- `providerSpec` (ProviderSpec): Provider specification
- `options` (RunOptions, optional): Execution options

**Returns:** `Promise<UniformResponse>`

### Utility Functions

#### `init()`

Initializes the Specado library.

#### `getVersion()`

Returns the library version string.

#### `getVersionInfo()`

Returns detailed version information.

## Types

### PromptSpec

```typescript
interface PromptSpec {
  model_class: string;
  messages: Message[];
  tools?: Tool[];
  tool_choice?: ToolChoice;
  response_format?: ResponseFormat;
  sampling?: SamplingParams;
  limits?: Limits;
  media?: MediaConfig;
  strict_mode: string;
}
```

### ProviderSpec

```typescript
interface ProviderSpec {
  name: string;
  version: string;
  base_url: string;
  auth: AuthConfig;
  models: ModelSpec[];
  rate_limits?: RateLimitConfig;
  mappings: ProviderMappings;
}
```

### TranslateResult

```typescript
interface TranslateResult {
  request: any;
  metadata: TranslationMetadata;
  warnings: string[];
}
```

### UniformResponse

```typescript
interface UniformResponse {
  content: string;
  usage?: UsageStats;
  metadata: ResponseMetadata;
  tool_calls?: ToolCall[];
  finish_reason?: string;
}
```

## Error Handling

The library provides structured error handling with specific error types:

```typescript
import { SpecadoErrorKind } from '@specado/nodejs';

try {
  const result = translate(prompt, providerSpec, modelId);
} catch (error) {
  if (error.kind === SpecadoErrorKind.InvalidInput) {
    console.error('Invalid input:', error.message);
  } else if (error.kind === SpecadoErrorKind.ProviderNotFound) {
    console.error('Provider not found:', error.details);
  }
}
```

### Error Types

- `InvalidInput`: Invalid parameters provided
- `JsonError`: JSON parsing or serialization error
- `ProviderNotFound`: Specified provider not found
- `ModelNotFound`: Specified model not found
- `NetworkError`: Network communication error
- `AuthenticationError`: Authentication failure
- `RateLimitError`: Rate limit exceeded
- `TimeoutError`: Operation timed out
- `InternalError`: Internal library error

## Advanced Usage

### Custom Translation Options

```typescript
const options = {
  mode: "strict",
  include_metadata: true,
  custom_rules: {
    // Custom transformation rules
  }
};

const result = translate(prompt, providerSpec, modelId, options);
```

### Execution with Retries

```typescript
const runOptions = {
  timeout_seconds: 60,
  max_retries: 3,
  follow_redirects: true,
  user_agent: "MyApp/1.0"
};

const response = await run(request, providerSpec, runOptions);
```

### Tool Usage

```typescript
const promptWithTools = {
  model_class: "Chat",
  messages: [
    { role: "user", content: "What's the weather like?" }
  ],
  tools: [
    {
      name: "get_weather",
      description: "Get current weather",
      parameters: {
        type: "object",
        properties: {
          location: { type: "string" }
        },
        required: ["location"]
      }
    }
  ],
  tool_choice: { choice_type: "auto" },
  strict_mode: "standard"
};
```

## Performance

The library is built with Rust and NAPI-RS for optimal performance:

- **Fast Translation**: Sub-millisecond prompt translation
- **Concurrent Execution**: Thread-safe concurrent request handling
- **Memory Efficient**: Minimal memory overhead and no leaks
- **Cross-Platform**: Optimized binaries for all platforms

## Development

### Building from Source

```bash
# Install dependencies
npm install

# Build the native module
npm run build

# Run tests
npm test

# Build for all platforms
npm run build:universal
```

### Testing

```bash
# Run unit tests
npm test

# Run with coverage
npm run test:coverage

# Run performance tests
npm run test:performance
```

## Platform Support

| Platform | Node.js 16+ | Node.js 18+ | Node.js 20+ |
|----------|-------------|-------------|-------------|
| Windows x64 | ✅ | ✅ | ✅ |
| Windows ARM64 | ✅ | ✅ | ✅ |
| macOS x64 | ✅ | ✅ | ✅ |
| macOS ARM64 | ✅ | ✅ | ✅ |
| Linux x64 | ✅ | ✅ | ✅ |
| Linux ARM64 | ✅ | ✅ | ✅ |

## License

Apache-2.0 License. See [LICENSE](LICENSE) for details.

## Support

- 📖 [Documentation](https://docs.specado.com)
- 🐛 [Issues](https://github.com/specado/specado/issues)
- 💬 [Discussions](https://github.com/specado/specado/discussions)
- 🌐 [Website](https://www.specado.com)

## Contributing

We welcome contributions! Please see our [Contributing Guide](../../CONTRIBUTING.md) for details.

---

Built with ❤️ by the Specado team.