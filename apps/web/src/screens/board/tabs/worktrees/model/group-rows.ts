/* @layer renderer-app @kind logic */
import type { WorktreesMeta } from '../../../../../data/worktrees';
import { NO_THREAD_TITLE } from '../worktrees.constants';
import { STATUS_LABEL } from './model.constants';
import type { RowGroup, WorktreeGroup, WorktreeRow } from './model.type';
import { threadOf } from './thread-of';

const keyOf = (row: WorktreeRow, group: WorktreeGroup, meta: WorktreesMeta | null): string =>
  group === 'thread' ? threadOf(row, meta) : row.status;

const labelOf = (key: string, group: WorktreeGroup): string => {
  if (group === 'thread') return key === '' ? NO_THREAD_TITLE : key;
  return STATUS_LABEL[key] ?? key;
};

const groupRows = (rows: readonly WorktreeRow[], group: WorktreeGroup, meta: WorktreesMeta | null): RowGroup[] => {
  const found = new Map<string, WorktreeRow[]>();
  for (const row of rows) {
    const key = keyOf(row, group, meta);
    found.set(key, [...(found.get(key) ?? []), row]);
  }
  return [...found].map(([key, list]) => ({ key, label: labelOf(key, group), rows: list }));
};

export { groupRows };
