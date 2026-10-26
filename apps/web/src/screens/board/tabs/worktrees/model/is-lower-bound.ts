/* @layer renderer-app @kind logic */
import type { Worktree } from '../../../../../data/worktrees';

const isLowerBound = (row: Worktree): boolean => row.leftover?.partial === true || row.leftover?.nodeModules === true;

export { isLowerBound };
