# Provider Inheritance Guide

This document explains how Specado models provider APIs using declarative inheritance. Use it as a checklist when adding new models or API surfaces.

## Directory layout

Each provider lives under `crates/specado-providers/providers/<provider>/`:

```
openai/
├── _base-responses.yaml            # Shared mappings for the Responses API
├── _base-chat-completions.yaml     # Shared mappings for the Chat Completions API
├── gpt-5/
│   ├── base.yaml                   # Responses API variants
│   └── chat.yaml                   # Chat Completions entry point
├── gpt-4/
│   ├── o.yaml
│   └── turbo.yaml
└── README.md                       # Notes about the layout

anthropic/
├── _base.yaml                      # Shared mappings for Claude Messages API
├── claude-4.5/
│   └── sonnet.yaml
├── claude-3/
│   └── opus.yaml
├── claude-3.5/
│   └── sonnet.yaml
├── claude-4/
│   └── opus.yaml
└── README.md
```

## Spec anatomy

Each spec YAML follows the schema defined in `crates/specado-schemas`:

- `inherits` points to another spec file (relative path) that will be merged first.
- `capabilities` stores provider-specific metadata with typed fields (context window, feature flags, reasoning controls).
- `capabilities_extra` allows experimental `x_*` capability keys without schema churn.
- `unsupported_parameters` lists PromptSpec paths that should trigger lossiness warnings.
- `mappings` describe declarative JSONPath translations for request and response payloads.

When loading a spec, the engine merges the inheritance chain, reporting an error if cycles are detected.

## Adding a new model family

1. Decide which base spec applies (e.g., `_base-responses.yaml` for GPT-5).
2. Create a new directory (for example `gpt-6/`) with one or more YAML specs (`base.yaml`, `chat.yaml`, etc.) that:
   - set `inherits` to the appropriate base file.
   - list supported `models`.
   - describe `capabilities` such as context window, tool support, reasoning controls.
   - declare `unsupported_parameters` for prompt fields the provider rejects.
3. Update docs and examples to reference the new spec files.
4. Extend tests (typically `provider_catalog`) to validate translation and lossiness behaviour.

## Lossiness reporting

- `unsupported_parameters` generates `LossinessCode::Unsupported` entries when users set fields the provider does not accept.
- Mapping `code`s such as `Relocate` or `Clamp` also surface lossiness entries automatically.

## Metadata requirements

- GPT-5 specs expect prompt metadata keys like `openai_model`, `openai_max_output_tokens`, `openai_reasoning_effort`, and `openai_text_verbosity`.
- Claude 4 specs expect keys such as `anthropic_model`, `anthropic_max_tokens`, `anthropic_thinking_type`, and `anthropic_thinking_budget`.

When adding new models, document their required metadata in Quickstart or provider README files so CLI users know which fields to supply.
