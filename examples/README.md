# Specado Examples

**Simple, production-ready examples showing how to use Specado across all surfaces.**

One spec, run anywhere: CLI, Python, Node.js, or Rust.

---

## 🚀 Quick Start (5 Minutes)

### 1. Set Your API Key

```bash
export OPENAI_API_KEY=sk-your-key-here
```

Or copy the template and keep keys in a local `.env` (loaded automatically by `demo.sh` and `setup.sh`):

```bash
cp .env.example .env
echo 'OPENAI_API_KEY=sk-your-key-here' >> .env
```

**Note**: Published Specado packages bundle the provider catalog—no extra configuration required. When you run straight from the repo, the scripts fall back to the bundled catalog inside each package.

### 2. Pick Your Language

**Python:**
```bash
cd python
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
python from_spec.py
```

**Node.js:**
```bash
cd node
npm install
npm run demo
```

**Rust:**
```bash
cd rust_basic
cargo run
```

**CLI:**
```bash
cargo install specado-cli-temp
specado ask "What is 2+2?" --provider openai
```

**Done.** That's all you need to see Specado in action.

---

## 📁 What's Included

```
examples/
├── prompts/                    # Reusable YAML prompt specs
│   ├── summarize_article.yaml
│   └── generate_function.yaml
│
├── python/                     # Python examples
│   ├── requirements.txt        # specado
│   ├── from_spec.py           # Load YAML and run
│   └── in_code.py             # Define in code
│
├── node/                       # Node.js examples
│   ├── package.json           # Dependencies + scripts
│   ├── from_spec.js          # Load YAML and run
│   └── in_code.js            # Define in code
│
├── rust_basic/                 # Rust example
│   ├── Cargo.toml
│   └── src/main.rs            # Both friendly + explicit APIs
│
├── setup.sh                    # Interactive setup
└── demo.sh                     # Run all examples
```

---

## 🎯 Installation & Usage

### CLI

**Install:**
```bash
cargo install specado-cli-temp
```

**Use:**
```bash
# Quick ask
specado ask "What is the capital of France?" --provider openai

# Run a spec file
specado run --prompt prompts/summarize_article.yaml --provider openai

# Validate first
specado validate --spec prompts/summarize_article.yaml

# Preview what gets sent to the provider
specado preview --prompt prompts/summarize_article.yaml --provider openai

# Interactive chat
specado ask --interactive --provider openai
```

### Python

**Install:**
```bash
cd python
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

**Run:**
```bash
# Make sure venv is activated first
source .venv/bin/activate

python from_spec.py   # Load from YAML
python in_code.py     # Define in code

# When done
deactivate
```

**Or use the automated scripts:**
```bash
cd ..
./setup.sh  # Creates venv automatically
./demo.sh python  # Activates venv and runs examples
```

**Code:**
```python
from specado import Client, build_prompt

client = Client("openai")

# Load from file
print(client.complete_file("../prompts/summarize_article.yaml"))

# Or define in code with helpers
prompt = build_prompt(
    "Explain closures in JavaScript.",
    system_message="You are a helpful assistant.",
    temperature=0.5,
)
print(client.complete(prompt))

# Quick shortcut for simple prompts
print(
    client.complete_text(
        "List three benefits of static typing.",
        system_message="You are a concise technical explainer.",
        temperature=0.5,
    )
)
```

### Node.js

**Install:**
```bash
cd node
npm install
```

**Run:**
```bash
npm run from-spec     # Load from YAML
npm run in-code       # Define in code
npm run demo          # Run both
```

**Code:**
```javascript
const { Client, simplePrompt } = require('specado');

