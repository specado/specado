# Property-Based Testing

Specado uses property-based testing via the `proptest` crate to verify that key invariants hold across a wide range of inputs. This approach complements traditional unit tests by automatically generating test cases and finding edge cases.

## Overview

Property-based testing verifies that certain properties or invariants always hold true, regardless of the specific input values. Instead of writing individual test cases, we define:

1. **Strategies**: Rules for generating random test data
2. **Properties**: Invariants that should always be true
3. **Shrinking**: Automatic minimization of failing test cases

## Test Categories

### Translation Engine Properties

Located in `crates/specado-core/tests/prop_translation.rs`:

- **Never Panics**: Translation should handle all valid inputs without panicking
- **Message Preservation**: Output messages ≥ input messages (system prompts may be added)
- **Strict Mode Ordering**: Stricter modes produce equal or more lossiness warnings
- **Valid JSON Output**: All translations produce valid, serializable JSON
- **Parameter Clamping**: Out-of-range parameters are properly clamped with warnings
- **Tool Handling**: Tools are either preserved or marked as unsupported
- **Role Preservation**: Message roles are maintained through translation

### JSONPath Properties

Located in `crates/specado-core/src/translation/jsonpath/prop_tests.rs`:

- **Parse Safety**: JSONPath parsing never panics on any input
- **Execution Safety**: Valid JSONPath execution never panics
- **Determinism**: Same path on same JSON always produces same result
- **Root Path Behavior**: "$" always returns the entire document
- **Array Bounds**: Array indices are properly bounds-checked
- **Type Safety**: Property access on non-objects handled gracefully
- **Wildcard Behavior**: Wildcards return all elements/values as expected
- **Path Consistency**: Nested paths equivalent to step-by-step access

### Schema Validation Properties

Located in `crates/specado-schemas/tests/prop_tests.rs`:

- **Validator Safety**: Validators never panic on any JSON input
- **Mode Ordering**: Stricter validation modes find equal or more errors
- **Determinism**: Validation results are consistent across multiple runs
- **Required Fields**: Missing required fields always fail validation
- **Valid Specs**: Properly formed specs pass appropriate validation modes

## Writing Property Tests

### 1. Define Value Strategies

```rust
/// Strategy for generating message roles
fn message_role_strategy() -> impl Strategy<Value = MessageRole> {
    prop_oneof![
        Just(MessageRole::System),
        Just(MessageRole::User),
        Just(MessageRole::Assistant),
    ]
}

/// Strategy for generating messages with controlled content
fn message_strategy() -> impl Strategy<Value = Message> {
    (
        message_role_strategy(),
        "[a-zA-Z0-9 .,!?]{1,200}",  // content regex
        proptest::option::of("[a-zA-Z0-9_]{1,20}"),  // optional name
    ).prop_map(|(role, content, name)| {
        Message { role, content, name, metadata: None }
    })
}
```

### 2. Define Properties

```rust
proptest! {
    /// Property: Translation should never panic
    #[test]
    fn prop_translation_never_panics(
        prompt_spec in prompt_spec_strategy()
    ) {
        let provider_spec = minimal_provider_spec();
        let model_id = "test-model";
        
        // Should either succeed or return error, but never panic
        let _ = translate(&prompt_spec, &provider_spec, model_id, prompt_spec.strict_mode);
    }
}
```

### 3. Use Recursive Strategies for Complex Data

```rust
/// Strategy for generating nested JSON with controlled depth
fn json_value_strategy() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(|n| Value::Number(n.into())),
        "[a-zA-Z0-9 ]{0,50}".prop_map(Value::String),
    ];
    
    leaf.prop_recursive(
        3,  // max depth
        10, // max size
        5,  // items per collection
        |inner| {
            prop_oneof![
                vec(inner.clone(), 0..5).prop_map(Value::Array),
                hash_map("[a-zA-Z_][a-zA-Z0-9_]{0,20}", inner, 0..5)
                    .prop_map(|m| Value::Object(m.into_iter().collect())),
            ]
        },
    )
}
```

## Running Property Tests

```bash
# Run all property tests
cargo test prop_

# Run with more test cases (default is 256)
PROPTEST_CASES=1000 cargo test prop_

# Run specific property test
cargo test prop_translation_never_panics

# Show minimal failing input on failure
cargo test prop_ -- --nocapture
```

## Configuration

Property test behavior can be configured via environment variables:

- `PROPTEST_CASES`: Number of test cases to generate (default: 256)
- `PROPTEST_MAX_SHRINK_ITERS`: Maximum shrinking iterations (default: 10000)
- `PROPTEST_FORK`: Run each test in a subprocess (default: false)

## Best Practices

1. **Start Simple**: Begin with basic properties like "never panics"
2. **Add Invariants**: Identify and test key system invariants
3. **Use Filters**: Filter generated data for valid test cases
4. **Leverage Shrinking**: Let proptest minimize failing cases automatically
5. **Combine with Unit Tests**: Property tests complement, don't replace, unit tests

## Common Property Patterns

### Safety Properties
- Functions never panic on valid inputs
- Operations are memory-safe
- Resources are properly cleaned up

### Correctness Properties
- Round-trip operations preserve data
- Transformations maintain invariants
- Operations are deterministic

### Performance Properties
- Operations complete within time bounds
- Memory usage stays within limits
- Complexity matches expectations

## Debugging Failures

When a property test fails:

1. **Check the minimal failing input**: Proptest automatically shrinks to smallest failing case
2. **Add the case as a regression test**: Convert to a unit test to prevent regressions
3. **Review the property definition**: Ensure the property correctly captures the invariant
4. **Check edge cases**: Common issues include empty collections, boundary values, special characters

## Future Enhancements

- **Stateful Testing**: Model-based testing for stateful systems
- **Custom Shrinking**: Implement domain-specific shrinking strategies
- **Performance Properties**: Add properties for performance characteristics
- **Coverage-Guided Generation**: Use code coverage to guide test generation