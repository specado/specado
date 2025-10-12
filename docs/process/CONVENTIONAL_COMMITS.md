# Conventional Commits Guide

This project uses [Conventional Commits](https://conventionalcommits.org/) to automate versioning and release management. Following these guidelines ensures that releases are created automatically with proper semantic versioning.

## Commit Message Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types

- **feat**: A new feature (triggers **MINOR** version bump: `1.2.3` → `1.3.0`)
- **fix**: A bug fix (triggers **PATCH** version bump: `1.2.3` → `1.2.4`)
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code (formatting, etc.)
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **chore**: Changes to the build process or auxiliary tools and libraries
- **ci**: Changes to CI/CD configuration
- **build**: Changes that affect the build system or external dependencies

### Breaking Changes

To indicate a **MAJOR** version bump (`1.2.3` → `2.0.0`), add `BREAKING CHANGE:` in the footer:

```
feat: remove deprecated API

BREAKING CHANGE: The `oldMethod()` has been removed. Use `newMethod()` instead.
```

Or include `!` after the type:

```
feat!: remove deprecated API
```

## Examples

### Feature Commit
```
feat: add support for OpenAI GPT-5 model

- Add GPT-5 model configuration
- Update provider schemas
- Add integration tests
```

### Bug Fix
```
fix: handle empty response from Anthropic API

Fixes issue where empty responses caused panic in streaming mode.
Closes #123
```

### Breaking Change
```
feat!: migrate to new authentication system

BREAKING CHANGE: The `api_key` parameter is now required for all requests.
Update your configuration files accordingly.
```

### Documentation
```
docs: update installation instructions

- Add Python 3.12 support
- Update Node.js version requirements
- Add troubleshooting section
```

### Refactoring
```
refactor: simplify error handling in HTTP client

- Extract common error handling logic
- Reduce code duplication
- Improve error messages
```

## Release Automation

When you push commits to the `main` branch:

1. **Semantic Release** analyzes your commit messages
2. **Automatic Version Bump** based on commit types:
   - `fix:` → Patch release (`1.2.3` → `1.2.4`)
   - `feat:` → Minor release (`1.2.3` → `1.3.0`)
   - `BREAKING CHANGE:` → Major release (`1.2.3` → `2.0.0`)
3. **GitHub Release** created automatically
4. **Packages Published** to npm, PyPI, and crates.io
5. **Changelog Updated** with release notes

## Workflow

### Development Workflow

1. **Create Feature Branch**
   ```bash
   git checkout -b feature/new-model-support
   ```

2. **Make Changes & Commit**
   ```bash
   # Make your changes
   git add .

   # Use conventional commit format
   git commit -m "feat: add support for Claude 3.5 Sonnet model

   - Add model configuration
   - Update provider detection
   - Add integration tests"
   ```

3. **Push & Create PR**
   ```bash
   git push origin feature/new-model-support
   # Create PR to main branch
   ```

4. **Merge to Main**
   ```bash
   # After PR approval and merge
   # Semantic release will automatically:
   # - Create version tag (e.g., v1.3.0)
   # - Create GitHub release
   # - Publish to all package registries
   ```

### Manual Release (Emergency)

If you need to trigger a release manually:

```bash
# Go to GitHub Actions → Semantic Release → Run workflow
# Choose release type: patch, minor, or major
```

## Version Numbers

Versions follow [Semantic Versioning](https://semver.org/):

- **MAJOR.MINOR.PATCH** (e.g., `1.2.3`)
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

## Pre-releases

For pre-releases, use tags like:
- `v1.0.0-alpha.1`
- `v1.0.0-beta.1`
- `v1.0.0-rc.1`

These are triggered by pushing tags with pre-release identifiers.

## Best Practices

1. **Write Clear Descriptions**: Make your commit messages descriptive
2. **Use Proper Types**: Choose the most appropriate commit type
3. **Reference Issues**: Use `Closes #123` or `Fixes #456` in commit messages
4. **Keep Commits Small**: Each commit should represent one logical change
5. **Test Before Pushing**: Ensure your changes work before pushing to main
6. **Review Commit Messages**: Check that your commit messages follow conventions

## Tools

### Commit Message Linting

Add this to your IDE or use tools like:

```bash
# Install commitizen for interactive conventional commits
npm install -g commitizen cz-conventional-changelog
echo '{ "path": "cz-conventional-changelog" }' > ~/.czrc

# Use instead of git commit
git cz
```

### Commit Message Validation

Git hooks can validate commit messages:

```bash
# Install husky for git hooks
npm install -g husky
husky init

# Add commit-msg hook
echo '#!/bin/sh
npx --no-install commitlint --edit "$1"' > .husky/commit-msg
chmod +x .husky/commit-msg
```

## Troubleshooting

### Release Didn't Trigger

- Check that commits are on `main` branch
- Verify commit messages follow conventional format
- Check GitHub Actions logs for errors

### Wrong Version Bump

- Review recent commit messages
- Use manual release workflow to override if needed
- Check semantic-release configuration

### Need Help?

- Read the [Conventional Commits specification](https://conventionalcommits.org/)
- Check existing commit messages for examples
- Ask in GitHub Discussions or Issues
