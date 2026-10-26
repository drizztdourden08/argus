/* @layer renderer-app @kind logic */
import type { Worktree } from '../../../../../data/worktrees';

const isYoung = (row: Worktree, limit: number): boolean => {
  const age = row.leftover?.ageDays;
  return age === undefined || age === null || age < limit;
};

export { isYoung };
