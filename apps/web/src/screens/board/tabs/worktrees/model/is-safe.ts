/* @layer renderer-app @kind logic */
import type { Worktree } from '../../../../../data/worktrees';

const keepsCommits = (row: Worktree): boolean =>
  row.referenced === true || (Boolean(row.branch) && !row.detached) || Boolean(row.merged) || row.unmerged === 0;

const isSafe = (row: Worktree): boolean => !row.dirty && keepsCommits(row);

export { isSafe };
