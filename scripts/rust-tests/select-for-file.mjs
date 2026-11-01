/* @layer tooling-scripts @kind logic */
import { dependentsOf } from './dependents.mjs';
import { usesModule } from './uses-module.mjs';

const specFile = /^src\/(?:.*\/)?(?:specs\/|tests\.rs$)/;
const testsDirectory = /^crates\/[^/]+\/tests\/([^/]+)\//;

const crateTests = (occurrences, name) => occurrences.filter((occurrence) => occurrence.crate === name && occurrence.hasTests);

const sameTarget = (a, b) => a.crate === b.crate && a.target.kind === b.target.kind && a.target.name === b.target.name;

const forMissing = (occurrences, crate, file) => {
  const directory = testsDirectory.exec(file)?.[1];
  const inTarget = crateTests(occurrences, crate.name).filter((o) => o.target.kind === 'test' && o.target.name === directory);
  return inTarget.length > 0 ? inTarget : crateTests(occurrences, crate.name);
};

const forTestPath = (occurrences, crate, file) => {
  const holding = occurrences.filter((occurrence) => occurrence.file === file);
  if (holding.length === 0) return forMissing(occurrences, crate, file);
  if (holding.some((occurrence) => occurrence.hasTests)) return holding.filter((occurrence) => occurrence.hasTests);
  return occurrences.filter((o) => o.hasTests && holding.some((held) => sameTarget(held, o) && usesModule(o, held.modPath)));
};

const forFile = (context, crate, file) => {
  const { crates, occurrences, depth } = context;
  const inside = file.slice(crate.dir.length + 1);
  if (inside === 'Cargo.toml') return crateTests(occurrences, crate.name);
  if (inside.startsWith('tests/') || specFile.test(inside)) return forTestPath(occurrences, crate, file);
  const names = [crate.name, ...dependentsOf(crates, crate.name, depth)];
  return names.flatMap((name) => crateTests(occurrences, name));
};

export { forFile };
