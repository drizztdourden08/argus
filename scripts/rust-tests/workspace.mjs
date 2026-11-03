/* @layer tooling-scripts @kind logic */
import { listDir } from './list-dir.mjs';
import { readSource } from './source.mjs';

const dependencySection = /^\[(?:target\.[^\]]+\.)?(?:dev-|build-)?dependencies(?:\.([\w-]+))?\]/;
const dependencyKey = /^([\w-]+)\s*(?:=|\.)/;
const packageName = /^name\s*=\s*"([^"]+)"/m;

const dependencyNames = (manifest) => {
  const names = [];
  let inDependencies = false;
  for (const line of manifest.split(/\r?\n/)) {
    if (line.startsWith('[')) {
      const section = dependencySection.exec(line);
      inDependencies = Boolean(section);
      if (section?.[1]) names.push(section[1]);
    } else if (inDependencies) {
      const key = dependencyKey.exec(line);
      if (key) names.push(key[1]);
    }
  }
  return names;
};

const readCrate = (root, dir) => {
  const manifest = readSource(root, `${dir}/Cargo.toml`);
  const name = manifest ? packageName.exec(manifest)?.[1] : null;
  return name ? { name, dir, deps: dependencyNames(manifest) } : null;
};

const loadWorkspace = (root) => {
  const dirs = listDir(root, 'crates').filter((entry) => entry.isDirectory());
  const crates = dirs.map((entry) => readCrate(root, `crates/${entry.name}`)).filter(Boolean);
  const names = new Set(crates.map((crate) => crate.name));
  const linked = crates.map((crate) => ({ ...crate, deps: crate.deps.filter((dep) => names.has(dep) && dep !== crate.name) }));
  return linked.sort((a, b) => a.name.localeCompare(b.name));
};

export { loadWorkspace };
