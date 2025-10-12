# Specado

Specado is a spec-driven orchestration layer that unifies prompt execution across multiple LLM providers. It provides a single, consistent interface for interacting with a wide range of language models, and allows you to define and customize providers and models using simple YAML specifications.

## Key Features

*   **Unified Interface**: A single, consistent API for interacting with multiple LLM providers, including OpenAI and Anthropic.
*   **Spec-Driven Configuration**: Define and customize providers, models, and parameter mappings using simple, powerful YAML files. See the [Provider Spec Guide](docs/PROVIDER_SPEC.md) for more details.
*   **Powerful CLI**: A user-friendly command-line interface with a rich set of features, including:
    *   Single-turn queries with the `ask` command.
    *   Interactive chat sessions with history.
    *   Support for advanced provider features like OpenAI "reasoning" and Anthropic "thinking" modes.
    *   Shell completion for Bash, Zsh, Fish, and more.
*   **Python and Node.js Libraries**: Integrate Specado into your own applications with our easy-to-use Python and Node.js libraries.
*   **Provider-Agnostic Design**: The core of Specado is provider-agnostic, allowing you to easily add new providers and models.

## Installation

_(Detailed installation instructions for different platforms will be added here once the project is packaged for distribution. For now, you will need to build from source.)_

### Building from Source

**Prerequisites:**

*   Rust 1.75 or newer
*   Node.js 18+
*   Python 3.9+

### Environment configuration

Specado reads credentials from the following locations (in order):

1. `~/.config/specado/.env` on Linux, `~/Library/Application Support/specado/.env` on macOS, or `%AppData%\specado\.env` on Windows. Create the directory if needed (e.g. `mkdir -p ~/.config/specado && chmod 700 ~/.config/specado` on Linux, `mkdir -p "$HOME/Library/Application Support/specado"` on macOS) and restrict the file (`chmod 600 …`).
2. A `.env` file in the current working directory.
3. Any variables already present in the process environment.

The CLI, Python demo, and Node demo all follow this loading order. Once your keys are stored in the global config file you can still override them per-repo by adding a `.env` next to your prompts, or export them manually with `export OPENAI_API_KEY=...`.

```sh
# 1. Clone the repository
git clone https://github.com/specado/specado.git
cd specado

# 2. Build the workspace
cargo build --workspace
```

## CLI Usage

The `specado` CLI is the primary way to interact with the system. The main binary is located at `target/debug/specado` after building from source.

### `specado ask`

The `ask` command is the main entrypoint for interacting with LLMs.

**Single-Turn Query:**

```sh
./target/debug/specado ask "What is the meaning of life?"
```

**Interactive Chat:**

```sh
./target/debug/specado ask --interactive
```

**Flags:**

*   `--provider <PROVIDER>`: Specify a provider to use (e.g., `openai`, `anthropic`).
*   `--model <MODEL>`: Specify a model to use (e.g., `gpt-4o`, `claude-3.5-sonnet-20240620`).
*   `--messages-file <PATH>`: Load a chat history from a JSON or YAML file.
*   `--reason`: Enable advanced reasoning/thinking modes.

### `specado completions`

Generate shell completion scripts.

```sh
# Example for Bash
./target/debug/specado completions bash > /usr/local/share/bash-completion/completions/specado
```

### `specado validate`

Validate a provider or prompt spec file.

```sh
./target/debug/specado validate --spec crates/specado-providers/providers/openai/gpt-4/o.yaml
```

### `specado preview`

Preview the translated payload for a given prompt and provider without making a network call.

```sh
./target/debug/specado preview --prompt examples/prompts/basic_chat.json --provider crates/specado-providers/providers/openai/gpt-4/o.yaml
```

### `specado run`

Execute a prompt against a provider.

```sh
./target/debug/specado run --prompt examples/prompts/basic_chat.json --provider crates/specado-providers/providers/openai/gpt-4/o.yaml
```

## Programmatic Usage

### Python

**Installation:**

```sh
pip install maturin
maturin develop -m crates/specado-py/Cargo.toml
```

**Usage:**

```python
from specado import Specado

# Initialize the client
client = Specado()

# Make a simple request
response = client.ask("Hello, world!")
print(response.content)

# Use a custom provider spec
custom_client = Specado(provider_spec="/path/to/my_provider.yaml")
response = custom_client.ask("Hello from my custom provider!")
print(response.content)
```

### Node.js

**Installation:**

```sh
(cd crates/specado-node && npm install && npm run build)
```

**Usage:**

```javascript
import { Specado } from 'specado';

async function main() {
  // Initialize the client
  const client = new Specado();

  // Make a simple request
  const response = await client.ask("Hello, world!");
  console.log(response.content);

  // Use a custom provider spec
  const customClient = new Specado({
    providerSpec: "/path/to/my_provider.yaml"
  });
  const customResponse = await customClient.ask("Hello from my custom provider!");
  console.log(customResponse.content);
}

main();
```

## Examples

The `examples/` directory contains a set of runnable examples that demonstrate the features of Specado. See the [Examples README](examples/README.md) for more details.

## Contributing

Contributions are welcome! This project uses automated releases based on [Conventional Commits](https://conventionalcommits.org/).

### Development Process

1. **Fork and Clone** the repository
2. **Create a Feature Branch**: `git checkout -b feature/your-feature-name`
3. **Make Changes** and ensure tests pass: `cargo test --workspace`
4. **Commit with Conventional Format**:
   ```bash
   # For new features
   git commit -m "feat: add new model support"

   # For bug fixes
   git commit -m "fix: handle empty API responses"

   # For breaking changes
   git commit -m "feat!: redesign authentication API

   BREAKING CHANGE: The auth config format has changed"
   ```
5. **Push and Create PR** to the `main` branch

### Automated Releases

When your PR is merged to `main`:
- ✅ **Automatic Versioning**: Based on your commit messages
- ✅ **GitHub Release**: Created automatically
- ✅ **Multi-Platform Publishing**: npm, PyPI, and crates.io updated simultaneously
- ✅ **Changelog**: Generated automatically

### Commit Message Guidelines

Please follow [Conventional Commits](docs/process/CONVENTIONAL_COMMITS.md) format:
- `feat:` for new features (minor version bump)
- `fix:` for bug fixes (patch version bump)
- `feat!:` or `BREAKING CHANGE:` for breaking changes (major version bump)

See [GitHub Tracking Guidelines](docs/process/GITHUB_TRACKING.md) for detailed process information.

## License

This project is licensed under the terms of the [LICENSE](LICENSE) file.
