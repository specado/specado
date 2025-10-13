# Release Workflows

## Overview

The Specado release pipeline is composed of a thin orchestration workflow (`release.yml`) that invokes a set of reusable workflow components that live alongside it. This modular layout makes it easier to reason about the release stages, run them independently for testing, and extend the pipeline without editing a monolithic YAML file.

## Workflow Structure

- `release.yml` &mdash; orchestrates the entire release by calling the reusable workflows below.
- `release-test.yml` &mdash; runs the full Rust, Node.js, and Python test suites.
- `release-build.yml` &mdash; produces platform-specific artifacts and smoke-tests the CLI.
- `release-package.yml` &mdash; aggregates artifacts into the final distributable bundles.
- `release-publish-testpypi.yml` &mdash; publishes to Test PyPI and verifies installation.
- `release-publish-npm.yml` &mdash; publishes the npm package.
- `release-publish-crates.yml` &mdash; publishes the crates.io packages (supports temporary package names).
- `release-publish-pypi.yml` &mdash; publishes the production PyPI package.
- `release-verify.yml` &mdash; verifies npm/PyPI/crates installs after publication.
- `release-github.yml` &mdash; creates or updates the GitHub release and uploads assets.

## Execution Flow

1. **prepare** &mdash; extracts version metadata, detects temporary crate names, and decides whether to run the release.
2. **test** &mdash; executes all tests; build steps only run if this succeeds.
3. **build** &mdash; cross-platform builds and smoke tests.
4. **package** &mdash; assembles multi-platform artifacts and computes checksums.
5. **publish** (parallel where safe):
   - Test PyPI (with verification)
   - npm
   - crates.io
6. **publish PyPI** &mdash; runs after Test PyPI succeeds (and optionally crates/npm).
7. **verify_all** &mdash; installs all published artifacts to ensure availability.
8. **github_release** &mdash; updates the GitHub release and uploads CLI assets.

## Testing Changes

1. Use `workflow_dispatch` with `dry_run=true` to exercise the pipeline without hitting registries.
2. Each reusable workflow exposes a `workflow_call` trigger, so you can add a temporary `workflow_dispatch` block for isolated testing.
3. For end-to-end validation, push a temporary tag such as `v0.0.0-test-*` and monitor the run.

## Adding New Publish Targets

1. Create `release-publish-<target>.yml` (or another descriptive `release-*.yml`) following the existing pattern (declare inputs, secrets, concurrency group).
2. Add the new job to `release.yml` with appropriate dependencies and gating conditions.
3. Extend `release-verify.yml` if post-publish verification is required.
4. Update this README with the new component.

## Troubleshooting

- **Artifacts missing:** Confirm the artifact names match between upload/download steps and that retention has not expired (default 14 days).
- **Secrets unavailable:** Reusable workflows must declare required secrets; ensure the caller passes them via the `secrets:` block.
- **Outputs undefined:** When a job may be skipped (e.g., dry runs), guard expressions with fallbacks such as `${{ needs.job.outputs.value || 'false' }}`.
- **Manual dry runs:** The `dry_run` input only affects publishing/verification jobs; tests, builds, and packaging still execute.

## Temporary Crate Names

While crates.io enforces a 24-hour re-registration delay, the release pipeline supports publishing to temporary `*-temp` crate names. The `prepare` job reads the active manifest names and passes them through to the publish and verify workflows, so reverting to the canonical names only requires updating the manifests once the embargo lifts.
