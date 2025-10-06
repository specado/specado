# specado-core

`specado-core` houses the orchestration engine that translates Specado prompts, communicates with providers, and normalizes responses. At this stage the crate only exposes scaffolding while the wider system comes together.

## Development

- Benchmarks use [Criterion](https://bheisler.github.io/criterion.rs/book/) and live under `benches/`. Run them with:
  ```bash
  cargo bench -p specado-core
  ```
- Unit tests rely on Tokio's `test-util` feature for async helpers.
- Schema validation flows through the reusable validators in `specado-schemas`.

Future tasks will populate the modules for translation, routing, HTTP orchestration, and resilience features described in `docs/process/SPECADO_PLAN.md`.
