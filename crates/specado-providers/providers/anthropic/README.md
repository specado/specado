# Anthropic Provider Specs

Claude variants share a single Messages API surface. `_base.yaml` contains the shared request/response mappings and authentication settings. Individual model specs such as `claude-sonnet-45.yaml` inherit from the base and declare only the model identifiers or capability overrides.