async function main() {
  const client = new Client("openai");

  const first = await client.completeFile('../prompts/summarize_article.yaml');
  console.log(first);

  const quick = await client.completeText(
    "Explain closures in JavaScript.",
    { system: "You are a helpful assistant.", temperature: 0.5 }
  );
  console.log(quick);

  const prompt = simplePrompt({
    user: "List three benefits of static typing.",
    system: "You are a concise technical explainer.",
  });
  const third = await client.complete(prompt);
  console.log(third);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
```

### Rust

**Install:**
```toml
[dependencies]
specado = "0.2.1"
tokio = { version = "1", features = ["full"] }
```

**Run:**
```bash
cd rust_basic
cargo run
```

**Code:**
```rust
use specado::{execute, ExecuteOptions, Message, MessageRole, PromptSpec, SamplingConfig};

#[tokio::main]
async fn main() -> Result<()> {
    let prompt = PromptSpec {
        version: "1".into(),
        messages: vec![
            Message {
                role: MessageRole::System,
                content: "You are a helpful assistant.".into()
            },
            Message {
                role: MessageRole::User,
                content: "Explain closures.".into()
            },
        ],
        sampling: SamplingConfig {
            temperature: Some(0.5),
            ..Default::default()
        },
        ..Default::default()
    };

    let response = execute(
        prompt,
        "openai",
        ExecuteOptions::for_model("gpt-5"),
        None,
    ).await?;

    println!("{}", response.content);
    Ok(())
}
```

---

## 🎪 Demo Guide

Perfect for presenting Specado to others.

### Pre-Demo Checklist

1. Set API key: `export OPENAI_API_KEY=sk-...`
2. Install dependencies: `./setup.sh`
3. Test it works: `./demo.sh all`

### 10-Minute Demo Script

**Introduction (1 min)**

"Specado is spec-first prompting. Write once, run anywhere. Let me show you."

**Show the Spec (2 mins)**

```bash
cat prompts/summarize_article.yaml
```

Key points:
- Clean YAML format
- Provider-agnostic
- Version-controlled
- All features: messages, sampling, response formats

**CLI Demo (2 mins)**

```bash
# Quick and simple
specado ask "What is the capital of France?" --provider openai

# Run the spec
specado run --prompt prompts/summarize_article.yaml --provider openai

# Validation
specado validate --spec prompts/summarize_article.yaml
```

**Python Demo (2 mins)**

```bash
cd python
cat from_spec.py    # Show the code
python from_spec.py # Run it
```

Key points:
- Two lines: `Client("openai")` + `client.complete(prompt)`
- Friendly provider names
- Same spec as CLI

**Node.js Demo (2 mins)**

```bash
cd ../node
cat from_spec.js   # Show the code
npm run from-spec   # Run it
```

Key points:
- Modern ES modules
- Async/await
- Identical experience

**The Power Move (1 min)**

```bash
cd ..
./demo.sh all
```

"Same spec, all languages. Write once, run anywhere."

### Common Questions

**"How do I use a different model?"**

CLI: `--model gpt-4`
Python: `Client("openai", model="gpt-4")`
Node: `new Client("openai", { model: "gpt-4" })`

**"What about Anthropic?"**

Just change `openai` to `anthropic`:
```bash
specado ask "test" --provider anthropic
```

**"Can I use custom providers?"**

Yes! Write a provider YAML spec. The repository bundles a full catalog under the `specado-providers` directory if you want reference implementations.

**"What about JSON responses?"**

See `prompts/generate_function.yaml`:
```yaml
response:
  format: json
```

Specado handles extraction and validation.

### Extended Demo (20 mins)

If you have more time:

1. **Show provider translation:**
   ```bash
   specado preview --prompt prompts/summarize_article.yaml --provider openai
   ```

2. **Show in-code prompts:**
   ```bash
   cd python && cat in_code.py && python in_code.py
   ```

3. **Customize a prompt live:**
   Edit `prompts/summarize_article.yaml`, change temperature, add seed, run again

4. **Show Rust for performance:**
   ```bash
   cd rust_basic && cargo run
   ```

### Closing

"That's Specado. Install, write your spec, go. No fragile scripts, no vendor lock-in."

### Links to Share

- GitHub: [github.com/specado/specado](https://github.com/specado/specado)
- Installation:
  - CLI: `cargo install specado-cli-temp`
  - Python: `pip install specado`
  - Node: `npm install specado`
  - Rust: `specado = "0.2.1"`

---

## 🧪 Testing

Verify everything works before a demo.

### Quick Test

```bash
# Test Python
cd python && python from_spec.py && cd ..

# Test Node
cd node && npm run from-spec && cd ..

# Test validation (no API call)
specado validate --spec prompts/summarize_article.yaml
```

### Full Test

```bash
./demo.sh all
```

### Per-Language Tests

**Python:**
```bash
cd python
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
python from_spec.py
python in_code.py
deactivate
```

**Node.js:**
```bash
cd node
npm install
npm run from-spec
npm run in-code
npm run demo
```

**Rust:**
```bash
cd rust_basic
cargo run
```

**CLI:**
```bash
specado validate --spec prompts/summarize_article.yaml
specado preview --prompt prompts/summarize_article.yaml --provider openai
specado run --prompt prompts/summarize_article.yaml --provider openai
specado ask "What is 2+2?" --provider openai
```

### Validate Prompt Specs

```bash
# Using Specado's helper
python3 -c "from specado import load_prompt; load_prompt('prompts/summarize_article.yaml')"

# Using Specado's validator
specado validate --spec prompts/summarize_article.yaml
specado validate --spec prompts/generate_function.yaml
```

### Error Handling Tests

```bash
# Missing API key
env -u OPENAI_API_KEY specado ask "test" --provider openai

# Invalid provider
python3 -c "from specado import Client; Client('invalid-provider')"

# Non-existent file
specado run --prompt nonexistent.yaml --provider openai
```

---

## 🛠️ Automation Scripts

### setup.sh - Interactive Setup

```bash
./setup.sh
```

Interactive menu to set up:
1. Python examples only (creates venv at `python/.venv`)
2. Node.js examples only
3. Rust examples only
4. All examples

Checks prerequisites, creates Python virtual environment, installs dependencies.

**Python venv details:**
- Location: `python/.venv`
- Automatically git-ignored
- Auto-created if missing when you run `./demo.sh`

### demo.sh - Run Examples

```bash
# Run all examples
./demo.sh all

# Run specific language
./demo.sh python
./demo.sh node
./demo.sh rust
```

Runs examples in sequence with nice output formatting.

---

## 📝 Prompt Specifications

The `prompts/` directory contains reusable specs demonstrating key features:

### summarize_article.yaml

```yaml
version: "1"
messages:
  - role: system
    content: "You are a helpful assistant..."
  - role: user
    content: "Please summarize..."
sampling:
  temperature: 0.3
strict_mode: Warn
```

Shows:
- Multiple message roles
- Sampling configuration
- Basic text response

### generate_function.yaml

```yaml
version: "1"
messages:
  - role: system
    content: "You are a code generation assistant..."
  - role: user
    content: "Generate a Python function..."
response:
  format: json
sampling:
  temperature: 0.2
strict_mode: Warn
```

Shows:
- JSON response format
- Lower temperature for deterministic output
- Code generation use case

---

## 🔧 Troubleshooting

### "Module not found: specado"

**Python:**
```bash
cd python
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

Or just run:
```bash
./setup.sh
```

**Node:**
```bash
npm install
```

### "API key not set"

```bash
export OPENAI_API_KEY=sk-...
```

Or check your key is exported:
```bash
echo $OPENAI_API_KEY
```

### "Command not found: specado"

Install the CLI:
```bash
cargo install specado-cli-temp
```

Or use cargo run from project root:
```bash
cargo run --bin specado-cli -- ask "test" --provider openai
```

### "Network timeout"

Show offline features:
```bash
specado validate --spec prompts/summarize_article.yaml
```

### "Rate limit exceeded"

Have a backup API key, or show validation/preview commands that don't hit APIs.

---

## 💡 Pro Tips

**For Demos:**
- Pre-install all dependencies
- Have backup API key ready
- Use `clear` before each command
- Show code with `cat` before running
- Keep terminal text large

**For Development:**
- Modify prompts in `prompts/` to experiment
- All examples use the same specs
- Changes apply across all languages
- Use `specado preview` to see transformations

**For CI/CD:**
- Use `specado validate` in pipelines
- Version-control your prompt specs
- Set API keys via environment
- Test with `demo.sh all`

---

## 🎯 What This Demonstrates

**For Developers:**
- Simple API (`Client("openai")`)
- Load from file or define in code
- Same behavior across languages

**For Architects:**
- Spec-first design
- Provider-agnostic
- Version-controlled prompts

**For DevOps:**
- Easy CI/CD integration
- Validation before execution
- Audit logging support

---

## 📚 Next Steps

1. **Pick your surface** - Choose Python, Node, Rust, or CLI
2. **Run the examples** - See how simple it is
3. **Modify prompts** - Experiment with different specs
4. **Read the docs** - Check out the [main README](../README.md)

---

## 🙋 Need Help?

- **Issues**: [github.com/specado/specado/issues](https://github.com/specado/specado/issues)
- **Docs**: [../docs/](../docs/)
- **Provider Specs**: See the `specado-providers` crate in this repository for the full catalog.

---

**Happy prompting! 🚀**
