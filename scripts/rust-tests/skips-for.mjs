/* @layer tooling-scripts @kind logic */
import { filtersOf } from './filters-of.mjs';

const leaks = (filters, text) => filters === null || filters.some((filter) => text.includes(filter));

const skipsFor = (filters, unselected) => {
  const own = filters ?? [];
  const found = new Set();
  for (const occurrence of unselected) {
    for (const text of filtersOf(occurrence) ?? []) {
      if (leaks(filters, text) && !own.some((filter) => filter.includes(text))) found.add(text);
    }
  }
  return [...found];
};

export { skipsFor };
