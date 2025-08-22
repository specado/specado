# Specado Codebase Structure

## Root Directory
```
specado/
├── Cargo.toml              # Workspace configuration
├── README.md              # Project overview
├── rust-toolchain.toml    # Rust toolchain configuration
├── .gitignore            # Git ignore rules
└── ISSUE_37_SUMMARY.md   # Issue tracking

## Main Directories
├── crates/               # Rust crates (workspace members)
├── providers/            # Provider specification files
├── schemas/              # JSON Schema definitions
├── golden-corpus/        # Golden test data
├── tests/               # Integration tests
└── docs/                # Documentation

## Crates Structure (Actual)
crates/
├── specado-cli/         # Command-line interface
│   └── src/
│       ├── main.rs     # CLI entry point
│       ├── handlers/   # Command handlers
│       └── output/     # Output formatting

├── specado-core/        # Core translation engine
│   ├── src/
│   │   ├── lib.rs      # Main library entry
│   │   ├── error.rs    # Error types
│   │   ├── types.rs    # Core type definitions
│   │   ├── http/       # HTTP client module
│   │   ├── provider_discovery/ # Provider discovery
│   │   └── translation/ # Translation modules
│   │       ├── mod.rs
│   │       ├── builder.rs      # Result builder
│   │       ├── conflict.rs     # Conflict resolution
│   │       ├── context.rs      # Translation context
│   │       ├── jsonpath.rs     # JSONPath mapping
│   │       ├── lossiness.rs    # Lossiness tracking
│   │       ├── mapper.rs       # Field mapping
│   │       ├── strictness.rs   # Strictness policies
│   │       ├── transformer.rs  # Value transformations
│   │       └── validator.rs    # Validation logic
│   ├── providers/       # Provider implementations
│   │   ├── anthropic/  # Anthropic provider
│   │   └── openai/     # OpenAI provider
│   ├── examples/       # Example usage
│   ├── benches/        # Benchmarks
│   ├── fuzz/          # Fuzzing tests
│   └── tests/         # Integration tests

├── specado-schemas/     # JSON Schema handling
│   ├── src/
│   │   ├── documentation/  # Schema documentation
│   │   ├── loader/        # Schema loading
│   │   ├── validation/    # Schema validation
│   │   └── versioning/    # Version management
│   ├── docs/          # Schema documentation
│   ├── examples/      # Schema examples
│   └── tests/         # Schema tests

└── specado-golden/     # Golden test infrastructure
    └── src/
        ├── lib.rs      # Library exports
        ├── runner.rs   # Test runner
        └── snapshot.rs # Snapshot management
```

## Key Files
- `crates/specado-core/src/lib.rs`: Main library exports and public API
- `crates/specado-core/src/translation/mod.rs`: Translation engine implementation
- `crates/specado-core/src/types.rs`: Core type definitions (PromptSpec, ProviderSpec, etc.)

## Test Organization
- Unit tests: In `mod tests` blocks within source files
- Integration tests: In `tests/` directories
- Property tests: Files with `prop_` prefix
- Golden tests: In `golden-corpus/` with snapshot validation
- Benchmarks: In `benches/` directories

## Provider Specifications
Provider specs define how to translate to specific LLM APIs:
- Located in `providers/` directory
- JSON format with mapping rules
- Validated against schemas