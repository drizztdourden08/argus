/* @layer tooling-scripts @kind logic */
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { cli } from './rust-tests/cli.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');

try {
  process.exitCode = await cli(root, process.argv.slice(2));
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 2;
}
