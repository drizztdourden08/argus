/* @layer renderer-app @kind logic */
import { isLeftover } from './is-leftover';
import { isNeeding } from './is-needing';
import type { WorktreeFilter, WorktreeRow } from './model.type';

const filterRows = (rows: readonly WorktreeRow[], filter: WorktreeFilter): WorktreeRow[] => {
  if (filter === 'need') return rows.filter(isNeeding);
  if (filter === 'left') return rows.filter(isLeftover);
  return [...rows];
};

export { filterRows };
