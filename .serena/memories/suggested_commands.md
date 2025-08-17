# Specado Development Commands

## Build Commands
```bash
# Build all crates in the workspace
cargo build --all

# Build in release mode with optimizations
cargo build --release --all

# Check code without building (faster)
cargo check --all
```

## Testing Commands
```bash
# Run all tests
cargo test --all

# Run tests with output
cargo test --all -- --nocapture

# Run specific test
cargo test test_name

# Run golden tests (integration tests)
cargo test --test golden_tests

# Run property-based tests
cargo test prop_
```

## Code Quality Commands
```bash
# Format code using rustfmt
cargo fmt --all

# Check formatting without applying
cargo fmt --all -- --check

# Run clippy linter
cargo clippy --all

# Run clippy with all targets (including tests)
cargo clippy --all --all-targets
```

## Benchmarking
```bash
# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench jsonpath_benchmarks
```

## Documentation
```bash
# Build documentation
cargo doc --all

# Build and open documentation in browser
cargo doc --all --open

# Build documentation with private items
cargo doc --all --document-private-items
```

## Git Commands (Darwin/macOS)
```bash
# Check status
git status

# Stage changes
git add -A

# Commit with message
git commit -m "feat: description"

# View diff
git diff

# View staged diff
git diff --staged
```

## System Utilities (Darwin/macOS)
```bash
# List files
ls -la

# Find files
find . -name "*.rs"

# Search in files (using ripgrep if available)
rg "pattern" --type rust

# Or using grep
grep -r "pattern" --include="*.rs"
```

## Workspace Management
```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Check for outdated dependencies
cargo outdated
```

## Quick Validation Workflow
When completing a task, run these commands in sequence:
```bash
cargo fmt --all
cargo clippy --all --all-targets
cargo test --all
cargo build --all
```