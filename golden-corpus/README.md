# Golden Test Corpus

This directory contains the golden test corpus for the Specado translation engine. Golden tests (snapshot tests) ensure that translation outputs remain consistent across changes.

## Directory Structure

```
golden-corpus/
├── basic/              # Simple, fundamental test cases
├── complex/            # Advanced features (tools, streaming, etc.)
├── edge-cases/         # Boundary conditions and error scenarios
├── providers/          # Provider-specific test cases
│   ├── openai/
│   └── anthropic/
├── regression/         # Tests for previously fixed bugs
└── snapshots/          # Expected output snapshots
```

## Running Golden Tests

### Run All Tests
```bash
UPDATE_GOLDEN=0 cargo test --test golden_tests -- --ignored
```

### Update Snapshots
```bash
UPDATE_GOLDEN=1 cargo test --test golden_tests -- --ignored
```

### Run Specific Categories
```bash
cargo test --test golden_tests golden_test_basic -- --ignored
cargo test --test golden_tests golden_test_edge_cases -- --ignored
```

## Test Case Format

Each test case is defined in a `test.json` file with the following structure:

```json
{
  "name": "test-name",
  "category": "category",
  "input": {
    "prompt_spec": { /* PromptSpec JSON */ },
    "provider_spec": null | "filename.json" | { /* inline spec */ }
  },
  "provider": "openai",
  "expectations": {
    "should_succeed": true,
    "error_pattern": null,
    "ignore_fields": ["metadata.timestamp"],
    "volatile_fields": [
      {
        "path": "field.path",
        "pattern": "regex-pattern"
      }
    ],
    "expected_lossiness": ["Clamp", "Unsupported"]
  },
  "metadata": {
    "description": "What this test validates",
    "tags": ["tag1", "tag2"],
    "enabled": true,
    "priority": 1
  }
}
```

## Environment Variables

- `UPDATE_GOLDEN=1` - Update snapshots instead of comparing
- `GOLDEN_CORPUS_DIR` - Override corpus directory location
- `GOLDEN_SNAPSHOT_DIR` - Override snapshot directory location
- `GOLDEN_VERBOSE=1` - Enable verbose output

## Adding New Tests

1. Create a new directory under the appropriate category
2. Add a `test.json` file with the test case definition
3. Run the test with `UPDATE_GOLDEN=1` to create the initial snapshot
4. Review the generated snapshot in `snapshots/`
5. Run the test normally to verify it passes

## Test Categories

### Basic
Fundamental translation scenarios:
- Simple chat completion
- Sampling parameters
- Token limits
- Response formats

### Complex
Advanced features:
- Function calling/tools
- Streaming responses
- Multi-modal inputs
- Parallel tool calls

### Edge Cases
Boundary conditions:
- Parameter clamping
- Invalid inputs
- Resource limits
- Error handling

### Provider-Specific
Tests tailored to specific providers:
- OpenAI GPT models
- Anthropic Claude models
- Google Gemini models

### Regression
Tests that ensure previously fixed bugs don't reoccur.

## Snapshot Management

Snapshots are stored in `snapshots/{category}/{test-name}.json` and contain:
- The expected translation output
- Metadata about when the snapshot was created/updated
- Fields to ignore during comparison
- Volatile field patterns for dynamic content

## CI Integration

The golden tests are designed to run in CI environments:
- Tests fail if snapshots don't match
- Use `UPDATE_GOLDEN=1` locally to update snapshots
- Commit snapshot changes alongside code changes
- CI runs with `UPDATE_GOLDEN=0` to verify consistency

## Troubleshooting

### Test Fails Due to Snapshot Mismatch
1. Review the diff output to understand what changed
2. If the change is expected, update with `UPDATE_GOLDEN=1`
3. If unexpected, investigate the code change that caused it

### Volatile Fields
For fields that change every run (timestamps, IDs):
1. Add them to `ignore_fields` to completely ignore
2. Or add to `volatile_fields` with a regex pattern to validate format

### Provider Specs
- Place shared provider specs in `providers/{provider}/`
- Reference them in tests with relative paths
- Or embed them directly in the test JSON