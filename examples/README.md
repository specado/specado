# Examples

This directory contains a set of examples to help you get started with Specado.

Before running these examples, please make sure you have followed the installation instructions in the main [README.md](../README.md). Specado looks for API keys in `~/.config/specado/.env` on Linux, `~/Library/Application Support/specado/.env` on macOS, and `%AppData%\specado\.env` on Windows before falling back to a local `.env`; create one of those files if you don't want to export variables manually (`mkdir -p ~/.config/specado` on Linux, `mkdir -p "$HOME/Library/Application Support/specado"` on macOS).

## CLI Demo

The `cli_demo.sh` script demonstrates how to use the `specado` CLI to interact with the advanced features of different providers.

### Usage

The script relies on the same environment loading rules (global config, local `.env`, or exported variables). Set up at least one of those locations with `OPENAI_API_KEY` and `ANTHROPIC_API_KEY`.

```sh
./examples/cli_demo.sh
```

## Python Example

The `python_basic.py` script shows how to use the Specado Python library.

### Usage

```sh
# Run the OpenAI reasoning demo
python examples/python_basic.py --scenario openai-reasoning

# Run the Anthropic thinking demo
python examples/python_basic.py --scenario anthropic-thinking
```

## Node.js Example

The `node_basic.mjs` script shows how to use the Specado Node.js library.

### Usage

```sh
# Run the OpenAI reasoning demo
node examples/node_basic.mjs --scenario openai-reasoning

# Run the Anthropic thinking demo
node examples/node_basic.mjs --scenario anthropic-thinking
```

## Prompts

This directory also contains the JSON prompt files that are used by the examples:

*   `prompts/basic_chat.json`: A minimal chat prompt.
*   `prompts/openai_reasoning.json`: A prompt that demonstrates OpenAI's reasoning controls.
*   `prompts/anthropic_thinking.json`: A prompt that demonstrates Anthropic's thinking mode.
