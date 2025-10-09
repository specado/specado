# OpenAI Provider Specs

This directory uses spec inheritance to separate shared behaviour for each API surface:

- `_base-responses.yaml` captures the common mappings for the `v1/responses` endpoint used by GPT-5 and newer reasoning models.
- `_base-chat-completions.yaml` keeps the legacy `v1/chat/completions` mechanics used by GPT-4 and earlier.
- Model families (for example `gpt-5.yaml`) inherit from the appropriate base file and only declare model identifiers, capabilities, or overrides.

Add new model specs by inheriting from the correct base file rather than duplicating mappings.
