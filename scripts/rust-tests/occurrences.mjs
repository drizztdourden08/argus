/* @layer tooling-scripts @kind logic */
import { crateRoots } from './crate-roots.mjs';
import { inlineTestMods } from './inline-mods.mjs';
import { readSource } from './source.mjs';
import { readTags } from './tags.mjs';
import { walkTree } from './walk-tree.mjs';

const occurrenceOf = (root, crate, target, entry) => {
  const text = readSource(root, entry.file) ?? '';
  const info = readTags(text);
  return {
    crate: crate.name,
    target: { kind: target.kind, name: target.name },
    file: entry.file,
    modPath: entry.modPath,
    info,
    hasTests: info.tests > 0,
    inline: inlineTestMods(text),
    text,
  };
};

const listOccurrences = (root, crates) => crates.flatMap((crate) => crateRoots(root, crate)
  .flatMap((target) => walkTree(root, target.file).map((entry) => occurrenceOf(root, crate, target, entry))));

export { listOccurrences };
