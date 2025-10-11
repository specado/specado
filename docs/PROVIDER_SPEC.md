# Provider Specification Format

Provider specs describe how the neutral `PromptSpec` produced by the CLI/bindings maps onto a vendor's HTTP API.

## Where specs live
- `crates/specado-providers/providers/<vendor>/`: canonical YAML files checked into the workspace.
- Base files (`_base*.yaml`) capture shared behaviour; family/model files inherit from them (see [Provider Inheritance](process/PROVIDER_INHERITANCE.md)).
- Overlays in `overlays/` add adapter defaults; keep experimental fields behind an `x_*` namespace.

## Top-level YAML anatomy
| Field | Required | Purpose |
| --- | --- | --- |
| `provider` | ✓ | Vendor slug. Matches `Auth` routing and adapter selection. |
| `models[]` | ✓ | List of canonical model IDs supported by the spec. |
| `inherits` |  | Relative path to another spec to merge first. |
| `interface` |  | Neutral hint (e.g. `text.generate`) that helps the adapter registry (see `docs/process/INTERFACE_TAXONOMY.md`). |
| `contract_version` |  | Semver contract used by validation/runtime overlays. |
| `auth` | ✓ | Authentication scheme (`bearer` or `apikey`). |
| `endpoints.chat` | ✓ | HTTP method/url/headers for the chat-like entry point. |
| `mappings.request[]` | ✓ | JSONPath mappings from `PromptSpec` → provider request payload. |
| `mappings.response[]` | ✓ | JSONPath mappings from provider response → uniform response. |
| `constraints.supports` | ✓ | Boolean capability switches used by validation/lossiness reporting. |
| `capabilities` |  | Rich capability flags (context window, reasoning controls, etc.). |
| `capabilities_extra` |  | Free-form `x_*` capability additions. |
| `extensions` |  | Reserved for experimental configuration (`x_*` keys). |
| `unsupported_parameters[]` |  | JSONPaths in `PromptSpec` that should emit `LossinessCode::Unsupported`. |

Specs are validated against `crates/specado-schemas/schemas/provider-spec.v1.schema.json`. Use `serde_json_path`-compatible JSONPaths; array indices and filter expressions are supported, but wildcards/globs must be resolved before write-time.

## Authentication and endpoints
- `auth` maps directly to `AuthScheme` (`bearer` requires `token_env`; `apikey` requires `header` + `key_env`).
- Each spec must expose at least the `chat` endpoint. Additional surfaces (embeddings, images, speech, video) can be added when the schema grows; keep headers deterministic and reference environment variables only.

## Capabilities vs. constraints
- `constraints.supports` is coarse-grained and feeds lossiness detection. Set `json_mode=false` or `tools=false` if the provider rejects those features.
- `capabilities` capture finer detail (context window, reasoning controls, seed availability, etc.). Omit entries or leave them `false` unless the provider guarantees support.
- Use `capabilities_extra` for experimental keys (`x-vendor-feature`). These must be scoped so downstream consumers can opt in explicitly.

## Request mappings
Each `mappings.request` entry has:
- `from`: JSONPath into the neutral prompt (root is `$`). Example: `$.metadata.openai_model`.
- `to`: JSONPath in the provider payload (`$` is the request root).
- `code` (optional): when present it should match a `LossinessCode` variant (`Relocate`, `Clamp`, `Unsupported`, ...) so the transformer records it in the lossiness report.
- `clamp` (optional): `[min, max]` range for numeric values. The transformer will clamp out-of-range inputs and emit a `Clamp` lossiness entry.

Favour neutral metadata keys (`$.metadata.reasoning.effort`, `$.metadata.thinking.type`) and retain vendor aliases only when external APIs require them. Align new specs with the conventions already present in the workspace.

## Response mappings
`mappings.response` translate the provider's raw payload into Specado's uniform response:
- `from` is a JSONPath evaluated against the vendor response.
- `to` is the field in the uniform response (`content`, `finish_reason`, etc.).

Keep response mappings minimal; prefer doing structural reshaping in the transformer (see `transformer::normalize`) when output requires more than JSONPath extraction.

## Unsupported parameters
List JSONPaths that the provider rejects outright. Paths are evaluated against the `PromptSpec` root (use expressions like `$.response[?(@.format != 'text')].format` to catch only non-text formats). Unsupported entries surface as `LossinessCode::Unsupported` with an actionable suggestion.

## Extensions and overlays
- Use `extensions` for stable `x_*` fields that runtime adapters understand today.
- Use overlays (`overlays/<provider>.<adapter>.yaml`) for adapter-specific defaults so base specs stay portable.

## Metadata conventions
- Neutral keys live under `$.metadata` and should be used when possible:
  - Reasoning controls: `$.metadata.reasoning.effort`, `$.metadata.reasoning.budget_tokens`.
  - Thinking controls: `$.metadata.thinking.type`, `$.metadata.thinking.budget_tokens`.
- Vendor-specific fallbacks (`$.metadata.openai_model`, `$.metadata.anthropic_model`) remain for compatibility with prompts that still rely on those keys. Document new additions in `docs/QUICKSTART.md` and example READMEs when you introduce them.

## Loading flow (Rust)
```rust
use std::path::Path;
use specado_core::{
    hot_reload::ProviderCache,
    translate,
    types::PromptSpec,
};

let provider = ProviderCache::new()
    .load_or_read(Path::new("crates/specado-providers/providers/openai/gpt-5/base.yaml"))?;

let prompt: PromptSpec = serde_json::from_str(include_str!("../examples/prompts/basic_chat.json"))?;
let (request_payload, lossiness) = translate(&prompt, &provider)?;

assert!(lossiness.entries.is_empty());
println!("{}", serde_json::to_string_pretty(&request_payload)?);
```

`ProviderCache` resolves inheritance, overlays, and reuses parsed specs across calls. The same API is used by the CLI and the golden tests.

## Validation and testing checklist
1. `cargo fmt`
2. `cargo test -p specado-core --test provider_catalog`
3. `cargo test -p specado-core --test golden`
4. `cargo test --workspace`

Run the `provider_catalog` test (or the golden suite) after editing specs; both validate against the JSON Schema and ensure translations stay stable.

## Adding or updating a provider
1. Inspect existing specs for the same provider family to decide whether to inherit from an existing base (`_base.yaml`) or create a new one.
2. Populate models, auth, endpoints, capabilities, and unsupported parameters.
3. Add request/response mappings using neutral metadata keys wherever possible.
4. Update docs (`docs/QUICKSTART.md`, `examples/` READMEs) if the prompt metadata surface changes.
5. Refresh golden snapshots if behaviour shifts (`./tests/golden/update_snapshots.sh`).
6. Run the validation checklist above and attach diffs to the owning issue/PR.

Keeping specs declarative ensures provider differences stay out of the core runtime and keeps the surface consistent across adapters.
