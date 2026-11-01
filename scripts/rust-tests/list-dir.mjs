/* @layer tooling-scripts @kind logic */
import { readdirSync } from 'node:fs';
import { join } from 'node:path';

const listDir = (root, rel) => {
  try {
    return readdirSync(join(root, rel), { withFileTypes: true });
  } catch {
    return [];
  }
};

export { listDir };
