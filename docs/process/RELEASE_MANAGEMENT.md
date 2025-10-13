# Release Management Guide

This document provides comprehensive instructions for releasing Specado packages to npm, Test PyPI, PyPI, and crates.io. It covers version management, automated publishing workflows, and secure token handling.

## Table of Contents

1. [Version Strategy](#version-strategy)
2. [Prerequisites](#prerequisites)
3. [Environment Setup](#environment-setup)
4. [Release Workflow](#release-workflow)
5. [Manual Publishing](#manual-publishing)
6. [Verification](#verification)
7. [Troubleshooting](#troubleshooting)

## Version Strategy

Specado follows [Semantic Versioning](https://semver.org/) with the following conventions:

- **Starting Version**: `0.2.0-alpha.1` (as requested)
- **Pre-releases**: `0.2.0-alpha.1`, `0.2.0-beta.1`, `0.2.0-rc.1`
- **Stable releases**: `0.2.0`, `0.3.0`, `1.0.0`
- **Breaking changes**: Increment major version
- **New features**: Increment minor version
- **Bug fixes**: Increment patch version

All packages (Rust crates, npm, PyPI) maintain synchronized versions.

## Prerequisites

### Required Software
- Rust 1.75+ with Cargo
- Node.js 18+
- Python 3.9+
- `maturin` (for Python bindings): `pip install maturin`
- `twine` (for PyPI publishing): `pip install twine`

### Required Tokens and Permissions

Store these securely in your environment or secure credential store:

#### npm Token
- **Token**: `npm_[your_npm_token_here]`
- **Scope**: Publishing to `specado` package
- **Setup**: Automation token from npmjs.com

#### Test PyPI Token
- **Token**: `pypi-[your_test_pypi_token_here]`
- **Scope**: Publishing to Test PyPI

#### PyPI Token
- **Token**: `pypi-[your_production_pypi_token_here]`
- **Scope**: Publishing to production PyPI

#### crates.io Token
- **Token**: `cio[your_crates_io_token_here]`
- **Scopes**: `publish-new`, `publish-update`, `yank`

## Environment Setup

### 1. Token Configuration

Create secure token storage. Choose one approach:

#### Option A: Environment Variables (Recommended for CI)
```bash
export NPM_TOKEN="npm_[your_npm_token_here]"
export TEST_PYPI_TOKEN="pypi-[your_test_pypi_token_here]"
export PYPI_TOKEN="pypi-[your_production_pypi_token_here]"
export CRATES_IO_TOKEN="cio[your_crates_io_token_here]"
```

#### Option B: Local Configuration Files

Create a `.env` file in the project root (this file is gitignored):
```bash
NPM_TOKEN=npm_[your_npm_token_here]
TEST_PYPI_TOKEN=pypi-[your_test_pypi_token_here]
PYPI_TOKEN=pypi-[your_production_pypi_token_here]
CRATES_IO_TOKEN=cio[your_crates_io_token_here]
```

**Note**: Never commit `.env` files containing real tokens to version control.

### 2. GitHub Repository Setup

Configure OpenID Connect publishers in GitHub repository settings:

#### npm Publisher
- **Repository**: specado/specado
- **Workflow**: `release.yml`
- **Environment**: `(Any)`

#### PyPI Publisher
- **Repository**: specado/specado
- **Workflow**: `release.yml`
- **Environment**: `(Any)`

### 3. GitHub Actions Secrets

Add these secrets to your GitHub repository:

- `NPM_TOKEN`: Your npm automation token
- `TEST_PYPI_TOKEN`: Test PyPI API token
- `PYPI_TOKEN`: Production PyPI API token
- `CRATES_IO_TOKEN`: crates.io API token

## Release Workflow

### Automated Release (Recommended)

1. **Create Release Branch**
   ```bash
   git checkout -b release/v0.2.0-alpha.1
   ```

2. **Update Versions**
   ```bash
   # Update workspace version
   sed -i 's/version = "0\.1\.0"/version = "0.2.0-alpha.1"/g' Cargo.toml

   # Update Node.js package
   sed -i 's/"version": "0\.1\.0"/"version": "0.2.0-alpha.1"/g' crates/specado-node/package.json

   # Update Python package
   sed -i 's/version = "0\.1\.0"/version = "0.2.0-alpha.1"/g' pyproject.toml
   ```

3. **Commit Version Changes**
   ```bash
   git add -A
   git commit -m "chore: bump version to 0.2.0-alpha.1"
   ```

4. **Push and Create PR**
   ```bash
   git push origin release/v0.2.0-alpha.1
   gh pr create --title "Release v0.2.0-alpha.1" --body "Automated release preparation"
   ```

5. **Create GitHub Release**
   After PR merge, create a GitHub release with tag `v0.2.0-alpha.1`. The release workflow will automatically:
   - Build all packages
   - Publish to Test PyPI
   - Publish to npm
   - Publish to crates.io
   - Optionally publish to production PyPI (manual trigger)

### Manual Publishing

If automated workflow fails, use these manual steps:

#### 1. Publish to Test PyPI

```bash
# Build Python wheels
maturin build -m crates/specado-py/Cargo.toml --release --out dist

# Publish to Test PyPI
TWINE_USERNAME=__token__ TWINE_PASSWORD=$TEST_PYPI_TOKEN twine upload --repository testpypi dist/*

# Verify installation
python -m venv test-env
source test-env/bin/activate
pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple specado
python -c "import specado; print(f'Version: {specado.__version__}')"
```

#### 2. Publish to npm

```bash
cd crates/specado-node

# Build native binaries
npm run build

# Test locally
npm test

# Publish to npm
echo "//registry.npmjs.org/:_authToken=$NPM_TOKEN" > ~/.npmrc
npm publish

# Verify
npm info specado
```

#### 3. Publish to crates.io

```bash
# Authenticate
cargo login $CRATES_IO_TOKEN

# Test build
cargo test --workspace

# Publish crates in dependency order
cargo publish -p specado-core
cargo publish -p specado-schemas
cargo publish -p specado-providers
cargo publish -p specado-cli

# Wait for propagation, then publish Python/Node if needed
cargo publish -p specado-py
cargo publish -p specado-node
```

#### 4. Publish to Production PyPI

```bash
# Build final wheels
maturin build -m crates/specado-py/Cargo.toml --release --out dist

# Publish to production PyPI
TWINE_USERNAME=__token__ TWINE_PASSWORD=$PYPI_TOKEN twine upload dist/*

# Verify
pip install specado
python -c "import specado; print(f'Version: {specado.__version__}')"
```

## Verification

### Post-Publish Checks

#### npm Verification
```bash
# Check package info
npm info specado

# Test installation
mkdir test-npm && cd test-npm
npm init -y
npm install specado

# Test CLI
npx specado --help

# Test Node.js API
node -e "const { Specado } = require('specado'); console.log('Import successful');"
```

#### PyPI Verification
```bash
# Test PyPI
pip install --index-url https://test.pypi.org/simple specado
python -c "import specado; print(specado.__version__)"

# Production PyPI
pip install specado
python -m specado.cli --help
```

#### crates.io Verification
```bash
# Test installation
cargo install specado-cli
specado --help

# Test library usage
cargo new test-specado
cd test-specado
echo 'specado = "0.2.0-alpha.1"' >> Cargo.toml
cargo build
```

### Cleaning Up PyPI / Test PyPI Releases

If a pre-release needs to be retracted, use the helper script to *yank* the release
while leaving its files for historical reference. Tokens from
`~/.config/specado/.env` are loaded automatically:

```bash
# Remove versions from Test PyPI
PYTHON=$(pyenv which python) ./scripts/remove_pypi_versions.sh testpypi 0.2.0a16 0.2.0a17

# Remove versions from production PyPI
PYTHON=$(pyenv which python) ./scripts/remove_pypi_versions.sh pypi 0.2.0a18
```

The script relies on `~/.config/specado/.pypirc` and `python -m twine`. It exits on
the first failure so you can rerun it safely once credentials or 2FA tokens are refreshed.
Set `YANK_REASON="some message"` to customize the yank reason.

### Integration Testing

Run the example scripts to verify end-to-end functionality:

```bash
# Test CLI examples
./examples/cli_demo.sh
./examples/cli_preview.sh

# Test Node.js examples
node examples/node_basic.mjs

# Test Python examples
python examples/python_basic.py
```

## GitHub Actions Workflow

The automated release workflow (`.github/workflows/release.yml`) handles:

### Triggers
- GitHub release creation with tag pattern `v*`
- Manual workflow dispatch for testing

### Jobs
1. **Test**: Run full test suite
2. **Build**: Build all packages
3. **Publish-TestPyPI**: Publish Python package to Test PyPI
4. **Publish-npm**: Publish Node.js package to npm
5. **Publish-crates**: Publish Rust crates to crates.io
6. **Publish-PyPI**: Manual trigger for production PyPI

### Environment Variables
- `CARGO_TERM_COLOR`: `always`
- `RUST_BACKTRACE`: `1`

### Permissions
- `contents`: `read`
- `id-token`: `write` (for OIDC publishing)

## Troubleshooting

### Common Issues

#### npm Publishing Fails
```bash
# Check token
npm whoami

# Check package.json
cat crates/specado-node/package.json | grep version

# Manual publish test
cd crates/specado-node
npm publish --dry-run
```

#### PyPI Publishing Fails
```bash
# Check token format
echo $TEST_PYPI_TOKEN | head -c 20

# Check wheel contents
tar -tf dist/*.whl

# Test upload to test instance first
twine upload --repository testpypi --verbose dist/*
```

#### crates.io Publishing Fails
```bash
# Check authentication
cargo login --dry-run

# Check crate metadata
cargo package -p specado-cli --allow-dirty

# Check publish order - dependencies first
cargo tree -p specado-cli
```

#### Version Synchronization Issues
```bash
# Check all version files
grep -r "0\.2\.0" --include="*.toml" --include="*.json" .

# Update if needed
find . -name "*.toml" -o -name "*.json" | xargs grep -l "version" | xargs sed -i 's/0\.1\.0/0.2.0-alpha.1/g'
```

### Rollback Procedures

#### npm Rollback
```bash
# Unpublish (within 72 hours)
npm unpublish specado@0.2.0-alpha.1

# Deprecate if needed
npm deprecate specado@0.2.0-alpha.1 "Deprecated due to [reason]"
```

#### PyPI Rollback
```bash
# PyPI doesn't allow deletion, but you can yank
# Contact PyPI support for critical issues
```

#### crates.io Rollback
```bash
# Yank the release
cargo yank --vers 0.2.0-alpha.1 specado-cli

# Delete if within grace period (contact crates.io support)
```

## Release Checklist

- [ ] All tests pass (`cargo test --workspace`, `npm test`, `pytest`)
- [ ] Version bumped in all packages (Cargo.toml, package.json, pyproject.toml)
- [ ] Changelog updated
- [ ] Git tag created and pushed
- [ ] GitHub release created
- [ ] Automated workflow completes successfully
- [ ] Manual verification of all package installations
- [ ] Examples run successfully
- [ ] Documentation updated with new version

## Best Practices

1. **Test Releases**: Always publish pre-releases to Test PyPI first
2. **Version Sync**: Keep all package versions synchronized
3. **Token Security**: Never commit tokens to version control
4. **Rollback Plan**: Have rollback procedures ready
5. **Documentation**: Update docs after successful releases
6. **Communication**: Announce releases to stakeholders
7. **Monitoring**: Monitor for issues post-release

## Support

For issues with this release process:
1. Check this document first
2. Review GitHub Actions logs
3. Test manual publishing steps
4. Contact maintainers with specific error messages
