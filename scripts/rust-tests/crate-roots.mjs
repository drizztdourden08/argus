/* @layer tooling-scripts @kind logic */
import { listDir } from './list-dir.mjs';
import { readSource } from './source.mjs';

const rootsIn = (root, dir, kind) => {
  const roots = [];
  for (const entry of listDir(root, dir)) {
    if (entry.isFile() && entry.name.endsWith('.rs')) {
      roots.push({ kind, name: entry.name.slice(0, -3), file: `${dir}/${entry.name}` });
    } else if (entry.isDirectory() && readSource(root, `${dir}/${entry.name}/main.rs`) !== null) {
      roots.push({ kind, name: entry.name, file: `${dir}/${entry.name}/main.rs` });
    }
  }
  return roots;
};

const crateRoots = (root, crate) => {
  const src = `${crate.dir}/src`;
  const roots = [...rootsIn(root, `${src}/bin`, 'bin'), ...rootsIn(root, `${crate.dir}/tests`, 'test')];
  if (readSource(root, `${src}/lib.rs`) !== null) roots.push({ kind: 'lib', name: crate.name, file: `${src}/lib.rs` });
  if (readSource(root, `${src}/main.rs`) !== null) roots.push({ kind: 'bin', name: crate.name, file: `${src}/main.rs` });
  return roots;
};

export { crateRoots };
