#!/usr/bin/env node
// Specado Node demo showcasing reasoning (OpenAI) and thinking (Anthropic).

import { readFile } from 'node:fs/promises';
import path from 'node:path';
import process from 'node:process';

import specado from '../crates/specado-node/index.js';

const { Client } = specado;

const SCENARIOS = {
  'openai-reasoning': {
    description: 'GPT-5 reasoning controls',
    provider: 'crates/specado-providers/providers/openai/gpt-5/base.yaml',
    prompt: 'examples/prompts/openai_reasoning.json',
    apiKey: 'OPENAI_API_KEY',
  },
  'anthropic-thinking': {
    description: 'Claude Sonnet thinking mode',
    provider: 'crates/specado-providers/providers/anthropic/claude-4.5/sonnet.yaml',
    prompt: 'examples/prompts/anthropic_thinking.json',
    apiKey: 'ANTHROPIC_API_KEY',
  },
};

function printUsage() {
  console.log(`Usage: node examples/node_basic.mjs [options]

Options:
  --scenario <name>   Demo to run (default: openai-reasoning)
  --provider <path>   Provider spec (override scenario default)
  --prompt <path>     Prompt spec JSON/YAML (override scenario default)
  --watch             Enable experimental watch plumbing
  --audit             Forward audit logs to stdout
  --redact <value>    Additional audit redaction pattern (repeatable)
  --help              Show this help message
`);
}

function parseArgs(argv) {
  const options = {
    scenario: 'openai-reasoning',
    provider: undefined,
    prompt: undefined,
    watch: false,
    audit: false,
    redact: [],
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    switch (arg) {
      case '--scenario':
        options.scenario = argv[++i] ?? options.scenario;
        break;
      case '--provider':
        options.provider = argv[++i] ?? options.provider;
        break;
      case '--prompt':
        options.prompt = argv[++i] ?? options.prompt;
        break;
      case '--watch':
        options.watch = true;
        break;
      case '--audit':
        options.audit = true;
        break;
      case '--redact':
        options.redact.push(argv[++i] ?? '');
        break;
      case '--help':
        printUsage();
        process.exit(0);
        break;
      default:
        console.error(`Unknown option: ${arg}`);
        printUsage();
        process.exit(1);
    }
  }

  return options;
}

async function loadPrompt(filePath) {
  const absolute = path.resolve(filePath);
  const contents = await readFile(absolute, 'utf8');
  const ext = path.extname(absolute).toLowerCase();

  if (ext === '.yaml' || ext === '.yml') {
    try {
      const module = await import('js-yaml');
      const yaml = module.default ?? module;
      return yaml.load(contents);
    } catch (error) {
      throw new Error(
        'Install js-yaml (npm install js-yaml) to consume YAML prompts or provide JSON instead.',
        { cause: error },
      );
    }
  }

  return JSON.parse(contents);
}

function warnIfNoApiKey(varName) {
  if (!varName) return;
  if (!process.env[varName]) {
    console.warn(`warning: ${varName} is not set. The provider call will likely fail.`);
  }
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const scenarioName = SCENARIOS[args.scenario] ? args.scenario : 'openai-reasoning';
  if (scenarioName !== args.scenario) {
    console.warn(`Unknown scenario '${args.scenario}', defaulting to openai-reasoning.`);
  }
  const scenario = SCENARIOS[scenarioName];

  const providerPath = args.provider ?? scenario.provider;
  const promptPath = args.prompt ?? scenario.prompt;
  const promptPayload = await loadPrompt(promptPath);

  warnIfNoApiKey(scenario.apiKey);

  const clientOptions = {};
  if (args.watch) {
    clientOptions.watch = { enable: true };
  }
  if (args.audit) {
    clientOptions.audit = { target: 'stdout', redact: args.redact.filter(Boolean) };
  }

  const client = new Client(providerPath, Object.keys(clientOptions).length ? clientOptions : undefined);
  const response = await client.complete(promptPayload);
  console.log(JSON.stringify(response, null, 2));

  console.error(
    `\nCompleted scenario '${scenarioName}' (${scenario.description}) using provider ${providerPath} and prompt ${promptPath}`,
  );
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
