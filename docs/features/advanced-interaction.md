# Advanced Interaction Modes

Specado surfaces advanced provider features through neutral prompt metadata so that
CLI callers never need to juggle vendor-specific payloads. This guide covers the
two capabilities currently exposed through the `specado` CLI: Anthropic’s thinking
mode and OpenAI’s reasoning controls.

## Capability Overview

| Mode | CLI flags | Provider capability | Notes |
| --- | --- | --- | --- |
| Unified Reasoning | `--reason`<br>`--reason-effort <low\|medium\|high>`<br>`--reason-budget <tokens>`<br>`--reason-seed <integer>` | `supports_extended_thinking`, `reasoning_controls`, `supports_seed` | Specado maps `--reason` onto the capability a provider exposes: Anthropic thinking, OpenAI reasoning controls, deterministic seeding where available. |

Both modes are guarded at runtime: if the selected provider or model does not
advertise the required capability, the CLI returns a clear validation error
instead of issuing an API call that would ultimately be rejected.

---

## Anthropic Thinking Mode (via `--reason`)

Anthropic’s Claude models can generate extended “thinking” traces before producing
the final assistant reply. The neutral `--reason` flag maps onto this capability,
and the legacy `--thinking` flag is retained for backwards compatibility. The
mode is only honoured when the provider spec reports `supports_extended_thinking`.

```sh
# single-turn thinking with overlay defaults (type: enabled, budget: 2000)
SPECADO_DEFAULT_PROVIDER=... \
SPECADO_ANTHROPIC_TOKEN=... \
specado ask \
  --provider crates/specado-providers/providers/anthropic/claude-4.5/sonnet.yaml \
  --reason \
  "Plan a customer onboarding sequence"

# override the thinking budget for the request
specado ask \
  --provider anthropic.yaml \
  --reason \
  --reason-budget 1000 \
  "Summarise the key blockers from this document"
```

### Defaults & Overlays

- `overlays/anthropic.messages.yaml` defines the default thinking payload:
  type `enabled`, budget `2000`, and temperature `1.0`.
- CLI overrides only supply the fields that are explicitly set. Leaving
  `--reason-budget` unspecified allows overlay defaults (or downstream model
  overrides) to remain in effect.
- Thinking mode automatically raises the sampling temperature to 1.0, mirroring
  Anthropic’s API requirements.

### Troubleshooting

- `Provider '<name>' does not support reasoning or extended thinking capabilities`: the provider spec or selected
  model does not advertise the necessary capability. Switch to a supported
  model (e.g., Claude 3.5/4+) or drop the flag.
- Thinking is neutral metadata: prompt history and metadata can safely round-trip
  through Specado without manual vendor-specific tweaks.

---

## OpenAI Reasoning Controls (via `--reason`)

OpenAI’s GPT-5 family exposes reasoning controls that govern how aggressively the
model explores intermediate steps. Specado maps these controls onto neutral prompt
metadata and surfaces them through the `--reason` family of flags.

```sh
# enable reasoning with explicit effort, budget, and deterministic seed
OPENAI_API_KEY=... \
specado ask \
  --provider crates/specado-providers/providers/openai/gpt-5/base.yaml \
  --reason \
  --reason-effort high \
  --reason-budget 800 \
  --reason-seed 123 \
  "Prove that the square root of 2 is irrational"
```

### Flag Behaviour

- `--reason` toggles reasoning mode. When the provider exposes the `effort`
  control, the CLI selects `medium` by default; otherwise (e.g., Anthropic) it
  enables thinking mode.
- `--reason-effort` allows finer control (`low`, `medium`, `high`) when the
  provider exposes the control. The CLI fails early if the model does not list
  `effort`.
- `--reason-budget` adjusts the reasoning or thinking budget when the provider
  honours it.
- `--reason-seed` sets the sampling seed when the provider advertises `supports_seed`.
  Deterministic seeding is optional and safe to combine with non-reasoning runs.

### Implementation Notes

- Provider specs map neutral metadata (`metadata.reasoning.effort`) to the vendor
  request payload. No CLI code references OpenAI-specific field names.
- The translator forwards the seed into the Responses API payload (`seed` field),
  enabling deterministic reasoning runs where supported.

### Troubleshooting

- `Provider '<name>' does not support reasoning controls`: the provider lacks the
  required capability. Switch to a GPT-5 reasoning-capable spec or remove the flag.
- `Provider '<name>' does not support deterministic seeding`: either the provider
  or adapter does not advertise `supports_seed`.

---

## Related Resources

- [`docs/QUICKSTART.md`](../QUICKSTART.md) – CLI overview and getting started examples.
- [`overlays/anthropic.messages.yaml`](../../overlays/anthropic.messages.yaml) – default thinking configuration.
- [`crates/specado-providers/providers/openai/_base-responses.yaml`](../../crates/specado-providers/providers/openai/_base-responses.yaml) – reasoning mappings for OpenAI models.
