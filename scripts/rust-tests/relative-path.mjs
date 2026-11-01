/* @layer tooling-scripts @kind logic */
import { isAbsolute, relative } from 'node:path';

const relativePath = (root, file) => {
  const inside = isAbsolute(file) ? relative(root, file) : file;
  return inside.replace(/\\/g, '/').replace(/^\.\//, '');
};

export { relativePath };
