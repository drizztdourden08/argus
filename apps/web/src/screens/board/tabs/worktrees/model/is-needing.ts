/* @layer renderer-app @kind logic */
import type { Worktree } from '../../../../../data/worktrees';
import { NEED_STATUSES } from './model.constants';

const isNeeding = (row: Worktree): boolean => NEED_STATUSES.includes(row.status);

export { isNeeding };
