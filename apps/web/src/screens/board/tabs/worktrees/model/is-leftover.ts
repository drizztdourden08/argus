/* @layer renderer-app @kind logic */
import type { Worktree } from '../../../../../data/worktrees';

const isLeftover = (row: Worktree): boolean => row.status === 'leftover' || row.status === 'deleting';

export { isLeftover };
