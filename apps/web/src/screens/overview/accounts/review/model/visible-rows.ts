/* @layer renderer-app @kind logic */
import type { ReviewRow } from '../../../../../data/types';
import type { ReviewFilter } from '../review.type';

const archivedAnywhere = (row: ReviewRow): boolean => Object.values(row.cells).some((cell) => cell?.present === true && cell.archived === true);

const matches = (row: ReviewRow, filter: ReviewFilter): boolean => {
  if (filter === 'differences') return row.diff !== null;
  return row.kind === 'chat' && archivedAnywhere(row);
};

const keptIds = (rows: readonly ReviewRow[], filter: ReviewFilter): Set<string> => {
  const keep = new Set<string>();
  const above: string[] = [];
  for (const row of rows) {
    above.length = row.depth;
    if (matches(row, filter)) {
      keep.add(row.id);
      for (const id of above) keep.add(id);
    }
    above[row.depth] = row.id;
  }
  return keep;
};

const withoutCollapsed = (rows: readonly ReviewRow[], collapsed: ReadonlySet<string>): ReviewRow[] => {
  const shown: ReviewRow[] = [];
  let hideBelow = Number.POSITIVE_INFINITY;
  for (const row of rows) {
    if (row.depth > hideBelow) continue;
    hideBelow = collapsed.has(row.id) ? row.depth : Number.POSITIVE_INFINITY;
    shown.push(row);
  }
  return shown;
};

const visibleRows = (rows: readonly ReviewRow[], filter: ReviewFilter, collapsed: ReadonlySet<string>): ReviewRow[] => {
  if (filter === 'all') return withoutCollapsed(rows, collapsed);
  const keep = keptIds(rows, filter);
  return withoutCollapsed(rows.filter((row) => keep.has(row.id)), collapsed);
};

export { visibleRows };
