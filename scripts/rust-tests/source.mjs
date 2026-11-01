/* @layer tooling-scripts @kind logic */
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const cache = new Map();

const loadText = (full) => {
  try {
    return readFileSync(full, 'utf8');
  } catch {
    return null;
  }
};

const readSource = (root, rel) => {
  const key = `${root}|${rel}`;
  if (!cache.has(key)) cache.set(key, loadText(join(root, rel)));
  return cache.get(key);
};

export { readSource };
