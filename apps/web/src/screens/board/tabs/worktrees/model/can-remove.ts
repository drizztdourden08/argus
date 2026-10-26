/* @layer renderer-app @kind logic */
import type { Worktree } from '../../../../../data/worktrees';
import { isLeftover } from './is-leftover';
import { NOT_REMOVABLE } from './model.constants';

const canRemoveWorktree = (row: Worktree): boolean => !row.main && !row.locked && !NOT_REMOVABLE.includes(row.status);

const canRemoveLeftover = (row: Worktree): boolean => {
  const facts = row.leftover;
  return row.status === 'leftover' && !facts?.pending && !facts?.hasNestedGit && !facts?.isLink;
};

const canRemove = (row: Worktree): boolean => (isLeftover(row) ? canRemoveLeftover(row) : canRemoveWorktree(row));

export { canRemove };
