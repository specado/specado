#!/usr/bin/env node
// Minimal Specado Node example for Issue #50.

import { readFile } from 'node:fs/promises';
import path from 'node:path';
import process from 'node:process';

import specado from '../crates/specado-node/index.js';

const { Client } = specado;

function printUsage() {
  console.log(`Usage: node examples/node_basic.mjs [options]

Options:
  --provider <path>   Provider spec (default: crates/specado-providers/providers/openai/gpt-5.yaml)
  --prompt <path>     Prompt spec JSON/YAML (default: examples/prompts/basic_chat.json)
  --watch             Enable experimental watch plumbing
  --audit             Forward audit logs to stdout
  --redact <value>    Additional audit redaction pattern (repeatable)
  --help              Show this help message
`);
}

function parseArgs(argv) {
  const options = {
    provider: 'crates/specado-providers/providers/openai/gpt-5.yaml',
    prompt: 'examples/prompts/basic_chat.json',
    watch: false,
    audit: false,
    redact: [],
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    switch (arg) {
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

function warnIfNoApiKey() {
  if (!process.env.OPENAI_API_KEY) {
    console.warn('warning: OPENAI_API_KEY is not set. The provider call will likely fail.');
  }
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const promptPayload = await loadPrompt(args.prompt);

  warnIfNoApiKey();

  const clientOptions = {};
  if (args.watch) {
    clientOptions.watch = { enable: true };
  }
  if (args.audit) {
    clientOptions.audit = { target: 'stdout', redact: args.redact.filter(Boolean) };
  }

  const client = new Client(args.provider, Object.keys(clientOptions).length ? clientOptions : undefined);
  const response = await client.complete(promptPayload);
  console.log(JSON.stringify(response, null, 2));
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
