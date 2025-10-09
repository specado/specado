# Anthropic Provider Specs

Claude variants share a single Messages API surface. `_base.yaml` contains the shared request/response mappings and authentication settings. Individual model specs live in subdirectories (for example `claude-4/opus.yaml`) and inherit from the base while declaring only the model identifiers or capability overrides.
