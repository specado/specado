# Specado Code Style and Conventions

## General Rust Conventions
- Follow standard Rust naming conventions (snake_case for functions/variables, CamelCase for types)
- Use `rustfmt` for automatic formatting
- Apply `clippy` recommendations for idiomatic Rust

## Documentation Style
- Module-level documentation with `//!` at the top of files
- Include copyright notice: `Copyright (c) 2025 Specado Team`
- License header: `Licensed under the Apache-2.0 license`
- Comprehensive doc comments for public APIs with examples
- Use `/// # Example` sections in documentation

## Error Handling
- Use `thiserror` for custom error types with derive macros
- Use `anyhow` for flexible error handling in applications
- Return `Result<T>` types with descriptive error messages
- Include context in errors with field names and expected values

## Module Organization
- Clear separation of concerns with submodules
- Public exports at module level using `pub use`
- Test modules with `#[cfg(test)]` attribute
- Integration tests in separate test files

## Testing Conventions
- Unit tests in `mod tests` blocks
- Property-based tests using `proptest` framework
- Golden tests for translation validation
- Benchmark tests using `criterion`
- Test helper functions prefixed with `create_test_`

## Type System Usage
- Strong typing with explicit type definitions
- Use of Arc<Mutex<>> for shared state
- Builder pattern for complex object construction
- Extensive use of enums for state representation

## Logging
- Use `log` crate macros (debug!, info!, warn!, error!)
- Include context in log messages
- Log at appropriate levels based on severity

## Performance Considerations
- Track operation timings (e.g., translation duration)
- Use `Instant` for performance measurements
- Implement benchmarks for critical paths