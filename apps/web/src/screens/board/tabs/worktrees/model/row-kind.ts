/* @layer renderer-app @kind logic */
import { isLeftover } from './is-leftover';
import type { WorktreeRow } from './model.type';

const rowKind = (row: WorktreeRow): string => {
  if (isLeftover(row)) return 'leftover';
  return row.main ? 'main' : 'worktree';
};

export { rowKind };
