# Integration Test Scaffolding

Integration tests exercise Specado end-to-end against mock providers and the sample manifests in `examples/`. This directory currently contains placeholders and guidance for writing those tests.

## Strategy
- Use Rust integration tests under `tests/integration/*.rs` to drive the CLI or core APIs via `assert_cmd` and `httpmock`.
- For language bindings, prefer invoking the sample Python and Node scripts from their respective test harnesses so behaviour stays consistent across surfaces.
- Keep fixtures small and deterministic; golden snapshots for translations live in `../golden`.

## Writing a new test
1. Add a Rust test file (e.g., `tests/integration/cli_preview.rs`) that launches the CLI with the assets in `examples/`.
2. Use the helper script `run_smoke.sh` as a template for the commands that should succeed before asserting on output.
3. Record any new fixtures in version control and document how to regenerate them.
4. Tag the owning GitHub issue in code comments or commit messages for traceability (Issue #50 for the initial scaffold).

## Smoke test script
Run the placeholder script to see the intended workflow once real assertions are in place:
```sh
./tests/integration/run_smoke.sh
```

The script currently stops after performing sanity checks and will evolve as automated assertions are added.
