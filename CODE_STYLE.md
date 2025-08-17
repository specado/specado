# Specado Code Style Conventions

## File Format Requirements
- **Trailing Newlines**: All source files must end with a trailing newline character
- Ensures consistent file formatting and proper git diff behavior

## Logging Standards
- **Use log crate**: Always use the `log` crate for all logging operations
  - `log::info!()` for informational messages
  - `log::warn!()` for warnings and non-critical issues
  - `log::debug!()` for debug information
  - `log::error!()` for error conditions
- **Avoid eprintln!**: Do not use `eprintln!` for logging in production code
- **Structured logging**: Prefer structured log messages with consistent formatting

## Error Handling
- **Avoid unwrap() in production**: Never use `.unwrap()` in production code paths
  - Use proper error handling with `Result` types
  - Use `.expect()` with descriptive messages only when failure is truly impossible
  - Use `?` operator for error propagation
- **unwrap() in tests**: `.unwrap()` is acceptable in test code where panics are expected behavior

## Implementation Guidelines
- These conventions apply to all Rust source files (.rs)
- Use automated tooling (clippy, rustfmt) to enforce where possible
- Code reviews should verify adherence to these standards