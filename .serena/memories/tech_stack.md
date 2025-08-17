# Specado Tech Stack

## Primary Language
- **Rust** (stable channel)
- Edition: 2021
- Workspace-based monorepo structure

## Core Dependencies
- **serde**: Serialization/deserialization framework
- **serde_json**: JSON handling
- **thiserror**: Error handling with derive macros
- **anyhow**: Flexible error handling
- **chrono**: Date/time handling
- **log**: Logging framework
- **regex**: Regular expression support

## Testing Dependencies
- **criterion**: Benchmarking framework
- **proptest**: Property-based testing (version 1.4)
- **specado-golden**: Golden test infrastructure for translation validation

## Build Tools
- **cargo**: Rust package manager and build tool
- **rustfmt**: Code formatting
- **clippy**: Rust linter

## Project Structure
- Workspace with multiple crates:
  - `specado-core`: Core translation engine
  - `specado-schemas`: JSON Schema definitions
  - `specado-providers`: Provider specifications
  - `specado-cli`: Command-line interface
  - `specado-ffi`: FFI bindings (Node.js/Python)
  - `specado-wasm`: WebAssembly bindings
  - `specado-golden`: Golden test infrastructure

## License
- Dual-licensed: MIT OR Apache-2.0
- **Important**: Always use Apache-2.0 license, never MIT alone