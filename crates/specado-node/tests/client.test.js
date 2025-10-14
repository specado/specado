const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const http = require('node:http');
const { Client, createPrompt, loadPrompt, simplePrompt } = require('..');

function createTempDir(prefix) {
  return fs.mkdtempSync(path.join(os.tmpdir(), prefix));
}

async function withTempDir(prefix, run) {
  const dir = createTempDir(prefix);
  try {
    return await run(dir);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

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

function writeOfflineProviderSpec(dir) {
  const providerPath = path.join(dir, 'provider.yaml');
  fs.writeFileSync(
    providerPath,
    `provider: noop
models:
  - id: noop
auth:
  type: none
endpoints:
  chat:
    method: POST
    url: http://127.0.0.1:65535/
`,
    'utf8'
  );
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

  try {
    await withTempDir('specado-node-', async (tmpDir) => {
      const providerPath = writeProviderSpec(tmpDir, url, tokenEnv);
      const client = new Client(providerPath);
      const prompt = {
        version: '1',
        messages: [{ role: 'user', content: 'Hello!' }]
      };

      const response = await client.complete(prompt);
      assert.equal(response.content, 'hello from node');
      assert.equal(response.finish_reason, 'stop');
    });
  } finally {
    delete process.env[tokenEnv];
    await new Promise((resolve) => server.close(resolve));
  }
});

test('loadPrompt loads YAML files', async () => {
  await withTempDir('specado-node-prompt-', async (dir) => {
    const promptPath = path.join(dir, 'example.yaml');
    fs.writeFileSync(
      promptPath,
      `version: "1"
messages:
  - role: user
    content: Hello from YAML
`,
      'utf8'
    );

    const loaded = loadPrompt(promptPath);
    assert.equal(loaded.messages[0].content, 'Hello from YAML');
  });
});

test('loadPrompt loads JSON files', async () => {
  await withTempDir('specado-node-prompt-', async (dir) => {
    const promptPath = path.join(dir, 'example.json');
    fs.writeFileSync(
      promptPath,
      JSON.stringify({
        version: '1',
        messages: [{ role: 'user', content: 'Hello from JSON' }]
      }),
      'utf8'
    );

    const loaded = loadPrompt(promptPath);
    assert.equal(loaded.messages[0].content, 'Hello from JSON');
  });
});

test('createPrompt returns a versioned payload with optional fields', () => {
  const prompt = createPrompt({
    messages: [{ role: 'user', content: 'Hi there' }],
    sampling: { temperature: 0.2 },
    metadata: { topic: 'demo' }
  });

  assert.equal(prompt.version, '1');
  assert.equal(prompt.sampling.temperature, 0.2);
  assert.equal(prompt.metadata.topic, 'demo');
});

test('simplePrompt builds spec from convenience arguments', () => {
  const prompt = simplePrompt({
    message: 'Hi from helper',
    system: 'Keep it concise',
    temperature: 0.1
  });

  assert.equal(prompt.messages.length, 2);
  assert.equal(prompt.messages[0].role, 'system');
  assert.equal(prompt.messages[1].content, 'Hi from helper');
  assert.equal(prompt.sampling.temperature, 0.1);
});

test('Client.completeFile rejects on network error', async () => {
  await withTempDir('specado-node-prompt-', async (dir) => {
    const promptPath = path.join(dir, 'example.yaml');
    fs.writeFileSync(
      promptPath,
      `version: "1"
messages:
  - role: user
    content: Hello
`,
      'utf8'
    );

    const providerPath = writeOfflineProviderSpec(dir);
    const client = new Client(providerPath);
    await assert.rejects(() => client.completeFile(promptPath));
  });
});

test('Client.completeText rejects on network error', async () => {
  await withTempDir('specado-node-provider-', async (dir) => {
    const providerPath = writeOfflineProviderSpec(dir);
    const client = new Client(providerPath);
    await assert.rejects(() => client.completeText('hi there'));
  });
});
