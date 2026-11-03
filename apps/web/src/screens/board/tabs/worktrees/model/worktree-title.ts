/* @layer renderer-app @kind logic */
import { nonEmpty } from '../../../shared';
import { nameOf } from './name-of';
import type { WorktreeRow } from './model.type';

const worktreeTitle = (row: WorktreeRow): string => nonEmpty(row.branch, nameOf(row));

export { worktreeTitle };
