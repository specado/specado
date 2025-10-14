#!/usr/bin/env node
/**
 * Specado Node.js Example: Execute a Prompt from a File
 */

const path = require('node:path');
const { Client } = require('specado');

async function main() {
  const client = new Client('openai');

  const promptPath = path.resolve(__dirname, '..', 'prompts', 'summarize_article.yaml');

  console.log('Executing prompt from:', promptPath);
  console.log('-'.repeat(60));

  const response = await client.completeFile(promptPath);

  console.log(JSON.stringify(response, null, 2));
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
