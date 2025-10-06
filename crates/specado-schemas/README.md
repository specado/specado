# specado-schemas

This crate hosts the JSON Schemas that define Specado's prompt and provider specifications. The schemas are compiled at runtime into reusable validators so other crates can enforce correctness consistently.

## Usage

```rust
use specado_schemas::get_validator;
use serde_json::json;

let validator = get_validator();
let prompt = json!({
    "version": "1",
    "messages": [
        { "role": "system", "content": "You are a helpful assistant." }
    ]
});

validator.validate_prompt(&prompt)?;
```

Both prompt and provider schemas produce friendly error messages by joining validation failures. The validator is cached globally using `once_cell::sync::Lazy` to avoid recompiling schemas during normal operation.
