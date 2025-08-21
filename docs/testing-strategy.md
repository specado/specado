# Testing Strategy

## Flaky Test Quarantine

This project uses a multi-layered approach to handle flaky tests:

### 1. Test Categories

- **Stable Tests**: Run in CI by default (`cargo test`)
- **Ignored Tests**: Flaky tests marked with `#[ignore]`, run separately (`cargo test -- --ignored`) 
- **Feature-Gated Tests**: Tests behind `#[cfg(feature = "flaky")]`, excluded by default

### 2. Flaky Test Examples

#### Jitter Test (Random Behavior)
```rust
#[test]
#[ignore] // Flaky due to random jitter - run separately with: cargo test -- --ignored  
#[cfg(feature = "flaky")] // Also gated behind flaky feature for CI control
fn test_retry_delay_with_jitter() {
    // Tests random jitter behavior - inherently non-deterministic
}
```

**Solution**: Added deterministic test `test_retry_delay_exponential_base` without jitter.

#### Environment-Dependent Tests
```rust
#[test]
fn test_missing_api_key() {
    // Save original env var value for restoration
    let original_key = std::env::var("OPENAI_API_KEY").ok();
    
    // Ensure env var is not set
    std::env::remove_var("OPENAI_API_KEY");
    
    // ... test logic ...
    
    // Restore original environment state
    if let Some(key) = original_key {
        std::env::set_var("OPENAI_API_KEY", key);
    }
}
```

**Solution**: Explicit environment isolation with save/restore pattern.

### 3. CI Strategy

```bash
# Default CI run (stable tests only)
cargo test

# Separate flaky test job (optional)  
cargo test --features flaky -- --ignored

# Development (run all)
cargo test --features flaky -- --include-ignored
```

### 4. Safety Net

- `cargo check` always runs to catch compilation errors
- Core unit tests remain fast and reliable
- Integration tests use deterministic fixtures
- Environment-dependent tests use proper isolation

This approach ensures CI reliability while maintaining comprehensive test coverage for development.