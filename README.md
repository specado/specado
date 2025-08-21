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

## Development

```bash
# Build all crates
cargo build --all

# Run tests
cargo test --all
```

## License

This project is dual-licensed under MIT and Apache-2.0.