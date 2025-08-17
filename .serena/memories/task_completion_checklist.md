# Task Completion Checklist for Specado

When completing any development task, follow this checklist to ensure code quality:

## 1. Code Formatting
Run `cargo fmt --all` to ensure consistent formatting across the codebase.

## 2. Linting
Run `cargo clippy --all --all-targets` to catch common mistakes and improve code quality.
- Fix all warnings unless there's a valid reason to suppress them
- Use `#[allow(...)]` attributes sparingly and with justification

## 3. Testing
Run `cargo test --all` to ensure all tests pass:
- Unit tests
- Integration tests
- Property-based tests
- Golden tests (if relevant to changes)

## 4. Build Verification
Run `cargo build --all` to ensure the project compiles without errors.

## 5. Documentation
- Ensure all public APIs have documentation comments
- Update examples if API changes were made
- Run `cargo doc --all` to verify documentation builds

## 6. Git Hygiene
Before committing:
- Review changes with `git diff`
- Stage appropriate files with `git add`
- Write clear commit messages following conventional format:
  - `feat:` for new features
  - `fix:` for bug fixes
  - `docs:` for documentation
  - `test:` for test additions/changes
  - `refactor:` for code refactoring
  - `chore:` for maintenance tasks

## Quick Command Sequence
```bash
cargo fmt --all && \
cargo clippy --all --all-targets && \
cargo test --all && \
cargo build --all
```

## Additional Checks for Major Changes
- Run benchmarks if performance-critical code was modified: `cargo bench`
- Update CHANGELOG if applicable
- Ensure backward compatibility or document breaking changes
- Consider adding property-based tests for complex logic