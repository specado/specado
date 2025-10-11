# Golden Snapshot Suite

Golden snapshots capture canonical translations so regressions surface immediately in CI. All comparisons trace back to the legacy corpus located at `../specado-newest-legacy/golden-corpus/`.

## Layout
- `cases/<category>/<name>/case.json` — Prompt input plus provider spec reference.
- `cases/<category>/<name>/snapshot.json` — Expected provider payload and full lossiness report.
- `update_snapshots.sh` — Helper script that refreshes every snapshot in one pass.

## Authoring guidelines
1. Model new cases after the legacy fixtures in `../specado-newest-legacy/golden-corpus/`, adapting prompts to the new `PromptSpec` and provider metadata keys.
2. Keep prompts deterministic (no timestamps, random data, or network calls). Rely on provider metadata rather than mutating provider specs.
3. Run the update script after editing a case so `snapshot.json` stays in sync.
4. Review diffs carefully; golden files change only when behaviour intentionally shifts and the owning issue explains the delta.

## Running the suite
```sh
cargo test -p specado-core --test golden
```

## Updating snapshots
```sh
./tests/golden/update_snapshots.sh
```

_Last updated for Issue #109 (test: migrate golden corpus)._ 
