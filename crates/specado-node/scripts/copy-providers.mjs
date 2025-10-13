import { cp, mkdir, rm } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const src = path.resolve(__dirname, '../../specado-providers/providers');
const dest = path.resolve(__dirname, '../providers');

async function main() {
  if (!existsSync(src)) {
    throw new Error(`Provider catalog not found at ${src}`);
  }

  await rm(dest, { recursive: true, force: true });
  await mkdir(dest, { recursive: true });
  await cp(src, dest, { recursive: true });
}

main().catch((error) => {
  console.error('Failed to copy provider catalog:', error);
  process.exitCode = 1;
});
