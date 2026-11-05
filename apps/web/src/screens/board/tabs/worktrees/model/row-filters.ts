/* @layer renderer-app @kind logic */
import type { WorktreesMeta } from '../../../../../data/worktrees';
import type { WorktreeFilters, WorktreeRow } from './model.type';
import { rowKind } from './row-kind';
import { threadOf } from './thread-of';

const allows = (list: readonly string[], value: string): boolean => list.length === 0 || list.includes(value);

const hasText = (row: WorktreeRow, query: string): boolean => {
  const needle = query.trim().toLowerCase();
  return needle === '' || [row.path, row.branch].some((part) => (part ?? '').toLowerCase().includes(needle));
};

const applyRowFilters = (rows: readonly WorktreeRow[], filters: WorktreeFilters, meta: WorktreesMeta | null): WorktreeRow[] =>
  rows.filter((row) => (
    hasText(row, filters.query)
    && allows(filters.threads, threadOf(row, meta))
    && allows(filters.statuses, row.status)
    && allows(filters.kinds, rowKind(row))
  ));

export { applyRowFilters };
