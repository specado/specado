import { rm } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const dest = path.resolve(__dirname, '../providers');

async function main() {
  await rm(dest, { recursive: true, force: true });
}

main().catch((error) => {
  console.error('Failed to clean provider catalog:', error);
  process.exitCode = 1;
});
