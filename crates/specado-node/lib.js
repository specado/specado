const { existsSync } = require('node:fs');
const path = require('node:path');

const native = require('./index.js');

const PROVIDER_DIR_CANDIDATES = [
  path.join(__dirname, 'providers'),
  path.join(__dirname, '../specado-providers/providers'),
];

function resolveProvidersDir() {
  return PROVIDER_DIR_CANDIDATES.find((candidate) => existsSync(candidate));
}

class Client extends native.Client {
  constructor(provider, options = null) {
    const normalized = options ? { ...options } : {};
    if (normalized.providersDir === undefined) {
      const detected = resolveProvidersDir();
      if (detected) {
        normalized.providersDir = detected;
      }
    }
    super(provider, normalized);
  }
}

module.exports = {
  ...native,
  Client,
};
