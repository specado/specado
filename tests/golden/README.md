# Golden Snapshot Scaffolding

Golden snapshots capture canonical translations and responses so that regressions are easy to spot during CI. This directory holds fixtures produced by `specado-core` integration tests.

## Layout
- `provider_preview.openai.json` &mdash; Example translation output for the OpenAI provider using `examples/prompts/basic_chat.json`.
- `update_snapshots.sh` &mdash; Helper script that documents how to regenerate snapshots once the automated tests are wired up.

## Authoring guidelines
1. Write or update the corresponding test in `tests/golden.rs` (to be added) so it emits deterministic output.
2. Run the helper script in this folder to regenerate fixtures.
3. Review diffs carefully; golden files are committed to source control and should only change when behaviour intentionally shifts.
4. Reference the owning GitHub issue (e.g., Issue #50) when updating snapshots so reviewers understand the motivation.

## Regenerating snapshots
When the golden tests exist, regenerate assets like so:
```sh
./tests/golden/update_snapshots.sh
```

The script is currently a placeholder but outlines the expected commands (`cargo test -- --update-snapshots`). Expand it as tests are implemented.

_Last updated for Issue #50 (Docs, Tests, Benches, Examples Scaffolding)._ 
