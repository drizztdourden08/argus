/* @layer renderer-app @kind logic */
import type { SortDirection } from '@drizztdourden08/tessera';
import { compareRows } from './compare-rows';
import type { WorktreeOrder, WorktreeRow } from './model.type';
import type { RowCompare } from './row-compare.type';

const ascending = (order: WorktreeOrder): RowCompare => {
  if (order === 'need') return (a, b) => compareRows(b, a);
  if (order === 'age') return (a, b) => (a.ageDays ?? 0) - (b.ageDays ?? 0);
  if (order === 'path') return (a, b) => a.path.localeCompare(b.path);
  return (a, b) => (a.leftover?.sizeMB ?? 0) - (b.leftover?.sizeMB ?? 0);
};

const sortRows = (rows: readonly WorktreeRow[], order: WorktreeOrder, direction: SortDirection): WorktreeRow[] => {
  const compare = ascending(order);
  return [...rows].sort(direction === 'asc' ? compare : (a, b) => compare(b, a));
};

export { sortRows };
