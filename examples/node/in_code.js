#!/usr/bin/env node
/**
 * Specado Node.js Example: Define Prompts in Code
 */

const { Client } = require('specado');

async function main() {
  const client = new Client('openai', { model: 'gpt-5' });

  console.log('Executing in-code prompt specification...');
  console.log('-'.repeat(60));

  const response = await client.completeText(
    'Explain what a closure is in JavaScript in one paragraph.',
    {
      system: 'You are a helpful assistant that explains programming concepts clearly.',
      temperature: 0.5,
    }
  );

  console.log(JSON.stringify(response, null, 2));
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
