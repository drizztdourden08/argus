/* @layer tooling-scripts @kind logic */
import { filtersOf } from './filters-of.mjs';
import { keyOf } from './occurrence-key.mjs';
import { skipsFor } from './skips-for.mjs';

const tagsFor = { critical: ['critical'], standard: ['critical', 'standard'], e2e: ['e2e'] };
const kindOrder = { lib: 0, bin: 1, test: 2 };

const targetFlags = (target) => {
  if (target.kind === 'lib') return ['--lib'];
  if (target.kind === 'bin') return ['--bin', target.name];
  return ['--test', target.name];
};

const targetKey = (occurrence) => `${occurrence.crate}|${occurrence.target.kind}|${occurrence.target.name}`;

const groupBy = (occurrences) => {
  const groups = new Map();
  for (const occurrence of occurrences) {
    const key = targetKey(occurrence);
    groups.set(key, [...(groups.get(key) ?? []), occurrence]);
  }
  return [...groups.values()];
};

const mergedFilters = (group) => {
  const lists = group.map(filtersOf);
  return lists.includes(null) ? null : [...new Set(lists.flat())].sort();
};

const compare = (a, b) => a[0].crate.localeCompare(b[0].crate)
  || kindOrder[a[0].target.kind] - kindOrder[b[0].target.kind]
  || a[0].target.name.localeCompare(b[0].target.name);

const commandFor = (group, everything, tag) => {
  const { crate, target } = group[0];
  const filters = mergedFilters(group);
  const chosen = new Set(group.map(keyOf));
  const sameTarget = everything.filter((o) => o.hasTests && targetKey(o) === targetKey(group[0]) && !chosen.has(keyOf(o)));
  const skips = skipsFor(filters, sameTarget);
  const tail = [...(filters ?? []), ...skips.flatMap((text) => ['--skip', text]), ...(tag === 'e2e' ? ['--ignored', '--test-threads=1'] : [])];
  const args = ['test', '-p', crate, '-j', '3', ...targetFlags(target), ...(tail.length > 0 ? ['--', ...tail] : [])];
  return { crate, args };
};

const planCommands = (selected, everything, tag) => {
  const wanted = selected.filter((occurrence) => tagsFor[tag].includes(occurrence.info.tags[0]));
  return groupBy(wanted).sort(compare).map((group) => commandFor(group, everything, tag));
};

export { planCommands };
