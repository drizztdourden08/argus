/* @layer renderer-app @kind logic */
import { nameOf } from './name-of';
import type { WorktreeRow } from './model.type';

const worktreeTitle = (row: WorktreeRow): string => row.branch || nameOf(row);

export { worktreeTitle };
