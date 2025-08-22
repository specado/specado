# Specado

Spec-driven LLM translation engine that compiles uniform prompts into provider-native requests with transparent lossiness reporting.

## Project Structure

```
specado/
├── crates/
│   ├── specado-core/        # Core translation engine
│   ├── specado-schemas/     # JSON Schema definitions
│   ├── specado-cli/         # Command-line interface
│   └── specado-golden/      # Golden test infrastructure
├── schemas/                  # JSON Schema files
├── providers/                # Provider spec files
└── tests/                    # Test suites
```

## Guiding Principles for a Minimal & Robust Schema

### 1. If the Runtime Needs It, It's Core
If your core translation engine must read a field to function, it belongs in the core schema.

### 2. If It's Descriptive, It's an Extension
If a field provides metadata about capabilities that a human or a higher-level client might use, it belongs in extensions.

### 3. Validation Lives in the Loader
The schema validates the shape of the data. The loader validates the semantics (e.g., "is this JSONPath valid?").

## Documentation

- **[Provider Specification Schema](docs/provider-spec-v2-simplified.md)** - Complete guide to the provider specification format
- **[Validation Rules](docs/validation-rules.md)** - Comprehensive validation rules and error handling

## Development

```bash
# Build all crates
cargo build --all

# Run tests
cargo test --all
```


## License

This project is dual-licensed under MIT and Apache-2.0.