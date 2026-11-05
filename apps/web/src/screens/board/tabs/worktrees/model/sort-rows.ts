/* @layer renderer-app @kind logic */
import type { SortDirection } from '@drizztdourden08/tessera';
import { compareRows } from './compare-rows';
import type { WorktreeOrder, WorktreeRow } from './model.type';

type Compare = (a: WorktreeRow, b: WorktreeRow) => number;

const ASCENDING: Readonly<Record<WorktreeOrder, Compare>> = {
  need: (a, b) => compareRows(b, a),
  age: (a, b) => (a.ageDays ?? 0) - (b.ageDays ?? 0),
  path: (a, b) => a.path.localeCompare(b.path),
  size: (a, b) => (a.leftover?.sizeMB ?? 0) - (b.leftover?.sizeMB ?? 0),
};

const sortRows = (rows: readonly WorktreeRow[], order: WorktreeOrder, direction: SortDirection): WorktreeRow[] => {
  const compare = ASCENDING[order];
  return [...rows].sort(direction === 'asc' ? compare : (a, b) => compare(b, a));
};

export { sortRows };
