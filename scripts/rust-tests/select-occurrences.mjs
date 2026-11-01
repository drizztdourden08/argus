/* @layer tooling-scripts @kind logic */
import { keyOf } from './occurrence-key.mjs';
import { forFile } from './select-for-file.mjs';

const crateOfFile = (crates, file) => crates.find((crate) => file.startsWith(`${crate.dir}/`));

const selectOccurrences = (context, files) => {
  const chosen = new Map();
  for (const file of files) {
    const crate = crateOfFile(context.crates, file);
    if (!crate) continue;
    for (const occurrence of forFile(context, crate, file)) chosen.set(keyOf(occurrence), occurrence);
  }
  return [...chosen.values()];
};

export { selectOccurrences };
