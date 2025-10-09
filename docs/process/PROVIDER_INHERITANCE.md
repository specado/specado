# Provider Inheritance Guide

This document explains how Specado models provider APIs using declarative inheritance. Use it as a checklist when adding new models or API surfaces.

## Directory layout

Each provider lives under `crates/specado-providers/providers/<provider>/`:

```
openai/
├── _base-responses.yaml          # Shared mappings for the Responses API
├── _base-chat-completions.yaml   # Shared mappings for the Chat Completions API
├── gpt-5-family.yaml             # Inherits from _base-responses
└── README.md                     # Notes about the layout

anthropic/
├── _base.yaml                    # Shared mappings for Claude Messages API
├── claude-sonnet-4-5.yaml        # Inherits from _base.yaml
└── README.md
```

## Spec anatomy

Each spec YAML follows the schema defined in `crates/specado-schemas`:

- `inherits` points to another spec file (relative path) that will be merged first.
- `capabilities` stores provider-specific metadata (context window, tool support, etc.).
- `unsupported_parameters` lists PromptSpec paths that should trigger lossiness warnings.
- `mappings` describe declarative JSONPath translations for request and response payloads.

When loading a spec, the engine merges the inheritance chain, reporting an error if cycles are detected.

## Adding a new model family

1. Decide which base spec applies (e.g., `_base-responses.yaml` for GPT-5).
2. Add a new `<model>-family.yaml` that sets:
   - `inherits` pointing to the base file.
   - `models` array listing all variants in the family.
   - `capabilities` with context windows, tool support, reasoning controls, etc.
   - `unsupported_parameters` for prompt fields the provider rejects.
3. Update docs and examples to reference the new spec file.
4. Extend tests (typically `provider_catalog`) to validate translation and lossiness behaviour.

## Lossiness reporting

- `unsupported_parameters` generates `LossinessCode::Unsupported` entries when users set fields the provider does not accept.
- Mapping `code`s such as `Relocate` or `Clamp` also surface lossiness entries automatically.

## Metadata requirements

- GPT-5 specs expect prompt metadata keys like `openai_model`, `openai_max_output_tokens`, `openai_reasoning_effort`, and `openai_text_verbosity`.
- Claude 4.5 specs expect keys such as `anthropic_model`, `anthropic_max_tokens`, `anthropic_thinking_type`, and `anthropic_thinking_budget`.

When adding new models, document their required metadata in Quickstart or provider README files so CLI users know which fields to supply.
