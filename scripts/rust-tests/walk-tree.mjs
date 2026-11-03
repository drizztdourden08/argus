/* @layer tooling-scripts @kind logic */
import { posix } from 'node:path';
import { readDeclarations } from './declarations.mjs';
import { readSource } from './source.mjs';

const resolveChild = (root, file, ownsDir, declaration) => {
  const dir = posix.dirname(file);
  if (declaration.path) {
    const target = posix.join(dir, declaration.path);
    return readSource(root, target) === null ? null : { file: target, ownsDir: true };
  }
  const base = ownsDir ? dir : posix.join(dir, posix.basename(file, '.rs'));
  const options = [`${base}/${declaration.name}.rs`, `${base}/${declaration.name}/mod.rs`];
  const hit = options.find((candidate) => readSource(root, candidate) !== null);
  return hit ? { file: hit, ownsDir: hit.endsWith('/mod.rs') } : null;
};

const walkTree = (root, rootFile) => {
  const found = [];
  const visit = (file, modPath, ownsDir) => {
    found.push({ file, modPath });
    const text = readSource(root, file);
    if (text === null) return;
    for (const declaration of readDeclarations(text)) {
      const child = resolveChild(root, file, ownsDir, declaration);
      if (child) visit(child.file, modPath ? `${modPath}::${declaration.name}` : declaration.name, child.ownsDir);
    }
  };
  visit(rootFile, '', true);
  return found;
};

export { walkTree };
