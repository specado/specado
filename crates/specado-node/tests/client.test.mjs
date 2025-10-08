import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import http from 'node:http';
import { Client } from '../index.js';

function writeProviderSpec(dir, url, tokenEnv) {
  const yaml = `provider: test
models:
  - id: test-model
auth:
  type: bearer
  token_env: ${tokenEnv}
endpoints:
  chat:
    method: POST
    url: ${url}
    headers:
      content-type: application/json
mappings:
  request:
    - from: $.messages
      to: $.body.messages
  response:
    - from: $.data.content
      to: content
    - from: $.data.finish_reason
      to: finish_reason
constraints:
  supports:
    json_mode: true
    tools: true
`;
  const providerPath = path.join(dir, 'provider.yaml');
  fs.writeFileSync(providerPath, yaml, 'utf8');
  return providerPath;
}

test('Client.complete posts to provider and normalizes response', async () => {
  const tokenEnv = 'SPECADO_NODE_TOKEN';
  process.env[tokenEnv] = 'node-secret';

  const server = http.createServer((req, res) => {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(
      JSON.stringify({
        data: {
          content: 'hello from node',
          finish_reason: 'stop'
        }
      })
    );
  });

  await new Promise((resolve) => server.listen(0, resolve));
  const { port } = server.address();
  const url = `http://127.0.0.1:${port}/chat`;

  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'specado-node-'));
  const providerPath = writeProviderSpec(tmpDir, url, tokenEnv);

  const client = new Client(providerPath);
  const prompt = {
    version: '1',
    messages: [
      { role: 'user', content: 'Hello!' }
    ]
  };

  try {
    const response = await client.complete(prompt);
    assert.equal(response.content, 'hello from node');
    assert.equal(response.finish_reason, 'stop');
  } finally {
    delete process.env[tokenEnv];
    await new Promise((resolve) => server.close(resolve));
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
});
